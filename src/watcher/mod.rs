//! Decides when the settled files of a unit are handed over as a batch.
//!
//! The machine is told what happened to files and what time it is, and answers with the batches that are
//! ready, so it runs without a filesystem or a clock.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Bound;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::config::{FOLDER_CONFIG, MAX_BATCH_FILES, Unit, Watch};

mod notification;

pub use notification::{Translated, rewrites, translate};

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

/// A batch the machine hands over: the settled files of one unit. Whether it is too large is decided
/// where the folder configurations that may raise the limit are read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ready {
    pub unit: PathBuf,
    pub files: Vec<PathBuf>,
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

/// What the machine looks at again when its time comes: a hold that ends, or a unit that may be due.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Waiting {
    Hold(PathBuf),
    Unit(PathBuf),
}

/// Decides when the settled files of a unit are handed over as a batch, from what it is told and what time it is.
///
/// What an event or a question costs does not grow with how many files wait: the times to look again are
/// kept in order, and whether a unit is held open is counted rather than asked of every hold.
pub struct Machine {
    unit: Unit,
    window: Duration,
    max_wait: Duration,
    pending: BTreeMap<PathBuf, Pending>,
    touched: BTreeMap<PathBuf, SystemTime>,
    holds: BTreeMap<PathBuf, Hold>,
    holds_in: BTreeMap<PathBuf, usize>,
    deadlines: BTreeSet<(SystemTime, Waiting)>,
}

impl Machine {
    /// Starts a machine for a watch, with its unit, window and maximum wait.
    pub fn new(watch: &Watch) -> Machine {
        Machine {
            unit: watch.unit.clone(),
            window: watch.batch_window,
            max_wait: watch.batch_max_wait,
            pending: BTreeMap::new(),
            touched: BTreeMap::new(),
            holds: BTreeMap::new(),
            holds_in: BTreeMap::new(),
            deadlines: BTreeSet::new(),
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
        for file in self.due(now, Waiting::held_file) {
            self.resolve_hold(&file, now);
        }
        let units: BTreeSet<PathBuf> = self.due(now, Waiting::unit).into_iter().collect();
        units
            .into_iter()
            .filter_map(|unit| self.close(&unit))
            .collect()
    }

    /// The earliest time at which asking again could change the answer, or none when nothing is waiting.
    pub fn next_deadline(&self) -> Option<SystemTime> {
        self.deadlines.first().map(|(at, _)| *at)
    }

    fn settle(&mut self, path: PathBuf, activity: SystemTime) {
        let unit = self.unit.of(&path);
        self.change(&unit, |machine| {
            machine.let_go(&path);
            let pending = machine
                .pending
                .entry(unit.clone())
                .or_insert_with(|| Pending {
                    first: activity,
                    files: BTreeSet::new(),
                });
            // One file past the ceiling is enough to know the batch is too large.
            if pending.files.len() <= MAX_BATCH_FILES {
                pending.files.insert(path);
            }
            machine.touch(&unit, activity);
        });
    }

    fn write(&mut self, path: PathBuf, now: SystemTime) {
        let unit = self.unit.of(&path);
        self.change(&unit, |machine| {
            machine.unsettle(&unit, &path);
            machine.hold(path, &unit, now, None);
            machine.touch(&unit, now);
        });
    }

    /// A file that is gone, or a folder that is gone with every file under it.
    fn forget(&mut self, path: &Path) {
        let held_files: Vec<PathBuf> = keys_under(&self.holds, path);
        let below: Vec<PathBuf> = keys_under(&self.pending, path);
        let above = path
            .ancestors()
            .skip(1)
            .filter(|unit| self.pending.contains_key(*unit));
        let units: BTreeSet<PathBuf> = held_files
            .iter()
            .filter_map(|file| self.holds.get(file))
            .map(|hold| hold.unit.clone())
            .chain(below.iter().cloned())
            .chain(above.map(Path::to_path_buf))
            .collect();

        for unit in &units {
            self.unschedule(unit);
        }
        for file in &held_files {
            self.let_go(file);
        }
        for unit in &below {
            self.pending.remove(unit);
        }
        for unit in path.ancestors().skip(1) {
            self.unsettle_under(unit, path);
        }
        for unit in &units {
            self.release(unit);
            self.schedule(unit);
        }
    }

    /// Forgets when a unit was last active once nothing in it is waiting or held.
    fn release(&mut self, unit: &Path) {
        if !self.pending.contains_key(unit) && !self.holds_open(unit) {
            self.touched.remove(unit);
        }
    }

    /// A modification time comes from the file and cannot be trusted: one in the future counts as now.
    fn found(&mut self, path: PathBuf, modified: SystemTime, now: SystemTime) {
        let modified = modified.min(now);
        let quiet_at = modified + self.window;
        if now >= quiet_at {
            self.settle(path, modified);
        } else {
            let unit = self.unit.of(&path);
            self.change(&unit, |machine| {
                machine.hold(path, &unit, now, Some(quiet_at));
                machine.touch(&unit, modified);
            });
        }
    }

    /// A file found at start settles once its window has passed with no writes; a hold that has not
    /// resolved for the maximum wait is dropped.
    fn resolve_hold(&mut self, file: &Path, now: SystemTime) {
        let Some(hold) = self.holds.get(file) else {
            return;
        };
        match hold.due {
            Some(due) if now >= due => self.settle(file.to_path_buf(), due - self.window),
            _ => {
                let unit = hold.unit.clone();
                self.change(&unit, |machine| {
                    machine.let_go(file);
                    machine.release(&unit);
                });
            }
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

    /// Takes the files under a folder out of a unit that reaches above that folder.
    fn unsettle_under(&mut self, unit: &Path, folder: &Path) {
        if let Some(pending) = self.pending.get_mut(unit) {
            for file in files_under(&pending.files, folder) {
                pending.files.remove(&file);
            }
            if pending.files.is_empty() {
                self.pending.remove(unit);
            }
        }
    }

    fn touch(&mut self, unit: &Path, when: SystemTime) {
        self.touched
            .entry(unit.to_path_buf())
            .and_modify(|last| *last = (*last).max(when))
            .or_insert(when);
    }

    /// Holds a file, in place of any hold it already had.
    fn hold(&mut self, file: PathBuf, unit: &Path, since: SystemTime, due: Option<SystemTime>) {
        self.let_go(&file);
        let hold = Hold {
            unit: unit.to_path_buf(),
            since,
            due,
        };
        self.deadlines
            .insert((self.hold_ends_at(&hold), Waiting::Hold(file.clone())));
        *self.holds_in.entry(hold.unit.clone()).or_default() += 1;
        self.holds.insert(file, hold);
    }

    fn let_go(&mut self, file: &Path) {
        let Some(hold) = self.holds.remove(file) else {
            return;
        };
        self.deadlines
            .remove(&(self.hold_ends_at(&hold), Waiting::Hold(file.to_path_buf())));
        if let Some(count) = self.holds_in.get_mut(&hold.unit) {
            *count -= 1;
            if *count == 0 {
                self.holds_in.remove(&hold.unit);
            }
        }
    }

    fn holds_open(&self, unit: &Path) -> bool {
        self.holds_in.contains_key(unit)
    }

    fn hold_ends_at(&self, hold: &Hold) -> SystemTime {
        let stale_at = hold.since + self.max_wait;
        hold.due.map_or(stale_at, |due| due.min(stale_at))
    }

    fn quiet_at(&self, unit: &Path, pending: &Pending) -> SystemTime {
        self.touched.get(unit).copied().unwrap_or(pending.first) + self.window
    }

    /// When a pending unit is due: at its maximum wait, or sooner once it is quiet with nothing held.
    fn due_at(&self, unit: &Path, pending: &Pending) -> SystemTime {
        let cap = pending.first + self.max_wait;
        if self.holds_open(unit) {
            cap
        } else {
            cap.min(self.quiet_at(unit, pending))
        }
    }

    /// Changes what a unit holds or waits for, keeping the time it is looked at again in step.
    fn change(&mut self, unit: &Path, change: impl FnOnce(&mut Machine)) {
        self.unschedule(unit);
        change(self);
        self.schedule(unit);
    }

    fn schedule(&mut self, unit: &Path) {
        if let Some(pending) = self.pending.get(unit) {
            let at = self.due_at(unit, pending);
            self.deadlines
                .insert((at, Waiting::Unit(unit.to_path_buf())));
        }
    }

    fn unschedule(&mut self, unit: &Path) {
        if let Some(pending) = self.pending.get(unit) {
            let at = self.due_at(unit, pending);
            self.deadlines
                .remove(&(at, Waiting::Unit(unit.to_path_buf())));
        }
    }

    /// What has come due of one kind, earliest first.
    fn due(&self, now: SystemTime, kind: fn(&Waiting) -> Option<&PathBuf>) -> Vec<PathBuf> {
        self.deadlines
            .iter()
            .take_while(|(at, _)| *at <= now)
            .filter_map(|(_, waiting)| kind(waiting))
            .cloned()
            .collect()
    }

    fn close(&mut self, unit: &Path) -> Option<Ready> {
        self.unschedule(unit);
        let pending = self.pending.remove(unit)?;
        self.touched.remove(unit);
        let files: Vec<PathBuf> = pending.files.into_iter().collect();
        if files.is_empty() {
            return None;
        }
        Some(Ready {
            unit: unit.to_path_buf(),
            files,
        })
    }
}

impl Waiting {
    fn held_file(&self) -> Option<&PathBuf> {
        match self {
            Waiting::Hold(file) => Some(file),
            Waiting::Unit(_) => None,
        }
    }

    fn unit(&self) -> Option<&PathBuf> {
        match self {
            Waiting::Unit(unit) => Some(unit),
            Waiting::Hold(_) => None,
        }
    }
}

/// The paths at or under a folder. Paths are ordered part by part, so a folder is followed at once by
/// everything under it and nothing else comes between.
fn keys_under<V>(map: &BTreeMap<PathBuf, V>, folder: &Path) -> Vec<PathBuf> {
    map.range::<Path, _>((Bound::Included(folder), Bound::Unbounded))
        .map(|(path, _)| path)
        .take_while(|path| path.starts_with(folder))
        .cloned()
        .collect()
}

fn files_under(files: &BTreeSet<PathBuf>, folder: &Path) -> Vec<PathBuf> {
    files
        .range::<Path, _>((Bound::Included(folder), Bound::Unbounded))
        .take_while(|path| path.starts_with(folder))
        .cloned()
        .collect()
}

fn is_folder_config(path: &Path) -> bool {
    path.file_name().is_some_and(|name| name == FOLDER_CONFIG)
}
