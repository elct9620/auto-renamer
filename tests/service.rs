mod common;

use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use auto_renamer::{Config, Processed, What, process_batch};
use common::Sandbox;

const MOVE_AS_SHOW: &str = r#"[{ format = "{show}" }, "move"]"#;

/// A sandbox with a `source` and a `target` folder, and a watch over them with the extra settings.
struct Setup {
    sandbox: Sandbox,
    config: Config,
}

fn setup(stages: &str, extra: &str) -> Setup {
    let sandbox = Sandbox::new();
    sandbox.make_dir("source");
    sandbox.make_dir("target");
    let text = format!(
        "[pipeline.p]\nstages = {stages}\n\n[watch.w]\nsource = \"{}\"\ntarget = \"{}\"\npipelines = [\"p\"]\nunit = \"directory\"\n{extra}\n",
        sandbox.path("source").display(),
        sandbox.path("target").display(),
    );
    let config = Config::parse(&text).expect("the configuration should be accepted");
    Setup { sandbox, config }
}

impl Setup {
    fn process(&self, unit: &str, files: &[&str]) -> Vec<Processed> {
        let files: Vec<PathBuf> = files.iter().map(PathBuf::from).collect();
        process_batch(&self.config.watches()[0], Path::new(unit), &files)
    }

    fn what(&self, processed: &[Processed], origin: &str) -> What {
        processed
            .iter()
            .find(|entry| entry.origin == Path::new(origin))
            .unwrap_or_else(|| panic!("nothing was said of {origin}: {processed:?}"))
            .what
            .clone()
    }
}

fn alpha(extra: &str) -> Setup {
    setup(
        MOVE_AS_SHOW,
        &format!("vars = {{ show = \"Alpha\" }}\n{extra}"),
    )
}

// @behavior SVC-001
#[test]
fn should_plan_and_move_the_files_of_a_batch_to_the_target() {
    let run = alpha("");
    run.sandbox.write("source/Show/x.mkv", "video");

    let processed = run.process("Show", &["Show/x.mkv"]);

    assert_eq!(
        run.sandbox.read("target/Show/Alpha.mkv").as_deref(),
        Some("video")
    );
    assert!(!run.sandbox.exists("source/Show/x.mkv"));
    assert_eq!(
        run.what(&processed, "Show/x.mkv"),
        What::Moved(run.sandbox.path("target/Show/Alpha.mkv"))
    );
}

// @behavior SVC-002
#[test]
fn should_apply_the_folder_configuration_of_the_folder() {
    let run = alpha("");
    run.sandbox.write("source/Show/x.mkv", "video");
    run.sandbox
        .write("source/Show/auto-renamer.toml", "[vars]\nshow = \"Beta\"\n");

    run.process("Show", &["Show/x.mkv"]);

    assert!(run.sandbox.exists("target/Show/Beta.mkv"));
}

// @behavior SVC-003
#[test]
fn should_let_a_nearer_folder_configuration_beat_a_farther_one() {
    let run = alpha("");
    run.sandbox.write("source/Show/x.mkv", "video");
    run.sandbox
        .write("source/auto-renamer.toml", "[vars]\nshow = \"Beta\"\n");
    run.sandbox.write(
        "source/Show/auto-renamer.toml",
        "[vars]\nshow = \"Gamma\"\n",
    );

    run.process("Show", &["Show/x.mkv"]);

    assert!(run.sandbox.exists("target/Show/Gamma.mkv"));
}

// @behavior SVC-004
#[test]
fn should_leave_a_file_no_pipeline_claims() {
    let run = setup(
        r#"[{ filter = { ext = ["mkv"] } }, { format = "{show}" }, "move"]"#,
        "vars = { show = \"Alpha\" }",
    );
    run.sandbox.write("source/Show/notes.nfo", "text");

    let processed = run.process("Show", &["Show/notes.nfo"]);

    assert_eq!(run.what(&processed, "Show/notes.nfo"), What::Unclaimed);
    assert!(run.sandbox.exists("source/Show/notes.nfo"));
}

// @behavior SVC-005
#[test]
fn should_leave_a_refused_file_and_say_why() {
    let run = setup(MOVE_AS_SHOW, "");
    run.sandbox.write("source/Show/x.mkv", "video");

    let processed = run.process("Show", &["Show/x.mkv"]);

    assert!(
        matches!(run.what(&processed, "Show/x.mkv"), What::Refused(reason) if reason.contains("show"))
    );
    assert!(run.sandbox.exists("source/Show/x.mkv"));
}

// @behavior SVC-006
#[test]
fn should_move_nothing_in_a_dry_run() {
    let run = alpha("dry_run = true");
    run.sandbox.write("source/Show/x.mkv", "video");

    let processed = run.process("Show", &["Show/x.mkv"]);

    assert_eq!(
        run.what(&processed, "Show/x.mkv"),
        What::Previewed(run.sandbox.path("target/Show/Alpha.mkv"))
    );
    assert!(run.sandbox.exists("source/Show/x.mkv"));
    assert!(!run.sandbox.exists("target/Show/Alpha.mkv"));
}

// @behavior SVC-007
#[test]
fn should_leave_a_batch_over_the_limit_as_it_is() {
    let run = alpha("batch_max = 1");
    run.sandbox.write("source/Show/x.mkv", "a");
    run.sandbox.write("source/Show/y.mkv", "b");

    let processed = run.process("Show", &["Show/x.mkv", "Show/y.mkv"]);

    assert_eq!(run.what(&processed, "Show/x.mkv"), What::LeftTooLarge);
    assert_eq!(run.what(&processed, "Show/y.mkv"), What::LeftTooLarge);
    assert!(run.sandbox.exists("source/Show/x.mkv") && run.sandbox.exists("source/Show/y.mkv"));
}

// @behavior SVC-008
#[test]
fn should_let_a_folder_configuration_raise_the_limit_for_its_batch() {
    let run = setup(
        r#"[{ format = "{name}-{show}" }, "move"]"#,
        "vars = { show = \"Alpha\" }\nbatch_max = 1",
    );
    run.sandbox.write("source/Show/x.mkv", "a");
    run.sandbox.write("source/Show/y.mkv", "b");
    run.sandbox
        .write("source/Show/auto-renamer.toml", "batch_max = 5\n");

    let processed = run.process("Show", &["Show/x.mkv", "Show/y.mkv"]);

    assert!(matches!(run.what(&processed, "Show/x.mkv"), What::Moved(_)));
    assert!(matches!(run.what(&processed, "Show/y.mkv"), What::Moved(_)));
}

// @behavior SVC-009
#[test]
fn should_skip_a_file_that_vanished() {
    let run = alpha("");
    run.sandbox.make_dir("source/Show");

    let processed = run.process("Show", &["Show/x.mkv"]);

    assert_eq!(
        run.what(&processed, "Show/x.mkv"),
        What::Skipped("missing".to_string())
    );
}

// @behavior SVC-010
#[test]
fn should_skip_a_link_and_leave_it_untouched() {
    let run = alpha("");
    run.sandbox.write("elsewhere.mkv", "video");
    run.sandbox.make_dir("source/Show");
    symlink(
        run.sandbox.path("elsewhere.mkv"),
        run.sandbox.path("source/Show/x.mkv"),
    )
    .unwrap();

    let processed = run.process("Show", &["Show/x.mkv"]);

    assert_eq!(
        run.what(&processed, "Show/x.mkv"),
        What::Skipped("link".to_string())
    );
    assert!(run.sandbox.exists("source/Show/x.mkv"));
    assert!(run.sandbox.exists("elsewhere.mkv"));
}

// @behavior SVC-011
#[test]
fn should_number_after_what_the_target_already_holds() {
    let run = setup(
        r#"[{ next = { into = "episode", like = "{show} s01e{episode:02}" } }, { format = "{show} s01e{episode:02}" }, "move"]"#,
        "vars = { show = \"Alpha\" }",
    );
    run.sandbox.write("target/Show/Alpha s01e04.mkv", "old");
    run.sandbox.write("source/Show/x.mkv", "new");

    run.process("Show", &["Show/x.mkv"]);

    assert_eq!(
        run.sandbox.read("target/Show/Alpha s01e05.mkv").as_deref(),
        Some("new")
    );
}

// @behavior SVC-012
#[test]
fn should_ignore_and_report_a_folder_configuration_that_cannot_be_used() {
    let run = alpha("");
    run.sandbox.write("source/Show/x.mkv", "video");
    run.sandbox
        .write("source/Show/auto-renamer.toml", "this is not toml");

    let processed = run.process("Show", &["Show/x.mkv"]);

    assert!(run.sandbox.exists("target/Show/Alpha.mkv"));
    assert!(matches!(
        run.what(&processed, "Show/auto-renamer.toml"),
        What::Refused(_)
    ));
}
