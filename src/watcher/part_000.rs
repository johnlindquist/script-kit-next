use notify::{recommended_watcher, RecursiveMode, Result as NotifyResult, Watcher};
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};
use crate::config;
/// Internal control messages for watcher threads
enum ControlMsg {
    /// Signal from notify callback with a file event
    Notify(notify::Result<notify::Event>),
    /// Signal to stop the watcher thread
    Stop,
}
/// Debounce configuration
const DEBOUNCE_MS: u64 = config::defaults::DEFAULT_WATCHER_DEBOUNCE_MS;
/// Storm threshold: if more than this many unique paths pending, collapse to FullReload
const STORM_THRESHOLD: usize = config::defaults::DEFAULT_WATCHER_STORM_THRESHOLD;
/// Initial backoff delay for supervisor restart (ms)
const INITIAL_BACKOFF_MS: u64 = config::defaults::DEFAULT_WATCHER_INITIAL_BACKOFF_MS;
/// Maximum backoff delay for supervisor restart (ms)
const MAX_BACKOFF_MS: u64 = config::defaults::DEFAULT_WATCHER_MAX_BACKOFF_MS;
/// Maximum consecutive notify errors before logging warning
const MAX_NOTIFY_ERRORS: u32 = config::defaults::DEFAULT_WATCHER_MAX_NOTIFY_ERRORS;
#[derive(Debug, Clone, Copy)]
struct WatcherSettings {
    debounce_ms: u64,
    storm_threshold: usize,
    initial_backoff_ms: u64,
    max_backoff_ms: u64,
    max_notify_errors: u32,
}
impl Default for WatcherSettings {
    fn default() -> Self {
        Self {
            debounce_ms: DEBOUNCE_MS,
            storm_threshold: STORM_THRESHOLD,
            initial_backoff_ms: INITIAL_BACKOFF_MS,
            max_backoff_ms: MAX_BACKOFF_MS,
            max_notify_errors: MAX_NOTIFY_ERRORS,
        }
    }
}
fn load_watcher_settings() -> WatcherSettings {
    let watcher = config::load_config().get_watcher();
    WatcherSettings {
        debounce_ms: watcher.debounce_ms,
        storm_threshold: watcher.storm_threshold.max(1),
        initial_backoff_ms: watcher.initial_backoff_ms.max(1),
        max_backoff_ms: watcher
            .max_backoff_ms
            .max(watcher.initial_backoff_ms.max(1)),
        max_notify_errors: watcher.max_notify_errors.max(1),
    }
}
/// Check if an event kind is relevant (not just Access events)
fn is_relevant_event_kind(kind: &notify::EventKind) -> bool {
    !matches!(kind, notify::EventKind::Access(_))
}
/// Compute exponential backoff delay, capped at MAX_BACKOFF_MS
fn compute_backoff(attempt: u32) -> Duration {
    let delay_ms = INITIAL_BACKOFF_MS.saturating_mul(2u64.saturating_pow(attempt));
    Duration::from_millis(delay_ms.min(MAX_BACKOFF_MS))
}
/// Sleep with interruptible checks against a stop flag
/// Returns true if sleep completed, false if stop was signaled
fn interruptible_sleep(duration: Duration, stop_flag: &std::sync::atomic::AtomicBool) -> bool {
    let check_interval = Duration::from_millis(100);
    let mut remaining = duration;

    while remaining > Duration::ZERO {
        if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
            return false;
        }
        let sleep_time = remaining.min(check_interval);
        thread::sleep(sleep_time);
        remaining = remaining.saturating_sub(sleep_time);
    }
    true
}
/// Event emitted when config needs to be reloaded
#[derive(Debug, Clone)]
pub enum ConfigReloadEvent {
    Reload,
}
/// Event emitted when theme needs to be reloaded
#[derive(Debug, Clone)]
pub enum ThemeReloadEvent {
    Reload,
}
/// Event emitted when scripts need to be reloaded
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptReloadEvent {
    /// A specific file was modified
    FileChanged(PathBuf),
    /// A new file was created
    FileCreated(PathBuf),
    /// A file was deleted
    FileDeleted(PathBuf),
    /// Fallback for complex events (e.g., bulk changes, renames)
    FullReload,
}
/// Watches ~/.scriptkit/kit/config.ts for changes and emits reload events
///
/// Uses trailing-edge debounce: each new event resets the deadline.
/// Handles atomic saves (rename/remove operations).
/// Properly shuts down via Stop control message.
/// Includes supervisor restart with exponential backoff on transient errors.
pub struct ConfigWatcher {
    tx: Option<Sender<ConfigReloadEvent>>,
    stop_flag: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    watcher_thread: Option<thread::JoinHandle<()>>,
}
impl ConfigWatcher {
    /// Create a new ConfigWatcher
    ///
    /// Returns a tuple of (watcher, receiver) where receiver will emit ConfigReloadEvent
    /// when the config file changes.
    pub fn new() -> (Self, Receiver<ConfigReloadEvent>) {
        let (tx, rx) = channel();
        let watcher = ConfigWatcher {
            tx: Some(tx),
            stop_flag: None,
            watcher_thread: None,
        };
        (watcher, rx)
    }

    /// Start watching the config file for changes
    ///
    /// This spawns a background thread that watches ~/.scriptkit/kit/config.ts and sends
    /// reload events through the receiver when changes are detected.
    /// On transient errors, the watcher will retry with exponential backoff.
    pub fn start(&mut self) -> NotifyResult<()> {
        let tx = self
            .tx
            .take()
            .ok_or_else(|| std::io::Error::other("watcher already started"))?;

        let stop_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let thread_stop_flag = stop_flag.clone();
        self.stop_flag = Some(stop_flag);

        let target_path = PathBuf::from(shellexpand::tilde("~/.scriptkit/kit/config.ts").as_ref());

        let thread_handle = thread::spawn(move || {
            Self::supervisor_loop(target_path, tx, thread_stop_flag);
        });

        self.watcher_thread = Some(thread_handle);
        Ok(())
    }

    /// Supervisor loop that restarts the watcher on failures with exponential backoff
    fn supervisor_loop(
        target_path: PathBuf,
        out_tx: Sender<ConfigReloadEvent>,
        stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) {
        let mut attempt: u32 = 0;

        loop {
            if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                info!(watcher = "config", "Config watcher supervisor stopping");
                break;
            }

            // Create channels for this watch attempt
            let (control_tx, control_rx) = channel::<ControlMsg>();

            match Self::watch_loop(
                target_path.clone(),
                out_tx.clone(),
                control_rx,
                control_tx,
                stop_flag.clone(),
            ) {
                Ok(()) => {
                    // Normal shutdown (via stop flag)
                    info!(watcher = "config", "Config watcher completed normally");
                    break;
                }
                Err(e) => {
                    if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                        break;
                    }

                    let backoff = compute_backoff(attempt);
                    warn!(
                        error = %e,
                        watcher = "config",
                        attempt = attempt,
                        backoff_ms = backoff.as_millis(),
                        "Config watcher error, retrying with backoff"
                    );

                    if !interruptible_sleep(backoff, &stop_flag) {
                        break;
                    }
                    attempt = attempt.saturating_add(1);
                }
            }
        }

        info!(
            watcher = "config",
            "Config watcher supervisor shutting down"
        );
    }

    /// Internal watch loop running in background thread
    fn watch_loop(
        target_path: PathBuf,
        out_tx: Sender<ConfigReloadEvent>,
        control_rx: Receiver<ControlMsg>,
        callback_tx: Sender<ControlMsg>,
        stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> NotifyResult<()> {
        let target_name: OsString = target_path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new(""))
            .to_owned();

        let watch_path = target_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));

        // Create the watcher with a callback that forwards to control channel
        let mut watcher: Box<dyn Watcher> = Box::new(recommended_watcher(
            move |res: notify::Result<notify::Event>| {
                let _ = callback_tx.send(ControlMsg::Notify(res));
            },
        )?);

        watcher.watch(watch_path, RecursiveMode::NonRecursive)?;

        info!(
            path = %watch_path.display(),
            target = ?target_name,
            "Config watcher started"
        );

        let mut consecutive_errors: u32 = 0;

        let debounce = Duration::from_millis(DEBOUNCE_MS);
        let mut deadline: Option<Instant> = None;

        loop {
            // Check stop flag before blocking
            if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }

            // Use a timeout even when no deadline to periodically check stop flag
            let timeout = deadline
                .map(|dl| dl.saturating_duration_since(Instant::now()))
                .unwrap_or(Duration::from_millis(500));

            let msg = match control_rx.recv_timeout(timeout) {
                Ok(m) => Some(m),
                Err(RecvTimeoutError::Timeout) => {
                    if let Some(dl) = deadline {
                        if Instant::now() >= dl {
                            // Quiet period ended => emit reload
                            debug!(file = ?target_name, "Config debounce complete, emitting reload");
                            let _ = out_tx.send(ConfigReloadEvent::Reload);
                            info!(file = ?target_name, "Config file changed, emitting reload event");
                            deadline = None;
                        }
                    }
                    continue;
                }
                Err(RecvTimeoutError::Disconnected) => break,
            };

            let Some(msg) = msg else { continue };

            match msg {
                ControlMsg::Stop => {
                    info!(watcher = "config", "Config watcher received stop signal");
                    break;
                }

                ControlMsg::Notify(Err(e)) => {
                    consecutive_errors = consecutive_errors.saturating_add(1);
                    warn!(
                        error = %e,
                        watcher = "config",
                        consecutive_errors = consecutive_errors,
                        "notify delivered error"
                    );

                    // If too many consecutive errors, return Err to trigger supervisor restart
                    if consecutive_errors >= MAX_NOTIFY_ERRORS {
                        warn!(
                            watcher = "config",
                            consecutive_errors = consecutive_errors,
                            "Too many consecutive errors, triggering restart"
                        );
                        return Err(notify::Error::generic("Too many consecutive notify errors"));
                    }
                }

                ControlMsg::Notify(Ok(event)) => {
                    // Reset error counter on successful event
                    consecutive_errors = 0;

                    // Filter: does this event mention the target filename?
                    let touches_target = event.paths.iter().any(|p| {
                        p.file_name()
                            .map(|n| n == target_name.as_os_str())
                            .unwrap_or(false)
                    });

                    // Treat everything except Access as potentially relevant
                    // This covers atomic saves (remove/rename) too
                    let relevant_kind = is_relevant_event_kind(&event.kind);

                    if touches_target && relevant_kind {
                        // Trailing-edge debounce: reset deadline on every hit
                        debug!(
                            file = ?target_name,
                            event_kind = ?event.kind,
                            "Config change detected, resetting debounce"
                        );
                        deadline = Some(Instant::now() + debounce);
                    }
                }
            }
        }

        info!(watcher = "config", "Config watcher shutting down");
        Ok(())
    }
}
impl Drop for ConfigWatcher {
    fn drop(&mut self) {
        // Signal stop via atomic flag
        if let Some(flag) = self.stop_flag.take() {
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        // Now join - the thread will exit because stop flag is set
        if let Some(handle) = self.watcher_thread.take() {
            let _ = handle.join();
        }
    }
}
/// Watches ~/.scriptkit/kit/theme.json for changes and emits reload events
///
/// Uses trailing-edge debounce: each new event resets the deadline.
/// Handles atomic saves (rename/remove operations).
/// Properly shuts down via Stop control message.
/// Includes supervisor restart with exponential backoff on transient errors.
pub struct ThemeWatcher {
    tx: Option<Sender<ThemeReloadEvent>>,
    stop_flag: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    watcher_thread: Option<thread::JoinHandle<()>>,
}
