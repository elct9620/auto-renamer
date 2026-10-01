mod common;

use std::collections::BTreeMap;

use auto_renamer::Value;
use common::{number, planned_batch, planned_record, record, text};

/// The value a field holds once one stage ran on a file whose `vars` gave `answers`.
fn after(stages: &str, path: &str, answers: &[(&str, Value)]) -> Option<Value> {
    let vars = answers
        .iter()
        .map(|(name, value)| (name.to_string(), value.clone()))
        .collect::<BTreeMap<_, _>>();
    let field = answers[0].0;

    let judged = planned_batch(&[("p", stages)], vec![record(path).with_vars(vars)]);

    planned_record(&judged, path).field(field).cloned()
}

// @behavior ANS-001
#[test]
fn should_leave_a_field_an_answer_gave_to_a_number() {
    let episode = after(
        r#"[{ number = { into = "episode" } }]"#,
        "Show 07.mkv",
        &[("episode", number(3))],
    );

    assert_eq!(episode, Some(number(3)));
}

// @behavior ANS-002
#[test]
fn should_leave_a_field_an_answer_gave_to_a_regex_capture() {
    let show = after(
        r#"[{ regex = { from = "path", pattern = '^Series/(?<show>[^/]+)' } }]"#,
        "Series/Alpha/Season 02/x.mkv",
        &[("show", text("Alpha Next"))],
    );

    assert_eq!(show, Some(text("Alpha Next")));
}

// @behavior ANS-003
#[test]
fn should_leave_a_field_an_answer_gave_to_a_fixed_value() {
    let show = after(
        r#"[{ set = { show = "Beta" } }]"#,
        "x.mkv",
        &[("show", text("Alpha"))],
    );

    assert_eq!(show, Some(text("Alpha")));
}

// @behavior ANS-004
#[test]
fn should_rework_an_answer() {
    let show = after(
        r#"[{ case = { to = "upper", field = "show" } }]"#,
        "x.mkv",
        &[("show", text("alpha"))],
    );

    assert_eq!(show, Some(text("ALPHA")));
}
