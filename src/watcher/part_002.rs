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
