//! The files a pipeline claimed, as its stages take them in turn.

use std::path::{Path, PathBuf};

use super::{Effect, Flow, Stop};
use crate::record::Record;

/// One file of a batch: going on while its flow holds a record, and stopped once a stage ended its way.
pub(crate) struct File {
    pub origin: PathBuf,
    pub flow: Flow,
    /// What is to be done to the file once it is planned.
    pub effects: Vec<Effect>,
}

/// The files a pipeline claimed in the order of their paths, each going on with its record or stopped by
/// one of the stages.
pub struct Batch {
    files: Vec<File>,
}

impl Batch {
    pub(crate) fn new(mut records: Vec<Record>) -> Batch {
        records.sort_by(|a, b| a.origin().cmp(b.origin()));
        let files = records
            .into_iter()
            .map(|record| File {
                origin: record.origin().to_path_buf(),
                flow: Ok(record),
                effects: Vec::new(),
            })
            .collect();
        Batch { files }
    }

    /// Runs a change over every record still going on. A record goes on as the change answers it, and
    /// what stops one ends its way, so no later change sees it.
    pub(crate) fn each(mut self, mut change: impl FnMut(Record) -> Flow) -> Batch {
        for file in &mut self.files {
            let flow = std::mem::replace(&mut file.flow, Err(Stop::Excluded));
            file.flow = flow.and_then(&mut change);
        }
        self
    }

    /// Notes an effect to be done to every file still going on.
    pub(crate) fn schedule(mut self, effect: Effect) -> Batch {
        for file in self.files.iter_mut().filter(|file| file.flow.is_ok()) {
            file.effects.push(effect.clone());
        }
        self
    }

    /// The records still going on, in the order of their paths.
    pub(crate) fn live(&self) -> impl Iterator<Item = &Record> {
        self.files.iter().filter_map(|file| file.flow.as_ref().ok())
    }

    /// Every file with the flow it holds now, in the order of their paths.
    pub(crate) fn files(&self) -> impl Iterator<Item = (&Path, &Flow)> {
        self.files
            .iter()
            .map(|file| (file.origin.as_path(), &file.flow))
    }

    /// The files as they ended, in the order of their paths.
    pub(crate) fn into_files(self) -> Vec<File> {
        self.files
    }
}
