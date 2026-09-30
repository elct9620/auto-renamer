mod common;

use auto_renamer::Value;
use common::{apply, assert_rejected_by, record, run, with};

fn name_after(declaration: &str, path: &str) -> Option<String> {
    match run(declaration, record(path)).field("name") {
        Some(Value::Text(name)) => Some(name.clone()),
        _ => None,
    }
}

// @behavior SET-001
#[test]
fn should_replace_a_field_when_set() {
    let record = run(
        "{ set = { season = 1 } }",
        with(record("x.mkv"), "season", Value::Number(5)),
    );

    assert_eq!(record.field("season"), Some(&Value::Number(1)));
}

// @behavior SET-002
#[test]
fn should_fill_a_field_that_is_unset_by_default() {
    let record = run("{ default = { season = 1 } }", record("x.mkv"));

    assert_eq!(record.field("season"), Some(&Value::Number(1)));
}

// @behavior SET-003
#[test]
fn should_keep_a_field_that_is_set_by_default() {
    let record = run(
        "{ default = { season = 1 } }",
        with(record("x.mkv"), "season", Value::Number(5)),
    );

    assert_eq!(record.field("season"), Some(&Value::Number(5)));
}

// @behavior REP-002
#[test]
fn should_swap_every_occurrence() {
    let stage = r#"{ replace = { find = "_", with = " " } }"#;

    assert_eq!(name_after(stage, "a_b_c.mkv"), Some("a b c".to_string()));
}

// @behavior REP-003
#[test]
fn should_refuse_a_replace_on_a_field_that_does_not_exist() {
    let outcome = apply(
        r#"{ replace = { find = "a", with = "b", field = "missing" } }"#,
        record("a.mkv"),
    );

    assert_rejected_by(outcome, "replace");
}

// @behavior REP-004
#[test]
fn should_refuse_a_replace_on_a_number() {
    let outcome = apply(
        r#"{ replace = { find = "a", with = "b", field = "episode" } }"#,
        with(record("a.mkv"), "episode", Value::Number(3)),
    );

    assert_rejected_by(outcome, "replace");
}

// @behavior CAS-001
#[test]
fn should_change_to_lower_case() {
    assert_eq!(
        name_after(r#"{ case = { to = "lower" } }"#, "ALPHA Beta.mkv"),
        Some("alpha beta".to_string())
    );
}

// @behavior CAS-002
#[test]
fn should_change_to_upper_case() {
    assert_eq!(
        name_after(r#"{ case = { to = "upper" } }"#, "alpha Beta.mkv"),
        Some("ALPHA BETA".to_string())
    );
}

// @behavior CAS-003
#[test]
fn should_capitalize_each_word_in_title_case() {
    assert_eq!(
        name_after(r#"{ case = { to = "title" } }"#, "aLPHA bETA.mkv"),
        Some("Alpha Beta".to_string())
    );
}

// @behavior CAS-004
#[test]
fn should_refuse_a_case_change_on_a_field_that_does_not_exist() {
    let outcome = apply(
        r#"{ case = { to = "lower", field = "missing" } }"#,
        record("a.mkv"),
    );

    assert_rejected_by(outcome, "case");
}

// @behavior STR-001
#[test]
fn should_remove_square_brackets_and_what_they_hold() {
    assert_eq!(
        name_after(r#"{ strip = {} }"#, "[Team] Show [1080p].mkv"),
        Some("Show".to_string())
    );
}

// @behavior STR-002
#[test]
fn should_keep_parentheses_unless_asked_for() {
    assert_eq!(
        name_after(r#"{ strip = {} }"#, "Show (2026).mkv"),
        Some("Show (2026)".to_string())
    );
}

// @behavior STR-003
#[test]
fn should_remove_other_bracket_groups_when_asked_for() {
    let stage = r#"{ strip = { groups = ["()"] } }"#;

    assert_eq!(
        name_after(stage, "Show (2026).mkv"),
        Some("Show".to_string())
    );
}

// @behavior STR-004
#[test]
fn should_collapse_and_trim_leftover_spaces() {
    assert_eq!(
        name_after(r#"{ strip = {} }"#, "[A]  Show   [B]  Two.mkv"),
        Some("Show Two".to_string())
    );
}

// @behavior STR-005
#[test]
fn should_keep_a_group_that_is_never_closed() {
    assert_eq!(
        name_after(r#"{ strip = {} }"#, "Show [1080p.mkv"),
        Some("Show [1080p".to_string())
    );
}

// @behavior STR-006
#[test]
fn should_remove_nested_groups_together() {
    assert_eq!(
        name_after(r#"{ strip = {} }"#, "Show [a [b] c].mkv"),
        Some("Show".to_string())
    );
}
