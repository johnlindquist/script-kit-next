use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use parking_lot::Mutex;

use crate::ai::agent_chat::events::{AgentChatEvent, AgentChatEventRx};
use crate::ai::agent_chat::runtime::AgentChatConnection;

const DEFAULT_STALE_ACQUIRED_TTL: Duration = Duration::from_secs(60);
const DEFAULT_PREPARE_READY_TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) trait AgentChatWarmRuntimeFactory: Send + Sync + 'static {
    fn spawn_connection(&self) -> Result<Arc<dyn AgentChatConnection>>;
}

impl<F> AgentChatWarmRuntimeFactory for F
where
    F: Fn() -> Result<Arc<dyn AgentChatConnection>> + Send + Sync + 'static,
{
    fn spawn_connection(&self) -> Result<Arc<dyn AgentChatConnection>> {
        self()
    }
}

#[derive(Clone)]
pub(crate) struct AgentChatWarmSessionSpec {
    pub key: String,
    pub cwd: PathBuf,
    pub factory: Arc<dyn AgentChatWarmRuntimeFactory>,
}

#[derive(Clone)]
pub(crate) struct AgentChatWarmSessionLease {
    pub key: String,
    pub generation: u64,
    pub ui_thread_id: String,
    pub cwd: PathBuf,
    pub connection: Arc<dyn AgentChatConnection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentChatWarmSessionSnapshot {
    pub key: String,
    pub generation: u64,
    pub ui_thread_id: Option<String>,
    pub state: AgentChatWarmSessionState,
    pub failure_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentChatWarmSessionState {
    Empty,
    Preparing,
    Ready,
    Acquired,
    Failed,
}

struct WarmSlot {
    spec: AgentChatWarmSessionSpec,
    generation: u64,
    ui_thread_id: Option<String>,
    state: AgentChatWarmSessionState,
    connection: Option<Arc<dyn AgentChatConnection>>,
    acquired_at: Option<Instant>,
    failure_message: Option<String>,
}

impl WarmSlot {
    fn snapshot(&self) -> AgentChatWarmSessionSnapshot {
        AgentChatWarmSessionSnapshot {
            key: self.spec.key.clone(),
            generation: self.generation,
            ui_thread_id: self.ui_thread_id.clone(),
            state: self.state,
            failure_message: self.failure_message.clone(),
        }
    }
}

pub(crate) struct AgentChatWarmSessionManager {
    inner: Arc<Mutex<WarmSessionInner>>,
    ui_thread_id_source: Arc<dyn Fn() -> String + Send + Sync>,
}

impl Clone for AgentChatWarmSessionManager {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            ui_thread_id_source: self.ui_thread_id_source.clone(),
        }
    }
}

#[derive(Default)]
struct WarmSessionInner {
    slots: HashMap<String, WarmSlot>,
    next_generation: u64,
}

impl Default for AgentChatWarmSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentChatWarmSessionManager {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(WarmSessionInner::default())),
            ui_thread_id_source: Arc::new(default_warm_ui_thread_id),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_ui_thread_id_source(
        ui_thread_id_source: Arc<dyn Fn() -> String + Send + Sync>,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(WarmSessionInner::default())),
            ui_thread_id_source,
        }
    }

    pub(crate) fn prepare_warm(
        &self,
        spec: AgentChatWarmSessionSpec,
    ) -> Result<AgentChatWarmSessionSnapshot> {
        self.cleanup_stale_acquired(DEFAULT_STALE_ACQUIRED_TTL);
        let (generation, ui_thread_id) = {
            let mut inner = self.inner.lock();
            if let Some(slot) = inner.slots.get(&spec.key) {
                if slot.state != AgentChatWarmSessionState::Failed {
                    return Ok(slot.snapshot());
                }
            }

            inner.next_generation += 1;
            let generation = inner.next_generation;
            let ui_thread_id = (self.ui_thread_id_source)();
            inner.slots.insert(
                spec.key.clone(),
                WarmSlot {
                    spec: spec.clone(),
                    generation,
                    ui_thread_id: Some(ui_thread_id.clone()),
                    state: AgentChatWarmSessionState::Preparing,
                    connection: None,
                    acquired_at: None,
                    failure_message: None,
                },
            );
            (generation, ui_thread_id)
        };

        let slot = self.prepare_slot_with_generation(spec, generation, ui_thread_id);
        let mut inner = self.inner.lock();
        let Some(current) = inner.slots.get(&slot.spec.key) else {
            let snapshot = slot.snapshot();
            inner.slots.insert(snapshot.key.clone(), slot);
            return Ok(snapshot);
        };
        if current.generation != generation || current.state != AgentChatWarmSessionState::Preparing
        {
            return Ok(current.snapshot());
        }

        let snapshot = slot.snapshot();
        inner.slots.insert(snapshot.key.clone(), slot);
        Ok(snapshot)
    }

    pub(crate) fn acquire_warm(&self, key: &str) -> Option<AgentChatWarmSessionLease> {
        self.cleanup_stale_acquired(DEFAULT_STALE_ACQUIRED_TTL);
        self.acquire_warm_ready(key)
    }

    pub(crate) fn prepare_warm_background(
        &self,
        spec: AgentChatWarmSessionSpec,
    ) -> Result<AgentChatWarmSessionSnapshot> {
        let (snapshot, should_spawn) = {
            let mut inner = self.inner.lock();
            if let Some(slot) = inner.slots.get(&spec.key) {
                if slot.state != AgentChatWarmSessionState::Failed {
                    return Ok(slot.snapshot());
                }
            }

            inner.next_generation += 1;
            let generation = inner.next_generation;
            let ui_thread_id = (self.ui_thread_id_source)();
            let slot = WarmSlot {
                spec: spec.clone(),
                generation,
                ui_thread_id: Some(ui_thread_id.clone()),
                state: AgentChatWarmSessionState::Preparing,
                connection: None,
                acquired_at: None,
                failure_message: None,
            };
            let snapshot = slot.snapshot();
            inner.slots.insert(spec.key.clone(), slot);
            (snapshot, (spec, generation, ui_thread_id))
        };

        let manager = self.clone();
        std::thread::Builder::new()
            .name("warm-prepare-background".to_string())
            .spawn(move || {
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "warm_background_thread_started",
                );
                let (spec, generation, ui_thread_id) = should_spawn;
                let slot = manager.prepare_slot_with_generation(spec, generation, ui_thread_id);
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "warm_background_thread_finished",
                    state = ?slot.state,
                );
                manager.insert_prepared_slot_if_current(slot, generation);
            })
            .map_err(|error| {
                tracing::error!(
                    target: "script_kit::tab_ai",
                    event = "warm_background_thread_spawn_failed",
                    %error,
                );
            })
            .ok();

        Ok(snapshot)
    }

    pub(crate) fn acquire_warm_ready(&self, key: &str) -> Option<AgentChatWarmSessionLease> {
        let mut inner = self.inner.lock();
        let slot = inner.slots.get_mut(key)?;
        if slot.state != AgentChatWarmSessionState::Ready {
            return None;
        }

        let connection = slot.connection.as_ref()?.clone();
        let ui_thread_id = slot.ui_thread_id.clone()?;
        slot.state = AgentChatWarmSessionState::Acquired;
        slot.acquired_at = Some(Instant::now());

        Some(AgentChatWarmSessionLease {
            key: key.to_string(),
            generation: slot.generation,
            ui_thread_id,
            cwd: slot.spec.cwd.clone(),
            connection,
        })
    }

    pub(crate) fn dismiss_reset_background(
        &self,
        lease: AgentChatWarmSessionLease,
    ) -> AgentChatWarmSessionSnapshot {
        let key = lease.key.clone();
        let generation = lease.generation;
        let old_ui_thread_id = lease.ui_thread_id.clone();
        let old_connection = lease.connection.clone();

        let (snapshot, spec, replacement_generation, replacement_ui_thread_id) = {
            let mut inner = self.inner.lock();
            let Some(slot) = inner.slots.get(&key) else {
                return AgentChatWarmSessionSnapshot {
                    key,
                    generation,
                    ui_thread_id: None,
                    state: AgentChatWarmSessionState::Empty,
                    failure_message: None,
                };
            };

            if slot.generation != generation || slot.state != AgentChatWarmSessionState::Acquired {
                return slot.snapshot();
            }

            let spec = slot.spec.clone();
            inner.next_generation += 1;
            let replacement_generation = inner.next_generation;
            let replacement_ui_thread_id = (self.ui_thread_id_source)();
            let replacement = WarmSlot {
                spec: spec.clone(),
                generation: replacement_generation,
                ui_thread_id: Some(replacement_ui_thread_id.clone()),
                state: AgentChatWarmSessionState::Preparing,
                connection: None,
                acquired_at: None,
                failure_message: None,
            };
            let snapshot = replacement.snapshot();
            inner.slots.insert(key.clone(), replacement);
            (
                snapshot,
                spec,
                replacement_generation,
                replacement_ui_thread_id,
            )
        };

        let manager = self.clone();
        std::thread::spawn(move || {
            let _ = old_connection.cancel_turn(old_ui_thread_id);
            drop(lease);
            let slot = manager.prepare_slot_with_generation(
                spec,
                replacement_generation,
                replacement_ui_thread_id,
            );
            manager.insert_prepared_slot_if_current(slot, replacement_generation);
        });

        snapshot
    }

    pub(crate) fn dismiss_reset(
        &self,
        lease: AgentChatWarmSessionLease,
    ) -> Result<AgentChatWarmSessionSnapshot> {
        let spec = {
            let inner = self.inner.lock();
            let Some(slot) = inner.slots.get(&lease.key) else {
                return Ok(AgentChatWarmSessionSnapshot {
                    key: lease.key,
                    generation: lease.generation,
                    ui_thread_id: None,
                    state: AgentChatWarmSessionState::Empty,
                    failure_message: None,
                });
            };

            if slot.generation != lease.generation
                || slot.state != AgentChatWarmSessionState::Acquired
            {
                return Ok(slot.snapshot());
            }

            slot.spec.clone()
        };

        let _ = lease.connection.cancel_turn(lease.ui_thread_id.clone());
        drop(lease);

        let slot = self.prepare_new_slot(spec);
        let snapshot = slot.snapshot();
        self.inner.lock().slots.insert(snapshot.key.clone(), slot);
        Ok(snapshot)
    }

    pub(crate) fn snapshot(&self, key: &str) -> Option<AgentChatWarmSessionSnapshot> {
        self.inner.lock().slots.get(key).map(WarmSlot::snapshot)
    }

    pub(crate) fn cleanup_stale_acquired(
        &self,
        ttl: Duration,
    ) -> Vec<AgentChatWarmSessionSnapshot> {
        let now = Instant::now();
        let stale = {
            let inner = self.inner.lock();
            inner
                .slots
                .iter()
                .filter_map(|(key, slot)| {
                    if slot.state != AgentChatWarmSessionState::Acquired {
                        return None;
                    }
                    let acquired_at = slot.acquired_at?;
                    if now.duration_since(acquired_at) < ttl {
                        return None;
                    }
                    let connection = slot.connection.as_ref()?.clone();
                    let ui_thread_id = slot.ui_thread_id.clone()?;
                    Some((
                        key.clone(),
                        slot.spec.clone(),
                        AgentChatWarmSessionLease {
                            key: key.clone(),
                            generation: slot.generation,
                            ui_thread_id,
                            cwd: slot.spec.cwd.clone(),
                            connection,
                        },
                    ))
                })
                .collect::<Vec<_>>()
        };

        let mut snapshots = Vec::new();
        for (key, spec, lease) in stale {
            let _ = lease.connection.cancel_turn(lease.ui_thread_id.clone());
            drop(lease);
            let slot = self.prepare_new_slot(spec);
            let snapshot = slot.snapshot();
            self.inner.lock().slots.insert(key, slot);
            snapshots.push(snapshot);
        }
        snapshots
    }

    fn prepare_new_slot(&self, spec: AgentChatWarmSessionSpec) -> WarmSlot {
        let generation = {
            let mut inner = self.inner.lock();
            inner.next_generation += 1;
            inner.next_generation
        };
        let ui_thread_id = (self.ui_thread_id_source)();

        self.prepare_slot_with_generation(spec, generation, ui_thread_id)
    }

    fn insert_prepared_slot_if_current(&self, slot: WarmSlot, generation: u64) {
        let mut inner = self.inner.lock();
        let Some(current) = inner.slots.get(&slot.spec.key) else {
            let snapshot = slot.snapshot();
            inner.slots.insert(snapshot.key.clone(), slot);
            return;
        };
        if current.generation != generation || current.state != AgentChatWarmSessionState::Preparing
        {
            return;
        }

        inner.slots.insert(slot.spec.key.clone(), slot);
    }

    fn prepare_slot_with_generation(
        &self,
        spec: AgentChatWarmSessionSpec,
        generation: u64,
        ui_thread_id: String,
    ) -> WarmSlot {
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "warm_prepare_slot_begin",
            generation,
            key = %spec.key,
        );
        match spec.factory.spawn_connection() {
            Ok(connection) => {
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "warm_prepare_slot_connection_spawned",
                    generation,
                    key = %spec.key,
                );
                let (state, failure_message) = match connection
                    .prepare_session(ui_thread_id.clone(), spec.cwd.clone())
                {
                    Ok(events) => {
                        tracing::info!(
                            target: "script_kit::tab_ai",
                            event = "warm_prepare_slot_session_sent",
                            generation,
                            key = %spec.key,
                        );
                        match wait_for_prepare_ready(events, DEFAULT_PREPARE_READY_TIMEOUT) {
                            PrepareReadyOutcome::Ready => {
                                tracing::info!(
                                    target: "script_kit::tab_ai",
                                    event = "warm_prepare_slot_ready",
                                    generation,
                                    key = %spec.key,
                                );
                                (AgentChatWarmSessionState::Ready, None)
                            }
                            PrepareReadyOutcome::RuntimeFailed(error) => {
                                tracing::warn!(
                                    target: "script_kit::tab_ai",
                                    event = "warm_prepare_slot_runtime_failed",
                                    generation,
                                    key = %spec.key,
                                    error = %error,
                                );
                                (AgentChatWarmSessionState::Failed, Some(error))
                            }
                            PrepareReadyOutcome::Timeout => {
                                tracing::warn!(
                                    target: "script_kit::tab_ai",
                                    event = "warm_prepare_slot_timeout",
                                    generation,
                                    key = %spec.key,
                                );
                                (
                                    AgentChatWarmSessionState::Failed,
                                    Some(format!(
                                        "Pi Agent Chat model warm-up timed out after {} ms",
                                        DEFAULT_PREPARE_READY_TIMEOUT.as_millis()
                                    )),
                                )
                            }
                            PrepareReadyOutcome::Closed => {
                                tracing::warn!(
                                    target: "script_kit::tab_ai",
                                    event = "warm_prepare_slot_events_closed",
                                    generation,
                                    key = %spec.key,
                                );
                                (
                                    AgentChatWarmSessionState::Failed,
                                    Some(
                                        "Pi Agent Chat model warm-up stream closed before reporting available models"
                                            .to_string(),
                                    ),
                                )
                            }
                        }
                    }
                    Err(error) => {
                        tracing::warn!(
                            target: "script_kit::tab_ai",
                            event = "warm_prepare_slot_session_error",
                            generation,
                            key = %spec.key,
                            %error,
                        );
                        (
                            AgentChatWarmSessionState::Failed,
                            Some(format!(
                                "Failed to prepare Pi Agent Chat session: {error:#}"
                            )),
                        )
                    }
                };

                WarmSlot {
                    spec,
                    generation,
                    ui_thread_id: Some(ui_thread_id),
                    state,
                    connection: (state == AgentChatWarmSessionState::Ready).then_some(connection),
                    acquired_at: None,
                    failure_message,
                }
            }
            Err(error) => {
                tracing::warn!(
                    target: "script_kit::tab_ai",
                    event = "warm_prepare_slot_spawn_failed",
                    generation,
                    key = %spec.key,
                    %error,
                );
                WarmSlot {
                    spec,
                    generation,
                    ui_thread_id: Some(ui_thread_id),
                    state: AgentChatWarmSessionState::Failed,
                    connection: None,
                    acquired_at: None,
                    failure_message: Some(format!(
                        "Failed to spawn Pi Agent Chat runtime: {error:#}"
                    )),
                }
            }
        }
    }

    #[cfg(test)]
    fn mark_acquired_as_stale_for_test(&self, key: &str, age: Duration) {
        let mut inner = self.inner.lock();
        if let Some(slot) = inner.slots.get_mut(key) {
            slot.acquired_at = Some(Instant::now() - age);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PrepareReadyOutcome {
    Ready,
    RuntimeFailed(String),
    Timeout,
    Closed,
}

fn wait_for_prepare_ready(events: AgentChatEventRx, timeout: Duration) -> PrepareReadyOutcome {
    let deadline = Instant::now() + timeout;

    loop {
        match events.try_recv() {
            Ok(AgentChatEvent::ModelsAvailable { .. }) => return PrepareReadyOutcome::Ready,
            Ok(AgentChatEvent::Failed { error }) => {
                return PrepareReadyOutcome::RuntimeFailed(error);
            }
            Ok(AgentChatEvent::SetupRequired {
                reason,
                auth_methods,
            }) => {
                let detail = if auth_methods.is_empty() {
                    format!("Pi Agent Chat setup required: {reason}")
                } else {
                    format!(
                        "Pi Agent Chat setup required: {reason}. Available methods: {}",
                        auth_methods.join(", ")
                    )
                };
                return PrepareReadyOutcome::RuntimeFailed(detail);
            }
            Ok(_) => continue,
            Err(async_channel::TryRecvError::Empty) => {
                if Instant::now() >= deadline {
                    tracing::warn!(
                        target: "script_kit::tab_ai",
                        event = "agent_chat_warm_prepare_ready_timeout",
                        timeout_ms = timeout.as_millis(),
                    );
                    return PrepareReadyOutcome::Timeout;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(async_channel::TryRecvError::Closed) => return PrepareReadyOutcome::Closed,
        }
    }
}

fn default_warm_ui_thread_id() -> String {
    format!("warm:{}", uuid::Uuid::new_v4())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::agent_chat::events::AgentChatEventRx;
    use crate::ai::agent_chat::runtime::AgentChatTurnRequest;
    use parking_lot::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::thread;

    #[derive(Default)]
    struct RecordingConnection {
        prepare_calls: Mutex<Vec<(String, PathBuf)>>,
        cancel_calls: Mutex<Vec<String>>,
        fail_prepare: bool,
        prepare_event: Option<AgentChatEvent>,
    }

    impl RecordingConnection {
        fn prepare_calls(&self) -> Vec<(String, PathBuf)> {
            self.prepare_calls.lock().clone()
        }

        fn cancel_calls(&self) -> Vec<String> {
            self.cancel_calls.lock().clone()
        }
    }

    impl AgentChatConnection for RecordingConnection {
        fn start_turn(&self, _request: AgentChatTurnRequest) -> Result<AgentChatEventRx> {
            let (_tx, rx) = async_channel::bounded(1);
            Ok(rx)
        }

        fn cancel_turn(&self, ui_thread_id: String) -> Result<()> {
            self.cancel_calls.lock().push(ui_thread_id);
            Ok(())
        }

        fn prepare_session(&self, ui_thread_id: String, cwd: PathBuf) -> Result<AgentChatEventRx> {
            self.prepare_calls.lock().push((ui_thread_id, cwd));
            if self.fail_prepare {
                anyhow::bail!("prepare failed");
            }
            let (tx, rx) = async_channel::bounded(1);
            if let Some(event) = self.prepare_event.clone() {
                tx.send_blocking(event)?;
            }
            Ok(rx)
        }
    }

    #[derive(Default)]
    struct RecordingFactory {
        spawned: Mutex<Vec<Arc<RecordingConnection>>>,
        fail_next_prepare: Mutex<bool>,
        next_prepare_event: Mutex<Option<AgentChatEvent>>,
    }

    impl RecordingFactory {
        fn spawned(&self) -> Vec<Arc<RecordingConnection>> {
            self.spawned.lock().clone()
        }

        fn fail_next_prepare(&self) {
            *self.fail_next_prepare.lock() = true;
        }

        fn set_next_prepare_event(&self, event: Option<AgentChatEvent>) {
            *self.next_prepare_event.lock() = event;
        }
    }

    impl AgentChatWarmRuntimeFactory for RecordingFactory {
        fn spawn_connection(&self) -> Result<Arc<dyn AgentChatConnection>> {
            let fail_prepare = std::mem::take(&mut *self.fail_next_prepare.lock());
            let prepare_event = self.next_prepare_event.lock().take().unwrap_or_else(|| {
                AgentChatEvent::ModelsAvailable {
                    current_model_id: None,
                    models: Vec::new(),
                }
            });
            let connection = Arc::new(RecordingConnection {
                fail_prepare,
                prepare_event: Some(prepare_event),
                ..Default::default()
            });
            self.spawned.lock().push(connection.clone());
            Ok(connection)
        }
    }

    struct BlockingFactory {
        spawned: AtomicUsize,
        started_tx: Mutex<Option<mpsc::Sender<()>>>,
        release_rx: Mutex<mpsc::Receiver<()>>,
    }

    impl BlockingFactory {
        fn new(started_tx: mpsc::Sender<()>, release_rx: mpsc::Receiver<()>) -> Self {
            Self {
                spawned: AtomicUsize::new(0),
                started_tx: Mutex::new(Some(started_tx)),
                release_rx: Mutex::new(release_rx),
            }
        }

        fn spawned_count(&self) -> usize {
            self.spawned.load(Ordering::SeqCst)
        }
    }

    impl AgentChatWarmRuntimeFactory for BlockingFactory {
        fn spawn_connection(&self) -> Result<Arc<dyn AgentChatConnection>> {
            self.spawned.fetch_add(1, Ordering::SeqCst);
            if let Some(started_tx) = self.started_tx.lock().take() {
                let _ = started_tx.send(());
            }
            let _ = self.release_rx.lock().recv();
            Ok(Arc::new(RecordingConnection {
                prepare_event: Some(AgentChatEvent::ModelsAvailable {
                    current_model_id: None,
                    models: Vec::new(),
                }),
                ..Default::default()
            }))
        }
    }

    fn manager() -> AgentChatWarmSessionManager {
        let counter = Arc::new(AtomicUsize::new(0));
        AgentChatWarmSessionManager::with_ui_thread_id_source(Arc::new(move || {
            let next = counter.fetch_add(1, Ordering::SeqCst) + 1;
            format!("thread-{next}")
        }))
    }

    fn spec(factory: Arc<RecordingFactory>) -> AgentChatWarmSessionSpec {
        AgentChatWarmSessionSpec {
            key: "key-a".to_string(),
            cwd: PathBuf::from("/tmp/a"),
            factory,
        }
    }

    fn spec_with_factory(
        factory: Arc<dyn AgentChatWarmRuntimeFactory>,
    ) -> AgentChatWarmSessionSpec {
        AgentChatWarmSessionSpec {
            key: "key-a".to_string(),
            cwd: PathBuf::from("/tmp/a"),
            factory,
        }
    }

    #[test]
    fn prepare_warm_spawns_runtime_and_prepares_session_once() {
        let manager = manager();
        let factory = Arc::new(RecordingFactory::default());

        let snapshot = manager.prepare_warm(spec(factory.clone())).unwrap();

        assert_eq!(snapshot.state, AgentChatWarmSessionState::Ready);
        assert_eq!(snapshot.ui_thread_id.as_deref(), Some("thread-1"));
        assert_eq!(factory.spawned().len(), 1);
        assert_eq!(
            factory.spawned()[0].prepare_calls(),
            vec![("thread-1".to_string(), PathBuf::from("/tmp/a"))]
        );
    }

    #[test]
    fn prepare_warm_is_idempotent_for_ready_key() {
        let manager = manager();
        let factory = Arc::new(RecordingFactory::default());

        let first = manager.prepare_warm(spec(factory.clone())).unwrap();
        let second = manager.prepare_warm(spec(factory.clone())).unwrap();

        assert_eq!(first, second);
        assert_eq!(factory.spawned().len(), 1);
    }

    #[test]
    fn prepare_warm_reserves_preparing_slot_before_spawning_runtime() {
        let manager = Arc::new(manager());
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let factory = Arc::new(BlockingFactory::new(started_tx, release_rx));

        let first_manager = manager.clone();
        let first_factory: Arc<dyn AgentChatWarmRuntimeFactory> = factory.clone();
        let first_prepare = thread::spawn(move || {
            first_manager
                .prepare_warm(spec_with_factory(first_factory))
                .expect("first prepare")
        });

        started_rx.recv().expect("first prepare started spawning");

        let second_factory: Arc<dyn AgentChatWarmRuntimeFactory> = factory.clone();
        let second = manager
            .prepare_warm(spec_with_factory(second_factory))
            .expect("second prepare");

        assert_eq!(second.state, AgentChatWarmSessionState::Preparing);
        assert_eq!(second.ui_thread_id.as_deref(), Some("thread-1"));
        assert_eq!(factory.spawned_count(), 1);

        release_tx.send(()).expect("release first prepare");
        let first = first_prepare.join().expect("first prepare thread");
        assert_eq!(first.state, AgentChatWarmSessionState::Ready);
        assert_eq!(factory.spawned_count(), 1);
        assert_eq!(
            manager.snapshot("key-a").unwrap().state,
            AgentChatWarmSessionState::Ready
        );
    }

    #[test]
    fn prepare_warm_background_returns_preparing_without_waiting_for_runtime() {
        let manager = manager();
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let factory = Arc::new(BlockingFactory::new(started_tx, release_rx));

        let snapshot = manager
            .prepare_warm_background(spec_with_factory(factory.clone()))
            .expect("background prepare");

        assert_eq!(snapshot.state, AgentChatWarmSessionState::Preparing);
        started_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("background prepare started spawning");
        assert_eq!(factory.spawned_count(), 1);
        assert_eq!(
            manager.snapshot("key-a").unwrap().state,
            AgentChatWarmSessionState::Preparing
        );

        release_tx.send(()).expect("release background prepare");
        for _ in 0..100 {
            if manager.snapshot("key-a").unwrap().state == AgentChatWarmSessionState::Ready {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("background prepare did not mark slot ready");
    }

    #[test]
    fn prepare_warm_background_is_idempotent_while_preparing() {
        let manager = manager();
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let factory = Arc::new(BlockingFactory::new(started_tx, release_rx));

        let first = manager
            .prepare_warm_background(spec_with_factory(factory.clone()))
            .expect("first background prepare");
        started_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("background prepare started spawning");
        let second = manager
            .prepare_warm_background(spec_with_factory(factory.clone()))
            .expect("second background prepare");

        assert_eq!(first, second);
        assert_eq!(factory.spawned_count(), 1);

        release_tx.send(()).expect("release background prepare");
    }

    #[test]
    fn acquire_warm_returns_ready_session_and_marks_it_acquired() {
        let manager = manager();
        let factory = Arc::new(RecordingFactory::default());

        let prepared = manager.prepare_warm(spec(factory)).unwrap();
        let lease = manager.acquire_warm("key-a").unwrap();
        let snapshot = manager.snapshot("key-a").unwrap();

        assert_eq!(lease.key, "key-a");
        assert_eq!(lease.generation, prepared.generation);
        assert_eq!(lease.ui_thread_id, "thread-1");
        assert_eq!(snapshot.state, AgentChatWarmSessionState::Acquired);
    }

    #[test]
    fn acquire_warm_consumes_ready_session_exactly_once() {
        let manager = manager();
        let factory = Arc::new(RecordingFactory::default());

        manager.prepare_warm(spec(factory)).unwrap();
        assert!(manager.acquire_warm("key-a").is_some());
        assert!(manager.acquire_warm("key-a").is_none());
    }

    #[test]
    fn acquire_warm_misses_for_wrong_key_without_spawning() {
        let manager = manager();
        let factory = Arc::new(RecordingFactory::default());

        manager.prepare_warm(spec(factory.clone())).unwrap();
        assert!(manager.acquire_warm("key-b").is_none());
        assert_eq!(factory.spawned().len(), 1);
    }

    #[test]
    fn dismiss_reset_cancels_acquired_session() {
        let manager = manager();
        let factory = Arc::new(RecordingFactory::default());

        manager.prepare_warm(spec(factory.clone())).unwrap();
        let lease = manager.acquire_warm("key-a").unwrap();
        manager.dismiss_reset(lease).unwrap();

        assert_eq!(factory.spawned()[0].cancel_calls(), vec!["thread-1"]);
    }

    #[test]
    fn dismiss_reset_prepares_replacement_with_new_ui_thread_id() {
        let manager = manager();
        let factory = Arc::new(RecordingFactory::default());

        manager.prepare_warm(spec(factory.clone())).unwrap();
        let lease = manager.acquire_warm("key-a").unwrap();
        let replacement = manager.dismiss_reset(lease).unwrap();

        assert_eq!(replacement.state, AgentChatWarmSessionState::Ready);
        assert_eq!(replacement.generation, 2);
        assert_eq!(replacement.ui_thread_id.as_deref(), Some("thread-2"));
        assert_eq!(
            factory.spawned()[1].prepare_calls(),
            vec![("thread-2".to_string(), PathBuf::from("/tmp/a"))]
        );
    }

    #[test]
    fn dismiss_reset_ignores_stale_generation_without_double_replacement() {
        let manager = manager();
        let factory = Arc::new(RecordingFactory::default());

        manager.prepare_warm(spec(factory.clone())).unwrap();
        let lease = manager.acquire_warm("key-a").unwrap();
        let mut stale = lease.clone();
        stale.generation = 0;

        let stale_snapshot = manager.dismiss_reset(stale).unwrap();
        assert_eq!(stale_snapshot.state, AgentChatWarmSessionState::Acquired);
        assert_eq!(factory.spawned().len(), 1);

        manager.dismiss_reset(lease).unwrap();
        assert_eq!(factory.spawned().len(), 2);
    }

    #[test]
    fn cleanup_stale_acquired_cancels_and_prepares_replacement() {
        let manager = manager();
        let factory = Arc::new(RecordingFactory::default());

        manager.prepare_warm(spec(factory.clone())).unwrap();
        let lease = manager.acquire_warm("key-a").unwrap();
        assert_eq!(lease.ui_thread_id, "thread-1");
        manager.mark_acquired_as_stale_for_test("key-a", Duration::from_secs(120));

        let snapshots = manager.cleanup_stale_acquired(Duration::from_secs(60));

        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].state, AgentChatWarmSessionState::Ready);
        assert_eq!(snapshots[0].generation, 2);
        assert_eq!(factory.spawned()[0].cancel_calls(), vec!["thread-1"]);
        assert_eq!(
            factory.spawned()[1].prepare_calls(),
            vec![("thread-2".to_string(), PathBuf::from("/tmp/a"))]
        );
    }

    #[test]
    fn cleanup_stale_acquired_leaves_recent_acquired_session_alone() {
        let manager = manager();
        let factory = Arc::new(RecordingFactory::default());

        manager.prepare_warm(spec(factory.clone())).unwrap();
        let _lease = manager.acquire_warm("key-a").unwrap();

        let snapshots = manager.cleanup_stale_acquired(Duration::from_secs(60));

        assert!(snapshots.is_empty());
        assert_eq!(
            manager.snapshot("key-a").unwrap().state,
            AgentChatWarmSessionState::Acquired
        );
        assert_eq!(factory.spawned().len(), 1);
        assert!(factory.spawned()[0].cancel_calls().is_empty());
    }

    #[test]
    fn failed_prepare_marks_failed_and_next_prepare_retries() {
        let manager = manager();
        let factory = Arc::new(RecordingFactory::default());
        factory.fail_next_prepare();

        let failed = manager.prepare_warm(spec(factory.clone())).unwrap();
        assert_eq!(failed.state, AgentChatWarmSessionState::Failed);
        assert_eq!(
            failed.failure_message.as_deref(),
            Some("Failed to prepare Pi Agent Chat session: prepare failed")
        );

        let retry = manager.prepare_warm(spec(factory.clone())).unwrap();
        assert_eq!(retry.state, AgentChatWarmSessionState::Ready);
        assert_eq!(retry.generation, 2);
        assert_eq!(factory.spawned().len(), 2);
    }

    #[test]
    fn prepare_warm_requires_models_available_readiness_event() {
        let manager = manager();
        let factory = Arc::new(RecordingFactory::default());
        factory.set_next_prepare_event(Some(AgentChatEvent::TurnFinished {
            stop_reason: "ignored".to_string(),
        }));

        let snapshot = manager.prepare_warm(spec(factory.clone())).unwrap();

        assert_eq!(snapshot.state, AgentChatWarmSessionState::Failed);
        assert_eq!(
            snapshot.failure_message.as_deref(),
            Some("Pi Agent Chat model warm-up stream closed before reporting available models")
        );
        assert!(manager.acquire_warm("key-a").is_none());
    }

    #[test]
    fn prepare_warm_marks_failed_when_prepare_reports_setup_required() {
        let manager = manager();
        let factory = Arc::new(RecordingFactory::default());
        factory.set_next_prepare_event(Some(AgentChatEvent::SetupRequired {
            reason: "login required".to_string(),
            auth_methods: vec!["browser".to_string()],
        }));

        let snapshot = manager.prepare_warm(spec(factory.clone())).unwrap();

        assert_eq!(snapshot.state, AgentChatWarmSessionState::Failed);
        assert_eq!(
            snapshot.failure_message.as_deref(),
            Some("Pi Agent Chat setup required: login required. Available methods: browser")
        );
        assert_eq!(factory.spawned().len(), 1);
    }
}
