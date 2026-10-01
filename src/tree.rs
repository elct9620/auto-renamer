use std::io;
use std::path::Path;

use chrono::{DateTime, Utc};

/// What a path holds, seen without following a link.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    File,
    Folder,
    Link,
    Other,
}

/// The files of the source and the target as far as processing a batch needs them, so the filesystem
/// or a virtual tree can stand behind it. Paths are the ones the watch names, not relative to a root.
pub trait Tree {
    /// What is at a path, without following a link; a path holding nothing is an error.
    fn kind(&self, path: &Path) -> io::Result<Kind>;

    /// When the file at a path was last modified.
    fn modified(&self, path: &Path) -> io::Result<DateTime<Utc>>;

    /// The text of the file at a path, never more than `limit` of its bytes.
    fn read(&self, path: &Path, limit: u64) -> io::Result<String>;
}
