mod common;

use std::path::{Path, PathBuf};

use auto_renamer::{Roots, cleanup_folders};
use common::{Sandbox, cleanup_stage};

/// The folders cleaned up after the file `origin` left, given the unit it belongs to.
fn cleaned(sandbox: &Sandbox, keep: &str, origin: &str, unit: &str) -> Vec<PathBuf> {
    let roots = Roots {
        source: sandbox.path("source"),
        target: sandbox.path("target"),
    };
    let stage = cleanup_stage(&format!("{{ cleanup = {{ keep = [{keep}] }} }}"));

    cleanup_folders(&stage, Path::new(origin), Path::new(unit), &roots)
        .expect("the cleanup should not fail")
}

fn paths(names: &[&str]) -> Vec<PathBuf> {
    names.iter().map(PathBuf::from).collect()
}

// @behavior CLN-001
#[test]
fn should_remove_the_folder_a_file_left_empty() {
    let sandbox = Sandbox::new();
    sandbox.make_dir("source/Rel");

    let removed = cleaned(&sandbox, "", "Rel/x.mkv", "Rel");

    assert_eq!(removed, paths(&["Rel"]));
    assert!(!sandbox.exists("source/Rel"));
}

// @behavior CLN-002
#[test]
fn should_keep_a_folder_that_still_holds_a_file() {
    let sandbox = Sandbox::new();
    sandbox.write("source/Rel/y.mkv", "video");

    let removed = cleaned(&sandbox, "", "Rel/x.mkv", "Rel");

    assert!(removed.is_empty());
    assert!(sandbox.exists("source/Rel/y.mkv"));
}

// @behavior CLN-003
#[test]
fn should_remove_folders_emptied_one_inside_another_upward() {
    let sandbox = Sandbox::new();
    sandbox.make_dir("source/Season/Rel/Subs");

    let removed = cleaned(&sandbox, "", "Season/Rel/Subs/x.ass", "Season");

    assert_eq!(removed, paths(&["Season/Rel/Subs", "Season/Rel", "Season"]));
    assert!(!sandbox.exists("source/Season"));
}

// @behavior CLN-004
#[test]
fn should_stop_at_a_folder_a_keep_pattern_names() {
    let sandbox = Sandbox::new();
    sandbox.make_dir("source/Series/Season 01/Rel");

    let removed = cleaned(
        &sandbox,
        r#""Season *""#,
        "Series/Season 01/Rel/x.mkv",
        "Series",
    );

    assert_eq!(removed, paths(&["Series/Season 01/Rel"]));
    assert!(sandbox.exists("source/Series/Season 01"));
}

// @behavior CLN-005
#[test]
fn should_never_remove_the_source_itself() {
    let sandbox = Sandbox::new();
    sandbox.make_dir("source/Rel");

    let removed = cleaned(&sandbox, "", "Rel/x.mkv", "");

    assert_eq!(removed, paths(&["Rel"]));
    assert!(sandbox.exists("source"));
}

// @behavior CLN-006
#[test]
fn should_not_touch_folders_above_the_unit() {
    let sandbox = Sandbox::new();
    sandbox.make_dir("source/Movies/A");

    let removed = cleaned(&sandbox, "", "Movies/A/x.mkv", "Movies/A");

    assert_eq!(removed, paths(&["Movies/A"]));
    assert!(sandbox.exists("source/Movies"));
}

// @behavior CLN-007
#[test]
fn should_keep_a_folder_holding_a_folder_configuration() {
    let sandbox = Sandbox::new();
    sandbox.write("source/Rel/auto-renamer.toml", "");

    let removed = cleaned(&sandbox, "", "Rel/x.mkv", "Rel");

    assert!(removed.is_empty());
    assert!(sandbox.exists("source/Rel/auto-renamer.toml"));
}

// @behavior CLN-008
#[test]
fn should_not_remove_a_linked_folder() {
    let sandbox = Sandbox::new();
    sandbox.make_dir("source");
    sandbox.make_dir("elsewhere");
    std::os::unix::fs::symlink(sandbox.path("elsewhere"), sandbox.path("source/Rel")).unwrap();

    let removed = cleaned(&sandbox, "", "Rel/x.mkv", "Rel");

    assert!(removed.is_empty());
    assert!(sandbox.exists("source/Rel"));
    assert!(sandbox.exists("elsewhere"));
}
