//! Persistent dictation history and ACP-facing provider payloads.

use crate::dictation::DictationTarget;
use chrono::{Datelike, Local};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

const HISTORY_COMPACT_LIMIT: usize = 200;
const RESOURCE_ITEMS_LIMIT: usize = 10;

type HistoryFileSignature = Option<(std::path::PathBuf, std::time::SystemTime, u64)>;

#[derive(Clone)]
struct DictationHistoryIndexCache {
    signature: HistoryFileSignature,
    entries: Vec<DictationHistoryEntry>,
}

static DICTATION_HISTORY_INDEX_CACHE: OnceLock<Mutex<Option<DictationHistoryIndexCache>>> =
    OnceLock::new();
static DICTATION_HISTORY_REFRESH_IN_FLIGHT: OnceLock<Mutex<bool>> = OnceLock::new();

fn dictation_history_index_cache() -> &'static Mutex<Option<DictationHistoryIndexCache>> {
    DICTATION_HISTORY_INDEX_CACHE.get_or_init(|| Mutex::new(None))
}

fn dictation_history_refresh_in_flight() -> &'static Mutex<bool> {
    DICTATION_HISTORY_REFRESH_IN_FLIGHT.get_or_init(|| Mutex::new(false))
}

fn invalidate_history_cache() {
    if let Some(cache) = DICTATION_HISTORY_INDEX_CACHE.get() {
        if let Ok(mut guard) = cache.lock() {
            *guard = None;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DictationHistoryEntry {
    pub id: String,
    pub timestamp: String,
    pub transcript: String,
    pub preview: String,
    pub target: String,
    pub audio_duration_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DictationHistorySearchField {
    Transcript,
    Target,
    Timestamp,
}

#[derive(Debug, Clone)]
pub struct DictationHistorySearchHit {
    pub entry: DictationHistoryEntry,
    pub score: u32,
    pub matched_field: DictationHistorySearchField,
}

#[derive(Debug, Clone)]
pub struct RootDictationHistorySearchHit {
    pub id: String,
    pub preview: String,
    pub target: String,
    pub timestamp: String,
    pub audio_duration_ms: u64,
    pub score: u32,
    pub matched_field: DictationHistorySearchField,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootDictationHistorySectionOptions {
    pub enabled: bool,
    pub max_results: usize,
    pub min_query_chars: usize,
    pub scan_limit: usize,
}

fn history_path() -> std::path::PathBuf {
    crate::setup::get_kit_path().join("dictation-history.jsonl")
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut out: String = value.chars().take(max_chars).collect();
    if value.chars().count() > max_chars {
        out.push('\u{2026}');
    }
    out
}

fn normalize_search_text(value: &str) -> String {
    collapse_whitespace(value).to_lowercase()
}

fn target_label(target: DictationTarget) -> String {
    match target {
        DictationTarget::MainWindowFilter => "Main Filter".to_string(),
        DictationTarget::MainWindowPrompt => "Prompt".to_string(),
        DictationTarget::NotesEditor => "Notes".to_string(),
        DictationTarget::AiChatComposer => "AI Chat".to_string(),
        DictationTarget::TabAiHarness => "Agent Chat".to_string(),
        DictationTarget::ExternalApp => crate::frontmost_app_tracker::get_last_real_app()
            .map(|app| app.name.trim().to_string())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| "Frontmost App".to_string()),
    }
}

pub fn format_history_timestamp(timestamp: &str) -> String {
    let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(timestamp) else {
        return timestamp.to_string();
    };

    let localized = parsed.with_timezone(&Local);
    let now = Local::now();
    let format = if localized.year() == now.year() {
        "%b %-d at %-I:%M %P"
    } else {
        "%b %-d, %Y at %-I:%M %P"
    };

    localized.format(format).to_string()
}

pub fn format_history_duration_ms(audio_duration_ms: u64) -> String {
    match audio_duration_ms {
        0..=999 => "under 1 sec".to_string(),
        1_000..=9_999 => format!("{:.1} sec", audio_duration_ms as f64 / 1_000.0),
        10_000..=59_999 => format!("{} sec", (audio_duration_ms + 500) / 1_000),
        _ => {
            let total_seconds = (audio_duration_ms + 500) / 1_000;
            let hours = total_seconds / 3_600;
            let minutes = (total_seconds % 3_600) / 60;
            let seconds = total_seconds % 60;

            if hours > 0 {
                if seconds == 0 {
                    format!("{hours} hr {minutes} min")
                } else {
                    format!("{hours} hr {minutes} min {seconds} sec")
                }
            } else if seconds == 0 {
                format!("{minutes} min")
            } else {
                format!("{minutes} min {seconds} sec")
            }
        }
    }
}

pub fn build_history_entry(
    transcript: &str,
    audio_duration: Duration,
    target: DictationTarget,
) -> DictationHistoryEntry {
    let now = chrono::Utc::now();
    let timestamp = now.to_rfc3339();
    let normalized = collapse_whitespace(transcript);
    let id = format!(
        "dictation-{}-{}",
        now.format("%Y%m%dT%H%M%S%.3fZ"),
        uuid::Uuid::new_v4().simple()
    );

    DictationHistoryEntry {
        id,
        timestamp,
        preview: truncate_chars(&normalized, 120),
        transcript: transcript.trim().to_string(),
        target: target_label(target),
        audio_duration_ms: audio_duration.as_millis() as u64,
    }
}

fn write_history(entries: &[DictationHistoryEntry]) -> std::io::Result<()> {
    use std::io::Write;

    let path = history_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = std::fs::File::create(path)?;
    for entry in entries {
        if let Ok(json) = serde_json::to_string(entry) {
            writeln!(file, "{json}")?;
        }
    }
    invalidate_history_cache();
    Ok(())
}

fn save_history_entry(entry: &DictationHistoryEntry) {
    let path = history_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let Ok(json) = serde_json::to_string(entry) else {
        return;
    };

    use std::io::Write;
    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    else {
        tracing::debug!(path = %path.display(), "dictation_history_write_failed");
        return;
    };
    let _ = writeln!(file, "{json}");

    if load_history().len() > HISTORY_COMPACT_LIMIT {
        let compacted: Vec<DictationHistoryEntry> = load_history()
            .into_iter()
            .take(HISTORY_COMPACT_LIMIT)
            .collect();
        let rewritten: Vec<DictationHistoryEntry> = compacted.into_iter().rev().collect();
        let _ = write_history(&rewritten);
    }
}

pub fn load_history() -> Vec<DictationHistoryEntry> {
    let path = history_path();
    let signature = history_file_signature(&path);
    if let Ok(guard) = dictation_history_index_cache().lock() {
        if let Some(cache) = guard.as_ref() {
            if cache.signature == signature {
                return cache.entries.clone();
            }
        }
    }

    let entries = std::fs::read_to_string(&path)
        .map(|content| parse_history_entries(&content))
        .unwrap_or_default();

    if let Ok(mut guard) = dictation_history_index_cache().lock() {
        *guard = Some(DictationHistoryIndexCache {
            signature,
            entries: entries.clone(),
        });
    }

    entries
}

fn history_file_signature(path: &std::path::Path) -> HistoryFileSignature {
    std::fs::metadata(path).ok().map(|metadata| {
        (
            path.to_path_buf(),
            metadata
                .modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
            metadata.len(),
        )
    })
}

fn parse_history_entries(content: &str) -> Vec<DictationHistoryEntry> {
    let mut entries: Vec<DictationHistoryEntry> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<DictationHistoryEntry>(line).ok())
        .collect();
    entries.reverse();
    entries
}

pub fn get_history_entry(id: &str) -> Option<DictationHistoryEntry> {
    load_history().into_iter().find(|entry| entry.id == id)
}

fn rank_history_entries(
    entries: Vec<DictationHistoryEntry>,
    query: &str,
    limit: usize,
) -> Vec<DictationHistorySearchHit> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return entries
            .into_iter()
            .take(limit)
            .map(|entry| DictationHistorySearchHit {
                entry,
                score: 0,
                matched_field: DictationHistorySearchField::Transcript,
            })
            .collect();
    }

    let tokens: Vec<String> = normalize_search_text(trimmed)
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect();

    let mut hits = Vec::new();

    for entry in entries {
        let transcript = normalize_search_text(&entry.transcript);
        let preview = normalize_search_text(&entry.preview);
        let target = normalize_search_text(&entry.target);
        let timestamp = normalize_search_text(&entry.timestamp);
        let display_timestamp = normalize_search_text(&format_history_timestamp(&entry.timestamp));
        let display_duration =
            normalize_search_text(&format_history_duration_ms(entry.audio_duration_ms));
        let combined =
            format!("{transcript}\n{preview}\n{target}\n{timestamp}\n{display_timestamp}\n{display_duration}");

        if !tokens.iter().all(|token| combined.contains(token)) {
            continue;
        }

        let transcript_score = tokens.iter().fold(0u32, |acc, token| {
            acc + if transcript.starts_with(token) {
                80
            } else if transcript.contains(token) || preview.contains(token) {
                30
            } else {
                0
            }
        });
        let target_score = tokens.iter().fold(0u32, |acc, token| {
            acc + if target.contains(token) { 20 } else { 0 }
        });
        let timestamp_score = tokens.iter().fold(0u32, |acc, token| {
            acc + if timestamp.contains(token)
                || display_timestamp.contains(token)
                || display_duration.contains(token)
            {
                5
            } else {
                0
            }
        });
        let total_score = transcript_score + target_score + timestamp_score;

        let matched_field =
            if transcript_score >= target_score && transcript_score >= timestamp_score {
                DictationHistorySearchField::Transcript
            } else if target_score >= timestamp_score {
                DictationHistorySearchField::Target
            } else {
                DictationHistorySearchField::Timestamp
            };

        hits.push(DictationHistorySearchHit {
            entry,
            score: total_score,
            matched_field,
        });
    }

    hits.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| b.entry.timestamp.cmp(&a.entry.timestamp))
    });
    hits.truncate(limit);
    hits
}

pub fn search_history(query: &str, limit: usize) -> Vec<DictationHistorySearchHit> {
    let hits = rank_history_entries(load_history(), query, limit);
    tracing::info!(
        category = "DICTATION",
        event = "dictation_history_search_executed",
        query = %query,
        limit,
        hit_count = hits.len(),
    );
    hits
}

fn cached_history_entries_if_fresh() -> Option<Vec<DictationHistoryEntry>> {
    let path = history_path();
    let signature = history_file_signature(&path);
    let guard = dictation_history_index_cache().lock().ok()?;
    let cache = guard.as_ref()?;
    (cache.signature == signature).then(|| cache.entries.clone())
}

fn ensure_history_cache_warm() {
    if let Ok(mut refreshing) = dictation_history_refresh_in_flight().lock() {
        if *refreshing {
            return;
        }
        *refreshing = true;
    } else {
        return;
    }

    let spawn_result = std::thread::Builder::new()
        .name("root-dictation-history-cache".to_string())
        .spawn(|| {
            let _ = load_history();
            if let Ok(mut refreshing) = dictation_history_refresh_in_flight().lock() {
                *refreshing = false;
            }
        });

    if spawn_result.is_err() {
        if let Ok(mut refreshing) = dictation_history_refresh_in_flight().lock() {
            *refreshing = false;
        }
    }
}

pub fn root_dictation_history_query_is_eligible(
    query: &str,
    options: RootDictationHistorySectionOptions,
) -> bool {
    let trimmed = query.trim();
    options.enabled
        && trimmed.len() >= options.min_query_chars
        && !trimmed.contains('\n')
        && !trimmed.contains('\r')
}

pub fn search_root_dictation_history(
    query: &str,
    options: RootDictationHistorySectionOptions,
) -> Vec<RootDictationHistorySearchHit> {
    let entries = load_history()
        .into_iter()
        .take(options.scan_limit)
        .collect::<Vec<_>>();
    let hits = rank_history_entries(entries, query, options.max_results)
        .into_iter()
        .map(|hit| RootDictationHistorySearchHit {
            id: hit.entry.id,
            preview: hit.entry.preview,
            target: hit.entry.target,
            timestamp: hit.entry.timestamp,
            audio_duration_ms: hit.entry.audio_duration_ms,
            score: hit.score,
            matched_field: hit.matched_field,
        })
        .collect::<Vec<_>>();
    tracing::info!(
        category = "DICTATION",
        event = "root_dictation_history_search_executed",
        query_len = query.trim().chars().count(),
        scan_limit = options.scan_limit,
        max_results = options.max_results,
        hit_count = hits.len(),
    );
    hits
}

/// Cache-only dictation history search for root launcher passive rows.
///
/// Cold JSONL reads warm a background index and return no hits for the current
/// frame, preserving the active search result projection while the user types.
pub fn search_root_dictation_history_cached(
    query: &str,
    options: RootDictationHistorySectionOptions,
) -> Vec<RootDictationHistorySearchHit> {
    if !root_dictation_history_query_is_eligible(query, options) {
        return Vec::new();
    }

    let Some(entries) = cached_history_entries_if_fresh() else {
        ensure_history_cache_warm();
        tracing::info!(
            category = "DICTATION",
            event = "root_dictation_history_search_cache_miss",
            query_len = query.trim().chars().count(),
            scan_limit = options.scan_limit,
            max_results = options.max_results,
        );
        return Vec::new();
    };

    let hits = rank_history_entries(
        entries.into_iter().take(options.scan_limit).collect(),
        query,
        options.max_results,
    )
    .into_iter()
    .map(|hit| RootDictationHistorySearchHit {
        id: hit.entry.id,
        preview: hit.entry.preview,
        target: hit.entry.target,
        timestamp: hit.entry.timestamp,
        audio_duration_ms: hit.entry.audio_duration_ms,
        score: hit.score,
        matched_field: hit.matched_field,
    })
    .collect::<Vec<_>>();
    tracing::info!(
        category = "DICTATION",
        event = "root_dictation_history_search_cache_hit",
        query_len = query.trim().chars().count(),
        scan_limit = options.scan_limit,
        max_results = options.max_results,
        hit_count = hits.len(),
    );
    hits
}

fn resource_payload(entries: &[DictationHistoryEntry]) -> String {
    if entries.is_empty() {
        return serde_json::json!({
            "schemaVersion": 1,
            "type": "dictation",
            "ok": true,
            "available": false,
            "source": "history",
            "items": [],
            "note": "No saved dictation history yet.",
            "nextStep": "Start dictation to capture text."
        })
        .to_string();
    }

    let items: Vec<serde_json::Value> = entries
        .iter()
        .map(|entry| {
            serde_json::json!({
                "id": entry.id,
                "timestamp": entry.timestamp,
                "displayTimestamp": format_history_timestamp(&entry.timestamp),
                "text": entry.transcript,
                "preview": entry.preview,
                "target": entry.target,
                "audioDurationMs": entry.audio_duration_ms,
                "displayDuration": format_history_duration_ms(entry.audio_duration_ms),
            })
        })
        .collect();

    serde_json::json!({
        "schemaVersion": 1,
        "type": "dictation",
        "ok": true,
        "available": true,
        "source": "history",
        "count": entries.len(),
        "current": items.first().cloned(),
        "items": items,
    })
    .to_string()
}

fn refresh_published_resource_from_entries(entries: &[DictationHistoryEntry]) {
    crate::mcp_resources::publish_dictation_json(resource_payload(entries));
}

pub fn hydrate_dictation_resource_from_history() {
    let entries = load_history();
    let latest: Vec<DictationHistoryEntry> =
        entries.into_iter().take(RESOURCE_ITEMS_LIMIT).collect();
    refresh_published_resource_from_entries(&latest);
}

pub fn record_dictation_history(
    transcript: &str,
    audio_duration: Duration,
    target: DictationTarget,
) -> DictationHistoryEntry {
    let entry = build_history_entry(transcript, audio_duration, target);
    save_history_entry(&entry);
    let recent: Vec<DictationHistoryEntry> = load_history()
        .into_iter()
        .take(RESOURCE_ITEMS_LIMIT)
        .collect();
    refresh_published_resource_from_entries(&recent);
    tracing::info!(
        category = "DICTATION",
        event = "dictation_history_entry_saved",
        entry_id = %entry.id,
        target = %entry.target,
        transcript_len = entry.transcript.len(),
        audio_duration_ms = entry.audio_duration_ms,
    );
    entry
}

pub fn delete_history_entry(entry_id: &str) -> anyhow::Result<()> {
    use anyhow::Context;

    let entries: Vec<DictationHistoryEntry> = load_history()
        .into_iter()
        .filter(|entry| entry.id != entry_id)
        .collect();
    let rewritten: Vec<DictationHistoryEntry> = entries.iter().cloned().rev().collect();
    write_history(&rewritten).with_context(|| format!("rewrite {}", history_path().display()))?;
    let recent: Vec<DictationHistoryEntry> =
        entries.into_iter().take(RESOURCE_ITEMS_LIMIT).collect();
    refresh_published_resource_from_entries(&recent);
    tracing::info!(
        category = "DICTATION",
        event = "dictation_history_entry_deleted",
        entry_id = %entry_id,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEnv {
        _sk_path_lock: std::sync::MutexGuard<'static, ()>,
        _provider_json_lock: std::sync::MutexGuard<'static, ()>,
        prev_sk_path: Option<String>,
        tempdir: tempfile::TempDir,
    }

    impl TestEnv {
        fn new() -> Self {
            let lock = crate::test_utils::SK_PATH_TEST_LOCK
                .get_or_init(|| std::sync::Mutex::new(()))
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let provider_json_lock = crate::test_utils::lock_provider_json_test();
            let tempdir = tempfile::tempdir().expect("tempdir");
            let prev_sk_path = std::env::var(crate::setup::SK_PATH_ENV).ok();
            std::env::set_var(crate::setup::SK_PATH_ENV, tempdir.path());
            crate::mcp_resources::clear_provider_json_slots();
            Self {
                _sk_path_lock: lock,
                _provider_json_lock: provider_json_lock,
                prev_sk_path,
                tempdir,
            }
        }
    }

    impl Drop for TestEnv {
        fn drop(&mut self) {
            match &self.prev_sk_path {
                Some(value) => std::env::set_var(crate::setup::SK_PATH_ENV, value),
                None => std::env::remove_var(crate::setup::SK_PATH_ENV),
            }
            crate::mcp_resources::clear_provider_json_slots();
            let _ = &self.tempdir;
        }
    }

    #[test]
    fn build_history_entry_captures_preview_and_target() {
        let entry = build_history_entry(
            "hello from dictation",
            Duration::from_secs(2),
            DictationTarget::AiChatComposer,
        );
        assert_eq!(entry.preview, "hello from dictation");
        assert_eq!(entry.target, "AI Chat");
        assert_eq!(entry.audio_duration_ms, 2_000);
    }

    #[test]
    fn format_history_duration_humanizes_common_values() {
        assert_eq!(format_history_duration_ms(450), "under 1 sec");
        assert_eq!(format_history_duration_ms(8_507), "8.5 sec");
        assert_eq!(format_history_duration_ms(12_200), "12 sec");
        assert_eq!(format_history_duration_ms(61_400), "1 min 1 sec");
    }

    #[test]
    fn record_and_load_history_round_trip() {
        let _env = TestEnv::new();
        let first = record_dictation_history(
            "first transcript",
            Duration::from_secs(1),
            DictationTarget::NotesEditor,
        );
        let second = record_dictation_history(
            "second transcript",
            Duration::from_secs(2),
            DictationTarget::AiChatComposer,
        );

        let loaded = load_history();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, second.id);
        assert_eq!(loaded[1].id, first.id);
    }

    #[test]
    fn search_history_matches_transcript_and_target() {
        let _env = TestEnv::new();
        record_dictation_history(
            "draft reply to the oauth ticket",
            Duration::from_secs(2),
            DictationTarget::AiChatComposer,
        );
        record_dictation_history(
            "quick note for the meeting",
            Duration::from_secs(1),
            DictationTarget::NotesEditor,
        );

        let ai_hits = search_history("oauth ai", 10);
        assert_eq!(ai_hits.len(), 1);
        assert_eq!(
            ai_hits[0].matched_field,
            DictationHistorySearchField::Transcript
        );

        let notes_hits = search_history("notes", 10);
        assert_eq!(notes_hits.len(), 1);
        assert_eq!(notes_hits[0].entry.target, "Notes");

        let duration_hits = search_history("ai 2 sec", 10);
        assert_eq!(duration_hits.len(), 1);
        assert_eq!(duration_hits[0].entry.target, "AI Chat");
    }

    #[test]
    fn delete_history_entry_rewrites_file_and_resource() {
        let _env = TestEnv::new();
        let keep = record_dictation_history(
            "keep me",
            Duration::from_secs(1),
            DictationTarget::MainWindowPrompt,
        );
        let drop = record_dictation_history(
            "drop me",
            Duration::from_secs(1),
            DictationTarget::ExternalApp,
        );

        delete_history_entry(&drop.id).expect("delete");
        let loaded = load_history();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, keep.id);
    }

    #[test]
    fn hydrate_publishes_empty_payload_when_no_history_exists() {
        let _env = TestEnv::new();
        crate::mcp_resources::clear_provider_json_slots();
        hydrate_dictation_resource_from_history();
        assert!(
            !crate::mcp_resources::has_provider_json_resource(
                crate::mcp_resources::ProviderJsonResourceKind::Dictation
            ),
            "empty history should not advertise dictation provider data"
        );
    }

    #[test]
    fn record_history_publishes_recent_items_to_provider_slot() {
        let _env = TestEnv::new();
        crate::mcp_resources::clear_provider_json_slots();

        record_dictation_history(
            "provider-backed dictation",
            Duration::from_secs(3),
            DictationTarget::AiChatComposer,
        );

        assert!(
            crate::mcp_resources::has_provider_json_resource(
                crate::mcp_resources::ProviderJsonResourceKind::Dictation
            ),
            "saved history should hydrate the dictation provider slot"
        );
    }
}
