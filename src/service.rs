//! Turns a ready batch of settled files into moved, unchanged or refused files.

use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use crate::config::{FOLDER_CONFIG, FolderConfig, MAX_FOLDER_CONFIG_BYTES, Watch};
use crate::context::Target;
use crate::effects::{Applied, Done, Roots, SkipReason, apply_effects};
use crate::engine::{Verdict, plan_batch};
use crate::record::Record;
use crate::stages::Effect;
use crate::tree::{Kind, Tree};

/// The target folder as the filesystem holds it.
pub struct FsTarget {
    root: PathBuf,
}

impl FsTarget {
    /// A target over the folder at the root.
    pub fn new(root: PathBuf) -> FsTarget {
        FsTarget { root }
    }
}

impl Target for FsTarget {
    fn files_in(&self, folder: &Path) -> Vec<String> {
        let Ok(entries) = fs::read_dir(self.root.join(folder)) else {
            return Vec::new();
        };
        entries
            .flatten()
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect()
    }
}

/// The tree the filesystem holds.
pub struct FsTree;

impl Tree for FsTree {
    fn kind(&self, path: &Path) -> io::Result<Kind> {
        let kind = fs::symlink_metadata(path)?.file_type();
        Ok(if kind.is_symlink() {
            Kind::Link
        } else if kind.is_file() {
            Kind::File
        } else if kind.is_dir() {
            Kind::Folder
        } else {
            Kind::Other
        })
    }

    fn modified(&self, path: &Path) -> io::Result<DateTime<Utc>> {
        Ok(DateTime::from(fs::symlink_metadata(path)?.modified()?))
    }

    fn read(&self, path: &Path, limit: u64) -> io::Result<String> {
        let mut text = String::new();
        fs::File::open(path)?
            .take(limit)
            .read_to_string(&mut text)?;
        Ok(text)
    }
}

/// What became of one file of a batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum What {
    Moved(PathBuf),
    /// Moved, and then a later effect of the pipeline failed.
    MovedThenFailed {
        to: PathBuf,
        reason: String,
    },
    Previewed(PathBuf),
    Unchanged,
    Skipped(SkipReason),
    Refused(String),
    Unclaimed,
    Excluded,
    LeftTooLarge,
}

/// One file of a processed batch and what became of it. A folder configuration that could not be
/// used is reported the same way, under its own path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Processed {
    pub origin: PathBuf,
    pub what: What,
}

/// The most times a file may be renamed in place in a row. A pipeline that names its own result again
/// would otherwise rename it for ever, since every rename is seen as a file that arrived.
pub const MAX_IN_PLACE_RENAMES: u32 = 5;

/// The most files whose renames in place are remembered, so the record stays bounded however many
/// files are renamed.
const MAX_REMEMBERED_RENAMES: usize = 100_000;

/// How many times each file was renamed in place in a row, kept while the program runs. A file the
/// pipeline leaves as it is has been settled by it and is forgotten.
#[derive(Debug, Default)]
pub struct Renames {
    counts: HashMap<PathBuf, u32>,
}

impl Renames {
    /// A record with no renames in it.
    pub fn new() -> Renames {
        Renames::default()
    }

    fn count(&self, file: &Path) -> u32 {
        self.counts.get(file).copied().unwrap_or(0)
    }

    fn forget(&mut self, file: &Path) {
        self.counts.remove(file);
    }

    fn renamed(&mut self, from: &Path, to: PathBuf, count: u32) {
        self.counts.remove(from);
        if self.counts.len() < MAX_REMEMBERED_RENAMES {
            self.counts.insert(to, count);
        }
    }
}

/// Plans a ready batch through the pipelines of its watch, with the folder configurations that apply,
/// and runs the effects on what was planned. Paths are relative to the source.
pub fn process_batch(
    tree: &dyn Tree,
    watch: &Watch,
    unit: &Path,
    files: &[PathBuf],
    renames: &mut Renames,
) -> Vec<Processed> {
    let mut processed = Vec::new();
    let layers = folder_configs(tree, &watch.source, unit, &mut processed);
    let effective = watch.under(&layers);

    if files.len() > effective.batch_max {
        processed.extend(files.iter().map(|origin| Processed {
            origin: origin.clone(),
            what: What::LeftTooLarge,
        }));
        return processed;
    }

    let target_root = effective
        .target
        .clone()
        .unwrap_or_else(|| effective.source.clone());
    let roots = Roots {
        source: effective.source.clone(),
        target: target_root.clone(),
    };
    let mut records = Vec::new();
    for origin in files {
        match read_record(tree, &effective, origin) {
            Ok(record) => records.push(record),
            Err(what) => processed.push(Processed {
                origin: origin.clone(),
                what,
            }),
        }
    }

    let pipelines = effective.pipelines();
    let target = FsTarget::new(target_root);
    let judged = plan_batch(&pipelines, records, &target);
    for entry in judged {
        let what = match entry.verdict {
            Verdict::Planned(record) => {
                apply_planned(&entry.effects, &record, unit, &roots, &effective, renames)
            }
            Verdict::Unclaimed => What::Unclaimed,
            Verdict::Excluded => What::Excluded,
            Verdict::Rejected(rejection) => {
                What::Refused(format!("{}: {}", rejection.stage, rejection.reason))
            }
        };
        processed.push(Processed {
            origin: entry.origin,
            what,
        });
    }
    processed
}

/// Runs the effects of a planned file, unless it has been renamed in place too many times in a row,
/// and keeps count of its renames in place.
fn apply_planned(
    effects: &[Effect],
    record: &Record,
    unit: &Path,
    roots: &Roots,
    watch: &Watch,
    renames: &mut Renames,
) -> What {
    let in_place = watch.target.is_none();
    let origin = record.origin();
    let count = renames.count(origin);
    if in_place && record.plan() != origin && count >= MAX_IN_PLACE_RENAMES {
        return What::Refused(format!(
            "renamed in place {count} times in a row; the pipeline may name its own result again"
        ));
    }
    let what = effects_of(effects, record, unit, roots, watch.dry_run);
    if in_place {
        match &what {
            What::Moved(to) | What::MovedThenFailed { to, .. } => {
                if let Ok(relative) = to.strip_prefix(&watch.source) {
                    renames.renamed(origin, relative.to_path_buf(), count + 1);
                }
            }
            What::Unchanged => renames.forget(origin),
            _ => {}
        }
    }
    what
}

/// The record of a file that can be handled: a regular file with a modification time and a path that is text.
fn read_record(tree: &dyn Tree, watch: &Watch, origin: &Path) -> Result<Record, What> {
    let path = watch.source.join(origin);
    match tree.kind(&path) {
        Err(_) => return Err(What::Skipped(SkipReason::Missing)),
        Ok(Kind::Link) => return Err(What::Skipped(SkipReason::Link)),
        Ok(Kind::File) => {}
        Ok(_) => return Err(What::Skipped(SkipReason::NotAFile)),
    }
    let modified = tree
        .modified(&path)
        .map_err(|_| What::Skipped(SkipReason::NoModificationTime))?;
    let record =
        Record::new(origin, modified).map_err(|error| What::Refused(format!("name: {error}")))?;
    Ok(record.with_vars(watch.vars.clone()))
}

fn effects_of(
    effects: &[Effect],
    record: &Record,
    unit: &Path,
    roots: &Roots,
    dry_run: bool,
) -> What {
    // A pipeline with no effect stage only ever previews.
    if effects.is_empty() {
        return What::Previewed(roots.target.join(record.plan()));
    }
    let run = apply_effects(effects, record, unit, roots, dry_run);
    let moved = run.done.into_iter().find_map(|done| match done {
        Done::Moved(applied) => Some(applied),
        Done::Cleaned(_) => None,
    });
    if let Some(error) = run.error {
        return match moved {
            Some(Applied::Moved { to, .. }) => What::MovedThenFailed {
                to,
                reason: error.to_string(),
            },
            _ => What::Refused(error.to_string()),
        };
    }
    match moved {
        Some(Applied::Moved { to, .. }) => What::Moved(to),
        Some(Applied::Preview { to, .. }) => What::Previewed(to),
        Some(Applied::Skipped(reason)) => What::Skipped(reason),
        Some(Applied::Unchanged(at)) if dry_run => What::Previewed(at),
        Some(Applied::Unchanged(_)) | None => What::Unchanged,
    }
}

/// The folder configurations from the source down to the unit, from the farthest to the nearest.
/// One that cannot be read or used is left out and reported.
fn folder_configs(
    tree: &dyn Tree,
    source: &Path,
    unit: &Path,
    processed: &mut Vec<Processed>,
) -> Vec<FolderConfig> {
    let mut folder = PathBuf::new();
    let mut layers = vec![folder.clone()];
    for component in unit.components() {
        folder.push(component);
        layers.push(folder.clone());
    }

    let mut configs = Vec::new();
    for layer in layers {
        let relative = layer.join(FOLDER_CONFIG);
        let Some(text) = read_folder_config(tree, &source.join(&relative)) else {
            continue;
        };
        match text
            .map_err(|error| error.to_string())
            .and_then(|text| FolderConfig::parse(&text).map_err(|error| error.to_string()))
        {
            Ok(config) => configs.push(config),
            Err(reason) => processed.push(Processed {
                origin: relative,
                what: What::Refused(reason),
            }),
        }
    }
    configs
}

/// The text of a folder configuration, or none when the folder has none. Only a regular file is read,
/// and never more than one byte past the size a folder configuration may have.
fn read_folder_config(tree: &dyn Tree, path: &Path) -> Option<io::Result<String>> {
    if tree.kind(path).ok()? != Kind::File {
        return None;
    }
    Some(tree.read(path, MAX_FOLDER_CONFIG_BYTES as u64 + 1))
}

/// Logs what became of one file of a batch.
pub fn report(entry: &Processed) {
    let origin = entry.origin.display();
    match &entry.what {
        What::Moved(to) => eprintln!("[info] {origin} -> {}", to.display()),
        What::MovedThenFailed { to, reason } => {
            eprintln!("[warn] {origin} -> {}, then failed: {reason}", to.display())
        }
        What::Previewed(to) => eprintln!("[info] {origin} would go to {}", to.display()),
        What::Unchanged => {}
        What::Unclaimed => eprintln!("[info] {origin} left: no pipeline claims it"),
        What::Excluded => eprintln!("[info] {origin} left: a filter excluded it"),
        What::Skipped(reason) => eprintln!("[warn] {origin} skipped: {reason}"),
        What::Refused(reason) => eprintln!("[warn] {origin} refused: {reason}"),
        What::LeftTooLarge => eprintln!("[warn] {origin} left: the batch is too large"),
    }
}
