use auto_renamer_wasm::{
    Configuration, Entry, Outcome, Simulation, check, read, render, simulate, stages,
};

const MOVE_AS_SHOW: &str = r#"[{ format = "{show}" }]"#;

fn config(stages: &str, extra: &str) -> String {
    config_route(stages, "", extra)
}

/// A configuration whose one route moves into `/dst`, with `route` written into the route.
fn config_route(stages: &str, route: &str, extra: &str) -> String {
    format!(
        "[pipeline.p]\nstages = {stages}\n\n[target.dst]\npath = \"/dst\"\n\n[watch.w]\nsource = \"/src\"\nroutes = [{{ pipeline = \"p\", move = \"dst\"{route} }}]\nunit = \"directory\"\nvars = {{ show = \"Alpha\" }}\n{extra}\n"
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
        &config(MOVE_AS_SHOW, "max_files = 1"),
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
fn should_refuse_a_file_whose_name_is_taken() {
    let simulation = run(
        &config(MOVE_AS_SHOW, ""),
        vec![file("/src/Show/x.mkv"), file("/dst/Show/Alpha.mkv")],
    );

    assert_eq!(
        outcome(&simulation, "Show/x.mkv").map(|o| o.what),
        Some("refused")
    );
    assert!(holds(&simulation, "/src/Show/x.mkv"));
}

// @behavior PLG-006
#[test]
fn should_clean_up_a_folder_the_move_left_empty() {
    let simulation = run(
        &config_route("[]", ", cleanup = {}", ""),
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

    assert_eq!(checked, Ok(()));
}

// @behavior PLG-010
#[test]
fn should_parse_the_same_once_read_and_written_again() {
    let text = format!("[default]\nquiet = \"1m\"\n\n{}", config(MOVE_AS_SHOW, ""));

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

    let simulation = run(&config(r#"[{ format = "{mtime:%Y}" }]"#, ""), vec![entry]);

    assert!(holds(&simulation, "/dst/Show/2024.mkv"));
}

// @behavior PLG-013
#[test]
fn should_accept_every_stage_offered_as_its_example() {
    for stage in stages() {
        let declared = match stage.example {
            "" => format!("[\"{}\"]", stage.name),
            example => format!("[{{ {} = {example} }}]", stage.name),
        };

        let checked = check(Configuration::Global, &config(&declared, ""));

        assert!(checked.is_ok(), "{}: {checked:?}", stage.name);
    }
}

// @behavior PLG-014
#[test]
fn should_carry_every_step_of_a_file() {
    let stages = r#"[{ number = { into = "episode" } }, { format = "{show} {episode}" }]"#;

    let simulation = run(&config(stages, ""), vec![file("/src/Show/x 07.mkv")]);

    let steps = &outcome(&simulation, "Show/x 07.mkv")
        .expect("the file is reported")
        .steps;
    let places: Vec<_> = steps.iter().map(|step| step.stage).collect();
    assert_eq!(places, [None, Some(0), Some(1)]);
    let names: Vec<_> = steps.iter().map(|step| step.name).collect();
    assert_eq!(names, [None, Some("number"), Some("format")]);
    assert_eq!(
        steps[1].fields.get("episode").map(String::as_str),
        Some("7")
    );
    assert_eq!(
        steps[2].fields.get("name").map(String::as_str),
        Some("Alpha 7")
    );
}

// @behavior PLG-015
#[test]
fn should_mark_a_file_a_replaced_pipeline_planned() {
    let entries = vec![
        file("/src/Show/x.mkv"),
        file_with(
            "/src/Show/auto-renamer.toml",
            "[pipeline.p]\nstages = [{ format = \"Beta\" }]\n",
        ),
        file("/src/Other/y.mkv"),
    ];

    let simulation = run(&config(MOVE_AS_SHOW, ""), entries);

    assert!(
        outcome(&simulation, "Show/x.mkv")
            .expect("the file is reported")
            .replaced
    );
    assert!(
        !outcome(&simulation, "Other/y.mkv")
            .expect("the file is reported")
            .replaced
    );
}
