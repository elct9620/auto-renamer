use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::record::Record;

/// What the target folder holds, as far as a stage needs to know.
pub trait Target {
    /// The names of the files in a folder of the target, relative to its root.
    fn files_in(&self, folder: &Path) -> Vec<String>;
}

/// What an earlier pipeline made of one file, as far as a later stage needs to know.
#[derive(Clone)]
pub(crate) struct Earlier {
    pub pipeline: String,
    pub origin: PathBuf,
    pub planned: Option<Record>,
}

/// What a run of stages shares: the target, what it was already asked, the numbers already handed out
/// in the batch, and what the pipelines before made of their files. Planning changes nothing in the
/// target, so a folder is asked for once.
pub struct Context<'a> {
    target: &'a dyn Target,
    listed: BTreeMap<PathBuf, Vec<String>>,
    handed_out: BTreeMap<(PathBuf, String), u64>,
    earlier: Vec<Earlier>,
}

impl<'a> Context<'a> {
    /// Starts a context over a target.
    pub fn new(target: &'a dyn Target) -> Context<'a> {
        Context {
            target,
            listed: BTreeMap::new(),
            handed_out: BTreeMap::new(),
            earlier: Vec::new(),
        }
    }

    pub(crate) fn files_in(&mut self, folder: &Path) -> &[String] {
        let target = self.target;
        self.listed
            .entry(folder.to_path_buf())
            .or_insert_with(|| target.files_in(folder))
    }

    pub(crate) fn last_handed_out(&self, key: &(PathBuf, String)) -> Option<u64> {
        self.handed_out.get(key).copied()
    }

    pub(crate) fn hand_out(&mut self, key: (PathBuf, String), number: u64) {
        self.handed_out.insert(key, number);
    }

    /// What the pipelines before the running one made of their files.
    pub(crate) fn earlier(&self) -> &[Earlier] {
        &self.earlier
    }

    /// Keeps what a pipeline made of its files, for the pipelines after it.
    pub(crate) fn remember(&mut self, files: impl IntoIterator<Item = Earlier>) {
        self.earlier.extend(files);
    }
}
