use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use parking_lot::Mutex;

use crate::ai::agent_chat::runtime::AgentChatConnection;

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
}

impl WarmSlot {
    fn snapshot(&self) -> AgentChatWarmSessionSnapshot {
        AgentChatWarmSessionSnapshot {
            key: self.spec.key.clone(),
            generation: self.generation,
            ui_thread_id: self.ui_thread_id.clone(),
            state: self.state,
        }
    }
}

pub(crate) struct AgentChatWarmSessionManager {
    inner: Mutex<WarmSessionInner>,
    ui_thread_id_source: Arc<dyn Fn() -> String + Send + Sync>,
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
            inner: Mutex::new(WarmSessionInner::default()),
            ui_thread_id_source: Arc::new(default_warm_ui_thread_id),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_ui_thread_id_source(
        ui_thread_id_source: Arc<dyn Fn() -> String + Send + Sync>,
    ) -> Self {
        Self {
            inner: Mutex::new(WarmSessionInner::default()),
            ui_thread_id_source,
        }
    }

    pub(crate) fn prepare_warm(
        &self,
        spec: AgentChatWarmSessionSpec,
    ) -> Result<AgentChatWarmSessionSnapshot> {
        {
            let inner = self.inner.lock();
            if let Some(slot) = inner.slots.get(&spec.key) {
                if slot.state != AgentChatWarmSessionState::Failed {
                    return Ok(slot.snapshot());
                }
            }
        }

        let slot = self.prepare_new_slot(spec);
        let snapshot = slot.snapshot();
        self.inner.lock().slots.insert(snapshot.key.clone(), slot);
        Ok(snapshot)
    }

    pub(crate) fn acquire_warm(&self, key: &str) -> Option<AgentChatWarmSessionLease> {
        let mut inner = self.inner.lock();
        let slot = inner.slots.get_mut(key)?;
        if slot.state != AgentChatWarmSessionState::Ready {
            return None;
        }

        let connection = slot.connection.as_ref()?.clone();
        let ui_thread_id = slot.ui_thread_id.clone()?;
        slot.state = AgentChatWarmSessionState::Acquired;

        Some(AgentChatWarmSessionLease {
            key: key.to_string(),
            generation: slot.generation,
            ui_thread_id,
            cwd: slot.spec.cwd.clone(),
            connection,
        })
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

    fn prepare_new_slot(&self, spec: AgentChatWarmSessionSpec) -> WarmSlot {
        let generation = {
            let mut inner = self.inner.lock();
            inner.next_generation += 1;
            inner.next_generation
        };
        let ui_thread_id = (self.ui_thread_id_source)();

        match spec.factory.spawn_connection() {
            Ok(connection) => {
                let state = if connection
                    .prepare_session(ui_thread_id.clone(), spec.cwd.clone())
                    .is_ok()
                {
                    AgentChatWarmSessionState::Ready
                } else {
                    AgentChatWarmSessionState::Failed
                };

                WarmSlot {
                    spec,
                    generation,
                    ui_thread_id: Some(ui_thread_id),
                    state,
                    connection: (state == AgentChatWarmSessionState::Ready).then_some(connection),
                }
            }
            Err(_) => WarmSlot {
                spec,
                generation,
                ui_thread_id: Some(ui_thread_id),
                state: AgentChatWarmSessionState::Failed,
                connection: None,
            },
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

    #[derive(Default)]
    struct RecordingConnection {
        prepare_calls: Mutex<Vec<(String, PathBuf)>>,
        cancel_calls: Mutex<Vec<String>>,
        fail_prepare: bool,
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
            let (_tx, rx) = async_channel::bounded(1);
            Ok(rx)
        }
    }

    #[derive(Default)]
    struct RecordingFactory {
        spawned: Mutex<Vec<Arc<RecordingConnection>>>,
        fail_next_prepare: Mutex<bool>,
    }

    impl RecordingFactory {
        fn spawned(&self) -> Vec<Arc<RecordingConnection>> {
            self.spawned.lock().clone()
        }

        fn fail_next_prepare(&self) {
            *self.fail_next_prepare.lock() = true;
        }
    }

    impl AgentChatWarmRuntimeFactory for RecordingFactory {
        fn spawn_connection(&self) -> Result<Arc<dyn AgentChatConnection>> {
            let fail_prepare = std::mem::take(&mut *self.fail_next_prepare.lock());
            let connection = Arc::new(RecordingConnection {
                fail_prepare,
                ..Default::default()
            });
            self.spawned.lock().push(connection.clone());
            Ok(connection)
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
    fn failed_prepare_marks_failed_and_next_prepare_retries() {
        let manager = manager();
        let factory = Arc::new(RecordingFactory::default());
        factory.fail_next_prepare();

        let failed = manager.prepare_warm(spec(factory.clone())).unwrap();
        assert_eq!(failed.state, AgentChatWarmSessionState::Failed);

        let retry = manager.prepare_warm(spec(factory.clone())).unwrap();
        assert_eq!(retry.state, AgentChatWarmSessionState::Ready);
        assert_eq!(retry.generation, 2);
        assert_eq!(factory.spawned().len(), 2);
    }
}
