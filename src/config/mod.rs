//! What the operator configured, and what a folder may say of the files in it.

mod reader;
mod unit;

use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use toml::{Table, Value as Toml};

use crate::pipeline::{Pipeline, PipelineError};
use crate::record::Value;
use reader::{Reader, invalid};
pub use unit::Unit;

/// The most a folder configuration may hold, because it comes from downloaded content.
pub(crate) const MAX_FOLDER_CONFIG_BYTES: usize = 64 * 1024;

/// The most files a batch may hold, whatever the configuration says, so that a unit cannot grow without bound.
pub const MAX_BATCH_FILES: usize = 100_000;
const DEFAULT_BATCH_MAX: usize = 1000;
const DEFAULT_BATCH_WINDOW: Duration = Duration::from_secs(5 * 60);
const DEFAULT_BATCH_MAX_WAIT: Duration = Duration::from_secs(30 * 60);

/// Why a configuration was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    Syntax(String),
    Unknown {
        scope: String,
        key: String,
    },
    Invalid {
        scope: String,
        key: String,
        reason: String,
    },
    Pipeline {
        name: String,
        error: PipelineError,
    },
    TooLarge,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Syntax(message) => write!(f, "not valid TOML: {message}"),
            ConfigError::Unknown { scope, key } => {
                write!(f, "{scope}: `{key}` is not something it can say")
            }
            ConfigError::Invalid { scope, key, reason } => write!(f, "{scope}: `{key}` {reason}"),
            ConfigError::Pipeline { name, error } => write!(f, "pipeline `{name}`: {error}"),
            ConfigError::TooLarge => write!(
                f,
                "a folder configuration may not be larger than {MAX_FOLDER_CONFIG_BYTES} bytes"
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

/// What the operator configured: the watches, each with its pipelines and settings.
#[derive(Debug, Clone)]
pub struct Config {
    watches: Vec<Watch>,
}

/// One watched folder, with where its files go and how they are grouped and named.
#[derive(Debug, Clone)]
pub struct Watch {
    pub name: String,
    pub source: PathBuf,
    pub target: Option<PathBuf>,
    pub unit: Unit,
    pub vars: BTreeMap<String, Value>,
    pub batch_window: Duration,
    pub batch_max_wait: Duration,
    pub batch_max: usize,
    pub dry_run: bool,
    pipeline_names: Vec<String>,
    definitions: BTreeMap<String, Pipeline>,
}

/// What a folder says of the files in it: variables, pipelines and a batch limit.
#[derive(Debug, Clone)]
pub struct FolderConfig {
    vars: BTreeMap<String, Value>,
    pipelines: BTreeMap<String, Pipeline>,
    batch_max: Option<usize>,
}

/// The settings a `[default]` and a watch have in common.
#[derive(Default)]
struct Settings {
    batch_window: Option<Duration>,
    batch_max_wait: Option<Duration>,
    batch_max: Option<usize>,
    pipelines: Option<Vec<String>>,
    vars: BTreeMap<String, Value>,
    unit: Option<Unit>,
    dry_run: Option<bool>,
}

impl Config {
    /// Reads the configuration, refusing everything that would leave a watch unable to run.
    pub fn parse(source: &str) -> Result<Config, ConfigError> {
        let document: Table = source
            .parse()
            .map_err(|error: toml::de::Error| ConfigError::Syntax(error.to_string()))?;
        let mut root = Reader::new("configuration", document);

        let default = match root.table("default")? {
            Some(table) => read_settings(Reader::new("default", table))?,
            None => Settings::default(),
        };
        let definitions = read_pipelines(root.table("pipeline")?.unwrap_or_default())?;
        let declared = root.table("watch")?.unwrap_or_default();
        root.finish()?;

        let mut watches = Vec::new();
        for (name, value) in declared {
            let scope = format!("watch.{name}");
            let mut reader = Reader::from_value(scope.clone(), &value)?;
            let source = reader
                .absolute_path("source")?
                .ok_or_else(|| reader.invalid("source", "is required"))?;
            let target = reader.absolute_path("target")?;
            let settings = read_settings(reader)?;
            watches.push(build_watch(
                name,
                source,
                target,
                settings,
                &default,
                &definitions,
            )?);
        }

        check_apart(&watches)?;
        Ok(Config { watches })
    }

    /// The watches in the order of their names.
    pub fn watches(&self) -> &[Watch] {
        &self.watches
    }

    /// What is legal but probably not what was meant, such as a pipeline that never moves anything.
    pub fn warnings(&self) -> Vec<String> {
        self.watches
            .iter()
            .flat_map(|watch| {
                watch
                    .pipelines()
                    .into_iter()
                    .filter(|(_, pipeline)| !pipeline.has_effect())
                    .map(|(name, _)| {
                        format!(
                            "watch `{}`: pipeline `{name}` has no effect stage, so it only previews",
                            watch.name
                        )
                    })
            })
            .collect()
    }
}

impl Watch {
    /// The pipelines of the watch, in the order it lists them.
    pub fn pipelines(&self) -> Vec<(String, Pipeline)> {
        self.pipeline_names
            .iter()
            .filter_map(|name| {
                self.definitions
                    .get(name)
                    .map(|pipeline| (name.clone(), pipeline.clone()))
            })
            .collect()
    }

    /// The watch as it is for the files of a folder that carries configurations of its own, from the farthest to the nearest.
    pub fn under(&self, folders: &[FolderConfig]) -> Watch {
        let mut watch = self.clone();
        for folder in folders {
            watch.vars.extend(folder.vars.clone());
            for (name, pipeline) in &folder.pipelines {
                if watch.pipeline_names.contains(name) {
                    watch.definitions.insert(name.clone(), pipeline.clone());
                }
            }
            watch.batch_max = folder.batch_max.unwrap_or(watch.batch_max);
        }
        watch
    }
}

impl FolderConfig {
    /// Reads a folder configuration, refusing what a folder may not decide or what is too large.
    pub fn parse(source: &str) -> Result<FolderConfig, ConfigError> {
        if source.len() > MAX_FOLDER_CONFIG_BYTES {
            return Err(ConfigError::TooLarge);
        }
        let document: Table = source
            .parse()
            .map_err(|error: toml::de::Error| ConfigError::Syntax(error.to_string()))?;
        let mut reader = Reader::new("folder configuration", document);

        let vars = read_vars(&mut reader)?;
        let pipelines = read_pipelines(reader.table("pipeline")?.unwrap_or_default())?;
        let batch_max = read_batch_max(&mut reader)?;
        reader.finish()?;
        Ok(FolderConfig {
            vars,
            pipelines,
            batch_max,
        })
    }
}

fn read_settings(mut reader: Reader) -> Result<Settings, ConfigError> {
    let batch_window = reader.duration("batch_window")?;
    let batch_max_wait = reader.duration("batch_max_wait")?;
    let batch_max = read_batch_max(&mut reader)?;
    let pipelines = reader.strings("pipelines")?;
    let dry_run = reader.boolean("dry_run")?;
    let vars = read_vars(&mut reader)?;
    let unit = read_unit(&mut reader)?;
    reader.finish()?;
    Ok(Settings {
        batch_window,
        batch_max_wait,
        batch_max,
        pipelines,
        vars,
        unit,
        dry_run,
    })
}

fn read_vars(reader: &mut Reader) -> Result<BTreeMap<String, Value>, ConfigError> {
    let mut vars = BTreeMap::new();
    for (name, value) in reader.table("vars")?.unwrap_or_default() {
        let value = match value {
            Toml::String(text) => Value::Text(text),
            Toml::Integer(number) if number >= 0 => Value::Number(number as u64),
            _ => return Err(invalid("vars", &name, "must be text or a whole number")),
        };
        vars.insert(name, value);
    }
    Ok(vars)
}

fn read_unit(reader: &mut Reader) -> Result<Option<Unit>, ConfigError> {
    match reader.take("unit") {
        None => Ok(None),
        Some(Toml::String(kind)) if kind == "directory" => Ok(Some(Unit::Directory)),
        Some(Toml::String(kind)) if kind == "source" => Ok(Some(Unit::Source)),
        Some(Toml::Table(table)) => {
            let mut roots = Reader::new("unit", table);
            let patterns = roots
                .strings("root")?
                .ok_or_else(|| reader.invalid("unit", "needs `root`, a list of folder patterns"))?;
            roots.finish()?;
            Unit::roots(&patterns)
                .map(Some)
                .map_err(|error| reader.invalid("unit", error.to_string()))
        }
        Some(_) => Err(reader.invalid(
            "unit",
            "must be \"directory\", \"source\" or { root = [...] }",
        )),
    }
}

fn read_pipelines(declared: Table) -> Result<BTreeMap<String, Pipeline>, ConfigError> {
    let mut pipelines = BTreeMap::new();
    for (name, value) in declared {
        let mut reader = Reader::from_value(format!("pipeline.{name}"), &value)?;
        let stages = match reader.take("stages") {
            Some(Toml::Array(stages)) => stages,
            _ => return Err(reader.invalid("stages", "must be a list of stages")),
        };
        reader.finish()?;
        let pipeline = Pipeline::declare(&stages).map_err(|error| ConfigError::Pipeline {
            name: name.clone(),
            error,
        })?;
        pipelines.insert(name, pipeline);
    }
    Ok(pipelines)
}

fn build_watch(
    name: String,
    source: PathBuf,
    target: Option<PathBuf>,
    settings: Settings,
    default: &Settings,
    definitions: &BTreeMap<String, Pipeline>,
) -> Result<Watch, ConfigError> {
    let scope = format!("watch.{name}");
    let batch_window = settings
        .batch_window
        .or(default.batch_window)
        .unwrap_or(DEFAULT_BATCH_WINDOW);
    let batch_max_wait = settings
        .batch_max_wait
        .or(default.batch_max_wait)
        .unwrap_or(DEFAULT_BATCH_MAX_WAIT);
    if batch_max_wait < batch_window {
        return Err(invalid(
            &scope,
            "batch_max_wait",
            "may not be shorter than `batch_window`",
        ));
    }
    if let Some(target) = &target
        && target.starts_with(&source)
    {
        return Err(invalid(
            &scope,
            "target",
            "may not be inside its own source",
        ));
    }

    let pipeline_names = settings
        .pipelines
        .or_else(|| default.pipelines.clone())
        .unwrap_or_default();
    if let Some(missing) = pipeline_names
        .iter()
        .find(|listed| !definitions.contains_key(*listed))
    {
        return Err(invalid(
            &scope,
            "pipelines",
            format!("`{missing}` is not defined"),
        ));
    }

    let mut vars = default.vars.clone();
    vars.extend(settings.vars);
    Ok(Watch {
        name,
        source,
        target,
        unit: settings
            .unit
            .or_else(|| default.unit.clone())
            .unwrap_or(Unit::Directory),
        vars,
        batch_window,
        batch_max_wait,
        batch_max: settings
            .batch_max
            .or(default.batch_max)
            .unwrap_or(DEFAULT_BATCH_MAX),
        dry_run: settings.dry_run.or(default.dry_run).unwrap_or(false),
        pipeline_names,
        definitions: definitions.clone(),
    })
}

/// Watches must not overlap, and no target may lie among the sources, or files would be planned again after they are moved.
fn check_apart(watches: &[Watch]) -> Result<(), ConfigError> {
    for (index, watch) in watches.iter().enumerate() {
        for other in &watches[index + 1..] {
            let scope = format!("watch.{}", other.name);
            if watch.source.starts_with(&other.source) || other.source.starts_with(&watch.source) {
                return Err(invalid(
                    &scope,
                    "source",
                    format!("overlaps the source of `{}`", watch.name),
                ));
            }
        }
        for other in watches.iter().filter(|other| other.name != watch.name) {
            if let Some(target) = &watch.target
                && (target.starts_with(&other.source) || other.source.starts_with(target))
            {
                let scope = format!("watch.{}", other.name);
                return Err(invalid(
                    &scope,
                    "source",
                    format!("overlaps the target of `{}`", watch.name),
                ));
            }
        }
    }
    Ok(())
}

/// The batch limit a table says, which may not be more than the ceiling.
fn read_batch_max(reader: &mut Reader) -> Result<Option<usize>, ConfigError> {
    match reader.positive("batch_max")? {
        Some(limit) if limit > MAX_BATCH_FILES => Err(reader.invalid(
            "batch_max",
            format!("may not be more than {MAX_BATCH_FILES}"),
        )),
        limit => Ok(limit),
    }
}
