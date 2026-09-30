//! Finds the files that were already there, or that arrived as a whole folder.

use std::fs::{self, ReadDir};
use std::path::{Path, PathBuf};

use crate::watcher::Event;

/// A walk over the regular files below a folder, each a found-at-start event named relative to the source.
///
/// It is taken a number of entries at a time, so whoever scans may do other work and come back for more.
/// Links are never followed: a linked file is not listed and a linked folder is not entered. What cannot
/// be read is left out, so a folder that is not there gives nothing.
pub struct Scan {
    source: PathBuf,
    folders: Vec<PathBuf>,
    reading: Option<ReadDir>,
}

impl Scan {
    /// Starts a scan of a folder, whose files are named relative to the source.
    pub fn new(source: &Path, folder: &Path) -> Scan {
        Scan {
            source: source.to_path_buf(),
            folders: vec![folder.to_path_buf()],
            reading: None,
        }
    }

    /// Looks at up to a number of entries and answers the files found among them. A folder that is
    /// opened counts as one, so many folders holding nothing cost as much as many files.
    pub fn look_at(&mut self, entries: usize) -> Vec<Event> {
        let mut found = Vec::new();
        for _ in 0..entries {
            let Some(reading) = self.reading.as_mut() else {
                let Some(folder) = self.folders.pop() else {
                    break;
                };
                self.reading = fs::read_dir(folder).ok();
                continue;
            };
            match reading.next() {
                Some(Ok(entry)) => found.extend(self.found(entry.path())),
                Some(Err(_)) => {}
                None => self.reading = None,
            }
        }
        found
    }

    /// Whether the scan has looked at everything below its folder.
    pub fn is_finished(&self) -> bool {
        self.reading.is_none() && self.folders.is_empty()
    }

    /// The file an entry is, when it is a regular one; a folder is kept to be read later.
    fn found(&mut self, path: PathBuf) -> Option<Event> {
        let metadata = fs::symlink_metadata(&path).ok()?;
        if metadata.is_dir() {
            self.folders.push(path);
            return None;
        }
        if !metadata.is_file() {
            return None;
        }
        let modified = metadata.modified().ok()?;
        let relative = path.strip_prefix(&self.source).ok()?;
        Some(Event::Found {
            path: relative.to_path_buf(),
            modified,
        })
    }
}
