#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use auto_renamer::{
    ConfigError, Context, Judged, Outcome, Pipeline, Record, Stage, Target, Value, Verdict,
    plan_batch,
};
use chrono::{TimeZone, Utc};

/// The record of a file, modified at a fixed time.
pub fn record(path: &str) -> Record {
    Record::new(
        Path::new(path),
        Utc.with_ymd_and_hms(2026, 9, 27, 12, 0, 0).unwrap(),
    )
}

/// The record of a file modified at noon UTC on the given day.
pub fn record_on(path: &str, year: i32, month: u32, day: u32) -> Record {
    Record::new(
        Path::new(path),
        Utc.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap(),
    )
}

pub fn with(mut record: Record, field: &str, value: Value) -> Record {
    record.set_field(field, value);
    record
}

pub fn text(value: &str) -> Value {
    Value::Text(value.to_string())
}

/// A stage as it is written in a pipeline, such as `{ number = { into = "episode" } }`.
pub fn stage(declaration: &str) -> Stage {
    let document: toml::Table = format!("stage = {declaration}")
        .parse()
        .expect("the declaration should be TOML");
    Stage::declare(&document["stage"]).expect("the stage should be declared")
}

/// The files a target folder holds, by folder.
pub struct Files(BTreeMap<String, Vec<String>>);

impl Files {
    pub fn none() -> Files {
        Files(BTreeMap::new())
    }

    pub fn of(folders: &[(&str, &[&str])]) -> Files {
        Files(
            folders
                .iter()
                .map(|(folder, names)| {
                    (
                        folder.to_string(),
                        names.iter().map(|name| name.to_string()).collect(),
                    )
                })
                .collect(),
        )
    }
}

impl Target for Files {
    fn files_in(&self, folder: &Path) -> Vec<String> {
        folder
            .to_str()
            .and_then(|folder| self.0.get(folder))
            .cloned()
            .unwrap_or_default()
    }
}

pub fn apply(declaration: &str, record: Record) -> Outcome {
    let files = Files::none();
    stage(declaration).apply(record, &mut Context::new(&files))
}

/// The record a stage passed on.
pub fn passed(outcome: Outcome) -> Record {
    match outcome {
        Outcome::Continue(record) => record,
        other => panic!("expected the record to go on, got {other:?}"),
    }
}

/// The record a stage passed on after running on `record`.
pub fn run(declaration: &str, record: Record) -> Record {
    passed(apply(declaration, record))
}

pub fn assert_rejected_by(outcome: Outcome, stage: &str) {
    match outcome {
        Outcome::Rejected(rejection) => assert_eq!(rejection.stage, stage),
        other => panic!("expected a refusal by `{stage}`, got {other:?}"),
    }
}

/// Pipelines by name, each written as the list of its stages.
pub fn pipelines(list: &[(&str, &str)]) -> Vec<(String, Pipeline)> {
    list.iter()
        .map(|(name, stages)| {
            let pipeline = Pipeline::from_toml(&format!("stages = {stages}"))
                .expect("the pipeline should be readable");
            (name.to_string(), pipeline)
        })
        .collect()
}

/// A batch planned against an empty target.
pub fn planned_batch(list: &[(&str, &str)], records: Vec<Record>) -> Vec<Judged> {
    let files = Files::none();
    plan_batch(&pipelines(list), records, &mut Context::new(&files))
}

pub fn verdict<'a>(judged: &'a [Judged], origin: &str) -> &'a Verdict {
    &judged
        .iter()
        .find(|entry| entry.origin == Path::new(origin))
        .unwrap_or_else(|| panic!("no verdict for {origin}"))
        .verdict
}

/// The record a batch planned for a file.
pub fn planned_record<'a>(judged: &'a [Judged], origin: &str) -> &'a Record {
    match verdict(judged, origin) {
        Verdict::Planned(record) => record,
        other => panic!("expected {origin} to be planned, got {other:?}"),
    }
}

pub fn assert_refused_by(judged: &[Judged], origin: &str, stage: &str) {
    match verdict(judged, origin) {
        Verdict::Rejected(rejection) => assert_eq!(rejection.stage, stage),
        other => panic!("expected {origin} to be refused by `{stage}`, got {other:?}"),
    }
}

pub fn number(value: u64) -> Value {
    Value::Number(value)
}

/// Whether a refused configuration names the key, or the pipeline, that was wanted.
pub fn names(error: &ConfigError, wanted: &str) -> bool {
    match error {
        ConfigError::Unknown { key, .. } | ConfigError::Invalid { key, .. } => key == wanted,
        ConfigError::Pipeline { name, .. } => name == wanted,
        _ => false,
    }
}

/// A folder of its own under the temporary folder, removed when it goes out of scope.
pub struct Sandbox {
    root: PathBuf,
}

static SANDBOXES: AtomicUsize = AtomicUsize::new(0);

impl Sandbox {
    pub fn new() -> Sandbox {
        Sandbox::under(&std::env::temp_dir())
    }

    /// A sandbox under another folder, such as one on another filesystem.
    pub fn under(parent: &Path) -> Sandbox {
        let count = SANDBOXES.fetch_add(1, Ordering::SeqCst);
        let root = parent.join(format!("auto-renamer-{}-{count}", std::process::id()));
        std::fs::create_dir_all(&root).expect("the sandbox should be created");
        Sandbox { root }
    }

    pub fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    pub fn write(&self, relative: &str, content: &str) {
        let path = self.path(relative);
        std::fs::create_dir_all(path.parent().unwrap()).expect("the folder should be created");
        std::fs::write(path, content).expect("the file should be written");
    }

    pub fn read(&self, relative: &str) -> Option<String> {
        std::fs::read_to_string(self.path(relative)).ok()
    }

    pub fn exists(&self, relative: &str) -> bool {
        std::fs::symlink_metadata(self.path(relative)).is_ok()
    }

    pub fn make_dir(&self, relative: &str) {
        std::fs::create_dir_all(self.path(relative)).expect("the folder should be created");
    }

    /// The names left in a folder, hidden ones included.
    pub fn names_in(&self, relative: &str) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(self.path(relative))
            .map(|entries| {
                entries
                    .filter_map(|entry| {
                        Some(entry.ok()?.file_name().to_string_lossy().into_owned())
                    })
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        names
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// A move stage as it is written in a pipeline, such as `"move"`.
pub fn move_stage(declaration: &str) -> auto_renamer::stages::Move {
    match stage(declaration) {
        Stage::Move(stage) => stage,
        other => panic!("expected a move stage, got {other:?}"),
    }
}
