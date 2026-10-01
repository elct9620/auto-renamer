use std::io;
use std::path::{Path, PathBuf};

use super::{EffectError, Kind, Roots, Tree, io_error};
use crate::stages::Cleanup;

/// Removes the folders a moved file left empty, from the folder it was in upward.
///
/// Only empty folders inside the unit go, and never the source itself. It stops at a folder that is not
/// empty, that holds a folder configuration, that a keep pattern names, or that is a link.
pub fn cleanup_folders(
    tree: &dyn Tree,
    stage: &Cleanup,
    origin: &Path,
    unit: &Path,
    roots: &Roots,
) -> Result<Vec<PathBuf>, EffectError> {
    let mut removed = Vec::new();
    let mut folder = origin.parent();

    while let Some(current) = folder {
        if current.as_os_str().is_empty() || !current.starts_with(unit) || is_kept(stage, current) {
            break;
        }
        let path = roots.source.join(current);
        if !is_empty_folder(tree, &path)? {
            break;
        }
        match tree.remove_folder(&path) {
            Ok(()) => {}
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::DirectoryNotEmpty | io::ErrorKind::NotFound
                ) =>
            {
                break;
            }
            Err(error) => return Err(io_error("remove the folder", &path, error.kind())),
        }
        removed.push(current.to_path_buf());
        folder = current.parent();
    }
    Ok(removed)
}

fn is_kept(stage: &Cleanup, folder: &Path) -> bool {
    folder
        .file_name()
        .is_some_and(|name| stage.keep.iter().any(|pattern| pattern.is_match(name)))
}

/// A real folder holding nothing at all: not a link, and not a file.
fn is_empty_folder(tree: &dyn Tree, path: &Path) -> Result<bool, EffectError> {
    match tree.kind(path) {
        Ok(Kind::Folder) => {}
        Ok(_) => return Ok(false),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(io_error("inspect", path, error.kind())),
    }
    tree.is_empty(path)
        .map_err(|error| io_error("read the folder", path, error.kind()))
}
