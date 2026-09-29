mod common;

use std::os::unix::fs::symlink;
use std::path::PathBuf;

use auto_renamer::{Event, scan_folder};
use common::Sandbox;

fn found_paths(events: &[Event]) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = events
        .iter()
        .map(|event| match event {
            Event::Found { path, .. } => path.clone(),
            other => panic!("expected a file found, got {other:?}"),
        })
        .collect();
    paths.sort();
    paths
}

// @behavior SCN-001
#[test]
fn should_find_the_regular_files_below_a_folder_with_their_times() {
    let sandbox = Sandbox::new();
    sandbox.write("Show/a.mkv", "a");
    sandbox.write("Show/Subs/a.ass", "b");

    let events = scan_folder(&sandbox.path(""), &sandbox.path("Show"));

    assert_eq!(
        found_paths(&events),
        [
            PathBuf::from("Show/Subs/a.ass"),
            PathBuf::from("Show/a.mkv")
        ]
    );
    let modified = std::fs::metadata(sandbox.path("Show/a.mkv"))
        .unwrap()
        .modified()
        .unwrap();
    assert!(events.contains(&Event::Found {
        path: PathBuf::from("Show/a.mkv"),
        modified
    }));
}

// @behavior SCN-002
#[test]
fn should_not_list_a_link_to_a_file() {
    let sandbox = Sandbox::new();
    sandbox.write("real.mkv", "a");
    symlink(sandbox.path("real.mkv"), sandbox.path("Show-x.mkv")).unwrap();
    sandbox.make_dir("Show");
    symlink(sandbox.path("real.mkv"), sandbox.path("Show/x.mkv")).unwrap();

    let events = scan_folder(&sandbox.path(""), &sandbox.path("Show"));

    assert_eq!(found_paths(&events), Vec::<PathBuf>::new());
}

// @behavior SCN-003
#[test]
fn should_not_enter_a_linked_folder() {
    let sandbox = Sandbox::new();
    sandbox.write("Elsewhere/a.mkv", "a");
    sandbox.make_dir("Show");
    symlink(sandbox.path("Elsewhere"), sandbox.path("Show/Linked")).unwrap();

    let events = scan_folder(&sandbox.path(""), &sandbox.path("Show"));

    assert_eq!(found_paths(&events), Vec::<PathBuf>::new());
}

// @behavior SCN-004
#[test]
fn should_find_nothing_in_a_folder_that_is_not_there() {
    let sandbox = Sandbox::new();

    assert_eq!(scan_folder(&sandbox.path(""), &sandbox.path("Missing")), []);
}
