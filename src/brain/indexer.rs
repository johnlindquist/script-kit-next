//! Brain indexer: the background metabolism.
//!
//! A single low-priority thread that, on a timer and on demand:
//! 1. syncs notes into `brain_docs` (the librarian's raw material),
//! 2. embeds docs whose vectors are missing or stale,
//! 3. keeps everything incremental — content hashes mean unchanged docs are
//!    never re-embedded.
//!
//! The indexer NEVER blocks UI: all work happens on its own thread, the
//! embedder is a subprocess, and each cycle processes a bounded batch.

use super::embedder::{resolve_embed_model, BrainEmbedder};
use super::store::{self, DocSource};
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::OnceLock;
use std::time::Duration;

const CYCLE_INTERVAL: Duration = Duration::from_secs(120);
const EMBED_BATCH: usize = 16;
const MAX_EMBED_PER_CYCLE: usize = 256;
/// Truncate doc text fed to the embedder; the index keeps full content.
const EMBED_TEXT_CAP: usize = 6_000;

static WAKE: OnceLock<Sender<()>> = OnceLock::new();
static STARTED: AtomicBool = AtomicBool::new(false);

/// Ask the indexer to run a cycle soon (e.g. after a chat turn ingests new
/// docs). Cheap; coalesces with pending wakes.
pub fn wake_indexer() {
    if let Some(tx) = WAKE.get() {
        let _ = tx.send(());
    }
}

/// Start the background indexer thread. Idempotent.
pub fn start_brain_indexer() {
    if STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    let (tx, rx) = mpsc::channel::<()>();
    let _ = WAKE.set(tx);
    let _ = std::thread::Builder::new()
        .name("script-kit-brain-indexer".to_string())
        .spawn(move || {
            // Let app startup settle before the first cycle.
            std::thread::sleep(Duration::from_secs(20));
            let mut embedder: Option<BrainEmbedder> = None;
            loop {
                if let Err(err) = run_cycle(&mut embedder) {
                    tracing::warn!(target: "script_kit::brain", error = %err, "brain index cycle failed");
                }
                match rx.recv_timeout(CYCLE_INTERVAL) {
                    Ok(()) | Err(RecvTimeoutError::Timeout) => continue,
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
        });
}

/// One full cycle: sync sources, then embed what's missing.
pub fn run_cycle(embedder: &mut Option<BrainEmbedder>) -> Result<()> {
    store::init_brain_db()?;
    let synced = sync_notes().unwrap_or_else(|err| {
        tracing::debug!(target: "script_kit::brain", error = %err, "notes sync skipped");
        0
    });
    let embedded = embed_pending(embedder)?;
    if synced > 0 || embedded > 0 {
        tracing::info!(
            target: "script_kit::brain",
            synced, embedded, "brain index cycle"
        );
    }
    Ok(())
}

/// Mirror active notes into brain_docs. Hash-guarded upserts make this cheap.
fn sync_notes() -> Result<usize> {
    let notes = crate::notes::get_all_notes()?;
    let mut synced = 0usize;
    for note in &notes {
        let source_id = note.id.to_string();
        let updated_at = note.updated_at.timestamp();
        store::upsert_doc(
            DocSource::Note,
            &source_id,
            &note.title,
            &note.content,
            updated_at,
        )?;
        synced += 1;
    }
    Ok(synced)
}

/// Embed docs with missing/stale vectors. Returns the number embedded.
fn embed_pending(embedder: &mut Option<BrainEmbedder>) -> Result<usize> {
    let Some(model) = resolve_embed_model() else {
        return Ok(0); // FTS-only mode: no model on disk.
    };
    if embedder
        .as_ref()
        .is_none_or(|e| e.model_id() != model.model_id)
    {
        *embedder = Some(BrainEmbedder::spawn(model)?);
    }
    let embedder = embedder.as_ref().expect("embedder just initialized");
    let mut total = 0usize;
    while total < MAX_EMBED_PER_CYCLE {
        let pending = store::docs_needing_embedding(embedder.model_id(), EMBED_BATCH)?;
        if pending.is_empty() {
            break;
        }
        let texts: Vec<String> = pending
            .iter()
            .map(|doc| {
                let mut text = format!("{}\n{}", doc.title, doc.content);
                if text.len() > EMBED_TEXT_CAP {
                    text.truncate(EMBED_TEXT_CAP);
                }
                text
            })
            .collect();
        let vectors = embedder.embed(texts)?;
        for (doc, vec) in pending.iter().zip(vectors.iter()) {
            if vec.is_empty() {
                continue;
            }
            store::store_embedding(doc.id, embedder.model_id(), &doc.title, &doc.content, vec)?;
            total += 1;
        }
        if vectors.len() < pending.len() {
            break;
        }
    }
    Ok(total)
}

/// Ingest one chat turn into the brain (called post-turn from Agent Chat).
/// `thread_id` + `turn_index` form the stable identity; re-ingesting the same
/// turn is a no-op thanks to hash guards.
pub fn ingest_chat_turn(
    thread_id: &str,
    turn_index: usize,
    user_text: &str,
    assistant_text: &str,
) -> Result<()> {
    store::init_brain_db()?;
    let user_text = user_text.trim();
    let assistant_text = assistant_text.trim();
    if user_text.is_empty() && assistant_text.is_empty() {
        return Ok(());
    }
    let title: String = user_text.chars().take(80).collect();
    let content = format!("User: {user_text}\n\nAssistant: {assistant_text}");
    let source_id = format!("{thread_id}#{turn_index}");
    let now = chrono::Utc::now().timestamp();
    store::upsert_doc(DocSource::ChatTurn, &source_id, &title, &content, now)?;
    // The user's own words are the strongest attention signal we have.
    for topic in extract_topics(user_text) {
        let _ = store::record_signal(&topic, 2, "chat");
    }
    wake_indexer();
    Ok(())
}

/// Cheap, deterministic topic extraction: significant lowercase words and
/// adjacent pairs. No model in the hot path, by design.
pub(crate) fn extract_topics(text: &str) -> Vec<String> {
    const STOP: &[&str] = &[
        "the", "and", "for", "that", "this", "with", "from", "what", "when", "where", "how", "why",
        "can", "could", "should", "would", "about", "into", "over", "just", "like", "want", "need",
        "have", "has", "had", "you", "your", "are", "was", "were", "will", "does", "doing", "done",
        "make", "made", "let", "lets", "please", "help", "use",
    ];
    let words: Vec<String> = text
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
        .filter(|w| w.len() > 3 && !STOP.contains(w))
        .map(|w| w.to_string())
        .collect();
    let mut topics: Vec<String> = Vec::new();
    for window in words.windows(2) {
        topics.push(format!("{} {}", window[0], window[1]));
    }
    for word in &words {
        topics.push(word.clone());
    }
    topics.truncate(8);
    topics
}
