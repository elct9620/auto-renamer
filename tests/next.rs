mod common;

use auto_renamer::{Outcome, Value};
use common::{apply, record, with};

// @behavior NXT-001
#[test]
fn should_leave_a_field_that_is_already_set() {
    let input = with(record("x.mkv"), "episode", Value::Number(7));

    let outcome = apply(
        r#"{ next = { into = "episode", like = "{name}" } }"#,
        input.clone(),
    );

    assert_eq!(outcome, Outcome::Continue(input));
}
