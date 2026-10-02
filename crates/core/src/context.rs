use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::record::Record;

/// What the target folder holds, as far as a stage needs to know.
pub trait Target {
    /// The names of the files in a folder of the target, relative to its root.
    fn files_in(&self, folder: &Path) -> Vec<String>;

    /// Where the target lies, which tells two targets apart.
    fn root(&self) -> &Path;
}

/// What an earlier pipeline made of one file, as far as a later stage needs to know.
pub(crate) struct Earlier {
    pub pipeline: String,
    pub origin: PathBuf,
    pub planned: Option<Record>,
}

/// What a run of stages shares: the target of the running pipeline, what each target was already asked,
/// the numbers already handed out in the batch, and what the pipelines before made of their files.
/// Planning changes nothing in a target, so a folder is asked for once, and pipelines moving into one
/// target count on from each other's numbers.
pub struct Context<'a> {
    target: &'a dyn Target,
    listed: BTreeMap<PathBuf, Vec<String>>,
    handed_out: BTreeMap<(PathBuf, String), u64>,
    claimed: BTreeSet<PathBuf>,
    earlier: Vec<Earlier>,
}

impl<'a> Context<'a> {
    /// Starts a context over a target.
    pub fn new(target: &'a dyn Target) -> Context<'a> {
        Context {
            target,
            listed: BTreeMap::new(),
            handed_out: BTreeMap::new(),
            claimed: BTreeSet::new(),
            earlier: Vec::new(),
        }
    }

    /// Looks into the target of the next pipeline to run.
    pub(crate) fn enter(&mut self, target: &'a dyn Target) {
        self.target = target;
    }

    pub(crate) fn files_in(&mut self, folder: &Path) -> &[String] {
        let target = self.target;
        self.listed
            .entry(target.root().join(folder))
            .or_insert_with(|| target.files_in(folder))
    }

    pub(crate) fn last_handed_out(&self, folder: &Path, pattern: &str) -> Option<u64> {
        self.handed_out.get(&self.key(folder, pattern)).copied()
    }

    pub(crate) fn hand_out(&mut self, folder: &Path, pattern: &str, number: u64) {
        let key = self.key(folder, pattern);
        self.handed_out.insert(key, number);
    }

    /// Takes a plan in the running target for one file, answering whether it was free: neither a file the
    /// target holds nor the plan of a file before it in the batch.
    pub(crate) fn claim(&mut self, plan: &Path) -> bool {
        let folder = plan.parent().unwrap_or(Path::new(""));
        let name = plan.file_name().and_then(|name| name.to_str());
        let held = name.is_some_and(|name| self.files_in(folder).iter().any(|file| file == name));
        let first = self.claimed.insert(self.target.root().join(plan));
        first && !held
    }

    fn key(&self, folder: &Path, pattern: &str) -> (PathBuf, String) {
        (self.target.root().join(folder), pattern.to_string())
    }

    /// What the pipelines before the running one made of their files.
    pub(crate) fn earlier(&self) -> &[Earlier] {
        &self.earlier
    }

    /// What the pipelines before the running one made of their files, to take a planned record back.
    pub(crate) fn earlier_mut(&mut self) -> &mut [Earlier] {
        &mut self.earlier
    }

    /// Keeps what a pipeline made of one of its files, for the pipelines after it.
    pub(crate) fn remember(&mut self, file: Earlier) {
        self.earlier.push(file);
    }

    /// What the pipelines made of their files, once no stage is left to ask.
    pub(crate) fn into_earlier(self) -> Vec<Earlier> {
        self.earlier
    }
}
