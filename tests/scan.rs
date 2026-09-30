mod common;

use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use auto_renamer::{Event, Scan};
use common::Sandbox;

fn scan_folder(source: &Path, folder: &Path) -> Vec<Event> {
    found_by(Scan::new(source, folder), usize::MAX)
}

/// What a scan finds when it looks at a number of entries at a time until it is finished.
fn found_by(mut scan: Scan, entries: usize) -> Vec<Event> {
    let mut found = Vec::new();
    while !scan.is_finished() {
        found.extend(scan.look_at(entries, |_| {}));
    }
    found
}

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

// @behavior SCN-005
#[test]
fn should_let_a_scan_stop_and_go_on() {
    let sandbox = Sandbox::new();
    let files = ["A/1.mkv", "A/2.mkv", "A/Subs/3.ass", "B/4.mkv", "B/C/5.mkv"];
    for file in files {
        sandbox.write(file, "x");
    }

    let found = found_by(Scan::new(&sandbox.path(""), &sandbox.path("")), 3);

    let mut expected: Vec<PathBuf> = files.iter().map(PathBuf::from).collect();
    expected.sort();
    assert_eq!(found_paths(&found), expected);
}

// @behavior SCN-006
#[test]
fn should_count_a_folder_a_scan_opens_as_an_entry_looked_at() {
    let sandbox = Sandbox::new();
    for number in 0..100 {
        sandbox.make_dir(&format!("Empty/{number}"));
    }
    let mut scan = Scan::new(&sandbox.path(""), &sandbox.path("Empty"));

    scan.look_at(110, |_| {});

    assert!(!scan.is_finished());
}

// @behavior SCN-007
#[test]
fn should_tell_of_each_folder_before_a_scan_reads_it() {
    let sandbox = Sandbox::new();
    sandbox.write("A/1.mkv", "x");
    sandbox.write("A/B/2.mkv", "x");
    let mut scan = Scan::new(&sandbox.path(""), &sandbox.path("A"));
    let mut seen: Vec<PathBuf> = Vec::new();

    while !scan.is_finished() {
        let mut entered = Vec::new();
        let found = scan.look_at(1, |folder| entered.push(folder.to_path_buf()));
        seen.extend(entered);
        seen.extend(
            found_paths(&found)
                .into_iter()
                .map(|file| sandbox.path("").join(file)),
        );
    }

    let place = |path: &str| seen.iter().position(|seen| *seen == sandbox.path(path));
    assert!(
        place("A").is_some() && place("A") < place("A/1.mkv"),
        "{seen:?}"
    );
    assert!(
        place("A/B").is_some() && place("A/B") < place("A/B/2.mkv"),
        "{seen:?}"
    );
}
