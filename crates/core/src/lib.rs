//! What auto-renamer decides about files: the pipeline that names them, the configuration that declares
//! it, and the rules a batch is processed and moved by, through a tree the caller stands behind.

pub mod config;
pub mod context;
pub mod effects;
pub mod engine;
pub mod pipeline;
mod reader;
pub mod record;
pub mod service;
pub mod stages;
pub mod template;

pub use config::{Config, ConfigError, FolderConfig, RejectedRoute, Route, Unit, Watch};
pub use context::{Context, Target};
pub use effects::{
    Applied, Cleanup, Done, Effect, EffectError, EffectsRun, Kind, Roots, SkipReason, Tree,
    apply_effects, cleanup_folders, move_file,
};
pub use engine::{Judged, Step, Verdict, plan_batch, plan_batch_observed};
pub use pipeline::{Pipeline, PipelineError};
pub use record::{NotUtf8, Record, Value};
pub use service::{
    Processed, Renames, What, process_batch, process_batch_observed, replaced_pipelines,
};
pub use stages::{Declaration, DeclareError, Declared, Flow, Parameter, ParameterKind, Stop};
pub use template::{RenderError, Template, TemplateError};
