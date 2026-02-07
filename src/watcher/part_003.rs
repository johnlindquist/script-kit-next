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
        let settings = load_watcher_settings();

        let stop_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let thread_stop_flag = stop_flag.clone();
        self.stop_flag = Some(stop_flag);

        // Watch paths: /Applications and ~/Applications
        let system_apps_path = PathBuf::from("/Applications");
        let user_apps_path = PathBuf::from(shellexpand::tilde("~/Applications").as_ref());

        let thread_handle = thread::spawn(move || {
            Self::supervisor_loop(system_apps_path, user_apps_path, tx, thread_stop_flag, settings);
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
        settings: WatcherSettings,
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
                settings,
            ) {
                Ok(()) => {
                    info!(watcher = "apps", "App watcher completed normally");
                    break;
                }
                Err(e) => {
                    if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                        break;
                    }

                    let backoff = compute_backoff(attempt, settings);
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
        settings: WatcherSettings,
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
        let debounce = Duration::from_millis(settings.debounce_ms);
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
                .unwrap_or(Duration::from_millis(settings.health_check_interval_ms));

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

                    if consecutive_errors >= settings.max_notify_errors {
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
                        if pending.len() >= settings.storm_threshold {
                            warn!(
                                pending_count = pending.len(),
                                threshold = settings.storm_threshold,
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
