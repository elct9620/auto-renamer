mod common;

use std::path::{Path, PathBuf};

use auto_renamer::{
    Config, FsTree, Processed, Renames, SkipReason, What, process_batch, replaced_pipelines,
};
use common::Sandbox;

const MOVE_AS_SHOW: &str = r#"[{ format = "{show}" }, "move"]"#;

/// A sandbox with a `source` and a `target` folder, and a watch over them with the extra settings.
struct Setup {
    sandbox: Sandbox,
    config: Config,
    renames: std::cell::RefCell<Renames>,
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
    Setup {
        sandbox,
        config,
        renames: Default::default(),
    }
}

/// A watch without a target, so that files are renamed where they are.
fn in_place(stages: &str) -> Setup {
    let sandbox = Sandbox::new();
    sandbox.make_dir("source");
    let text = format!(
        "[pipeline.p]\nstages = {stages}\n\n[watch.w]\nsource = \"{}\"\npipelines = [\"p\"]\nunit = \"source\"\n",
        sandbox.path("source").display(),
    );
    let config = Config::parse(&text).expect("the configuration should be accepted");
    Setup {
        sandbox,
        config,
        renames: Default::default(),
    }
}

impl Setup {
    fn process(&self, unit: &str, files: &[&str]) -> Vec<Processed> {
        let files: Vec<PathBuf> = files.iter().map(PathBuf::from).collect();
        process_batch(
            &FsTree,
            &self.config.watches()[0],
            Path::new(unit),
            &files,
            &mut self.renames.borrow_mut(),
        )
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
    let run = alpha("max_files = 1");
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
        "vars = { show = \"Alpha\" }\nmax_files = 1",
    );
    run.sandbox.write("source/Show/x.mkv", "a");
    run.sandbox.write("source/Show/y.mkv", "b");
    run.sandbox
        .write("source/Show/auto-renamer.toml", "max_files = 5\n");

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
        What::Skipped(SkipReason::Missing)
    );
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

const PREFIX: &str = r#"[{ format = "x{name}" }, "move"]"#;

/// Renames the file the way the prefix pipeline does, and says what became of it.
fn rename_again(run: &Setup, current: &mut String) -> What {
    let processed = run.process("", &[current.as_str()]);
    let what = run.what(&processed, current);
    if matches!(what, What::Moved(_) | What::MovedThenFailed { .. }) {
        *current = format!("x{current}");
    }
    what
}

// @behavior SVC-013
#[test]
fn should_leave_a_file_renamed_in_place_again_and_again_after_a_limit() {
    let run = in_place(PREFIX);
    run.sandbox.write("source/a.mkv", "video");
    let mut current = "a.mkv".to_string();

    for _ in 0..5 {
        assert!(matches!(rename_again(&run, &mut current), What::Moved(_)));
    }
    let sixth = rename_again(&run, &mut current);

    assert!(
        matches!(&sixth, What::Refused(reason) if reason.contains("in a row")),
        "{sixth:?}"
    );
    assert!(run.sandbox.exists(&format!("source/{current}")));
}

// @behavior SVC-014
#[test]
fn should_forget_the_renames_of_a_file_that_is_left_as_it_is() {
    let run = in_place(PREFIX);
    run.sandbox.write("source/a.mkv", "video");
    let mut current = "a.mkv".to_string();
    for _ in 0..4 {
        assert!(matches!(rename_again(&run, &mut current), What::Moved(_)));
    }

    run.sandbox.write(
        "source/auto-renamer.toml",
        "[pipeline.p]\nstages = [{ format = \"{name}\" }, \"move\"]\n",
    );
    let settled = run.process("", &[current.as_str()]);
    assert_eq!(run.what(&settled, &current), What::Unchanged);
    std::fs::remove_file(run.sandbox.path("source/auto-renamer.toml")).unwrap();

    for _ in 0..5 {
        assert!(matches!(rename_again(&run, &mut current), What::Moved(_)));
    }
}

// @behavior SVC-015
#[test]
fn should_report_a_file_moved_before_a_later_effect_failed_as_moved_with_the_failure() {
    let run = setup(
        r#"[{ format = "{show}" }, "move", "move"]"#,
        "vars = { show = \"Alpha\" }",
    );
    run.sandbox.write("source/Show/x.mkv", "video");

    let processed = run.process("Show", &["Show/x.mkv"]);

    assert!(matches!(
        run.what(&processed, "Show/x.mkv"),
        What::MovedThenFailed { to, reason }
            if to == run.sandbox.path("target/Show/Alpha.mkv") && reason.contains("is not there")
    ));
}

// @behavior SVC-016
#[test]
fn should_preview_where_each_file_would_go_with_a_pipeline_without_an_effect_stage() {
    let run = setup(r#"[{ format = "{show}" }]"#, "vars = { show = \"Alpha\" }");
    run.sandbox.write("source/Show/x.mkv", "video");

    let processed = run.process("Show", &["Show/x.mkv"]);

    assert_eq!(
        run.what(&processed, "Show/x.mkv"),
        What::Previewed(run.sandbox.path("target/Show/Alpha.mkv"))
    );
}

// @behavior SVC-017
#[test]
fn should_preview_a_file_whose_plan_is_where_it_already_is_in_a_dry_run() {
    let sandbox = Sandbox::new();
    sandbox.make_dir("source");
    let text = format!(
        "[pipeline.p]\nstages = [\"move\"]\n\n[watch.w]\nsource = \"{}\"\npipelines = [\"p\"]\ndry_run = true\n",
        sandbox.path("source").display(),
    );
    let config = Config::parse(&text).expect("the configuration should be accepted");
    sandbox.write("source/x.mkv", "video");

    let processed = process_batch(
        &FsTree,
        &config.watches()[0],
        Path::new(""),
        &[PathBuf::from("x.mkv")],
        &mut Renames::new(),
    );

    assert_eq!(
        processed[0].what,
        What::Previewed(sandbox.path("source/x.mkv"))
    );
}

// @behavior SVC-013
#[test]
fn should_count_a_rename_in_place_that_a_later_effect_failed_after() {
    let run = in_place(r#"[{ format = "x{name}" }, "move", "move"]"#);
    run.sandbox.write("source/a.mkv", "video");
    let mut current = "a.mkv".to_string();

    for _ in 0..5 {
        let what = rename_again(&run, &mut current);
        assert!(matches!(what, What::MovedThenFailed { .. }), "{what:?}");
    }
    let sixth = rename_again(&run, &mut current);

    assert!(
        matches!(&sixth, What::Refused(reason) if reason.contains("in a row")),
        "{sixth:?}"
    );
}

// @behavior SVC-018
#[test]
fn should_skip_a_link_in_a_batch_as_a_link() {
    let run = alpha("");
    run.sandbox.write("outside.mkv", "video");
    run.sandbox.make_dir("source/Show");
    std::os::unix::fs::symlink(
        run.sandbox.path("outside.mkv"),
        run.sandbox.path("source/Show/x.mkv"),
    )
    .unwrap();

    let processed = run.process("Show", &["Show/x.mkv"]);

    assert_eq!(
        run.what(&processed, "Show/x.mkv"),
        What::Skipped(SkipReason::Link)
    );
    assert!(run.sandbox.exists("source/Show/x.mkv"));
}

// @behavior SVC-019
#[test]
fn should_refuse_a_folder_configuration_one_byte_over_the_limit_on_disk() {
    let run = alpha("");
    run.sandbox.write("source/Show/x.mkv", "video");
    let setting = "[vars]\nshow = \"Beta\"\n";
    let padding = 64 * 1024 + 1 - setting.len() - 3;
    run.sandbox.write(
        "source/Show/auto-renamer.toml",
        &format!("{setting}# {}\n", "x".repeat(padding)),
    );

    let processed = run.process("Show", &["Show/x.mkv"]);

    assert!(matches!(
        run.what(&processed, "Show/auto-renamer.toml"),
        What::Refused(_)
    ));
}

// @behavior SVC-020
#[test]
fn should_not_count_a_folder_in_the_target_as_a_numbered_file() {
    let run = setup(
        r#"[{ next = { into = "episode", like = "e{episode}" } }, { format = "e{episode}" }, "move"]"#,
        "",
    );
    run.sandbox.write("source/Show/x.mkv", "video");
    run.sandbox.write("target/Show/e1.mkv", "earlier");
    run.sandbox.make_dir("target/Show/e7");

    run.process("Show", &["Show/x.mkv"]);

    assert!(run.sandbox.exists("target/Show/e2.mkv"));
}

// @behavior SVC-021
#[test]
fn should_name_the_pipelines_a_folder_configuration_replaces() {
    let sandbox = Sandbox::new();
    sandbox.make_dir("source/Show");
    sandbox.write(
        "source/Show/auto-renamer.toml",
        "[pipeline.video]\nstages = [\"move\"]\n",
    );
    let text = format!(
        "[pipeline.video]\nstages = []\n\n[pipeline.subtitle]\nstages = []\n\n[watch.w]\nsource = \"{}\"\npipelines = [\"video\", \"subtitle\"]\n",
        sandbox.path("source").display(),
    );
    let config = Config::parse(&text).expect("the configuration should be accepted");

    let replaced = replaced_pipelines(&FsTree, &config.watches()[0], Path::new("Show"));

    assert_eq!(replaced, ["video"]);
}
