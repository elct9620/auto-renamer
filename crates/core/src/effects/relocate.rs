use std::io;
use std::path::{Component, Path, PathBuf};

use super::{Applied, EffectError, Kind, Roots, SkipReason, Tree, io_error};
use crate::record::{Record, split_extension};
use crate::stages::{Move, OnConflict};

/// Moves the file of a planned record to its plan under the target, or says where it would go.
///
/// Nothing is overwritten and no link is followed. A path that could leave the source or the target is
/// refused before anything is touched.
pub fn move_file(
    tree: &dyn Tree,
    stage: &Move,
    record: &Record,
    roots: &Roots,
    dry_run: bool,
) -> Result<Applied, EffectError> {
    ensure_inside(record.origin())?;
    ensure_inside(record.plan())?;
    let from = roots.source.join(record.origin());
    let mut to = roots.target.join(record.plan());

    if let Some(reason) = skip_reason(tree, &roots.source, record.origin())? {
        return Ok(Applied::Skipped(reason));
    }
    if from == to {
        return Ok(Applied::Unchanged(from));
    }
    if tree.kind(&to).is_ok() {
        to = settle_conflict(tree, stage, &to)?;
    }
    if dry_run {
        return Ok(Applied::Preview { from, to });
    }

    tree.place(&from, &to)?;
    Ok(Applied::Moved { from, to })
}

/// A relative path made only of names, so joining it to a root cannot leave that root.
fn ensure_inside(path: &Path) -> Result<(), EffectError> {
    let inside = path.components().count() > 0
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)));
    if inside {
        Ok(())
    } else {
        Err(EffectError::Unsafe(path.to_path_buf()))
    }
}

/// Walks from the source down to the file without following anything: a link on the way, or a file that
/// is not a regular file, means it is left alone.
fn skip_reason(
    tree: &dyn Tree,
    source: &Path,
    origin: &Path,
) -> Result<Option<SkipReason>, EffectError> {
    let mut current = source.to_path_buf();
    let last = origin.components().count() - 1;
    for (position, part) in origin.components().enumerate() {
        current.push(part);
        let kind = tree.kind(&current).map_err(|error| match error.kind() {
            io::ErrorKind::NotFound => EffectError::Missing(current.clone()),
            kind => io_error("inspect", &current, kind),
        })?;
        if kind == Kind::Link {
            return Ok(Some(SkipReason::Link));
        }
        if position == last && kind != Kind::File {
            return Ok(Some(SkipReason::NotAFile));
        }
    }
    Ok(None)
}

/// What to do when the target is taken: refuse, or try once more with the suffix before the extension.
fn settle_conflict(tree: &dyn Tree, stage: &Move, taken: &Path) -> Result<PathBuf, EffectError> {
    match stage.on_conflict {
        OnConflict::Reject => Err(EffectError::Conflict(taken.to_path_buf())),
        OnConflict::Suffix => {
            let Some(file_name) = taken.file_name().and_then(|name| name.to_str()) else {
                return Err(EffectError::Conflict(taken.to_path_buf()));
            };
            let (name, ext) = split_extension(file_name);
            let suffixed = if ext.is_empty() {
                format!("{name}{}", stage.suffix)
            } else {
                format!("{name}{}.{ext}", stage.suffix)
            };
            let candidate = taken.with_file_name(suffixed);
            if tree.kind(&candidate).is_ok() {
                Err(EffectError::Conflict(candidate))
            } else {
                Ok(candidate)
            }
        }
    }
}
