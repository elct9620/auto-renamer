#![allow(dead_code)]

use std::path::Path;

use auto_renamer::{Outcome, Record, Stage, Value};
use chrono::{TimeZone, Utc};

/// The record of a file, modified at a fixed time.
pub fn record(path: &str) -> Record {
    Record::new(
        Path::new(path),
        Utc.with_ymd_and_hms(2026, 9, 27, 12, 0, 0).unwrap(),
    )
}

pub fn with(mut record: Record, field: &str, value: Value) -> Record {
    record.set_field(field, value);
    record
}

pub fn text(value: &str) -> Value {
    Value::Text(value.to_string())
}

/// A stage as it is written in a pipeline, such as `{ number = { into = "episode" } }`.
pub fn stage(declaration: &str) -> Stage {
    let document: toml::Table = format!("stage = {declaration}")
        .parse()
        .expect("the declaration should be TOML");
    Stage::declare(&document["stage"]).expect("the stage should be declared")
}

pub fn apply(declaration: &str, record: Record) -> Outcome {
    stage(declaration).apply(record)
}

/// The record a stage passed on.
pub fn passed(outcome: Outcome) -> Record {
    match outcome {
        Outcome::Continue(record) => record,
        other => panic!("expected the record to go on, got {other:?}"),
    }
}

/// The record a stage passed on after running on `record`.
pub fn run(declaration: &str, record: Record) -> Record {
    passed(apply(declaration, record))
}

pub fn assert_rejected_by(outcome: Outcome, stage: &str) {
    match outcome {
        Outcome::Rejected(rejection) => assert_eq!(rejection.stage, stage),
        other => panic!("expected a refusal by `{stage}`, got {other:?}"),
    }
}
