use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

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

/// Whether a notification says a file was written, created or replaced, as opposed to only read.
///
/// Reading a file also makes notifications about it, so a program that reads its own configuration
/// must not take every notification for a change.
pub fn rewrites(notification: &notify::Event, file: &Path) -> bool {
    let changed = matches!(
        notification.kind,
        EventKind::Access(AccessKind::Close(AccessMode::Write))
            | EventKind::Create(_)
            | EventKind::Modify(ModifyKind::Data(_))
            | EventKind::Modify(ModifyKind::Name(RenameMode::To | RenameMode::Both))
    );
    changed && notification.paths.iter().any(|path| path == file)
}

/// The handler to give the filesystem watcher: it passes on what says something changed and every error.
///
/// A file or folder that is only opened, read or has its attributes changed is reported too, by every
/// program that looks into the source, and none of it concerns the watcher. Dropping it here keeps it
/// from taking the time and memory of what does.
pub fn pass_changes(
    sender: Sender<notify::Result<notify::Event>>,
) -> impl FnMut(notify::Result<notify::Event>) + Send + 'static {
    move |notification| {
        if notification.as_ref().map_or(true, changes) {
            // The runner may already be gone, and then nobody is left to tell.
            let _ = sender.send(notification);
        }
    }
}

fn changes(notification: &notify::Event) -> bool {
    matches!(
        notification.kind,
        EventKind::Access(AccessKind::Close(AccessMode::Write))
            | EventKind::Create(_)
            | EventKind::Modify(ModifyKind::Data(_) | ModifyKind::Name(_))
            | EventKind::Remove(_)
    )
}
