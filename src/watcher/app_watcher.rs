use super::{
    is_app_bundle, load_watcher_settings, merge_app_event, next_app_deadline, AppReloadEvent,
    GenericWatcher, GenericWatcherSettings, WatcherSettings, WatcherSpec,
};
use notify::{RecursiveMode, Result as NotifyResult, Watcher};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

const APP_WATCHER_CHANNEL_CAPACITY: usize = 100;
const IDLE_DEADLINE_SECS: u64 = 24 * 60 * 60;

/// Watches /Applications and ~/Applications for .app bundle changes.
///
/// Uses per-file trailing-edge debounce with storm coalescing.
/// Filters to only .app directories.
pub struct AppWatcher {
    watcher: GenericWatcher<AppReloadEvent, async_channel::Sender<AppReloadEvent>, AppWatcherSpec>,
}

impl AppWatcher {
    /// Create a new AppWatcher.
    ///
    /// Returns a tuple of (watcher, receiver) where receiver will emit AppReloadEvent
    /// when .app bundles in /Applications or ~/Applications change.
    pub fn new() -> (Self, async_channel::Receiver<AppReloadEvent>) {
        let (tx, rx) = async_channel::bounded(APP_WATCHER_CHANNEL_CAPACITY);
        let settings = load_watcher_settings();

        let spec = AppWatcherSpec::new(
            PathBuf::from("/Applications"),
            PathBuf::from(shellexpand::tilde("~/Applications").as_ref()),
            settings,
        );

        let watcher =
            GenericWatcher::new(tx, spec, generic_settings_from_watcher_settings(settings));

        (Self { watcher }, rx)
    }

    /// Start watching the applications directories for changes.
    pub fn start(&mut self) -> NotifyResult<()> {
        self.watcher.start()
    }
}

#[derive(Debug)]
pub(crate) struct AppWatcherSpec {
    system_apps_path: PathBuf,
    user_apps_path: PathBuf,
    debounce: Duration,
    storm_threshold: usize,
    pending: HashMap<PathBuf, (AppReloadEvent, Instant)>,
    full_reload_at: Option<Instant>,
}

impl AppWatcherSpec {
    fn new(system_apps_path: PathBuf, user_apps_path: PathBuf, settings: WatcherSettings) -> Self {
        Self {
            system_apps_path,
            user_apps_path,
            debounce: Duration::from_millis(settings.debounce_ms),
            storm_threshold: settings.storm_threshold,
            pending: HashMap::new(),
            full_reload_at: None,
        }
    }

    fn ensure_user_apps_dir_exists(path: &Path) {
        // Best-effort creation so a later app install under ~/Applications is always visible.
        if let Err(error) = fs::create_dir_all(path) {
            warn!(
                path = %path.display(),
                error = %error,
                "App watcher failed to pre-create user Applications directory; continuing"
            );
        }
    }

    fn drain_expired_events(&mut self) -> Vec<AppReloadEvent> {
        let now = Instant::now();

        if let Some(reload_time) = self.full_reload_at {
            if now.duration_since(reload_time) >= self.debounce {
                debug!("App FullReload debounce complete, flushing");
                self.full_reload_at = None;
                self.pending.clear();
                return vec![AppReloadEvent::FullReload];
            }
        }

        let mut events = Vec::new();
        self.pending.retain(|path, (event, timestamp)| {
            if now.duration_since(*timestamp) >= self.debounce {
                debug!(
                    path = %path.display(),
                    event = ?event,
                    "App debounce complete, flushing"
                );
                events.push(event.clone());
                false
            } else {
                true
            }
        });

        events
    }
}

impl WatcherSpec<AppReloadEvent> for AppWatcherSpec {
    fn label(&self) -> &str {
        "apps"
    }

    fn setup(&mut self, watcher: &mut dyn Watcher) -> NotifyResult<()> {
        if self.system_apps_path.exists() {
            watcher.watch(&self.system_apps_path, RecursiveMode::NonRecursive)?;
            info!(
                path = %self.system_apps_path.display(),
                recursive = false,
                "System Applications watcher started"
            );
        } else {
            debug!(
                path = %self.system_apps_path.display(),
                "System Applications path does not exist, skipping"
            );
        }

        // Keep this before `watch` so we don't permanently miss first-time ~/Applications installs.
        Self::ensure_user_apps_dir_exists(&self.user_apps_path);
        if self.user_apps_path.exists() {
            watcher.watch(&self.user_apps_path, RecursiveMode::NonRecursive)?;
            info!(
                path = %self.user_apps_path.display(),
                recursive = false,
                "User Applications watcher started"
            );
        } else {
            debug!(
                path = %self.user_apps_path.display(),
                "User Applications path does not exist after create_dir_all attempt, skipping"
            );
        }

        Ok(())
    }

    fn next_deadline(&self) -> Instant {
        next_app_deadline(&self.pending, self.full_reload_at, self.debounce)
            .unwrap_or_else(|| Instant::now() + Duration::from_secs(IDLE_DEADLINE_SECS))
    }

    fn on_timeout(&mut self) -> Vec<AppReloadEvent> {
        self.drain_expired_events()
    }

    fn on_notify(&mut self, event: notify::Event) -> Vec<AppReloadEvent> {
        for path in &event.paths {
            if !is_app_bundle(path) {
                continue;
            }

            let now = Instant::now();
            match &event.kind {
                notify::EventKind::Create(_) => {
                    debug!(
                        path = %path.display(),
                        event_kind = ?event.kind,
                        "App change detected (create), merging event"
                    );
                    merge_app_event(
                        &mut self.pending,
                        path,
                        AppReloadEvent::AppAdded(path.clone()),
                        now,
                    );
                }
                notify::EventKind::Modify(_) => {
                    debug!(
                        path = %path.display(),
                        event_kind = ?event.kind,
                        "App change detected (modify), updating pending"
                    );
                    self.pending.insert(
                        path.clone(),
                        (AppReloadEvent::AppUpdated(path.clone()), now),
                    );
                }
                notify::EventKind::Remove(_) => {
                    debug!(
                        path = %path.display(),
                        event_kind = ?event.kind,
                        "App change detected (remove), merging event"
                    );
                    merge_app_event(
                        &mut self.pending,
                        path,
                        AppReloadEvent::AppRemoved(path.clone()),
                        now,
                    );
                }
                notify::EventKind::Access(_) => continue,
                _ => {
                    debug!(
                        path = %path.display(),
                        event_kind = ?event.kind,
                        "Unknown event kind, triggering global FullReload"
                    );
                    self.full_reload_at = Some(now);
                    self.pending.clear();
                }
            }

            if self.pending.len() >= self.storm_threshold {
                warn!(
                    pending_count = self.pending.len(),
                    threshold = self.storm_threshold,
                    "App event storm detected, collapsing to FullReload"
                );
                self.full_reload_at = Some(Instant::now());
                self.pending.clear();
            }
        }

        Vec::new()
    }
}

fn generic_settings_from_watcher_settings(settings: WatcherSettings) -> GenericWatcherSettings {
    GenericWatcherSettings {
        initial_backoff_ms: settings.initial_backoff_ms,
        max_backoff_ms: settings.max_backoff_ms,
        max_notify_errors: settings.max_notify_errors,
        health_check_interval_ms: settings.health_check_interval_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::AppWatcherSpec;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_app_watcher_spec_creates_user_applications_dir_when_missing() {
        let test_root = std::env::temp_dir().join(format!(
            "script-kit-gpui-app-watcher-spec-{}",
            std::process::id()
        ));
        let user_apps_path = test_root.join("Applications");

        if user_apps_path.exists() {
            fs::remove_dir_all(&user_apps_path)
                .expect("test setup should remove stale user Applications dir");
        }

        assert!(!user_apps_path.exists());
        AppWatcherSpec::ensure_user_apps_dir_exists(&user_apps_path);
        assert!(user_apps_path.exists());

        if let Err(error) = fs::remove_dir_all(PathBuf::from(&test_root)) {
            if error.kind() != std::io::ErrorKind::NotFound {
                panic!("test cleanup should remove temp directory: {error}");
            }
        }
    }
}
