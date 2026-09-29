mod common;

use auto_renamer::{Record, Value};
use common::{apply, assert_rejected_by, record, run, with};

/// The episode a number stage found, given extra parameters such as `, nth = 2`.
fn episode(parameters: &str, record: Record) -> Option<u64> {
    let stage = format!(r#"{{ number = {{ into = "episode"{parameters} }} }}"#);
    match run(&stage, record).field("episode") {
        Some(Value::Number(number)) => Some(*number),
        _ => None,
    }
}

fn episode_of(name: &str) -> Option<u64> {
    episode("", record(name))
}

// @behavior NUM-001
#[test]
fn should_read_the_episode_from_a_season_and_episode_marker() {
    assert_eq!(episode_of("Show S01E12.mkv"), Some(12));
}

// @behavior NUM-002
#[test]
fn should_read_the_episode_from_an_episode_marker() {
    assert_eq!(episode_of("Show EP07.mkv"), Some(7));
}

// @behavior NUM-003
#[test]
fn should_read_the_episode_after_a_dash() {
    assert_eq!(episode_of("[Team] Show - 12 [1080p].mkv"), Some(12));
}

// @behavior NUM-004
#[test]
fn should_read_the_episode_after_the_word_episode() {
    assert_eq!(episode_of("Show episode 5.mkv"), Some(5));
}

// @behavior NUM-005
#[test]
fn should_prefer_a_marker_over_other_numbers() {
    assert_eq!(episode_of("[Team 7] Show - 12 [1080p].mkv"), Some(12));
}

// @behavior NUM-006
#[test]
fn should_take_the_only_number_left() {
    assert_eq!(episode_of("Show [12][WEB].mkv"), Some(12));
}

// @behavior NUM-007
#[test]
fn should_not_count_a_resolution() {
    assert_eq!(episode_of("Show [12][1080p][2160p].mkv"), Some(12));
}

// @behavior NUM-008
#[test]
fn should_not_count_a_codec() {
    assert_eq!(episode_of("Show [12][x264][h.265].mkv"), Some(12));
}

// @behavior NUM-009
#[test]
fn should_not_count_a_bit_depth() {
    assert_eq!(episode_of("Show [12][HEVC-10bit].mkv"), Some(12));
}

// @behavior NUM-010
#[test]
fn should_not_count_a_version() {
    assert_eq!(episode_of("Show 18v2.mkv"), Some(18));
}

// @behavior NUM-011
#[test]
fn should_not_count_a_date() {
    assert_eq!(episode_of("Show [12][2026.09.26].mkv"), Some(12));
}

// @behavior NUM-012
#[test]
fn should_not_count_a_year() {
    assert_eq!(episode_of("Show [12](2026).mkv"), Some(12));
}

// @behavior NUM-013
#[test]
fn should_not_count_an_audio_specification() {
    assert_eq!(episode_of("Show [12] OPUS 2.0.mkv"), Some(12));
}

// @behavior NUM-014
#[test]
fn should_not_count_a_season_marker() {
    assert_eq!(episode_of("Show S03 [10].mkv"), Some(10));
}

// @behavior NUM-015
#[test]
fn should_not_count_a_hash() {
    assert_eq!(episode_of("Show [12][5E9D2F64].mkv"), Some(12));
}

// @behavior NUM-016
#[test]
fn should_not_count_a_superscript_number() {
    assert_eq!(episode_of("[Team7³] Show [10].mkv"), Some(10));
}

// @behavior NUM-017
#[test]
fn should_accept_four_digits_as_an_episode() {
    assert_eq!(episode_of("Show [1354][2026.09.26].mkv"), Some(1354));
}

// @behavior NUM-018
#[test]
fn should_leave_the_field_unset_when_several_candidates_remain() {
    assert_eq!(episode_of("[Team-7][Show 17][03].mkv"), None);
}

fn with_season_17(name: &str) -> Record {
    with(record(name), "season", Value::Number(17))
}

// @behavior NUM-019
#[test]
fn should_take_the_nth_candidate_when_asked_for() {
    let found = episode(
        r#", exclude = ["season"], nth = 2"#,
        with_season_17("[Team-7][Show 17][03].mkv"),
    );

    assert_eq!(found, Some(3));
}

// @behavior NUM-020
#[test]
fn should_count_a_negative_nth_from_the_end() {
    let found = episode(
        r#", exclude = ["season"], nth = -2"#,
        with_season_17("[Team-7][Show 17][03].mkv"),
    );

    assert_eq!(found, Some(7));
}

// @behavior NUM-021
#[test]
fn should_leave_the_field_unset_when_nth_is_beyond_the_candidates() {
    assert_eq!(episode(", nth = 2", record("Show [12].mkv")), None);
}

// @behavior NUM-022
#[test]
fn should_not_count_a_number_equal_to_an_excluded_field() {
    let found = episode(
        r#", exclude = ["season"]"#,
        with_season_17("Show 17 [03].mkv"),
    );

    assert_eq!(found, Some(3));
}

// @behavior NUM-023
#[test]
fn should_exclude_nothing_for_a_field_that_is_unset() {
    assert_eq!(
        episode(r#", exclude = ["season"]"#, record("Show 01.mkv")),
        Some(1)
    );
}

// @behavior NUM-024
#[test]
fn should_skip_markers_when_the_nth_candidate_is_asked_for() {
    assert_eq!(episode(", nth = 2", record("Show - 12 [7].mkv")), Some(7));
}

// @behavior NUM-025
#[test]
fn should_take_the_number_after_the_prefix() {
    let stage = r#"{ number = { into = "season", from = "path", prefix = "Season" } }"#;

    let record = run(stage, record("Series/Alpha/Season 03/x.mkv"));

    assert_eq!(record.field("season"), Some(&Value::Number(3)));
}

// @behavior NUM-026
#[test]
fn should_leave_the_field_unset_when_the_prefix_is_not_in_the_text() {
    let stage = r#"{ number = { into = "season", from = "path", prefix = "Season" } }"#;

    let record = run(stage, record("Series/Alpha/x.mkv"));

    assert_eq!(record.field("season"), None);
}

// @behavior NUM-027
#[test]
fn should_refuse_a_source_field_that_does_not_exist() {
    let outcome = apply(
        r#"{ number = { into = "episode", from = "missing" } }"#,
        record("Show 12.mkv"),
    );

    assert_rejected_by(outcome, "number");
}

// @behavior NUM-028
#[test]
fn should_leave_the_field_unset_for_a_name_without_numbers() {
    assert_eq!(episode_of("Show.mkv"), None);
}

// @behavior NUM-029
#[test]
fn should_read_another_field_when_asked_to() {
    let found = episode(r#", from = "dir""#, record("Series/Show 12/x.mkv"));

    assert_eq!(found, Some(12));
}

// @behavior NUM-030
#[test]
fn should_refuse_a_date_field_as_the_source() {
    let outcome = apply(
        r#"{ number = { into = "episode", from = "mtime" } }"#,
        record("Show 12.mkv"),
    );

    assert_rejected_by(outcome, "number");
}
