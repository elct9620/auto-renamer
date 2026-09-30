mod common;

use auto_renamer::{Context, Record, Value};
use common::{Files, apply, assert_rejected_by, passed, record, stage, with};

const LIKE: &str = r#"{ next = { into = "episode", like = "{show} s{season:02}e{episode:02}" } }"#;

fn known(path: &str) -> Record {
    with(
        with(record(path), "show", common::text("Show")),
        "season",
        Value::Number(1),
    )
}

/// The episode filled in for a file, given what the target folder holds.
fn episode_given(files: &Files, path: &str) -> Option<u64> {
    let outcome = stage(LIKE).apply(known(path), &mut Context::new(files));
    match passed(outcome).field("episode") {
        Some(Value::Number(number)) => Some(*number),
        _ => None,
    }
}

fn episode_in(folder: &str, names: &[&str]) -> Option<u64> {
    episode_given(&Files::of(&[(folder, names)]), &format!("{folder}/x.mkv"))
}

fn episodes_of_a_batch(files: &Files, paths: &[&str]) -> Vec<Option<u64>> {
    let stage = stage(LIKE);
    let mut context = Context::new(files);
    paths
        .iter()
        .map(
            |path| match passed(stage.apply(known(path), &mut context)).field("episode") {
                Some(Value::Number(number)) => Some(*number),
                _ => None,
            },
        )
        .collect()
}

// @behavior NXT-001
#[test]
fn should_leave_a_field_that_is_already_set() {
    let input = with(record("x.mkv"), "episode", Value::Number(7));

    let outcome = apply(
        r#"{ next = { into = "episode", like = "{name} {episode}" } }"#,
        input.clone(),
    );

    assert_eq!(outcome, Ok(input));
}

// @behavior NXT-002
#[test]
fn should_follow_the_highest_number_in_the_target() {
    let found = episode_in("Series/Show", &["Show s01e01.mkv", "Show s01e02.mkv"]);

    assert_eq!(found, Some(3));
}

// @behavior NXT-003
#[test]
fn should_start_at_one_when_the_target_is_empty() {
    assert_eq!(episode_in("Series/Show", &[]), Some(1));
}

// @behavior NXT-004
#[test]
fn should_not_count_names_that_are_not_written_that_way() {
    let found = episode_in(
        "Series/Show",
        &["Show s01e05.mkv", "Other s01e09.mkv", "notes.txt"],
    );

    assert_eq!(found, Some(6));
}

// @behavior NXT-005
#[test]
fn should_not_compare_the_extension() {
    let found = episode_in("Series/Show", &["Show s01e01.mkv", "Show s01e04.mp4"]);

    assert_eq!(found, Some(5));
}

// @behavior NXT-006
#[test]
fn should_take_a_known_field_at_its_value() {
    let found = episode_in("Series/Show", &["Show s01e02.mkv", "Show s02e09.mkv"]);

    assert_eq!(found, Some(3));
}

// @behavior NXT-007
#[test]
fn should_read_back_a_number_padded_wider_than_the_template() {
    assert_eq!(episode_in("Series/Show", &["Show s01e007.mkv"]), Some(8));
}

// @behavior NXT-008
#[test]
fn should_refuse_when_the_record_lacks_a_field_the_template_needs() {
    let input = with(record("Series/Show/x.mkv"), "season", Value::Number(1));
    let files = Files::none();

    let outcome = stage(LIKE).apply(input, &mut Context::new(&files));

    assert_rejected_by(outcome, "next");
}

// @behavior NXT-009
#[test]
fn should_read_only_the_folder_of_the_plan() {
    let files = Files::of(&[("Series/Show", &[]), ("Series/Other", &["Show s01e09.mkv"])]);

    assert_eq!(episode_given(&files, "Series/Show/x.mkv"), Some(1));
}

// @behavior NXT-010
#[test]
fn should_give_the_records_of_one_batch_consecutive_numbers() {
    let episodes = episodes_of_a_batch(&Files::none(), &["Series/Show/a.mkv", "Series/Show/b.mkv"]);

    assert_eq!(episodes, [Some(1), Some(2)]);
}

// @behavior NXT-011
#[test]
fn should_continue_a_batch_after_the_target() {
    let files = Files::of(&[("Series/Show", &["Show s01e01.mkv", "Show s01e02.mkv"])]);

    let episodes = episodes_of_a_batch(&files, &["Series/Show/a.mkv", "Series/Show/b.mkv"]);

    assert_eq!(episodes, [Some(3), Some(4)]);
}

// @behavior NXT-012
#[test]
fn should_count_folders_apart() {
    let episodes = episodes_of_a_batch(&Files::none(), &["Series/A/x.mkv", "Series/B/y.mkv"]);

    assert_eq!(episodes, [Some(1), Some(1)]);
}

fn episode_with_optional_tag(tag: Option<&str>) -> Option<u64> {
    let stage = stage(r#"{ next = { into = "episode", like = "{show}[ {tag}] {episode}" } }"#);
    let mut input = with(record("Series/Show/x.mkv"), "show", common::text("Show"));
    if let Some(tag) = tag {
        input = with(input, "tag", common::text(tag));
    }
    let files = Files::of(&[("Series/Show", &["Show 5.mkv", "Show a 9.mkv"])]);

    match passed(stage.apply(input, &mut Context::new(&files))).field("episode") {
        Some(Value::Number(number)) => Some(*number),
        _ => None,
    }
}

// @behavior NXT-013
#[test]
fn should_leave_an_optional_part_out_when_its_field_is_missing() {
    assert_eq!(episode_with_optional_tag(None), Some(6));
}

// @behavior NXT-014
#[test]
fn should_require_an_optional_part_when_its_field_is_present() {
    assert_eq!(episode_with_optional_tag(Some("a")), Some(10));
}
