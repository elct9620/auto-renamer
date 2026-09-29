//! What the operator configured, and what a folder may say of the files in it.

mod unit;

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use toml::{Table, Value as Toml};

use crate::pipeline::{Pipeline, PipelineError};
use crate::reader::{Reader, Scope};
use crate::record::Value;
pub use unit::Unit;

/// The name of a folder configuration, which is read as settings and never renamed.
pub(crate) const FOLDER_CONFIG: &str = "auto-renamer.toml";

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
        let mut root = Reader::new("configuration".to_string(), document);

        let default = match root.table("default")? {
            Some(table) => read_settings(Reader::new("default".to_string(), table))?,
            None => Settings::default(),
        };
        let definitions = read_pipelines(root.table("pipeline")?.unwrap_or_default())?;
        let declared = root.table("watch")?.unwrap_or_default();
        root.finish()?;

        let mut watches = Vec::new();
        for (name, value) in declared {
            let scope = format!("watch.{name}");
            let mut reader = table_reader(scope.clone(), &value)?;
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

        let config = Config { watches };
        check_apart(&config.watches, None, |path| path.to_path_buf())?;
        Ok(config)
    }

    /// The watches in the order of their names.
    pub fn watches(&self) -> &[Watch] {
        &self.watches
    }

    /// Checks where the configuration file, the sources and the targets lie against one another, with
    /// each path taken to where `resolve` says it really is, such as past the links on its way.
    pub fn check_paths(
        &self,
        config_file: &Path,
        resolve: impl Fn(&Path) -> PathBuf,
    ) -> Result<(), ConfigError> {
        check_apart(&self.watches, Some(config_file), resolve)
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
        let mut reader = Reader::new("folder configuration".to_string(), document);

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

fn read_settings(mut reader: Reader<String>) -> Result<Settings, ConfigError> {
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

fn read_vars(reader: &mut Reader<String>) -> Result<BTreeMap<String, Value>, ConfigError> {
    let vars = reader.table("vars")?.unwrap_or_default();
    Reader::new("vars".to_string(), vars).into_values()
}

fn read_unit(reader: &mut Reader<String>) -> Result<Option<Unit>, ConfigError> {
    match reader.take("unit") {
        None => Ok(None),
        Some(Toml::String(kind)) if kind == "directory" => Ok(Some(Unit::Directory)),
        Some(Toml::String(kind)) if kind == "source" => Ok(Some(Unit::Source)),
        Some(Toml::Table(table)) => {
            let mut roots = Reader::new("unit".to_string(), table);
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
        let mut reader = table_reader(format!("pipeline.{name}"), &value)?;
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

/// Watches must not overlap, and no target may lie among the sources, or files would be planned again
/// after they are moved; the configuration file may not lie in a source either, or it would be taken for
/// a file to process. Each path is compared where `resolve` puts it.
fn check_apart(
    watches: &[Watch],
    config_file: Option<&Path>,
    resolve: impl Fn(&Path) -> PathBuf,
) -> Result<(), ConfigError> {
    let config_file = config_file.map(&resolve);
    let placed: Vec<(&Watch, PathBuf, Option<PathBuf>)> = watches
        .iter()
        .map(|watch| {
            let target = watch.target.as_deref().map(&resolve);
            (watch, resolve(&watch.source), target)
        })
        .collect();
    let overlap = |a: &Path, b: &Path| a.starts_with(b) || b.starts_with(a);

    for (index, (watch, source, target)) in placed.iter().enumerate() {
        let scope = format!("watch.{}", watch.name);
        if let Some(target) = target
            && target.starts_with(source)
        {
            return Err(invalid(
                &scope,
                "target",
                "may not be inside its own source",
            ));
        }
        if let Some(file) = &config_file
            && file.starts_with(source)
        {
            return Err(invalid(
                &scope,
                "source",
                format!("holds the configuration file `{}`", file.display()),
            ));
        }
        for (other, other_source, _) in &placed[index + 1..] {
            if overlap(source, other_source) {
                return Err(invalid(
                    &format!("watch.{}", other.name),
                    "source",
                    format!("overlaps the source of `{}`", watch.name),
                ));
            }
        }
        for (other, other_source, _) in placed.iter().filter(|(other, ..)| other.name != watch.name)
        {
            if let Some(target) = target
                && overlap(target, other_source)
            {
                return Err(invalid(
                    &format!("watch.{}", other.name),
                    "source",
                    format!("overlaps the target of `{}`", watch.name),
                ));
            }
        }
    }
    Ok(())
}

/// The batch limit a table says, which may not be more than the ceiling.
fn read_batch_max(reader: &mut Reader<String>) -> Result<Option<usize>, ConfigError> {
    match reader.positive("batch_max")? {
        Some(limit) if limit > MAX_BATCH_FILES => Err(reader.invalid(
            "batch_max",
            format!("may not be more than {MAX_BATCH_FILES}"),
        )),
        limit => Ok(limit),
    }
}

impl Scope for String {
    type Error = ConfigError;

    fn invalid(&self, key: &str, reason: String) -> ConfigError {
        invalid(self, key, reason)
    }

    fn unknown(&self, key: &str) -> ConfigError {
        ConfigError::Unknown {
            scope: self.clone(),
            key: key.to_string(),
        }
    }
}

/// The reader of a value that has to be a table, such as one watch or one pipeline.
fn table_reader(scope: String, value: &Toml) -> Result<Reader<String>, ConfigError> {
    match value {
        Toml::Table(table) => Ok(Reader::new(scope, table.clone())),
        _ => Err(invalid(&scope, &scope, "must be a table")),
    }
}

fn invalid(scope: &str, key: &str, reason: impl Into<String>) -> ConfigError {
    ConfigError::Invalid {
        scope: scope.to_string(),
        key: key.to_string(),
        reason: reason.into(),
    }
}
