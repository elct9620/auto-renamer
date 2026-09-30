//! The files of a batch as the pipelines of a watch take them in turn.

use std::path::PathBuf;

use super::take::{Earlier, EarlierFiles};
use super::{Flow, Stop};
use crate::record::Record;

/// Where one file of a batch stands.
enum State {
    Waiting(Record),
    Held { pipeline: usize, record: Record },
    Stopped { pipeline: usize, stop: Stop },
}

struct Slot {
    origin: PathBuf,
    state: State,
}

/// The files of a batch in the order of their paths, each waiting for a pipeline, held by the one
/// that claimed it, or stopped by one of its stages.
pub struct Batch {
    slots: Vec<Slot>,
    /// The names of the pipelines that have claimed so far; the last one is running.
    pipelines: Vec<String>,
}

impl Batch {
    pub(crate) fn new(mut records: Vec<Record>) -> Batch {
        records.sort_by(|a, b| a.origin().cmp(b.origin()));
        let slots = records
            .into_iter()
            .map(|record| Slot {
                origin: record.origin().to_path_buf(),
                state: State::Waiting(record),
            })
            .collect();
        Batch {
            slots,
            pipelines: Vec::new(),
        }
    }

    /// The next pipeline claims each waiting file it accepts, and is the one running from here on.
    pub(crate) fn claim(&mut self, pipeline: &str, mut accepts: impl FnMut(&Record) -> bool) {
        let claiming = self.pipelines.len();
        self.pipelines.push(pipeline.to_string());
        self.update(|state| match state {
            State::Waiting(record) if accepts(&record) => State::Held {
                pipeline: claiming,
                record,
            },
            other => other,
        });
    }

    /// Runs a change over every record the running pipeline still holds. A record goes on as the
    /// change answers it, and what stops one ends its way, so no later change sees it.
    pub(crate) fn each(&mut self, mut change: impl FnMut(Record) -> Flow) {
        let running = self.running();
        self.update(|state| match state {
            State::Held { pipeline, record } if Some(pipeline) == running => match change(record) {
                Ok(record) => State::Held { pipeline, record },
                Err(stop) => State::Stopped { pipeline, stop },
            },
            other => other,
        });
    }

    /// The records the running pipeline still holds, in the order of their paths.
    pub(crate) fn live(&self) -> impl Iterator<Item = &Record> {
        let running = self.running();
        self.slots.iter().filter_map(move |slot| match &slot.state {
            State::Held { pipeline, record } if Some(*pipeline) == running => Some(record),
            _ => None,
        })
    }

    /// What the pipelines before the running one made of their files.
    pub(crate) fn earlier(&self) -> EarlierFiles {
        let running = self.running();
        let files = self.slots.iter().filter_map(|slot| {
            let (pipeline, planned) = match &slot.state {
                State::Held { pipeline, record } => (*pipeline, Some(record.clone())),
                State::Stopped { pipeline, .. } => (*pipeline, None),
                State::Waiting(_) => return None,
            };
            (Some(pipeline) != running).then(|| Earlier {
                pipeline: self.pipelines[pipeline].clone(),
                origin: slot.origin.clone(),
                planned,
            })
        });
        EarlierFiles::new(files.collect())
    }

    /// How each file ended, in the order of their paths: the pipeline that claimed it with the
    /// record it planned or what stopped it, or none for a file no pipeline claimed.
    pub(crate) fn into_files(self) -> impl Iterator<Item = (PathBuf, Option<(usize, Flow)>)> {
        self.slots.into_iter().map(|slot| {
            let claimed = match slot.state {
                State::Waiting(_) => None,
                State::Held { pipeline, record } => Some((pipeline, Ok(record))),
                State::Stopped { pipeline, stop } => Some((pipeline, Err(stop))),
            };
            (slot.origin, claimed)
        })
    }

    fn running(&self) -> Option<usize> {
        self.pipelines.len().checked_sub(1)
    }

    fn update(&mut self, mut change: impl FnMut(State) -> State) {
        self.slots = std::mem::take(&mut self.slots)
            .into_iter()
            .map(|slot| Slot {
                origin: slot.origin,
                state: change(slot.state),
            })
            .collect();
    }
}
