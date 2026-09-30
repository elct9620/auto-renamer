#![cfg(target_os = "linux")]

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use auto_renamer::{Options, RunError, run};
use common::{Sandbox, eventually, eventually_within};

fn config_of(sandbox: &Sandbox, stages: &str) -> String {
    format!(
        "[pipeline.p]\nstages = {stages}\n\n[watch.w]\nsource = \"{}\"\ntarget = \"{}\"\npipelines = [\"p\"]\nunit = \"source\"\nbatch_window = \"1s\"\nbatch_max_wait = \"3s\"\n",
        sandbox.path("source").display(),
        sandbox.path("target").display(),
    )
}

/// A watcher running on another thread over `sandbox`, stopped when it goes out of scope.
struct Running {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<Result<(), RunError>>>,
}

impl Running {
    fn start(sandbox: &Sandbox, stages: &str) -> Running {
        sandbox.make_dir("source");
        sandbox.make_dir("target");
        sandbox.write("config.toml", &config_of(sandbox, stages));
        Running::spawn(sandbox)
    }

    /// Starts the watcher and gives it time to be watching.
    fn spawn(sandbox: &Sandbox) -> Running {
        let running = Running::spawn_without_waiting(sandbox);
        thread::sleep(Duration::from_millis(300));
        running
    }

    fn spawn_without_waiting(sandbox: &Sandbox) -> Running {
        let stop = Arc::new(AtomicBool::new(false));
        let options = Options {
            config: sandbox.path("config.toml"),
        };
        let flag = stop.clone();
        let thread = thread::spawn(move || run(&options, &flag, &AtomicBool::new(false)));
        Running {
            stop,
            thread: Some(thread),
        }
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Long enough for a change to the configuration to be read.
fn let_the_configuration_settle() {
    thread::sleep(Duration::from_millis(2500));
}

// @behavior RUN-001
#[test]
fn should_move_a_file_dropped_in_the_source_to_the_target() {
    let sandbox = Sandbox::new();
    let _running = Running::start(&sandbox, r#"["move"]"#);

    sandbox.write("source/a.mkv", "video");

    assert!(eventually(|| sandbox.exists("target/a.mkv")));
    assert!(!sandbox.exists("source/a.mkv"));
}

// @behavior RUN-002
#[test]
fn should_process_the_files_already_there_at_start() {
    let sandbox = Sandbox::new();
    sandbox.write("source/a.mkv", "video");
    let _running = Running::start(&sandbox, r#"["move"]"#);

    assert!(eventually(|| sandbox.exists("target/a.mkv")));
}

// @behavior RUN-003
#[test]
fn should_follow_a_change_to_the_configuration() {
    let sandbox = Sandbox::new();
    let _running = Running::start(&sandbox, r#"["move"]"#);

    sandbox.write(
        "config.toml",
        &config_of(&sandbox, r#"[{ format = "renamed" }, "move"]"#),
    );
    let_the_configuration_settle();
    sandbox.write("source/a.mkv", "video");

    assert!(eventually(|| sandbox.exists("target/renamed.mkv")));
}

// @behavior RUN-004
#[test]
fn should_keep_the_running_configuration_when_the_new_one_is_not_valid() {
    let sandbox = Sandbox::new();
    let _running = Running::start(&sandbox, r#"[{ format = "kept" }, "move"]"#);

    sandbox.write("config.toml", "this is not toml");
    let_the_configuration_settle();
    sandbox.write("source/a.mkv", "video");

    assert!(eventually(|| sandbox.exists("target/kept.mkv")));
}

// @behavior RUN-008
#[test]
fn should_process_a_folder_moved_into_the_source_with_what_it_holds() {
    let sandbox = Sandbox::new();
    let _running = Running::start(&sandbox, r#"["move"]"#);
    sandbox.write("outside/Show/a.mkv", "video");

    std::fs::rename(sandbox.path("outside/Show"), sandbox.path("source/Show")).unwrap();

    assert!(eventually(|| sandbox.exists("target/Show/a.mkv")));
}

// @behavior RUN-019
#[test]
fn should_not_miss_a_file_written_into_a_folder_that_has_just_appeared() {
    let sandbox = Sandbox::new();
    let _running = Running::start(&sandbox, r#"["move"]"#);
    let folders: Vec<String> = (0..300).map(|number| format!("Big/{number:04}")).collect();
    for folder in &folders {
        sandbox.make_dir(&format!("outside/{folder}"));
    }

    std::fs::rename(sandbox.path("outside/Big"), sandbox.path("source/Big")).unwrap();
    for folder in &folders {
        sandbox.write(&format!("source/{folder}/a.mkv"), "video");
    }

    let missing = || {
        folders
            .iter()
            .filter(|folder| !sandbox.exists(&format!("target/{folder}/a.mkv")))
            .count()
    };
    assert!(
        eventually_within(30, || missing() == 0),
        "{} missing",
        missing()
    );
}

// @behavior RUN-021
#[test]
fn should_process_a_folder_renamed_inside_the_source_under_its_new_name() {
    let sandbox = Sandbox::new();
    let _running = Running::start(&sandbox, r#"["move"]"#);
    sandbox.write("source/Old/a.mkv", "video");
    thread::sleep(Duration::from_millis(200));

    std::fs::rename(sandbox.path("source/Old"), sandbox.path("source/New")).unwrap();

    assert!(eventually(|| sandbox.exists("target/New/a.mkv")));
    assert!(!sandbox.exists("target/Old"));
}

// @behavior RUN-022
#[test]
fn should_not_hand_a_batch_over_before_the_scan_that_finds_its_files_is_finished() {
    let sandbox = Sandbox::new();
    for number in 0..600 {
        sandbox.write(&format!("source/Show/f{number:03}.mkv"), "video");
    }
    thread::sleep(Duration::from_millis(1200));

    let numbered =
        r#"[{ rank = { into = "index", by = ["ext"] } }, { format = "{index:03}" }, "move"]"#;
    let _running = Running::start(&sandbox, numbered);

    assert!(
        eventually(|| sandbox.names_in("target/Show").len() == 600),
        "{} in the target",
        sandbox.names_in("target/Show").len()
    );
}

// @behavior RUN-023
#[test]
fn should_answer_a_stop_between_batches() {
    let sandbox = Sandbox::new();
    for number in 0..10_000 {
        sandbox.write(&format!("source/T{number:05}/a.mkv"), "video");
    }
    sandbox.make_dir("target");
    let each_folder_a_unit =
        config_of(&sandbox, r#"["move"]"#).replace(r#"unit = "source""#, r#"unit = "directory""#);
    sandbox.write("config.toml", &each_folder_a_unit);
    thread::sleep(Duration::from_millis(1200));
    let running = Running::spawn_without_waiting(&sandbox);

    let until = Instant::now() + Duration::from_secs(10);
    while sandbox.names_in("target").is_empty() && Instant::now() < until {
        thread::sleep(Duration::from_millis(1));
    }
    drop(running);
    assert!(!sandbox.names_in("target").is_empty());

    let left = (0..10_000)
        .filter(|number| sandbox.exists(&format!("source/T{number:05}/a.mkv")))
        .count();
    assert!(
        left > 0,
        "every batch was processed before the stop was answered"
    );
}

// @behavior RUN-024
#[test]
fn should_process_a_file_written_into_a_folder_that_was_there_at_start() {
    let sandbox = Sandbox::new();
    sandbox.make_dir("source/Show/Season 01");
    let _running = Running::start(&sandbox, r#"["move"]"#);

    sandbox.write("source/Show/Season 01/a.mkv", "video");

    assert!(eventually(|| sandbox.exists("target/Show/Season 01/a.mkv")));
}

// @behavior RUN-009
#[test]
fn should_follow_a_configuration_replaced_by_another_file() {
    let sandbox = Sandbox::new();
    let _running = Running::start(&sandbox, r#"["move"]"#);

    sandbox.write(
        "next.toml",
        &config_of(&sandbox, r#"[{ format = "replaced" }, "move"]"#),
    );
    std::fs::rename(sandbox.path("next.toml"), sandbox.path("config.toml")).unwrap();
    let_the_configuration_settle();
    sandbox.write("source/a.mkv", "video");

    assert!(eventually(|| sandbox.exists("target/replaced.mkv")));
}

// @behavior RUN-017
#[test]
fn should_refuse_a_configuration_file_inside_a_source() {
    let sandbox = Sandbox::new();
    sandbox.make_dir("source");
    sandbox.make_dir("target");
    sandbox.write(
        "source/config.toml",
        &config_of(&sandbox, r#"[{ format = "renamed" }, "move"]"#),
    );
    let options = Options {
        config: sandbox.path("source/config.toml"),
    };

    // Told to stop at once, so a watcher that wrongly starts returns rather than runs on.
    let result = run(&options, &AtomicBool::new(true), &AtomicBool::new(false));

    assert!(
        matches!(&result, Err(RunError::Config(message)) if message.contains("watch.w: `source`")),
        "{result:?}"
    );
}

// @behavior RUN-018
#[test]
fn should_refuse_a_configuration_file_reached_through_a_link_to_a_source() {
    let sandbox = Sandbox::new();
    sandbox.make_dir("settings");
    sandbox.make_dir("target");
    std::os::unix::fs::symlink(sandbox.path("settings"), sandbox.path("source")).unwrap();
    sandbox.write(
        "settings/config.toml",
        &config_of(&sandbox, r#"[{ format = "renamed" }, "move"]"#),
    );
    let options = Options {
        config: sandbox.path("settings/config.toml"),
    };

    // Told to stop at once, so a watcher that wrongly starts returns rather than runs on.
    let result = run(&options, &AtomicBool::new(true), &AtomicBool::new(false));

    assert!(
        matches!(&result, Err(RunError::Config(message)) if message.contains("watch.w: `source`")),
        "{result:?}"
    );
}
