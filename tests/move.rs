mod common;

use std::path::{Path, PathBuf};

use auto_renamer::{Applied, EffectError, FsTree, Record, Roots, SkipReason, move_file};
use common::{Sandbox, move_stage, record};

fn reject() -> auto_renamer::stages::Move {
    move_stage(r#""move""#)
}

fn suffix() -> auto_renamer::stages::Move {
    move_stage(r#"{ move = { on_conflict = "suffix" } }"#)
}

/// A record for a file of the source, planned to go to `plan`.
fn planned(origin: &str, plan: &str) -> Record {
    let mut input = record(origin);
    input.set_plan(PathBuf::from(plan));
    input
}

fn roots(sandbox: &Sandbox) -> Roots {
    Roots {
        source: sandbox.path("source"),
        target: sandbox.path("target"),
    }
}

// @behavior MV-001
#[test]
fn should_move_a_file_to_its_plan_under_the_target_making_the_folders() {
    let sandbox = Sandbox::new();
    sandbox.write("source/Series/x.mkv", "video");

    let done = move_file(
        &FsTree,
        &reject(),
        &planned("Series/x.mkv", "Series/Alpha/y.mkv"),
        &roots(&sandbox),
        false,
    );

    assert!(matches!(done, Ok(Applied::Moved { .. })));
    assert!(sandbox.exists("target/Series/Alpha/y.mkv"));
}

// @behavior MV-002
#[test]
fn should_remove_the_file_from_the_source_and_keep_its_content() {
    let sandbox = Sandbox::new();
    sandbox.write("source/x.mkv", "some bytes");

    move_file(
        &FsTree,
        &reject(),
        &planned("x.mkv", "y.mkv"),
        &roots(&sandbox),
        false,
    )
    .unwrap();

    assert!(!sandbox.exists("source/x.mkv"));
    assert_eq!(sandbox.read("target/y.mkv").as_deref(), Some("some bytes"));
}

// @behavior MV-003
#[test]
fn should_rename_in_place_without_a_separate_target() {
    let sandbox = Sandbox::new();
    sandbox.write("source/x.mkv", "video");
    let roots = Roots {
        source: sandbox.path("source"),
        target: sandbox.path("source"),
    };

    move_file(
        &FsTree,
        &reject(),
        &planned("x.mkv", "y.mkv"),
        &roots,
        false,
    )
    .unwrap();

    assert!(sandbox.exists("source/y.mkv"));
    assert!(!sandbox.exists("source/x.mkv"));
}

// @behavior MV-004
#[test]
fn should_leave_a_file_already_at_its_plan() {
    let sandbox = Sandbox::new();
    sandbox.write("source/x.mkv", "video");
    let roots = Roots {
        source: sandbox.path("source"),
        target: sandbox.path("source"),
    };

    let done = move_file(
        &FsTree,
        &reject(),
        &planned("x.mkv", "x.mkv"),
        &roots,
        false,
    );

    assert_eq!(done, Ok(Applied::Unchanged(sandbox.path("source/x.mkv"))));
    assert_eq!(sandbox.read("source/x.mkv").as_deref(), Some("video"));
}

// @behavior MV-005
#[test]
fn should_not_overwrite_a_file_at_the_plan() {
    let sandbox = Sandbox::new();
    sandbox.write("source/x.mkv", "new");
    sandbox.write("target/y.mkv", "old");

    let done = move_file(
        &FsTree,
        &reject(),
        &planned("x.mkv", "y.mkv"),
        &roots(&sandbox),
        false,
    );

    assert_eq!(
        done,
        Err(EffectError::Conflict(sandbox.path("target/y.mkv")))
    );
    assert_eq!(sandbox.read("source/x.mkv").as_deref(), Some("new"));
    assert_eq!(sandbox.read("target/y.mkv").as_deref(), Some("old"));
}

// @behavior MV-006
#[test]
fn should_settle_a_conflict_with_a_suffix_when_asked() {
    let sandbox = Sandbox::new();
    sandbox.write("source/x.mkv", "new");
    sandbox.write("target/y.mkv", "old");

    move_file(
        &FsTree,
        &suffix(),
        &planned("x.mkv", "y.mkv"),
        &roots(&sandbox),
        false,
    )
    .unwrap();

    assert_eq!(sandbox.read("target/y_v2.mkv").as_deref(), Some("new"));
    assert_eq!(sandbox.read("target/y.mkv").as_deref(), Some("old"));
}

// @behavior MV-007
#[test]
fn should_refuse_a_suffix_that_also_conflicts() {
    let sandbox = Sandbox::new();
    sandbox.write("source/x.mkv", "new");
    sandbox.write("target/y.mkv", "old");
    sandbox.write("target/y_v2.mkv", "older");

    let done = move_file(
        &FsTree,
        &suffix(),
        &planned("x.mkv", "y.mkv"),
        &roots(&sandbox),
        false,
    );

    assert_eq!(
        done,
        Err(EffectError::Conflict(sandbox.path("target/y_v2.mkv")))
    );
    assert_eq!(sandbox.read("target/y_v2.mkv").as_deref(), Some("older"));
    assert_eq!(sandbox.read("source/x.mkv").as_deref(), Some("new"));
}

// @behavior MV-008
#[test]
fn should_put_the_suffix_before_the_extension() {
    let sandbox = Sandbox::new();
    sandbox.write("source/x", "new");
    sandbox.write("target/y", "old");

    move_file(
        &FsTree,
        &suffix(),
        &planned("x", "y"),
        &roots(&sandbox),
        false,
    )
    .unwrap();

    assert_eq!(sandbox.read("target/y_v2").as_deref(), Some("new"));
}

// @behavior MV-009
#[test]
fn should_refuse_a_plan_that_leaves_the_target() {
    let sandbox = Sandbox::new();
    sandbox.write("source/x.mkv", "video");

    let done = move_file(
        &FsTree,
        &reject(),
        &planned("x.mkv", "../y.mkv"),
        &roots(&sandbox),
        false,
    );

    assert!(matches!(done, Err(EffectError::Unsafe(_))));
    assert!(sandbox.exists("source/x.mkv"));
    assert!(!sandbox.exists("y.mkv"));
}

// @behavior MV-009
#[test]
fn should_refuse_an_absolute_plan() {
    let sandbox = Sandbox::new();
    sandbox.write("source/x.mkv", "video");

    let done = move_file(
        &FsTree,
        &reject(),
        &planned("x.mkv", "/tmp/auto-renamer-escape.mkv"),
        &roots(&sandbox),
        false,
    );

    assert!(matches!(done, Err(EffectError::Unsafe(_))));
    assert!(sandbox.exists("source/x.mkv"));
    assert!(!Path::new("/tmp/auto-renamer-escape.mkv").exists());
}

// @behavior MV-011
#[test]
fn should_not_move_a_link() {
    let sandbox = Sandbox::new();
    sandbox.write("elsewhere/secret", "secret");
    sandbox.make_dir("source");
    std::os::unix::fs::symlink(
        sandbox.path("elsewhere/secret"),
        sandbox.path("source/x.mkv"),
    )
    .unwrap();

    let done = move_file(
        &FsTree,
        &reject(),
        &planned("x.mkv", "y.mkv"),
        &roots(&sandbox),
        false,
    );

    assert_eq!(done, Ok(Applied::Skipped(SkipReason::Link)));
    assert!(sandbox.exists("source/x.mkv"));
    assert!(!sandbox.exists("target/y.mkv"));
}

// @behavior MV-012
#[test]
fn should_not_move_a_file_below_a_linked_folder() {
    let sandbox = Sandbox::new();
    sandbox.write("elsewhere/x.mkv", "video");
    sandbox.make_dir("source");
    std::os::unix::fs::symlink(sandbox.path("elsewhere"), sandbox.path("source/Linked")).unwrap();

    let done = move_file(
        &FsTree,
        &reject(),
        &planned("Linked/x.mkv", "y.mkv"),
        &roots(&sandbox),
        false,
    );

    assert_eq!(done, Ok(Applied::Skipped(SkipReason::Link)));
    assert!(sandbox.exists("elsewhere/x.mkv"));
    assert!(!sandbox.exists("target/y.mkv"));
}

// @behavior MV-013
#[test]
fn should_not_move_a_folder() {
    let sandbox = Sandbox::new();
    sandbox.make_dir("source/Season");

    let done = move_file(
        &FsTree,
        &reject(),
        &planned("Season", "Other"),
        &roots(&sandbox),
        false,
    );

    assert_eq!(done, Ok(Applied::Skipped(SkipReason::NotAFile)));
    assert!(sandbox.exists("source/Season"));
}

// @behavior MV-015
#[test]
fn should_refuse_a_file_that_is_not_there() {
    let sandbox = Sandbox::new();
    sandbox.make_dir("source");

    let done = move_file(
        &FsTree,
        &reject(),
        &planned("x.mkv", "y.mkv"),
        &roots(&sandbox),
        false,
    );

    assert_eq!(
        done,
        Err(EffectError::Missing(sandbox.path("source/x.mkv")))
    );
}

/// A folder on another filesystem than the temporary folder, where there is one to be had.
#[cfg(target_os = "linux")]
fn other_filesystem() -> Option<Sandbox> {
    use std::os::unix::fs::MetadataExt;

    let shared = Path::new("/dev/shm");
    let here = std::fs::metadata(std::env::temp_dir()).ok()?.dev();
    let there = std::fs::metadata(shared).ok()?.dev();
    (here != there).then(|| Sandbox::under(shared))
}

// @behavior MV-016
#[cfg(target_os = "linux")]
#[test]
fn should_move_through_a_temporary_name_between_filesystems() {
    let Some(target) = other_filesystem() else {
        eprintln!("skipped: no second filesystem here");
        return;
    };
    let source = Sandbox::new();
    source.write("x.mkv", "some bytes");
    let roots = Roots {
        source: source.path(""),
        target: target.path(""),
    };

    move_file(
        &FsTree,
        &reject(),
        &planned("x.mkv", "y.mkv"),
        &roots,
        false,
    )
    .unwrap();

    assert_eq!(target.read("y.mkv").as_deref(), Some("some bytes"));
    assert!(!source.exists("x.mkv"));
    assert_eq!(target.names_in(""), ["y.mkv"]);
}

// @behavior MV-020
#[cfg(target_os = "linux")]
#[test]
fn should_not_be_blocked_by_a_temporary_file_an_earlier_move_left() {
    let Some(target) = other_filesystem() else {
        eprintln!("skipped: no second filesystem here");
        return;
    };
    let source = Sandbox::new();
    source.write("x.mkv", "some bytes");
    target.write(".y.mkv.1.part", "half");
    let roots = Roots {
        source: source.path(""),
        target: target.path(""),
    };

    move_file(
        &FsTree,
        &reject(),
        &planned("x.mkv", "y.mkv"),
        &roots,
        false,
    )
    .unwrap();

    assert_eq!(target.read("y.mkv").as_deref(), Some("some bytes"));
}

// @behavior MV-021
#[cfg(target_os = "linux")]
#[test]
fn should_be_seen_as_a_file_moved_in_when_renamed_within_a_folder() {
    use auto_renamer::{Event, Translated, translate};
    use notify::Watcher;

    let sandbox = Sandbox::new();
    sandbox.write("x.mkv", "some bytes");
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(sender).unwrap();
    watcher
        .watch(&sandbox.path(""), notify::RecursiveMode::Recursive)
        .unwrap();
    let roots = Roots {
        source: sandbox.path(""),
        target: sandbox.path(""),
    };

    move_file(
        &FsTree,
        &reject(),
        &planned("x.mkv", "y.mkv"),
        &roots,
        false,
    )
    .unwrap();

    std::thread::sleep(std::time::Duration::from_millis(500));
    let seen: Vec<Translated> = receiver
        .try_iter()
        .filter_map(Result::ok)
        .flat_map(|notification| translate(&notification, &sandbox.path(""), |_| false))
        .collect();
    let settled = Translated::Event(Event::Settled("y.mkv".into()));
    let writing = Translated::Event(Event::Writing("y.mkv".into()));
    assert!(seen.contains(&settled), "{seen:?}");
    assert!(!seen.contains(&writing), "{seen:?}");
}

/// A folder on a filesystem of 1 MiB, where the test container has one.
#[cfg(target_os = "linux")]
fn small_filesystem() -> Option<Sandbox> {
    Path::new("/small")
        .is_dir()
        .then(|| Sandbox::under(Path::new("/small")))
}

// @behavior MV-028
#[cfg(target_os = "linux")]
#[test]
fn should_leave_the_file_in_the_source_when_mv_cannot_finish() {
    let Some(target) = small_filesystem() else {
        eprintln!("skipped: no small filesystem here");
        return;
    };
    let source = Sandbox::new();
    std::fs::write(source.path("x.mkv"), vec![7u8; 2 * 1024 * 1024]).unwrap();
    let roots = Roots {
        source: source.path(""),
        target: target.path(""),
    };

    let moved = move_file(
        &FsTree,
        &reject(),
        &planned("x.mkv", "y.mkv"),
        &roots,
        false,
    );

    assert!(moved.is_err());
    assert!(source.exists("x.mkv"));
    assert!(target.names_in("").is_empty());
}
