use auto_renamer_wasm::{Configuration, Entry, Outcome, Simulation, check, read, render, simulate};

const MOVE_AS_SHOW: &str = r#"[{ format = "{show}" }, "move"]"#;

fn config(stages: &str, extra: &str) -> String {
    format!(
        "[pipeline.p]\nstages = {stages}\n\n[watch.w]\nsource = \"/src\"\ntarget = \"/dst\"\npipelines = [\"p\"]\nunit = \"directory\"\nvars = {{ show = \"Alpha\" }}\n{extra}\n"
    )
}

fn file(path: &str) -> Entry {
    file_with(path, "")
}

fn file_with(path: &str, text: &str) -> Entry {
    Entry {
        path: path.to_string(),
        folder: false,
        modified: 1_700_000_000_000,
        text: text.to_string(),
    }
}

fn run(config: &str, entries: Vec<Entry>) -> Simulation {
    simulate(config, "w", entries).expect("the simulation should run")
}

fn holds(simulation: &Simulation, path: &str) -> bool {
    simulation.entries.iter().any(|entry| entry.path == path)
}

fn outcome<'a>(simulation: &'a Simulation, origin: &str) -> Option<&'a Outcome> {
    simulation
        .outcomes
        .iter()
        .find(|outcome| outcome.origin == origin)
}

// @behavior PLG-001
#[test]
fn should_move_a_virtual_file_by_the_pipeline() {
    let simulation = run(&config(MOVE_AS_SHOW, ""), vec![file("/src/Show/x.mkv")]);

    assert!(holds(&simulation, "/dst/Show/Alpha.mkv"));
    assert!(!holds(&simulation, "/src/Show/x.mkv"));
}

// @behavior PLG-002
#[test]
fn should_apply_a_folder_configuration_in_the_virtual_tree() {
    let simulation = run(
        &config(MOVE_AS_SHOW, ""),
        vec![
            file("/src/Show/x.mkv"),
            file_with("/src/Show/auto-renamer.toml", "[vars]\nshow = \"Beta\"\n"),
        ],
    );

    assert!(holds(&simulation, "/dst/Show/Beta.mkv"));
}

// @behavior PLG-003
#[test]
fn should_not_process_a_folder_configuration_as_a_file() {
    let simulation = run(
        &config(MOVE_AS_SHOW, ""),
        vec![
            file("/src/Show/x.mkv"),
            file_with("/src/Show/auto-renamer.toml", "[vars]\nshow = \"Beta\"\n"),
        ],
    );

    assert_eq!(outcome(&simulation, "Show/auto-renamer.toml"), None);
    assert!(holds(&simulation, "/src/Show/auto-renamer.toml"));
}

// @behavior PLG-004
#[test]
fn should_process_each_unit_as_its_own_batch() {
    let simulation = run(
        &config(MOVE_AS_SHOW, "batch_max = 1"),
        vec![file("/src/A/x.mkv"), file("/src/B/y.mkv")],
    );

    assert_eq!(
        outcome(&simulation, "A/x.mkv").map(|o| o.what),
        Some("moved")
    );
    assert_eq!(
        outcome(&simulation, "B/y.mkv").map(|o| o.what),
        Some("moved")
    );
}

// @behavior PLG-005
#[test]
fn should_settle_a_taken_name_by_the_rule_of_the_move() {
    let simulation = run(
        &config(
            r#"[{ format = "{show}" }, { move = { on_conflict = "suffix" } }]"#,
            "",
        ),
        vec![file("/src/Show/x.mkv"), file("/dst/Show/Alpha.mkv")],
    );

    assert!(holds(&simulation, "/dst/Show/Alpha_v2.mkv"));
}

// @behavior PLG-006
#[test]
fn should_clean_up_a_folder_the_move_left_empty() {
    let simulation = run(
        &config(r#"["move", "cleanup"]"#, ""),
        vec![file("/src/Show/Season/x.mkv")],
    );

    assert!(holds(&simulation, "/dst/Show/Season/x.mkv"));
    assert!(!holds(&simulation, "/src/Show/Season"));
}

// @behavior PLG-007
#[test]
fn should_move_nothing_in_a_dry_run_and_say_where_a_file_would_go() {
    let simulation = run(
        &config(MOVE_AS_SHOW, "dry_run = true"),
        vec![file("/src/Show/x.mkv")],
    );

    let reported = outcome(&simulation, "Show/x.mkv").expect("the file should be reported");
    assert_eq!(reported.what, "previewed");
    assert_eq!(reported.to.as_deref(), Some("/dst/Show/Alpha.mkv"));
    assert!(holds(&simulation, "/src/Show/x.mkv"));
}

// @behavior PLG-008
#[test]
fn should_refuse_a_configuration_that_is_not_valid_with_why() {
    let refused = check(Configuration::Global, &config(r#"["shred"]"#, ""));

    assert!(refused.expect_err("it should be refused").contains("shred"));
}

// @behavior PLG-009
#[test]
fn should_check_a_folder_configuration_as_one() {
    let checked = check(Configuration::Folder, "[vars]\nshow = \"Beta\"\n");

    assert_eq!(checked, Ok(Vec::new()));
}

// @behavior PLG-010
#[test]
fn should_parse_the_same_once_read_and_written_again() {
    let text = format!(
        "[default]\nbatch_window = \"1m\"\n\n{}",
        config(MOVE_AS_SHOW, "")
    );

    let written = render(&read(&text).expect("the text should be read"));

    assert_eq!(read(&written), read(&text));
    assert!(check(Configuration::Global, &written).is_ok());
}

// @behavior PLG-011
#[test]
fn should_drop_comments_when_written_again() {
    let text = format!("# keep the episodes apart\n{}", config(MOVE_AS_SHOW, ""));

    let written = render(&read(&text).expect("the text should be read"));

    assert!(!written.contains('#'), "{written}");
}

// @behavior PLG-012
#[test]
fn should_take_the_time_field_from_the_modification_time_of_a_virtual_file() {
    let mut entry = file("/src/Show/x.mkv");
    entry.modified = 1_704_153_600_000; // 2024-01-02T00:00:00Z

    let simulation = run(
        &config(r#"[{ format = "{mtime:%Y}" }, "move"]"#, ""),
        vec![entry],
    );

    assert!(holds(&simulation, "/dst/Show/2024.mkv"));
}
