//! The watcher itself: watches the sources, hands ready batches to be processed and follows the configuration file.

use std::collections::VecDeque;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::cli::Options;
use crate::config::Config;
use crate::filesystem::FsTree;
use crate::scan::Scan;
use crate::service::{Processed, Renames, What, process_batch};
use crate::watcher::{Machine, Queue, Ready, Translated, rewrites, translate};

/// How long a change to the configuration file is awaited for more changes before it is read.
const RELOAD_DELAY: Duration = Duration::from_secs(1);

/// The longest the runner waits before looking at the stop flag again.
const POLL: Duration = Duration::from_secs(1);

/// How many notifications one turn of the loop takes, and how many entries a scan looks at in one. A
/// turn stays short this way, so a stop, a notification and a batch each get their turn while a long
/// scan or a long run of notifications is still going on.
const NOTIFICATIONS_PER_TURN: usize = 256;
const SCANNED_PER_TURN: usize = 256;

/// How many notifications may wait for the runner. A batch is processed in one turn, and every file
/// it moves out of the source is reported, so a batch of more files than this loses notifications and
/// is followed by a scan.
const QUEUE_CAPACITY: usize = 8192;

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

/// A configuration as its file held it, kept to tell whether the file has changed since.
struct Loaded {
    text: String,
    config: Config,
}

/// A configuration in force: its watches with their machines, the watcher that feeds them through a
/// queue of its own, and the work still to be done a turn at a time. A scan and a batch are kept with
/// the place of their watch.
///
/// The queue belongs to the session, so that what the watcher of an earlier session still reports
/// never reaches the machines of this one: they know only what this session's scans and watches told.
struct Session {
    loaded: Loaded,
    queue: Queue,
    machines: Vec<Machine>,
    renames: Vec<Renames>,
    watcher: RecommendedWatcher,
    scans: VecDeque<(usize, Scan)>,
    /// How many scans each watch still has going.
    scanning: Vec<usize>,
    /// How many of the scans the program starts with are unfinished. While any is, nothing is moved and
    /// a folder that cannot be watched stops the program; a later session starts with none.
    starting: usize,
    ready: VecDeque<(usize, Ready)>,
}

/// Whether a session is the one the program starts with, or one that takes over while it runs.
#[derive(Clone, Copy, PartialEq)]
enum Start {
    First,
    Later,
}

impl Session {
    fn start(loaded: Loaded, config_path: &Path, start: Start) -> Result<Session, RunError> {
        let config = &loaded.config;
        config
            .check_paths(config_path, real_path)
            .map_err(|error| RunError::Config(error.to_string()))?;
        for warning in config.warnings() {
            eprintln!("[warn] {warning}");
        }
        let queue = Queue::new(QUEUE_CAPACITY);
        let mut watcher = RecommendedWatcher::new(
            queue.handler(),
            notify::Config::default().with_follow_symlinks(false),
        )
        .map_err(|error| RunError::Watch(error.to_string()))?;
        let mut machines = Vec::new();
        let mut renames = Vec::new();
        let mut scans = VecDeque::new();
        for (index, watch) in config.watches().iter().enumerate() {
            watcher
                .watch(&watch.source, RecursiveMode::NonRecursive)
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
            queue,
            scanning: vec![1; machines.len()],
            machines,
            renames,
            watcher,
            starting: if start == Start::First {
                scans.len()
            } else {
                0
            },
            scans,
            ready: VecDeque::new(),
            loaded,
        })
    }

    fn observe(&mut self, notification: &notify::Event, now: SystemTime) {
        let is_folder = |path: &Path| fs::symlink_metadata(path).is_ok_and(|meta| meta.is_dir());
        for (index, watch) in self.loaded.config.watches().iter().enumerate() {
            for translated in translate(notification, &watch.source, is_folder) {
                match translated {
                    Translated::Event(event) => self.machines[index].observe(event, now),
                    Translated::Scan(folder) => {
                        self.scanning[index] += 1;
                        self.scans
                            .push_back((index, Scan::new(&watch.source, &folder)));
                    }
                }
            }
        }
    }

    /// Goes on with the scan that is next for one turn, watching each folder before it is read.
    ///
    /// A folder that cannot be watched stops a program that is starting, as a source that cannot be
    /// watched does. Once the program runs, such a folder is named and the watcher goes on.
    fn scan(&mut self, now: SystemTime) -> Result<(), RunError> {
        let Some((index, scan)) = self.scans.front_mut() else {
            return Ok(());
        };
        let source = &self.loaded.config.watches()[*index].source;
        let watcher = &mut self.watcher;
        let mut unwatched = Vec::new();
        let found = scan.look_at(SCANNED_PER_TURN, |folder| {
            if let Err(error) = watcher.watch(folder, RecursiveMode::NonRecursive)
                && !matches!(error.kind, notify::ErrorKind::PathNotFound)
            {
                unwatched.push((folder.to_path_buf(), error));
            }
        });
        if self.starting > 0
            && let Some((folder, error)) = unwatched.first()
        {
            return Err(RunError::Watch(format!("{}: {error}", folder.display())));
        }
        for (folder, error) in unwatched {
            let name = folder.strip_prefix(source).unwrap_or(&folder);
            eprintln!("[warn] {} is not watched: {error}", name.display());
        }
        let finished = scan.is_finished();
        let machine = &mut self.machines[*index];
        for event in found {
            machine.observe(event, now);
        }
        if finished {
            self.scanning[*index] -= 1;
            self.scans.pop_front();
            self.starting = self.starting.saturating_sub(1);
        }
        Ok(())
    }

    /// Processes one batch that is ready. A watch with a scan still going hands nothing over, since a
    /// scan finds the files of one batch over several turns, and nothing is processed until the scans
    /// the program starts with are done, since any of them may still stop it.
    fn process_one(&mut self, now: SystemTime) {
        if self.starting > 0 {
            return;
        }
        if self.ready.is_empty() {
            for (index, machine) in self.machines.iter_mut().enumerate() {
                if self.scanning[index] == 0 {
                    let batches = machine.ready(now).into_iter();
                    self.ready.extend(batches.map(|batch| (index, batch)));
                }
            }
        }
        if let Some((index, Ready { unit, files })) = self.ready.pop_front() {
            let watch = &self.loaded.config.watches()[index];
            process_batch(&FsTree, watch, &unit, &files, &mut self.renames[index])
                .iter()
                .for_each(report);
        }
    }

    /// Starts over with the configuration in force: what the session waited for is dropped, and the
    /// sources are watched and scanned again. It is how lost notifications are made up for, since the
    /// scan finds whatever they would have told.
    ///
    /// The watcher is let go first, so that its watches are not counted beside the ones asked for
    /// again for longer than its own thread takes to give them up. The renames in place stay counted,
    /// so that starting over does not let a pipeline that names its own result again go on for ever.
    fn start_over(self, config_path: &Path) -> Result<Session, RunError> {
        let start = self.successor();
        let Session {
            loaded,
            watcher,
            queue,
            renames,
            ..
        } = self;
        drop(watcher);
        drop(queue);
        let mut session = Session::start(loaded, config_path, start)?;
        session.renames = renames;
        Ok(session)
    }

    /// What a session that takes over from this one is: still the one the program starts with while
    /// this one has not finished starting, so that taking over early gives up nothing a start promises.
    fn successor(&self) -> Start {
        if self.starting > 0 {
            Start::First
        } else {
            Start::Later
        }
    }

    /// Whether the configuration file no longer holds what this session read from it.
    fn is_outdated(&self, config_path: &Path) -> bool {
        fs::read_to_string(config_path).is_ok_and(|text| text != self.loaded.text)
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

/// Watches the sources of a configuration until told to stop, reading the configuration file again when
/// it is rewritten or when `reload` is set, which is for a file no notification reaches, such as one
/// mounted into a container on its own.
///
/// A configuration that is not valid at start is an error; one that stops being valid later is reported
/// and the running one is kept.
pub fn run(options: &Options, stop: &AtomicBool, reload: &AtomicBool) -> Result<(), RunError> {
    let config_path = std::path::absolute(&options.config)
        .map_err(|error| RunError::Config(format!("{}: {error}", options.config.display())))?;
    let mut session = Session::start(load(&config_path)?, &config_path, Start::First)?;
    let mut reload_at = changed_since_read(&session, &config_path, SystemTime::now());

    while !stop.load(Ordering::SeqCst) {
        let timeout = wait(&session, reload_at, SystemTime::now());
        for notification in session.queue.take(timeout, NOTIFICATIONS_PER_TURN) {
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

        // Which session goes on is settled before anything is scanned or moved, so that nothing is
        // moved by a configuration already known to be replaced.
        let now = SystemTime::now();
        let lost = session.queue.lost();
        if lost {
            eprintln!("[warn] notifications were lost, so every source is scanned again");
        }
        if reload.swap(false, Ordering::SeqCst)
            // The notification of a change to the configuration may be among the lost ones.
            || (lost && session.is_outdated(&config_path))
        {
            reload_at = Some(now);
        }
        let mut read = None;
        if reload_at.is_some_and(|due| due <= now) {
            reload_at = None;
            match read_again(&session, &config_path) {
                Ok(fresh) => {
                    eprintln!("[info] the configuration was read again");
                    read = Some(fresh);
                }
                Err(error) => eprintln!("[warn] the configuration is kept as it was: {error}"),
            }
        }
        let replaced = read.is_some() || lost;
        if let Some(fresh) = read {
            session = fresh;
        } else if lost {
            session = session.start_over(&config_path)?;
        }
        if replaced {
            reload_at = changed_since_read(&session, &config_path, now);
        }

        session.scan(now)?;
        session.process_one(now);
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

fn load(path: &Path) -> Result<Loaded, RunError> {
    let text = fs::read_to_string(path)
        .map_err(|error| RunError::Config(format!("{}: {error}", path.display())))?;
    let config = Config::parse(&text).map_err(|error| RunError::Config(error.to_string()))?;
    Ok(Loaded { text, config })
}

/// When to read the configuration again for a session that has just started: a change made after the
/// file was read and before it was watched is reported to nobody, so the file is compared instead.
fn changed_since_read(
    session: &Session,
    config_path: &Path,
    now: SystemTime,
) -> Option<SystemTime> {
    session
        .is_outdated(config_path)
        .then_some(now + RELOAD_DELAY)
}

/// A session over the configuration as its file holds it now.
fn read_again(session: &Session, config_path: &Path) -> Result<Session, RunError> {
    load(config_path).and_then(|loaded| Session::start(loaded, config_path, session.successor()))
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

/// Logs what became of one file of a batch.
fn report(entry: &Processed) {
    let origin = entry.origin.display();
    match &entry.what {
        What::Moved(to) => eprintln!("[info] {origin} -> {}", to.display()),
        What::MovedThenFailed { to, reason } => {
            eprintln!("[warn] {origin} -> {}, then failed: {reason}", to.display())
        }
        What::Previewed(to) => eprintln!("[info] {origin} would go to {}", to.display()),
        What::Unchanged => {}
        What::Unclaimed => eprintln!("[info] {origin} left: no pipeline claims it"),
        What::Skipped(reason) => eprintln!("[warn] {origin} skipped: {reason}"),
        What::Refused(reason) => eprintln!("[warn] {origin} refused: {reason}"),
        What::LeftTooLarge => eprintln!("[warn] {origin} left: the batch is too large"),
    }
}
