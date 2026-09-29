//! Finds the files that were already there, or that arrived as a whole folder.

use std::fs;
use std::path::{Path, PathBuf};

use crate::watcher::Event;

/// Lists the regular files below a folder as found-at-start events, named relative to the source.
///
/// Links are never followed: a linked file is not listed and a linked folder is not entered. What cannot
/// be read is left out, so a folder that is not there gives nothing.
pub fn scan_folder(source: &Path, folder: &Path) -> Vec<Event> {
    let mut found = Vec::new();
    let mut folders: Vec<PathBuf> = vec![folder.to_path_buf()];
    while let Some(current) = folders.pop() {
        let Ok(entries) = fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(metadata) = fs::symlink_metadata(&path) else {
                continue;
            };
            if metadata.is_dir() {
                folders.push(path);
            } else if metadata.is_file()
                && let (Ok(relative), Ok(modified)) =
                    (path.strip_prefix(source), metadata.modified())
            {
                found.push(Event::Found {
                    path: relative.to_path_buf(),
                    modified,
                });
            }
        }
    }
    found
}
