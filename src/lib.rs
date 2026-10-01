//! Renames and relocates files according to a configurable pipeline of small stages.

pub mod cli;
pub mod filesystem;
#[cfg(target_os = "linux")]
pub mod runner;
pub mod scan;
pub mod watcher;

pub use auto_renamer_core::*;
pub use cli::{Command, Options, parse_args};
pub use filesystem::FsTree;
#[cfg(target_os = "linux")]
pub use runner::{RunError, run};
pub use scan::Scan;
pub use watcher::{Event, Machine, Queue, Ready, Translated, rewrites, translate};
