mod common;

use auto_renamer::Judged;
use common::{assert_refused_by, number, planned_batch, planned_record, record, with};

const RANK: &str = r#"[{ rank = { into = "index", by = ["episode"], prefer = ["cht", "chs"] } }]"#;

fn subtitle(path: &str, episode: u64) -> auto_renamer::Record {
    with(record(path), "episode", number(episode))
}

fn index_of(judged: &[Judged], origin: &str) -> Option<u64> {
    match planned_record(judged, origin).field("index") {
        Some(auto_renamer::Value::Number(index)) => Some(*index),
        _ => None,
    }
}

// @behavior RNK-001
#[test]
fn should_number_the_files_of_a_group_from_one_in_name_order() {
    let judged = planned_batch(
        &[("s", r#"[{ rank = { into = "index", by = ["episode"] } }]"#)],
        vec![subtitle("b.ass", 1), subtitle("a.ass", 1)],
    );

    assert_eq!(index_of(&judged, "a.ass"), Some(1));
    assert_eq!(index_of(&judged, "b.ass"), Some(2));
}

// @behavior RNK-002
#[test]
fn should_put_a_name_holding_a_preferred_word_first() {
    let judged = planned_batch(
        &[("s", RANK)],
        vec![subtitle("a.ass", 1), subtitle("b.cht.ass", 1)],
    );

    assert_eq!(index_of(&judged, "b.cht.ass"), Some(1));
    assert_eq!(index_of(&judged, "a.ass"), Some(2));
}

// @behavior RNK-003
#[test]
fn should_put_an_earlier_preferred_word_before_a_later_one() {
    let records = vec![
        subtitle("a.chs.ass", 1),
        subtitle("b.cht.ass", 1),
        subtitle("c.ass", 1),
    ];

    let judged = planned_batch(&[("s", RANK)], records);

    assert_eq!(index_of(&judged, "b.cht.ass"), Some(1));
    assert_eq!(index_of(&judged, "a.chs.ass"), Some(2));
    assert_eq!(index_of(&judged, "c.ass"), Some(3));
}

// @behavior RNK-004
#[test]
fn should_not_number_a_file_alone_in_its_group() {
    let judged = planned_batch(&[("s", RANK)], vec![subtitle("a.ass", 1)]);

    assert_eq!(index_of(&judged, "a.ass"), None);
}

// @behavior RNK-005
#[test]
fn should_number_files_whose_fields_differ_apart() {
    let judged = planned_batch(
        &[("s", RANK)],
        vec![subtitle("a.ass", 1), subtitle("b.ass", 2)],
    );

    assert_eq!(index_of(&judged, "a.ass"), None);
    assert_eq!(index_of(&judged, "b.ass"), None);
}

// @behavior RNK-006
#[test]
fn should_refuse_a_file_that_lacks_a_field_the_group_needs() {
    let judged = planned_batch(&[("s", RANK)], vec![record("a.ass")]);

    assert_refused_by(&judged, "a.ass", "rank");
}

// @behavior RNK-007
#[test]
fn should_ignore_case_in_the_preference() {
    let judged = planned_batch(
        &[("s", RANK)],
        vec![subtitle("a.ass", 1), subtitle("b.CHT.ass", 1)],
    );

    assert_eq!(index_of(&judged, "b.CHT.ass"), Some(1));
}

// @behavior RNK-008
#[test]
fn should_leave_a_record_ranked_on_its_own_without_a_number() {
    let ranked = common::run(
        r#"{ rank = { into = "index", by = ["episode"] } }"#,
        subtitle("a.ass", 1),
    );

    assert_eq!(ranked.field("index"), None);
}
