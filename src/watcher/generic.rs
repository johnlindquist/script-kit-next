//! Shared watcher engine for concrete watcher implementations.
//!
//! This module provides:
//! - `WatcherSpec` for watcher-specific setup/debounce behavior
//! - `EventSink` to abstract sync/async channel senders
//! - `GenericWatcher` for lifecycle/supervisor/backoff handling

use crate::config;
use notify::{recommended_watcher, RecursiveMode, Result as NotifyResult, Watcher};
use parking_lot::Mutex;
use std::ffi::OsString;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Internal control messages for generic watcher threads.
#[derive(Debug)]
enum ControlMsg {
    /// Signal from notify callback with a file event.
    Notify(notify::Result<notify::Event>),
    /// Signal to stop the watcher thread immediately.
    Stop,
}

/// Shared runtime settings used by generic watcher lifecycle logic.
#[derive(Debug, Clone, Copy)]
pub struct GenericWatcherSettings {
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub max_notify_errors: u32,
    pub health_check_interval_ms: u64,
}

impl Default for GenericWatcherSettings {
    fn default() -> Self {
        Self {
            initial_backoff_ms: config::defaults::DEFAULT_WATCHER_INITIAL_BACKOFF_MS,
            max_backoff_ms: config::defaults::DEFAULT_WATCHER_MAX_BACKOFF_MS,
            max_notify_errors: config::defaults::DEFAULT_WATCHER_MAX_NOTIFY_ERRORS,
            health_check_interval_ms: config::defaults::DEFAULT_HEALTH_CHECK_INTERVAL_MS,
        }
    }
}

/// Shared debounce spec for watchers that observe one file and emit one reload event.
pub struct SingleFileReloadSpec<E>
where
    E: Clone + Send + 'static,
{
    label: String,
    watch_path: PathBuf,
    target_name: OsString,
    target_path: PathBuf,
    debounce: Duration,
    pending_deadline: Option<Instant>,
    emit_event: E,
}

impl<E> SingleFileReloadSpec<E>
where
    E: Clone + Send + 'static,
{
    pub fn new(
        label: impl Into<String>,
        target_path: PathBuf,
        debounce: Duration,
        emit_event: E,
    ) -> Self {
        let target_name = target_path
            .file_name()
            .map(std::ffi::OsStr::to_owned)
            .unwrap_or_default();
        let watch_path = target_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_path_buf();

        Self {
            label: label.into(),
            watch_path,
            target_name,
            target_path,
            debounce,
            pending_deadline: None,
            emit_event,
        }
    }
}

impl<E> WatcherSpec<E> for SingleFileReloadSpec<E>
where
    E: Clone + Send + 'static,
{
    fn label(&self) -> &str {
        &self.label
    }

    fn setup(&mut self, watcher: &mut dyn Watcher) -> NotifyResult<()> {
        watcher.watch(&self.watch_path, RecursiveMode::NonRecursive)?;
        info!(
            watcher = %self.label,
            watch_path = %self.watch_path.display(),
            target_path = %self.target_path.display(),
            "Single-file watcher started"
        );
        Ok(())
    }

    fn next_deadline(&self) -> Instant {
        self.pending_deadline
            .unwrap_or_else(|| Instant::now() + Duration::from_secs(24 * 60 * 60))
    }

    fn on_timeout(&mut self) -> Vec<E> {
        let Some(deadline) = self.pending_deadline else {
            return Vec::new();
        };

        if Instant::now() >= deadline {
            self.pending_deadline = None;
            debug!(
                watcher = %self.label,
                target_path = %self.target_path.display(),
                "Single-file debounce complete; emitting reload"
            );
            vec![self.emit_event.clone()]
        } else {
            Vec::new()
        }
    }

    fn on_notify(&mut self, event: notify::Event) -> Vec<E> {
        let touches_target = event.paths.iter().any(|path| {
            path.file_name()
                .map(|name| name == self.target_name.as_os_str())
                .unwrap_or(false)
        });

        if touches_target && is_relevant_event_kind(&event.kind) {
            self.pending_deadline = Some(Instant::now() + self.debounce);
            debug!(
                watcher = %self.label,
                target_path = %self.target_path.display(),
                event_kind = ?event.kind,
                "Single-file change detected; reset debounce deadline"
            );
        }

        Vec::new()
    }
}

/// Trait for watcher-specific behavior that runs on top of `GenericWatcher`.
///
/// Implementors own their debounce/pending state and produce output events to emit.
pub trait WatcherSpec<E>: Send + 'static
where
    E: Send + 'static,
{
    /// Human-readable label used in logs (e.g. "config", "scripts", "apps").
    fn label(&self) -> &str;

    /// Register all watch paths/modes on the provided notify watcher.
    fn setup(&mut self, watcher: &mut dyn Watcher) -> NotifyResult<()>;

    /// Return the next deadline for debounce flushing.
    ///
    /// Implementations should return an absolute deadline (`Instant`) in the future.
    fn next_deadline(&self) -> Instant;

    /// Called when a timeout window expires.
    ///
    /// Return events that should be emitted to the output sink.
    fn on_timeout(&mut self) -> Vec<E>;

    /// Called for each notify event from the OS watcher.
    ///
    /// Return events that should be emitted to the output sink.
    fn on_notify(&mut self, event: notify::Event) -> Vec<E>;
}

/// Unified error type for `EventSink`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventSinkError {
    /// The receiving side was disconnected.
    Disconnected,
}

/// Abstraction for sending watcher output events.
pub trait EventSink<E>: Clone + Send + 'static
where
    E: Send + 'static,
{
    fn send_event(&self, event: E) -> Result<(), EventSinkError>;
}

impl<E> EventSink<E> for Sender<E>
where
    E: Send + 'static,
{
    fn send_event(&self, event: E) -> Result<(), EventSinkError> {
        self.send(event).map_err(|_e| EventSinkError::Disconnected)
    }
}

impl<E> EventSink<E> for async_channel::Sender<E>
where
    E: Send + 'static,
{
    fn send_event(&self, event: E) -> Result<(), EventSinkError> {
        self.send_blocking(event)
            .map_err(|_e| EventSinkError::Disconnected)
    }
}

/// Generic watcher engine that owns thread lifecycle and supervision.
pub struct GenericWatcher<E, Tx, S>
where
    E: Send + 'static,
    Tx: EventSink<E>,
    S: WatcherSpec<E>,
{
    tx: Option<Tx>,
    spec: Option<S>,
    settings: GenericWatcherSettings,
    stop_flag: Option<Arc<AtomicBool>>,
    control_tx: Arc<Mutex<Option<Sender<ControlMsg>>>>,
    watcher_thread: Option<JoinHandle<()>>,
    watcher_label: String,
    _event_type: PhantomData<E>,
}

impl<E, Tx, S> GenericWatcher<E, Tx, S>
where
    E: Send + 'static,
    Tx: EventSink<E>,
    S: WatcherSpec<E>,
{
    /// Construct a new generic watcher.
    pub fn new(tx: Tx, spec: S, settings: GenericWatcherSettings) -> Self {
        Self {
            tx: Some(tx),
            watcher_label: spec.label().to_string(),
            spec: Some(spec),
            settings,
            stop_flag: None,
            control_tx: Arc::new(Mutex::new(None)),
            watcher_thread: None,
            _event_type: PhantomData,
        }
    }

    /// Start the watcher in a background thread.
    pub fn start(&mut self) -> NotifyResult<()> {
        let tx = self
            .tx
            .take()
            .ok_or_else(|| std::io::Error::other("watcher already started"))?;
        let spec = self
            .spec
            .take()
            .ok_or_else(|| std::io::Error::other("watcher already started"))?;

        let stop_flag = Arc::new(AtomicBool::new(false));
        let thread_stop_flag = Arc::clone(&stop_flag);
        self.stop_flag = Some(stop_flag);

        let control_tx = Arc::clone(&self.control_tx);
        let settings = self.settings;
        let watcher_label = self.watcher_label.clone();

        let thread_handle = thread::spawn(move || {
            Self::supervisor_loop(
                spec,
                tx,
                thread_stop_flag,
                control_tx,
                settings,
                watcher_label,
            );
        });

        self.watcher_thread = Some(thread_handle);
        Ok(())
    }

    fn supervisor_loop(
        mut spec: S,
        out_tx: Tx,
        stop_flag: Arc<AtomicBool>,
        control_tx: Arc<Mutex<Option<Sender<ControlMsg>>>>,
        settings: GenericWatcherSettings,
        watcher_label: String,
    ) {
        let mut attempt: u32 = 0;

        loop {
            if stop_flag.load(Ordering::Relaxed) {
                info!(watcher = %watcher_label, "Watcher supervisor stopping");
                break;
            }

            let (attempt_control_tx, attempt_control_rx) = channel::<ControlMsg>();
            {
                let mut slot = control_tx.lock();
                *slot = Some(attempt_control_tx.clone());
            }

            match Self::watch_loop(
                &mut spec,
                out_tx.clone(),
                attempt_control_rx,
                attempt_control_tx,
                Arc::clone(&stop_flag),
                settings,
                &watcher_label,
            ) {
                Ok(()) => {
                    info!(watcher = %watcher_label, "Watcher completed normally");
                    break;
                }
                Err(error) => {
                    if stop_flag.load(Ordering::Relaxed) {
                        break;
                    }

                    let backoff = compute_backoff(attempt, settings);
                    warn!(
                        watcher = %watcher_label,
                        attempt,
                        backoff_ms = backoff.as_millis(),
                        error = %error,
                        "Watcher loop failed; retrying with backoff"
                    );

                    if !interruptible_sleep(backoff, &stop_flag) {
                        break;
                    }
                    attempt = attempt.saturating_add(1);
                }
            }
        }

        {
            let mut slot = control_tx.lock();
            *slot = None;
        }

        info!(watcher = %watcher_label, "Watcher supervisor shut down");
    }

    fn watch_loop(
        spec: &mut S,
        out_tx: Tx,
        control_rx: Receiver<ControlMsg>,
        callback_tx: Sender<ControlMsg>,
        stop_flag: Arc<AtomicBool>,
        settings: GenericWatcherSettings,
        watcher_label: &str,
    ) -> NotifyResult<()> {
        let mut watcher: Box<dyn Watcher> = Box::new(recommended_watcher(move |res| {
            let _ = callback_tx.send(ControlMsg::Notify(res));
        })?);

        spec.setup(&mut *watcher)?;
        info!(watcher = %watcher_label, "Watcher loop started");

        let mut consecutive_errors: u32 = 0;
        let health_check_interval = Duration::from_millis(settings.health_check_interval_ms.max(1));

        loop {
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            let timeout = next_timeout(spec.next_deadline(), health_check_interval);

            let message = match control_rx.recv_timeout(timeout) {
                Ok(msg) => Some(msg),
                Err(RecvTimeoutError::Timeout) => {
                    let emitted = spec.on_timeout();
                    emit_events(&out_tx, emitted, watcher_label, "timeout");
                    continue;
                }
                Err(RecvTimeoutError::Disconnected) => break,
            };

            let Some(message) = message else { continue };

            match message {
                ControlMsg::Stop => {
                    info!(watcher = %watcher_label, "Watcher received stop signal");
                    break;
                }
                ControlMsg::Notify(Err(error)) => {
                    consecutive_errors = consecutive_errors.saturating_add(1);
                    warn!(
                        watcher = %watcher_label,
                        consecutive_errors,
                        error = %error,
                        "notify callback delivered error"
                    );

                    if consecutive_errors >= settings.max_notify_errors.max(1) {
                        warn!(
                            watcher = %watcher_label,
                            consecutive_errors,
                            max_notify_errors = settings.max_notify_errors.max(1),
                            "Watcher exceeded notify error budget; restarting"
                        );
                        return Err(notify::Error::generic(
                            "too many consecutive notify callback errors",
                        ));
                    }
                }
                ControlMsg::Notify(Ok(event)) => {
                    consecutive_errors = 0;
                    let emitted = spec.on_notify(event);
                    emit_events(&out_tx, emitted, watcher_label, "notify");
                }
            }
        }

        info!(watcher = %watcher_label, "Watcher loop shutting down");
        Ok(())
    }
}

impl<E, Tx, S> Drop for GenericWatcher<E, Tx, S>
where
    E: Send + 'static,
    Tx: EventSink<E>,
    S: WatcherSpec<E>,
{
    fn drop(&mut self) {
        if let Some(flag) = self.stop_flag.take() {
            flag.store(true, Ordering::Relaxed);
        }

        // Wake watch_loop immediately instead of waiting up to health_check_interval_ms.
        let maybe_control_tx = {
            let mut slot = self.control_tx.lock();
            slot.take()
        };

        if let Some(control_tx) = maybe_control_tx {
            if control_tx.send(ControlMsg::Stop).is_err() {
                debug!(
                    watcher = %self.watcher_label,
                    "Stop control channel already closed while dropping watcher"
                );
            }
        }

        if let Some(handle) = self.watcher_thread.take() {
            if handle.join().is_err() {
                warn!(
                    watcher = %self.watcher_label,
                    "Watcher thread panicked while joining during drop"
                );
            }
        }
    }
}

fn compute_backoff(attempt: u32, settings: GenericWatcherSettings) -> Duration {
    let initial = settings.initial_backoff_ms.max(1);
    let max = settings.max_backoff_ms.max(initial);
    let delay_ms = initial.saturating_mul(2u64.saturating_pow(attempt));
    Duration::from_millis(delay_ms.min(max))
}

fn interruptible_sleep(duration: Duration, stop_flag: &AtomicBool) -> bool {
    let check_interval = Duration::from_millis(100);
    let mut remaining = duration;

    while remaining > Duration::ZERO {
        if stop_flag.load(Ordering::Relaxed) {
            return false;
        }

        let sleep_time = remaining.min(check_interval);
        thread::sleep(sleep_time);
        remaining = remaining.saturating_sub(sleep_time);
    }

    true
}

fn next_timeout(next_deadline: Instant, health_check_interval: Duration) -> Duration {
    next_deadline
        .saturating_duration_since(Instant::now())
        .min(health_check_interval)
}

fn is_relevant_event_kind(kind: &notify::EventKind) -> bool {
    !matches!(kind, notify::EventKind::Access(_))
}

fn emit_events<E, Tx>(sink: &Tx, events: Vec<E>, watcher_label: &str, source: &'static str)
where
    E: Send + 'static,
    Tx: EventSink<E>,
{
    for event in events {
        if sink.send_event(event).is_err() {
            debug!(
                watcher = %watcher_label,
                source,
                "Dropping emitted event because receiver is disconnected"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::RecursiveMode;
    use std::sync::mpsc::channel;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestEvent {
        TimeoutTick,
    }

    struct IdleSpec {
        deadline: Instant,
    }

    impl IdleSpec {
        fn new(deadline: Instant) -> Self {
            Self { deadline }
        }
    }

    impl WatcherSpec<TestEvent> for IdleSpec {
        fn label(&self) -> &str {
            "idle-test"
        }

        fn setup(&mut self, _watcher: &mut dyn Watcher) -> NotifyResult<()> {
            Ok(())
        }

        fn next_deadline(&self) -> Instant {
            self.deadline
        }

        fn on_timeout(&mut self) -> Vec<TestEvent> {
            Vec::new()
        }

        fn on_notify(&mut self, _event: notify::Event) -> Vec<TestEvent> {
            Vec::new()
        }
    }

    struct TimeoutSpec {
        deadline: Instant,
        emitted: bool,
    }

    impl TimeoutSpec {
        fn new() -> Self {
            Self {
                deadline: Instant::now() + Duration::from_millis(5),
                emitted: false,
            }
        }
    }

    impl WatcherSpec<TestEvent> for TimeoutSpec {
        fn label(&self) -> &str {
            "timeout-test"
        }

        fn setup(&mut self, _watcher: &mut dyn Watcher) -> NotifyResult<()> {
            Ok(())
        }

        fn next_deadline(&self) -> Instant {
            self.deadline
        }

        fn on_timeout(&mut self) -> Vec<TestEvent> {
            self.deadline = Instant::now() + Duration::from_secs(60);
            if self.emitted {
                Vec::new()
            } else {
                self.emitted = true;
                vec![TestEvent::TimeoutTick]
            }
        }

        fn on_notify(&mut self, _event: notify::Event) -> Vec<TestEvent> {
            Vec::new()
        }
    }

    #[test]
    fn test_event_sink_mpsc_sender_sends_event() {
        let (tx, rx) = channel::<u8>();
        tx.send_event(7).expect("mpsc sender should accept event");
        assert_eq!(rx.recv().expect("receiver should get event"), 7);
    }

    #[test]
    fn test_event_sink_async_sender_sends_event() {
        let (tx, rx) = async_channel::bounded::<u8>(1);
        tx.send_event(9).expect("async sender should accept event");
        assert_eq!(rx.recv_blocking().expect("receiver should get event"), 9);
    }

    #[test]
    fn test_generic_watcher_emits_timeout_events() {
        let (tx, rx) = channel::<TestEvent>();
        let spec = TimeoutSpec::new();
        let settings = GenericWatcherSettings {
            health_check_interval_ms: 20,
            ..GenericWatcherSettings::default()
        };
        let mut watcher = GenericWatcher::new(tx, spec, settings);

        watcher.start().expect("watcher should start");
        let received = rx
            .recv_timeout(Duration::from_secs(1))
            .expect("timeout event should be emitted");
        assert_eq!(received, TestEvent::TimeoutTick);
    }

    #[test]
    fn test_generic_watcher_drop_sends_stop_without_waiting_for_health_tick() {
        let (tx, _rx) = channel::<TestEvent>();
        let spec = IdleSpec::new(Instant::now() + Duration::from_secs(3600));
        let settings = GenericWatcherSettings {
            health_check_interval_ms: 5_000,
            ..GenericWatcherSettings::default()
        };
        let mut watcher = GenericWatcher::new(tx, spec, settings);

        watcher.start().expect("watcher should start");
        thread::sleep(Duration::from_millis(50));
        let started_at = Instant::now();
        drop(watcher);

        assert!(
            started_at.elapsed() < Duration::from_millis(500),
            "drop should stop promptly without waiting for health check timeout"
        );
    }

    #[test]
    fn test_watcher_spec_setup_can_register_watch_paths() {
        struct SetupSpec;

        impl WatcherSpec<TestEvent> for SetupSpec {
            fn label(&self) -> &str {
                "setup-test"
            }

            fn setup(&mut self, watcher: &mut dyn Watcher) -> NotifyResult<()> {
                // Register current directory as a smoke test for setup plumbing.
                watcher.watch(std::path::Path::new("."), RecursiveMode::NonRecursive)
            }

            fn next_deadline(&self) -> Instant {
                Instant::now() + Duration::from_secs(30)
            }

            fn on_timeout(&mut self) -> Vec<TestEvent> {
                Vec::new()
            }

            fn on_notify(&mut self, _event: notify::Event) -> Vec<TestEvent> {
                Vec::new()
            }
        }

        let (tx, _rx) = channel::<TestEvent>();
        let spec = SetupSpec;
        let mut watcher = GenericWatcher::new(tx, spec, GenericWatcherSettings::default());

        watcher.start().expect("setup should run without error");
    }

    #[test]
    fn test_single_file_reload_spec_emits_once_after_timeout() {
        let target_path = PathBuf::from("/tmp/config.ts");
        let mut spec =
            SingleFileReloadSpec::new("config", target_path.clone(), Duration::ZERO, 7u8);

        let event = notify::Event {
            kind: notify::EventKind::Modify(notify::event::ModifyKind::Any),
            paths: vec![target_path],
            attrs: Default::default(),
        };

        assert!(spec.on_notify(event).is_empty());
        assert_eq!(spec.on_timeout(), vec![7u8]);
        assert!(spec.on_timeout().is_empty());
    }

    #[test]
    fn test_single_file_reload_spec_ignores_non_target_paths() {
        let mut spec = SingleFileReloadSpec::new(
            "config",
            PathBuf::from("/tmp/config.ts"),
            Duration::from_millis(1),
            7u8,
        );

        let event = notify::Event {
            kind: notify::EventKind::Modify(notify::event::ModifyKind::Any),
            paths: vec![PathBuf::from("/tmp/other.ts")],
            attrs: Default::default(),
        };

        assert!(spec.on_notify(event).is_empty());
        assert!(spec.on_timeout().is_empty());
    }

    #[test]
    fn test_single_file_reload_spec_ignores_access_events() {
        let target_path = PathBuf::from("/tmp/config.ts");
        let mut spec =
            SingleFileReloadSpec::new("config", target_path.clone(), Duration::ZERO, 7u8);

        let event = notify::Event {
            kind: notify::EventKind::Access(notify::event::AccessKind::Any),
            paths: vec![target_path],
            attrs: Default::default(),
        };

        assert!(spec.on_notify(event).is_empty());
        assert!(spec.on_timeout().is_empty());
    }
}
