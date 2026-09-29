//! The watcher itself: watches the sources, hands ready batches to be processed and follows the configuration file.

use std::fmt;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, SystemTime};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::cli::Options;
use crate::config::Config;
use crate::scan::scan_folder;
use crate::service::process_batch;
use crate::watcher::{Machine, Ready, Translated, translate};

/// How long a change to the configuration file is awaited for more changes before it is read.
const RELOAD_DELAY: Duration = Duration::from_secs(1);

/// The longest the runner waits before looking at the stop flag again.
const POLL: Duration = Duration::from_secs(1);

/// Why the watcher could not start or stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    Config(String),
    Watch(String),
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunError::Config(message) => write!(f, "configuration: {message}"),
            RunError::Watch(message) => write!(f, "watching: {message}"),
        }
    }
}

impl std::error::Error for RunError {}

type Notification = notify::Result<notify::Event>;

/// A configuration in force: its watches with their machines, and the watcher that feeds them.
struct Session {
    config: Config,
    machines: Vec<Machine>,
    _watcher: RecommendedWatcher,
}

impl Session {
    fn start(
        config: Config,
        config_path: &Path,
        sender: &Sender<Notification>,
        now: SystemTime,
    ) -> Result<Session, RunError> {
        let mut watcher = RecommendedWatcher::new(
            sender.clone(),
            notify::Config::default().with_follow_symlinks(false),
        )
        .map_err(|error| RunError::Watch(error.to_string()))?;
        let mut machines = Vec::new();
        for watch in config.watches() {
            watcher
                .watch(&watch.source, RecursiveMode::Recursive)
                .map_err(|error| RunError::Watch(format!("{}: {error}", watch.source.display())))?;
            let mut machine = Machine::new(watch);
            for event in scan_folder(&watch.source, &watch.source) {
                machine.observe(event, now);
            }
            machines.push(machine);
        }
        if let Some(folder) = config_path.parent()
            && !config
                .watches()
                .iter()
                .any(|watch| folder.starts_with(&watch.source))
        {
            watcher
                .watch(folder, RecursiveMode::NonRecursive)
                .map_err(|error| RunError::Watch(format!("{}: {error}", folder.display())))?;
        }
        Ok(Session {
            config,
            machines,
            _watcher: watcher,
        })
    }

    fn observe(&mut self, notification: &notify::Event, now: SystemTime) {
        let is_folder = |path: &Path| fs::symlink_metadata(path).is_ok_and(|meta| meta.is_dir());
        for (watch, machine) in self.config.watches().iter().zip(&mut self.machines) {
            for translated in translate(notification, &watch.source, is_folder) {
                match translated {
                    Translated::Event(event) => machine.observe(event, now),
                    Translated::Scan(folder) => {
                        for event in scan_folder(&watch.source, &folder) {
                            machine.observe(event, now);
                        }
                    }
                }
            }
        }
    }

    fn process_ready(&mut self, now: SystemTime) {
        for (watch, machine) in self.config.watches().iter().zip(&mut self.machines) {
            for ready in machine.ready(now) {
                match ready {
                    Ready::Batch { unit, files } | Ready::Skipped { unit, files } => {
                        process_batch(watch, &unit, &files);
                    }
                }
            }
        }
    }

    fn next_deadline(&self) -> Option<SystemTime> {
        self.machines
            .iter()
            .filter_map(Machine::next_deadline)
            .min()
    }
}

/// Watches the sources of a configuration until told to stop, following the configuration file as it changes.
///
/// A configuration that is not valid at start is an error; one that stops being valid later is reported
/// and the running one is kept.
pub fn run(options: &Options, stop: &AtomicBool) -> Result<(), RunError> {
    let config_path = std::path::absolute(&options.config)
        .map_err(|error| RunError::Config(format!("{}: {error}", options.config.display())))?;
    let (sender, receiver) = mpsc::channel();
    let mut session = Session::start(
        load(&config_path)?,
        &config_path,
        &sender,
        SystemTime::now(),
    )?;
    let mut reload_at: Option<SystemTime> = None;

    while !stop.load(Ordering::SeqCst) {
        let timeout = wait(&session, reload_at, SystemTime::now());
        match receive(&receiver, timeout)? {
            Some(Ok(notification)) => {
                if notification.paths.iter().any(|path| path == &config_path) {
                    reload_at = Some(SystemTime::now() + RELOAD_DELAY);
                }
                session.observe(&notification, SystemTime::now());
            }
            Some(Err(error)) => eprintln!("[warn] the filesystem reported: {error}"),
            None => {}
        }

        let now = SystemTime::now();
        session.process_ready(now);
        if reload_at.is_some_and(|due| due <= now) {
            reload_at = None;
            session = reload(session, &config_path, &sender, now);
        }
    }
    Ok(())
}

fn load(path: &Path) -> Result<Config, RunError> {
    let text = fs::read_to_string(path)
        .map_err(|error| RunError::Config(format!("{}: {error}", path.display())))?;
    Config::parse(&text).map_err(|error| RunError::Config(error.to_string()))
}

/// The configuration read again, or the one running when the new one cannot be used.
fn reload(
    session: Session,
    config_path: &Path,
    sender: &Sender<Notification>,
    now: SystemTime,
) -> Session {
    let started =
        load(config_path).and_then(|config| Session::start(config, config_path, sender, now));
    match started {
        Ok(fresh) => {
            eprintln!("[info] the configuration was read again");
            fresh
        }
        Err(error) => {
            eprintln!("[warn] the configuration is kept as it was: {error}");
            session
        }
    }
}

/// How long to wait for the next notification: until the earliest thing due, and never more than a poll.
fn wait(session: &Session, reload_at: Option<SystemTime>, now: SystemTime) -> Duration {
    [session.next_deadline(), reload_at]
        .into_iter()
        .flatten()
        .min()
        .map(|due| due.duration_since(now).unwrap_or(Duration::ZERO))
        .map_or(POLL, |until| until.min(POLL))
}

fn receive(
    receiver: &Receiver<Notification>,
    timeout: Duration,
) -> Result<Option<Notification>, RunError> {
    match receiver.recv_timeout(timeout) {
        Ok(notification) => Ok(Some(notification)),
        Err(RecvTimeoutError::Timeout) => Ok(None),
        Err(RecvTimeoutError::Disconnected) => {
            Err(RunError::Watch("the watcher stopped".to_string()))
        }
    }
}
