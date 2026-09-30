use std::path::PathBuf;

use crate::context::Context;
use crate::pipeline::Pipeline;
use crate::record::Record;
use crate::stages::{Batch, Effect, Flow, Rejection, Stop};

/// What a batch made of one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Planned(Record),
    Excluded,
    Unclaimed,
    Rejected(Rejection),
}

impl Verdict {
    /// The verdict on a file its pipeline ran to the end, or stopped.
    fn of(flow: Flow) -> Verdict {
        match flow {
            Ok(record) => Verdict::Planned(record),
            Err(Stop::Excluded) => Verdict::Excluded,
            Err(Stop::Rejected(rejection)) => Verdict::Rejected(rejection),
        }
    }
}

/// One file of a batch with the verdict on it.
#[derive(Debug, Clone)]
pub struct Judged<'p> {
    pub origin: PathBuf,
    /// Where the pipeline that claimed the file stands in the list planned with, or none when no pipeline did.
    pub pipeline: Option<usize>,
    pub verdict: Verdict,
    /// What the pipeline asks to be done to a planned file, in the order its effect stages are written.
    pub effects: Vec<Effect<'p>>,
}

/// Plans a whole batch through the pipelines of a watch, in the order they are listed.
///
/// Files are taken in the order of their paths. Each is claimed by the first pipeline whose leading filters
/// accept it, and the stages of a pipeline run over all of its files one stage at a time. Nothing touches the
/// filesystem: a stage that would only notes its effect on the files that reach it.
pub fn plan_batch<'p>(
    pipelines: &'p [(String, Pipeline)],
    records: Vec<Record>,
    context: &mut Context,
) -> Vec<Judged<'p>> {
    let mut batch = Batch::new(records);

    for (name, pipeline) in pipelines {
        let (filters, rest) = pipeline.split_at_claim();
        batch.claim(name, |record| {
            filters.iter().all(|filter| filter.accepts(record))
        });
        for stage in rest {
            stage.run(&mut batch, context);
        }
    }

    batch
        .into_files()
        .map(|file| {
            let (pipeline, verdict) = match file.claimed {
                None => (None, Verdict::Unclaimed),
                Some((pipeline, flow)) => (Some(pipeline), Verdict::of(flow)),
            };
            Judged {
                origin: file.origin,
                pipeline,
                verdict,
                effects: file.effects,
            }
        })
        .collect()
}
