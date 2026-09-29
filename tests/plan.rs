mod common;

use std::path::Path;

use auto_renamer::{Context, Outcome, Pipeline};
use common::{Files, record};

fn plan(pipeline: &Pipeline, input: auto_renamer::Record) -> Outcome {
    let files = Files::none();
    pipeline.plan(input, &mut Context::new(&files))
}

fn pipeline(stages: &str) -> Pipeline {
    Pipeline::from_toml(&format!("stages = {stages}")).expect("the pipeline should be readable")
}

// @behavior PLN-001
#[test]
fn should_run_the_stages_in_order_each_on_the_result_of_the_last() {
    let pipeline = pipeline(r#"[{ number = { into = "episode" } }, { format = "e{episode:02}" }]"#);

    let Outcome::Continue(planned) = plan(&pipeline, record("Show - 12.mkv")) else {
        panic!("expected the record to go on");
    };

    assert_eq!(planned.plan(), Path::new("e12.mkv"));
}

// @behavior PLN-002
#[test]
fn should_stop_at_an_excluded_record() {
    let pipeline = pipeline(r#"[{ filter = { ext = ["mkv"] } }, { format = "x" }]"#);

    assert_eq!(plan(&pipeline, record("x.nfo")), Outcome::Excluded);
}

// @behavior PLN-003
#[test]
fn should_stop_at_a_refusal_and_name_the_stage() {
    let pipeline = pipeline(r#"[{ format = "x" }, { format = "{missing}" }]"#);

    let Outcome::Rejected(rejection) = plan(&pipeline, record("x.mkv")) else {
        panic!("expected a refusal");
    };

    assert_eq!(rejection.stage, "format");
}

// @behavior PLN-004
#[test]
fn should_stop_before_the_first_effect_stage() {
    let pipeline = pipeline(r#"[{ format = "renamed" }, "move"]"#);

    let Outcome::Continue(planned) = plan(&pipeline, record("x.mkv")) else {
        panic!("expected the record to go on");
    };

    assert_eq!(planned.plan(), Path::new("renamed.mkv"));
}
