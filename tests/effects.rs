mod common;

use std::path::{Path, PathBuf};

use auto_renamer::{
    Applied, Cleanup, Config, Done, Effect, EffectError, EffectsRun, FsTree, Record, Renames,
    Roots, What, apply_effects, process_batch,
};
use common::{Sandbox, record};

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

/// Carries out the effects of a route that moves and then cleans up, on a file planned to `y.mkv`.
fn carry_out(sandbox: &Sandbox, dry_run: bool) -> EffectsRun {
    let effects = [Effect::Move, Effect::Cleanup(Cleanup::default())];

    apply_effects(
        &FsTree,
        &effects,
        &planned(),
        Path::new("Rel"),
        &roots(sandbox),
        dry_run,
    )
}

// @behavior EFF-001
#[test]
fn should_move_and_then_clean_up() {
    let sandbox = Sandbox::new();
    sandbox.write("source/Rel/x.mkv", "video");

    let run = carry_out(&sandbox, false);

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

    let run = carry_out(&sandbox, false);

    assert!(matches!(run.error, Some(EffectError::Conflict(_))));
    assert!(run.done.is_empty());
    assert!(sandbox.exists("source/Rel/x.mkv"));
}

// @behavior EFF-003
#[test]
fn should_preview_the_move_and_run_no_cleanup_in_a_dry_run() {
    let sandbox = Sandbox::new();
    sandbox.write("source/Rel/x.mkv", "video");

    let run = carry_out(&sandbox, true);

    assert_eq!(run.error, None);
    assert!(matches!(
        run.done.as_slice(),
        [Done::Moved(Applied::Preview { .. })]
    ));
    assert!(sandbox.exists("source/Rel/x.mkv"));
    assert!(!sandbox.exists("target"));
}

// @behavior EFF-008
#[test]
fn should_move_a_file_by_the_route_that_claimed_it() {
    let sandbox = Sandbox::new();
    sandbox.write("source/x.mkv", "video");
    sandbox.make_dir("a");
    sandbox.make_dir("b");
    let config = Config::parse(&format!(
        "[pipeline.video]\nstages = [{{ filter = {{ ext = [\"mkv\"] }} }}]\n\n[pipeline.rest]\nstages = []\n\n\
         [target.a]\npath = \"{}\"\n\n[target.b]\npath = \"{}\"\n\n\
         [watch.w]\nsource = \"{}\"\nroutes = [{{ pipeline = \"video\", move = \"a\" }}, {{ pipeline = \"rest\", move = \"b\" }}]\n",
        sandbox.path("a").display(),
        sandbox.path("b").display(),
        sandbox.path("source").display(),
    ))
    .expect("the configuration should be accepted");

    let processed = process_batch(
        &FsTree,
        &config.watches()[0],
        Path::new(""),
        &[PathBuf::from("x.mkv")],
        &mut Renames::new(),
    );

    assert_eq!(processed[0].what, What::Moved(sandbox.path("a/x.mkv")));
}
