mod common;

use auto_renamer::{Value, Verdict, plan_batch};
use common::{Files, number, pipelines, planned_batch, planned_record, record, verdict, with};

fn kind_of(judged: &[auto_renamer::Judged], origin: &str) -> Option<String> {
    match planned_record(judged, origin).field("kind") {
        Some(Value::Text(kind)) => Some(kind.clone()),
        _ => None,
    }
}

// @behavior BAT-001
#[test]
fn should_plan_files_in_the_order_of_their_names() {
    let records = vec![record("b.mkv"), record("c.mkv"), record("a.mkv")];

    let judged = planned_batch(&[("all", r#"[{ set = { seen = 1 } }]"#)], records);

    let order: Vec<_> = judged
        .iter()
        .map(|entry| entry.origin.to_str().unwrap())
        .collect();
    assert_eq!(order, ["a.mkv", "b.mkv", "c.mkv"]);
}

// @behavior BAT-002
#[test]
fn should_give_a_file_to_the_first_pipeline_whose_filter_accepts_it() {
    let list = [
        (
            "video",
            r#"[{ filter = { ext = ["mkv"] } }, { set = { kind = "video" } }]"#,
        ),
        (
            "subtitle",
            r#"[{ filter = { ext = ["ass"] } }, { set = { kind = "subtitle" } }]"#,
        ),
    ];

    let judged = planned_batch(&list, vec![record("a.mkv"), record("a.ass")]);

    assert_eq!(kind_of(&judged, "a.mkv"), Some("video".to_string()));
    assert_eq!(kind_of(&judged, "a.ass"), Some("subtitle".to_string()));
}

// @behavior BAT-003
#[test]
fn should_leave_a_file_unclaimed_when_every_pipeline_turns_it_away() {
    let list = [("video", r#"[{ filter = { ext = ["mkv"] } }]"#)];

    let judged = planned_batch(&list, vec![record("a.nfo")]);

    assert_eq!(verdict(&judged, "a.nfo"), &Verdict::Unclaimed);
}

// @behavior BAT-004
#[test]
fn should_let_a_pipeline_without_a_leading_filter_claim_what_is_left() {
    let list = [
        ("video", r#"[{ filter = { ext = ["mkv"] } }]"#),
        ("rest", r#"[{ set = { kind = "rest" } }]"#),
    ];

    let judged = planned_batch(&list, vec![record("a.mkv"), record("a.nfo")]);

    assert_eq!(kind_of(&judged, "a.nfo"), Some("rest".to_string()));
}

// @behavior BAT-005
#[test]
fn should_exclude_a_file_for_good_when_a_later_filter_turns_it_away() {
    let list = [
        (
            "first",
            r#"[{ set = { kind = "first" } }, { filter = { ext = ["mkv"] } }]"#,
        ),
        ("second", r#"[{ set = { kind = "second" } }]"#),
    ];

    let judged = planned_batch(&list, vec![record("a.nfo")]);

    assert_eq!(verdict(&judged, "a.nfo"), &Verdict::Excluded);
}

// @behavior BAT-006
#[test]
fn should_leave_the_other_files_planned_when_one_is_refused() {
    let list = [("all", r#"[{ format = "{episode}" }]"#)];
    let records = vec![with(record("a.mkv"), "episode", number(1)), record("b.mkv")];

    let judged = planned_batch(&list, records);

    assert_eq!(
        planned_record(&judged, "a.mkv").plan(),
        std::path::Path::new("1.mkv")
    );
    common::assert_refused_by(&judged, "b.mkv", "format");
}

// @behavior BAT-007
#[test]
fn should_run_a_stage_over_the_whole_batch_in_the_order_of_the_names() {
    let list = [(
        "all",
        r#"[{ next = { into = "episode", like = "e{episode}" } }]"#,
    )];

    let judged = planned_batch(&list, vec![record("b.mkv"), record("a.mkv")]);

    assert_eq!(
        planned_record(&judged, "a.mkv").field("episode"),
        Some(&number(1))
    );
    assert_eq!(
        planned_record(&judged, "b.mkv").field("episode"),
        Some(&number(2))
    );
}

// @behavior BAT-008
#[test]
fn should_stop_a_pipeline_before_its_first_effect_stage() {
    let list = [("all", r#"[{ format = "renamed" }, "move"]"#)];

    let judged = planned_batch(&list, vec![record("a.mkv")]);

    assert_eq!(
        planned_record(&judged, "a.mkv").plan(),
        std::path::Path::new("renamed.mkv")
    );
}

// @behavior BAT-009
#[test]
fn should_keep_the_numbers_handed_out_across_pipelines_of_one_batch() {
    let files = Files::none();
    let list = pipelines(&[
        (
            "a",
            r#"[{ filter = { ext = ["mkv"] } }, { next = { into = "episode", like = "e{episode}" } }]"#,
        ),
        (
            "b",
            r#"[{ filter = { ext = ["mp4"] } }, { next = { into = "episode", like = "e{episode}" } }]"#,
        ),
    ]);

    let judged = plan_batch(&list, vec![record("x.mkv"), record("y.mp4")], &files);

    assert_eq!(
        planned_record(&judged, "x.mkv").field("episode"),
        Some(&number(1))
    );
    assert_eq!(
        planned_record(&judged, "y.mp4").field("episode"),
        Some(&number(2))
    );
}

// @behavior BAT-011
#[test]
fn should_plan_one_file_as_a_batch_of_one_each_stage_on_the_result_of_the_last() {
    let list = [(
        "all",
        r#"[{ number = { into = "episode" } }, { format = "e{episode:02}" }]"#,
    )];

    let judged = planned_batch(&list, vec![record("Show - 12.mkv")]);

    assert_eq!(
        planned_record(&judged, "Show - 12.mkv").plan(),
        std::path::Path::new("e12.mkv")
    );
}
