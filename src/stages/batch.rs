//! The files of a batch as the pipelines of a watch take them in turn.

use std::path::PathBuf;

use super::take::{Earlier, EarlierFiles};
use super::{Effect, Flow, Stop};
use crate::record::Record;

/// One file of a batch: waiting while no pipeline has claimed it, going on while its flow holds a
/// record, and stopped once a stage of its pipeline ended its way.
pub(crate) struct File {
    pub origin: PathBuf,
    /// Where the pipeline that claimed the file stands among those that have claimed.
    pub pipeline: Option<usize>,
    pub flow: Flow,
    /// What is to be done to the file once it is planned.
    pub effects: Vec<Effect>,
}

/// The files of a batch in the order of their paths, as the pipelines of a watch take them in turn.
pub struct Batch {
    files: Vec<File>,
    /// The names of the pipelines that have claimed so far; the last one is running.
    pipelines: Vec<String>,
}

impl Batch {
    pub(crate) fn new(mut records: Vec<Record>) -> Batch {
        records.sort_by(|a, b| a.origin().cmp(b.origin()));
        let files = records
            .into_iter()
            .map(|record| File {
                origin: record.origin().to_path_buf(),
                pipeline: None,
                flow: Ok(record),
                effects: Vec::new(),
            })
            .collect();
        Batch {
            files,
            pipelines: Vec::new(),
        }
    }

    /// The next pipeline claims each waiting file it accepts, and is the one running from here on.
    pub(crate) fn claim(&mut self, pipeline: &str, mut accepts: impl FnMut(&Record) -> bool) {
        let claiming = self.pipelines.len();
        self.pipelines.push(pipeline.to_string());
        for file in &mut self.files {
            if file.pipeline.is_none() && file.flow.as_ref().is_ok_and(&mut accepts) {
                file.pipeline = Some(claiming);
            }
        }
    }

    /// Runs a change over every record the running pipeline still holds. A record goes on as the
    /// change answers it, and what stops one ends its way, so no later change sees it.
    pub(crate) fn each(&mut self, mut change: impl FnMut(Record) -> Flow) {
        for file in self.running_mut() {
            let flow = std::mem::replace(&mut file.flow, Err(Stop::Excluded));
            file.flow = flow.and_then(&mut change);
        }
    }

    /// Notes an effect to be done to every file the running pipeline still holds.
    pub(crate) fn schedule(&mut self, effect: Effect) {
        for file in self.running_mut().filter(|file| file.flow.is_ok()) {
            file.effects.push(effect.clone());
        }
    }

    /// The records the running pipeline still holds, in the order of their paths.
    pub(crate) fn live(&self) -> impl Iterator<Item = &Record> {
        let running = self.running();
        self.files
            .iter()
            .filter(move |file| running.is_some() && file.pipeline == running)
            .filter_map(|file| file.flow.as_ref().ok())
    }

    /// What the pipelines before the running one made of their files.
    pub(crate) fn earlier(&self) -> EarlierFiles {
        let running = self.running();
        let earlier = self.files.iter().filter_map(|file| {
            let pipeline = file
                .pipeline
                .filter(|pipeline| Some(*pipeline) != running)?;
            Some(Earlier {
                pipeline: self.pipelines[pipeline].clone(),
                origin: file.origin.clone(),
                planned: file.flow.as_ref().ok().cloned(),
            })
        });
        EarlierFiles::new(earlier.collect())
    }

    /// The files as they ended, in the order of their paths.
    pub(crate) fn into_files(self) -> Vec<File> {
        self.files
    }

    fn running(&self) -> Option<usize> {
        self.pipelines.len().checked_sub(1)
    }

    fn running_mut(&mut self) -> impl Iterator<Item = &mut File> {
        let running = self.running();
        self.files
            .iter_mut()
            .filter(move |file| running.is_some() && file.pipeline == running)
    }
}
