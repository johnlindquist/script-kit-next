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
/// Hard latency budget for query embedding on the submit path. When the
/// model isn't warm yet the caller falls back to lexical recall.
const QUERY_EMBED_BUDGET: Duration = Duration::from_millis(200);

enum IndexerRequest {
    Wake,
    EmbedQuery {
        text: String,
        reply: Sender<Option<(String, Vec<f32>)>>,
    },
}

static WAKE: OnceLock<Sender<IndexerRequest>> = OnceLock::new();
static STARTED: AtomicBool = AtomicBool::new(false);

/// Ask the indexer to run a cycle soon (e.g. after a chat turn ingests new
/// docs). Cheap; coalesces with pending wakes.
pub fn wake_indexer() {
    if let Some(tx) = WAKE.get() {
        let _ = tx.send(IndexerRequest::Wake);
    }
}

/// Embed a query using the indexer's warm model, within a hard latency
/// budget. Returns `(model_id, vector)`, or `None` when no model is on disk,
/// the model isn't warm yet, or the indexer is mid-cycle — callers fall back
/// to lexical recall. Never blocks longer than [`QUERY_EMBED_BUDGET`].
pub fn embed_query_within_budget(text: &str) -> Option<(String, Vec<f32>)> {
    let tx = WAKE.get()?;
    let (reply_tx, reply_rx) = mpsc::channel();
    tx.send(IndexerRequest::EmbedQuery {
        text: text.to_string(),
        reply: reply_tx,
    })
    .ok()?;
    reply_rx.recv_timeout(QUERY_EMBED_BUDGET).ok().flatten()
}

/// Start the background indexer thread. Idempotent.
pub fn start_brain_indexer() {
    if STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    let (tx, rx) = mpsc::channel::<IndexerRequest>();
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
                // Serve wake/embed requests until the next cycle is due.
                let deadline = std::time::Instant::now() + CYCLE_INTERVAL;
                loop {
                    let remaining = deadline.saturating_duration_since(std::time::Instant::now());
                    if remaining.is_zero() {
                        break;
                    }
                    match rx.recv_timeout(remaining) {
                        Ok(IndexerRequest::Wake) => break,
                        Ok(IndexerRequest::EmbedQuery { text, reply }) => {
                            let result = embedder.as_ref().and_then(|embedder| {
                                embedder
                                    .embed(vec![text])
                                    .ok()
                                    .and_then(|mut vectors| vectors.pop())
                                    .filter(|vector| !vector.is_empty())
                                    .map(|vector| (embedder.model_id().to_string(), vector))
                            });
                            let _ = reply.send(result);
                        }
                        Err(RecvTimeoutError::Timeout) => break,
                        Err(RecvTimeoutError::Disconnected) => return,
                    }
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
    let promoted = sync_pinned_clipboard().unwrap_or_else(|err| {
        tracing::debug!(target: "script_kit::brain", error = %err, "clipboard sync skipped");
        0
    });
    let captured = sync_capture_stores().unwrap_or_else(|err| {
        tracing::debug!(target: "script_kit::brain", error = %err, "capture sync skipped");
        0
    });
    sync_browser_attention();
    // Embedding trouble (missing helper binary, model load failure) must
    // never take down the rest of the metabolism — lexical search, journal,
    // and the curator all work without vectors.
    let embedded = embed_pending(embedder).unwrap_or_else(|err| {
        tracing::warn!(target: "script_kit::brain", error = %err, "embedding pass skipped");
        0
    });
    if synced > 0 || promoted > 0 || captured > 0 || embedded > 0 {
        tracing::info!(
            target: "script_kit::brain",
            synced, promoted, captured, embedded, "brain index cycle"
        );
    }
    // Daily distillation pass (no-op until due; silently skips without pi).
    super::curator::run_if_due();
    prune_ambient_if_due();
    // Heartbeat for the kit://brain health surface.
    let _ = store::meta_set(
        "last_index_cycle",
        &chrono::Utc::now().timestamp().to_string(),
    );
    Ok(())
}

/// Once a day, age out the brain's own ambient records (old activity
/// journals, stale attention signals). User-created content is never pruned.
fn prune_ambient_if_due() {
    const PRUNE_INTERVAL_SECS: i64 = 24 * 60 * 60;
    let now = chrono::Utc::now().timestamp();
    let last = store::meta_get("ambient_prune_last")
        .ok()
        .flatten()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0);
    if now - last < PRUNE_INTERVAL_SECS {
        return;
    }
    let _ = store::meta_set("ambient_prune_last", &now.to_string());
    match store::prune_ambient_data() {
        Ok((journals, signals, inbox)) if journals > 0 || signals > 0 || inbox > 0 => {
            tracing::info!(
                target: "script_kit::brain",
                journals, signals, inbox, "brain pruned aged ambient data"
            );
        }
        Ok(_) => {}
        Err(error) => {
            tracing::debug!(target: "script_kit::brain", error = %error, "ambient prune skipped");
        }
    }
}

/// Promote PINNED clipboard entries into the brain. Pinning is an explicit
/// "this matters" act — the cleanest ambient-learning signal the clipboard
/// emits, with zero surveillance creep (raw history stays in its own store
/// with its own retention). Image entries contribute their OCR text.
fn sync_pinned_clipboard() -> Result<usize> {
    let entries = crate::clipboard_history::get_clipboard_history(500);
    let mut promoted = 0usize;
    let mut pinned_ids = Vec::new();
    for entry in entries.iter().filter(|entry| entry.pinned) {
        let text = match entry.content_type {
            crate::clipboard_history::ContentType::Text => entry.content.clone(),
            _ => entry.ocr_text.clone().unwrap_or_default(),
        };
        let text = text.trim();
        if text.is_empty() {
            continue;
        }
        let title: String = format!(
            "Pinned clipboard: {}",
            text.chars().take(60).collect::<String>()
        );
        store::upsert_doc(
            DocSource::Clipboard,
            &entry.id,
            &title,
            text,
            entry.timestamp,
        )?;
        pinned_ids.push(entry.id.clone());
        promoted += 1;
    }
    // Unpinning is the user revoking the "this matters" signal — forget it.
    let _ = store::retain_docs(DocSource::Clipboard, &pinned_ids)?;
    Ok(promoted)
}

/// Browser attention: hosts the user visited repeatedly in the last day
/// become attention SIGNALS — ranking hints only, never documents. Page
/// titles and URLs are not stored in the brain; the raw history stays in
/// the browser. This is the sensory-buffer pattern: observe, distill,
/// discard.
fn sync_browser_attention() {
    const SIGNAL_WINDOW_MS: i64 = 24 * 60 * 60 * 1000;
    const MIN_VISITS: i64 = 3;
    let last_marker = store::meta_get("browser_attention_last")
        .ok()
        .flatten()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0);
    let now = chrono::Utc::now().timestamp();
    // At most once per cycle interval x4 — browser reads hit other apps'
    // sqlite files; be a polite neighbor.
    if now - last_marker < 8 * 60 {
        return;
    }
    let _ = store::meta_set("browser_attention_last", &now.to_string());
    let Ok(entries) = crate::browser_history::list_recent_history(250) else {
        return;
    };
    let cutoff_ms = chrono::Utc::now().timestamp_millis() - SIGNAL_WINDOW_MS;
    let mut by_host: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for entry in entries {
        if entry.last_visited_at_ms >= cutoff_ms {
            *by_host.entry(entry.host.to_string()).or_default() += 1;
        }
    }
    for (host, visits) in by_host {
        if visits >= MIN_VISITS && !host.is_empty() {
            let topic = host
                .trim_start_matches("www.")
                .split('.')
                .next()
                .unwrap_or(&host)
                .to_string();
            let _ = store::record_signal(&topic, 1, "browser");
        }
    }
}

/// Mirror active notes into brain_docs. Hash-guarded upserts make this
/// cheap, and deletion sync forgets notes the user deleted — a brain that
/// remembers what its owner erased is a bug, not a feature.
fn sync_notes() -> Result<usize> {
    let notes = crate::notes::get_all_notes()?;
    let mut synced = 0usize;
    let mut live_ids = Vec::with_capacity(notes.len());
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
        live_ids.push(source_id);
        synced += 1;
    }
    let removed = store::retain_docs(DocSource::Note, &live_ids)?;
    if removed > 0 {
        tracing::info!(target: "script_kit::brain", removed, "brain forgot deleted notes");
    }
    Ok(synced)
}

/// Mirror `;` capture stores into brain_docs: todos (todos.jsonl), links
/// (links.md), and snippets (snippets.md). Notes captured via `;note` already
/// arrive through [`sync_notes`]. Same contract as the other mirrors:
/// hash-guarded upserts, and deletion sync so a todo/link/snippet the user
/// removed is forgotten by the brain.
fn sync_capture_stores() -> Result<usize> {
    sync_capture_stores_in_sk_path(&capture_sk_path())
}

fn capture_sk_path() -> std::path::PathBuf {
    if let Ok(path) = std::env::var(crate::setup::SK_PATH_ENV) {
        if !path.trim().is_empty() {
            return std::path::PathBuf::from(path);
        }
    }
    dirs::home_dir()
        .map(|home| home.join(".scriptkit"))
        .unwrap_or_else(|| std::path::PathBuf::from(".scriptkit"))
}

fn file_mtime_timestamp(path: &std::path::Path) -> i64 {
    std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|mtime| {
            mtime
                .duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| d.as_secs() as i64)
        })
        .unwrap_or(0)
}

pub(crate) fn sync_capture_stores_in_sk_path(sk_path: &std::path::Path) -> Result<usize> {
    let mut synced = 0usize;
    let mut live_ids: Vec<String> = Vec::new();

    // Todos: ~/.scriptkit/menu-syntax/todos.jsonl. Done todos stay (a kept
    // commitment is still memory); deleted ones are erased here too.
    let todos_path = sk_path.join("menu-syntax").join("todos.jsonl");
    if let Ok(contents) = std::fs::read_to_string(&todos_path) {
        let fallback_ts = file_mtime_timestamp(&todos_path);
        for line in contents.lines() {
            let Ok(record) = serde_json::from_str::<serde_json::Value>(line.trim()) else {
                continue;
            };
            let deleted = record
                .get("deletedAt")
                .and_then(|v| v.as_str())
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false)
                || record
                    .get("status")
                    .and_then(|v| v.as_str())
                    .map(|v| v.eq_ignore_ascii_case("deleted"))
                    .unwrap_or(false);
            if deleted {
                continue;
            }
            let Some(id) = record.get("id").and_then(|v| v.as_str()) else {
                continue;
            };
            let body = record.get("body").and_then(|v| v.as_str()).unwrap_or("");
            if body.trim().is_empty() {
                continue;
            }
            let mut content = body.to_string();
            for (key, label) in [
                ("status", "status"),
                ("due", "due"),
                ("remindAt", "remind at"),
                ("snoozeUntil", "snoozed until"),
                ("deferUntil", "deferred until"),
            ] {
                if let Some(value) = record.get(key).and_then(|v| v.as_str()) {
                    content.push_str(&format!("\n{label}: {value}"));
                }
            }
            if let Some(tags) = record.get("tags").and_then(|v| v.as_array()) {
                let tags: Vec<&str> = tags.iter().filter_map(|t| t.as_str()).collect();
                if !tags.is_empty() {
                    content.push_str(&format!("\ntags: {}", tags.join(", ")));
                }
            }
            let updated_at = record
                .get("updatedAt")
                .and_then(|v| v.as_str())
                .and_then(|v| chrono::DateTime::parse_from_rfc3339(v).ok())
                .map(|dt| dt.timestamp())
                .unwrap_or(fallback_ts);
            let title: String = format!("Todo: {}", body.chars().take(60).collect::<String>());
            let source_id = format!("todo:{id}");
            store::upsert_doc(DocSource::Capture, &source_id, &title, &content, updated_at)?;
            live_ids.push(source_id);
            synced += 1;
        }
    }

    // Links: plugins/main/scriptlets/links.md sections.
    let links_path = crate::scriptlets::link_markdown_store::links_markdown_path(sk_path);
    let links_ts = file_mtime_timestamp(&links_path);
    if let Ok(sections) = crate::scriptlets::link_markdown_store::load_link_sections(&links_path) {
        for section in sections {
            if section.id.trim().is_empty() {
                continue;
            }
            let mut content = section.url.clone().unwrap_or_default();
            if let Some(description) = &section.description {
                content.push_str(&format!("\n{description}"));
            }
            let title = format!("Link: {}", section.title);
            let source_id = format!("link:{}", section.id);
            store::upsert_doc(DocSource::Capture, &source_id, &title, &content, links_ts)?;
            live_ids.push(source_id);
            synced += 1;
        }
    }

    // Snippets: plugins/main/scriptlets/snippets.md sections.
    let snippets_path = crate::scriptlets::snippet_markdown_store::snippets_markdown_path(sk_path);
    let snippets_ts = file_mtime_timestamp(&snippets_path);
    if let Ok(sections) =
        crate::scriptlets::snippet_markdown_store::load_snippet_sections(&snippets_path)
    {
        for section in sections {
            if section.id.trim().is_empty() {
                continue;
            }
            let mut content = section.body.clone();
            if let Some(keyword) = &section.keyword {
                content.push_str(&format!("\nkeyword: {keyword}"));
            }
            if let Some(description) = &section.description {
                content.push_str(&format!("\n{description}"));
            }
            let title = format!("Snippet: {}", section.name);
            let source_id = format!("snippet:{}", section.id);
            store::upsert_doc(
                DocSource::Capture,
                &source_id,
                &title,
                &content,
                snippets_ts,
            )?;
            live_ids.push(source_id);
            synced += 1;
        }
    }

    let removed = store::retain_docs(DocSource::Capture, &live_ids)?;
    if removed > 0 {
        tracing::info!(target: "script_kit::brain", removed, "brain forgot deleted captures");
    }
    Ok(synced)
}

/// Embed docs with missing/stale vectors. Returns the number embedded.
fn embed_pending(embedder: &mut Option<BrainEmbedder>) -> Result<usize> {
    if resolve_embed_model().is_none() {
        // Zero-setup semantic search: fetch the model once the brain has
        // content worth embedding (politeness rules in brain::download).
        let (docs, _, _) = store::doc_stats().unwrap_or((0, 0, 0));
        if !super::download::ensure_embed_model(docs > 0) {
            return Ok(0); // FTS-only mode: no model on disk (yet).
        }
    }
    let Some(model) = resolve_embed_model() else {
        return Ok(0);
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

/// Words too generic to ever stand as an attention topic on their own —
/// conversational filler ("again", "else", "really") accumulates ask weight
/// fast and then surfaces verbatim as Brain Inbox drift titles. Shared with
/// the curator's drift gate via [`is_substantive_topic`] so both layers
/// agree on what counts as substance. Words of 3 chars or fewer are already
/// dropped by the length filter and don't need listing.
const GENERIC_WORDS: &[&str] = &[
    "about",
    "after",
    "again",
    "also",
    "another",
    "anything",
    "around",
    "back",
    "because",
    "been",
    "before",
    "being",
    "could",
    "does",
    "doing",
    "done",
    "down",
    "each",
    "else",
    "even",
    "ever",
    "every",
    "everything",
    "first",
    "from",
    "getting",
    "give",
    "goes",
    "going",
    "gonna",
    "good",
    "have",
    "help",
    "here",
    "into",
    "just",
    "know",
    "later",
    "lets",
    "like",
    "made",
    "make",
    "many",
    "maybe",
    "more",
    "most",
    "much",
    "need",
    "never",
    "nothing",
    "okay",
    "only",
    "other",
    "over",
    "please",
    "really",
    "right",
    "same",
    "second",
    "should",
    "show",
    "some",
    "somehow",
    "something",
    "soon",
    "still",
    "stuff",
    "sure",
    "take",
    "tell",
    "than",
    "thanks",
    "that",
    "their",
    "them",
    "then",
    "there",
    "these",
    "they",
    "thing",
    "things",
    "think",
    "this",
    "those",
    "time",
    "today",
    "tomorrow",
    "trying",
    "very",
    "want",
    "well",
    "were",
    "what",
    "when",
    "where",
    "which",
    "while",
    "will",
    "with",
    "without",
    "would",
    "yeah",
    "yesterday",
    "your",
];

/// Whether a candidate word carries enough meaning to be (part of) a topic.
fn is_substantive_word(word: &str) -> bool {
    word.len() > 3 && !GENERIC_WORDS.contains(&word)
}

/// Whether a topic/title string contains at least one substantive word.
/// The curator uses this to refuse drift inbox titles like "again" or
/// "else" while still accepting "second brain" or "build script".
pub(crate) fn is_substantive_topic(topic: &str) -> bool {
    topic
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
        .any(is_substantive_word)
}

/// Cheap, deterministic topic extraction: significant lowercase words and
/// adjacent pairs. No model in the hot path, by design.
pub(crate) fn extract_topics(text: &str) -> Vec<String> {
    let words: Vec<String> = text
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
        .filter(|w| is_substantive_word(w))
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
