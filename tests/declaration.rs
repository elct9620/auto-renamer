use auto_renamer::{Declaration, DeclareError, Declared, ParameterKind, Pipeline, PipelineError};
use toml::{Table, Value};

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
    assert_eq!(names(&declared(r#"["strip"]"#)), ["strip"]);
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

// @behavior DEC-007
#[test]
fn should_refuse_a_filter_with_neither_extension_nor_pattern() {
    let error = refused_declaration("[{ filter = { invert = true } }]");

    assert_invalid(error, "filter", Some("ext"));
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

// @behavior DEC-038
#[test]
fn should_refuse_a_lift_naming_both_a_pattern_and_folders_to_keep() {
    let error = refused_declaration(r#"[{ lift = { to = "Season *", keep = 1 } }]"#);

    assert_invalid(error, "lift", None);
}

// @behavior DEC-020
#[test]
fn should_refuse_ranking_without_the_fields_that_make_a_group() {
    let error = refused_declaration(r#"[{ rank = { into = "index", by = [] } }]"#);

    assert_invalid(error, "rank", Some("by"));
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

// @behavior DEC-025
#[test]
fn should_keep_the_order_the_stages_were_written_in() {
    let pipeline = declared(r#"[{ filter = { ext = ["mkv"] } }, { format = "{name}" }, "strip"]"#);

    assert_eq!(names(&pipeline), ["filter", "format", "strip"]);
}

// @behavior DEC-026
#[test]
fn should_refuse_stages_that_are_not_a_list() {
    assert_eq!(
        Pipeline::from_toml(r#"stages = "strip""#).unwrap_err(),
        PipelineError::NotAList
    );
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

/// The stage declared from its example, with the example's table changed by `change`.
fn declare(stage: &Declaration, change: impl FnOnce(&mut Table)) -> Result<Declared, DeclareError> {
    let example: Value = format!(
        "value = {}",
        if stage.example.is_empty() {
            "{}"
        } else {
            stage.example
        }
    )
    .parse::<Table>()
    .expect("an example is inline TOML")
    .remove("value")
    .expect("the example is read as a value");
    let parameters = match example {
        Value::Table(mut table) => {
            change(&mut table);
            Value::Table(table)
        }
        other => other,
    };
    let declaration = match (stage.example.is_empty(), &parameters) {
        (true, Value::Table(table)) if table.is_empty() => Value::String(stage.name.to_string()),
        _ => Value::Table(Table::from_iter([(stage.name.to_string(), parameters)])),
    };
    Declared::read(&declaration)
}

fn sample(kind: ParameterKind) -> Value {
    match kind {
        ParameterKind::Text => Value::String("x".to_string()),
        ParameterKind::Texts => Value::Array(vec![Value::String("x".to_string())]),
        ParameterKind::Integer => Value::Integer(1),
        ParameterKind::Boolean => Value::Boolean(true),
    }
}

// @behavior DEC-033
#[test]
fn should_accept_every_stage_declared_with_its_example() {
    for stage in Declared::declarations() {
        let declared = declare(stage, |_| {});

        assert!(declared.is_ok(), "{}: {declared:?}", stage.name);
    }
}

// @behavior DEC-034
#[test]
fn should_accept_every_described_parameter_as_a_parameter_of_its_stage() {
    for stage in Declared::declarations() {
        for parameter in stage.parameters {
            let declared = declare(stage, |table| {
                table.insert(parameter.name.to_string(), sample(parameter.kind));
            });

            if let Err(DeclareError::Invalid { reason, .. }) = &declared {
                assert_ne!(
                    reason, "is not a parameter of this stage",
                    "{}.{}",
                    stage.name, parameter.name
                );
            }
        }
    }
}

// @behavior DEC-035
#[test]
fn should_read_only_described_parameters() {
    let lift_to: Value = r#"lift = { to = "Season *" }"#.parse::<Table>().unwrap().into();

    for stage in Declared::declarations() {
        declare(stage, |_| {}).expect("the example is accepted");
    }
    Declared::read(&lift_to).expect("a lift to a pattern is accepted");
}

// @behavior DEC-036
#[test]
fn should_refuse_a_stage_without_a_required_parameter() {
    for stage in Declared::declarations() {
        for parameter in stage
            .parameters
            .iter()
            .filter(|parameter| parameter.required)
        {
            if stage.value.is_some() {
                continue;
            }
            let declared = declare(stage, |table| {
                table.remove(parameter.name);
            });

            assert!(declared.is_err(), "{}.{}", stage.name, parameter.name);
        }
    }
}

// @behavior DEC-037
#[test]
fn should_accept_each_choice_of_a_parameter() {
    for stage in Declared::declarations() {
        for parameter in stage.parameters {
            for choice in parameter.choices {
                let declared = declare(stage, |table| {
                    table.insert(
                        parameter.name.to_string(),
                        Value::String(choice.to_string()),
                    );
                });

                assert!(
                    declared.is_ok(),
                    "{}.{} = {choice}: {declared:?}",
                    stage.name,
                    parameter.name
                );
            }
        }
    }
}

// @behavior DEC-039
#[test]
fn should_refuse_move_as_a_stage() {
    let error = refused_declaration(r#"["move"]"#);

    assert_eq!(error, DeclareError::UnknownStage("move".to_string()));
}
