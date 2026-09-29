//! Renames and relocates files according to a configurable pipeline of small stages.

pub mod config;
pub mod context;
pub mod effects;
pub mod engine;
pub mod pipeline;
pub mod record;
pub mod stages;
pub mod template;

pub use config::{Config, ConfigError, FolderConfig, Unit, Watch};
pub use context::{Context, Target};
pub use effects::{Applied, EffectError, Roots, SkipReason, move_file};
pub use engine::{Judged, Verdict, plan_batch};
pub use pipeline::{Pipeline, PipelineError};
pub use record::{Record, Value};
pub use stages::{DeclareError, Outcome, Rejection, Stage};
pub use template::{RenderError, Template, TemplateError};
