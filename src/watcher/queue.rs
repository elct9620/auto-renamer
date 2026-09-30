use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::time::Duration;

use super::notification::changes;

type Notification = notify::Result<notify::Event>;

/// The notifications that say something changed, waiting for the runner.
///
/// It holds a bounded number, so what waits takes no more memory than that however long the runner is
/// busy. What does not fit is lost, and so is whatever the kernel dropped from its own queue; a queue
/// that lost notifications says so, and the runner starts over from a scan, which finds what they would
/// have told.
pub struct Queue {
    sender: SyncSender<Notification>,
    receiver: Receiver<Notification>,
    lost: Arc<AtomicBool>,
}

impl Queue {
    /// Starts a queue with room for a number of notifications.
    pub fn new(capacity: usize) -> Queue {
        let (sender, receiver) = mpsc::sync_channel(capacity);
        Queue {
            sender,
            receiver,
            lost: Arc::new(AtomicBool::new(false)),
        }
    }

    /// The handler to give the filesystem watcher. It never waits for room, so the watcher is never
    /// kept from answering the runner.
    ///
    /// A file or folder that is only opened, read or has its attributes changed is reported too, by
    /// every program that looks into the source, and none of it concerns the watcher: it is dropped.
    pub fn handler(&self) -> impl FnMut(Notification) + Send + 'static {
        let sender = self.sender.clone();
        let lost = Arc::clone(&self.lost);
        move |notification| {
            let overflowed = matches!(&notification, Ok(event) if event.need_rescan());
            let kept = notification.as_ref().map_or(true, changes);
            if overflowed || (kept && sender.try_send(notification).is_err()) {
                lost.store(true, Ordering::SeqCst);
            }
        }
    }

    /// The notifications of one turn: the first is waited for, and those already there follow it, up to
    /// a limit.
    pub fn take(&self, timeout: Duration, limit: usize) -> Vec<Notification> {
        match self.receiver.recv_timeout(timeout) {
            Ok(first) => std::iter::once(first)
                .chain(self.receiver.try_iter())
                .take(limit)
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Whether notifications were lost since this was last asked. What still waits is dropped with a
    /// yes: whoever asks starts over from a scan, and the scan comes after all of it.
    pub fn lost(&self) -> bool {
        if !self.lost.load(Ordering::SeqCst) {
            return false;
        }
        self.receiver.try_iter().for_each(drop);
        self.lost.store(false, Ordering::SeqCst);
        true
    }
}
