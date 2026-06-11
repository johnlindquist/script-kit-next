//! The Curator: a scheduled, headless distillation pass.
//!
//! Once a day (and on demand via `kit://brain/focus?refresh=1`), the curator
//! asks the user's existing pi/codex setup to distill the brain's recent
//! evidence — attention signals and activity journals — into a short
//! **Focus review**: what the user actually worked on, what's heating up,
//! what stalled. The review is stored as a brain doc, so it's recallable
//! ("what did I work on this week?") and feeds future context staging.
//!
//! Design constraints:
//! - Runs `pi -p --no-tools` as a one-shot subprocess (no RPC plumbing, no
//!   tool surface, nothing to go wrong beyond a text completion).
//! - Uses the same binary resolution and codex auth as Agent Chat — if the
//!   user never set up Agent Chat, the curator silently skips.
//! - Never blocks anything: invoked from the indexer thread.

use super::inbox::{self, InboxKind};
use super::store::{self, DocSource};
use anyhow::{Context as _, Result};

const LAST_RUN_MARKER: &str = "curator_last_run";
const RUN_INTERVAL_SECS: i64 = 24 * 60 * 60;
const FOCUS_SOURCE_ID: &str = "focus-review";

/// Inbox extraction evidence window: chat turns updated this recently.
const INBOX_EVIDENCE_DAYS: i64 = 2;
/// At most this many chat docs feed one extraction prompt.
const INBOX_CHAT_DOC_CAP: usize = 20;
/// Per-doc content cap (chars) inside the extraction prompt.
const INBOX_DOC_CONTENT_CAP: usize = 1500;
/// Hard cap on accepted items per category from one model response.
const INBOX_MAX_PER_CATEGORY: usize = 8;
/// A pinned note untouched this long is flagged as a stale pin.
const STALE_PIN_DAYS: i64 = 14;

/// Run the curator if it's due. Called from the indexer cycle.
pub fn run_if_due() {
    let now = chrono::Utc::now().timestamp();
    let last = store::meta_get(LAST_RUN_MARKER)
        .ok()
        .flatten()
        .and_then(|value| value.parse::<i64>().ok());
    let Some(last) = last else {
        // Fresh database: stamp the marker and wait a full interval. A new
        // install must not fire an LLM call (and surprise inbox items)
        // seconds after first launch — distillation starts after a day of use.
        let _ = store::meta_set(LAST_RUN_MARKER, &now.to_string());
        return;
    };
    if now - last < RUN_INTERVAL_SECS {
        return;
    }
    // Mark first so a crashing model call can't hot-loop.
    let _ = store::meta_set(LAST_RUN_MARKER, &now.to_string());
    match run_focus_review() {
        Ok(true) => {
            tracing::info!(target: "script_kit::brain", "curator wrote focus review")
        }
        Ok(false) => {}
        Err(error) => {
            tracing::debug!(target: "script_kit::brain", error = %error, "curator skipped");
        }
    }
    match run_inbox_extraction() {
        Ok(count) if count > 0 => {
            tracing::info!(target: "script_kit::brain", count, "curator filed inbox items")
        }
        Ok(_) => {}
        Err(error) => {
            tracing::debug!(target: "script_kit::brain", error = %error, "inbox extraction skipped");
        }
    }
    let stale_pins = collect_stale_pins();
    if stale_pins > 0 {
        tracing::info!(target: "script_kit::brain", count = stale_pins, "curator flagged stale pins");
    }
}

/// Force a focus review now (e.g. kit://brain/focus?refresh=1).
pub fn run_focus_review() -> Result<bool> {
    let signals = store::recent_signals(200)?;
    let journals = recent_activity_journals(3)?;
    if signals.is_empty() && journals.is_empty() {
        return Ok(false); // Nothing to distill yet.
    }

    let topics = super::search::aggregate_signals(&signals);
    let topics_block = topics
        .iter()
        .take(12)
        .map(|(topic, weight)| format!("- {topic} (weight {weight})"))
        .collect::<Vec<_>>()
        .join("\n");
    let journal_block = journals.join("\n\n");
    let today = chrono::Local::now().format("%Y-%m-%d");

    let prompt = format!(
        "You are the curator of a personal knowledge base. Distill the \
         evidence below into a focus review for {today}. Write 6-12 terse \
         lines of markdown: '## Current focus' (ranked topics with one-line \
         why), '## Recent activity' (what the user actually did, grouped), \
         '## Drifting' (topics with attention but no recent activity, if \
         any). Facts only — no advice, no filler, no preamble.\n\n\
         ATTENTION SIGNALS (topic, accumulated weight):\n{topics_block}\n\n\
         ACTIVITY JOURNALS (newest first):\n{journal_block}"
    );

    let Some(output) = super::pi_oneshot(&prompt)? else {
        return Ok(false); // No agent setup — curator waits.
    };
    let review = output.trim();
    if review.is_empty() {
        return Ok(false);
    }
    store::upsert_doc(
        DocSource::Activity,
        FOCUS_SOURCE_ID,
        &format!("Focus review {today}"),
        review,
        chrono::Utc::now().timestamp(),
    )?;
    Ok(true)
}

/// The most recent N daily activity journals' contents.
fn recent_activity_journals(n: usize) -> Result<Vec<String>> {
    let mut journals = Vec::new();
    for back in 0..n {
        let day = (chrono::Local::now() - chrono::Duration::days(back as i64))
            .format("%Y-%m-%d")
            .to_string();
        if let Some(doc) = store::get_doc(DocSource::Activity, &format!("activity:{day}"))? {
            journals.push(format!("### {}\n{}", doc.title, doc.content));
        }
    }
    Ok(journals)
}

/// One extraction item from the model's JSON. `source_id` is the chat
/// thread/turn the item came from, when the model could tell.
#[derive(Debug, Default, serde::Deserialize)]
#[serde(default)]
pub struct ExtractedInboxItem {
    pub title: String,
    pub detail: String,
    #[serde(rename = "sourceId")]
    pub source_id: String,
}

/// The strict-JSON shape the extraction prompt demands.
#[derive(Debug, Default, serde::Deserialize)]
#[serde(default)]
pub struct InboxExtraction {
    pub commitments: Vec<ExtractedInboxItem>,
    pub questions: Vec<ExtractedInboxItem>,
    pub drift: Vec<ExtractedInboxItem>,
}

/// Parse the model's extraction response. Tolerates markdown code fences and
/// leading/trailing prose by slicing from the first `{` to the last `}`.
/// Empty titles are dropped and each category is capped at
/// [`INBOX_MAX_PER_CATEGORY`]. Drift items additionally pass a quality gate:
/// a drift title with no substantive word ("again", "else") is filler that
/// leaked through topic extraction, not a real subject, and is discarded
/// rather than shown as an inbox title. Pure — unit-testable without a model.
pub fn parse_inbox_extraction(raw: &str) -> Result<InboxExtraction> {
    let start = raw
        .find('{')
        .context("inbox extraction response contains no JSON object")?;
    let end = raw
        .rfind('}')
        .context("inbox extraction response contains no JSON object")?;
    if end < start {
        anyhow::bail!("inbox extraction response contains no JSON object");
    }
    let mut parsed: InboxExtraction = serde_json::from_str(&raw[start..=end])
        .context("inbox extraction response is not valid JSON")?;
    for list in [
        &mut parsed.commitments,
        &mut parsed.questions,
        &mut parsed.drift,
    ] {
        list.retain(|item| !item.title.trim().is_empty());
        list.truncate(INBOX_MAX_PER_CATEGORY);
    }
    parsed
        .drift
        .retain(|item| super::indexer::is_substantive_topic(&item.title));
    Ok(parsed)
}

/// Distill recent chat evidence (plus the current focus review) into inbox
/// items: commitments the user made, questions that got no answer, and
/// drifting topics. One pi call; strict-JSON response. Returns the number of
/// NEW items filed (dedupe makes re-runs idempotent).
pub fn run_inbox_extraction() -> Result<usize> {
    let since = chrono::Utc::now().timestamp() - INBOX_EVIDENCE_DAYS * 86_400;
    let chats = store::recent_docs_for_source(DocSource::ChatTurn, since, INBOX_CHAT_DOC_CAP)?;
    let focus = store::get_doc(DocSource::Activity, FOCUS_SOURCE_ID)?;
    if chats.is_empty() && focus.is_none() {
        tracing::debug!(target: "script_kit::brain", "inbox extraction: no evidence yet");
        return Ok(0);
    }

    let focus_block = focus
        .map(|doc| doc.content)
        .unwrap_or_else(|| "(none yet)".to_string());
    let chat_block = if chats.is_empty() {
        "(none in the window)".to_string()
    } else {
        chats
            .iter()
            .map(|doc| {
                let content: String = doc.content.chars().take(INBOX_DOC_CONTENT_CAP).collect();
                format!("### source id: {}\n{}", doc.source_id, content)
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    };

    let prompt = format!(
        "You are the curator of a personal knowledge base. Extract inbox \
         items from the evidence below. Respond with STRICT JSON only — no \
         markdown fences, no prose — exactly this shape:\n\
         {{\"commitments\":[{{\"title\":\"...\",\"detail\":\"...\",\
         \"sourceId\":\"<chat source id if known else empty>\"}}],\
         \"questions\":[{{\"title\":\"...\",\"detail\":\"...\",\
         \"sourceId\":\"...\"}}],\
         \"drift\":[{{\"title\":\"...\",\"detail\":\"...\"}}]}}\n\
         Commitments are things the user explicitly said they would do in \
         the chats. Questions are questions the user raised that received no \
         answer. Drift is topics with attention but no recent activity (the \
         focus review's Drifting section is evidence) — only include drift \
         topics that name a concrete project, tool, or subject; never \
         generic words like 'again', 'else', or 'second'. Titles under ten \
         words; details one or two sentences of context. At most \
         {INBOX_MAX_PER_CATEGORY} per category; use empty arrays when there \
         is nothing. Facts only — no advice, no invented items.\n\n\
         FOCUS REVIEW:\n{focus_block}\n\n\
         RECENT CHATS (each labeled with its source id):\n{chat_block}"
    );

    let Some(output) = super::pi_oneshot(&prompt)? else {
        tracing::debug!(target: "script_kit::brain", "inbox extraction: no agent setup");
        return Ok(0);
    };
    let extraction = parse_inbox_extraction(&output)?;
    let mut inserted = 0usize;
    for (kind, items) in [
        (InboxKind::Commitment, &extraction.commitments),
        (InboxKind::Question, &extraction.questions),
    ] {
        for item in items {
            if inbox::insert_inbox_item(
                kind,
                &item.title,
                &item.detail,
                DocSource::ChatTurn.as_str(),
                item.source_id.trim(),
            )? {
                inserted += 1;
            } else {
                tracing::debug!(
                    target: "script_kit::brain",
                    kind = kind.as_str(), title = %item.title, "inbox item skipped (dupe/blank)"
                );
            }
        }
    }
    for item in &extraction.drift {
        if inbox::insert_inbox_item(
            InboxKind::Drift,
            &item.title,
            &item.detail,
            DocSource::Activity.as_str(),
            FOCUS_SOURCE_ID,
        )? {
            inserted += 1;
        } else {
            tracing::debug!(
                target: "script_kit::brain",
                kind = "drift", title = %item.title, "inbox item skipped (dupe/blank)"
            );
        }
    }
    Ok(inserted)
}

/// Code-only stale-pin scan (no model): pinned notes untouched for
/// [`STALE_PIN_DAYS`] become `stale_pin` inbox items. Never errors the daily
/// pass — an unavailable notes DB just logs and yields 0.
pub fn collect_stale_pins() -> usize {
    let notes = match crate::notes::get_all_notes() {
        Ok(notes) => notes,
        Err(error) => {
            tracing::debug!(
                target: "script_kit::brain",
                error = %error, "stale pin scan skipped: notes unavailable"
            );
            return 0;
        }
    };
    let rows: Vec<(String, chrono::DateTime<chrono::Utc>, bool, String)> = notes
        .iter()
        .map(|note| {
            (
                note.title.clone(),
                note.updated_at,
                note.is_pinned,
                note.id.as_str(),
            )
        })
        .collect();
    let mut inserted = 0usize;
    for (title, detail, note_id) in stale_pins_from(&rows, chrono::Utc::now()) {
        match inbox::insert_inbox_item(
            InboxKind::StalePin,
            &title,
            &detail,
            DocSource::Note.as_str(),
            &note_id,
        ) {
            Ok(true) => inserted += 1,
            Ok(false) => {}
            Err(error) => {
                tracing::debug!(
                    target: "script_kit::brain",
                    error = %error, note_id = %note_id, "stale pin insert failed"
                );
            }
        }
    }
    inserted
}

/// Pure core of [`collect_stale_pins`]: from `(title, updated_at, is_pinned,
/// note_id)` rows, the `(title, detail, note_id)` inbox rows to file. Pinned
/// notes untouched for [`STALE_PIN_DAYS`] qualify; blank titles fall back to
/// "Pinned note".
pub(crate) fn stale_pins_from(
    notes: &[(String, chrono::DateTime<chrono::Utc>, bool, String)],
    now: chrono::DateTime<chrono::Utc>,
) -> Vec<(String, String, String)> {
    let cutoff = now - chrono::Duration::days(STALE_PIN_DAYS);
    notes
        .iter()
        .filter(|(_, updated_at, is_pinned, _)| *is_pinned && *updated_at < cutoff)
        .map(|(title, updated_at, _, note_id)| {
            let title = title.trim();
            let title = if title.is_empty() {
                "Pinned note".to_string()
            } else {
                title.to_string()
            };
            let detail = format!(
                "Pinned but untouched since {}",
                updated_at.format("%Y-%m-%d")
            );
            (title, detail, note_id.clone())
        })
        .collect()
}
