//! Free launch previews via `md explain <flow> --json` (protocol §2).
//!
//! Explain is guaranteed engine-call-free, so the Lens variation can preview
//! the selected flow on every selection change. Cache key follows the
//! protocol: (path, mtimeMs, cwd, mdflow version, config fingerprint) — we
//! key on (path, mtimeMs, cwd) locally and let the fingerprint invalidate
//! server-side changes on refetch.

use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;

use parking_lot::Mutex;

use super::catalog::mdflow_binary;
use super::model::{ExplainInfo, FlowDescriptor, FLOW_UX_PROTOCOL_VERSION};

/// Keep the selected flow plus a small MRU set resolved (council guidance:
/// selected + top 3 MRU, no speculative fan-out).
const EXPLAIN_CACHE_CAPACITY: usize = 8;

#[derive(Clone, PartialEq, Eq, Hash)]
struct ExplainKey {
    path: String,
    mtime_ms: u64,
    cwd: String,
}

#[derive(Clone)]
pub enum ExplainState {
    Loading,
    Ready(Arc<ExplainInfo>),
    Failed(String),
}

static CACHE: Mutex<Option<Arc<ExplainCache>>> = Mutex::new(None);

pub fn explain_cache() -> Arc<ExplainCache> {
    let mut guard = CACHE.lock();
    guard
        .get_or_insert_with(|| Arc::new(ExplainCache::default()))
        .clone()
}

#[derive(Default)]
pub struct ExplainCache {
    entries: Mutex<HashMap<ExplainKey, ExplainState>>,
    mru: Mutex<Vec<ExplainKey>>,
    notify: Mutex<Option<Box<dyn Fn() + Send + Sync>>>,
}

impl ExplainCache {
    pub fn set_notify_hook(&self, hook: impl Fn() + Send + Sync + 'static) {
        *self.notify.lock() = Some(Box::new(hook));
    }

    fn notify(&self) {
        if let Some(hook) = self.notify.lock().as_ref() {
            hook();
        }
    }

    /// Non-blocking lookup; spawns a background resolve on miss. Renderers
    /// call this per frame for the selected flow.
    pub fn state_for(self: &Arc<Self>, flow: &FlowDescriptor, cwd: &str) -> ExplainState {
        let key = ExplainKey {
            path: flow.path.clone(),
            mtime_ms: flow.mtime_ms,
            cwd: cwd.to_string(),
        };
        self.touch_mru(&key);
        {
            let entries = self.entries.lock();
            if let Some(state) = entries.get(&key) {
                return state.clone();
            }
        }
        self.entries
            .lock()
            .insert(key.clone(), ExplainState::Loading);
        let cache = Arc::clone(self);
        std::thread::Builder::new()
            .name("flow-explain-fetch".into())
            .spawn(move || {
                let state = fetch_explain_blocking(&key.path, &key.cwd);
                cache.entries.lock().insert(key, state);
                cache.evict_beyond_capacity();
                cache.notify();
            })
            .ok();
        ExplainState::Loading
    }

    fn touch_mru(&self, key: &ExplainKey) {
        let mut mru = self.mru.lock();
        mru.retain(|k| k != key);
        mru.push(key.clone());
    }

    fn evict_beyond_capacity(&self) {
        let mru = self.mru.lock();
        if mru.len() <= EXPLAIN_CACHE_CAPACITY {
            return;
        }
        let keep: Vec<ExplainKey> = mru
            .iter()
            .rev()
            .take(EXPLAIN_CACHE_CAPACITY)
            .cloned()
            .collect();
        self.entries.lock().retain(|k, _| keep.contains(k));
    }
}

fn fetch_explain_blocking(path: &str, cwd: &str) -> ExplainState {
    let Some(binary) = mdflow_binary() else {
        return ExplainState::Failed("mdflow CLI not found on PATH".to_string());
    };
    let output = Command::new(binary)
        .arg("explain")
        .arg(path)
        .arg("--json")
        .current_dir(cwd)
        .output();
    let output = match output {
        Ok(output) => output,
        Err(err) => return ExplainState::Failed(format!("explain failed to spawn: {err}")),
    };
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let first = stderr.lines().next().unwrap_or("explain failed");
        return ExplainState::Failed(first.to_string());
    }
    parse_explain_output(&String::from_utf8_lossy(&output.stdout))
}

fn parse_explain_output(stdout: &str) -> ExplainState {
    match serde_json::from_str::<ExplainInfo>(stdout) {
        Ok(info) if info.protocol_version == FLOW_UX_PROTOCOL_VERSION => {
            ExplainState::Ready(Arc::new(info))
        }
        Ok(info) => ExplainState::Failed(format!(
            "unsupported explain protocol version {}",
            info.protocol_version
        )),
        Err(err) => ExplainState::Failed(format!("explain parse error: {err}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_explain_output_round_trips_protocol_v1() {
        let state = parse_explain_output(
            r#"{
                "protocolVersion": 1,
                "flowId": "project:review",
                "path": "/p/flows/review.md",
                "engine": "pi",
                "command": "pi",
                "args": ["--print"],
                "cwd": "/p",
                "prompt": "Review the diff",
                "promptTokensEstimate": 4,
                "inputs": [],
                "warnings": [],
                "configFingerprint": "sha256:abc"
            }"#,
        );
        match state {
            ExplainState::Ready(info) => {
                assert_eq!(info.engine, "pi");
                assert_eq!(info.args, vec!["--print".to_string()]);
                assert_eq!(info.config_fingerprint.as_deref(), Some("sha256:abc"));
            }
            _ => panic!("expected Ready"),
        }
    }

    #[test]
    fn parse_explain_output_rejects_future_protocol() {
        let state = parse_explain_output(
            r#"{"protocolVersion":9,"flowId":"x","path":"/p","engine":"pi","command":"pi","args":[],"cwd":"/p","prompt":"","promptTokensEstimate":0,"inputs":[],"warnings":[]}"#,
        );
        assert!(matches!(state, ExplainState::Failed(_)));
    }

    #[test]
    fn parse_explain_output_reports_garbage() {
        assert!(matches!(
            parse_explain_output("nope"),
            ExplainState::Failed(_)
        ));
    }
}
