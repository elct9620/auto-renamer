use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::context::{Context, Earlier, Target};
use crate::pipeline::Pipeline;
use crate::record::{Record, Value};
use crate::stages::{Batch, File, Flow, Stop};

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

/// One pipeline as a batch is planned through it: its name, its stages, the target it moves into, and
/// which files it takes.
pub(crate) struct Planned<'a> {
    pub name: &'a str,
    pub pipeline: &'a Pipeline,
    pub target: &'a dyn Target,
    pub takes: Takes,
}

/// The files a pipeline takes: those no pipeline before it claimed, or those the pipeline at an earlier
/// place refused, as a rejected route takes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Takes {
    Fresh,
    RefusedBy(usize),
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
    plan(&into_one(pipelines, target), records, &[], None)
}

/// Plans a batch as [`plan_batch`] does, telling `observe` each step of every file as it happens: the claim,
/// then every stage the file reaches.
pub fn plan_batch_observed(
    pipelines: &[(String, Pipeline)],
    records: Vec<Record>,
    target: &dyn Target,
    observe: &mut dyn FnMut(Step),
) -> Vec<Judged> {
    plan(&into_one(pipelines, target), records, &[], Some(observe))
}

/// Pipelines that all look into the same target, each taking the files no pipeline before it claimed.
fn into_one<'a>(pipelines: &'a [(String, Pipeline)], target: &'a dyn Target) -> Vec<Planned<'a>> {
    pipelines
        .iter()
        .map(|(name, pipeline)| Planned {
            name,
            pipeline,
            target,
            takes: Takes::Fresh,
        })
        .collect()
}

/// Plans a batch through pipelines, each looking into its own target, telling the steps only when someone
/// observes them, so planning without an observer keeps nothing for them.
///
/// Once every pipeline taking fresh files has planned, a file planned in the group of a refused file,
/// whose `group` fields all agree with it, is refused too. A pipeline that takes what an earlier one refused
/// runs after that, on the refused records restarted from their origins; what it refuses in turn stays
/// refused.
pub(crate) fn plan(
    pipelines: &[Planned],
    records: Vec<Record>,
    group: &[String],
    mut observe: Option<&mut dyn FnMut(Step)>,
) -> Vec<Judged> {
    let Some(first) = pipelines.first() else {
        let mut judged = unclaimed(records);
        judged.sort_by(|a, b| a.origin.cmp(&b.origin));
        return judged;
    };
    let sends_on: HashSet<usize> = pipelines
        .iter()
        .filter_map(|planned| match planned.takes {
            Takes::RefusedBy(index) => Some(index),
            Takes::Fresh => None,
        })
        .collect();
    let mut context = Context::new(first.target);
    let mut waiting = records;
    let mut refused: HashMap<usize, Vec<Record>> = HashMap::new();
    let mut refused_groups: HashSet<Vec<String>> = HashSet::new();
    let mut outcomes: BTreeMap<PathBuf, Outcome> = BTreeMap::new();
    let mut grouped = group.is_empty();

    for (index, planned) in pipelines.iter().enumerate() {
        if !grouped && planned.takes != Takes::Fresh {
            refuse_groups(
                &mut context,
                &mut outcomes,
                &mut refused,
                &refused_groups,
                group,
                &sends_on,
            );
            grouped = true;
        }
        context.enter(planned.target);
        let offered = match planned.takes {
            Takes::Fresh => std::mem::take(&mut waiting),
            Takes::RefusedBy(earlier) => refused.remove(&earlier).unwrap_or_default(),
        };
        let (filters, _) = planned.pipeline.split_at_claim();
        let (claimed, passed): (Vec<Record>, Vec<Record>) = offered
            .into_iter()
            .partition(|record| filters.iter().all(|filter| filter.accepts(record)));
        if planned.takes == Takes::Fresh {
            waiting = passed;
        }

        let keeping = sends_on.contains(&index) || !group.is_empty();
        let files = plan_through(planned, claimed, keeping, &mut context, &mut observe);
        for file in files {
            let (record, stopped) = match file.flow {
                Ok(record) => (Some(record), Ok(())),
                Err(stop) => {
                    if let Some(kept) = file.stopped {
                        if let Some(key) = group_key(&kept, group) {
                            refused_groups.insert(key);
                        }
                        if sends_on.contains(&index) {
                            refused
                                .entry(index)
                                .or_default()
                                .push(kept.restarted(&stop.planned, &stop.stage));
                        }
                    }
                    (None, Err(stop))
                }
            };
            outcomes.insert(
                file.origin.clone(),
                Outcome {
                    origin: file.origin.clone(),
                    pipeline: index,
                    stopped,
                },
            );
            context.remember(Earlier {
                pipeline: planned.name.to_string(),
                origin: file.origin,
                planned: record,
            });
        }
    }

    if !grouped {
        refuse_groups(
            &mut context,
            &mut outcomes,
            &mut refused,
            &refused_groups,
            group,
            &sends_on,
        );
    }

    // The records planned went on to the context for the pipelines after theirs; they come back by origin.
    let mut planned: HashMap<PathBuf, Record> = context
        .into_earlier()
        .into_iter()
        .filter_map(|earlier| Some((earlier.origin, earlier.planned?)))
        .collect();
    let mut judged: Vec<Judged> = outcomes
        .into_values()
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

/// Refuses every planned file in the group of a refused one, handing it on to its pipeline's rejected route
/// when there is one.
fn refuse_groups(
    context: &mut Context,
    outcomes: &mut BTreeMap<PathBuf, Outcome>,
    refused: &mut HashMap<usize, Vec<Record>>,
    refused_groups: &HashSet<Vec<String>>,
    group: &[String],
    sends_on: &HashSet<usize>,
) {
    for earlier in context.earlier_mut() {
        let in_refused_group = earlier
            .planned
            .as_ref()
            .and_then(|record| group_key(record, group))
            .is_some_and(|key| refused_groups.contains(&key));
        if !in_refused_group {
            continue;
        }
        let Some(record) = earlier.planned.take() else {
            continue;
        };
        let Some(outcome) = outcomes.get_mut(&earlier.origin) else {
            continue;
        };
        let mut stop = Stop::rejected("group", "a file of its group was refused");
        stop.planned = record.plan().to_path_buf();
        if sends_on.contains(&outcome.pipeline) {
            refused
                .entry(outcome.pipeline)
                .or_default()
                .push(record.restarted(&stop.planned, &stop.stage));
        }
        outcome.stopped = Err(stop);
    }
}

/// What makes a record one of a group: the values of the `group` fields, or nothing when one is missing
/// or no fields make a group.
fn group_key(record: &Record, group: &[String]) -> Option<Vec<String>> {
    if group.is_empty() {
        return None;
    }
    group
        .iter()
        .map(|field| match record.field(field)? {
            Value::Text(text) => Some(text.clone()),
            Value::Number(number) => Some(number.to_string()),
            Value::Date(date) => Some(date.to_rfc3339()),
        })
        .collect()
}

/// Runs the stages of one pipeline over the files it claimed, then claims each plan in its target, telling
/// each step to whoever observes. A file is told up to the step that stopped it, and no more after.
fn plan_through(
    planned: &Planned,
    claimed: Vec<Record>,
    keeping: bool,
    context: &mut Context,
    observe: &mut Option<&mut dyn FnMut(Step)>,
) -> Vec<File> {
    let (filters, rest) = planned.pipeline.split_at_claim();
    let mut stopped = HashSet::new();
    let mut tell = |batch: &Batch, stage: Option<(usize, &'static str)>| {
        let Some(observe) = observe else { return };
        for (origin, flow) in batch.files() {
            if stopped.contains(origin) {
                continue;
            }
            observe(Step {
                origin,
                pipeline: planned.name,
                stage,
                flow,
            });
            if flow.is_err() {
                stopped.insert(origin.to_path_buf());
            }
        }
    };
    let mut claimed = Batch::new(claimed);
    if keeping {
        claimed = claimed.keeping_stopped();
    }
    tell(&claimed, None);
    let staged = rest
        .iter()
        .enumerate()
        .fold(claimed, |batch, (index, stage)| {
            let batch = stage.run(batch, context);
            tell(&batch, Some((filters.len() + index, stage.name())));
            batch
        });
    let placed = staged.each(|record| take_place(record, context));
    tell(&placed, Some((filters.len() + rest.len(), "move")));
    placed.into_files()
}

/// Claims the plan of a file in its target, refusing it when the target holds a file there or a file
/// before it in the batch was planned there. A file left where it is takes no place but its own.
fn take_place(record: Record, context: &mut Context) -> Flow {
    let free = context.claim(record.plan());
    if free || record.plan() == record.origin() {
        Ok(record)
    } else {
        Err(Stop::rejected("move", "the plan is already taken"))
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
