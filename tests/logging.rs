#![cfg(target_os = "linux")]

mod common;

use std::thread;
use std::time::Duration;

use common::{Program, Sandbox, eventually};

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
