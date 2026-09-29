mod common;

use std::path::Path;

use auto_renamer::Value;
use common::{apply, assert_rejected_by, record, run, text, with};

// @behavior FMT-001
#[test]
fn should_rewrite_the_file_name_and_keep_the_folder() {
    let record = with(
        with(record("Series/Alpha/x.mkv"), "show", text("Alpha")),
        "episode",
        Value::Number(7),
    );

    let record = run(r#"{ format = "{show} s01e{episode:02}" }"#, record);

    assert_eq!(record.plan(), Path::new("Series/Alpha/Alpha s01e07.mkv"));
}

// @behavior FMT-002
#[test]
fn should_add_no_dot_to_a_file_without_an_extension() {
    let record = run(
        r#"{ format = "{show}" }"#,
        with(record("x"), "show", text("Alpha")),
    );

    assert_eq!(record.plan(), Path::new("Alpha"));
}

// @behavior FMT-003
#[test]
fn should_refuse_a_template_that_cannot_be_rendered() {
    assert_rejected_by(
        apply(r#"{ format = "{missing}" }"#, record("x.mkv")),
        "format",
    );
}

// @behavior FMT-004
#[test]
fn should_refuse_a_name_holding_a_slash() {
    let outcome = apply(
        r#"{ format = "{show}" }"#,
        with(record("x.mkv"), "show", text("a/b")),
    );

    assert_rejected_by(outcome, "format");
}

// @behavior FMT-005
#[test]
fn should_refuse_an_empty_name() {
    let outcome = apply(
        r#"{ format = "{show}" }"#,
        with(record("x.mkv"), "show", text("")),
    );

    assert_rejected_by(outcome, "format");
}

// @behavior FMT-006
#[test]
fn should_refuse_a_name_of_dots_only() {
    let outcome = apply(
        r#"{ format = "{show}" }"#,
        with(record("x.mkv"), "show", text("..")),
    );

    assert_rejected_by(outcome, "format");
}

// @behavior FMT-007
#[test]
fn should_make_the_name_field_follow_the_format() {
    let record = run(
        r#"{ format = "{show}" }"#,
        with(record("x.mkv"), "show", text("Alpha")),
    );

    assert_eq!(record.field("name"), Some(&text("Alpha")));
}
