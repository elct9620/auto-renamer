mod common;

use common::{apply, assert_rejected_by, record, text, with};

// @behavior FLT-001
#[test]
fn should_let_a_listed_extension_pass() {
    let outcome = apply(r#"{ filter = { ext = ["mkv"] } }"#, record("Alpha.mkv"));

    assert_eq!(outcome, Ok(record("Alpha.mkv")));
}

// @behavior FLT-002
#[test]
fn should_refuse_an_unlisted_extension() {
    let outcome = apply(r#"{ filter = { ext = ["mkv"] } }"#, record("Alpha.nfo"));

    assert_rejected_by(outcome, "filter");
}

// @behavior FLT-003
#[test]
fn should_compare_extensions_without_regard_to_case() {
    let outcome = apply(r#"{ filter = { ext = ["mkv"] } }"#, record("Alpha.MKV"));

    assert_eq!(outcome, Ok(record("Alpha.MKV")));
}

// @behavior FLT-004
#[test]
fn should_match_a_name_pattern_against_the_whole_file_name() {
    let outcome = apply(
        r#"{ filter = { glob = "Alpha*.mkv" } }"#,
        record("Alpha 12.mkv"),
    );

    assert_eq!(outcome, Ok(record("Alpha 12.mkv")));
}

// @behavior FLT-005
#[test]
fn should_turn_a_match_into_a_refusal_when_inverted() {
    let outcome = apply(
        r#"{ filter = { ext = ["nfo"], invert = true } }"#,
        record("Alpha.nfo"),
    );

    assert_rejected_by(outcome, "filter");
}

// @behavior FLT-006
#[test]
fn should_let_everything_else_through_when_inverted() {
    let outcome = apply(
        r#"{ filter = { ext = ["nfo"], invert = true } }"#,
        record("Alpha.mkv"),
    );

    assert_eq!(outcome, Ok(record("Alpha.mkv")));
}

// @behavior FLT-007
#[test]
fn should_need_both_the_extension_and_the_pattern_to_match() {
    let outcome = apply(
        r#"{ filter = { ext = ["mkv"], glob = "Beta*" } }"#,
        record("Alpha.mkv"),
    );

    assert_rejected_by(outcome, "filter");
}

// @behavior FLT-008
#[test]
fn should_take_only_the_files_refused_for_the_reasons_named() {
    let declaration = r#"{ filter = { reason = ["move"] } }"#;
    let by_move = with(record("Alpha.mkv"), "reason", text("move"));
    let by_format = with(record("Alpha.mkv"), "reason", text("format"));

    assert_eq!(apply(declaration, by_move.clone()), Ok(by_move));
    assert_rejected_by(apply(declaration, by_format), "filter");
}
