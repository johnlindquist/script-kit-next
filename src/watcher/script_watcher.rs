use super::{
    discover_kit_watch_paths, is_relevant_script_file, load_watcher_settings, merge_script_event,
    next_deadline, to_generic_watcher_settings, GenericWatcher, ScriptReloadEvent, WatcherSettings,
    WatcherSpec,
};
use notify::{RecursiveMode, Result as NotifyResult, Watcher};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

const IDLE_DEADLINE_SECS: u64 = 24 * 60 * 60;

type ScriptGenericWatcher =
    GenericWatcher<ScriptReloadEvent, Sender<ScriptReloadEvent>, ScriptWatcherSpec>;
type DiscoverKitWatchPaths = Arc<dyn Fn() -> super::KitWatchPaths + Send + Sync>;

/// Watches ~/.scriptkit/kit/*/scripts, ~/.scriptkit/kit/*/extensions, and
/// ~/.scriptkit/kit/*/agents directories for changes.
///
/// Uses per-file trailing-edge debounce with storm coalescing.
/// Includes supervisor restart with exponential backoff on transient errors.
/// Dynamically watches extensions and agents directories when they appear.
pub struct ScriptWatcher {
    watcher: ScriptGenericWatcher,
}

impl ScriptWatcher {
    /// Create a new ScriptWatcher.
    ///
    /// Returns a tuple of (watcher, receiver) where receiver will emit ScriptReloadEvent
    /// when files in the scripts directory change.
    pub fn new() -> (Self, Receiver<ScriptReloadEvent>) {
        let (tx, rx) = channel();
        let settings = load_watcher_settings();
        let spec = ScriptWatcherSpec::new(settings);
        let watcher = GenericWatcher::new(tx, spec, to_generic_watcher_settings(settings));

        (Self { watcher }, rx)
    }

    /// Start watching the scripts directory for changes.
    pub fn start(&mut self) -> NotifyResult<()> {
        self.watcher.start()
    }
}

pub(crate) struct ScriptWatcherSpec {
    debounce: Duration,
    storm_threshold: usize,
    pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)>,
    full_reload_at: Option<Instant>,
    kit_path: PathBuf,
    tracked_scripts_paths: HashSet<PathBuf>,
    tracked_extensions_paths: HashSet<PathBuf>,
    tracked_agents_paths: HashSet<PathBuf>,
    watching_scripts: HashSet<PathBuf>,
    watching_extensions: HashSet<PathBuf>,
    watching_agents: HashSet<PathBuf>,
    discover_paths: DiscoverKitWatchPaths,
}

impl ScriptWatcherSpec {
    fn new(settings: WatcherSettings) -> Self {
        Self::with_discover_paths(settings, Arc::new(discover_kit_watch_paths))
    }

    fn with_discover_paths(
        settings: WatcherSettings,
        discover_paths: DiscoverKitWatchPaths,
    ) -> Self {
        Self {
            debounce: Duration::from_millis(settings.debounce_ms),
            storm_threshold: settings.storm_threshold,
            pending: HashMap::new(),
            full_reload_at: None,
            kit_path: PathBuf::from(shellexpand::tilde("~/.scriptkit/kit").as_ref()),
            tracked_scripts_paths: HashSet::new(),
            tracked_extensions_paths: HashSet::new(),
            tracked_agents_paths: HashSet::new(),
            watching_scripts: HashSet::new(),
            watching_extensions: HashSet::new(),
            watching_agents: HashSet::new(),
            discover_paths,
        }
    }

    fn register_kit_dirs(&mut self, kit_dir: &Path) {
        let scripts_dir = kit_dir.join("scripts");
        let extensions_dir = kit_dir.join("extensions");
        let agents_dir = kit_dir.join("agents");

        self.tracked_scripts_paths.insert(scripts_dir);
        self.tracked_extensions_paths.insert(extensions_dir);
        self.tracked_agents_paths.insert(agents_dir);
    }

    fn watch_path_if_exists(
        watcher: &mut dyn Watcher,
        path: &Path,
        recursive_mode: RecursiveMode,
        category: &'static str,
    ) -> bool {
        if !path.exists() {
            return false;
        }

        match watcher.watch(path, recursive_mode) {
            Ok(()) => {
                info!(
                    watcher = "scripts",
                    path = %path.display(),
                    recursive = recursive_mode == RecursiveMode::Recursive,
                    category,
                    "Script watcher started for directory"
                );
                true
            }
            Err(error) => {
                warn!(
                    watcher = "scripts",
                    path = %path.display(),
                    recursive = recursive_mode == RecursiveMode::Recursive,
                    category,
                    error = %error,
                    "Failed to watch directory"
                );
                false
            }
        }
    }

    fn watch_pending_tracked_dirs(&mut self, watcher: &mut dyn Watcher) {
        for scripts_path in self.tracked_scripts_paths.clone() {
            if self.watching_scripts.contains(&scripts_path) {
                continue;
            }

            if Self::watch_path_if_exists(
                watcher,
                scripts_path.as_path(),
                RecursiveMode::Recursive,
                "scripts",
            ) {
                self.watching_scripts.insert(scripts_path);
            }
        }

        for extensions_path in self.tracked_extensions_paths.clone() {
            if self.watching_extensions.contains(&extensions_path) {
                continue;
            }

            if Self::watch_path_if_exists(
                watcher,
                extensions_path.as_path(),
                RecursiveMode::Recursive,
                "extensions",
            ) {
                self.watching_extensions.insert(extensions_path);
            }
        }

        for agents_path in self.tracked_agents_paths.clone() {
            if self.watching_agents.contains(&agents_path) {
                continue;
            }

            if Self::watch_path_if_exists(
                watcher,
                agents_path.as_path(),
                RecursiveMode::Recursive,
                "agents",
            ) {
                self.watching_agents.insert(agents_path);
            }
        }
    }

    fn handle_new_kit_directories(&mut self, event: &notify::Event, watcher: &mut dyn Watcher) {
        if !matches!(event.kind, notify::EventKind::Create(_)) {
            return;
        }

        for path in &event.paths {
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with('.'))
            {
                continue;
            }

            let is_new_kit_dir = path
                .parent()
                .is_some_and(|parent| parent == self.kit_path.as_path())
                && path.is_dir();

            if !is_new_kit_dir {
                continue;
            }

            self.register_kit_dirs(path);
            self.watch_pending_tracked_dirs(watcher);
            info!(
                watcher = "scripts",
                kit_dir = %path.display(),
                "Detected new kit directory and registered nested watches"
            );
        }
    }

    fn drain_expired_events(&mut self) -> Vec<ScriptReloadEvent> {
        let now = Instant::now();

        if let Some(reload_time) = self.full_reload_at {
            if now.duration_since(reload_time) >= self.debounce {
                debug!(
                    watcher = "scripts",
                    "FullReload debounce complete, flushing"
                );
                self.full_reload_at = None;
                self.pending.clear();
                return vec![ScriptReloadEvent::FullReload];
            }
        }

        let mut events = Vec::new();
        self.pending.retain(|path, (event, timestamp)| {
            if now.duration_since(*timestamp) >= self.debounce {
                debug!(
                    watcher = "scripts",
                    path = %path.display(),
                    event = ?event,
                    "Script debounce complete, flushing"
                );
                events.push(event.clone());
                false
            } else {
                true
            }
        });

        events
    }

    fn process_script_event(&mut self, event: notify::Event) {
        let kind = event.kind;

        for path in &event.paths {
            if !is_relevant_script_file(path) {
                continue;
            }

            let now = Instant::now();

            match &kind {
                notify::EventKind::Create(_) => {
                    debug!(
                        watcher = "scripts",
                        path = %path.display(),
                        event_kind = ?kind,
                        "Script change detected (create), merging event"
                    );
                    merge_script_event(
                        &mut self.pending,
                        path,
                        ScriptReloadEvent::FileCreated(path.clone()),
                        now,
                    );
                }
                notify::EventKind::Modify(_) => {
                    debug!(
                        watcher = "scripts",
                        path = %path.display(),
                        event_kind = ?kind,
                        "Script change detected (modify), updating pending"
                    );
                    self.pending.insert(
                        path.clone(),
                        (ScriptReloadEvent::FileChanged(path.clone()), now),
                    );
                }
                notify::EventKind::Remove(_) => {
                    debug!(
                        watcher = "scripts",
                        path = %path.display(),
                        event_kind = ?kind,
                        "Script change detected (remove), merging event"
                    );
                    merge_script_event(
                        &mut self.pending,
                        path,
                        ScriptReloadEvent::FileDeleted(path.clone()),
                        now,
                    );
                }
                notify::EventKind::Access(_) => continue,
                _ => {
                    debug!(
                        watcher = "scripts",
                        path = %path.display(),
                        event_kind = ?kind,
                        "Unknown event kind, triggering global FullReload"
                    );
                    self.full_reload_at = Some(now);
                    self.pending.clear();
                }
            }

            if self.pending.len() >= self.storm_threshold {
                warn!(
                    watcher = "scripts",
                    pending_count = self.pending.len(),
                    threshold = self.storm_threshold,
                    "Event storm detected, collapsing to FullReload"
                );
                self.full_reload_at = Some(Instant::now());
                self.pending.clear();
            }
        }
    }
}

impl WatcherSpec<ScriptReloadEvent> for ScriptWatcherSpec {
    fn label(&self) -> &str {
        "scripts"
    }

    fn setup(&mut self, watcher: &mut dyn Watcher) -> NotifyResult<()> {
        self.pending.clear();
        self.full_reload_at = None;

        self.tracked_scripts_paths.clear();
        self.tracked_extensions_paths.clear();
        self.tracked_agents_paths.clear();
        self.watching_scripts.clear();
        self.watching_extensions.clear();
        self.watching_agents.clear();

        let paths = (self.discover_paths)();
        self.kit_path = paths.kit_path.clone();

        for scripts_path in paths.scripts_paths {
            self.tracked_scripts_paths.insert(scripts_path);
        }

        for extensions_path in paths.extensions_paths {
            if let Some(kit_dir) = extensions_path.parent() {
                self.register_kit_dirs(kit_dir);
            }
        }

        for agents_path in paths.agents_paths {
            if let Some(kit_dir) = agents_path.parent() {
                self.register_kit_dirs(kit_dir);
            }
        }

        if self.kit_path.exists() {
            if let Err(error) = watcher.watch(&self.kit_path, RecursiveMode::NonRecursive) {
                warn!(
                    watcher = "scripts",
                    path = %self.kit_path.display(),
                    error = %error,
                    "Failed to watch kit directory for new kit creation"
                );
            } else {
                debug!(
                    watcher = "scripts",
                    path = %self.kit_path.display(),
                    "Watching kit directory for new kits"
                );
            }
        }

        self.watch_pending_tracked_dirs(watcher);

        info!(
            watcher = "scripts",
            scripts_watching = self.watching_scripts.len(),
            extensions_watching = self.watching_extensions.len(),
            extensions_total = self.tracked_extensions_paths.len(),
            agents_watching = self.watching_agents.len(),
            agents_total = self.tracked_agents_paths.len(),
            "Script watcher setup complete"
        );

        Ok(())
    }

    fn next_deadline(&self) -> Instant {
        next_deadline(&self.pending, self.full_reload_at, self.debounce)
            .unwrap_or_else(|| Instant::now() + Duration::from_secs(IDLE_DEADLINE_SECS))
    }

    fn on_timeout(&mut self) -> Vec<ScriptReloadEvent> {
        self.drain_expired_events()
    }

    fn on_notify(&mut self, event: notify::Event) -> Vec<ScriptReloadEvent> {
        self.process_script_event(event);
        Vec::new()
    }

    fn on_notify_with_watcher(
        &mut self,
        event: notify::Event,
        watcher: &mut dyn Watcher,
    ) -> Vec<ScriptReloadEvent> {
        self.watch_pending_tracked_dirs(watcher);
        self.handle_new_kit_directories(&event, watcher);
        self.process_script_event(event);
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{ScriptWatcherSpec, WatcherSpec};
    use crate::watcher::{KitWatchPaths, ScriptReloadEvent, WatcherSettings};
    use notify::{Config, RecursiveMode, Result as NotifyResult, Watcher, WatcherKind};
    use parking_lot::Mutex;
    use std::collections::VecDeque;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tempfile::TempDir;

    #[derive(Default, Debug)]
    struct RecordingWatcher {
        watch_calls: Vec<(PathBuf, RecursiveMode)>,
    }

    impl Watcher for RecordingWatcher {
        fn new<F: notify::EventHandler>(_event_handler: F, _config: Config) -> NotifyResult<Self>
        where
            Self: Sized,
        {
            Ok(Self::default())
        }

        fn watch(&mut self, path: &Path, recursive_mode: RecursiveMode) -> NotifyResult<()> {
            self.watch_calls.push((path.to_path_buf(), recursive_mode));
            Ok(())
        }

        fn unwatch(&mut self, _path: &Path) -> NotifyResult<()> {
            Ok(())
        }

        fn kind() -> WatcherKind
        where
            Self: Sized,
        {
            WatcherKind::NullWatcher
        }
    }

    fn mk_paths(temp: &TempDir, kits: &[&str]) -> KitWatchPaths {
        let kit_path = temp.path().join("kit");
        fs::create_dir_all(&kit_path).expect("kit root should be created for test");

        let mut scripts_paths = Vec::new();
        let mut extensions_paths = Vec::new();
        let mut agents_paths = Vec::new();

        for kit_name in kits {
            let kit_dir = kit_path.join(kit_name);
            let scripts_dir = kit_dir.join("scripts");
            let extensions_dir = kit_dir.join("extensions");
            let agents_dir = kit_dir.join("agents");

            fs::create_dir_all(&scripts_dir).expect("scripts dir should be created for test");
            scripts_paths.push(scripts_dir);
            extensions_paths.push(extensions_dir);
            agents_paths.push(agents_dir);
        }

        KitWatchPaths {
            kit_path,
            scripts_paths,
            extensions_paths,
            agents_paths,
        }
    }

    #[test]
    fn test_setup_rediscovers_paths_when_restarted() {
        let temp = TempDir::new().expect("tempdir should be created");
        let first = mk_paths(&temp, &["alpha"]);
        let second = mk_paths(&temp, &["alpha", "beta"]);
        let second_beta_extensions = second.kit_path.join("beta").join("extensions");
        let second_beta_agents = second.kit_path.join("beta").join("agents");

        let discover_calls = Arc::new(AtomicUsize::new(0));
        let queue = Arc::new(Mutex::new(VecDeque::from(vec![first, second])));

        let discover = {
            let discover_calls = Arc::clone(&discover_calls);
            let queue = Arc::clone(&queue);
            Arc::new(move || {
                discover_calls.fetch_add(1, Ordering::Relaxed);
                queue
                    .lock()
                    .pop_front()
                    .expect("test queue should have discover result")
            })
        };

        let mut spec = ScriptWatcherSpec::with_discover_paths(WatcherSettings::default(), discover);
        let mut watcher = RecordingWatcher::default();

        spec.setup(&mut watcher)
            .expect("first setup should register discovered paths");
        spec.setup(&mut watcher)
            .expect("second setup should rediscover paths");

        assert_eq!(discover_calls.load(Ordering::Relaxed), 2);
        assert!(spec
            .tracked_extensions_paths
            .contains(&second_beta_extensions));
        assert!(spec.tracked_agents_paths.contains(&second_beta_agents));
    }

    #[test]
    fn test_on_notify_with_watcher_registers_new_kit_directories() {
        let temp = TempDir::new().expect("tempdir should be created");
        let kit_root = temp.path().join("kit");
        fs::create_dir_all(&kit_root).expect("kit root should be created");

        let discover = {
            let kit_root = kit_root.clone();
            Arc::new(move || KitWatchPaths {
                kit_path: kit_root.clone(),
                scripts_paths: Vec::new(),
                extensions_paths: Vec::new(),
                agents_paths: Vec::new(),
            })
        };

        let mut spec = ScriptWatcherSpec::with_discover_paths(WatcherSettings::default(), discover);
        let mut watcher = RecordingWatcher::default();
        spec.setup(&mut watcher)
            .expect("setup should watch the kit root");

        let new_kit_dir = kit_root.join("fresh-kit");
        let new_scripts_dir = new_kit_dir.join("scripts");
        let new_extensions_dir = new_kit_dir.join("extensions");
        let new_agents_dir = new_kit_dir.join("agents");
        fs::create_dir_all(&new_scripts_dir).expect("scripts dir should be created for test");
        fs::create_dir_all(&new_extensions_dir).expect("extensions dir should be created for test");
        fs::create_dir_all(&new_agents_dir).expect("agents dir should be created for test");

        let event = notify::Event {
            kind: notify::EventKind::Create(notify::event::CreateKind::Folder),
            paths: vec![new_kit_dir],
            attrs: Default::default(),
        };

        let emitted = spec.on_notify_with_watcher(event, &mut watcher);
        assert!(emitted.is_empty());
        assert!(spec.watching_scripts.contains(&new_scripts_dir));
        assert!(spec.watching_extensions.contains(&new_extensions_dir));
        assert!(spec.watching_agents.contains(&new_agents_dir));
    }

    #[test]
    fn test_on_timeout_emits_full_reload_when_full_reload_debounce_expires() {
        let mut spec = ScriptWatcherSpec::new(WatcherSettings::default());
        spec.full_reload_at = Some(Instant::now() - Duration::from_secs(1));
        spec.pending.insert(
            PathBuf::from("/tmp/script.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/tmp/script.ts")),
                Instant::now() - Duration::from_secs(1),
            ),
        );

        let events = spec.on_timeout();
        assert_eq!(events, vec![ScriptReloadEvent::FullReload]);
        assert!(spec.pending.is_empty());
        assert!(spec.full_reload_at.is_none());
    }
}
