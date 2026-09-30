//! The watcher itself: watches the sources, hands ready batches to be processed and follows the configuration file.

use std::collections::VecDeque;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, SystemTime};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::cli::Options;
use crate::config::Config;
use crate::scan::Scan;
use crate::service::{Renames, process_batch};
use crate::watcher::{Machine, Ready, Translated, pass_changes, rewrites, translate};

/// How long a change to the configuration file is awaited for more changes before it is read.
const RELOAD_DELAY: Duration = Duration::from_secs(1);

/// The longest the runner waits before looking at the stop flag again.
const POLL: Duration = Duration::from_secs(1);

/// How many notifications one turn of the loop takes, and how many entries a scan looks at in one. A
/// turn stays short this way, so a stop, a notification and a batch each get their turn while a long
/// scan or a long run of notifications is still going on.
const NOTIFICATIONS_PER_TURN: usize = 256;
const SCANNED_PER_TURN: usize = 256;

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

/// A configuration in force: its watches with their machines, the watcher that feeds them, and the work
/// still to be done a turn at a time. A scan and a batch are kept with the place of their watch.
struct Session {
    config: Config,
    machines: Vec<Machine>,
    renames: Vec<Renames>,
    watcher: RecommendedWatcher,
    scans: VecDeque<(usize, Scan)>,
    ready: VecDeque<(usize, Ready)>,
}

impl Session {
    fn start(
        config: Config,
        config_path: &Path,
        sender: &Sender<Notification>,
    ) -> Result<Session, RunError> {
        config
            .check_paths(config_path, real_path)
            .map_err(|error| RunError::Config(error.to_string()))?;
        for warning in config.warnings() {
            eprintln!("[warn] {warning}");
        }
        let mut watcher = RecommendedWatcher::new(
            pass_changes(sender.clone()),
            notify::Config::default().with_follow_symlinks(false),
        )
        .map_err(|error| RunError::Watch(error.to_string()))?;
        let mut machines = Vec::new();
        let mut renames = Vec::new();
        let mut scans = VecDeque::new();
        for (index, watch) in config.watches().iter().enumerate() {
            watcher
                .watch(&watch.source, RecursiveMode::Recursive)
                .map_err(|error| RunError::Watch(format!("{}: {error}", watch.source.display())))?;
            machines.push(Machine::new(watch));
            renames.push(Renames::new());
            scans.push_back((index, Scan::new(&watch.source, &watch.source)));
        }
        if let Some(folder) = config_path.parent() {
            watcher
                .watch(folder, RecursiveMode::NonRecursive)
                .map_err(|error| RunError::Watch(format!("{}: {error}", folder.display())))?;
        }
        Ok(Session {
            config,
            machines,
            renames,
            watcher,
            scans,
            ready: VecDeque::new(),
        })
    }

    fn observe(&mut self, notification: &notify::Event, now: SystemTime) {
        let is_folder = |path: &Path| fs::symlink_metadata(path).is_ok_and(|meta| meta.is_dir());
        for (index, watch) in self.config.watches().iter().enumerate() {
            for translated in translate(notification, &watch.source, is_folder) {
                match translated {
                    Translated::Event(event) => self.machines[index].observe(event, now),
                    Translated::Scan(folder) => {
                        confirm_watched(&mut self.watcher, &watch.source, &folder);
                        self.scans
                            .push_back((index, Scan::new(&watch.source, &folder)));
                    }
                }
            }
        }
    }

    /// Goes on with the scan that is next for one turn.
    fn scan(&mut self, now: SystemTime) {
        let Some((index, scan)) = self.scans.front_mut() else {
            return;
        };
        let found = scan.look_at(SCANNED_PER_TURN);
        let finished = scan.is_finished();
        let machine = &mut self.machines[*index];
        for event in found {
            machine.observe(event, now);
        }
        if finished {
            self.scans.pop_front();
        }
    }

    /// Processes one batch that is ready. A watch with a scan still going hands nothing over, since a
    /// scan finds the files of one batch over several turns.
    fn process_one(&mut self, now: SystemTime) {
        if self.ready.is_empty() {
            for (index, machine) in self.machines.iter_mut().enumerate() {
                if !self.scans.iter().any(|(scanned, _)| *scanned == index) {
                    let batches = machine.ready(now).into_iter();
                    self.ready.extend(batches.map(|batch| (index, batch)));
                }
            }
        }
        if let Some((index, Ready { unit, files })) = self.ready.pop_front() {
            let watch = &self.config.watches()[index];
            process_batch(watch, &unit, &files, &mut self.renames[index]);
        }
    }

    fn has_work(&self) -> bool {
        !self.scans.is_empty() || !self.ready.is_empty()
    }

    fn next_deadline(&self) -> Option<SystemTime> {
        self.machines
            .iter()
            .filter_map(Machine::next_deadline)
            .min()
    }
}

/// Has a folder that appeared watched before it is scanned. The notification of a folder arrives before
/// the watch on it is in place, so a file created in between would be neither found by the scan nor
/// reported; asking for the watch waits until it is there. A folder that is gone again needs none.
fn confirm_watched(watcher: &mut RecommendedWatcher, source: &Path, folder: &Path) {
    if let Err(error) = watcher.watch(folder, RecursiveMode::Recursive)
        && !matches!(error.kind, notify::ErrorKind::PathNotFound)
    {
        let name = folder.strip_prefix(source).unwrap_or(folder);
        eprintln!("[warn] {} is not watched: {error}", name.display());
    }
}

/// Watches the sources of a configuration until told to stop, reading the configuration file again when
/// it is rewritten or when `reload` is set, which is for a file no notification reaches, such as one
/// mounted into a container on its own.
///
/// A configuration that is not valid at start is an error; one that stops being valid later is reported
/// and the running one is kept.
pub fn run(options: &Options, stop: &AtomicBool, reload: &AtomicBool) -> Result<(), RunError> {
    let config_path = std::path::absolute(&options.config)
        .map_err(|error| RunError::Config(format!("{}: {error}", options.config.display())))?;
    let (sender, receiver) = mpsc::channel();
    let mut session = Session::start(load(&config_path)?, &config_path, &sender)?;
    let mut reload_at: Option<SystemTime> = None;

    while !stop.load(Ordering::SeqCst) {
        let timeout = wait(&session, reload_at, SystemTime::now());
        for notification in receive(&receiver, timeout)? {
            match notification {
                Ok(notification) => {
                    if rewrites(&notification, &config_path) {
                        reload_at = Some(SystemTime::now() + RELOAD_DELAY);
                    }
                    session.observe(&notification, SystemTime::now());
                }
                Err(error) => eprintln!("[warn] the filesystem reported: {error}"),
            }
        }

        let now = SystemTime::now();
        if reload.swap(false, Ordering::SeqCst) {
            reload_at = Some(now);
        }
        session.scan(now);
        session.process_one(now);
        if reload_at.is_some_and(|due| due <= now) {
            reload_at = None;
            session = read_again(session, &config_path, &sender);
        }
    }
    Ok(())
}

/// Where a path really is, past every link on its way. What does not exist yet, such as a target made on
/// the first move, is placed under the real location of the part that does.
fn real_path(path: &Path) -> PathBuf {
    match fs::canonicalize(path) {
        Ok(real) => real,
        Err(_) => match (path.parent(), path.file_name()) {
            (Some(parent), Some(name)) => real_path(parent).join(name),
            _ => path.to_path_buf(),
        },
    }
}

fn load(path: &Path) -> Result<Config, RunError> {
    let text = fs::read_to_string(path)
        .map_err(|error| RunError::Config(format!("{}: {error}", path.display())))?;
    Config::parse(&text).map_err(|error| RunError::Config(error.to_string()))
}

/// The configuration read again, or the one running when the new one cannot be used.
fn read_again(session: Session, config_path: &Path, sender: &Sender<Notification>) -> Session {
    let started = load(config_path).and_then(|config| Session::start(config, config_path, sender));
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

/// How long to wait for the next notification: not at all while there is work to go on with, and
/// otherwise until the earliest thing due and never more than a poll.
fn wait(session: &Session, reload_at: Option<SystemTime>, now: SystemTime) -> Duration {
    if session.has_work() {
        return Duration::ZERO;
    }
    [session.next_deadline(), reload_at]
        .into_iter()
        .flatten()
        .min()
        .map(|due| due.duration_since(now).unwrap_or(Duration::ZERO))
        .map_or(POLL, |until| until.min(POLL))
}

/// The notifications of one turn: the first is waited for, and those already there follow it.
fn receive(
    receiver: &Receiver<Notification>,
    timeout: Duration,
) -> Result<Vec<Notification>, RunError> {
    match receiver.recv_timeout(timeout) {
        Ok(first) => Ok(std::iter::once(first)
            .chain(receiver.try_iter())
            .take(NOTIFICATIONS_PER_TURN)
            .collect()),
        Err(RecvTimeoutError::Timeout) => Ok(Vec::new()),
        Err(RecvTimeoutError::Disconnected) => {
            Err(RunError::Watch("the watcher stopped".to_string()))
        }
    }
}
