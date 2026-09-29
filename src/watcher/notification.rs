use std::path::{Path, PathBuf};

use notify::EventKind;
use notify::event::{
    AccessKind, AccessMode, CreateKind, DataChange, ModifyKind, RemoveKind, RenameMode,
};

use super::Event;

/// What a filesystem notification means for the watcher: an event for a file, or a folder to be scanned
/// because it arrived whole and its files were never reported one by one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Translated {
    Event(Event),
    Scan(PathBuf),
}

/// Translates a filesystem notification into what happened to files of the source.
///
/// Only a write that was closed, or a file moved in, settles a file: creation and data changes mean it is
/// still being written. Paths outside the source, changes to folders and other kinds of notification say nothing.
pub fn translate(
    notification: &notify::Event,
    source: &Path,
    is_dir: impl Fn(&Path) -> bool,
) -> Vec<Translated> {
    let event = |make: fn(PathBuf) -> Event, path: &Path| {
        relative_to(source, path).map(|relative| Translated::Event(make(relative.to_path_buf())))
    };
    let arrived = |path: &Path| {
        if is_dir(path) {
            relative_to(source, path).map(|_| Translated::Scan(path.to_path_buf()))
        } else {
            event(Event::Settled, path)
        }
    };

    let paths = &notification.paths;
    let translated: Vec<Option<Translated>> = match &notification.kind {
        EventKind::Access(AccessKind::Close(AccessMode::Write)) => paths
            .iter()
            .map(|path| event(Event::Settled, path))
            .collect(),
        EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
            paths.iter().map(|path| arrived(path)).collect()
        }
        EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
            paths.iter().map(|path| event(Event::Gone, path)).collect()
        }
        EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => match paths.as_slice() {
            [from, to] => vec![event(Event::Gone, from), arrived(to)],
            _ => Vec::new(),
        },
        EventKind::Modify(ModifyKind::Data(
            DataChange::Any | DataChange::Size | DataChange::Content | DataChange::Other,
        )) => paths
            .iter()
            .filter(|path| !is_dir(path))
            .map(|path| event(Event::Writing, path))
            .collect(),
        EventKind::Create(CreateKind::File) => paths
            .iter()
            .map(|path| event(Event::Writing, path))
            .collect(),
        EventKind::Create(CreateKind::Folder) => paths
            .iter()
            .map(|path| relative_to(source, path).map(|_| Translated::Scan(path.to_path_buf())))
            .collect(),
        EventKind::Remove(RemoveKind::File) => {
            paths.iter().map(|path| event(Event::Gone, path)).collect()
        }
        _ => Vec::new(),
    };
    translated.into_iter().flatten().collect()
}

fn relative_to<'a>(source: &Path, path: &'a Path) -> Option<&'a Path> {
    path.strip_prefix(source)
        .ok()
        .filter(|relative| !relative.as_os_str().is_empty())
}
