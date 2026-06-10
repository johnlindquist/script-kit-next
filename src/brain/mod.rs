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

pub mod curator;
pub mod download;
pub mod embedder;
pub mod inbox;
pub mod indexer;
pub mod launcher;
pub mod resources;
pub mod search;
pub mod seed;
pub mod store;
pub mod telegram;

#[cfg(test)]
mod tests;

use anyhow::Result;

pub use inbox::{
    count_open_inbox, insert_inbox_item, open_inbox_items, resolve_inbox_item,
    response_prompt_for_inbox_item, InboxItem, InboxKind,
};
pub use indexer::{ingest_chat_turn, start_brain_indexer, wake_indexer};
pub use launcher::{
    root_brain_inbox_subtitle, root_brain_query_is_eligible, search_root_brain_direct,
    search_root_brain_semantic, semantic_root_brain_hits_for_query, RootBrainInboxSectionOptions,
    RootBrainSearchHit, RootBrainSectionOptions,
};
pub use search::{brain_search, render_context_block, BrainHit};
pub use store::{init_brain_db, record_signal, DocSource};

/// Maximum characters of retrieved memory injected into a chat turn.
pub const BRAIN_CONTEXT_MAX_CHARS: usize = 4_000;

/// How many memories to stage per chat turn.
pub const BRAIN_CONTEXT_HITS: usize = 5;

/// Retrieve memory for a query and render it as a prompt-ready block.
/// Hybrid when the indexer's embedding model is warm (bounded by a hard
/// ~200ms budget), lexical+signals otherwise. Returns `None` when the brain
/// has nothing relevant.
pub fn recall_context_block(query: &str) -> Result<Option<String>> {
    let query = query.trim();
    if query.len() < 3 {
        return Ok(None);
    }
    let query_embedding = indexer::embed_query_within_budget(query);
    let hits = match &query_embedding {
        Some((model_id, vector)) => {
            brain_search(query, Some(vector), Some(model_id), BRAIN_CONTEXT_HITS)?
        }
        None => brain_search(query, None, None, BRAIN_CONTEXT_HITS)?,
    };
    if hits.is_empty() {
        return Ok(None);
    }
    let block = render_context_block(&hits, BRAIN_CONTEXT_MAX_CHARS);
    Ok((!block.is_empty()).then_some(block))
}

/// Timeout for one-shot pi completions (curator, Telegram bridge).
const PI_ONESHOT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);

/// One-shot `pi -p --no-tools` completion, shared by the curator and the
/// Telegram bridge. Uses the same binary resolution as Agent Chat; returns
/// `Ok(None)` when no pi binary is installed (callers degrade gracefully).
pub(crate) fn pi_oneshot(prompt: &str) -> Result<Option<String>> {
    let Some(pi_binary) = crate::ai::agent_chat::pi::binary::default_pi_binary() else {
        return Ok(None);
    };
    run_pi_print(&pi_binary, prompt).map(Some)
}

fn run_pi_print(pi_binary: &std::path::Path, prompt: &str) -> Result<String> {
    use anyhow::Context as _;
    let mut child = std::process::Command::new(pi_binary)
        .args([
            "-p",
            "--no-tools",
            "--provider",
            crate::ai::agent_chat::profiles::DEFAULT_PI_PROVIDER,
            "--model",
            crate::ai::agent_chat::profiles::DEFAULT_PI_MODEL,
            prompt,
        ])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("spawn brain pi")?;
    let deadline = std::time::Instant::now() + PI_ONESHOT_TIMEOUT;
    loop {
        match child.try_wait().context("brain pi wait")? {
            Some(status) => {
                let mut output = String::new();
                if let Some(mut stdout) = child.stdout.take() {
                    use std::io::Read as _;
                    let _ = stdout.read_to_string(&mut output);
                }
                if !status.success() {
                    anyhow::bail!("brain pi exited with {status}");
                }
                return Ok(output);
            }
            None if std::time::Instant::now() > deadline => {
                let _ = child.kill();
                anyhow::bail!("brain pi timed out");
            }
            None => std::thread::sleep(std::time::Duration::from_millis(250)),
        }
    }
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
            let _ = store::append_activity(&format!(
                "in the launcher, searched \"{query}\" and chose {selected}"
            ));
        });
}

/// Record a deliberate `;` capture — the single strongest attention signal
/// the launcher emits (the user stopped what they were doing to write this
/// down). Topic + tag signals at chat-turn weight, an activity-journal line,
/// and an immediate indexer wake so the captured content becomes brain-
/// searchable without waiting for the next timer cycle. Fire-and-forget
/// off-thread; never blocks the capture path.
pub fn record_capture_signals(target: &str, body: &str, tags: &[String]) {
    let target = target.to_string();
    let body = body.to_string();
    let tags = tags.to_vec();
    let _ = std::thread::Builder::new()
        .name("script-kit-brain-capture".to_string())
        .spawn(move || {
            for topic in indexer::extract_topics(&body) {
                let _ = record_signal(&topic, 2, "capture");
            }
            for tag in &tags {
                let _ = record_signal(tag, 2, "capture");
            }
            let excerpt: String = body.chars().take(120).collect();
            let _ = store::append_activity(&format!("captured {target} \"{excerpt}\""));
            indexer::wake_indexer();
        });
}

/// Record that the user accepted a brain-grounded ghost suggestion in Notes
/// (attention signals + activity journal), so the memories that produced a
/// useful hint get reinforced. Fire-and-forget off-thread; never blocks the
/// editor input path.
pub fn record_ghost_accept_signals(line_prefix: &str, accepted_suffix: &str) {
    let line = format!("{line_prefix}{accepted_suffix}");
    let _ = std::thread::Builder::new()
        .name("script-kit-brain-ghost-accept".to_string())
        .spawn(move || {
            for topic in indexer::extract_topics(&line) {
                let _ = record_signal(&topic, 2, "ghost_accept");
            }
            let _ = store::append_activity(&format!(
                "in notes, accepted a brain ghost suggestion on the line \"{line}\""
            ));
        });
}

/// Record a user decision/action into today's activity journal — the brain's
/// answer to "what did I just do?". Fire-and-forget off-thread; never blocks
/// the action path. `kind` is a short verb phrase ("opened file",
/// "ran script"), `detail` the specifics.
pub fn record_activity(kind: &str, detail: &str) {
    let line = format!("{kind} {detail}");
    let _ = std::thread::Builder::new()
        .name("script-kit-brain-activity".to_string())
        .spawn(move || {
            if let Err(error) = store::append_activity(&line) {
                tracing::debug!(target: "script_kit::brain", error = %error, "activity append failed");
            }
        });
}
