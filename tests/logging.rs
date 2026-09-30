#![cfg(target_os = "linux")]

mod common;

use std::thread;
use std::time::Duration;

use common::{Program, Sandbox, eventually, eventually_within};

// @behavior RUN-005
#[test]
fn should_not_follow_a_linked_folder() {
    let sandbox = Sandbox::new();
    sandbox.make_dir("outside");
    sandbox.make_dir("source");
    std::os::unix::fs::symlink(sandbox.path("outside"), sandbox.path("source/Linked")).unwrap();
    let program = Program::start(&sandbox, r#"["move"]"#, "");

    sandbox.write("outside/a.mkv", "video");
    thread::sleep(Duration::from_secs(5));

    assert!(sandbox.exists("outside/a.mkv"));
    assert_eq!(sandbox.names_in("target"), Vec::<String>::new());
    assert!(!program.log().contains("Linked"), "{}", program.log());
}

// @behavior RUN-010
#[test]
fn should_report_where_a_file_would_go_in_a_dry_run() {
    let sandbox = Sandbox::new();
    let program = Program::start(&sandbox, r#"["move"]"#, "dry_run = true");

    sandbox.write("source/a.mkv", "video");

    assert!(
        eventually(|| program.log().contains("a.mkv would go to")),
        "{}",
        program.log()
    );
    assert!(sandbox.exists("source/a.mkv"));
}

// @behavior RUN-011
#[test]
fn should_report_why_a_file_was_refused() {
    let sandbox = Sandbox::new();
    let program = Program::start(&sandbox, r#"[{ format = "{show}" }, "move"]"#, "");

    sandbox.write("source/a.mkv", "video");

    assert!(
        eventually(|| program.log().contains("a.mkv refused")),
        "{}",
        program.log()
    );
    assert!(sandbox.exists("source/a.mkv"));
}

// @behavior RUN-012
#[test]
fn should_report_a_file_that_no_pipeline_claims() {
    let sandbox = Sandbox::new();
    let program = Program::start(&sandbox, r#"[{ filter = { ext = ["mkv"] } }, "move"]"#, "");

    sandbox.write("source/notes.nfo", "text");

    assert!(
        eventually(|| program.log().contains("notes.nfo left")),
        "{}",
        program.log()
    );
}

// @behavior RUN-013
#[test]
fn should_read_the_configuration_again_once_for_one_change() {
    let sandbox = Sandbox::new();
    let program = Program::start(&sandbox, r#"["move"]"#, "");
    let changed = format!("{}\n# changed\n", sandbox.read("config.toml").unwrap());

    sandbox.write("config.toml", &changed);
    thread::sleep(Duration::from_secs(5));

    let reads = program
        .log()
        .matches("the configuration was read again")
        .count();
    assert_eq!(reads, 1, "{}", program.log());
}

// @behavior RUN-014
#[test]
fn should_read_the_configuration_again_on_a_hangup() {
    let sandbox = Sandbox::new();
    // The configuration is reached through a link, so that changing the file it points at sends
    // no notification to the folder the program watches for it.
    let program = Program::start(&sandbox, r#"["move"]"#, "");
    sandbox.make_dir("real");
    std::fs::rename(
        sandbox.path("config.toml"),
        sandbox.path("real/config.toml"),
    )
    .unwrap();
    std::os::unix::fs::symlink(
        sandbox.path("real/config.toml"),
        sandbox.path("config.toml"),
    )
    .unwrap();
    thread::sleep(Duration::from_secs(3));
    let renamed = sandbox
        .read("real/config.toml")
        .unwrap()
        .replace(r#"["move"]"#, r#"[{ format = "renamed" }, "move"]"#);

    sandbox.write("real/config.toml", &renamed);
    program.signal("HUP");
    thread::sleep(Duration::from_secs(1));
    sandbox.write("source/a.mkv", "video");

    assert!(
        eventually(|| sandbox.exists("target/renamed.mkv")),
        "{}",
        program.log()
    );
}

// @behavior RUN-015
#[test]
fn should_not_let_a_file_renamed_in_place_hold_the_folder() {
    let sandbox = Sandbox::new();
    let program = Program::start_in_place(&sandbox, r#"[{ strip = {} }, "move"]"#);

    sandbox.write("source/[Team] a.mkv", "one");
    assert!(
        eventually(|| sandbox.exists("source/a.mkv")),
        "{}",
        program.log()
    );
    sandbox.write("source/[Team] b.mkv", "two");

    assert!(
        eventually(|| sandbox.exists("source/b.mkv")),
        "{}",
        program.log()
    );
}

// @behavior RUN-016
#[test]
fn should_stop_a_pipeline_that_names_its_own_result_again() {
    let sandbox = Sandbox::new();
    let program = Program::start_in_place(&sandbox, r#"[{ format = "x{name}" }, "move"]"#);

    sandbox.write("source/a.mkv", "video");

    assert!(
        eventually_within(30, || program.log().contains("times in a row")),
        "{}",
        program.log()
    );
    let names = sandbox.names_in("source");
    assert_eq!(names.len(), 1, "{names:?}");
    assert_eq!(names[0].matches('x').count(), 5, "{names:?}");
}

// @behavior RUN-020
#[test]
fn should_name_a_folder_that_cannot_be_watched() {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    let sandbox = Sandbox::new();
    let program = Program::start_unprivileged(&sandbox, r#"["move"]"#);

    std::fs::DirBuilder::new()
        .mode(0o000)
        .create(sandbox.path("source/Locked"))
        .unwrap();

    let named =
        eventually(|| program.log().contains("not watched") && program.log().contains("Locked"));
    let open = std::fs::Permissions::from_mode(0o755);
    std::fs::set_permissions(sandbox.path("source/Locked"), open).unwrap();
    assert!(named, "{}", program.log());
}

// @behavior RUN-025
#[test]
fn should_stop_for_a_folder_that_cannot_be_watched_at_start() {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    let sandbox = Sandbox::new();
    sandbox.make_dir("source");
    std::fs::DirBuilder::new()
        .mode(0o000)
        .create(sandbox.path("source/Locked"))
        .unwrap();

    let mut program = Program::start_unprivileged(&sandbox, r#"["move"]"#);

    let failed = program.exits_with_a_failure();
    let open = std::fs::Permissions::from_mode(0o755);
    std::fs::set_permissions(sandbox.path("source/Locked"), open).unwrap();
    assert!(failed, "{}", program.log());
    assert!(program.log().contains("Locked"), "{}", program.log());
}
