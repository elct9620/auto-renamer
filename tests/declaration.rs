use auto_renamer::stages::OnConflict;
use auto_renamer::{DeclareError, Declared, Pipeline, PipelineError};

fn read(stages: &str) -> Result<Pipeline, PipelineError> {
    Pipeline::from_toml(&format!("stages = {stages}"))
}

fn declared(stages: &str) -> Pipeline {
    read(stages).expect("the pipeline should be readable")
}

fn refused(stages: &str) -> PipelineError {
    read(stages).expect_err("the pipeline should be refused")
}

fn refused_declaration(stages: &str) -> DeclareError {
    match refused(stages) {
        PipelineError::Declare { error, .. } => error,
        other => panic!("expected a refused declaration, got {other:?}"),
    }
}

fn names(pipeline: &Pipeline) -> Vec<&'static str> {
    pipeline.stages().iter().map(Declared::name).collect()
}

fn assert_invalid(error: DeclareError, stage: &str, parameter: Option<&str>) {
    match error {
        DeclareError::Invalid {
            stage: found,
            parameter: found_parameter,
            ..
        } => {
            assert_eq!(found, stage);
            assert_eq!(found_parameter.as_deref(), parameter);
        }
        other => panic!("expected an invalid parameter, got {other:?}"),
    }
}

// @behavior DEC-001
#[test]
fn should_declare_a_stage_by_its_bare_name() {
    assert_eq!(names(&declared(r#"["move"]"#)), ["move"]);
}

// @behavior DEC-002
#[test]
fn should_declare_a_stage_with_parameters_by_a_one_key_table() {
    assert_eq!(names(&declared(r#"[{ format = "{name}" }]"#)), ["format"]);
}

// @behavior DEC-003
#[test]
fn should_refuse_a_table_with_two_keys() {
    assert_eq!(
        refused_declaration(r#"[{ format = "{name}", move = true }]"#),
        DeclareError::NotOneKey
    );
}

// @behavior DEC-004
#[test]
fn should_refuse_an_unknown_stage_by_name() {
    assert_eq!(
        refused_declaration(r#"["shred"]"#),
        DeclareError::UnknownStage("shred".to_string())
    );
}

// @behavior DEC-005
#[test]
fn should_refuse_an_unknown_parameter_by_name() {
    let error = refused_declaration(r#"[{ filter = { ext = ["mkv"], colour = "red" } }]"#);

    assert_invalid(error, "filter", Some("colour"));
}

// @behavior DEC-006
#[test]
fn should_refuse_a_bare_stage_that_needs_parameters() {
    assert_eq!(
        refused_declaration(r#"["number"]"#),
        DeclareError::NeedsParameters("number".to_string())
    );
}

// @behavior DEC-007
#[test]
fn should_refuse_a_filter_with_neither_extension_nor_pattern() {
    let error = refused_declaration("[{ filter = { invert = true } }]");

    assert_invalid(error, "filter", Some("ext"));
}

// @behavior DEC-008
#[test]
fn should_hold_extensions_in_lower_case() {
    let pipeline = declared(r#"[{ filter = { ext = ["MKV", "Mp4"] } }]"#);

    let Declared::Filter(filter) = &pipeline.stages()[0] else {
        panic!("expected a filter");
    };
    assert_eq!(filter.ext, ["mkv", "mp4"]);
}

// @behavior DEC-009
#[test]
fn should_refuse_number_extraction_without_a_field_to_write_into() {
    let error = refused_declaration(r#"[{ number = { prefix = "Season" } }]"#);

    assert_invalid(error, "number", Some("into"));
}

// @behavior DEC-010
#[test]
fn should_refuse_the_zeroth_candidate() {
    let error = refused_declaration(r#"[{ number = { into = "episode", nth = 0 } }]"#);

    assert_invalid(error, "number", Some("nth"));
}

// @behavior DEC-011
#[test]
fn should_refuse_a_pattern_that_is_not_a_regular_expression() {
    let error = refused_declaration(r#"[{ regex = { pattern = "(", into = "x" } }]"#);

    assert_invalid(error, "regex", Some("pattern"));
}

// @behavior DEC-012
#[test]
fn should_refuse_a_regular_expression_that_both_extracts_and_rewrites() {
    let error =
        refused_declaration(r#"[{ regex = { pattern = "a", into = "x", replace = "b" } }]"#);

    assert_invalid(error, "regex", Some("replace"));
}

// @behavior DEC-013
#[test]
fn should_refuse_a_pattern_that_compiles_to_something_enormous() {
    let error =
        refused_declaration(r#"[{ regex = { pattern = "((a{1000}){1000}){1000}", into = "x" } }]"#);

    assert_invalid(error, "regex", Some("pattern"));
}

// @behavior DEC-014
#[test]
fn should_refuse_a_fixed_value_that_is_neither_text_nor_a_whole_number() {
    let error = refused_declaration("[{ set = { season = true } }]");

    assert_invalid(error, "set", Some("season"));
}

// @behavior DEC-015
#[test]
fn should_refuse_a_case_change_other_than_lower_upper_or_title() {
    let error = refused_declaration(r#"[{ case = { to = "shout" } }]"#);

    assert_invalid(error, "case", Some("to"));
}

// @behavior DEC-016
#[test]
fn should_refuse_a_bracket_group_that_is_not_two_characters() {
    let error = refused_declaration(r#"[{ strip = { groups = ["[[]"] } }]"#);

    assert_invalid(error, "strip", Some("groups"));
}

// @behavior DEC-017
#[test]
fn should_refuse_a_malformed_template_when_the_pipeline_is_read() {
    let error = refused_declaration(r#"[{ format = "{name" }]"#);

    assert_invalid(error, "format", None);
}

// @behavior DEC-018
#[test]
fn should_declare_a_lift_by_a_number_of_levels() {
    assert_eq!(names(&declared("[{ lift = 1 }]")), ["lift"]);
}

// @behavior DEC-019
#[test]
fn should_refuse_a_lift_target_that_is_not_a_name_pattern() {
    let error = refused_declaration(r#"[{ lift = { to = "[" } }]"#);

    assert_invalid(error, "lift", Some("to"));
}

// @behavior DEC-020
#[test]
fn should_refuse_ranking_without_the_fields_that_make_a_group() {
    let error = refused_declaration(r#"[{ rank = { into = "index", by = [] } }]"#);

    assert_invalid(error, "rank", Some("by"));
}

// @behavior DEC-021
#[test]
fn should_refuse_a_conflict_policy_other_than_reject_or_suffix() {
    let error = refused_declaration(r#"[{ move = { on_conflict = "overwrite" } }]"#);

    assert_invalid(error, "move", Some("on_conflict"));
}

// @behavior DEC-022
#[test]
fn should_refuse_a_plan_stage_after_an_effect() {
    let refused = refused(r#"["move", { format = "{name}" }]"#);

    assert_eq!(
        refused,
        PipelineError::PureAfterEffect {
            stage: "format".to_string()
        }
    );
}

// @behavior DEC-023
#[test]
fn should_refuse_a_path_stage_after_next() {
    let refused =
        refused(r#"[{ next = { into = "episode", like = "{name} {episode}" } }, { lift = 1 }]"#);

    assert_eq!(
        refused,
        PipelineError::PathAfterNext {
            stage: "lift".to_string()
        }
    );
}

// @behavior DEC-024
#[test]
fn should_report_a_pipeline_without_an_effect_stage() {
    let pipeline = declared(r#"[{ format = "{name}" }]"#);

    assert!(!pipeline.has_effect());
}

// @behavior DEC-025
#[test]
fn should_keep_the_order_the_stages_were_written_in() {
    let pipeline = declared(r#"[{ filter = { ext = ["mkv"] } }, { format = "{name}" }, "move"]"#);

    assert_eq!(names(&pipeline), ["filter", "format", "move"]);
}

// @behavior DEC-026
#[test]
fn should_refuse_stages_that_are_not_a_list() {
    assert_eq!(
        Pipeline::from_toml(r#"stages = "move""#).unwrap_err(),
        PipelineError::NotAList
    );
}

// @behavior DEC-027
#[test]
fn should_refuse_a_conflict_suffix_that_reaches_outside_the_file_name() {
    let error = refused_declaration(r#"[{ move = { on_conflict = "suffix", suffix = "/../x" } }]"#);

    assert_invalid(error, "move", Some("suffix"));
}

// @behavior DEC-028
#[test]
fn should_default_the_conflict_policy_to_reject() {
    let pipeline = declared(r#"["move"]"#);

    let Declared::Move(policy) = &pipeline.stages()[0] else {
        panic!("expected a move");
    };
    assert_eq!(policy.on_conflict, OnConflict::Reject);
}

// @behavior DEC-029
#[test]
fn should_accept_the_pipelines_the_design_gives_as_examples() {
    let video = r#"[
        { filter = { ext = ["mkv", "mp4"] } },
        { number = { from = "path", into = "season", prefix = "Season" } },
        { number = { into = "episode", exclude = ["season"] } },
        { default = { season = 1 } },
        { lift = { to = "Season *" } },
        { next = { into = "episode", like = "{show} s{season:02}e{episode:02}" } },
        { format = "{show} s{season:02}e{episode:02}" },
        "move",
        { cleanup = { keep = ["Season *"] } },
    ]"#;
    let subtitle = r#"[
        { filter = { ext = ["ass", "srt"] } },
        { number = { from = "path", into = "season", prefix = "Season" } },
        { number = { into = "episode", exclude = ["season"] } },
        { default = { season = 1 } },
        { rank = { into = "index", by = ["season", "episode"], prefer = ["cht"] } },
        { lift = { to = "Season *" } },
        { format = "{show} s{season:02}e{episode:02}.zh[.{index:02}]" },
        "move",
    ]"#;
    let movie = r#"[{ filter = { ext = ["mkv", "mp4"] } }, { strip = {} }, "move"]"#;
    let music = r#"[
        { filter = { ext = ["mp3", "flac"] } },
        { regex = { pattern = '^(?<track>\d+)\s*-\s*(?<title>.+)$' } },
        { format = "{track:02} {title}" },
        "move",
    ]"#;
    let photo = r#"[
        { filter = { ext = ["jpg", "png"] } },
        { folder = "{mtime:%Y}/{mtime:%m}" },
        "move",
    ]"#;

    for source in [video, subtitle, movie, music, photo] {
        assert!(read(source).is_ok(), "should accept {source}");
    }
}

// @behavior DEC-030
#[test]
fn should_refuse_a_literal_replace_that_looks_for_nothing() {
    let error = refused_declaration(r#"[{ replace = { find = "", with = "x" } }]"#);

    assert_invalid(error, "replace", Some("find"));
}

// @behavior DEC-031
#[test]
fn should_refuse_a_next_template_that_does_not_mention_the_field_it_fills() {
    let error = refused_declaration(r#"[{ next = { into = "episode", like = "{name}" } }]"#);

    assert_invalid(error, "next", Some("like"));
}

// @behavior DEC-032
#[test]
fn should_refuse_a_pipeline_of_too_many_stages() {
    let stages = vec![r#""move""#; 65].join(", ");

    assert_eq!(
        refused(&format!("[{stages}]")),
        PipelineError::TooManyStages
    );
}
