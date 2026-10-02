#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use auto_renamer::{
    Config, ConfigError, Context, Declared, Flow, Judged, Pipeline, Record, Target, Value, Verdict,
    plan_batch,
};
use chrono::{TimeZone, Utc};

/// The record of a file, modified at a fixed time.
pub fn record(path: &str) -> Record {
    Record::new(
        Path::new(path),
        Utc.with_ymd_and_hms(2026, 9, 27, 12, 0, 0).unwrap(),
    )
    .expect("the path should be text")
}

/// The record of a file modified at noon UTC on the given day.
pub fn record_on(path: &str, year: i32, month: u32, day: u32) -> Record {
    Record::new(
        Path::new(path),
        Utc.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap(),
    )
    .expect("the path should be text")
}

pub fn with(mut record: Record, field: &str, value: Value) -> Record {
    record.set_field(field, value);
    record
}

pub fn text(value: &str) -> Value {
    Value::Text(value.to_string())
}

/// A stage as it is written in a pipeline, such as `{ number = { into = "episode" } }`.
pub fn stage(declaration: &str) -> Declared {
    let document: toml::Table = format!("stage = {declaration}")
        .parse()
        .expect("the declaration should be TOML");
    Declared::read(&document["stage"]).expect("the stage should be declared")
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

    fn root(&self) -> &Path {
        Path::new("")
    }
}

pub fn apply(declaration: &str, record: Record) -> Flow {
    let files = Files::none();
    stage(declaration).apply(record, &mut Context::new(&files))
}

/// The record a stage passed on.
pub fn passed(flow: Flow) -> Record {
    flow.unwrap_or_else(|stop| panic!("expected the record to go on, got {stop:?}"))
}

/// The record a stage passed on after running on `record`.
pub fn run(declaration: &str, record: Record) -> Record {
    passed(apply(declaration, record))
}

pub fn assert_rejected_by(flow: Flow, stage: &str) {
    match flow {
        Err(stop) => assert_eq!(stop.stage, stage),
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
    plan_batch(&pipelines(list), records, &files)
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

/// The cleanup of a route as it is written, keeping the folders `keep` names, such as `"Season *"`.
pub fn cleanup_keeping(keep: &str) -> auto_renamer::Cleanup {
    let config = Config::parse(&format!(
        "[pipeline.p]\nstages = []\n\n[watch.w]\nsource = \"/source\"\nroutes = [{{ pipeline = \"p\", cleanup = {{ keep = [{keep}] }} }}]\n"
    ))
    .expect("the route should be read");
    config.watches()[0].routes()[0]
        .cleanup
        .clone()
        .expect("the route cleans up")
}

/// The user and group every Linux keeps for what should own nothing.
const NOBODY: u32 = 65534;

#[derive(PartialEq)]
enum User {
    Same,
    Unprivileged,
}

fn runs_as_root(sandbox: &Sandbox) -> bool {
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata(sandbox.path("")).is_ok_and(|folder| folder.uid() == 0)
}

/// The target table and the move of the route into it.
fn target_of(sandbox: &Sandbox) -> (String, &'static str) {
    sandbox.make_dir("target");
    (
        format!(
            "[target.t]\npath = \"{}\"\n\n",
            sandbox.path("target").display()
        ),
        ", move = \"t\"",
    )
}

/// The program running over a sandbox with `source` and `target` folders, killed if a test ends before it did.
pub struct Program {
    child: std::process::Child,
    log: PathBuf,
}

impl Program {
    /// Starts the program with a pipeline of `stages`, and `extra` lines in the watch.
    pub fn start(sandbox: &Sandbox, stages: &str, extra: &str) -> Program {
        Program::start_without_waiting(sandbox, stages, extra).watching()
    }

    /// Starts the program and comes back at once, for a test that looks at what it does from the start.
    pub fn start_without_waiting(sandbox: &Sandbox, stages: &str, extra: &str) -> Program {
        Program::launch(sandbox, stages, target_of(sandbox), "3s", extra, User::Same)
    }

    /// Starts the program without a target, so that files are renamed where they are, with a long maximum wait.
    pub fn start_in_place(sandbox: &Sandbox, stages: &str) -> Program {
        Program::launch(sandbox, stages, (String::new(), ""), "60s", "", User::Same).watching()
    }

    /// Starts the program as a user that a folder can be closed to, which the user of the tests is not
    /// when they run as root.
    pub fn start_unprivileged(sandbox: &Sandbox, stages: &str, extra: &str) -> Program {
        Program::launch(
            sandbox,
            stages,
            target_of(sandbox),
            "3s",
            extra,
            User::Unprivileged,
        )
        .watching()
    }

    fn launch(
        sandbox: &Sandbox,
        stages: &str,
        (target, moving): (String, &str),
        max_wait: &str,
        extra: &str,
        user: User,
    ) -> Program {
        sandbox.make_dir("source");
        sandbox.write(
            "config.toml",
            &format!(
                "[pipeline.p]\nstages = {stages}\n\n{target}[watch.w]\nsource = \"{}\"\nroutes = [{{ pipeline = \"p\"{moving} }}]\nunit = \"source\"\nquiet = \"1s\"\nmax_wait = \"{max_wait}\"\n{extra}\n",
                sandbox.path("source").display(),
            ),
        );
        let log = sandbox.path("log.txt");
        let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_auto-renamer"));
        command
            .arg("--config")
            .arg(sandbox.path("config.toml"))
            .stderr(std::fs::File::create(&log).expect("the log should be created"));
        if user == User::Unprivileged && runs_as_root(sandbox) {
            use std::os::unix::process::CommandExt;
            command.uid(NOBODY).gid(NOBODY);
        }
        let child = command.spawn().expect("the program should start");
        Program { child, log }
    }

    /// The program once it has had time to be watching.
    fn watching(self) -> Program {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        self
    }

    pub fn signal(&self, name: &str) {
        let status = std::process::Command::new("kill")
            .arg(format!("-{name}"))
            .arg(self.child.id().to_string())
            .status()
            .expect("kill should run");
        assert!(status.success());
    }

    /// Whether the program exited successfully within five seconds.
    pub fn exits_successfully(&mut self) -> bool {
        self.exit_status().is_some_and(|status| status.success())
    }

    /// Whether the program exited with a failure within five seconds.
    pub fn exits_with_a_failure(&mut self) -> bool {
        self.exit_status().is_some_and(|status| !status.success())
    }

    /// Whether the program has not ended.
    pub fn is_running(&mut self) -> bool {
        self.child
            .try_wait()
            .expect("the program should be waited on")
            .is_none()
    }

    /// How the program ended, when it did within five seconds.
    fn exit_status(&mut self) -> Option<std::process::ExitStatus> {
        let until = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while std::time::Instant::now() < until {
            if let Some(status) = self
                .child
                .try_wait()
                .expect("the program should be waited on")
            {
                return Some(status);
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        None
    }

    /// The CPU time the program has used so far, in the hundredths of a second Linux counts it in.
    pub fn cpu_ticks(&self) -> u64 {
        let stat = std::fs::read_to_string(format!("/proc/{}/stat", self.child.id()))
            .expect("the process should be there to read");
        let fields: Vec<&str> = stat.rsplit(')').next().unwrap_or("").split(' ').collect();
        // After the name come the state and ten numbers, then the user and the system time.
        let ticks = |place: usize| fields[place].parse::<u64>().expect("a number of ticks");
        ticks(12) + ticks(13)
    }

    /// What the program has written to its error output so far.
    pub fn log(&self) -> String {
        std::fs::read_to_string(&self.log).unwrap_or_default()
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Whether the condition came true within ten seconds.
pub fn eventually(condition: impl Fn() -> bool) -> bool {
    eventually_within(10, condition)
}

/// Whether the condition came true within the seconds given.
pub fn eventually_within(seconds: u64, condition: impl Fn() -> bool) -> bool {
    let until = std::time::Instant::now() + std::time::Duration::from_secs(seconds);
    while std::time::Instant::now() < until {
        if condition() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    condition()
}
