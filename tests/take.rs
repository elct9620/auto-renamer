mod common;

use auto_renamer::{Judged, Record, Value};
use common::{assert_refused_by, number, planned_batch, planned_record, record, with};

const VIDEO: &str = r#"[{ filter = { ext = ["mkv", "mp4"] } }]"#;
const SUBTITLE: &str = r#"[{ filter = { ext = ["ass"] } }, { take = { fields = ["episode"] } }]"#;

fn video(path: &str, episode: u64) -> Record {
    with(record(path), "episode", number(episode))
}

fn episode_of(judged: &[Judged], origin: &str) -> Option<u64> {
    match planned_record(judged, origin).field("episode") {
        Some(Value::Number(episode)) => Some(*episode),
        _ => None,
    }
}

// @behavior TAK-001
#[test]
fn should_take_fields_from_the_file_whose_name_begins_this_one() {
    let judged = planned_batch(
        &[("video", VIDEO), ("subtitle", SUBTITLE)],
        vec![video("Show 27.mkv", 27), record("Show 27.cht.ass")],
    );

    assert_eq!(episode_of(&judged, "Show 27.cht.ass"), Some(27));
}

// @behavior TAK-002
#[test]
fn should_prefer_the_longest_beginning() {
    let records = vec![
        video("Show.mkv", 1),
        video("Show 27.mkv", 27),
        record("Show 27.cht.ass"),
    ];

    let judged = planned_batch(&[("video", VIDEO), ("subtitle", SUBTITLE)], records);

    assert_eq!(episode_of(&judged, "Show 27.cht.ass"), Some(27));
}

// @behavior TAK-003
#[test]
fn should_refuse_a_file_with_nothing_to_take_from() {
    let judged = planned_batch(&[("subtitle", SUBTITLE)], vec![record("Show 27.cht.ass")]);

    assert_refused_by(&judged, "Show 27.cht.ass", "take");
}

// @behavior TAK-004
#[test]
fn should_refuse_two_equally_long_beginnings() {
    let records = vec![
        video("Show 27.mkv", 27),
        video("Show 27.mp4", 27),
        record("Show 27.cht.ass"),
    ];

    let judged = planned_batch(&[("video", VIDEO), ("subtitle", SUBTITLE)], records);

    assert_refused_by(&judged, "Show 27.cht.ass", "take");
}

// @behavior TAK-005
#[test]
fn should_refuse_a_file_that_took_from_a_refused_file() {
    let list = [
        (
            "video",
            r#"[{ filter = { ext = ["mkv"] } }, { format = "{missing}" }]"#,
        ),
        ("subtitle", SUBTITLE),
    ];

    let judged = planned_batch(
        &list,
        vec![video("Show 27.mkv", 27), record("Show 27.cht.ass")],
    );

    assert_refused_by(&judged, "Show 27.mkv", "format");
    assert_refused_by(&judged, "Show 27.cht.ass", "take");
}

// @behavior TAK-006
#[test]
fn should_take_only_from_the_pipeline_asked_for() {
    let list = [
        ("video", r#"[{ filter = { ext = ["mkv"] } }]"#),
        ("extra", r#"[{ filter = { ext = ["mp4"] } }]"#),
        (
            "subtitle",
            r#"[{ filter = { ext = ["ass"] } }, { take = { fields = ["episode"], from = "extra" } }]"#,
        ),
    ];
    let records = vec![
        video("Show 27.mkv", 1),
        video("Show 27.mp4", 2),
        record("Show 27.cht.ass"),
    ];

    let judged = planned_batch(&list, records);

    assert_eq!(episode_of(&judged, "Show 27.cht.ass"), Some(2));
}

// @behavior TAK-007
#[test]
fn should_refuse_when_the_other_file_lacks_the_field() {
    let judged = planned_batch(
        &[("video", VIDEO), ("subtitle", SUBTITLE)],
        vec![record("Show 27.mkv"), record("Show 27.cht.ass")],
    );

    assert_refused_by(&judged, "Show 27.cht.ass", "take");
}

// @behavior TAK-008
#[test]
fn should_not_take_from_a_pipeline_that_has_not_run_yet() {
    let judged = planned_batch(
        &[("subtitle", SUBTITLE), ("video", VIDEO)],
        vec![video("Show 27.mkv", 27), record("Show 27.cht.ass")],
    );

    assert_refused_by(&judged, "Show 27.cht.ass", "take");
}

// @behavior TAK-009
#[test]
fn should_refuse_a_record_taking_fields_on_its_own() {
    let outcome = common::apply(r#"{ take = { fields = ["episode"] } }"#, record("a.ass"));

    assert!(
        matches!(&outcome, Err(auto_renamer::Stop::Rejected(rejection))
            if rejection.stage == "take" && rejection.reason.contains("no earlier file")),
        "{outcome:?}"
    );
}
