//! Turns a ready batch of settled files into moved, unchanged or refused files.

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use crate::config::{FOLDER_CONFIG, FolderConfig, MAX_FOLDER_CONFIG_BYTES, Watch};
use crate::context::Target;
use crate::effects::{Applied, Done, Effect, Kind, Roots, SkipReason, Tree, apply_effects};
use crate::engine::{Planned, Step, Takes, Verdict, plan};
use crate::pipeline::Pipeline;
use crate::record::Record;

/// The target as a stage asks for it, answered by the tree under the root of the target.
struct TargetIn<'a> {
    tree: &'a dyn Tree,
    root: PathBuf,
}

impl Target for TargetIn<'_> {
    fn files_in(&self, folder: &Path) -> Vec<String> {
        self.tree.files(&self.root.join(folder))
    }

    fn root(&self) -> &Path {
        &self.root
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
    process(tree, watch, unit, files, renames, None)
}

/// Processes a ready batch as [`process_batch`] does, telling `observe` every step of its planning.
pub fn process_batch_observed(
    tree: &dyn Tree,
    watch: &Watch,
    unit: &Path,
    files: &[PathBuf],
    renames: &mut Renames,
    observe: &mut dyn FnMut(Step),
) -> Vec<Processed> {
    process(tree, watch, unit, files, renames, Some(observe))
}

fn process(
    tree: &dyn Tree,
    watch: &Watch,
    unit: &Path,
    files: &[PathBuf],
    renames: &mut Renames,
    observe: Option<&mut dyn FnMut(Step)>,
) -> Vec<Processed> {
    let mut processed = Vec::new();
    let layers = folder_configs(tree, &watch.source, unit, &mut processed);
    let effective = watch.under(&layers);

    if files.len() > effective.max_files {
        processed.extend(files.iter().map(|origin| Processed {
            origin: origin.clone(),
            what: What::LeftTooLarge,
        }));
        return processed;
    }

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

    let legs = legs_of(&effective);
    let targets: Vec<TargetIn> = legs
        .iter()
        .map(|leg| TargetIn {
            tree,
            root: leg.root(&effective),
        })
        .collect();
    let planned: Vec<Planned> = legs
        .iter()
        .zip(&targets)
        .map(|(leg, target)| Planned {
            name: &leg.name,
            pipeline: &leg.pipeline,
            target,
            takes: leg.takes,
        })
        .collect();
    let judged = plan(&planned, records, &effective.group, observe);
    for entry in judged {
        let what = match (entry.verdict, entry.pipeline) {
            (Verdict::Planned(record), Some(index)) => {
                apply_planned(tree, &legs[index], &record, unit, &effective, renames)
            }
            (Verdict::Rejected(stop), _) => What::Refused(format!(
                "{}: {}, planned as `{}`",
                stop.stage,
                stop.reason,
                stop.planned.display()
            )),
            _ => What::Unclaimed,
        };
        processed.push(Processed {
            origin: entry.origin,
            what,
        });
    }
    processed
}

/// One pipeline of a watch as its batch is planned: a route's own, or the one its rejected route takes the
/// refused files with, together with where it moves them and what it does to them.
struct Leg {
    name: String,
    pipeline: Pipeline,
    target: Option<PathBuf>,
    effects: Vec<Effect>,
    takes: Takes,
}

impl Leg {
    /// Where the leg moves its files: its target, or the source when it renames them in place.
    fn root(&self, watch: &Watch) -> PathBuf {
        self.target.clone().unwrap_or_else(|| watch.source.clone())
    }
}

/// The routes of a watch in their order, and after them the rejected routes, each taking what its route
/// refused.
fn legs_of(watch: &Watch) -> Vec<Leg> {
    let routes = watch.routes();
    let mut legs: Vec<Leg> = routes
        .iter()
        .map(|route| Leg {
            name: route.pipeline.clone(),
            pipeline: watch.pipeline(&route.pipeline).clone(),
            target: route.target.clone(),
            effects: route.effects(),
            takes: Takes::Fresh,
        })
        .collect();
    for (index, route) in routes.iter().enumerate() {
        if let Some(rejected) = &route.rejected {
            legs.push(Leg {
                name: rejected.pipeline.clone().unwrap_or_default(),
                pipeline: rejected
                    .pipeline
                    .as_deref()
                    .map(|name| watch.pipeline(name).clone())
                    .unwrap_or_default(),
                target: rejected.target.clone(),
                effects: rejected.effects(),
                takes: Takes::RefusedBy(index),
            });
        }
    }
    legs
}

/// Runs the effects of the leg on a planned file, unless it has been renamed in place too many times
/// in a row, and keeps count of its renames in place.
fn apply_planned(
    tree: &dyn Tree,
    leg: &Leg,
    record: &Record,
    unit: &Path,
    watch: &Watch,
    renames: &mut Renames,
) -> What {
    let roots = Roots {
        source: watch.source.clone(),
        target: leg.root(watch),
    };
    let in_place = leg.target.is_none();
    let origin = record.origin();
    let count = renames.count(origin);
    if in_place && record.plan() != origin && count >= MAX_IN_PLACE_RENAMES {
        return What::Refused(format!(
            "renamed in place {count} times in a row; the pipeline may name its own result again"
        ));
    }
    let what = effects_of(tree, &leg.effects, record, unit, &roots, watch.dry_run);
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
    tree: &dyn Tree,
    effects: &[Effect],
    record: &Record,
    unit: &Path,
    roots: &Roots,
    dry_run: bool,
) -> What {
    let run = apply_effects(tree, effects, record, unit, roots, dry_run);
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
/// The pipelines of a watch that the folder configurations applying to a unit replace, in the watch's order.
/// A folder configuration that cannot be read replaces nothing; processing the batch reports it.
pub fn replaced_pipelines(tree: &dyn Tree, watch: &Watch, unit: &Path) -> Vec<String> {
    let layers = folder_configs(tree, &watch.source, unit, &mut Vec::new());
    watch
        .pipelines()
        .into_iter()
        .map(|(name, _)| name)
        .filter(|name| layers.iter().any(|layer| layer.declares(name)))
        .collect()
}

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
