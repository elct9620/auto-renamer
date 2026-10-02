mod common;

use std::path::{Path, PathBuf};

use auto_renamer::{
    Config, EffectError, FsTree, Kind, Processed, Renames, SkipReason, Tree, What, process_batch,
    replaced_pipelines,
};
use common::Sandbox;

const MOVE_AS_SHOW: &str = r#"[{ format = "{show}" }]"#;

/// A sandbox with a `source` and a `target` folder, and a watch over them with the extra settings.
struct Setup {
    sandbox: Sandbox,
    config: Config,
    renames: std::cell::RefCell<Renames>,
}

fn setup(stages: &str, extra: &str) -> Setup {
    setup_route(stages, "", extra)
}

/// A watch whose one route moves into `target`, with `route` written into the route.
fn setup_route(stages: &str, route: &str, extra: &str) -> Setup {
    let sandbox = Sandbox::new();
    sandbox.make_dir("source");
    sandbox.make_dir("target");
    let text = format!(
        "[pipeline.p]\nstages = {stages}\n\n[target.t]\npath = \"{}\"\n\n[watch.w]\nsource = \"{}\"\nroutes = [{{ pipeline = \"p\", move = \"t\"{route} }}]\nunit = \"directory\"\n{extra}\n",
        sandbox.path("target").display(),
        sandbox.path("source").display(),
    );
    let config = Config::parse(&text).expect("the configuration should be accepted");
    Setup {
        sandbox,
        config,
        renames: Default::default(),
    }
}

/// A watch whose route names no target, so that files are renamed where they are.
fn in_place(stages: &str) -> Setup {
    in_place_route(stages, "")
}

fn in_place_route(stages: &str, route: &str) -> Setup {
    let sandbox = Sandbox::new();
    sandbox.make_dir("source");
    let text = format!(
        "[pipeline.p]\nstages = {stages}\n\n[watch.w]\nsource = \"{}\"\nroutes = [{{ pipeline = \"p\"{route} }}]\nunit = \"source\"\n",
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
        self.process_on(&FsTree, unit, files)
    }

    fn process_on(&self, tree: &dyn Tree, unit: &str, files: &[&str]) -> Vec<Processed> {
        let files: Vec<PathBuf> = files.iter().map(PathBuf::from).collect();
        process_batch(
            tree,
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
        r#"[{ filter = { ext = ["mkv"] } }, { format = "{show}" }]"#,
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
        r#"[{ format = "{name}-{show}" }]"#,
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
        r#"[{ next = { into = "episode", like = "{show} s01e{episode:02}" } }, { format = "{show} s01e{episode:02}" }]"#,
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

const PREFIX: &str = r#"[{ format = "x{name}" }]"#;

/// Renames the file the way the prefix pipeline does, and says what became of it.
fn rename_again(run: &Setup, current: &mut String) -> What {
    rename_again_on(&FsTree, run, current)
}

/// Processes the file once more through `tree`, following it to its new name when it was renamed.
fn rename_again_on(tree: &dyn Tree, run: &Setup, current: &mut String) -> What {
    let processed = run.process_on(tree, "", &[current.as_str()]);
    let what = run.what(&processed, current);
    if matches!(what, What::Moved(_) | What::MovedThenFailed { .. }) {
        let path = Path::new(current.as_str());
        let name = path.file_name().unwrap().to_str().unwrap();
        *current = path
            .with_file_name(format!("x{name}"))
            .display()
            .to_string();
    }
    what
}

/// The filesystem, except that every folder looks empty and none can be removed, so a cleanup fails.
struct Unremovable;

impl Tree for Unremovable {
    fn kind(&self, path: &Path) -> std::io::Result<Kind> {
        FsTree.kind(path)
    }

    fn modified(&self, path: &Path) -> std::io::Result<chrono::DateTime<chrono::Utc>> {
        FsTree.modified(path)
    }

    fn read(&self, path: &Path, limit: u64) -> std::io::Result<String> {
        FsTree.read(path, limit)
    }

    fn files(&self, folder: &Path) -> Vec<String> {
        FsTree.files(folder)
    }

    fn is_empty(&self, _: &Path) -> std::io::Result<bool> {
        Ok(true)
    }

    fn place(&self, from: &Path, to: &Path) -> Result<(), EffectError> {
        FsTree.place(from, to)
    }

    fn remove_folder(&self, _: &Path) -> std::io::Result<()> {
        Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
    }
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
        "[pipeline.p]\nstages = [{ format = \"{name}\" }]\n",
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
fn should_report_a_file_moved_before_its_cleanup_failed_as_moved_with_the_failure() {
    let run = setup_route(
        r#"[{ format = "{show}" }]"#,
        ", cleanup = {}",
        "vars = { show = \"Alpha\" }",
    );
    run.sandbox.write("source/Show/x.mkv", "video");

    let processed = run.process_on(&Unremovable, "Show", &["Show/x.mkv"]);

    assert!(matches!(
        run.what(&processed, "Show/x.mkv"),
        What::MovedThenFailed { to, reason }
            if to == run.sandbox.path("target/Show/Alpha.mkv") && reason.contains("remove")
    ));
}

// @behavior SVC-017
#[test]
fn should_preview_a_file_whose_plan_is_where_it_already_is_in_a_dry_run() {
    let sandbox = Sandbox::new();
    sandbox.make_dir("source");
    let text = format!(
        "[pipeline.p]\nstages = []\n\n[watch.w]\nsource = \"{}\"\nroutes = [{{ pipeline = \"p\" }}]\ndry_run = true\n",
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
    let run = in_place_route(r#"[{ format = "x{name}" }]"#, ", cleanup = {}");
    run.sandbox.write("source/Sub/a.mkv", "video");
    let mut current = "Sub/a.mkv".to_string();

    for _ in 0..5 {
        let what = rename_again_on(&Unremovable, &run, &mut current);
        assert!(matches!(what, What::MovedThenFailed { .. }), "{what:?}");
    }
    let sixth = rename_again_on(&Unremovable, &run, &mut current);

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
        r#"[{ next = { into = "episode", like = "e{episode}" } }, { format = "e{episode}" }]"#,
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
        "[pipeline.video]\nstages = []\n",
    );
    let text = format!(
        "[pipeline.video]\nstages = []\n\n[pipeline.subtitle]\nstages = []\n\n[watch.w]\nsource = \"{}\"\nroutes = [{{ pipeline = \"video\" }}, {{ pipeline = \"subtitle\" }}]\n",
        sandbox.path("source").display(),
    );
    let config = Config::parse(&text).expect("the configuration should be accepted");

    let replaced = replaced_pipelines(&FsTree, &config.watches()[0], Path::new("Show"));

    assert_eq!(replaced, ["video"]);
}

/// A watch whose route runs `stages` into `target`, which already holds `Show/Alpha.mkv`, and sends what it
/// refuses to a rejected route moving into `conflict`, with `rejected` written into that route and `tables`
/// after the watch.
fn refusing(stages: &str, route: &str, rejected: &str, tables: &str) -> Setup {
    let sandbox = Sandbox::new();
    sandbox.make_dir("source");
    sandbox.make_dir("conflict");
    sandbox.write("target/Show/Alpha.mkv", "old");
    let text = format!(
        "[pipeline.p]\nstages = {stages}\n\n[target.t]\npath = \"{}\"\n\n[target.c]\npath = \"{}\"\n\n\
         [watch.w]\nsource = \"{}\"\nroutes = [{{ pipeline = \"p\", move = \"t\"{route}, rejected = {{ move = \"c\"{rejected} }} }}]\n\
         unit = \"directory\"\nvars = {{ show = \"Alpha\" }}\n\n{tables}",
        sandbox.path("target").display(),
        sandbox.path("conflict").display(),
        sandbox.path("source").display(),
    );
    let config = Config::parse(&text).expect("the configuration should be accepted");
    Setup {
        sandbox,
        config,
        renames: Default::default(),
    }
}

// @behavior SVC-022
#[test]
fn should_send_a_refused_file_to_the_rejected_route_with_its_own_name() {
    let run = refusing(MOVE_AS_SHOW, "", "", "");
    run.sandbox.write("source/Show/x.mkv", "new");

    let processed = run.process("Show", &["Show/x.mkv"]);

    assert_eq!(
        run.what(&processed, "Show/x.mkv"),
        What::Moved(run.sandbox.path("conflict/Show/x.mkv"))
    );
}

// @behavior SVC-023
#[test]
fn should_let_a_rejected_file_carry_its_plan_and_why_it_was_refused() {
    let run = refusing(
        MOVE_AS_SHOW,
        "",
        ", pipeline = \"r\"",
        "[pipeline.r]\nstages = [{ regex = { from = \"planned\", pattern = '(?<shown>[^/]+)\\.mkv$' } }, { format = \"{reason} {shown}\" }]\n",
    );
    run.sandbox.write("source/Show/x.mkv", "new");

    let processed = run.process("Show", &["Show/x.mkv"]);

    assert_eq!(
        run.what(&processed, "Show/x.mkv"),
        What::Moved(run.sandbox.path("conflict/Show/move Alpha.mkv"))
    );
}

// @behavior SVC-024
#[test]
fn should_let_a_rejected_file_keep_the_fields_found_before_it_was_refused() {
    let run = refusing(
        r#"[{ number = { into = "episode" } }, { format = "{show}" }]"#,
        "",
        ", pipeline = \"r\"",
        "[pipeline.r]\nstages = [{ format = \"{episode}\" }]\n",
    );
    run.sandbox.write("source/Show/x 07.mkv", "new");

    let processed = run.process("Show", &["Show/x 07.mkv"]);

    assert_eq!(
        run.what(&processed, "Show/x 07.mkv"),
        What::Moved(run.sandbox.path("conflict/Show/7.mkv"))
    );
}

// @behavior SVC-025
#[test]
fn should_let_a_rejected_route_take_only_what_its_pipeline_claims() {
    let run = refusing(
        MOVE_AS_SHOW,
        "",
        ", pipeline = \"r\"",
        "[pipeline.r]\nstages = [{ filter = { reason = [\"format\"] } }]\n",
    );
    run.sandbox.write("source/Show/x.mkv", "new");

    let processed = run.process("Show", &["Show/x.mkv"]);

    assert!(matches!(
        run.what(&processed, "Show/x.mkv"),
        What::Refused(reason) if reason.starts_with("move:")
    ));
    assert!(run.sandbox.exists("source/Show/x.mkv"));
}

// @behavior SVC-026
#[test]
fn should_leave_a_file_refused_again_on_its_rejected_route() {
    let run = refusing(
        MOVE_AS_SHOW,
        "",
        ", pipeline = \"r\"",
        "[pipeline.r]\nstages = [{ format = \"{missing}\" }]\n",
    );
    run.sandbox.write("source/Show/x.mkv", "new");

    let processed = run.process("Show", &["Show/x.mkv"]);

    assert!(matches!(
        run.what(&processed, "Show/x.mkv"),
        What::Refused(reason) if reason.starts_with("format:")
    ));
    assert!(run.sandbox.exists("source/Show/x.mkv"));
}

// @behavior SVC-027
#[test]
fn should_clean_up_on_a_rejected_route_only_when_it_says_so() {
    let run = refusing(
        r#"[{ format = "{show}" }, { lift = 1 }]"#,
        ", cleanup = {}",
        "",
        "",
    );
    run.sandbox.write("source/Show/Rel/x.mkv", "new");

    let processed = run.process("Show/Rel", &["Show/Rel/x.mkv"]);

    assert_eq!(
        run.what(&processed, "Show/Rel/x.mkv"),
        What::Moved(run.sandbox.path("conflict/Show/Rel/x.mkv"))
    );
    assert!(run.sandbox.exists("source/Show/Rel"));
}

// @behavior SVC-028
#[test]
fn should_report_a_refused_file_with_the_plan_it_had() {
    let run = alpha("");
    run.sandbox.write("source/Show/x.mkv", "new");
    run.sandbox.write("target/Show/Alpha.mkv", "old");

    let processed = run.process("Show", &["Show/x.mkv"]);

    assert!(matches!(
        run.what(&processed, "Show/x.mkv"),
        What::Refused(reason) if reason.starts_with("move:") && reason.contains("`Show/Alpha.mkv`")
    ));
}
