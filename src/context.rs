use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// What the target folder holds, as far as a stage needs to know.
pub trait Target {
    /// The names of the files in a folder of the target, relative to its root.
    fn files_in(&self, folder: &Path) -> Vec<String>;
}

/// What a run of stages shares: the target, what it was already asked, and the numbers already handed
/// out in the batch. Planning changes nothing in the target, so a folder is asked for once.
pub struct Context<'a> {
    target: &'a dyn Target,
    listed: BTreeMap<PathBuf, Vec<String>>,
    handed_out: BTreeMap<(PathBuf, String), u64>,
}

impl<'a> Context<'a> {
    /// Starts a context over a target.
    pub fn new(target: &'a dyn Target) -> Context<'a> {
        Context {
            target,
            listed: BTreeMap::new(),
            handed_out: BTreeMap::new(),
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
}
