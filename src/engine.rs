use std::path::PathBuf;

use crate::context::{Context, Earlier, Target};
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
pub struct Judged {
    pub origin: PathBuf,
    /// Where the pipeline that claimed the file stands in the list planned with, or none when no pipeline did.
    pub pipeline: Option<usize>,
    pub verdict: Verdict,
    /// What the pipeline asks to be done to a planned file, in the order its effect stages are written.
    pub effects: Vec<Effect>,
}

/// Plans a whole batch through the pipelines of a watch, in the order they are listed.
///
/// Files are taken in the order of their paths. Each is claimed by the first pipeline whose leading filters
/// accept it, and the stages of a pipeline run over all of its files one stage at a time. Nothing touches the
/// filesystem: a stage that would only notes its effect on the files that reach it.
///
/// What the stages share lives only for this one batch, so files of different units never share numbers
/// or fields.
pub fn plan_batch(
    pipelines: &[(String, Pipeline)],
    records: Vec<Record>,
    target: &dyn Target,
) -> Vec<Judged> {
    let mut context = Context::new(target);
    let mut waiting = records;
    let mut judged = Vec::new();

    for (index, (name, pipeline)) in pipelines.iter().enumerate() {
        let (filters, rest) = pipeline.split_at_claim();
        let (claimed, unclaimed): (Vec<Record>, Vec<Record>) = waiting
            .into_iter()
            .partition(|record| filters.iter().all(|filter| filter.accepts(record)));
        waiting = unclaimed;

        let files = rest
            .iter()
            .fold(Batch::new(claimed), |batch, stage| {
                stage.run(batch, &mut context)
            })
            .into_files();

        context.remember(files.iter().map(|file| Earlier {
            pipeline: name.clone(),
            origin: file.origin.clone(),
            planned: file.flow.as_ref().ok().cloned(),
        }));
        judged.extend(files.into_iter().map(|file| Judged {
            origin: file.origin,
            pipeline: Some(index),
            verdict: Verdict::of(file.flow),
            effects: file.effects,
        }));
    }

    judged.extend(waiting.into_iter().map(|record| Judged {
        origin: record.origin().to_path_buf(),
        pipeline: None,
        verdict: Verdict::Unclaimed,
        effects: Vec::new(),
    }));
    judged.sort_by(|a, b| a.origin.cmp(&b.origin));
    judged
}
