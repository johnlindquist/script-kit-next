//! The Brain: Script Kit's built-in, fully local memory.
//!
//! Every install gets its own brain — no accounts, no servers, no setup:
//!
//! - **Store** ([`store`]): `~/.scriptkit/db/brain.sqlite` — documents
//!   normalized from notes, chat turns, and (over time) other sensors, with
//!   FTS5 lexical search, embedded vectors, and an append-only attention
//!   signal log.
//! - **Embedder** ([`embedder`]): sentence embeddings via the existing
//!   `script-kit-ghost-llm-helper` subprocess (llama.cpp, Metal). Drop a GGUF
//!   embedding model in `~/.scriptkit/models/brain/` and semantic search
//!   lights up; without one the brain runs lexical-only.
//! - **Search** ([`search`]): hybrid BM25 + cosine fused with RRF, boosted by
//!   recent attention signals — the qmd retrieval recipe, native.
//! - **Indexer** ([`indexer`]): a background thread that keeps the store and
//!   vectors current without ever blocking the UI.
//!
//! Privacy invariants: everything lives under `~/.scriptkit/`; nothing leaves
//! the machine except through Agent Chat sessions the user already runs; the
//! store is plain sqlite the user can inspect or delete.

pub mod embedder;
pub mod indexer;
pub mod resources;
pub mod search;
pub mod seed;
pub mod store;

#[cfg(test)]
mod tests;

use anyhow::Result;

pub use indexer::{ingest_chat_turn, start_brain_indexer, wake_indexer};
pub use search::{brain_search, render_context_block, BrainHit};
pub use store::{init_brain_db, record_signal, DocSource};

/// Maximum characters of retrieved memory injected into a chat turn.
pub const BRAIN_CONTEXT_MAX_CHARS: usize = 4_000;

/// How many memories to stage per chat turn.
pub const BRAIN_CONTEXT_HITS: usize = 5;

/// Retrieve memory for a query and render it as a prompt-ready block.
/// Lexical + signals only (fast path, no subprocess): used where latency is
/// user-facing. Returns `None` when the brain has nothing relevant.
pub fn recall_context_block(query: &str) -> Result<Option<String>> {
    let query = query.trim();
    if query.len() < 3 {
        return Ok(None);
    }
    let hits = brain_search(query, None, None, BRAIN_CONTEXT_HITS)?;
    if hits.is_empty() {
        return Ok(None);
    }
    let block = render_context_block(&hits, BRAIN_CONTEXT_MAX_CHARS);
    Ok((!block.is_empty()).then_some(block))
}

/// Record that the user asked the brain something (attention signal).
pub fn record_ask_signals(query: &str) {
    for topic in indexer::extract_topics(query) {
        let _ = record_signal(&topic, 2, "ask");
    }
}

/// Record a launcher search → selection pair (ambient learning). Spawned to a
/// short-lived thread so the input path never touches sqlite synchronously.
pub fn record_search_selection_signals(query: &str, selected_result_key: &str) {
    let query = query.to_string();
    let selected = selected_result_key.to_string();
    let _ = std::thread::Builder::new()
        .name("script-kit-brain-signal".to_string())
        .spawn(move || {
            for topic in indexer::extract_topics(&query) {
                let _ = record_signal(&topic, 1, "search");
            }
            // The chosen result's human-readable tail (after any kind prefix)
            // is itself a topic: choosing "script:kill-port" teaches the brain
            // that "kill-port" matters.
            let tail = selected.rsplit(':').next().unwrap_or(&selected);
            let _ = record_signal(&tail.replace(['-', '_'], " "), 2, "selection");
        });
}
