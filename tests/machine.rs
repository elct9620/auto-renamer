use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

use auto_renamer::{Config, Event, Machine, Ready};

/// A machine for a watch with a window of 5 minutes and a maximum wait of 30, plus the extra settings.
fn machine(extra: &str) -> Machine {
    let text = format!(
        "[pipeline.p]\nstages = [\"move\"]\n\n[watch.w]\nsource = \"/s\"\npipelines = [\"p\"]\nbatch_window = \"5m\"\nbatch_max_wait = \"30m\"\n{extra}\n"
    );
    let config = Config::parse(&text).expect("the configuration should be accepted");
    Machine::new(&config.watches()[0])
}

fn at(minutes: u64, seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000 + minutes * 60 + seconds)
}

fn path(name: &str) -> PathBuf {
    PathBuf::from(name)
}

fn settled(machine: &mut Machine, name: &str, when: SystemTime) {
    machine.observe(Event::Settled(path(name)), when);
}

fn batch(unit: &str, files: &[&str]) -> Ready {
    Ready {
        unit: path(unit),
        files: files.iter().map(|file| path(file)).collect(),
    }
}

// @behavior WCH-001
#[test]
fn should_hand_a_unit_over_once_it_has_been_quiet_for_the_window() {
    let mut machine = machine("");
    settled(&mut machine, "Show/a.mkv", at(0, 0));
    settled(&mut machine, "Show/b.mkv", at(0, 0));

    let ready = machine.ready(at(5, 0));

    assert_eq!(ready, [batch("Show", &["Show/a.mkv", "Show/b.mkv"])]);
}

// @behavior WCH-002
#[test]
fn should_hand_nothing_over_before_the_window_has_passed() {
    let mut machine = machine("");
    settled(&mut machine, "Show/a.mkv", at(0, 0));

    assert_eq!(machine.ready(at(4, 59)), []);
}

// @behavior WCH-003
#[test]
fn should_restart_the_wait_when_a_file_settles_in_the_unit() {
    let mut machine = machine("");
    settled(&mut machine, "Show/a.mkv", at(0, 0));
    settled(&mut machine, "Show/b.mkv", at(3, 0));

    assert_eq!(machine.ready(at(5, 0)), []);
    assert_eq!(
        machine.ready(at(8, 0)),
        [batch("Show", &["Show/a.mkv", "Show/b.mkv"])]
    );
}

// @behavior WCH-004
#[test]
fn should_hand_units_over_apart() {
    let mut machine = machine("");
    settled(&mut machine, "A/a.mkv", at(0, 0));
    settled(&mut machine, "B/b.mkv", at(3, 0));

    assert_eq!(machine.ready(at(5, 0)), [batch("A", &["A/a.mkv"])]);
}

// @behavior WCH-005
#[test]
fn should_close_a_batch_that_never_goes_quiet_at_the_maximum_wait() {
    let mut machine = machine("");
    for step in 0..8 {
        settled(&mut machine, &format!("Show/{step}.mkv"), at(step * 4, 0));
    }

    let ready = machine.ready(at(30, 0));

    assert!(matches!(ready.as_slice(), [Ready { files, .. }] if files.len() == 8));
}

// @behavior WCH-006
#[test]
fn should_hand_files_over_in_the_order_of_their_paths() {
    let mut machine = machine("");
    for name in ["Show/b.mkv", "Show/c.mkv", "Show/a.mkv"] {
        settled(&mut machine, name, at(0, 0));
    }

    let ready = machine.ready(at(5, 0));

    assert_eq!(
        ready,
        [batch("Show", &["Show/a.mkv", "Show/b.mkv", "Show/c.mkv"])]
    );
}

// @behavior WCH-007
#[test]
fn should_never_let_a_folder_configuration_into_a_batch() {
    let mut machine = machine("");
    settled(&mut machine, "Show/auto-renamer.toml", at(0, 0));
    settled(&mut machine, "Show/a.mkv", at(0, 0));

    assert_eq!(machine.ready(at(5, 0)), [batch("Show", &["Show/a.mkv"])]);
}

// @behavior WCH-008
#[test]
fn should_hand_over_a_batch_over_the_limit_for_the_service_to_judge() {
    let mut machine = machine("batch_max = 2");
    for name in ["Show/a.mkv", "Show/b.mkv", "Show/c.mkv"] {
        settled(&mut machine, name, at(0, 0));
    }

    let ready = machine.ready(at(5, 0));

    assert_eq!(
        ready,
        [batch("Show", &["Show/a.mkv", "Show/b.mkv", "Show/c.mkv"])]
    );
}

// @behavior WCH-010
#[test]
fn should_keep_a_unit_open_while_a_file_in_it_is_being_written() {
    let mut machine = machine("");
    settled(&mut machine, "Show/a.mkv", at(0, 0));
    machine.observe(Event::Writing(path("Show/b.mkv")), at(4, 0));

    assert_eq!(machine.ready(at(20, 0)), []);
}

// @behavior WCH-011
#[test]
fn should_let_a_file_that_is_gone_leave_the_batch_and_release_the_hold() {
    let mut machine = machine("");
    settled(&mut machine, "Show/a.mkv", at(0, 0));
    machine.observe(Event::Writing(path("Show/b.mkv")), at(1, 0));
    machine.observe(Event::Gone(path("Show/b.mkv")), at(2, 0));

    assert_eq!(machine.ready(at(7, 0)), [batch("Show", &["Show/a.mkv"])]);
}

// @behavior WCH-012
#[test]
fn should_take_a_settled_file_written_to_again_out_until_it_settles_again() {
    let mut machine = machine("");
    settled(&mut machine, "Show/a.mkv", at(0, 0));
    machine.observe(Event::Writing(path("Show/a.mkv")), at(1, 0));
    settled(&mut machine, "Show/a.mkv", at(2, 0));

    assert_eq!(machine.ready(at(7, 0)), [batch("Show", &["Show/a.mkv"])]);
}

// @behavior WCH-013
#[test]
fn should_settle_a_file_found_at_start_that_was_changed_long_ago() {
    let mut machine = machine("");
    machine.observe(
        Event::Found {
            path: path("Show/a.mkv"),
            modified: at(0, 0),
        },
        at(10, 0),
    );

    assert_eq!(machine.ready(at(15, 0)), [batch("Show", &["Show/a.mkv"])]);
}

// @behavior WCH-014
#[test]
fn should_settle_a_file_found_at_start_after_a_window_with_no_writes() {
    let mut machine = machine("");
    machine.observe(
        Event::Found {
            path: path("Show/a.mkv"),
            modified: at(9, 0),
        },
        at(10, 0),
    );

    assert_eq!(machine.ready(at(13, 0)), []);
    assert_eq!(machine.ready(at(14, 0)), [batch("Show", &["Show/a.mkv"])]);
}

// @behavior WCH-015
#[test]
fn should_make_a_file_found_at_start_and_written_again_wait_for_its_close() {
    let mut machine = machine("");
    machine.observe(
        Event::Found {
            path: path("Show/a.mkv"),
            modified: at(9, 0),
        },
        at(10, 0),
    );
    machine.observe(Event::Writing(path("Show/a.mkv")), at(12, 0));

    assert_eq!(machine.ready(at(30, 0)), []);
}

// @behavior WCH-016
#[test]
fn should_drop_a_hold_that_never_resolves_after_the_maximum_wait() {
    let mut machine = machine("");
    machine.observe(Event::Writing(path("Show/a.mkv")), at(0, 0));
    settled(&mut machine, "Show/b.mkv", at(1, 0));

    assert_eq!(machine.ready(at(31, 0)), [batch("Show", &["Show/b.mkv"])]);
}

// @behavior WCH-017
#[test]
fn should_say_when_to_look_again() {
    let mut machine = machine("");
    assert_eq!(machine.next_deadline(), None);

    settled(&mut machine, "Show/a.mkv", at(0, 0));

    assert_eq!(machine.next_deadline(), Some(at(5, 0)));
}

// @behavior WCH-018
#[test]
fn should_follow_the_unit_of_the_watch() {
    let mut machine = machine("unit = { root = [\"Movies/*\"] }");
    settled(&mut machine, "Movies/A/a.mkv", at(0, 0));
    settled(&mut machine, "Movies/A/Subs/a.ass", at(0, 0));

    let ready = machine.ready(at(5, 0));

    assert_eq!(
        ready,
        [batch(
            "Movies/A",
            &["Movies/A/Subs/a.ass", "Movies/A/a.mkv"]
        )]
    );
}

// @behavior WCH-019
#[test]
fn should_take_a_modification_time_in_the_future_as_now() {
    let mut machine = machine("");
    machine.observe(
        Event::Found {
            path: path("Show/a.mkv"),
            modified: at(60 * 24 * 365, 0),
        },
        at(0, 0),
    );

    assert_eq!(machine.ready(at(5, 0)), [batch("Show", &["Show/a.mkv"])]);
}

// @behavior WCH-020
#[test]
fn should_not_stop_for_an_extreme_modification_time() {
    let mut machine = machine("");
    machine.observe(
        Event::Found {
            path: path("Show/a.mkv"),
            modified: SystemTime::UNIX_EPOCH + Duration::from_secs(i64::MAX as u64 - 1),
        },
        at(0, 0),
    );

    assert_eq!(machine.ready(at(5, 0)), [batch("Show", &["Show/a.mkv"])]);
}

// @behavior WCH-021
#[test]
fn should_include_the_end_of_a_hold_in_the_deadline() {
    let mut machine = machine("");
    machine.observe(Event::Writing(path("Show/a.mkv")), at(0, 0));

    assert_eq!(machine.next_deadline(), Some(at(30, 0)));
}

// @behavior WCH-022
#[test]
fn should_stop_collecting_files_of_a_unit_past_the_ceiling() {
    let mut machine = machine("unit = \"directory\"\nbatch_max = 100000");
    for number in 0..100_005 {
        settled(&mut machine, &format!("Show/{number}.mkv"), at(0, 0));
    }

    let ready = machine.ready(at(6, 0));

    match ready.as_slice() {
        [Ready { files, .. }] => assert!(files.len() <= 100_001, "{}", files.len()),
        other => panic!("expected one batch, got {} entries", other.len()),
    }
}

// @behavior WCH-023
#[test]
fn should_take_the_files_under_a_folder_that_is_gone_out_of_the_batch() {
    let mut machine = machine("unit = \"source\"");
    settled(&mut machine, "Sub/a.mkv", at(0, 0));
    settled(&mut machine, "Show/b.mkv", at(0, 0));

    machine.observe(Event::Gone(path("Sub")), at(1, 0));

    assert_eq!(machine.ready(at(5, 0)), [batch("", &["Show/b.mkv"])]);
}

// @behavior WCH-024
#[test]
fn should_release_the_holds_under_a_folder_that_is_gone() {
    let mut machine = machine("unit = \"source\"");
    settled(&mut machine, "Show/a.mkv", at(0, 0));
    machine.observe(Event::Writing(path("Sub/b.mkv")), at(0, 0));

    machine.observe(Event::Gone(path("Sub")), at(1, 0));

    assert_eq!(machine.ready(at(5, 0)), [batch("", &["Show/a.mkv"])]);
}

// @behavior WCH-025
#[test]
fn should_take_the_units_under_a_folder_that_is_gone_and_no_other() {
    let mut machine = machine("");
    settled(&mut machine, "Show/S1/a.mkv", at(0, 0));
    settled(&mut machine, "Show/S2/b.mkv", at(0, 0));
    settled(&mut machine, "Show-2/c.mkv", at(0, 0));

    machine.observe(Event::Gone(path("Show")), at(1, 0));

    assert_eq!(
        machine.ready(at(5, 0)),
        [batch("Show-2", &["Show-2/c.mkv"])]
    );
}

// @behavior WCH-026
#[test]
fn should_leave_nothing_to_wait_for_of_the_units_under_a_folder_that_is_gone() {
    let mut machine = machine("");
    machine.observe(Event::Writing(path("Show/S1/a.mkv")), at(0, 0));

    machine.observe(Event::Gone(path("Show")), at(1, 0));

    assert_eq!(machine.next_deadline(), None);
}

/// How long a machine may take over many files before its cost is taken to grow with how many wait.
/// Work that grows with them takes minutes here, and work that does not takes a fraction of a second.
const QUICK: Duration = Duration::from_secs(5);

/// Runs the step the given number of times, stopping as soon as it has taken too long.
fn stays_quick(times: usize, mut step: impl FnMut(usize)) {
    let started = Instant::now();
    for turn in 0..times {
        step(turn);
        assert!(started.elapsed() < QUICK, "too slow by turn {turn}");
    }
}

fn file(folder: usize, number: usize) -> PathBuf {
    PathBuf::from(format!("T{folder:05}/{number:02}.mkv"))
}

// @behavior WCH-027
#[test]
fn should_stay_quick_to_ask_however_many_files_are_held() {
    let mut machine = machine("");
    for folder in 0..10_000 {
        for number in 0..10 {
            let found = Event::Found {
                path: file(folder, number),
                modified: at(0, 0),
            };
            machine.observe(found, at(1, 0));
        }
    }

    stays_quick(10_000, |_| {
        machine.ready(at(1, 0));
        machine.next_deadline();
    });
}

// @behavior WCH-028
#[test]
fn should_stay_quick_to_ask_with_many_units_waiting_beside_many_held_files() {
    let mut machine = machine("");
    for folder in 0..5_000 {
        settled(&mut machine, &format!("S{folder:05}/a.mkv"), at(0, 0));
    }
    for folder in 0..5_000 {
        for number in 0..10 {
            machine.observe(Event::Writing(file(folder, number)), at(0, 0));
        }
    }

    stays_quick(1_000, |_| {
        machine.ready(at(1, 0));
        machine.next_deadline();
    });
}

// @behavior WCH-029
#[test]
fn should_stay_quick_to_forget_files_however_many_are_waiting() {
    let mut machine = machine("");
    for folder in 0..10_000 {
        for number in 0..10 {
            machine.observe(Event::Settled(file(folder, number)), at(0, 0));
        }
    }

    stays_quick(10_000, |turn| {
        machine.observe(Event::Gone(file(turn, 0)), at(1, 0));
    });
}

fn found(machine: &mut Machine, name: &str, modified: SystemTime, now: SystemTime) {
    let found = Event::Found {
        path: path(name),
        modified,
    };
    machine.observe(found, now);
}

// @behavior WCH-030
#[test]
fn should_make_a_file_being_written_that_a_scan_then_finds_wait_for_its_close() {
    let mut machine = machine("");
    machine.observe(Event::Writing(path("Show/a.mkv")), at(0, 0));

    found(&mut machine, "Show/a.mkv", at(1, 0), at(1, 0));

    assert_eq!(machine.ready(at(20, 0)), []);
}

// @behavior WCH-031
#[test]
fn should_not_hold_back_a_settled_file_that_a_scan_then_finds() {
    let mut machine = machine("");
    settled(&mut machine, "Show/a.mkv", at(0, 0));

    found(&mut machine, "Show/a.mkv", at(1, 0), at(1, 0));

    assert_eq!(machine.ready(at(5, 0)), [batch("Show", &["Show/a.mkv"])]);
}

// @behavior WCH-032
#[test]
fn should_stay_quick_to_be_told_of_files_however_many_it_holds() {
    let mut machine = machine("");

    stays_quick(2_000, |folder| {
        for number in 0..10 {
            found(
                &mut machine,
                &format!("F{folder:05}/{number:02}.mkv"),
                at(0, 0),
                at(1, 0),
            );
            machine.observe(
                Event::Writing(path(&format!("W{folder:05}/{number:02}.mkv"))),
                at(1, 0),
            );
            settled(
                &mut machine,
                &format!("S{folder:05}/{number:02}.mkv"),
                at(1, 0),
            );
        }
    });
}
