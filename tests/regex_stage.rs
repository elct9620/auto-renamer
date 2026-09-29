mod common;

use auto_renamer::{Outcome, Value};
use common::{apply, assert_rejected_by, record, run, text};

// @behavior RGX-001
#[test]
fn should_write_the_first_group_into_the_field_asked_for() {
    let stage = r#"{ regex = { pattern = 'Show (\w+)', into = "word" } }"#;

    assert_eq!(
        run(stage, record("Show Alpha.mkv")).field("word"),
        Some(&text("Alpha"))
    );
}

// @behavior RGX-002
#[test]
fn should_write_the_whole_match_when_there_is_no_group() {
    let stage = r#"{ regex = { pattern = 'A[a-z]+', into = "word" } }"#;

    assert_eq!(
        run(stage, record("Show Alpha.mkv")).field("word"),
        Some(&text("Alpha"))
    );
}

// @behavior RGX-003
#[test]
fn should_turn_digits_alone_into_a_number() {
    let stage = r#"{ regex = { pattern = '([0-9]+)', into = "episode" } }"#;

    assert_eq!(
        run(stage, record("Show 007.mkv")).field("episode"),
        Some(&Value::Number(7))
    );
}

// @behavior RGX-004
#[test]
fn should_turn_named_groups_into_fields() {
    let stage = r#"{ regex = { pattern = '(?<track>[0-9]+) - (?<title>.+)' } }"#;

    let record = run(stage, record("03 - Song.mp3"));

    assert_eq!(record.field("track"), Some(&Value::Number(3)));
    assert_eq!(record.field("title"), Some(&text("Song")));
}

// @behavior RGX-005
#[test]
fn should_replace_every_match_when_rewriting() {
    let stage = r#"{ regex = { pattern = '[._]', replace = " " } }"#;

    assert_eq!(
        run(stage, record("a.b_c.mkv")).field("name"),
        Some(&text("a b c"))
    );
}

// @behavior RGX-006
#[test]
fn should_use_the_groups_when_rewriting() {
    let stage = r#"{ regex = { pattern = '(\w+)-(\w+)', replace = "$2-$1" } }"#;

    assert_eq!(
        run(stage, record("ab-cd.mkv")).field("name"),
        Some(&text("cd-ab"))
    );
}

// @behavior RGX-007
#[test]
fn should_leave_the_record_unchanged_when_nothing_matches() {
    let outcome = apply(
        r#"{ regex = { pattern = 'Z+', into = "word" } }"#,
        record("Show.mkv"),
    );

    assert_eq!(outcome, Outcome::Continue(record("Show.mkv")));
}

// @behavior RGX-008
#[test]
fn should_refuse_a_source_field_that_does_not_exist() {
    let outcome = apply(
        r#"{ regex = { pattern = 'a', into = "x", from = "missing" } }"#,
        record("Show.mkv"),
    );

    assert_rejected_by(outcome, "regex");
}

// @behavior RGX-009
#[test]
fn should_read_another_field_when_asked_to() {
    let stage = r#"{ regex = { pattern = 'Season ([0-9]+)', into = "season", from = "path" } }"#;

    assert_eq!(
        run(stage, record("Show/Season 4/x.mkv")).field("season"),
        Some(&Value::Number(4))
    );
}

// @behavior RGX-010
#[test]
fn should_refuse_a_date_field_as_the_source() {
    let outcome = apply(
        r#"{ regex = { pattern = 'a', into = "x", from = "mtime" } }"#,
        record("Show.mkv"),
    );

    assert_rejected_by(outcome, "regex");
}

// @behavior RGX-011
#[test]
fn should_refuse_a_rewrite_that_makes_the_name_unusable() {
    let outcome = apply(
        r#"{ regex = { pattern = 'a', replace = "/" } }"#,
        record("a.mkv"),
    );

    assert_rejected_by(outcome, "regex");
}
