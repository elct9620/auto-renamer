//! Renames and relocates files according to a configurable pipeline of small stages.

pub mod context;
pub mod pipeline;
pub mod record;
pub mod stages;
pub mod template;

pub use context::{Context, Target};
pub use pipeline::{Pipeline, PipelineError};
pub use record::{Record, Value};
pub use stages::{DeclareError, Outcome, Rejection, Stage};
pub use template::{RenderError, Template, TemplateError};
