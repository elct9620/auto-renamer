mod common;

use std::path::{Path, PathBuf};

use auto_renamer::{Applied, EffectError, Record, Roots, SkipReason, move_file};
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

    move_file(&reject(), &planned("x.mkv", "y.mkv"), &roots, false).unwrap();

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

    let done = move_file(&reject(), &planned("x.mkv", "x.mkv"), &roots, false);

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

    move_file(&suffix(), &planned("x", "y"), &roots(&sandbox), false).unwrap();

    assert_eq!(sandbox.read("target/y_v2").as_deref(), Some("new"));
}

// @behavior MV-009
#[test]
fn should_refuse_a_plan_that_leaves_the_target() {
    let sandbox = Sandbox::new();
    sandbox.write("source/x.mkv", "video");

    let done = move_file(
        &reject(),
        &planned("x.mkv", "../y.mkv"),
        &roots(&sandbox),
        false,
    );

    assert!(matches!(done, Err(EffectError::Unsafe(_))));
    assert!(sandbox.exists("source/x.mkv"));
    assert!(!sandbox.exists("y.mkv"));
}

// @behavior MV-010
#[test]
fn should_refuse_an_absolute_plan() {
    let sandbox = Sandbox::new();
    sandbox.write("source/x.mkv", "video");

    let done = move_file(
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
        &reject(),
        &planned("Season", "Other"),
        &roots(&sandbox),
        false,
    );

    assert_eq!(done, Ok(Applied::Skipped(SkipReason::NotAFile)));
    assert!(sandbox.exists("source/Season"));
}

// @behavior MV-014
#[test]
fn should_move_nothing_and_say_where_the_file_would_go_in_a_dry_run() {
    let sandbox = Sandbox::new();
    sandbox.write("source/x.mkv", "video");

    let done = move_file(
        &reject(),
        &planned("x.mkv", "Series/y.mkv"),
        &roots(&sandbox),
        true,
    );

    assert_eq!(
        done,
        Ok(Applied::Preview {
            from: sandbox.path("source/x.mkv"),
            to: sandbox.path("target/Series/y.mkv"),
        })
    );
    assert!(sandbox.exists("source/x.mkv"));
    assert!(!sandbox.exists("target"));
}

// @behavior MV-015
#[test]
fn should_refuse_a_file_that_is_not_there() {
    let sandbox = Sandbox::new();
    sandbox.make_dir("source");

    let done = move_file(
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
fn should_copy_and_then_remove_when_moving_between_filesystems() {
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

    move_file(&reject(), &planned("x.mkv", "y.mkv"), &roots, false).unwrap();

    assert_eq!(target.read("y.mkv").as_deref(), Some("some bytes"));
    assert!(!source.exists("x.mkv"));
    assert_eq!(target.names_in(""), ["y.mkv"]);
}

// @behavior MV-017
#[cfg(target_os = "linux")]
#[test]
fn should_keep_the_modification_time_when_moving_between_filesystems() {
    let Some(target) = other_filesystem() else {
        eprintln!("skipped: no second filesystem here");
        return;
    };
    let source = Sandbox::new();
    source.write("x.mkv", "some bytes");
    let past = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000_000);
    std::fs::File::options()
        .write(true)
        .open(source.path("x.mkv"))
        .unwrap()
        .set_modified(past)
        .unwrap();
    let roots = Roots {
        source: source.path(""),
        target: target.path(""),
    };

    move_file(&reject(), &planned("x.mkv", "y.mkv"), &roots, false).unwrap();

    let moved = std::fs::metadata(target.path("y.mkv"))
        .unwrap()
        .modified()
        .unwrap();
    assert_eq!(moved, past);
}

// @behavior MV-019
#[cfg(target_os = "linux")]
#[test]
fn should_not_carry_special_permission_bits_between_filesystems() {
    use std::os::unix::fs::PermissionsExt;

    let Some(target) = other_filesystem() else {
        eprintln!("skipped: no second filesystem here");
        return;
    };
    let source = Sandbox::new();
    source.write("x.mkv", "some bytes");
    std::fs::set_permissions(
        source.path("x.mkv"),
        std::fs::Permissions::from_mode(0o4755),
    )
    .unwrap();
    let roots = Roots {
        source: source.path(""),
        target: target.path(""),
    };

    move_file(&reject(), &planned("x.mkv", "y.mkv"), &roots, false).unwrap();

    let mode = std::fs::metadata(target.path("y.mkv"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o7000, 0);
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
    target.write(".y.mkv.part", "half");
    let roots = Roots {
        source: source.path(""),
        target: target.path(""),
    };

    move_file(&reject(), &planned("x.mkv", "y.mkv"), &roots, false).unwrap();

    assert_eq!(target.read("y.mkv").as_deref(), Some("some bytes"));
}

// @behavior MV-022
#[cfg(target_os = "linux")]
#[test]
fn should_bring_a_file_larger_than_one_part_whole_between_filesystems() {
    let Some(target) = other_filesystem() else {
        eprintln!("skipped: no second filesystem here");
        return;
    };
    let source = Sandbox::new();
    // No two stretches of it are alike, so a part missing, repeated or out of place shows.
    let bytes: Vec<u8> = (0..40 * 1024 * 1024 + 123_u32)
        .map(|place| (place.wrapping_mul(2_654_435_761) >> 24) as u8)
        .collect();
    std::fs::write(source.path("x.mkv"), &bytes).unwrap();
    let roots = Roots {
        source: source.path(""),
        target: target.path(""),
    };

    move_file(&reject(), &planned("x.mkv", "y.mkv"), &roots, false).unwrap();

    let moved = std::fs::read(target.path("y.mkv")).unwrap();
    assert_eq!(moved.len(), bytes.len());
    assert!(moved == bytes, "the bytes differ");
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

    move_file(&reject(), &planned("x.mkv", "y.mkv"), &roots, false).unwrap();

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
