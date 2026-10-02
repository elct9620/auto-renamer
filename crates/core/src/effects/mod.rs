//! What `move` and `cleanup` do to the files of the source and the target, through a tree.

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

mod cleanup;
mod relocate;
mod tree;

use globset::GlobMatcher;

use crate::record::Record;

pub use cleanup::cleanup_folders;
pub use relocate::move_file;
pub use tree::{Kind, Tree};

/// What a route does to a planned file: a move to its target, or in place, and a cleanup.
#[derive(Debug, Clone)]
pub enum Effect {
    Move,
    Cleanup(Cleanup),
}

/// What a cleanup keeps however empty it is: the folders the patterns name.
#[derive(Debug, Clone, Default)]
pub struct Cleanup {
    pub keep: Vec<GlobMatcher>,
}

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

/// Why a file was left alone before it could be planned or moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    Link,
    NotAFile,
    Missing,
    NoModificationTime,
}

impl fmt::Display for SkipReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            SkipReason::Link => "link",
            SkipReason::NotAFile => "not a file",
            SkipReason::Missing => "missing",
            SkipReason::NoModificationTime => "no modification time",
        })
    }
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

/// What one effect did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Done {
    Moved(Applied),
    Cleaned(Vec<PathBuf>),
}

/// What the effects of a route did to one file, and the failure that stopped them if there was one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectsRun {
    pub done: Vec<Done>,
    pub error: Option<EffectError>,
}

/// Carries out the effects a planned file carries, in their order, stopping at the first failure so
/// nothing after a failed effect acts on what it did not do.
pub fn apply_effects(
    tree: &dyn Tree,
    effects: &[Effect],
    record: &Record,
    unit: &Path,
    roots: &Roots,
    dry_run: bool,
) -> EffectsRun {
    let mut run = EffectsRun {
        done: Vec::new(),
        error: None,
    };
    for effect in effects {
        let done = match effect {
            Effect::Move => move_file(tree, record, roots, dry_run).map(Done::Moved),
            // A dry run leaves the file where it is, so no folder it would empty is empty yet.
            Effect::Cleanup(_) if dry_run => continue,
            Effect::Cleanup(policy) => {
                cleanup_folders(tree, policy, record.origin(), unit, roots).map(Done::Cleaned)
            }
        };
        match done {
            Ok(done) => run.done.push(done),
            Err(error) => {
                run.error = Some(error);
                break;
            }
        }
    }
    run
}

pub fn io_error(action: &'static str, path: &Path, kind: io::ErrorKind) -> EffectError {
    EffectError::Io {
        action,
        path: path.to_path_buf(),
        kind,
    }
}
