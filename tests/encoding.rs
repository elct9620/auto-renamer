#![cfg(unix)]

mod common;

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use auto_renamer::Record;
use chrono::{TimeZone, Utc};

/// A path in `Show` whose name is not valid UTF-8.
fn odd_path() -> PathBuf {
    Path::new("Show").join(OsStr::from_bytes(b"\xff\xfe.mkv"))
}

// @behavior ENC-001
#[test]
fn should_make_no_record_of_a_path_that_is_not_utf8() {
    let mtime = Utc.with_ymd_and_hms(2026, 9, 27, 12, 0, 0).unwrap();

    let refusal = Record::new(&odd_path(), mtime).expect_err("no record should be made");

    assert!(refusal.to_string().contains("UTF-8"), "{refusal}");
}

// @behavior ENC-002
#[cfg(target_os = "linux")]
#[test]
fn should_refuse_a_file_whose_path_is_not_utf8_and_leave_it() {
    let sandbox = common::Sandbox::new();
    sandbox.make_dir("source/Show");
    sandbox.make_dir("target");
    let name = OsStr::from_bytes(b"\xff\xfe.mkv");
    std::fs::write(sandbox.path("source/Show").join(name), "video").unwrap();
    let config = auto_renamer::Config::parse(&format!(
        "[pipeline.p]\nstages = [\"move\"]\n\n[watch.w]\nsource = \"{}\"\ntarget = \"{}\"\npipelines = [\"p\"]\n",
        sandbox.path("source").display(),
        sandbox.path("target").display(),
    ))
    .unwrap();

    let processed = auto_renamer::process_batch(
        &config.watches()[0],
        Path::new("Show"),
        &[odd_path()],
        &mut auto_renamer::Renames::new(),
    );

    assert!(
        matches!(&processed[0].what, auto_renamer::What::Refused(reason) if reason == "name: the file name is not valid UTF-8"),
        "{:?}",
        processed[0].what
    );
    assert!(sandbox.path("source/Show").join(name).exists());
    assert!(!sandbox.path("target/Show").join(name).exists());
}

// @behavior ENC-003
#[cfg(target_os = "linux")]
#[test]
fn should_not_make_a_conflict_suffix_from_a_name_that_is_not_utf8() {
    let sandbox = common::Sandbox::new();
    sandbox.write("source/Show/x.mkv", "new");
    sandbox.make_dir("target/Show");
    let name = OsStr::from_bytes(b"\xff\xfe.mkv");
    std::fs::write(sandbox.path("target/Show").join(name), "old").unwrap();
    let mut planned = common::record("Show/x.mkv");
    planned.set_plan(odd_path());
    let roots = auto_renamer::Roots {
        source: sandbox.path("source"),
        target: sandbox.path("target"),
    };

    let result = auto_renamer::move_file(
        &common::move_stage(r#"{ move = { on_conflict = "suffix" } }"#),
        &planned,
        &roots,
        false,
    );

    assert!(matches!(
        result,
        Err(auto_renamer::EffectError::Conflict(_))
    ));
    assert!(sandbox.exists("source/Show/x.mkv"));
}
