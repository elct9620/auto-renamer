//! What the operator configured, and what a folder may say of the files in it.

mod unit;

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use globset::Glob;
use toml::{Table, Value as Toml};

use crate::effects::{Cleanup, Effect};
use crate::pipeline::{Pipeline, PipelineError};
use crate::reader::{Reader, Scope};
use crate::record::Value;
pub use unit::Unit;

/// The name of a folder configuration, which is read as settings and never renamed.
pub const FOLDER_CONFIG: &str = "auto-renamer.toml";

/// The most a folder configuration may hold, because it comes from downloaded content.
pub(crate) const MAX_FOLDER_CONFIG_BYTES: usize = 64 * 1024;

/// The most files a batch may hold, whatever the configuration says, so that a unit cannot grow without bound.
pub const MAX_BATCH_FILES: usize = 100_000;
const DEFAULT_MAX_FILES: usize = 1000;
const DEFAULT_QUIET: Duration = Duration::from_secs(5 * 60);
const DEFAULT_MAX_WAIT: Duration = Duration::from_secs(30 * 60);

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

/// What the operator configured: the watches, each with its routes and settings, and the targets they move to.
#[derive(Debug, Clone)]
pub struct Config {
    watches: Vec<Watch>,
    targets: BTreeMap<String, PathBuf>,
}

/// One watched folder, with how its files are grouped and the routes that name and move them.
#[derive(Debug, Clone)]
pub struct Watch {
    pub name: String,
    pub source: PathBuf,
    pub unit: Unit,
    pub vars: BTreeMap<String, Value>,
    pub quiet: Duration,
    pub max_wait: Duration,
    pub max_files: usize,
    pub dry_run: bool,
    routes: Vec<Route>,
    definitions: BTreeMap<String, Pipeline>,
}

/// One entry of a watch's routes: the pipeline that claims and names files, the target root it moves them
/// to, or none to rename them in place, and the folders its cleanup keeps when it cleans up.
#[derive(Debug, Clone)]
pub struct Route {
    pub pipeline: String,
    pub target: Option<PathBuf>,
    pub cleanup: Option<Cleanup>,
}

impl Route {
    /// What the route does to a file it planned: the move, then the cleanup if it has one.
    pub fn effects(&self) -> Vec<Effect> {
        let mut effects = vec![Effect::Move];
        effects.extend(self.cleanup.clone().map(Effect::Cleanup));
        effects
    }
}

/// A route as it is written, before its pipeline and target are looked up.
#[derive(Debug, Clone)]
struct Written {
    pipeline: String,
    target: Option<String>,
    cleanup: Option<Cleanup>,
}

/// What a folder says of the files in it: variables, pipelines and a batch limit.
#[derive(Debug, Clone)]
pub struct FolderConfig {
    vars: BTreeMap<String, Value>,
    pipelines: BTreeMap<String, Pipeline>,
    max_files: Option<usize>,
}

/// The settings a `[default]` and a watch have in common.
#[derive(Default)]
struct Settings {
    quiet: Option<Duration>,
    max_wait: Option<Duration>,
    max_files: Option<usize>,
    routes: Option<Vec<Written>>,
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
        let targets = read_targets(root.table("target")?.unwrap_or_default())?;
        let declared = root.table("watch")?.unwrap_or_default();
        root.finish()?;

        let mut watches = Vec::new();
        for (name, value) in declared {
            let scope = format!("watch.{name}");
            let mut reader = table_reader(scope.clone(), &value)?;
            let source = reader
                .absolute_path("source")?
                .ok_or_else(|| reader.invalid("source", "is required"))?;
            let settings = read_settings(reader)?;
            watches.push(build_watch(
                name,
                source,
                settings,
                &default,
                &definitions,
                &targets,
            )?);
        }

        let config = Config { watches, targets };
        config.check_apart(None, |path| path.to_path_buf())?;
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
        self.check_apart(Some(config_file), resolve)
    }

    /// Sources must not overlap one another or any target, or files would be planned again after they
    /// are moved; the configuration file may not lie in a source either, or it would be taken for a file
    /// to process. Each path is compared where `resolve` puts it.
    fn check_apart(
        &self,
        config_file: Option<&Path>,
        resolve: impl Fn(&Path) -> PathBuf,
    ) -> Result<(), ConfigError> {
        let config_file = config_file.map(&resolve);
        let sources: Vec<(&Watch, PathBuf)> = self
            .watches
            .iter()
            .map(|watch| (watch, resolve(&watch.source)))
            .collect();
        let overlap = |a: &Path, b: &Path| a.starts_with(b) || b.starts_with(a);

        for (index, (watch, source)) in sources.iter().enumerate() {
            if let Some(file) = &config_file
                && file.starts_with(source)
            {
                return Err(invalid(
                    &format!("watch.{}", watch.name),
                    "source",
                    format!("holds the configuration file `{}`", file.display()),
                ));
            }
            for (other, other_source) in &sources[index + 1..] {
                if overlap(source, other_source) {
                    return Err(invalid(
                        &format!("watch.{}", other.name),
                        "source",
                        format!("overlaps the source of `{}`", watch.name),
                    ));
                }
            }
            for (name, target) in &self.targets {
                if overlap(&resolve(target), source) {
                    return Err(invalid(
                        &format!("target.{name}"),
                        "path",
                        format!("overlaps the source of `{}`", watch.name),
                    ));
                }
            }
        }
        Ok(())
    }
}

impl Watch {
    /// The routes of the watch, in the order they claim.
    pub fn routes(&self) -> &[Route] {
        &self.routes
    }

    /// The pipeline of each of the watch's routes, at the same place as its route.
    pub fn pipelines(&self) -> Vec<(String, Pipeline)> {
        self.routes
            .iter()
            .map(|route| {
                let pipeline = self.definitions.get(&route.pipeline).expect(
                    "a route names a pipeline that reading the configuration found defined",
                );
                (route.pipeline.clone(), pipeline.clone())
            })
            .collect()
    }

    /// The watch as it is for the files of a folder that carries configurations of its own, from the farthest to the nearest.
    pub fn under(&self, folders: &[FolderConfig]) -> Watch {
        let mut watch = self.clone();
        for folder in folders {
            watch.vars.extend(folder.vars.clone());
            for (name, pipeline) in &folder.pipelines {
                if watch.routes.iter().any(|route| &route.pipeline == name) {
                    watch.definitions.insert(name.clone(), pipeline.clone());
                }
            }
            watch.max_files = folder.max_files.unwrap_or(watch.max_files);
        }
        watch
    }
}

impl FolderConfig {
    /// Whether the folder configuration declares a pipeline of this name, replacing the watch's own.
    pub(crate) fn declares(&self, pipeline: &str) -> bool {
        self.pipelines.contains_key(pipeline)
    }

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
        let max_files = read_max_files(&mut reader)?;
        reader.finish()?;
        Ok(FolderConfig {
            vars,
            pipelines,
            max_files,
        })
    }
}

fn read_settings(mut reader: Reader<String>) -> Result<Settings, ConfigError> {
    let quiet = reader.duration("quiet")?;
    let max_wait = reader.duration("max_wait")?;
    let max_files = read_max_files(&mut reader)?;
    let routes = read_routes(&mut reader)?;
    let dry_run = reader.boolean("dry_run")?;
    let vars = read_vars(&mut reader)?;
    let unit = read_unit(&mut reader)?;
    reader.finish()?;
    Ok(Settings {
        quiet,
        max_wait,
        max_files,
        routes,
        vars,
        unit,
        dry_run,
    })
}

/// The routes a table lists, in their order, as they are written.
fn read_routes(reader: &mut Reader<String>) -> Result<Option<Vec<Written>>, ConfigError> {
    let Some(value) = reader.take("routes") else {
        return Ok(None);
    };
    let Toml::Array(entries) = value else {
        return Err(reader.invalid("routes", "must be a list of routes"));
    };
    entries
        .iter()
        .map(|entry| {
            let mut route = table_reader("routes".to_string(), entry)?;
            let pipeline = route.required_string("pipeline")?;
            let target = route.string("move")?;
            let cleanup = match route.table("cleanup")? {
                Some(table) => Some(read_cleanup(Reader::new("cleanup".to_string(), table))?),
                None => None,
            };
            route.finish()?;
            Ok(Written {
                pipeline,
                target,
                cleanup,
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

fn read_cleanup(mut reader: Reader<String>) -> Result<Cleanup, ConfigError> {
    let keep = reader
        .strings("keep")?
        .unwrap_or_default()
        .iter()
        .map(|pattern| {
            Glob::new(pattern)
                .map(|glob| glob.compile_matcher())
                .map_err(|error| reader.invalid("keep", error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    reader.finish()?;
    Ok(Cleanup { keep })
}

/// The targets by name, each with the root it stands for.
fn read_targets(declared: Table) -> Result<BTreeMap<String, PathBuf>, ConfigError> {
    let mut targets = BTreeMap::new();
    for (name, value) in declared {
        let mut reader = table_reader(format!("target.{name}"), &value)?;
        let path = reader
            .absolute_path("path")?
            .ok_or_else(|| reader.invalid("path", "is required"))?;
        reader.finish()?;
        targets.insert(name, path);
    }
    Ok(targets)
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
    settings: Settings,
    default: &Settings,
    definitions: &BTreeMap<String, Pipeline>,
    targets: &BTreeMap<String, PathBuf>,
) -> Result<Watch, ConfigError> {
    let scope = format!("watch.{name}");
    let quiet = settings.quiet.or(default.quiet).unwrap_or(DEFAULT_QUIET);
    let max_wait = settings
        .max_wait
        .or(default.max_wait)
        .unwrap_or(DEFAULT_MAX_WAIT);
    if max_wait < quiet {
        return Err(invalid(
            &scope,
            "max_wait",
            "may not be shorter than `quiet`",
        ));
    }
    let routes = settings
        .routes
        .or_else(|| default.routes.clone())
        .unwrap_or_default()
        .into_iter()
        .map(|written| {
            if !definitions.contains_key(&written.pipeline) {
                return Err(invalid(
                    &scope,
                    "routes",
                    format!("the pipeline `{}` is not defined", written.pipeline),
                ));
            }
            let target = match written.target {
                Some(target) => Some(targets.get(&target).cloned().ok_or_else(|| {
                    invalid(
                        &scope,
                        "routes",
                        format!("the target `{target}` is not declared"),
                    )
                })?),
                None => None,
            };
            Ok(Route {
                pipeline: written.pipeline,
                target,
                cleanup: written.cleanup,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut vars = default.vars.clone();
    vars.extend(settings.vars);
    Ok(Watch {
        name,
        source,
        unit: settings
            .unit
            .or_else(|| default.unit.clone())
            .unwrap_or(Unit::Directory),
        vars,
        quiet,
        max_wait,
        max_files: settings
            .max_files
            .or(default.max_files)
            .unwrap_or(DEFAULT_MAX_FILES),
        dry_run: settings.dry_run.or(default.dry_run).unwrap_or(false),
        routes,
        definitions: definitions.clone(),
    })
}

/// The batch limit a table says, which may not be more than the ceiling.
fn read_max_files(reader: &mut Reader<String>) -> Result<Option<usize>, ConfigError> {
    match reader.positive("max_files")? {
        Some(limit) if limit > MAX_BATCH_FILES => Err(reader.invalid(
            "max_files",
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
