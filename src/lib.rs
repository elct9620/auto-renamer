//! Renames and relocates files according to a configurable pipeline of small stages.

pub mod record;
pub mod template;

pub use record::{Record, Value};
pub use template::{RenderError, Template, TemplateError};
