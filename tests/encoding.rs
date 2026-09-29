#![cfg(unix)]

mod common;

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use auto_renamer::{Outcome, Record, Verdict};
use chrono::{TimeZone, Utc};
#[cfg(target_os = "linux")]
use common::Sandbox;
use common::{apply, planned_batch};

/// A path in `Show` whose name is not valid UTF-8.
fn odd_path() -> PathBuf {
    Path::new("Show").join(OsStr::from_bytes(b"\xff\xfe.mkv"))
}

fn odd_record() -> Record {
    Record::new(
        &odd_path(),
        Utc.with_ymd_and_hms(2026, 9, 27, 12, 0, 0).unwrap(),
    )
}

// @behavior ENC-001
#[test]
fn should_give_a_path_that_is_not_utf8_no_name_fields() {
    let record = odd_record();

    assert!(!record.is_readable());
    for field in ["name", "ext", "dir", "path"] {
        assert_eq!(record.field(field), None, "{field}");
    }
    assert!(record.field("mtime").is_some());
}

// @behavior ENC-002
#[test]
fn should_refuse_a_stage_that_reads_names() {
    match apply("{ strip = {} }", odd_record()) {
        Outcome::Rejected(rejection) => {
            assert_eq!(rejection.stage, "strip");
            assert!(rejection.reason.contains("UTF-8"), "{}", rejection.reason);
        }
        other => panic!("expected a refusal by `strip`, got {other:?}"),
    }
}

// @behavior ENC-003
#[test]
fn should_not_let_a_filter_claim_such_a_file() {
    let judged = planned_batch(
        &[("video", r#"[{ filter = { ext = ["mkv"] } }, "move"]"#)],
        vec![odd_record()],
    );

    assert_eq!(judged[0].pipeline, None);
}

// @behavior ENC-004
#[test]
fn should_refuse_for_its_name_a_file_no_pipeline_claims() {
    let judged = planned_batch(
        &[("video", r#"[{ filter = { ext = ["mkv"] } }, "move"]"#)],
        vec![odd_record()],
    );

    match &judged[0].verdict {
        Verdict::Rejected(rejection) => assert_eq!(rejection.stage, "name"),
        other => panic!("expected the file to be refused, got {other:?}"),
    }
}

// @behavior ENC-005
#[test]
fn should_keep_the_bytes_of_the_name_under_a_folder_stage() {
    let judged = planned_batch(
        &[("photo", r#"[{ folder = "{mtime:%Y}" }, "move"]"#)],
        vec![odd_record()],
    );

    let Verdict::Planned(record) = &judged[0].verdict else {
        panic!(
            "expected the file to be planned, got {:?}",
            judged[0].verdict
        );
    };
    let expected = Path::new("Show/2026").join(OsStr::from_bytes(b"\xff\xfe.mkv"));
    assert_eq!(record.plan(), expected);
}

// @behavior ENC-006
#[cfg(target_os = "linux")]
#[test]
fn should_move_a_file_by_its_own_bytes() {
    let sandbox = Sandbox::new();
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

    auto_renamer::process_batch(
        &config.watches()[0],
        Path::new("Show"),
        &[odd_path()],
        &mut auto_renamer::Renames::new(),
    );

    let moved = std::fs::read_to_string(sandbox.path("target/Show").join(name));
    assert_eq!(moved.as_deref().ok(), Some("video"));
    assert!(!sandbox.path("source/Show").join(name).exists());
}

// @behavior ENC-007
#[cfg(target_os = "linux")]
#[test]
fn should_not_make_a_conflict_suffix_from_such_a_name() {
    let sandbox = Sandbox::new();
    sandbox.make_dir("source/Show");
    sandbox.make_dir("target/Show");
    let name = OsStr::from_bytes(b"\xff\xfe.mkv");
    std::fs::write(sandbox.path("source/Show").join(name), "new").unwrap();
    std::fs::write(sandbox.path("target/Show").join(name), "old").unwrap();
    let roots = auto_renamer::Roots {
        source: sandbox.path("source"),
        target: sandbox.path("target"),
    };

    let result = auto_renamer::move_file(
        &common::move_stage(r#"{ move = { on_conflict = "suffix" } }"#),
        &odd_record(),
        &roots,
        false,
    );

    assert!(matches!(
        result,
        Err(auto_renamer::EffectError::Conflict(_))
    ));
    assert!(sandbox.path("source/Show").join(name).exists());
}
