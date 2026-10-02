//! The files a pipeline claimed, as its stages take them in turn.

use std::path::{Path, PathBuf};

use super::Flow;
use crate::record::Record;

/// One file of a batch: going on while its flow holds a record, and stopped once a stage ended its way.
pub(crate) struct File {
    pub origin: PathBuf,
    pub flow: Flow,
    /// The record as it was before the stage that stopped it, kept when the batch was asked to.
    pub stopped: Option<Record>,
}

/// The files a pipeline claimed in the order of their paths, each going on with its record or stopped by
/// one of the stages.
pub struct Batch {
    files: Vec<File>,
    keeping: bool,
}

impl Batch {
    pub(crate) fn new(mut records: Vec<Record>) -> Batch {
        records.sort_by(|a, b| a.origin().cmp(b.origin()));
        let files = records
            .into_iter()
            .map(|record| File {
                origin: record.origin().to_path_buf(),
                flow: Ok(record),
                stopped: None,
            })
            .collect();
        Batch {
            files,
            keeping: false,
        }
    }

    /// Keeps each record a stage stops as it was before that stage, for a route that sends refused files on.
    pub(crate) fn keeping_stopped(mut self) -> Batch {
        self.keeping = true;
        self
    }

    /// Runs a change over every record still going on. A record goes on as the change answers it, and
    /// what stops one ends its way, so no later change sees it; the stop says the plan the record had.
    pub(crate) fn each(mut self, mut change: impl FnMut(Record) -> Flow) -> Batch {
        let keeping = self.keeping;
        self.files = self
            .files
            .into_iter()
            .map(|mut file| {
                if let Ok(record) = file.flow {
                    let planned = record.plan().to_path_buf();
                    let before = keeping.then(|| record.clone());
                    file.flow = change(record).map_err(|mut stop| {
                        stop.planned = planned;
                        file.stopped = before;
                        stop
                    });
                }
                file
            })
            .collect();
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
