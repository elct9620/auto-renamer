use std::path::{Path, PathBuf};
use std::time::Duration;

use auto_renamer::{Queue, rewrites, translate};
use notify::EventKind;
use notify::event::{
    AccessKind, AccessMode, CreateKind, DataChange, Flag, MetadataKind, ModifyKind, RemoveKind,
    RenameMode,
};

type Notification = notify::Result<notify::Event>;

fn notification(kind: EventKind, paths: &[&str]) -> notify::Event {
    paths.iter().fold(notify::Event::new(kind), |event, path| {
        event.add_path(PathBuf::from(path))
    })
}

fn closed_after_writing(name: &str) -> notify::Event {
    let kind = EventKind::Access(AccessKind::Close(AccessMode::Write));
    notification(kind, &[&format!("/s/{name}")])
}

/// A queue with room for a number of notifications, after its handler was handed some.
fn queue_handed(capacity: usize, handed: Vec<Notification>) -> Queue {
    let queue = Queue::new(capacity);
    handed.into_iter().for_each(queue.handler());
    queue
}

fn everything_in(queue: &Queue) -> Vec<Notification> {
    queue.take(Duration::ZERO, usize::MAX)
}

fn queued(kind: EventKind) -> bool {
    let handed = vec![Ok(notification(kind, &["/s/Show/a.mkv"]))];
    !everything_in(&queue_handed(8, handed)).is_empty()
}

// @behavior QUE-001
#[test]
fn should_queue_a_notification_that_something_changed() {
    let closed = closed_after_writing("a.mkv");

    let queue = queue_handed(8, vec![Ok(closed.clone())]);

    let came: Vec<notify::Event> = everything_in(&queue).into_iter().flatten().collect();
    assert_eq!(came, [closed]);
}

// @behavior QUE-002
#[test]
fn should_drop_a_notification_that_nothing_changed() {
    let nothing_changed = [
        EventKind::Access(AccessKind::Open(AccessMode::Any)),
        EventKind::Access(AccessKind::Close(AccessMode::Read)),
        EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any)),
    ];

    let came: Vec<EventKind> = nothing_changed
        .into_iter()
        .filter(|kind| queued(*kind))
        .collect();

    assert_eq!(came, []);
}

// @behavior QUE-003
#[test]
fn should_queue_an_error() {
    let failed = Err(notify::Error::generic("the watcher failed"));

    let came = everything_in(&queue_handed(8, vec![failed]));

    assert!(matches!(came.as_slice(), [Err(_)]), "{came:?}");
}

// @behavior QUE-004
#[test]
fn should_queue_whatever_is_translated_or_rewrites_the_configuration() {
    let kinds = [
        EventKind::Any,
        EventKind::Other,
        EventKind::Access(AccessKind::Any),
        EventKind::Access(AccessKind::Read),
        EventKind::Access(AccessKind::Open(AccessMode::Any)),
        EventKind::Access(AccessKind::Close(AccessMode::Read)),
        EventKind::Access(AccessKind::Close(AccessMode::Write)),
        EventKind::Create(CreateKind::Any),
        EventKind::Create(CreateKind::File),
        EventKind::Create(CreateKind::Folder),
        EventKind::Create(CreateKind::Other),
        EventKind::Modify(ModifyKind::Any),
        EventKind::Modify(ModifyKind::Other),
        EventKind::Modify(ModifyKind::Data(DataChange::Any)),
        EventKind::Modify(ModifyKind::Data(DataChange::Size)),
        EventKind::Modify(ModifyKind::Data(DataChange::Content)),
        EventKind::Modify(ModifyKind::Data(DataChange::Other)),
        EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any)),
        EventKind::Modify(ModifyKind::Name(RenameMode::Any)),
        EventKind::Modify(ModifyKind::Name(RenameMode::To)),
        EventKind::Modify(ModifyKind::Name(RenameMode::From)),
        EventKind::Modify(ModifyKind::Name(RenameMode::Both)),
        EventKind::Modify(ModifyKind::Name(RenameMode::Other)),
        EventKind::Remove(RemoveKind::Any),
        EventKind::Remove(RemoveKind::File),
        EventKind::Remove(RemoveKind::Folder),
        EventKind::Remove(RemoveKind::Other),
    ];
    let answered = |kind: EventKind| {
        let of_a_file = notification(kind, &["/s/Show/a.mkv", "/s/Show/b.mkv"]);
        let of_a_folder = notification(kind, &["/s/Show", "/s/Show"]);
        let is_folder = |path: &Path| path == Path::new("/s/Show");
        !translate(&of_a_file, Path::new("/s"), is_folder).is_empty()
            || !translate(&of_a_folder, Path::new("/s"), is_folder).is_empty()
            || rewrites(&of_a_file, Path::new("/s/Show/a.mkv"))
    };

    let dropped: Vec<EventKind> = kinds
        .into_iter()
        .filter(|kind| answered(*kind) && !queued(*kind))
        .collect();

    assert_eq!(dropped, []);
}

fn closed(names: &[&str]) -> Vec<Notification> {
    names
        .iter()
        .map(|name| Ok(closed_after_writing(name)))
        .collect()
}

// @behavior QUE-005
#[test]
fn should_lose_nothing_while_there_is_room() {
    let queue = queue_handed(2, closed(&["a.mkv", "b.mkv"]));

    assert!(!queue.lost());
}

// @behavior QUE-006
#[test]
fn should_say_so_when_a_queue_with_no_room_loses_what_comes() {
    let queue = queue_handed(2, closed(&["a.mkv", "b.mkv", "c.mkv"]));

    assert!(queue.lost());
}

// @behavior QUE-007
#[test]
fn should_count_notifications_the_kernel_lost_as_lost() {
    let overflowed = notify::Event::new(EventKind::Other).set_flag(Flag::Rescan);

    let queue = queue_handed(8, vec![Ok(overflowed)]);

    assert!(queue.lost());
}

// @behavior QUE-008
#[test]
fn should_drop_what_waited_once_a_loss_is_told() {
    let queue = queue_handed(2, closed(&["a.mkv", "b.mkv", "c.mkv"]));
    queue.lost();

    assert!(everything_in(&queue).is_empty());
}

// @behavior QUE-009
#[test]
fn should_tell_a_loss_once() {
    let queue = queue_handed(2, closed(&["a.mkv", "b.mkv", "c.mkv"]));
    queue.lost();

    assert!(!queue.lost());
}

// @behavior QUE-010
#[test]
fn should_take_no_more_notifications_in_a_turn_than_its_limit() {
    let queue = queue_handed(8, closed(&["a.mkv", "b.mkv", "c.mkv", "d.mkv", "e.mkv"]));

    let turns = [
        queue.take(Duration::ZERO, 3).len(),
        queue.take(Duration::ZERO, 3).len(),
    ];

    assert_eq!(turns, [3, 2]);
}

// @behavior QUE-011
#[test]
fn should_take_nothing_in_a_turn_with_a_limit_of_none() {
    let queue = queue_handed(8, closed(&["a.mkv"]));

    let turns = [
        queue.take(Duration::ZERO, 0).len(),
        queue.take(Duration::ZERO, 1).len(),
    ];

    assert_eq!(turns, [0, 1]);
}
