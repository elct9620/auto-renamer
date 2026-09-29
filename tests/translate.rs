use std::path::{Path, PathBuf};

use auto_renamer::{Event, Translated, translate};
use notify::EventKind;
use notify::event::{
    AccessKind, AccessMode, CreateKind, DataChange, ModifyKind, RemoveKind, RenameMode,
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
