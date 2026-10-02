//! What the playground page reaches through WebAssembly: a configuration checked, read and written as
//! the CLI reads it, and a watch simulated over a virtual tree by the same processing the CLI runs.
//! The page holds all state, so each call is answered from what it is handed.

#[cfg(target_arch = "wasm32")]
mod bindings;
mod tree;

use std::collections::BTreeMap;
use std::path::PathBuf;

pub use auto_renamer_core::Declaration;
use auto_renamer_core::{
    Config, Declared, FolderConfig, Processed, Renames, Step, Value, What, process_batch_observed,
    replaced_pipelines,
};
use serde::{Deserialize, Serialize};
pub use toml::Table;

use tree::VirtualTree;

/// Which configuration a text is: the global one or a folder configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Configuration {
    Global,
    Folder,
}

/// Reads a configuration as the CLI reads it, answering its warnings, or why it is refused.
pub fn check(kind: Configuration, text: &str) -> Result<Vec<String>, String> {
    match kind {
        Configuration::Global => Config::parse(text).map(|config| config.warnings()),
        Configuration::Folder => FolderConfig::parse(text).map(|_| Vec::new()),
    }
    .map_err(|error| error.to_string())
}

/// The table a configuration text declares, for the page to edit.
pub fn read(text: &str) -> Result<Table, String> {
    text.parse()
        .map_err(|error: toml::de::Error| error.to_string())
}

/// The configuration text of a table, without comments or the layout it was read with.
pub fn render(table: &Table) -> String {
    table.to_string()
}

/// How each stage is declared, for the page to offer the stages and make their forms.
pub fn stages() -> &'static [Declaration] {
    Declared::declarations()
}

/// One file or folder of the virtual tree. A file carries its modification time, in milliseconds since
/// the Unix epoch, and its text, which only a folder configuration is read for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub path: String,
    #[serde(default)]
    pub folder: bool,
    #[serde(default)]
    pub modified: i64,
    #[serde(default)]
    pub text: String,
}

/// What became of one file of a simulated batch: `what` names it, `to` is where the file went or would
/// go, and `reason` says why it was skipped, refused or failed. `steps` are how its pipeline planned it,
/// and `replaced` says a folder configuration replaced that pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Outcome {
    pub origin: String,
    pub what: &'static str,
    pub to: Option<String>,
    pub reason: Option<String>,
    pub steps: Vec<SimulatedStep>,
    pub replaced: bool,
}

/// One step of a file's planning: the pipeline, the stage by its place and name or none for the claim, and either
/// the plan and fields the file went on with or why it stopped there.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SimulatedStep {
    pub pipeline: String,
    pub stage: Option<usize>,
    pub name: Option<&'static str>,
    pub plan: Option<String>,
    pub fields: BTreeMap<String, String>,
    pub stop: Option<String>,
}

impl SimulatedStep {
    fn of(step: &Step) -> SimulatedStep {
        let (plan, fields, stop) = match step.flow {
            Ok(record) => (
                Some(record.plan().display().to_string()),
                record
                    .fields()
                    .map(|(name, value)| (name.to_string(), shown(value)))
                    .collect(),
                None,
            ),
            Err(stop) => (None, BTreeMap::new(), Some(stop.reason.clone())),
        };
        SimulatedStep {
            pipeline: step.pipeline.to_string(),
            stage: step.stage.map(|(place, _)| place),
            name: step.stage.map(|(_, name)| name),
            plan,
            fields,
            stop,
        }
    }
}

/// A field as the page shows it: text as it is, a number in digits, a date in RFC 3339.
fn shown(value: &Value) -> String {
    match value {
        Value::Text(text) => text.clone(),
        Value::Number(number) => number.to_string(),
        Value::Date(date) => date.to_rfc3339(),
    }
}

/// What a simulation reported, and the virtual tree it left.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Simulation {
    pub outcomes: Vec<Outcome>,
    pub entries: Vec<Entry>,
}

/// Runs a watch of a configuration over a virtual tree: the files of its source are grouped into units as
/// the watcher groups them, and each unit is processed as one batch.
pub fn simulate(config: &str, watch: &str, entries: Vec<Entry>) -> Result<Simulation, String> {
    let config = Config::parse(config).map_err(|error| error.to_string())?;
    let watch = config
        .watches()
        .iter()
        .find(|candidate| candidate.name == watch)
        .ok_or_else(|| format!("the configuration has no watch `{watch}`"))?;
    let tree = VirtualTree::new(entries);

    let mut units: BTreeMap<PathBuf, Vec<PathBuf>> = BTreeMap::new();
    for file in tree.files_under(&watch.source) {
        units.entry(watch.unit.of(&file)).or_default().push(file);
    }
    let mut renames = Renames::new();
    let mut outcomes = Vec::new();
    for (unit, files) in units {
        let replaced = replaced_pipelines(&tree, watch, &unit);
        let mut steps: BTreeMap<PathBuf, Vec<SimulatedStep>> = BTreeMap::new();
        let processed =
            process_batch_observed(&tree, watch, &unit, &files, &mut renames, &mut |step| {
                steps
                    .entry(step.origin.to_path_buf())
                    .or_default()
                    .push(SimulatedStep::of(&step));
            });
        outcomes.extend(processed.into_iter().map(|entry| {
            let steps = steps.remove(&entry.origin).unwrap_or_default();
            let replaced = steps
                .first()
                .is_some_and(|step| replaced.contains(&step.pipeline));
            Outcome {
                steps,
                replaced,
                ..outcome(entry)
            }
        }));
    }
    Ok(Simulation {
        outcomes,
        entries: tree.into_entries(),
    })
}

fn outcome(entry: Processed) -> Outcome {
    let shown = |path: PathBuf| Some(path.display().to_string());
    let (what, to, reason) = match entry.what {
        What::Moved(to) => ("moved", shown(to), None),
        What::MovedThenFailed { to, reason } => ("moved then failed", shown(to), Some(reason)),
        What::Previewed(to) => ("previewed", shown(to), None),
        What::Unchanged => ("unchanged", None, None),
        What::Skipped(reason) => ("skipped", None, Some(reason.to_string())),
        What::Refused(reason) => ("refused", None, Some(reason)),
        What::Unclaimed => ("unclaimed", None, None),
        What::LeftTooLarge => ("left too large", None, None),
    };
    Outcome {
        origin: entry.origin.display().to_string(),
        what,
        to,
        reason,
        steps: Vec::new(),
        replaced: false,
    }
}
