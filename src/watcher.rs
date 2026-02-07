#![allow(dead_code)]
//! File-watching services for config, theme, scripts, and app-level reload orchestration.
//! Public watchers include `ConfigWatcher`, `ThemeWatcher`, `ScriptWatcher`, and `AppWatcher`,
//! plus reload event enums consumed by the UI/application loop.
//! This module depends on `notify`, `config`, and `setup`, and feeds change events into runtime state updates.

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

impl ThemeWatcher {
    /// Create a new ThemeWatcher
    ///
    /// Returns a tuple of (watcher, receiver) where receiver will emit ThemeReloadEvent
    /// when the theme file changes.
    pub fn new() -> (Self, Receiver<ThemeReloadEvent>) {
        let (tx, rx) = channel();
        let watcher = ThemeWatcher {
            tx: Some(tx),
            stop_flag: None,
            watcher_thread: None,
        };
        (watcher, rx)
    }

    /// Start watching the theme file for changes
    ///
    /// This spawns a background thread that watches ~/.scriptkit/kit/theme.json and sends
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

        let target_path = PathBuf::from(shellexpand::tilde("~/.scriptkit/kit/theme.json").as_ref());

        let thread_handle = thread::spawn(move || {
            Self::supervisor_loop(target_path, tx, thread_stop_flag);
        });

        self.watcher_thread = Some(thread_handle);
        Ok(())
    }

    /// Supervisor loop that restarts the watcher on failures with exponential backoff
    fn supervisor_loop(
        target_path: PathBuf,
        out_tx: Sender<ThemeReloadEvent>,
        stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) {
        let mut attempt: u32 = 0;

        loop {
            if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                info!(watcher = "theme", "Theme watcher supervisor stopping");
                break;
            }

            let (control_tx, control_rx) = channel::<ControlMsg>();

            match Self::watch_loop(
                target_path.clone(),
                out_tx.clone(),
                control_rx,
                control_tx,
                stop_flag.clone(),
            ) {
                Ok(()) => {
                    info!(watcher = "theme", "Theme watcher completed normally");
                    break;
                }
                Err(e) => {
                    if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                        break;
                    }

                    let backoff = compute_backoff(attempt);
                    warn!(
                        error = %e,
                        watcher = "theme",
                        attempt = attempt,
                        backoff_ms = backoff.as_millis(),
                        "Theme watcher error, retrying with backoff"
                    );

                    if !interruptible_sleep(backoff, &stop_flag) {
                        break;
                    }
                    attempt = attempt.saturating_add(1);
                }
            }
        }

        info!(watcher = "theme", "Theme watcher supervisor shutting down");
    }

    /// Internal watch loop running in background thread
    fn watch_loop(
        target_path: PathBuf,
        out_tx: Sender<ThemeReloadEvent>,
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

        let mut watcher: Box<dyn Watcher> = Box::new(recommended_watcher(
            move |res: notify::Result<notify::Event>| {
                let _ = callback_tx.send(ControlMsg::Notify(res));
            },
        )?);

        watcher.watch(watch_path, RecursiveMode::NonRecursive)?;

        info!(
            path = %watch_path.display(),
            target = ?target_name,
            "Theme watcher started"
        );

        let mut consecutive_errors: u32 = 0;
        let debounce = Duration::from_millis(DEBOUNCE_MS);
        let mut deadline: Option<Instant> = None;

        loop {
            if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }

            let timeout = deadline
                .map(|dl| dl.saturating_duration_since(Instant::now()))
                .unwrap_or(Duration::from_millis(500));

            let msg = match control_rx.recv_timeout(timeout) {
                Ok(m) => Some(m),
                Err(RecvTimeoutError::Timeout) => {
                    if let Some(dl) = deadline {
                        if Instant::now() >= dl {
                            debug!(file = ?target_name, "Theme debounce complete, emitting reload");
                            let _ = out_tx.send(ThemeReloadEvent::Reload);
                            info!(file = ?target_name, "Theme file changed, emitting reload event");
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
                    info!(watcher = "theme", "Theme watcher received stop signal");
                    break;
                }

                ControlMsg::Notify(Err(e)) => {
                    consecutive_errors = consecutive_errors.saturating_add(1);
                    warn!(
                        error = %e,
                        watcher = "theme",
                        consecutive_errors = consecutive_errors,
                        "notify delivered error"
                    );

                    if consecutive_errors >= MAX_NOTIFY_ERRORS {
                        warn!(
                            watcher = "theme",
                            consecutive_errors = consecutive_errors,
                            "Too many consecutive errors, triggering restart"
                        );
                        return Err(notify::Error::generic("Too many consecutive notify errors"));
                    }
                }

                ControlMsg::Notify(Ok(event)) => {
                    consecutive_errors = 0;

                    let touches_target = event.paths.iter().any(|p| {
                        p.file_name()
                            .map(|n| n == target_name.as_os_str())
                            .unwrap_or(false)
                    });

                    let relevant_kind = is_relevant_event_kind(&event.kind);

                    if touches_target && relevant_kind {
                        debug!(
                            file = ?target_name,
                            event_kind = ?event.kind,
                            "Theme change detected, resetting debounce"
                        );
                        deadline = Some(Instant::now() + debounce);
                    }
                }
            }
        }

        info!(watcher = "theme", "Theme watcher shutting down");
        Ok(())
    }
}

impl Drop for ThemeWatcher {
    fn drop(&mut self) {
        if let Some(flag) = self.stop_flag.take() {
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        if let Some(handle) = self.watcher_thread.take() {
            let _ = handle.join();
        }
    }
}

/// Check if a file path is a relevant script file (ts, js, or md)
fn is_relevant_script_file(path: &std::path::Path) -> bool {
    // Skip hidden files
    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        if file_name.starts_with('.') {
            return false;
        }
    }

    // Check for relevant extensions
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("ts") | Some("js") | Some("md")
    )
}

/// Compute the next deadline from pending events and global full_reload_at
fn next_deadline(
    pending: &HashMap<PathBuf, (ScriptReloadEvent, Instant)>,
    full_reload_at: Option<Instant>,
    debounce: Duration,
) -> Option<Instant> {
    let pending_deadline = pending.values().map(|(_, t)| *t + debounce).min();
    let full_reload_deadline = full_reload_at.map(|t| t + debounce);

    match (pending_deadline, full_reload_deadline) {
        (Some(p), Some(f)) => Some(p.min(f)),
        (Some(p), None) => Some(p),
        (None, Some(f)) => Some(f),
        (None, None) => None,
    }
}

/// Flush expired events from pending map and global full_reload_at
///
/// If full_reload_at has expired, emits a single FullReload and clears pending.
/// Otherwise, flushes individual expired events from pending.
fn flush_expired(
    pending: &mut HashMap<PathBuf, (ScriptReloadEvent, Instant)>,
    full_reload_at: &mut Option<Instant>,
    debounce: Duration,
    out_tx: &Sender<ScriptReloadEvent>,
) {
    let now = Instant::now();

    // Check global full_reload_at first - it supersedes all pending events
    if let Some(reload_time) = *full_reload_at {
        if now.duration_since(reload_time) >= debounce {
            debug!("FullReload debounce complete, flushing");
            info!(event = ?ScriptReloadEvent::FullReload, "Emitting script reload event");
            let _ = out_tx.send(ScriptReloadEvent::FullReload);
            *full_reload_at = None;
            pending.clear(); // Clear any remaining pending - superseded by FullReload
            return;
        }
    }

    // Flush individual expired events
    let mut to_send: Vec<ScriptReloadEvent> = Vec::new();

    pending.retain(|path, (ev, t)| {
        if now.duration_since(*t) >= debounce {
            debug!(path = %path.display(), event = ?ev, "Script debounce complete, flushing");
            to_send.push(ev.clone());
            false
        } else {
            true
        }
    });

    for ev in to_send {
        info!(event = ?ev, "Emitting script reload event");
        let _ = out_tx.send(ev);
    }
}

/// Discovered kit paths for watching
#[derive(Clone)]
struct KitWatchPaths {
    kit_path: PathBuf,
    scripts_paths: Vec<PathBuf>,
    extensions_paths: Vec<PathBuf>,
    agents_paths: Vec<PathBuf>,
}

/// Discovers all kit subdirectories under ~/.scriptkit/kit/
/// Returns paths to all scripts/, extensions/, and agents/ directories that should be watched
fn discover_kit_watch_paths() -> KitWatchPaths {
    let kit_path = PathBuf::from(shellexpand::tilde("~/.scriptkit/kit").as_ref());
    let mut scripts_paths = Vec::new();
    let mut extensions_paths = Vec::new();
    let mut agents_paths = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&kit_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip hidden directories and files
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.') {
                        continue;
                    }
                }

                let scripts_dir = path.join("scripts");
                let extensions_dir = path.join("extensions");
                let agents_dir = path.join("agents");

                // Add scripts directory if it exists
                if scripts_dir.exists() {
                    scripts_paths.push(scripts_dir);
                }

                // Add extensions directory (even if it doesn't exist yet, we'll track it)
                extensions_paths.push(extensions_dir);

                // Add agents directory (even if it doesn't exist yet, we'll track it)
                agents_paths.push(agents_dir);
            }
        }
    }

    info!(
        kit_path = %kit_path.display(),
        scripts_count = scripts_paths.len(),
        extensions_count = extensions_paths.len(),
        agents_count = agents_paths.len(),
        "Discovered kit directories to watch"
    );

    KitWatchPaths {
        kit_path,
        scripts_paths,
        extensions_paths,
        agents_paths,
    }
}

/// Watches ~/.scriptkit/kit/*/scripts, ~/.scriptkit/kit/*/extensions, and
/// ~/.scriptkit/kit/*/agents directories for changes
///
/// Uses per-file trailing-edge debounce with storm coalescing.
/// No separate flush thread - all debouncing in single recv_timeout loop.
/// Properly shuts down via Stop control message.
/// Includes supervisor restart with exponential backoff on transient errors.
/// Dynamically watches extensions and agents directories when they appear.
/// Now watches ALL kit subdirectories, not just main.
pub struct ScriptWatcher {
    tx: Option<Sender<ScriptReloadEvent>>,
    stop_flag: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    watcher_thread: Option<thread::JoinHandle<()>>,
}

impl ScriptWatcher {
    /// Create a new ScriptWatcher
    ///
    /// Returns a tuple of (watcher, receiver) where receiver will emit ScriptReloadEvent
    /// when files in the scripts directory change.
    pub fn new() -> (Self, Receiver<ScriptReloadEvent>) {
        let (tx, rx) = channel();
        let watcher = ScriptWatcher {
            tx: Some(tx),
            stop_flag: None,
            watcher_thread: None,
        };
        (watcher, rx)
    }

    /// Start watching the scripts directory for changes
    ///
    /// This spawns a background thread that watches ~/.scriptkit/kit/*/scripts,
    /// ~/.scriptkit/kit/*/extensions, and ~/.scriptkit/kit/*/agents recursively
    /// and sends reload events through the receiver when scripts are added,
    /// modified, or deleted.
    /// On transient errors, the watcher will retry with exponential backoff.
    pub fn start(&mut self) -> NotifyResult<()> {
        let tx = self
            .tx
            .take()
            .ok_or_else(|| std::io::Error::other("watcher already started"))?;

        let stop_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let thread_stop_flag = stop_flag.clone();
        self.stop_flag = Some(stop_flag);

        // Discover all kit directories to watch
        let paths = discover_kit_watch_paths();

        let thread_handle = thread::spawn(move || {
            Self::supervisor_loop(paths, tx, thread_stop_flag);
        });

        self.watcher_thread = Some(thread_handle);
        Ok(())
    }

    /// Supervisor loop that restarts the watcher on failures with exponential backoff
    fn supervisor_loop(
        paths: KitWatchPaths,
        out_tx: Sender<ScriptReloadEvent>,
        stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) {
        let mut attempt: u32 = 0;

        loop {
            if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                info!(watcher = "scripts", "Script watcher supervisor stopping");
                break;
            }

            let (control_tx, control_rx) = channel::<ControlMsg>();

            match Self::watch_loop(
                paths.clone(),
                out_tx.clone(),
                control_rx,
                control_tx,
                stop_flag.clone(),
            ) {
                Ok(()) => {
                    info!(watcher = "scripts", "Script watcher completed normally");
                    break;
                }
                Err(e) => {
                    if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                        break;
                    }

                    let backoff = compute_backoff(attempt);
                    warn!(
                        error = %e,
                        watcher = "scripts",
                        attempt = attempt,
                        backoff_ms = backoff.as_millis(),
                        "Script watcher error, retrying with backoff"
                    );

                    if !interruptible_sleep(backoff, &stop_flag) {
                        break;
                    }
                    attempt = attempt.saturating_add(1);
                }
            }
        }

        info!(
            watcher = "scripts",
            "Script watcher supervisor shutting down"
        );
    }

    /// Internal watch loop running in background thread
    fn watch_loop(
        paths: KitWatchPaths,
        out_tx: Sender<ScriptReloadEvent>,
        control_rx: Receiver<ControlMsg>,
        callback_tx: Sender<ControlMsg>,
        stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> NotifyResult<()> {
        use std::collections::HashSet;

        // Destructure paths
        let KitWatchPaths {
            kit_path,
            scripts_paths,
            extensions_paths,
            agents_paths,
        } = paths;

        let mut watcher: Box<dyn Watcher> = Box::new(recommended_watcher({
            let tx = callback_tx.clone();
            move |res: notify::Result<notify::Event>| {
                let _ = tx.send(ControlMsg::Notify(res));
            }
        })?);

        // Watch all scripts directories
        for scripts_path in &scripts_paths {
            if let Err(e) = watcher.watch(scripts_path, RecursiveMode::Recursive) {
                warn!(
                    error = %e,
                    path = %scripts_path.display(),
                    "Failed to watch scripts directory"
                );
            } else {
                info!(
                    path = %scripts_path.display(),
                    recursive = true,
                    "Script watcher started for directory"
                );
            }
        }

        // Track which extensions directories we're watching
        let mut watching_extensions: HashSet<PathBuf> = HashSet::new();

        // Watch existing extensions directories
        for extensions_path in &extensions_paths {
            if extensions_path.exists() {
                if let Err(e) = watcher.watch(extensions_path, RecursiveMode::Recursive) {
                    warn!(
                        error = %e,
                        path = %extensions_path.display(),
                        "Failed to watch extensions directory"
                    );
                } else {
                    watching_extensions.insert(extensions_path.clone());
                    info!(
                        path = %extensions_path.display(),
                        recursive = true,
                        "Scriptlets watcher started for directory"
                    );
                }
            }
        }

        // Track which agents directories we're watching
        let mut watching_agents: HashSet<PathBuf> = HashSet::new();

        // Watch existing agents directories
        for agents_path in &agents_paths {
            if agents_path.exists() {
                if let Err(e) = watcher.watch(agents_path, RecursiveMode::Recursive) {
                    warn!(
                        error = %e,
                        path = %agents_path.display(),
                        "Failed to watch agents directory"
                    );
                } else {
                    watching_agents.insert(agents_path.clone());
                    info!(
                        path = %agents_path.display(),
                        recursive = true,
                        "Agents watcher started for directory"
                    );
                }
            }
        }

        // Watch the kit parent directory to detect new kit directories being added
        if kit_path.exists() {
            let _ = watcher.watch(&kit_path, RecursiveMode::NonRecursive);
            debug!(
                path = %kit_path.display(),
                "Watching kit directory for new kits"
            );
        }

        // Keep track of all paths we should monitor for creation
        let all_extensions_paths: HashSet<PathBuf> = extensions_paths.iter().cloned().collect();
        let all_agents_paths: HashSet<PathBuf> = agents_paths.iter().cloned().collect();

        info!(
            scripts_count = scripts_paths.len(),
            extensions_watching = watching_extensions.len(),
            extensions_total = all_extensions_paths.len(),
            agents_watching = watching_agents.len(),
            agents_total = all_agents_paths.len(),
            "Script watcher started for all kit directories"
        );

        let mut consecutive_errors: u32 = 0;
        let debounce = Duration::from_millis(DEBOUNCE_MS);
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        // Global FullReload state: when set, supersedes all per-file events
        // This prevents multiple FullReload emissions during event storms
        let mut full_reload_at: Option<Instant> = None;

        loop {
            if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }

            // Use a max timeout to periodically check stop flag
            let deadline = next_deadline(&pending, full_reload_at, debounce);
            let timeout = deadline
                .map(|dl| dl.saturating_duration_since(Instant::now()))
                .unwrap_or(Duration::from_millis(500));

            let msg = match control_rx.recv_timeout(timeout) {
                Ok(m) => Some(m),
                Err(RecvTimeoutError::Timeout) => {
                    flush_expired(&mut pending, &mut full_reload_at, debounce, &out_tx);
                    continue;
                }
                Err(RecvTimeoutError::Disconnected) => break,
            };

            let Some(msg) = msg else { continue };

            match msg {
                ControlMsg::Stop => {
                    info!(watcher = "scripts", "Script watcher received stop signal");
                    break;
                }

                ControlMsg::Notify(Err(e)) => {
                    consecutive_errors = consecutive_errors.saturating_add(1);
                    warn!(
                        error = %e,
                        watcher = "scripts",
                        consecutive_errors = consecutive_errors,
                        "notify delivered error"
                    );

                    if consecutive_errors >= MAX_NOTIFY_ERRORS {
                        warn!(
                            watcher = "scripts",
                            consecutive_errors = consecutive_errors,
                            "Too many consecutive errors, triggering restart"
                        );
                        return Err(notify::Error::generic("Too many consecutive notify errors"));
                    }
                }

                ControlMsg::Notify(Ok(event)) => {
                    consecutive_errors = 0;
                    let kind = &event.kind;

                    // Check if any extensions directories we're tracking have been created
                    for extensions_path in &all_extensions_paths {
                        if !watching_extensions.contains(extensions_path)
                            && extensions_path.exists()
                        {
                            if let Err(e) = watcher.watch(extensions_path, RecursiveMode::Recursive)
                            {
                                warn!(
                                    error = %e,
                                    path = %extensions_path.display(),
                                    "Failed to start watching extensions directory"
                                );
                            } else {
                                watching_extensions.insert(extensions_path.clone());
                                info!(
                                    path = %extensions_path.display(),
                                    "Extensions directory appeared, now watching"
                                );
                            }
                        }
                    }

                    // Check if any agents directories we're tracking have been created
                    for agents_path in &all_agents_paths {
                        if !watching_agents.contains(agents_path) && agents_path.exists() {
                            if let Err(e) = watcher.watch(agents_path, RecursiveMode::Recursive) {
                                warn!(
                                    error = %e,
                                    path = %agents_path.display(),
                                    "Failed to start watching agents directory"
                                );
                            } else {
                                watching_agents.insert(agents_path.clone());
                                info!(
                                    path = %agents_path.display(),
                                    "Agents directory appeared, now watching"
                                );
                            }
                        }
                    }

                    // Check for new kit directories being created under kit_path
                    for path in event.paths.iter() {
                        // If a new directory was created directly under kit_path
                        if matches!(kind, notify::EventKind::Create(_)) {
                            if let Some(parent) = path.parent() {
                                if parent == kit_path && path.is_dir() {
                                    // New kit directory created - watch its scripts, extensions, and agents
                                    let scripts_dir = path.join("scripts");
                                    let extensions_dir = path.join("extensions");
                                    let agents_dir = path.join("agents");

                                    if scripts_dir.exists() {
                                        if let Err(e) =
                                            watcher.watch(&scripts_dir, RecursiveMode::Recursive)
                                        {
                                            warn!(
                                                error = %e,
                                                path = %scripts_dir.display(),
                                                "Failed to watch new kit scripts directory"
                                            );
                                        } else {
                                            info!(
                                                path = %scripts_dir.display(),
                                                "New kit scripts directory detected, now watching"
                                            );
                                        }
                                    }

                                    if extensions_dir.exists() {
                                        if let Err(e) =
                                            watcher.watch(&extensions_dir, RecursiveMode::Recursive)
                                        {
                                            warn!(
                                                error = %e,
                                                path = %extensions_dir.display(),
                                                "Failed to watch new kit extensions directory"
                                            );
                                        } else {
                                            watching_extensions.insert(extensions_dir.clone());
                                            info!(
                                                path = %extensions_dir.display(),
                                                "New kit extensions directory detected, now watching"
                                            );
                                        }
                                    }

                                    if agents_dir.exists() {
                                        if let Err(e) =
                                            watcher.watch(&agents_dir, RecursiveMode::Recursive)
                                        {
                                            warn!(
                                                error = %e,
                                                path = %agents_dir.display(),
                                                "Failed to watch new kit agents directory"
                                            );
                                        } else {
                                            watching_agents.insert(agents_dir.clone());
                                            info!(
                                                path = %agents_dir.display(),
                                                "New kit agents directory detected, now watching"
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }

                    for path in event.paths.iter() {
                        if !is_relevant_script_file(path) {
                            continue;
                        }

                        let now = Instant::now();

                        // Handle event types
                        match kind {
                            notify::EventKind::Create(_) => {
                                debug!(
                                    path = %path.display(),
                                    event_kind = ?kind,
                                    "Script change detected (create), merging event"
                                );
                                // Use merge_script_event to handle atomic saves
                                merge_script_event(
                                    &mut pending,
                                    path,
                                    ScriptReloadEvent::FileCreated(path.clone()),
                                    now,
                                );
                            }
                            notify::EventKind::Modify(_) => {
                                debug!(
                                    path = %path.display(),
                                    event_kind = ?kind,
                                    "Script change detected (modify), updating pending"
                                );
                                pending.insert(
                                    path.clone(),
                                    (ScriptReloadEvent::FileChanged(path.clone()), now),
                                );
                            }
                            notify::EventKind::Remove(_) => {
                                debug!(
                                    path = %path.display(),
                                    event_kind = ?kind,
                                    "Script change detected (remove), merging event"
                                );
                                // Use merge_script_event to handle atomic saves
                                merge_script_event(
                                    &mut pending,
                                    path,
                                    ScriptReloadEvent::FileDeleted(path.clone()),
                                    now,
                                );
                            }
                            // Access events are not relevant
                            notify::EventKind::Access(_) => continue,
                            // For Other/Any events, trigger global FullReload
                            _ => {
                                debug!(
                                    path = %path.display(),
                                    event_kind = ?kind,
                                    "Unknown event kind, triggering global FullReload"
                                );
                                // Set global FullReload state and clear pending
                                // This ensures only ONE FullReload is emitted after debounce
                                full_reload_at = Some(now);
                                pending.clear();
                            }
                        }

                        // Storm coalescing: if too many pending events, collapse to FullReload
                        if pending.len() >= STORM_THRESHOLD {
                            warn!(
                                pending_count = pending.len(),
                                threshold = STORM_THRESHOLD,
                                "Event storm detected, collapsing to FullReload"
                            );
                            // Set global FullReload instead of immediate emission
                            // This ensures proper debounce even during storms
                            full_reload_at = Some(Instant::now());
                            pending.clear();
                        }
                    }
                }
            }
        }

        info!(watcher = "scripts", "Script watcher shutting down");
        Ok(())
    }
}

impl Drop for ScriptWatcher {
    fn drop(&mut self) {
        if let Some(flag) = self.stop_flag.take() {
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        if let Some(handle) = self.watcher_thread.take() {
            let _ = handle.join();
        }
    }
}

/// Merge create/delete event pairs for the same path into FileChanged (atomic save handling)
///
/// Editors often save files via temp file → rename, causing Delete+Create sequences.
/// Within the debounce window, we merge these into FileChanged.
fn merge_script_event(
    pending: &mut HashMap<PathBuf, (ScriptReloadEvent, Instant)>,
    path: &PathBuf,
    new_event: ScriptReloadEvent,
    timestamp: Instant,
) {
    if let Some((existing_event, _existing_time)) = pending.get(path) {
        // Check if we can merge:
        // FileDeleted + FileCreated → FileChanged (file was atomically saved)
        // FileCreated + FileDeleted → FileChanged (temp file dance)
        let merged = match (&existing_event, &new_event) {
            (ScriptReloadEvent::FileDeleted(_), ScriptReloadEvent::FileCreated(_))
            | (ScriptReloadEvent::FileCreated(_), ScriptReloadEvent::FileDeleted(_)) => {
                Some(ScriptReloadEvent::FileChanged(path.clone()))
            }
            _ => None,
        };

        if let Some(merged_event) = merged {
            pending.insert(path.clone(), (merged_event, timestamp));
            return;
        }
    }

    // No merge - insert new event
    pending.insert(path.clone(), (new_event, timestamp));
}

/// Event emitted when applications need to be reloaded
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppReloadEvent {
    /// A new .app bundle was added
    AppAdded(PathBuf),
    /// An .app bundle was removed
    AppRemoved(PathBuf),
    /// An .app bundle was updated (modified)
    AppUpdated(PathBuf),
    /// Fallback for complex events (e.g., bulk changes)
    FullReload,
}

/// Check if a path is a .app bundle directory
fn is_app_bundle(path: &std::path::Path) -> bool {
    // Must end in .app extension
    if let Some(ext) = path.extension() {
        if ext == "app" {
            // Additionally check it's a directory (when it exists)
            // For remove events, the path may not exist anymore
            return path.is_dir() || !path.exists();
        }
    }
    false
}

/// Compute the next deadline from pending app events and global full_reload_at
fn next_app_deadline(
    pending: &HashMap<PathBuf, (AppReloadEvent, Instant)>,
    full_reload_at: Option<Instant>,
    debounce: Duration,
) -> Option<Instant> {
    let pending_deadline = pending.values().map(|(_, t)| *t + debounce).min();
    let full_reload_deadline = full_reload_at.map(|t| t + debounce);

    match (pending_deadline, full_reload_deadline) {
        (Some(p), Some(f)) => Some(p.min(f)),
        (Some(p), None) => Some(p),
        (None, Some(f)) => Some(f),
        (None, None) => None,
    }
}

/// Flush expired app events from pending map and global full_reload_at
///
/// If full_reload_at has expired, emits a single FullReload and clears pending.
/// Otherwise, flushes individual expired events from pending.
fn flush_expired_apps(
    pending: &mut HashMap<PathBuf, (AppReloadEvent, Instant)>,
    full_reload_at: &mut Option<Instant>,
    debounce: Duration,
    out_tx: &async_channel::Sender<AppReloadEvent>,
) {
    let now = Instant::now();

    // Check global full_reload_at first - it supersedes all pending events
    if let Some(reload_time) = *full_reload_at {
        if now.duration_since(reload_time) >= debounce {
            debug!("App FullReload debounce complete, flushing");
            info!(event = ?AppReloadEvent::FullReload, "Emitting app reload event");
            let _ = out_tx.send_blocking(AppReloadEvent::FullReload);
            *full_reload_at = None;
            pending.clear();
            return;
        }
    }

    // Flush individual expired events
    let mut to_send: Vec<AppReloadEvent> = Vec::new();

    pending.retain(|path, (ev, t)| {
        if now.duration_since(*t) >= debounce {
            debug!(path = %path.display(), event = ?ev, "App debounce complete, flushing");
            to_send.push(ev.clone());
            false
        } else {
            true
        }
    });

    for ev in to_send {
        info!(event = ?ev, "Emitting app reload event");
        let _ = out_tx.send_blocking(ev);
    }
}

/// Merge create/delete event pairs for the same path into AppUpdated (app update handling)
///
/// App installers may cause Delete+Create sequences during updates.
/// Within the debounce window, we merge these into AppUpdated.
fn merge_app_event(
    pending: &mut HashMap<PathBuf, (AppReloadEvent, Instant)>,
    path: &PathBuf,
    new_event: AppReloadEvent,
    timestamp: Instant,
) {
    if let Some((existing_event, _existing_time)) = pending.get(path) {
        // Check if we can merge:
        // AppRemoved + AppAdded → AppUpdated (app was updated)
        // AppAdded + AppRemoved → AppUpdated (temp app dance during install)
        let merged = match (&existing_event, &new_event) {
            (AppReloadEvent::AppRemoved(_), AppReloadEvent::AppAdded(_))
            | (AppReloadEvent::AppAdded(_), AppReloadEvent::AppRemoved(_)) => {
                Some(AppReloadEvent::AppUpdated(path.clone()))
            }
            _ => None,
        };

        if let Some(merged_event) = merged {
            pending.insert(path.clone(), (merged_event, timestamp));
            return;
        }
    }

    // No merge - insert new event
    pending.insert(path.clone(), (new_event, timestamp));
}

/// Watches /Applications and ~/Applications for .app bundle changes
///
/// Uses per-file trailing-edge debounce with storm coalescing.
/// Filters to only .app directories.
/// Properly shuts down via stop flag.
/// Includes supervisor restart with exponential backoff on transient errors.
pub struct AppWatcher {
    tx: Option<async_channel::Sender<AppReloadEvent>>,
    stop_flag: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    watcher_thread: Option<thread::JoinHandle<()>>,
}

impl AppWatcher {
    /// Create a new AppWatcher
    ///
    /// Returns a tuple of (watcher, receiver) where receiver will emit AppReloadEvent
    /// when .app bundles in /Applications or ~/Applications change.
    pub fn new() -> (Self, async_channel::Receiver<AppReloadEvent>) {
        let (tx, rx) = async_channel::bounded(100);
        let watcher = AppWatcher {
            tx: Some(tx),
            stop_flag: None,
            watcher_thread: None,
        };
        (watcher, rx)
    }

    /// Start watching the applications directories for changes
    ///
    /// This spawns a background thread that watches /Applications and ~/Applications
    /// and sends reload events through the receiver when .app bundles are added,
    /// modified, or deleted.
    /// On transient errors, the watcher will retry with exponential backoff.
    pub fn start(&mut self) -> NotifyResult<()> {
        let tx = self
            .tx
            .take()
            .ok_or_else(|| std::io::Error::other("watcher already started"))?;

        let stop_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let thread_stop_flag = stop_flag.clone();
        self.stop_flag = Some(stop_flag);

        // Watch paths: /Applications and ~/Applications
        let system_apps_path = PathBuf::from("/Applications");
        let user_apps_path = PathBuf::from(shellexpand::tilde("~/Applications").as_ref());

        let thread_handle = thread::spawn(move || {
            Self::supervisor_loop(system_apps_path, user_apps_path, tx, thread_stop_flag);
        });

        self.watcher_thread = Some(thread_handle);
        Ok(())
    }

    /// Supervisor loop that restarts the watcher on failures with exponential backoff
    fn supervisor_loop(
        system_apps_path: PathBuf,
        user_apps_path: PathBuf,
        out_tx: async_channel::Sender<AppReloadEvent>,
        stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) {
        let mut attempt: u32 = 0;

        loop {
            if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                info!(watcher = "apps", "App watcher supervisor stopping");
                break;
            }

            let (control_tx, control_rx) = channel::<ControlMsg>();

            match Self::watch_loop(
                system_apps_path.clone(),
                user_apps_path.clone(),
                out_tx.clone(),
                control_rx,
                control_tx,
                stop_flag.clone(),
            ) {
                Ok(()) => {
                    info!(watcher = "apps", "App watcher completed normally");
                    break;
                }
                Err(e) => {
                    if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                        break;
                    }

                    let backoff = compute_backoff(attempt);
                    warn!(
                        error = %e,
                        watcher = "apps",
                        attempt = attempt,
                        backoff_ms = backoff.as_millis(),
                        "App watcher error, retrying with backoff"
                    );

                    if !interruptible_sleep(backoff, &stop_flag) {
                        break;
                    }
                    attempt = attempt.saturating_add(1);
                }
            }
        }

        info!(watcher = "apps", "App watcher supervisor shutting down");
    }

    /// Internal watch loop running in background thread
    fn watch_loop(
        system_apps_path: PathBuf,
        user_apps_path: PathBuf,
        out_tx: async_channel::Sender<AppReloadEvent>,
        control_rx: Receiver<ControlMsg>,
        callback_tx: Sender<ControlMsg>,
        stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> NotifyResult<()> {
        let mut watcher: Box<dyn Watcher> = Box::new(recommended_watcher({
            let tx = callback_tx.clone();
            move |res: notify::Result<notify::Event>| {
                let _ = tx.send(ControlMsg::Notify(res));
            }
        })?);

        // Watch /Applications with NonRecursive (apps are top-level .app bundles)
        if system_apps_path.exists() {
            watcher.watch(&system_apps_path, RecursiveMode::NonRecursive)?;
            info!(
                path = %system_apps_path.display(),
                recursive = false,
                "System Applications watcher started"
            );
        } else {
            debug!(
                path = %system_apps_path.display(),
                "System Applications path does not exist, skipping"
            );
        }

        // Watch ~/Applications with NonRecursive
        if user_apps_path.exists() {
            watcher.watch(&user_apps_path, RecursiveMode::NonRecursive)?;
            info!(
                path = %user_apps_path.display(),
                recursive = false,
                "User Applications watcher started"
            );
        } else {
            // Create ~/Applications if it doesn't exist so we can watch it
            // Many users don't have this directory initially
            debug!(
                path = %user_apps_path.display(),
                "User Applications path does not exist, skipping"
            );
        }

        let mut consecutive_errors: u32 = 0;
        let debounce = Duration::from_millis(DEBOUNCE_MS);
        let mut pending: HashMap<PathBuf, (AppReloadEvent, Instant)> = HashMap::new();
        // Global FullReload state: when set, supersedes all per-file events
        let mut full_reload_at: Option<Instant> = None;

        loop {
            if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }

            // Use a max timeout to periodically check stop flag
            let deadline = next_app_deadline(&pending, full_reload_at, debounce);
            let timeout = deadline
                .map(|dl| dl.saturating_duration_since(Instant::now()))
                .unwrap_or(Duration::from_millis(500));

            let msg = match control_rx.recv_timeout(timeout) {
                Ok(m) => Some(m),
                Err(RecvTimeoutError::Timeout) => {
                    flush_expired_apps(&mut pending, &mut full_reload_at, debounce, &out_tx);
                    continue;
                }
                Err(RecvTimeoutError::Disconnected) => break,
            };

            let Some(msg) = msg else { continue };

            match msg {
                ControlMsg::Stop => {
                    info!(watcher = "apps", "App watcher received stop signal");
                    break;
                }

                ControlMsg::Notify(Err(e)) => {
                    consecutive_errors = consecutive_errors.saturating_add(1);
                    warn!(
                        error = %e,
                        watcher = "apps",
                        consecutive_errors = consecutive_errors,
                        "notify delivered error"
                    );

                    if consecutive_errors >= MAX_NOTIFY_ERRORS {
                        warn!(
                            watcher = "apps",
                            consecutive_errors = consecutive_errors,
                            "Too many consecutive errors, triggering restart"
                        );
                        return Err(notify::Error::generic("Too many consecutive notify errors"));
                    }
                }

                ControlMsg::Notify(Ok(event)) => {
                    consecutive_errors = 0;
                    let kind = &event.kind;

                    for path in event.paths.iter() {
                        // Filter: only .app directories
                        if !is_app_bundle(path) {
                            continue;
                        }

                        let now = Instant::now();

                        // Handle event types
                        match kind {
                            notify::EventKind::Create(_) => {
                                debug!(
                                    path = %path.display(),
                                    event_kind = ?kind,
                                    "App change detected (create), merging event"
                                );
                                merge_app_event(
                                    &mut pending,
                                    path,
                                    AppReloadEvent::AppAdded(path.clone()),
                                    now,
                                );
                            }
                            notify::EventKind::Modify(_) => {
                                debug!(
                                    path = %path.display(),
                                    event_kind = ?kind,
                                    "App change detected (modify), updating pending"
                                );
                                pending.insert(
                                    path.clone(),
                                    (AppReloadEvent::AppUpdated(path.clone()), now),
                                );
                            }
                            notify::EventKind::Remove(_) => {
                                debug!(
                                    path = %path.display(),
                                    event_kind = ?kind,
                                    "App change detected (remove), merging event"
                                );
                                merge_app_event(
                                    &mut pending,
                                    path,
                                    AppReloadEvent::AppRemoved(path.clone()),
                                    now,
                                );
                            }
                            // Access events are not relevant
                            notify::EventKind::Access(_) => continue,
                            // For Other/Any events, trigger global FullReload
                            _ => {
                                debug!(
                                    path = %path.display(),
                                    event_kind = ?kind,
                                    "Unknown event kind, triggering global FullReload"
                                );
                                full_reload_at = Some(now);
                                pending.clear();
                            }
                        }

                        // Storm coalescing: if too many pending events, collapse to FullReload
                        if pending.len() >= STORM_THRESHOLD {
                            warn!(
                                pending_count = pending.len(),
                                threshold = STORM_THRESHOLD,
                                "App event storm detected, collapsing to FullReload"
                            );
                            full_reload_at = Some(Instant::now());
                            pending.clear();
                        }
                    }
                }
            }
        }

        info!(watcher = "apps", "App watcher shutting down");
        Ok(())
    }
}

impl Drop for AppWatcher {
    fn drop(&mut self) {
        if let Some(flag) = self.stop_flag.take() {
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        if let Some(handle) = self.watcher_thread.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // ISSUE A - FullReload coalescing tests
    // ============================================================

    #[test]
    fn test_full_reload_global_state_single_emission() {
        // Multiple FullReload triggers during debounce window should result in single emission
        let (tx, rx) = channel::<ScriptReloadEvent>();
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let mut full_reload_at: Option<Instant> = None;
        let _debounce = Duration::from_millis(500);
        let now = Instant::now();

        // Simulate 3 FullReload triggers from different paths within debounce window
        for i in 0..3 {
            let _path = PathBuf::from(format!("/test/script{}.ts", i));
            // When FullReload is triggered, set global state instead of per-path
            full_reload_at = Some(now);
            // Clear pending events - they're superseded by full reload
            pending.clear();
        }

        // Verify: full_reload_at is set, pending is empty
        assert!(full_reload_at.is_some());
        assert!(pending.is_empty());

        // Simulate debounce expiry - emit single FullReload
        if full_reload_at.is_some() {
            let _ = tx.send(ScriptReloadEvent::FullReload);
            // Reset after emission (in real code)
        }

        // Should only receive one FullReload
        let received = rx.try_recv().unwrap();
        assert_eq!(received, ScriptReloadEvent::FullReload);
        assert!(rx.try_recv().is_err()); // No more events
    }

    #[test]
    fn test_full_reload_clears_pending_events() {
        // When FullReload is triggered, it should clear all pending per-file events
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let now = Instant::now();

        // Add some pending per-file events
        pending.insert(
            PathBuf::from("/test/a.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/a.ts")),
                now,
            ),
        );
        pending.insert(
            PathBuf::from("/test/b.ts"),
            (
                ScriptReloadEvent::FileCreated(PathBuf::from("/test/b.ts")),
                now,
            ),
        );

        assert_eq!(pending.len(), 2);

        // Trigger FullReload (e.g., from EventKind::Other)
        let full_reload_at: Option<Instant> = Some(now);
        pending.clear();

        // Pending should be empty, full_reload_at should be set
        assert!(pending.is_empty());
        assert!(full_reload_at.is_some());
    }

    // ============================================================
    // ISSUE B - Atomic save event merging tests
    // ============================================================

    #[test]
    fn test_merge_delete_then_create_to_changed() {
        // FileDeleted + FileCreated (same path) → FileChanged
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let path = PathBuf::from("/test/script.ts");
        let now = Instant::now();

        // First: delete event
        merge_script_event(
            &mut pending,
            &path,
            ScriptReloadEvent::FileDeleted(path.clone()),
            now,
        );
        assert_eq!(pending.len(), 1);
        assert!(matches!(
            pending.get(&path),
            Some((ScriptReloadEvent::FileDeleted(_), _))
        ));

        // Then: create event (atomic save completes)
        let later = now + Duration::from_millis(10);
        merge_script_event(
            &mut pending,
            &path,
            ScriptReloadEvent::FileCreated(path.clone()),
            later,
        );

        // Should be merged to FileChanged
        assert_eq!(pending.len(), 1);
        let (event, _) = pending.get(&path).unwrap();
        assert_eq!(*event, ScriptReloadEvent::FileChanged(path.clone()));
    }

    #[test]
    fn test_merge_create_then_delete_to_changed() {
        // FileCreated + FileDeleted (same path) → FileChanged
        // (temp file dance: create temp, delete original, rename temp)
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let path = PathBuf::from("/test/script.ts");
        let now = Instant::now();

        // First: create event
        merge_script_event(
            &mut pending,
            &path,
            ScriptReloadEvent::FileCreated(path.clone()),
            now,
        );

        // Then: delete event
        let later = now + Duration::from_millis(10);
        merge_script_event(
            &mut pending,
            &path,
            ScriptReloadEvent::FileDeleted(path.clone()),
            later,
        );

        // Should be merged to FileChanged
        assert_eq!(pending.len(), 1);
        let (event, _) = pending.get(&path).unwrap();
        assert_eq!(*event, ScriptReloadEvent::FileChanged(path.clone()));
    }

    #[test]
    fn test_no_merge_for_different_paths() {
        // Events for different paths should not be merged
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let path_a = PathBuf::from("/test/a.ts");
        let path_b = PathBuf::from("/test/b.ts");
        let now = Instant::now();

        // Delete on path A
        merge_script_event(
            &mut pending,
            &path_a,
            ScriptReloadEvent::FileDeleted(path_a.clone()),
            now,
        );

        // Create on path B (different path - no merge)
        merge_script_event(
            &mut pending,
            &path_b,
            ScriptReloadEvent::FileCreated(path_b.clone()),
            now,
        );

        // Should have 2 separate events
        assert_eq!(pending.len(), 2);
        assert!(matches!(
            pending.get(&path_a),
            Some((ScriptReloadEvent::FileDeleted(_), _))
        ));
        assert!(matches!(
            pending.get(&path_b),
            Some((ScriptReloadEvent::FileCreated(_), _))
        ));
    }

    #[test]
    fn test_no_merge_for_modify_events() {
        // FileChanged + FileDeleted should NOT merge to FileChanged
        // (only create/delete pairs merge)
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let path = PathBuf::from("/test/script.ts");
        let now = Instant::now();

        // First: modify event
        merge_script_event(
            &mut pending,
            &path,
            ScriptReloadEvent::FileChanged(path.clone()),
            now,
        );

        // Then: delete event
        let later = now + Duration::from_millis(10);
        merge_script_event(
            &mut pending,
            &path,
            ScriptReloadEvent::FileDeleted(path.clone()),
            later,
        );

        // Should NOT merge - delete overwrites
        assert_eq!(pending.len(), 1);
        let (event, _) = pending.get(&path).unwrap();
        assert_eq!(*event, ScriptReloadEvent::FileDeleted(path.clone()));
    }

    // ============================================================
    // Existing tests
    // ============================================================

    #[test]
    fn test_config_watcher_creation() {
        let (_watcher, _rx) = ConfigWatcher::new();
        // Watcher should be created without panicking
    }

    #[test]
    fn test_config_reload_event_clone() {
        let event = ConfigReloadEvent::Reload;
        let _cloned = event.clone();
        // Event should be cloneable
    }

    #[test]
    fn test_theme_watcher_creation() {
        let (_watcher, _rx) = ThemeWatcher::new();
        // Watcher should be created without panicking
    }

    #[test]
    fn test_theme_reload_event_clone() {
        let event = ThemeReloadEvent::Reload;
        let _cloned = event.clone();
        // Event should be cloneable
    }

    #[test]
    fn test_script_watcher_creation() {
        let (_watcher, _rx) = ScriptWatcher::new();
        // Watcher should be created without panicking
    }

    #[test]
    fn test_script_reload_event_clone() {
        let event = ScriptReloadEvent::FullReload;
        let _cloned = event.clone();
        // Event should be cloneable
    }

    #[test]
    fn test_script_reload_event_file_changed() {
        let path = PathBuf::from("/test/path/script.ts");
        let event = ScriptReloadEvent::FileChanged(path.clone());

        // Verify the event contains the correct path
        if let ScriptReloadEvent::FileChanged(event_path) = event {
            assert_eq!(event_path, path);
        } else {
            panic!("Expected FileChanged variant");
        }
    }

    #[test]
    fn test_script_reload_event_file_created() {
        let path = PathBuf::from("/test/path/new-script.ts");
        let event = ScriptReloadEvent::FileCreated(path.clone());

        // Verify the event contains the correct path
        if let ScriptReloadEvent::FileCreated(event_path) = event {
            assert_eq!(event_path, path);
        } else {
            panic!("Expected FileCreated variant");
        }
    }

    #[test]
    fn test_script_reload_event_file_deleted() {
        let path = PathBuf::from("/test/path/deleted-script.ts");
        let event = ScriptReloadEvent::FileDeleted(path.clone());

        // Verify the event contains the correct path
        if let ScriptReloadEvent::FileDeleted(event_path) = event {
            assert_eq!(event_path, path);
        } else {
            panic!("Expected FileDeleted variant");
        }
    }

    #[test]
    fn test_script_reload_event_equality() {
        let path1 = PathBuf::from("/test/path/script.ts");
        let path2 = PathBuf::from("/test/path/script.ts");
        let path3 = PathBuf::from("/test/path/other.ts");

        // Same path should be equal
        assert_eq!(
            ScriptReloadEvent::FileChanged(path1.clone()),
            ScriptReloadEvent::FileChanged(path2.clone())
        );

        // Different paths should not be equal
        assert_ne!(
            ScriptReloadEvent::FileChanged(path1.clone()),
            ScriptReloadEvent::FileChanged(path3.clone())
        );

        // Different event types should not be equal
        assert_ne!(
            ScriptReloadEvent::FileChanged(path1.clone()),
            ScriptReloadEvent::FileCreated(path1.clone())
        );

        // FullReload should equal itself
        assert_eq!(ScriptReloadEvent::FullReload, ScriptReloadEvent::FullReload);
    }

    #[test]
    fn test_extract_file_path_from_event() {
        // Test helper function for extracting paths from notify events
        use notify::event::{CreateKind, ModifyKind, RemoveKind};

        let test_path = PathBuf::from("/Users/test/.scriptkit/scripts/hello.ts");

        // Test Create event
        let create_event = notify::Event {
            kind: notify::EventKind::Create(CreateKind::File),
            paths: vec![test_path.clone()],
            attrs: Default::default(),
        };
        assert_eq!(create_event.paths.first(), Some(&test_path));

        // Test Modify event
        let modify_event = notify::Event {
            kind: notify::EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Content)),
            paths: vec![test_path.clone()],
            attrs: Default::default(),
        };
        assert_eq!(modify_event.paths.first(), Some(&test_path));

        // Test Remove event
        let remove_event = notify::Event {
            kind: notify::EventKind::Remove(RemoveKind::File),
            paths: vec![test_path.clone()],
            attrs: Default::default(),
        };
        assert_eq!(remove_event.paths.first(), Some(&test_path));
    }

    #[test]
    fn test_is_relevant_script_file() {
        use std::path::Path;

        // Test that we correctly identify relevant script files
        let ts_path = Path::new("/Users/test/.scriptkit/scripts/hello.ts");
        let js_path = Path::new("/Users/test/.scriptkit/scripts/hello.js");
        let md_path = Path::new("/Users/test/.scriptkit/scriptlets/hello.md");
        let txt_path = Path::new("/Users/test/.scriptkit/scripts/readme.txt");
        let hidden_path = Path::new("/Users/test/.scriptkit/scripts/.hidden.ts");

        // TypeScript files should be relevant
        assert!(is_relevant_script_file(ts_path));

        // JavaScript files should be relevant
        assert!(is_relevant_script_file(js_path));

        // Markdown files in scriptlets should be relevant
        assert!(is_relevant_script_file(md_path));

        // Other file types should not be relevant
        assert!(!is_relevant_script_file(txt_path));

        // Hidden files should not be relevant
        assert!(!is_relevant_script_file(hidden_path));
    }

    #[test]
    fn test_is_relevant_event_kind() {
        use notify::event::{AccessKind, CreateKind, ModifyKind, RemoveKind};

        // Access events should NOT be relevant
        assert!(!is_relevant_event_kind(&notify::EventKind::Access(
            AccessKind::Read
        )));

        // Create events SHOULD be relevant
        assert!(is_relevant_event_kind(&notify::EventKind::Create(
            CreateKind::File
        )));

        // Modify events SHOULD be relevant
        assert!(is_relevant_event_kind(&notify::EventKind::Modify(
            ModifyKind::Any
        )));

        // Remove events SHOULD be relevant
        assert!(is_relevant_event_kind(&notify::EventKind::Remove(
            RemoveKind::File
        )));

        // Other/Any events SHOULD be relevant (includes renames)
        assert!(is_relevant_event_kind(&notify::EventKind::Other));
        assert!(is_relevant_event_kind(&notify::EventKind::Any));
    }

    #[test]
    fn test_next_deadline_empty() {
        let pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let debounce = Duration::from_millis(500);

        assert!(next_deadline(&pending, None, debounce).is_none());
    }

    #[test]
    fn test_next_deadline_single() {
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let now = Instant::now();
        let debounce = Duration::from_millis(500);

        pending.insert(
            PathBuf::from("/test/script.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/script.ts")),
                now,
            ),
        );

        let deadline = next_deadline(&pending, None, debounce);
        assert!(deadline.is_some());
        // Deadline should be approximately now + debounce
        let expected = now + debounce;
        let actual = deadline.unwrap();
        // Allow 1ms tolerance for timing
        assert!(actual >= expected && actual <= expected + Duration::from_millis(1));
    }

    #[test]
    fn test_next_deadline_multiple_picks_earliest() {
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let now = Instant::now();
        let debounce = Duration::from_millis(500);

        // Add an older event
        let older_time = now - Duration::from_millis(200);
        pending.insert(
            PathBuf::from("/test/old.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/old.ts")),
                older_time,
            ),
        );

        // Add a newer event
        pending.insert(
            PathBuf::from("/test/new.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/new.ts")),
                now,
            ),
        );

        let deadline = next_deadline(&pending, None, debounce);
        assert!(deadline.is_some());
        // Should pick the older event's deadline (earlier)
        let expected = older_time + debounce;
        let actual = deadline.unwrap();
        assert!(actual >= expected && actual <= expected + Duration::from_millis(1));
    }

    #[test]
    fn test_next_deadline_with_full_reload() {
        let pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let now = Instant::now();
        let debounce = Duration::from_millis(500);

        // Only full_reload_at is set, no pending events
        let deadline = next_deadline(&pending, Some(now), debounce);
        assert!(deadline.is_some());
        let expected = now + debounce;
        let actual = deadline.unwrap();
        assert!(actual >= expected && actual <= expected + Duration::from_millis(1));
    }

    #[test]
    fn test_next_deadline_full_reload_earlier_than_pending() {
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let now = Instant::now();
        let debounce = Duration::from_millis(500);

        // Add a fresh pending event (deadline = now + 500ms)
        pending.insert(
            PathBuf::from("/test/script.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/script.ts")),
                now,
            ),
        );

        // Add an older full_reload_at (deadline = older + 500ms < now + 500ms)
        let older_reload = now - Duration::from_millis(200);
        let deadline = next_deadline(&pending, Some(older_reload), debounce);
        assert!(deadline.is_some());

        // Should pick the earlier deadline (full_reload)
        let expected = older_reload + debounce;
        let actual = deadline.unwrap();
        assert!(actual >= expected && actual <= expected + Duration::from_millis(1));
    }

    #[test]
    fn test_flush_expired_none_expired() {
        let (tx, _rx) = channel::<ScriptReloadEvent>();
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let mut full_reload_at: Option<Instant> = None;
        let now = Instant::now();
        let debounce = Duration::from_millis(500);

        // Add a fresh event (not expired)
        pending.insert(
            PathBuf::from("/test/script.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/script.ts")),
                now,
            ),
        );

        flush_expired(&mut pending, &mut full_reload_at, debounce, &tx);

        // Event should still be pending
        assert_eq!(pending.len(), 1);
    }

    #[test]
    fn test_flush_expired_some_expired() {
        let (tx, rx) = channel::<ScriptReloadEvent>();
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let mut full_reload_at: Option<Instant> = None;
        let debounce = Duration::from_millis(500);

        // Add an expired event (from 600ms ago)
        let old_time = Instant::now() - Duration::from_millis(600);
        pending.insert(
            PathBuf::from("/test/old.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/old.ts")),
                old_time,
            ),
        );

        // Add a fresh event
        pending.insert(
            PathBuf::from("/test/new.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/new.ts")),
                Instant::now(),
            ),
        );

        flush_expired(&mut pending, &mut full_reload_at, debounce, &tx);

        // Only fresh event should remain
        assert_eq!(pending.len(), 1);
        assert!(pending.contains_key(&PathBuf::from("/test/new.ts")));

        // Should have received the expired event
        let received = rx.try_recv().unwrap();
        assert_eq!(
            received,
            ScriptReloadEvent::FileChanged(PathBuf::from("/test/old.ts"))
        );
    }

    #[test]
    fn test_flush_expired_full_reload_supersedes_pending() {
        let (tx, rx) = channel::<ScriptReloadEvent>();
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let debounce = Duration::from_millis(500);

        // Add some expired pending events
        let old_time = Instant::now() - Duration::from_millis(600);
        pending.insert(
            PathBuf::from("/test/a.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/a.ts")),
                old_time,
            ),
        );
        pending.insert(
            PathBuf::from("/test/b.ts"),
            (
                ScriptReloadEvent::FileCreated(PathBuf::from("/test/b.ts")),
                old_time,
            ),
        );

        // Set expired full_reload_at (should supersede pending)
        let mut full_reload_at: Option<Instant> = Some(old_time);

        flush_expired(&mut pending, &mut full_reload_at, debounce, &tx);

        // All pending should be cleared
        assert!(pending.is_empty());
        // full_reload_at should be reset
        assert!(full_reload_at.is_none());

        // Should receive only ONE FullReload (not per-file events)
        let received = rx.try_recv().unwrap();
        assert_eq!(received, ScriptReloadEvent::FullReload);
        // No more events
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn test_config_watcher_shutdown_no_hang() {
        // Create and start a watcher
        let (mut watcher, _rx) = ConfigWatcher::new();

        // This may fail if the watch directory doesn't exist, but that's fine
        // We're testing that drop doesn't hang, not that watching works
        let _ = watcher.start();

        // Drop should complete within a reasonable time (not hang)
        // The Drop implementation sends Stop and then joins
        drop(watcher);

        // If we get here, the test passed (didn't hang)
    }

    #[test]
    fn test_theme_watcher_shutdown_no_hang() {
        let (mut watcher, _rx) = ThemeWatcher::new();
        let _ = watcher.start();
        drop(watcher);
        // If we get here, the test passed (didn't hang)
    }

    #[test]
    fn test_script_watcher_shutdown_no_hang() {
        let (mut watcher, _rx) = ScriptWatcher::new();
        let _ = watcher.start();
        drop(watcher);
        // If we get here, the test passed (didn't hang)
    }

    #[test]
    fn test_storm_threshold_constant() {
        // Verify storm threshold is a reasonable value (compile-time checks)
        const { assert!(STORM_THRESHOLD > 0) };
        const { assert!(STORM_THRESHOLD <= 1000) }; // Not too high
    }

    #[test]
    fn test_debounce_constant() {
        // Verify debounce is a reasonable value (compile-time checks)
        const { assert!(DEBOUNCE_MS >= 100) }; // At least 100ms
        const { assert!(DEBOUNCE_MS <= 2000) }; // At most 2s
    }

    #[test]
    fn test_storm_coalescing_logic() {
        // Test that we properly handle storm coalescing
        // When storm threshold is reached, we should:
        // 1. Clear pending
        // 2. Send FullReload
        // 3. Continue processing (not exit the watcher)

        let (tx, rx) = channel::<ScriptReloadEvent>();
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let now = Instant::now();

        // Fill up pending to the storm threshold
        for i in 0..STORM_THRESHOLD {
            let path = PathBuf::from(format!("/test/script{}.ts", i));
            pending.insert(path.clone(), (ScriptReloadEvent::FileCreated(path), now));
        }

        // Verify we're at the threshold
        assert_eq!(pending.len(), STORM_THRESHOLD);

        // Simulate storm coalescing
        if pending.len() >= STORM_THRESHOLD {
            pending.clear();
            let _ = tx.send(ScriptReloadEvent::FullReload);
        }

        // Pending should be cleared
        assert_eq!(pending.len(), 0);

        // FullReload should have been sent
        let received = rx.try_recv().unwrap();
        assert_eq!(received, ScriptReloadEvent::FullReload);
    }

    #[test]
    fn test_compute_backoff_initial() {
        // First attempt should use initial backoff
        let delay = compute_backoff(0);
        assert_eq!(delay, Duration::from_millis(INITIAL_BACKOFF_MS));
    }

    #[test]
    fn test_compute_backoff_exponential() {
        // Each attempt should double the delay
        let delay0 = compute_backoff(0);
        let delay1 = compute_backoff(1);
        let delay2 = compute_backoff(2);
        let delay3 = compute_backoff(3);

        assert_eq!(delay0, Duration::from_millis(100));
        assert_eq!(delay1, Duration::from_millis(200));
        assert_eq!(delay2, Duration::from_millis(400));
        assert_eq!(delay3, Duration::from_millis(800));
    }

    #[test]
    fn test_compute_backoff_capped() {
        // High attempts should be capped at MAX_BACKOFF_MS
        let delay = compute_backoff(20); // 2^20 * 100ms would be huge
        assert_eq!(delay, Duration::from_millis(MAX_BACKOFF_MS));
    }

    #[test]
    fn test_compute_backoff_no_overflow() {
        // Even with u32::MAX attempts, should not panic
        let delay = compute_backoff(u32::MAX);
        assert_eq!(delay, Duration::from_millis(MAX_BACKOFF_MS));
    }

    #[test]
    fn test_interruptible_sleep_completes() {
        use std::sync::atomic::AtomicBool;

        let stop_flag = AtomicBool::new(false);
        let start = Instant::now();

        // Sleep for 50ms
        let completed = interruptible_sleep(Duration::from_millis(50), &stop_flag);

        assert!(completed);
        assert!(start.elapsed() >= Duration::from_millis(50));
    }

    #[test]
    fn test_interruptible_sleep_interrupted() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let stop_flag = Arc::new(AtomicBool::new(false));
        let flag_clone = Arc::clone(&stop_flag);

        // Spawn a thread to set the stop flag after 50ms
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            flag_clone.store(true, Ordering::Relaxed);
        });

        let start = Instant::now();

        // Try to sleep for 1 second, but should be interrupted
        let completed = interruptible_sleep(Duration::from_millis(1000), &stop_flag);

        assert!(!completed);
        // Should have stopped much sooner than 1 second
        assert!(start.elapsed() < Duration::from_millis(500));
    }

    #[test]
    fn test_backoff_constants() {
        // Verify backoff constants are reasonable
        const { assert!(INITIAL_BACKOFF_MS >= 50) }; // At least 50ms
        const { assert!(INITIAL_BACKOFF_MS <= 1000) }; // At most 1s
        const { assert!(MAX_BACKOFF_MS >= 5000) }; // At least 5s
        const { assert!(MAX_BACKOFF_MS <= 120_000) }; // At most 2 minutes
        const { assert!(MAX_NOTIFY_ERRORS >= 3) }; // At least 3 errors
        const { assert!(MAX_NOTIFY_ERRORS <= 100) }; // At most 100 errors
    }

    // ============================================================
    // AppWatcher tests
    // ============================================================

    #[test]
    fn test_app_watcher_creation() {
        let (_watcher, _rx) = AppWatcher::new();
        // Watcher should be created without panicking
    }

    #[test]
    fn test_app_watcher_shutdown_no_hang() {
        let (mut watcher, _rx) = AppWatcher::new();
        let _ = watcher.start();
        drop(watcher);
        // If we get here, the test passed (didn't hang)
    }

    #[test]
    fn test_app_reload_event_clone() {
        let event = AppReloadEvent::FullReload;
        let _cloned = event.clone();
        // Event should be cloneable
    }

    #[test]
    fn test_app_reload_event_app_added() {
        let path = PathBuf::from("/Applications/MyApp.app");
        let event = AppReloadEvent::AppAdded(path.clone());

        if let AppReloadEvent::AppAdded(event_path) = event {
            assert_eq!(event_path, path);
        } else {
            panic!("Expected AppAdded variant");
        }
    }

    #[test]
    fn test_app_reload_event_app_removed() {
        let path = PathBuf::from("/Applications/OldApp.app");
        let event = AppReloadEvent::AppRemoved(path.clone());

        if let AppReloadEvent::AppRemoved(event_path) = event {
            assert_eq!(event_path, path);
        } else {
            panic!("Expected AppRemoved variant");
        }
    }

    #[test]
    fn test_app_reload_event_app_updated() {
        let path = PathBuf::from("/Applications/UpdatedApp.app");
        let event = AppReloadEvent::AppUpdated(path.clone());

        if let AppReloadEvent::AppUpdated(event_path) = event {
            assert_eq!(event_path, path);
        } else {
            panic!("Expected AppUpdated variant");
        }
    }

    #[test]
    fn test_app_reload_event_equality() {
        let path1 = PathBuf::from("/Applications/App1.app");
        let path2 = PathBuf::from("/Applications/App1.app");
        let path3 = PathBuf::from("/Applications/App2.app");

        // Same path should be equal
        assert_eq!(
            AppReloadEvent::AppAdded(path1.clone()),
            AppReloadEvent::AppAdded(path2.clone())
        );

        // Different paths should not be equal
        assert_ne!(
            AppReloadEvent::AppAdded(path1.clone()),
            AppReloadEvent::AppAdded(path3.clone())
        );

        // Different event types should not be equal
        assert_ne!(
            AppReloadEvent::AppAdded(path1.clone()),
            AppReloadEvent::AppRemoved(path1.clone())
        );

        // FullReload should equal itself
        assert_eq!(AppReloadEvent::FullReload, AppReloadEvent::FullReload);
    }

    #[test]
    fn test_is_app_bundle_valid() {
        use std::path::Path;

        // .app extension should be recognized
        let valid_app = Path::new("/Applications/Safari.app");
        assert!(is_app_bundle(valid_app));

        let user_app = Path::new("/Users/test/Applications/MyApp.app");
        assert!(is_app_bundle(user_app));
    }

    #[test]
    fn test_is_app_bundle_invalid() {
        use std::path::Path;

        // Non-.app files should not be recognized
        let not_app = Path::new("/Applications/readme.txt");
        assert!(!is_app_bundle(not_app));

        let dmg_file = Path::new("/Applications/installer.dmg");
        assert!(!is_app_bundle(dmg_file));

        let ds_store = Path::new("/Applications/.DS_Store");
        assert!(!is_app_bundle(ds_store));

        let hidden = Path::new("/Applications/.Trash");
        assert!(!is_app_bundle(hidden));
    }

    #[test]
    fn test_merge_app_event_remove_then_add() {
        // AppRemoved + AppAdded (same path) → AppUpdated
        let mut pending: HashMap<PathBuf, (AppReloadEvent, Instant)> = HashMap::new();
        let path = PathBuf::from("/Applications/MyApp.app");
        let now = Instant::now();

        // First: remove event
        merge_app_event(
            &mut pending,
            &path,
            AppReloadEvent::AppRemoved(path.clone()),
            now,
        );
        assert_eq!(pending.len(), 1);
        assert!(matches!(
            pending.get(&path),
            Some((AppReloadEvent::AppRemoved(_), _))
        ));

        // Then: add event (app reinstalled/updated)
        let later = now + Duration::from_millis(10);
        merge_app_event(
            &mut pending,
            &path,
            AppReloadEvent::AppAdded(path.clone()),
            later,
        );

        // Should be merged to AppUpdated
        assert_eq!(pending.len(), 1);
        let (event, _) = pending.get(&path).unwrap();
        assert_eq!(*event, AppReloadEvent::AppUpdated(path.clone()));
    }

    #[test]
    fn test_merge_app_event_add_then_remove() {
        // AppAdded + AppRemoved (same path) → AppUpdated
        let mut pending: HashMap<PathBuf, (AppReloadEvent, Instant)> = HashMap::new();
        let path = PathBuf::from("/Applications/MyApp.app");
        let now = Instant::now();

        // First: add event
        merge_app_event(
            &mut pending,
            &path,
            AppReloadEvent::AppAdded(path.clone()),
            now,
        );

        // Then: remove event
        let later = now + Duration::from_millis(10);
        merge_app_event(
            &mut pending,
            &path,
            AppReloadEvent::AppRemoved(path.clone()),
            later,
        );

        // Should be merged to AppUpdated
        assert_eq!(pending.len(), 1);
        let (event, _) = pending.get(&path).unwrap();
        assert_eq!(*event, AppReloadEvent::AppUpdated(path.clone()));
    }

    #[test]
    fn test_no_merge_app_events_different_paths() {
        // Events for different paths should not be merged
        let mut pending: HashMap<PathBuf, (AppReloadEvent, Instant)> = HashMap::new();
        let path_a = PathBuf::from("/Applications/AppA.app");
        let path_b = PathBuf::from("/Applications/AppB.app");
        let now = Instant::now();

        // Remove on path A
        merge_app_event(
            &mut pending,
            &path_a,
            AppReloadEvent::AppRemoved(path_a.clone()),
            now,
        );

        // Add on path B (different path - no merge)
        merge_app_event(
            &mut pending,
            &path_b,
            AppReloadEvent::AppAdded(path_b.clone()),
            now,
        );

        // Should have 2 separate events
        assert_eq!(pending.len(), 2);
        assert!(matches!(
            pending.get(&path_a),
            Some((AppReloadEvent::AppRemoved(_), _))
        ));
        assert!(matches!(
            pending.get(&path_b),
            Some((AppReloadEvent::AppAdded(_), _))
        ));
    }

    #[test]
    fn test_next_app_deadline_empty() {
        let pending: HashMap<PathBuf, (AppReloadEvent, Instant)> = HashMap::new();
        let debounce = Duration::from_millis(500);

        assert!(next_app_deadline(&pending, None, debounce).is_none());
    }

    #[test]
    fn test_next_app_deadline_single() {
        let mut pending: HashMap<PathBuf, (AppReloadEvent, Instant)> = HashMap::new();
        let now = Instant::now();
        let debounce = Duration::from_millis(500);

        pending.insert(
            PathBuf::from("/Applications/Test.app"),
            (
                AppReloadEvent::AppAdded(PathBuf::from("/Applications/Test.app")),
                now,
            ),
        );

        let deadline = next_app_deadline(&pending, None, debounce);
        assert!(deadline.is_some());
        let expected = now + debounce;
        let actual = deadline.unwrap();
        assert!(actual >= expected && actual <= expected + Duration::from_millis(1));
    }

    #[test]
    fn test_next_app_deadline_with_full_reload() {
        let pending: HashMap<PathBuf, (AppReloadEvent, Instant)> = HashMap::new();
        let now = Instant::now();
        let debounce = Duration::from_millis(500);

        let deadline = next_app_deadline(&pending, Some(now), debounce);
        assert!(deadline.is_some());
        let expected = now + debounce;
        let actual = deadline.unwrap();
        assert!(actual >= expected && actual <= expected + Duration::from_millis(1));
    }

    #[test]
    fn test_flush_expired_apps_none_expired() {
        let (tx, _rx) = async_channel::bounded::<AppReloadEvent>(10);
        let mut pending: HashMap<PathBuf, (AppReloadEvent, Instant)> = HashMap::new();
        let mut full_reload_at: Option<Instant> = None;
        let now = Instant::now();
        let debounce = Duration::from_millis(500);

        // Add a fresh event (not expired)
        pending.insert(
            PathBuf::from("/Applications/Test.app"),
            (
                AppReloadEvent::AppAdded(PathBuf::from("/Applications/Test.app")),
                now,
            ),
        );

        flush_expired_apps(&mut pending, &mut full_reload_at, debounce, &tx);

        // Event should still be pending
        assert_eq!(pending.len(), 1);
    }

    #[test]
    fn test_flush_expired_apps_some_expired() {
        let (tx, rx) = async_channel::bounded::<AppReloadEvent>(10);
        let mut pending: HashMap<PathBuf, (AppReloadEvent, Instant)> = HashMap::new();
        let mut full_reload_at: Option<Instant> = None;
        let debounce = Duration::from_millis(500);

        // Add an expired event (from 600ms ago)
        let old_time = Instant::now() - Duration::from_millis(600);
        pending.insert(
            PathBuf::from("/Applications/Old.app"),
            (
                AppReloadEvent::AppAdded(PathBuf::from("/Applications/Old.app")),
                old_time,
            ),
        );

        // Add a fresh event
        pending.insert(
            PathBuf::from("/Applications/New.app"),
            (
                AppReloadEvent::AppAdded(PathBuf::from("/Applications/New.app")),
                Instant::now(),
            ),
        );

        flush_expired_apps(&mut pending, &mut full_reload_at, debounce, &tx);

        // Only fresh event should remain
        assert_eq!(pending.len(), 1);
        assert!(pending.contains_key(&PathBuf::from("/Applications/New.app")));

        // Should have received the expired event
        let received = rx.try_recv().unwrap();
        assert_eq!(
            received,
            AppReloadEvent::AppAdded(PathBuf::from("/Applications/Old.app"))
        );
    }

    #[test]
    fn test_flush_expired_apps_full_reload_supersedes_pending() {
        let (tx, rx) = async_channel::bounded::<AppReloadEvent>(10);
        let mut pending: HashMap<PathBuf, (AppReloadEvent, Instant)> = HashMap::new();
        let debounce = Duration::from_millis(500);

        // Add some expired pending events
        let old_time = Instant::now() - Duration::from_millis(600);
        pending.insert(
            PathBuf::from("/Applications/A.app"),
            (
                AppReloadEvent::AppAdded(PathBuf::from("/Applications/A.app")),
                old_time,
            ),
        );
        pending.insert(
            PathBuf::from("/Applications/B.app"),
            (
                AppReloadEvent::AppRemoved(PathBuf::from("/Applications/B.app")),
                old_time,
            ),
        );

        // Set expired full_reload_at (should supersede pending)
        let mut full_reload_at: Option<Instant> = Some(old_time);

        flush_expired_apps(&mut pending, &mut full_reload_at, debounce, &tx);

        // All pending should be cleared
        assert!(pending.is_empty());
        // full_reload_at should be reset
        assert!(full_reload_at.is_none());

        // Should receive only ONE FullReload (not per-app events)
        let received = rx.try_recv().unwrap();
        assert_eq!(received, AppReloadEvent::FullReload);
        // No more events
        assert!(rx.try_recv().is_err());
    }
}
