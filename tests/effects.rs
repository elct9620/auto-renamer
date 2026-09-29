mod common;

use std::path::PathBuf;

use auto_renamer::{Applied, Done, EffectError, Pipeline, Record, Roots, apply_effects};
use common::{Sandbox, record};

fn pipeline(stages: &str) -> Pipeline {
    Pipeline::from_toml(&format!("stages = {stages}")).expect("the pipeline should be readable")
}

fn planned() -> Record {
    let mut input = record("Rel/x.mkv");
    input.set_plan(PathBuf::from("y.mkv"));
    input
}

fn roots(sandbox: &Sandbox) -> Roots {
    Roots {
        source: sandbox.path("source"),
        target: sandbox.path("target"),
    }
}

const EFFECTS: &str = r#"["move", "cleanup"]"#;

// @behavior EFF-001
#[test]
fn should_run_the_effects_in_the_order_they_are_written() {
    let sandbox = Sandbox::new();
    sandbox.write("source/Rel/x.mkv", "video");

    let run = apply_effects(
        &pipeline(EFFECTS),
        &planned(),
        std::path::Path::new("Rel"),
        &roots(&sandbox),
        false,
    );

    assert_eq!(run.error, None);
    assert!(
        matches!(run.done.as_slice(), [Done::Moved(Applied::Moved { .. }), Done::Cleaned(folders)] if folders == &[PathBuf::from("Rel")])
    );
    assert!(sandbox.exists("target/y.mkv"));
    assert!(!sandbox.exists("source/Rel"));
}

// @behavior EFF-002
#[test]
fn should_stop_the_effects_after_a_failed_move() {
    let sandbox = Sandbox::new();
    sandbox.write("source/Rel/x.mkv", "new");
    sandbox.write("target/y.mkv", "old");

    let run = apply_effects(
        &pipeline(EFFECTS),
        &planned(),
        std::path::Path::new("Rel"),
        &roots(&sandbox),
        false,
    );

    assert!(matches!(run.error, Some(EffectError::Conflict(_))));
    assert!(run.done.is_empty());
    assert!(sandbox.exists("source/Rel/x.mkv"));
}

// @behavior EFF-003
#[test]
fn should_preview_every_effect_and_change_nothing_in_a_dry_run() {
    let sandbox = Sandbox::new();
    sandbox.write("source/Rel/x.mkv", "video");

    let run = apply_effects(
        &pipeline(EFFECTS),
        &planned(),
        std::path::Path::new("Rel"),
        &roots(&sandbox),
        true,
    );

    assert_eq!(run.error, None);
    assert!(matches!(
        run.done.first(),
        Some(Done::Moved(Applied::Preview { .. }))
    ));
    assert!(sandbox.exists("source/Rel/x.mkv"));
    assert!(!sandbox.exists("target"));
}

// @behavior EFF-004
#[test]
fn should_do_nothing_for_a_pipeline_without_effects() {
    let sandbox = Sandbox::new();
    sandbox.write("source/Rel/x.mkv", "video");

    let run = apply_effects(
        &pipeline(r#"[{ format = "z" }]"#),
        &planned(),
        std::path::Path::new("Rel"),
        &roots(&sandbox),
        false,
    );

    assert!(run.done.is_empty());
    assert_eq!(run.error, None);
    assert!(sandbox.exists("source/Rel/x.mkv"));
}

// @behavior EFF-005
#[test]
fn should_pass_the_record_on_unchanged_when_an_effect_is_applied_while_planning() {
    let outcome = common::apply(r#""move""#, record("x.mkv"));

    assert_eq!(common::passed(outcome), record("x.mkv"));
}
