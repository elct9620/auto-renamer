use std::path::{Path, PathBuf};
use std::sync::mpsc;

use auto_renamer::{Event, Translated, pass_changes, rewrites, translate};
use notify::EventKind;
use notify::event::{
    AccessKind, AccessMode, CreateKind, DataChange, MetadataKind, ModifyKind, RemoveKind,
    RenameMode,
};

fn notification(kind: EventKind, paths: &[&str]) -> notify::Event {
    paths.iter().fold(notify::Event::new(kind), |event, path| {
        event.add_path(PathBuf::from(path))
    })
}

/// What the source `/s` makes of a notification, where `/s/Show` is the only folder.
fn translated(kind: EventKind, paths: &[&str]) -> Vec<Translated> {
    translate(&notification(kind, paths), Path::new("/s"), |path| {
        path == Path::new("/s/Show")
    })
}

fn file(make: fn(PathBuf) -> Event, name: &str) -> Translated {
    Translated::Event(make(PathBuf::from(name)))
}

// @behavior TRN-001
#[test]
fn should_settle_a_file_closed_after_writing() {
    let kind = EventKind::Access(AccessKind::Close(AccessMode::Write));

    assert_eq!(
        translated(kind, &["/s/Show/a.mkv"]),
        [file(Event::Settled, "Show/a.mkv")]
    );
}

// @behavior TRN-002
#[test]
fn should_settle_a_file_moved_in() {
    let kind = EventKind::Modify(ModifyKind::Name(RenameMode::To));

    assert_eq!(
        translated(kind, &["/s/Show/a.mkv"]),
        [file(Event::Settled, "Show/a.mkv")]
    );
}

// @behavior TRN-003
#[test]
fn should_scan_a_folder_moved_in() {
    let kind = EventKind::Modify(ModifyKind::Name(RenameMode::To));

    assert_eq!(
        translated(kind, &["/s/Show"]),
        [Translated::Scan(PathBuf::from("/s/Show"))]
    );
}

// @behavior TRN-004
#[test]
fn should_treat_a_created_file_as_being_written() {
    let kind = EventKind::Create(CreateKind::File);

    assert_eq!(
        translated(kind, &["/s/Show/a.mkv"]),
        [file(Event::Writing, "Show/a.mkv")]
    );
}

// @behavior TRN-005
#[test]
fn should_scan_a_created_folder() {
    let kind = EventKind::Create(CreateKind::Folder);

    assert_eq!(
        translated(kind, &["/s/Show"]),
        [Translated::Scan(PathBuf::from("/s/Show"))]
    );
}

// @behavior TRN-006
#[test]
fn should_treat_a_change_of_data_as_a_write_in_progress() {
    let kind = EventKind::Modify(ModifyKind::Data(DataChange::Content));

    assert_eq!(
        translated(kind, &["/s/Show/a.mkv"]),
        [file(Event::Writing, "Show/a.mkv")]
    );
}

// @behavior TRN-007
#[test]
fn should_treat_a_file_moved_out_as_gone() {
    let kind = EventKind::Modify(ModifyKind::Name(RenameMode::From));

    assert_eq!(
        translated(kind, &["/s/Show/a.mkv"]),
        [file(Event::Gone, "Show/a.mkv")]
    );
}

// @behavior TRN-008
#[test]
fn should_treat_a_removed_file_as_gone() {
    let kind = EventKind::Remove(RemoveKind::File);

    assert_eq!(
        translated(kind, &["/s/Show/a.mkv"]),
        [file(Event::Gone, "Show/a.mkv")]
    );
}

// @behavior TRN-009
#[test]
fn should_report_a_rename_as_gone_at_the_old_name_and_settled_at_the_new() {
    let kind = EventKind::Modify(ModifyKind::Name(RenameMode::Both));

    assert_eq!(
        translated(kind, &["/s/a.mkv", "/s/b.mkv"]),
        [file(Event::Gone, "a.mkv"), file(Event::Settled, "b.mkv")]
    );
}

// @behavior TRN-010
#[test]
fn should_ignore_paths_outside_the_source() {
    let kind = EventKind::Access(AccessKind::Close(AccessMode::Write));

    assert_eq!(translated(kind, &["/elsewhere/a.mkv"]), []);
}

// @behavior TRN-011
#[test]
fn should_not_take_a_change_to_a_folder_for_a_write() {
    let kind = EventKind::Modify(ModifyKind::Data(DataChange::Any));

    assert_eq!(translated(kind, &["/s/Show"]), []);
}

// @behavior TRN-012
#[test]
fn should_ignore_other_kinds_of_notification() {
    let kind = EventKind::Access(AccessKind::Open(AccessMode::Read));

    assert_eq!(translated(kind, &["/s/Show/a.mkv"]), []);
}

fn rewrites_config(kind: EventKind, path: &str) -> bool {
    rewrites(&notification(kind, &[path]), Path::new("/c/config.toml"))
}

// @behavior TRN-013
#[test]
fn should_take_a_closed_write_as_rewriting_the_file() {
    let kind = EventKind::Access(AccessKind::Close(AccessMode::Write));

    assert!(rewrites_config(kind, "/c/config.toml"));
}

// @behavior TRN-014
#[test]
fn should_take_another_file_moved_over_it_as_rewriting_the_file() {
    let kind = EventKind::Modify(ModifyKind::Name(RenameMode::To));

    assert!(rewrites_config(kind, "/c/config.toml"));
}

// @behavior TRN-015
#[test]
fn should_not_take_a_read_as_rewriting_the_file() {
    for kind in [
        EventKind::Access(AccessKind::Open(AccessMode::Read)),
        EventKind::Access(AccessKind::Read),
        EventKind::Access(AccessKind::Close(AccessMode::Read)),
    ] {
        assert!(!rewrites_config(kind, "/c/config.toml"), "{kind:?}");
    }
}

// @behavior TRN-016
#[test]
fn should_not_take_a_notification_about_another_file_as_rewriting_it() {
    let kind = EventKind::Access(AccessKind::Close(AccessMode::Write));

    assert!(!rewrites_config(kind, "/c/other.toml"));
}

/// What reaches the runner of the notifications handed to the handler of the filesystem watcher.
fn passed_on(handed: Vec<notify::Result<notify::Event>>) -> Vec<notify::Result<notify::Event>> {
    let (sender, receiver) = mpsc::channel();
    let mut handler = pass_changes(sender);
    handed.into_iter().for_each(&mut handler);
    drop(handler);
    receiver.into_iter().collect()
}

fn passes(kind: EventKind) -> bool {
    !passed_on(vec![Ok(notification(kind, &["/s/Show/a.mkv"]))]).is_empty()
}

// @behavior TRN-017
#[test]
fn should_pass_on_a_notification_that_something_changed() {
    let closed = notification(
        EventKind::Access(AccessKind::Close(AccessMode::Write)),
        &["/s/Show/a.mkv"],
    );

    let passed: Vec<notify::Event> = passed_on(vec![Ok(closed.clone())])
        .into_iter()
        .flatten()
        .collect();

    assert_eq!(passed, [closed]);
}

// @behavior TRN-018
#[test]
fn should_drop_a_notification_that_nothing_changed() {
    let nothing_changed = [
        EventKind::Access(AccessKind::Open(AccessMode::Any)),
        EventKind::Access(AccessKind::Close(AccessMode::Read)),
        EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any)),
    ];

    let passed: Vec<EventKind> = nothing_changed
        .into_iter()
        .filter(|kind| passes(*kind))
        .collect();

    assert_eq!(passed, []);
}

// @behavior TRN-019
#[test]
fn should_pass_on_an_error() {
    let passed = passed_on(vec![Err(notify::Error::generic("the watcher failed"))]);

    assert!(matches!(passed.as_slice(), [Err(_)]), "{passed:?}");
}

// @behavior TRN-020
#[test]
fn should_pass_on_whatever_is_translated_or_rewrites_the_configuration() {
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
        .filter(|kind| answered(*kind) && !passes(*kind))
        .collect();

    assert_eq!(dropped, []);
}
