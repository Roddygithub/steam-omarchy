//! One worker owns ordering; notifications are hints, every update rereads current state.
use crate::theme::{self, Theme};
use notify::{RecursiveMode, Watcher};
use std::{
    collections::BTreeSet,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    time::{Duration, Instant},
};
pub static SNAPSHOT: Mutex<Option<Theme>> = Mutex::new(None);
pub fn snapshot() -> Theme {
    SNAPSHOT
        .lock()
        .unwrap()
        .clone()
        .unwrap_or_else(|| theme::decode("").unwrap())
}
fn roots(paths: &[(PathBuf, &str)]) -> BTreeSet<PathBuf> {
    let mut roots = BTreeSet::new();
    for (p, _) in paths {
        for ancestor in p.ancestors().skip(1).take(3) {
            if let Some(existing) = ancestor.ancestors().find(|p| p.is_dir()) {
                roots.insert(existing.to_path_buf());
            }
        }
    }
    roots
}
pub struct Worker {
    stop: Arc<AtomicBool>,
    wake: mpsc::SyncSender<()>,
    done: mpsc::Receiver<()>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Worker {
    pub fn start(publish: impl FnMut(Theme) + Send + 'static) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let (wake, rx) = mpsc::sync_channel(1);
        let (finished, done) = mpsc::channel();
        let flag = stop.clone();
        let tx = wake.clone();
        let thread = std::thread::spawn(move || {
            run(publish, flag, tx, rx);
            let _ = finished.send(());
        });
        Self {
            stop,
            wake,
            done,
            thread: Some(thread),
        }
    }
    pub fn request_stop(&self) {
        self.stop.store(true, Ordering::Release);
        // A full queue already wakes recv; the flag also interrupts debounce.
        let _ = self.wake.try_send(());
    }
    pub fn shutdown(&mut self, budget: Duration) -> bool {
        self.request_stop();
        if self.thread.is_none() {
            return true;
        }
        if self.done.recv_timeout(budget).is_ok() {
            return self.thread.take().unwrap().join().is_ok();
        }
        false
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.request_stop();
    }
}
fn run(
    mut publish: impl FnMut(Theme),
    stop: Arc<AtomicBool>,
    tx: mpsc::SyncSender<()>,
    rx: mpsc::Receiver<()>,
) {
    let mut watcher = None;
    let mut watched: BTreeSet<PathBuf> = BTreeSet::new();
    let mut generation = 0;
    while !stop.load(Ordering::Acquire) {
        if watcher.is_none() {
            let sender = tx.clone();
            watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                if event
                    .as_ref()
                    .is_ok_and(|e| matches!(e.kind, notify::EventKind::Access(_)))
                {
                    return;
                }
                if let Err(e) = event {
                    eprintln!("Theme watcher: {e}; resynchronising");
                }
                let _ = sender.try_send(());
            })
            .map_err(|e| eprintln!("Theme watcher unavailable: {e}"))
            .ok();
        }
        let desired = roots(&theme::paths());
        let mut failed = watcher.is_none();
        if let Some(w) = watcher.as_mut() {
            // Re-register path watches: an atomic rename can preserve the path but replace its inode.
            for p in &watched {
                let _ = w.unwatch(p);
            }
            for p in &desired {
                if w.watch(p, RecursiveMode::NonRecursive).is_err() {
                    failed = true;
                }
            }
            if !failed {
                watched = desired;
            }
        }
        let mut next = theme::read();
        if failed {
            next.warnings
                .push("Surveillance indisponible : resynchronisation toutes les 30 s".into());
        }
        let previous = SNAPSHOT.lock().unwrap().clone();
        next.generation = previous.as_ref().map_or(0, |t| t.generation);
        if previous.as_ref() != Some(&next) {
            generation += 1;
            next.generation = generation;
            *SNAPSHOT.lock().unwrap() = Some(next.clone());
            publish(next);
        }
        // Periodic resync also covers suspend/resume and lost/overflow events.
        match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(()) => {
                let start = Instant::now();
                while !stop.load(Ordering::Acquire) && start.elapsed() < Duration::from_millis(500)
                {
                    if rx.recv_timeout(Duration::from_millis(80)).is_err() {
                        break;
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                break;
            }
        }
    }
}
