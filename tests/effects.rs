mod common;

use std::path::{Path, PathBuf};

use auto_renamer::{
    Applied, Done, Effect, EffectError, EffectsRun, Record, Roots, Verdict, apply_effects,
};
use common::{Sandbox, planned_batch, planned_record, record};

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

/// Plans the file through a pipeline of the stages, then carries out the effects it was planned with.
fn carry_out(stages: &str, sandbox: &Sandbox, dry_run: bool) -> EffectsRun {
    let judged = planned_batch(&[("p", stages)], vec![planned()]);
    let planned = planned_record(&judged, "Rel/x.mkv");

    apply_effects(
        &judged[0].effects,
        planned,
        Path::new("Rel"),
        &roots(sandbox),
        dry_run,
    )
}

// @behavior EFF-001
#[test]
fn should_run_the_effects_in_the_order_they_are_written() {
    let sandbox = Sandbox::new();
    sandbox.write("source/Rel/x.mkv", "video");

    let run = carry_out(EFFECTS, &sandbox, false);

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

    let run = carry_out(EFFECTS, &sandbox, false);

    assert!(matches!(run.error, Some(EffectError::Conflict(_))));
    assert!(run.done.is_empty());
    assert!(sandbox.exists("source/Rel/x.mkv"));
}

// @behavior EFF-003
#[test]
fn should_preview_the_move_and_run_no_cleanup_in_a_dry_run() {
    let sandbox = Sandbox::new();
    sandbox.write("source/Rel/x.mkv", "video");

    let run = carry_out(EFFECTS, &sandbox, true);

    assert_eq!(run.error, None);
    assert!(matches!(
        run.done.as_slice(),
        [Done::Moved(Applied::Preview { .. })]
    ));
    assert!(sandbox.exists("source/Rel/x.mkv"));
    assert!(!sandbox.exists("target"));
}

// @behavior EFF-004
#[test]
fn should_do_nothing_for_a_pipeline_without_effects() {
    let sandbox = Sandbox::new();
    sandbox.write("source/Rel/x.mkv", "video");

    let run = carry_out(r#"[{ set = { show = "Alpha" } }]"#, &sandbox, false);

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

// @behavior EFF-006
#[test]
fn should_plan_a_file_with_the_effects_of_its_pipeline_in_the_order_they_are_written() {
    let judged = planned_batch(&[("p", r#"["cleanup", "move"]"#)], vec![record("x.mkv")]);

    assert!(
        matches!(
            judged[0].effects.as_slice(),
            [Effect::Cleanup(_), Effect::Move(_)]
        ),
        "{:?}",
        judged[0].effects
    );
}

// @behavior EFF-007
#[test]
fn should_note_no_effect_for_a_file_a_stage_stopped() {
    let judged = planned_batch(
        &[("p", r#"[{ format = "{show}" }, "move"]"#)],
        vec![record("x.mkv")],
    );

    assert!(matches!(judged[0].verdict, Verdict::Rejected(_)));
    assert!(judged[0].effects.is_empty());
}

// @behavior EFF-008
#[test]
fn should_note_only_the_effects_of_the_pipeline_that_claimed_the_file() {
    let judged = planned_batch(
        &[
            ("video", r#"[{ filter = { ext = ["mkv"] } }]"#),
            ("rest", r#"["move"]"#),
        ],
        vec![record("x.mkv")],
    );

    assert_eq!(judged[0].pipeline, Some(0));
    assert!(judged[0].effects.is_empty(), "{:?}", judged[0].effects);
}
