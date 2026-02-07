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
