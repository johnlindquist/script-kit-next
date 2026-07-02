//! Brain health snapshot: a visible answer to "is the brain healthy, and if
//! not, why?".
//!
//! The indexer records one [`BrainHealth`] snapshot at the end of every cycle
//! (success or failure) into the existing `brain_meta` KV under the `health`
//! key. The `kit://brain` status resource surfaces it so a user — or an agent
//! debugging the app — can see the last cycle's outcome, the last error, doc
//! counts, and whether recall is running semantic or lexical-only, instead of
//! errors being swallowed into a `tracing::warn!`.

use super::store;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BrainHealth {
    pub last_cycle_started_unix: Option<u64>,
    pub last_cycle_finished_unix: Option<u64>,
    pub last_cycle_ok: Option<bool>,
    pub last_error: Option<String>,
    pub docs_total: i64,
    pub docs_pending_embedding: i64,
    pub recall_mode: String, // "semantic" | "lexical-only"
    pub embedder_alive: bool,
}

/// Persist the latest health snapshot into the brain meta KV. Best-effort:
/// health recording must never take down the metabolism, so callers ignore
/// the error.
pub fn record_health(h: &BrainHealth) -> anyhow::Result<()> {
    let json = serde_json::to_string(h)?;
    store::meta_set("health", &json)
}

/// Read the latest recorded health snapshot, or `None` when no cycle has run
/// yet (or the value is unreadable/corrupt).
pub fn read_health() -> Option<BrainHealth> {
    let json = store::meta_get("health").ok().flatten()?;
    serde_json::from_str(&json).ok()
}
