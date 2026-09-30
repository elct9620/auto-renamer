use std::path::PathBuf;

use crate::context::Context;
use crate::pipeline::Pipeline;
use crate::record::Record;
use crate::stages::{Batch, Flow, Rejection, Stage, Stop};

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Judged {
    pub origin: PathBuf,
    /// Where the pipeline that claimed the file stands in the list planned with, or none when no pipeline did.
    pub pipeline: Option<usize>,
    pub verdict: Verdict,
}

/// Plans a whole batch through the pipelines of a watch, in the order they are listed.
///
/// Files are taken in the order of their paths. Each is claimed by the first pipeline whose leading filters
/// accept it, and the stages of a pipeline run over all of its files one stage at a time, up to the first stage
/// that touches the filesystem.
pub fn plan_batch(
    pipelines: &[(String, Pipeline)],
    records: Vec<Record>,
    context: &mut Context,
) -> Vec<Judged> {
    let mut batch = Batch::new(records);

    for (name, pipeline) in pipelines {
        let (filters, rest) = pipeline.split_at_claim();
        batch.claim(name, |record| {
            filters
                .iter()
                .all(|filter| filter.apply(record.clone(), context).is_ok())
        });
        for stage in rest.iter().take_while(|stage| !stage.is_effect()) {
            match stage {
                Stage::Rank(rank) => {
                    let mut flows = rank.run(batch.live().cloned().collect()).into_iter();
                    batch.each(|_| flows.next().expect("a rank answers once for each record"));
                }
                Stage::Take(take) => {
                    let earlier = batch.earlier();
                    batch.each(|record| take.run(record, &earlier));
                }
                _ => batch.each(|record| stage.apply(record, context)),
            }
        }
    }

    batch
        .into_files()
        .map(|(origin, claimed)| {
            let (pipeline, verdict) = match claimed {
                None => (None, Verdict::Unclaimed),
                Some((pipeline, flow)) => (Some(pipeline), Verdict::of(flow)),
            };
            Judged {
                origin,
                pipeline,
                verdict,
            }
        })
        .collect()
}
