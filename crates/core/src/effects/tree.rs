use std::io;
use std::path::Path;

use chrono::{DateTime, Utc};

use super::EffectError;

/// What a path holds, seen without following a link.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    File,
    Folder,
    Link,
    Other,
}

/// The files of the source and the target as processing a batch reads them and its effects change them,
/// so the filesystem or a virtual tree can stand behind it. Paths are the ones the watch names, not
/// relative to a root.
pub trait Tree {
    /// What is at a path, without following a link; a path holding nothing is an error.
    fn kind(&self, path: &Path) -> io::Result<Kind>;

    /// When the file at a path was last modified.
    fn modified(&self, path: &Path) -> io::Result<DateTime<Utc>>;

    /// The text of the file at a path, never more than `limit` of its bytes.
    fn read(&self, path: &Path, limit: u64) -> io::Result<String>;

    /// The names of the regular files in a folder, none when it cannot be listed.
    fn files(&self, folder: &Path) -> Vec<String>;

    /// Whether a folder holds nothing at all.
    fn is_empty(&self, folder: &Path) -> io::Result<bool>;

    /// Puts a file where it is wanted, making the folders on the way, without ever replacing what is there.
    fn place(&self, from: &Path, to: &Path) -> Result<(), EffectError>;

    /// Removes a folder that holds nothing.
    fn remove_folder(&self, folder: &Path) -> io::Result<()>;
}
