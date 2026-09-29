//! The only code that touches the files of the source and the target.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

mod cleanup;
mod relocate;

use crate::pipeline::Pipeline;
use crate::record::Record;
use crate::stages::Stage;

pub use cleanup::cleanup_folders;
pub use relocate::move_file;

/// The source a file comes from and the target it goes to, which are one folder when files are renamed in place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Roots {
    pub source: PathBuf,
    pub target: PathBuf,
}

/// What a move did: moved the file, left it as it was, skipped it, or only said what it would do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Applied {
    Moved { from: PathBuf, to: PathBuf },
    Unchanged(PathBuf),
    Skipped(SkipReason),
    Preview { from: PathBuf, to: PathBuf },
}

/// Why a file was skipped rather than moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    Link,
    NotAFile,
}

/// Why a move was refused or failed, with the file as it was.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectError {
    Unsafe(PathBuf),
    Conflict(PathBuf),
    Missing(PathBuf),
    Io {
        action: &'static str,
        path: PathBuf,
        kind: io::ErrorKind,
    },
}

impl fmt::Display for EffectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EffectError::Unsafe(path) => write!(
                f,
                "`{}` could lead outside the source or the target",
                path.display()
            ),
            EffectError::Conflict(path) => write!(f, "`{}` already exists", path.display()),
            EffectError::Missing(path) => write!(f, "`{}` is not there", path.display()),
            EffectError::Io { action, path, kind } => {
                write!(f, "could not {action} `{}`: {kind}", path.display())
            }
        }
    }
}

impl std::error::Error for EffectError {}

/// What one effect stage did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Done {
    Moved(Applied),
    Cleaned(Vec<PathBuf>),
}

/// What the effect stages of a pipeline did to one file, and the failure that stopped them if there was one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectsRun {
    pub done: Vec<Done>,
    pub error: Option<EffectError>,
}

/// Runs the effect stages of a pipeline in the order they are written on a planned file,
/// stopping at the first failure so nothing after a failed step acts on what it did not do.
pub fn apply_effects(
    pipeline: &Pipeline,
    record: &Record,
    unit: &Path,
    roots: &Roots,
    dry_run: bool,
) -> EffectsRun {
    let mut run = EffectsRun {
        done: Vec::new(),
        error: None,
    };
    for stage in pipeline.stages().iter().filter(|stage| stage.is_effect()) {
        let step = match stage {
            Stage::Move(policy) => move_file(policy, record, roots, dry_run).map(Done::Moved),
            Stage::Cleanup(policy) => {
                cleanup_folders(policy, record.origin(), unit, roots, dry_run).map(Done::Cleaned)
            }
            _ => continue,
        };
        match step {
            Ok(done) => run.done.push(done),
            Err(error) => {
                run.error = Some(error);
                break;
            }
        }
    }
    run
}

pub(crate) fn exists(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

pub(crate) fn io_error(action: &'static str, path: &Path, kind: io::ErrorKind) -> EffectError {
    EffectError::Io {
        action,
        path: path.to_path_buf(),
        kind,
    }
}
