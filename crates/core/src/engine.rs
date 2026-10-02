use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::context::{Context, Earlier, Target};
use crate::pipeline::Pipeline;
use crate::record::Record;
use crate::stages::{Batch, Flow, Stop};

/// What a batch made of one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Planned(Record),
    Unclaimed,
    Rejected(Stop),
}

impl Verdict {
    /// The verdict on a file its pipeline ran to the end, or stopped.
    fn of(flow: Flow) -> Verdict {
        match flow {
            Ok(record) => Verdict::Planned(record),
            Err(stop) => Verdict::Rejected(stop),
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
}

/// What one file holds at one point of its planning.
#[derive(Debug)]
pub struct Step<'a> {
    pub origin: &'a Path,
    pub pipeline: &'a str,
    /// The stage just run, by its place among the pipeline's stages and its name; none for the claim by its
    /// leading filters.
    pub stage: Option<(usize, &'static str)>,
    pub flow: &'a Flow,
}

/// What a pipeline decided for one file it claimed, apart from the record the file left with the context.
struct Outcome {
    origin: PathBuf,
    pipeline: usize,
    stopped: Result<(), Stop>,
}

/// Plans a whole batch through pipelines that all look into one target, in the order they are listed.
///
/// Files are taken in the order of their paths, each path at most once in a batch. Each is claimed by the
/// first pipeline whose leading filters accept it, and the stages of a pipeline run over all of its files
/// one stage at a time. Nothing touches the filesystem.
///
/// What the stages share lives only for this one batch, so files of different units never share numbers
/// or fields.
pub fn plan_batch(
    pipelines: &[(String, Pipeline)],
    records: Vec<Record>,
    target: &dyn Target,
) -> Vec<Judged> {
    plan(&into_one(pipelines, target), records, None)
}

/// Plans a batch as [`plan_batch`] does, telling `observe` each step of every file as it happens: the claim,
/// then every stage the file reaches.
pub fn plan_batch_observed(
    pipelines: &[(String, Pipeline)],
    records: Vec<Record>,
    target: &dyn Target,
    observe: &mut dyn FnMut(Step),
) -> Vec<Judged> {
    plan(&into_one(pipelines, target), records, Some(observe))
}

/// Pipelines that all look into the same target.
fn into_one<'a>(
    pipelines: &'a [(String, Pipeline)],
    target: &'a dyn Target,
) -> Vec<(&'a str, &'a Pipeline, &'a dyn Target)> {
    pipelines
        .iter()
        .map(|(name, pipeline)| (name.as_str(), pipeline, target))
        .collect()
}

/// Plans a batch through pipelines, each looking into the target of its route, telling the steps only when
/// someone observes them, so planning without an observer keeps nothing for them.
pub(crate) fn plan(
    pipelines: &[(&str, &Pipeline, &dyn Target)],
    records: Vec<Record>,
    mut observe: Option<&mut dyn FnMut(Step)>,
) -> Vec<Judged> {
    let Some(&(_, _, first)) = pipelines.first() else {
        let mut judged = unclaimed(records);
        judged.sort_by(|a, b| a.origin.cmp(&b.origin));
        return judged;
    };
    let mut context = Context::new(first);
    let mut waiting = records;
    let mut outcomes = Vec::new();

    for (index, &(name, pipeline, target)) in pipelines.iter().enumerate() {
        context.enter(target);
        let (filters, rest) = pipeline.split_at_claim();
        let (claimed, unclaimed): (Vec<Record>, Vec<Record>) = waiting
            .into_iter()
            .partition(|record| filters.iter().all(|filter| filter.accepts(record)));
        waiting = unclaimed;

        // A file is told up to the step that stopped it, and no more after.
        let mut stopped = HashSet::new();
        let mut tell = |batch: &Batch,
                        stage: Option<(usize, &'static str)>,
                        observe: &mut Option<&mut dyn FnMut(Step)>| {
            let Some(observe) = observe else { return };
            for (origin, flow) in batch.files() {
                if stopped.contains(origin) {
                    continue;
                }
                observe(Step {
                    origin,
                    pipeline: name,
                    stage,
                    flow,
                });
                if flow.is_err() {
                    stopped.insert(origin.to_path_buf());
                }
            }
        };
        let claimed = Batch::new(claimed);
        tell(&claimed, None, &mut observe);
        let staged = rest
            .iter()
            .enumerate()
            .fold(claimed, |batch, (index, stage)| {
                let batch = stage.run(batch, &mut context);
                tell(
                    &batch,
                    Some((filters.len() + index, stage.name())),
                    &mut observe,
                );
                batch
            });
        let placed = staged.each(|record| take_place(record, &mut context));
        tell(
            &placed,
            Some((filters.len() + rest.len(), "move")),
            &mut observe,
        );
        let files = placed.into_files();

        for file in files {
            let (planned, stopped) = match file.flow {
                Ok(record) => (Some(record), Ok(())),
                Err(stop) => (None, Err(stop)),
            };
            outcomes.push(Outcome {
                origin: file.origin.clone(),
                pipeline: index,
                stopped,
            });
            context.remember(Earlier {
                pipeline: name.to_string(),
                origin: file.origin,
                planned,
            });
        }
    }

    // The records planned went on to the context for the pipelines after theirs; they come back by origin.
    let mut planned: HashMap<PathBuf, Record> = context
        .into_earlier()
        .into_iter()
        .filter_map(|earlier| Some((earlier.origin, earlier.planned?)))
        .collect();
    let mut judged: Vec<Judged> = outcomes
        .into_iter()
        .map(|outcome| {
            let flow = outcome.stopped.map(|()| {
                planned
                    .remove(&outcome.origin)
                    .expect("a planned file leaves its record with the context")
            });
            Judged {
                origin: outcome.origin,
                pipeline: Some(outcome.pipeline),
                verdict: Verdict::of(flow),
            }
        })
        .collect();

    judged.extend(unclaimed(waiting));
    judged.sort_by(|a, b| a.origin.cmp(&b.origin));
    judged
}

/// Claims the plan of a file in its target, refusing it when the target holds a file there or a file
/// before it in the batch was planned there. A file left where it is takes no place but its own.
fn take_place(record: Record, context: &mut Context) -> Flow {
    let free = context.claim(record.plan());
    if free || record.plan() == record.origin() {
        Ok(record)
    } else {
        Err(Stop::rejected(
            "move",
            format!("`{}` is already taken", record.plan().display()),
        ))
    }
}

fn unclaimed(records: Vec<Record>) -> Vec<Judged> {
    records
        .into_iter()
        .map(|record| Judged {
            origin: record.origin().to_path_buf(),
            pipeline: None,
            verdict: Verdict::Unclaimed,
        })
        .collect()
}
