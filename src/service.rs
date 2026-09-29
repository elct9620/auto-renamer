//! Turns a ready batch of settled files into moved, unchanged or refused files.

use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use crate::config::{FolderConfig, MAX_BATCH_FILES, MAX_FOLDER_CONFIG_BYTES, Watch};
use crate::context::{Context, Target};
use crate::effects::{Applied, Done, Roots, SkipReason, apply_effects};
use crate::engine::{Verdict, plan_batch};
use crate::record::Record;

/// The name of a folder configuration.
const FOLDER_CONFIG: &str = "auto-renamer.toml";

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

/// What became of one file of a batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum What {
    Moved(PathBuf),
    Previewed(PathBuf),
    Unchanged,
    Skipped(String),
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
        // Past the ceiling a batch may hold the record stops growing, so its memory stays bounded.
        if self.counts.len() < MAX_BATCH_FILES {
            self.counts.insert(to, count);
        }
    }
}

/// Plans a ready batch through the pipelines of its watch, with the folder configurations that apply,
/// and runs the effects on what was planned. Paths are relative to the source.
pub fn process_batch(
    watch: &Watch,
    unit: &Path,
    files: &[PathBuf],
    renames: &mut Renames,
) -> Vec<Processed> {
    let mut processed = Vec::new();
    let layers = folder_configs(&watch.source, unit, &mut processed);
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
        match read_record(&effective, origin) {
            Ok(record) => records.push(record),
            Err(reason) => processed.push(Processed {
                origin: origin.clone(),
                what: What::Skipped(reason.to_string()),
            }),
        }
    }

    let pipelines = effective.pipelines();
    let target = FsTarget::new(target_root);
    let judged = plan_batch(&pipelines, records, &mut Context::new(&target));
    for entry in judged {
        let what = match entry.verdict {
            Verdict::Planned(record) => {
                let pipeline = pipelines
                    .iter()
                    .find(|(name, _)| Some(name) == entry.pipeline.as_ref())
                    .map(|(_, pipeline)| pipeline);
                let in_place = effective.target.is_none();
                let count = renames.count(&entry.origin);
                match pipeline {
                    Some(_)
                        if in_place
                            && record.plan() != record.origin()
                            && count >= MAX_IN_PLACE_RENAMES =>
                    {
                        What::Refused(format!(
                            "renamed in place {count} times in a row; the pipeline may name its own result again"
                        ))
                    }
                    Some(pipeline) => {
                        let what = effects_of(pipeline, &record, unit, &roots, effective.dry_run);
                        if in_place {
                            match &what {
                                What::Moved(to) => {
                                    if let Ok(relative) = to.strip_prefix(&effective.source) {
                                        renames.renamed(
                                            &entry.origin,
                                            relative.to_path_buf(),
                                            count + 1,
                                        );
                                    }
                                }
                                What::Unchanged => renames.forget(&entry.origin),
                                _ => {}
                            }
                        }
                        what
                    }
                    None => What::Unclaimed,
                }
            }
            Verdict::Excluded => What::Excluded,
            Verdict::Unclaimed => What::Unclaimed,
            Verdict::Rejected(rejection) => {
                What::Refused(format!("{}: {}", rejection.stage, rejection.reason))
            }
        };
        processed.push(Processed {
            origin: entry.origin,
            what,
        });
    }
    processed.iter().for_each(report);
    processed
}

fn read_record(watch: &Watch, origin: &Path) -> Result<Record, &'static str> {
    let metadata = fs::symlink_metadata(watch.source.join(origin)).map_err(|_| "missing")?;
    if metadata.file_type().is_symlink() {
        return Err("link");
    }
    if !metadata.is_file() {
        return Err("not a file");
    }
    let modified = metadata.modified().map_err(|_| "no modification time")?;
    Ok(Record::new(origin, DateTime::<Utc>::from(modified)).with_vars(watch.vars.clone()))
}

fn effects_of(
    pipeline: &crate::pipeline::Pipeline,
    record: &Record,
    unit: &Path,
    roots: &Roots,
    dry_run: bool,
) -> What {
    let run = apply_effects(pipeline, record, unit, roots, dry_run);
    if let Some(error) = run.error {
        return What::Refused(error.to_string());
    }
    let moved = run.done.into_iter().find_map(|done| match done {
        Done::Moved(applied) => Some(applied),
        Done::Cleaned(_) => None,
    });
    match moved {
        Some(Applied::Moved { to, .. }) => What::Moved(to),
        Some(Applied::Preview { to, .. }) => What::Previewed(to),
        Some(Applied::Skipped(SkipReason::Link)) => What::Skipped("link".to_string()),
        Some(Applied::Skipped(SkipReason::NotAFile)) => What::Skipped("not a file".to_string()),
        Some(Applied::Unchanged(_)) | None => What::Unchanged,
    }
}

/// The folder configurations from the source down to the unit, from the farthest to the nearest.
/// One that cannot be read or used is left out and reported.
fn folder_configs(source: &Path, unit: &Path, processed: &mut Vec<Processed>) -> Vec<FolderConfig> {
    let mut folder = PathBuf::new();
    let mut layers = vec![folder.clone()];
    for component in unit.components() {
        folder.push(component);
        layers.push(folder.clone());
    }

    let mut configs = Vec::new();
    for layer in layers {
        let relative = layer.join(FOLDER_CONFIG);
        let Some(text) = read_folder_config(&source.join(&relative)) else {
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
fn read_folder_config(path: &Path) -> Option<std::io::Result<String>> {
    let metadata = fs::symlink_metadata(path).ok()?;
    if !metadata.is_file() {
        return None;
    }
    let mut text = String::new();
    let read = fs::File::open(path).and_then(|file| {
        file.take(MAX_FOLDER_CONFIG_BYTES as u64 + 1)
            .read_to_string(&mut text)
    });
    Some(read.map(|_| text))
}

fn report(entry: &Processed) {
    let origin = entry.origin.display();
    match &entry.what {
        What::Moved(to) => eprintln!("[info] {origin} -> {}", to.display()),
        What::Previewed(to) => eprintln!("[info] {origin} would go to {}", to.display()),
        What::Unchanged => {}
        What::Unclaimed => eprintln!("[info] {origin} left: no pipeline claims it"),
        What::Excluded => eprintln!("[info] {origin} left: a filter excluded it"),
        What::Skipped(reason) => eprintln!("[warn] {origin} skipped: {reason}"),
        What::Refused(reason) => eprintln!("[warn] {origin} refused: {reason}"),
        What::LeftTooLarge => eprintln!("[warn] {origin} left: the batch is too large"),
    }
}
