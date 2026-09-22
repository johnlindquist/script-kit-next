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

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum BrainHealthRead {
    NotYetIndexed,
    Available {
        snapshot: BrainHealth,
    },
    Unavailable {
        code: &'static str,
        diagnostic: String,
    },
}

pub fn read_health_checked() -> BrainHealthRead {
    let result = store::meta_get("health").and_then(|value| {
        value
            .map(|json| serde_json::from_str::<BrainHealth>(&json).map_err(anyhow::Error::from))
            .transpose()
    });
    match result {
        Ok(None) => BrainHealthRead::NotYetIndexed,
        Ok(Some(mut snapshot)) => {
            if let Some(error) = snapshot.last_error.take() {
                let diagnostic = crate::logging::log_private_user_value(&error).sha256;
                snapshot.last_error =
                    Some(format!("Brain indexing failed. Diagnostic: {diagnostic}"));
            }
            BrainHealthRead::Available { snapshot }
        }
        Err(error) => BrainHealthRead::Unavailable {
            code: "brain_health_unavailable",
            diagnostic: crate::logging::log_private_user_value(&error.to_string()).sha256,
        },
    }
}
