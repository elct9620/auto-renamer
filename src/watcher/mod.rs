//! Decides when the settled files of a unit are handed over as a batch.
//!
//! The machine is told what happened to files and what time it is, and answers with the batches that are
//! ready, so it runs without a filesystem or a clock.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::config::{Unit, Watch};

mod notification;

pub use notification::{Translated, translate};

/// The name of a folder configuration, which is read as settings and never renamed.
const FOLDER_CONFIG: &str = "auto-renamer.toml";

/// What happened to a file of the source, as far as the machine needs to know. Paths are relative to the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// The file was written and closed, or moved in from elsewhere.
    Settled(PathBuf),
    /// The file is being written to.
    Writing(PathBuf),
    /// The file is gone.
    Gone(PathBuf),
    /// The file was found at start, last changed at `modified`.
    Found { path: PathBuf, modified: SystemTime },
}

/// A batch the machine hands over, or one it skipped for being too large.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ready {
    Batch { unit: PathBuf, files: Vec<PathBuf> },
    Skipped { unit: PathBuf, files: Vec<PathBuf> },
}

struct Pending {
    first: SystemTime,
    files: BTreeSet<PathBuf>,
}

/// A file that is not settled yet and so keeps its unit open: one being written to, or one found at
/// start that was changed so lately that it may still be written.
struct Hold {
    unit: PathBuf,
    since: SystemTime,
    due: Option<SystemTime>,
}

/// Decides when the settled files of a unit are handed over as a batch, from what it is told and what time it is.
pub struct Machine {
    unit: Unit,
    window: Duration,
    max_wait: Duration,
    limit: Option<usize>,
    pending: BTreeMap<PathBuf, Pending>,
    touched: BTreeMap<PathBuf, SystemTime>,
    holds: BTreeMap<PathBuf, Hold>,
}

impl Machine {
    /// Starts a machine for a watch, with its unit, window, maximum wait and batch limit.
    pub fn new(watch: &Watch) -> Machine {
        Machine {
            unit: watch.unit.clone(),
            window: watch.batch_window,
            max_wait: watch.batch_max_wait,
            limit: watch.batch_max,
            pending: BTreeMap::new(),
            touched: BTreeMap::new(),
            holds: BTreeMap::new(),
        }
    }

    /// Tells the machine what happened to a file and when.
    pub fn observe(&mut self, event: Event, now: SystemTime) {
        match event {
            Event::Settled(path) if !is_folder_config(&path) => self.settle(path, now),
            Event::Writing(path) if !is_folder_config(&path) => self.write(path, now),
            Event::Gone(path) => self.forget(&path),
            Event::Found { path, modified } if !is_folder_config(&path) => {
                self.found(path, modified, now)
            }
            _ => {}
        }
    }

    /// The batches that are ready at a time, each handed over once.
    pub fn ready(&mut self, now: SystemTime) -> Vec<Ready> {
        self.resolve_holds(now);
        let due: Vec<PathBuf> = self
            .pending
            .iter()
            .filter(|(unit, pending)| self.is_due(unit, pending, now))
            .map(|(unit, _)| unit.clone())
            .collect();
        due.into_iter()
            .filter_map(|unit| self.close(&unit))
            .collect()
    }

    /// The earliest time at which asking again could change the answer, or none when nothing is waiting.
    pub fn next_deadline(&self) -> Option<SystemTime> {
        let batches = self
            .pending
            .iter()
            .map(|(unit, pending)| self.checks_again_at(unit, pending));
        let holds = self.holds.values().map(|hold| self.hold_ends_at(hold));
        batches.chain(holds).min()
    }

    fn settle(&mut self, path: PathBuf, activity: SystemTime) {
        let unit = self.unit.of(&path);
        self.holds.remove(&path);
        self.pending
            .entry(unit.clone())
            .or_insert_with(|| Pending {
                first: activity,
                files: BTreeSet::new(),
            })
            .files
            .insert(path);
        self.touch(unit, activity);
    }

    fn write(&mut self, path: PathBuf, now: SystemTime) {
        let unit = self.unit.of(&path);
        self.unsettle(&unit, &path);
        self.holds.insert(
            path,
            Hold {
                unit: unit.clone(),
                since: now,
                due: None,
            },
        );
        self.touch(unit, now);
    }

    fn forget(&mut self, path: &Path) {
        let unit = self.unit.of(path);
        self.unsettle(&unit, path);
        self.holds.remove(path);
    }

    /// A modification time comes from the file and cannot be trusted: one in the future counts as now.
    fn found(&mut self, path: PathBuf, modified: SystemTime, now: SystemTime) {
        let modified = modified.min(now);
        let quiet_at = modified + self.window;
        if now >= quiet_at {
            self.settle(path, modified.min(now));
        } else {
            let unit = self.unit.of(&path);
            self.holds.insert(
                path,
                Hold {
                    unit: unit.clone(),
                    since: now,
                    due: Some(quiet_at),
                },
            );
            self.touch(unit, modified);
        }
    }

    /// A file found at start settles once its window has passed with no writes; a hold that has not
    /// resolved for the maximum wait is dropped.
    fn resolve_holds(&mut self, now: SystemTime) {
        let window = self.window;
        let mut quiet = Vec::new();
        let mut stale = Vec::new();
        for (path, hold) in &self.holds {
            match hold.due {
                Some(due) if now >= due => quiet.push((path.clone(), due - window)),
                _ if now >= hold.since + self.max_wait => stale.push(path.clone()),
                _ => {}
            }
        }
        for path in stale {
            self.holds.remove(&path);
        }
        for (path, modified) in quiet {
            self.settle(path, modified);
        }
    }

    fn unsettle(&mut self, unit: &Path, path: &Path) {
        if let Some(pending) = self.pending.get_mut(unit) {
            pending.files.remove(path);
            if pending.files.is_empty() {
                self.pending.remove(unit);
            }
        }
    }

    fn touch(&mut self, unit: PathBuf, when: SystemTime) {
        self.touched
            .entry(unit)
            .and_modify(|last| *last = (*last).max(when))
            .or_insert(when);
    }

    fn holds_open(&self, unit: &Path) -> bool {
        self.holds.values().any(|hold| hold.unit == unit)
    }

    fn hold_ends_at(&self, hold: &Hold) -> SystemTime {
        let stale_at = hold.since + self.max_wait;
        hold.due.map_or(stale_at, |due| due.min(stale_at))
    }

    fn quiet_at(&self, unit: &Path, pending: &Pending) -> SystemTime {
        self.touched.get(unit).copied().unwrap_or(pending.first) + self.window
    }

    fn is_due(&self, unit: &Path, pending: &Pending, now: SystemTime) -> bool {
        now >= pending.first + self.max_wait
            || (now >= self.quiet_at(unit, pending) && !self.holds_open(unit))
    }

    fn checks_again_at(&self, unit: &Path, pending: &Pending) -> SystemTime {
        let cap = pending.first + self.max_wait;
        let holds_end = self
            .holds
            .values()
            .filter(|hold| hold.unit == unit)
            .map(|hold| self.hold_ends_at(hold))
            .min();
        match holds_end {
            Some(ends) => cap.min(ends),
            None => cap.min(self.quiet_at(unit, pending)),
        }
    }

    fn close(&mut self, unit: &Path) -> Option<Ready> {
        let pending = self.pending.remove(unit)?;
        self.touched.remove(unit);
        let files: Vec<PathBuf> = pending.files.into_iter().collect();
        match self.limit {
            _ if files.is_empty() => None,
            Some(limit) if files.len() > limit => Some(Ready::Skipped {
                unit: unit.to_path_buf(),
                files,
            }),
            _ => Some(Ready::Batch {
                unit: unit.to_path_buf(),
                files,
            }),
        }
    }
}

fn is_folder_config(path: &Path) -> bool {
    path.file_name().is_some_and(|name| name == FOLDER_CONFIG)
}
