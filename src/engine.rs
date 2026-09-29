use std::path::PathBuf;

use crate::context::Context;
use crate::pipeline::Pipeline;
use crate::record::Record;
use crate::stages::{Earlier, EarlierFiles, Outcome, Rejection, Stage};

/// What a batch made of one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Planned(Record),
    Excluded,
    Unclaimed,
    Rejected(Rejection),
}

/// One file of a batch with the verdict on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Judged {
    pub origin: PathBuf,
    /// Where the pipeline that claimed the file stands in the list planned with, or none when no pipeline did.
    pub pipeline: Option<usize>,
    pub verdict: Verdict,
}

enum Slot {
    Waiting(Record),
    Live {
        pipeline: usize,
        record: Record,
    },
    Done {
        pipeline: Option<usize>,
        verdict: Verdict,
    },
}

impl Slot {
    fn is_live_in(&self, pipeline: usize) -> bool {
        matches!(self, Slot::Live { pipeline: owner, .. } if *owner == pipeline)
    }

    /// Takes the record out of a slot that is running in the pipeline, leaving the slot to be filled again.
    fn take_live(&mut self, pipeline: usize) -> Option<Record> {
        if !self.is_live_in(pipeline) {
            return None;
        }
        let vacated = Slot::Done {
            pipeline: None,
            verdict: Verdict::Unclaimed,
        };
        match std::mem::replace(self, vacated) {
            Slot::Live { record, .. } => Some(record),
            _ => None,
        }
    }

    fn settle(pipeline: usize, outcome: Outcome) -> Slot {
        match outcome {
            Outcome::Continue(record) => Slot::Live { pipeline, record },
            Outcome::Excluded => Slot::Done {
                pipeline: Some(pipeline),
                verdict: Verdict::Excluded,
            },
            Outcome::Rejected(rejection) => Slot::Done {
                pipeline: Some(pipeline),
                verdict: Verdict::Rejected(rejection),
            },
        }
    }
}

/// Plans a whole batch through the pipelines of a watch, in the order they are listed.
///
/// Files are taken in the order of their paths. Each is claimed by the first pipeline whose leading filters
/// accept it, and the stages of a pipeline run over all of its files one stage at a time, up to the first stage
/// that touches the filesystem.
pub fn plan_batch(
    pipelines: &[(String, Pipeline)],
    mut records: Vec<Record>,
    context: &mut Context,
) -> Vec<Judged> {
    records.sort_by(|a, b| a.origin().cmp(b.origin()));
    let origins: Vec<PathBuf> = records
        .iter()
        .map(|record| record.origin().to_path_buf())
        .collect();
    let mut slots: Vec<Slot> = records.into_iter().map(Slot::Waiting).collect();

    for (index, (_, pipeline)) in pipelines.iter().enumerate() {
        claim(pipeline, index, &mut slots, context);
        run_stages(pipelines, index, &origins, &mut slots, context);
        finish(index, &mut slots);
    }

    slots
        .into_iter()
        .zip(origins)
        .map(|(slot, origin)| {
            let (pipeline, verdict) = match slot {
                Slot::Waiting(record) if !record.is_readable() => (
                    None,
                    Verdict::Rejected(Rejection {
                        stage: "name".to_string(),
                        reason: "the file name is not valid UTF-8".to_string(),
                    }),
                ),
                Slot::Waiting(_) => (None, Verdict::Unclaimed),
                Slot::Live { pipeline, record } => (Some(pipeline), Verdict::Planned(record)),
                Slot::Done { pipeline, verdict } => (pipeline, verdict),
            };
            Judged {
                origin,
                pipeline,
                verdict,
            }
        })
        .collect()
}

fn leading_filters(pipeline: &Pipeline) -> usize {
    pipeline
        .stages()
        .iter()
        .take_while(|stage| matches!(stage, Stage::Filter(_)))
        .count()
}

/// The pipeline claims each waiting file that every one of its leading filters lets through.
fn claim(pipeline: &Pipeline, index: usize, slots: &mut [Slot], context: &mut Context) {
    let filters = &pipeline.stages()[..leading_filters(pipeline)];
    for slot in slots {
        let Slot::Waiting(record) = slot else {
            continue;
        };
        let claimed = filters
            .iter()
            .all(|filter| matches!(filter.apply(record.clone(), context), Outcome::Continue(_)));
        if claimed {
            *slot = Slot::Live {
                pipeline: index,
                record: record.clone(),
            };
        }
    }
}

fn run_stages(
    pipelines: &[(String, Pipeline)],
    index: usize,
    origins: &[PathBuf],
    slots: &mut [Slot],
    context: &mut Context,
) {
    let (_, pipeline) = &pipelines[index];
    let stages = pipeline
        .stages()
        .iter()
        .skip(leading_filters(pipeline))
        .take_while(|stage| !stage.is_effect());

    for stage in stages {
        match stage {
            Stage::Rank(_) => run_rank(stage, index, slots),
            Stage::Take(_) => run_take(stage, pipelines, index, origins, slots),
            _ => map_live(slots, index, |record| stage.apply(record, context)),
        }
    }
}

/// Runs a change over every record the pipeline still holds live, settling each on what the change answers.
fn map_live(slots: &mut [Slot], index: usize, mut change: impl FnMut(Record) -> Outcome) {
    for slot in slots {
        if let Some(record) = slot.take_live(index) {
            *slot = Slot::settle(index, change(record));
        }
    }
}

fn run_rank(stage: &Stage, index: usize, slots: &mut [Slot]) {
    let live: Vec<usize> = (0..slots.len())
        .filter(|&position| slots[position].is_live_in(index))
        .collect();
    let records: Vec<Record> = live
        .iter()
        .filter_map(|&position| slots[position].take_live(index))
        .collect();
    let outcomes = stage.rank_batch(records).unwrap_or_default();
    for (position, outcome) in live.into_iter().zip(outcomes) {
        slots[position] = Slot::settle(index, outcome);
    }
}

fn run_take(
    stage: &Stage,
    pipelines: &[(String, Pipeline)],
    index: usize,
    origins: &[PathBuf],
    slots: &mut [Slot],
) {
    let earlier: Vec<Earlier> = slots
        .iter()
        .zip(origins)
        .filter_map(|(slot, origin)| match slot {
            Slot::Done {
                pipeline: Some(owner),
                verdict,
            } if *owner < index => Some(Earlier {
                pipeline: pipelines[*owner].0.clone(),
                origin: origin.clone(),
                planned: match verdict {
                    Verdict::Planned(record) => Some(record.clone()),
                    _ => None,
                },
            }),
            _ => None,
        })
        .collect();

    let earlier = EarlierFiles::new(earlier);
    map_live(slots, index, |record| stage.take_from(record, &earlier));
}

/// What the pipeline still holds live is planned.
fn finish(index: usize, slots: &mut [Slot]) {
    for slot in slots {
        if let Some(record) = slot.take_live(index) {
            *slot = Slot::Done {
                pipeline: Some(index),
                verdict: Verdict::Planned(record),
            };
        }
    }
}
