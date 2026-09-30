//! Renames and relocates files according to a configurable pipeline of small stages.

pub mod cli;
pub mod config;
pub mod context;
pub mod effects;
pub mod engine;
pub mod pipeline;
mod reader;
pub mod record;
#[cfg(target_os = "linux")]
pub mod runner;
pub mod scan;
pub mod service;
pub mod stages;
pub mod template;
pub mod watcher;

pub use cli::{Command, Options, parse_args};
pub use config::{Config, ConfigError, FolderConfig, Unit, Watch};
pub use context::{Context, Target};
pub use effects::{
    Applied, Done, EffectError, EffectsRun, Roots, SkipReason, apply_effects, cleanup_folders,
    move_file,
};
pub use engine::{Judged, Verdict, plan_batch};
pub use pipeline::{Pipeline, PipelineError};
pub use record::{NotUtf8, Record, Value};
#[cfg(target_os = "linux")]
pub use runner::{RunError, run};
pub use scan::scan_folder;
pub use service::{FsTarget, Processed, Renames, What, process_batch};
pub use stages::{DeclareError, Outcome, Rejection, Stage};
pub use template::{RenderError, Template, TemplateError};
pub use watcher::{Event, Machine, Ready, Translated, rewrites, translate};
