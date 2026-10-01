use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use auto_renamer_core::config::FOLDER_CONFIG;
use auto_renamer_core::effects::io_error;
use auto_renamer_core::{EffectError, Kind, Tree};
use chrono::{DateTime, Utc};

use crate::Entry;

enum Node {
    Folder,
    File { modified: i64, text: String },
}

/// The tree the page describes, held in memory. Every folder on the way to an entry exists, as it would
/// on a filesystem.
pub(crate) struct VirtualTree {
    nodes: RefCell<BTreeMap<PathBuf, Node>>,
}

impl VirtualTree {
    pub(crate) fn new(entries: Vec<Entry>) -> VirtualTree {
        let mut nodes = BTreeMap::new();
        for entry in entries {
            let path = PathBuf::from(entry.path);
            add_folders_above(&mut nodes, &path);
            let node = if entry.folder {
                Node::Folder
            } else {
                Node::File {
                    modified: entry.modified,
                    text: entry.text,
                }
            };
            nodes.insert(path, node);
        }
        VirtualTree {
            nodes: RefCell::new(nodes),
        }
    }

    /// The files under a folder, relative to it, leaving out folder configurations as the watcher does.
    pub(crate) fn files_under(&self, root: &Path) -> Vec<PathBuf> {
        self.nodes
            .borrow()
            .iter()
            .filter(|(_, node)| matches!(node, Node::File { .. }))
            .filter_map(|(path, _)| path.strip_prefix(root).ok())
            .filter(|relative| {
                relative
                    .file_name()
                    .is_some_and(|name| name != FOLDER_CONFIG)
            })
            .map(Path::to_path_buf)
            .collect()
    }

    pub(crate) fn into_entries(self) -> Vec<Entry> {
        self.nodes
            .into_inner()
            .into_iter()
            .map(|(path, node)| {
                let path = path.display().to_string();
                match node {
                    Node::Folder => Entry {
                        path,
                        folder: true,
                        modified: 0,
                        text: String::new(),
                    },
                    Node::File { modified, text } => Entry {
                        path,
                        folder: false,
                        modified,
                        text,
                    },
                }
            })
            .collect()
    }

    fn children(&self, folder: &Path) -> Vec<(PathBuf, bool)> {
        self.nodes
            .borrow()
            .iter()
            .filter(|(path, _)| path.parent() == Some(folder))
            .map(|(path, node)| (path.clone(), matches!(node, Node::File { .. })))
            .collect()
    }
}

fn add_folders_above(nodes: &mut BTreeMap<PathBuf, Node>, path: &Path) {
    // The root holds every watch and is no folder of one, so it is left out like an empty path.
    for ancestor in path.ancestors().skip(1) {
        if ancestor.parent().is_none() {
            break;
        }
        nodes.entry(ancestor.to_path_buf()).or_insert(Node::Folder);
    }
}

fn not_found() -> io::Error {
    io::Error::from(io::ErrorKind::NotFound)
}

impl Tree for VirtualTree {
    fn kind(&self, path: &Path) -> io::Result<Kind> {
        match self.nodes.borrow().get(path) {
            Some(Node::Folder) => Ok(Kind::Folder),
            Some(Node::File { .. }) => Ok(Kind::File),
            None => Err(not_found()),
        }
    }

    fn modified(&self, path: &Path) -> io::Result<DateTime<Utc>> {
        match self.nodes.borrow().get(path) {
            Some(Node::File { modified, .. }) => DateTime::from_timestamp_millis(*modified)
                .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData)),
            _ => Err(not_found()),
        }
    }

    fn read(&self, path: &Path, limit: u64) -> io::Result<String> {
        match self.nodes.borrow().get(path) {
            Some(Node::File { text, .. }) => {
                let end = text.len().min(usize::try_from(limit).unwrap_or(usize::MAX));
                text.get(..end)
                    .map(str::to_string)
                    .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))
            }
            _ => Err(not_found()),
        }
    }

    fn files(&self, folder: &Path) -> Vec<String> {
        self.children(folder)
            .into_iter()
            .filter(|(_, is_file)| *is_file)
            .filter_map(|(path, _)| path.file_name()?.to_str().map(str::to_string))
            .collect()
    }

    fn is_empty(&self, folder: &Path) -> io::Result<bool> {
        Ok(self.children(folder).is_empty())
    }

    fn place(&self, from: &Path, to: &Path) -> Result<(), EffectError> {
        if self.kind(to).is_ok() {
            return Err(EffectError::Conflict(to.to_path_buf()));
        }
        let mut nodes = self.nodes.borrow_mut();
        if let Some(folder) = to.parent()
            && folder
                .ancestors()
                .any(|ancestor| matches!(nodes.get(ancestor), Some(Node::File { .. })))
        {
            return Err(io_error(
                "create the folder",
                folder,
                io::ErrorKind::NotADirectory,
            ));
        }
        if !matches!(nodes.get(from), Some(Node::File { .. })) {
            return Err(EffectError::Missing(from.to_path_buf()));
        }
        let Some(file) = nodes.remove(from) else {
            return Err(EffectError::Missing(from.to_path_buf()));
        };
        add_folders_above(&mut nodes, to);
        nodes.insert(to.to_path_buf(), file);
        Ok(())
    }

    fn remove_folder(&self, folder: &Path) -> io::Result<()> {
        if !matches!(self.nodes.borrow().get(folder), Some(Node::Folder)) {
            return Err(not_found());
        }
        if !self.children(folder).is_empty() {
            return Err(io::Error::from(io::ErrorKind::DirectoryNotEmpty));
        }
        self.nodes.borrow_mut().remove(folder);
        Ok(())
    }
}
