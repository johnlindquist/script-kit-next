//! Captures inverse-browser artifact reader.
//!
//! Oracle iter 007 commit 7: supply a tolerant reader over the local-first
//! JSONL files capture handlers append to under `$SK_PATH/menu-syntax/`, plus
//! the payload tempfile directory. The reader is pure — it does not list or
//! delete files outside the expected set, does not touch GPUI, and skips
//! dirty JSONL lines with a capped warning list so a single bad row never
//! crashes the Captures built-in view.
//!
//! File layout expected under `$SK_PATH/menu-syntax/`:
//!
//! - `todos.jsonl` — `+todo` captures, one JSON line per entry.
//! - `events.jsonl` — `+cal` captures.
//! - `notes.jsonl` — `+note` captures (shipped example uses per-day markdown
//!   but scaffolded handlers default to JSONL, so both are valid — the
//!   reader only enumerates JSONL for now).
//! - `drafts.jsonl` — `+social` draft append log.
//! - `bookmarks.jsonl` — `+link` captures.
//! - `payloads/capture_v1-*.json` — per-execution payload tempfiles (written
//!   by `menu_syntax::execute::write_payload_tempfile`).

use std::fs::{read_dir, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// Maximum warning messages retained per `ReadArtifactReport`. Oracle iter 004
/// explicit rule: the reader must tolerate dirty JSONL, surface warning
/// counts, and never crash the builtin. A 10-warning cap is enough to
/// summarize the problem without flooding the UI with spam for files that
/// are entirely garbage.
pub const MAX_WARNINGS: usize = 10;

/// Maximum characters retained in the per-row snippet preview. The Captures
/// inverse browser renders one row per entry; the snippet is what the user
/// sees before they open the full artifact. Long bodies get truncated with
/// a trailing `…` so row height stays bounded.
pub const MAX_SNIPPET_CHARS: usize = 200;

/// Discriminator for capture artifacts. Matches the on-disk filenames used
/// by shipped examples and by the scaffold emitted from
/// [`super::templates::render_capture_handler_template`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CaptureArtifactKind {
    Todo,
    CalendarEvent,
    Note,
    SocialDraft,
    Bookmark,
    /// A per-execution payload tempfile (`capture_v1-*.json`). These are not
    /// captured artifacts in the user sense — they are the raw payloads the
    /// launcher wrote for the handler to consume. Retention (commit 8) runs
    /// against this kind specifically.
    Payload,
}

impl CaptureArtifactKind {
    /// All kinds that participate in the Captures inverse browser's aggregate
    /// view. Order matches the UI Oracle sketched in iter 004 (todos first
    /// because they're the most common capture, bookmarks last because
    /// clicking them usually hands off to a browser).
    pub const BROWSER_ORDER: &'static [Self] = &[
        Self::Todo,
        Self::CalendarEvent,
        Self::Note,
        Self::SocialDraft,
        Self::Bookmark,
    ];

    pub fn filename(self) -> &'static str {
        match self {
            Self::Todo => "todos.jsonl",
            Self::CalendarEvent => "events.jsonl",
            Self::Note => "notes.jsonl",
            Self::SocialDraft => "drafts.jsonl",
            Self::Bookmark => "bookmarks.jsonl",
            Self::Payload => "payloads",
        }
    }
}

/// One parsed artifact row. `raw_line` is retained so the UI (or the tests)
/// can present the unparsed source when needed; `snippet` is a pre-truncated
/// preview suitable for a single-line list row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureArtifact {
    pub kind: CaptureArtifactKind,
    pub path: PathBuf,
    /// 1-based line number within the source file. `None` for payload
    /// tempfiles — they are individual files, not lines in a shared log.
    pub line_number: Option<usize>,
    /// Best-effort creation timestamp extracted from the JSON (e.g. the
    /// `createdAt` field shipped examples write). `None` when absent or
    /// unparseable — the UI falls back to the file's mtime if it needs one.
    pub created_at: Option<String>,
    /// Truncated single-line preview. Never longer than [`MAX_SNIPPET_CHARS`].
    pub snippet: String,
    /// Original source line or filename. Bounded to the raw bytes that made
    /// it into the file — the reader never rewrites this.
    pub raw_line: String,
}

/// Aggregate result of one artifact read. The reader is infallible by design;
/// every failure surfaces as a warning string plus a bumped `skipped` count.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReadArtifactReport {
    pub entries: Vec<CaptureArtifact>,
    pub skipped: usize,
    pub warnings: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RootTodoSectionOptions {
    pub enabled: bool,
    pub max_results: usize,
    pub min_query_chars: usize,
}

impl Default for RootTodoSectionOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            max_results: 10,
            min_query_chars: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootTodoSearchHit {
    pub stable_key: String,
    pub title: String,
    pub body: String,
    pub subtitle: String,
    pub tags: Vec<String>,
    pub priority: Option<u8>,
    pub due: Option<String>,
    pub created_at: Option<String>,
    pub path: PathBuf,
    pub line_number: Option<usize>,
    pub raw_line: String,
}

impl ReadArtifactReport {
    /// Merge another report into this one. Entries are appended in order,
    /// skipped counts sum, and warnings are capped at [`MAX_WARNINGS`] so a
    /// single pathological file can't blow up the combined report.
    pub fn merge(&mut self, mut other: ReadArtifactReport) {
        self.entries.append(&mut other.entries);
        self.skipped = self.skipped.saturating_add(other.skipped);
        for warning in other.warnings {
            if self.warnings.len() >= MAX_WARNINGS {
                break;
            }
            self.warnings.push(warning);
        }
    }
}

/// Read a single JSONL artifact file. Missing files yield an empty report
/// (no error). Unreadable files yield one warning and `skipped += 1`. Every
/// non-empty line that fails to parse yields a warning (up to the cap) and
/// bumps `skipped`. Lines that parse but are not objects are still included
/// in `entries` with best-effort snippets.
pub fn read_jsonl_artifact(path: &Path, kind: CaptureArtifactKind) -> ReadArtifactReport {
    let mut report = ReadArtifactReport::default();

    let file = match File::open(path) {
        Ok(f) => f,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return report,
        Err(err) => {
            push_warning(
                &mut report,
                format!("could not open {}: {err}", path.display()),
            );
            report.skipped = report.skipped.saturating_add(1);
            return report;
        }
    };

    let reader = BufReader::new(file);
    for (idx, line_result) in reader.lines().enumerate() {
        let line = match line_result {
            Ok(l) => l,
            Err(err) => {
                push_warning(
                    &mut report,
                    format!("{}:{} read error: {err}", path.display(), idx + 1),
                );
                report.skipped = report.skipped.saturating_add(1);
                continue;
            }
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parsed = match serde_json::from_str::<serde_json::Value>(trimmed) {
            Ok(v) => v,
            Err(err) => {
                push_warning(
                    &mut report,
                    format!("{}:{} invalid JSON: {err}", path.display(), idx + 1),
                );
                report.skipped = report.skipped.saturating_add(1);
                continue;
            }
        };

        let created_at = extract_created_at(&parsed);
        let snippet = snippet_for_value(&parsed);
        report.entries.push(CaptureArtifact {
            kind,
            path: path.to_path_buf(),
            line_number: Some(idx + 1),
            created_at,
            snippet,
            raw_line: line,
        });
    }

    report
}

/// Read every `capture_v1-*.json` file under `payload_dir`. Missing directory
/// yields an empty report. Each file is read once; the entire JSON object is
/// the entry, so `line_number` is always `None`.
pub fn read_payload_dir(payload_dir: &Path) -> ReadArtifactReport {
    let mut report = ReadArtifactReport::default();

    let entries = match read_dir(payload_dir) {
        Ok(e) => e,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return report,
        Err(err) => {
            push_warning(
                &mut report,
                format!("could not read {}: {err}", payload_dir.display()),
            );
            return report;
        }
    };

    for dirent in entries.flatten() {
        let path = dirent.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.starts_with("capture_v1-") || !name.ends_with(".json") {
            continue;
        }
        let raw = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(err) => {
                push_warning(
                    &mut report,
                    format!("could not read {}: {err}", path.display()),
                );
                report.skipped = report.skipped.saturating_add(1);
                continue;
            }
        };
        let parsed = match serde_json::from_str::<serde_json::Value>(raw.trim()) {
            Ok(v) => v,
            Err(err) => {
                push_warning(
                    &mut report,
                    format!("{} invalid JSON: {err}", path.display()),
                );
                report.skipped = report.skipped.saturating_add(1);
                continue;
            }
        };
        let created_at = extract_created_at(&parsed);
        let snippet = snippet_for_value(&parsed);
        report.entries.push(CaptureArtifact {
            kind: CaptureArtifactKind::Payload,
            path,
            line_number: None,
            created_at,
            snippet,
            raw_line: raw,
        });
    }

    report
}

/// Read every known artifact kind under `sk_path/menu-syntax/` into a single
/// aggregate report. Caller is expected to pass the resolved `SK_PATH` root
/// (defaulting to `~/.scriptkit`). Reports are merged in
/// [`CaptureArtifactKind::BROWSER_ORDER`] followed by payloads last.
pub fn read_all_artifacts(sk_path: &Path) -> ReadArtifactReport {
    let mut report = ReadArtifactReport::default();
    let base = sk_path.join("menu-syntax");

    for kind in CaptureArtifactKind::BROWSER_ORDER {
        let filename = kind.filename();
        let path = base.join(filename);
        let sub = read_jsonl_artifact(&path, *kind);
        report.merge(sub);
    }

    let payload_dir = base.join(CaptureArtifactKind::Payload.filename());
    let payload_sub = read_payload_dir(&payload_dir);
    report.merge(payload_sub);

    report
}

pub fn root_todo_query_is_eligible(query: &str, options: RootTodoSectionOptions) -> bool {
    options.enabled && query.chars().count() >= options.min_query_chars && !query.contains('\n')
}

pub fn search_root_todos_direct(
    query: &str,
    options: RootTodoSectionOptions,
) -> Vec<RootTodoSearchHit> {
    if !root_todo_query_is_eligible(query, options) {
        return Vec::new();
    }
    search_root_todos_in_sk_path(query, options, &default_sk_path())
}

pub fn search_root_todos_in_sk_path(
    query: &str,
    options: RootTodoSectionOptions,
    sk_path: &Path,
) -> Vec<RootTodoSearchHit> {
    if !options.enabled {
        return Vec::new();
    }
    let path = sk_path
        .join("menu-syntax")
        .join(CaptureArtifactKind::Todo.filename());
    let report = read_jsonl_artifact(&path, CaptureArtifactKind::Todo);
    let normalized_query = normalize_match_text(query);
    let mut hits = report
        .entries
        .into_iter()
        .rev()
        .filter_map(todo_hit_from_artifact)
        .filter(|hit| todo_hit_matches(hit, &normalized_query))
        .take(options.max_results)
        .collect::<Vec<_>>();
    hits.shrink_to_fit();
    hits
}

fn todo_hit_from_artifact(artifact: CaptureArtifact) -> Option<RootTodoSearchHit> {
    if artifact.kind != CaptureArtifactKind::Todo {
        return None;
    }
    let parsed = serde_json::from_str::<serde_json::Value>(&artifact.raw_line).ok()?;
    if parsed
        .get("deletedAt")
        .and_then(|value| value.as_str())
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
        || parsed
            .get("status")
            .and_then(|value| value.as_str())
            .map(|value| value.eq_ignore_ascii_case("deleted"))
            .unwrap_or(false)
    {
        return None;
    }
    let body = parsed
        .get("body")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(artifact.snippet.as_str())
        .to_string();
    let tags = parsed
        .get("tags")
        .and_then(|value| value.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_str())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let priority = parsed
        .get("priority")
        .and_then(|value| value.as_u64())
        .and_then(|value| u8::try_from(value).ok());
    let due = parsed
        .get("due")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .or_else(|| first_date_display(&parsed));
    let subtitle = todo_subtitle(
        &tags,
        priority,
        due.as_deref(),
        artifact.created_at.as_deref(),
    );
    let line_number = artifact.line_number;
    let stable_key = format!(
        "todo/{}:{}",
        artifact.path.display(),
        line_number.unwrap_or_default()
    );
    Some(RootTodoSearchHit {
        stable_key,
        title: body.clone(),
        body,
        subtitle,
        tags,
        priority,
        due,
        created_at: artifact.created_at,
        path: artifact.path,
        line_number,
        raw_line: artifact.raw_line,
    })
}

fn first_date_display(parsed: &serde_json::Value) -> Option<String> {
    parsed
        .get("dates")
        .and_then(|value| value.as_array())
        .and_then(|dates| dates.first())
        .and_then(|date| {
            date.get("iso")
                .or_else(|| date.get("source"))
                .and_then(|value| value.as_str())
        })
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn todo_subtitle(
    tags: &[String],
    priority: Option<u8>,
    due: Option<&str>,
    created_at: Option<&str>,
) -> String {
    let mut parts = Vec::new();
    if let Some(priority) = priority {
        parts.push(format!("p{priority}"));
    }
    if let Some(due) = due.filter(|value| !value.trim().is_empty()) {
        parts.push(format!("due {due}"));
    }
    if !tags.is_empty() {
        parts.push(
            tags.iter()
                .map(|tag| format!("#{tag}"))
                .collect::<Vec<_>>()
                .join(" "),
        );
    }
    if let Some(created_at) = created_at.filter(|value| !value.trim().is_empty()) {
        parts.push(created_at.to_string());
    }
    if parts.is_empty() {
        "Captured todo".to_string()
    } else {
        parts.join(" · ")
    }
}

fn todo_hit_matches(hit: &RootTodoSearchHit, normalized_query: &str) -> bool {
    if normalized_query.is_empty() {
        return true;
    }
    let mut haystack = String::new();
    haystack.push_str(&hit.body);
    haystack.push(' ');
    haystack.push_str(&hit.raw_line);
    haystack.push(' ');
    haystack.push_str(&hit.tags.join(" "));
    if let Some(due) = &hit.due {
        haystack.push(' ');
        haystack.push_str(due);
    }
    normalize_match_text(&haystack).contains(normalized_query)
}

fn normalize_match_text(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn default_sk_path() -> PathBuf {
    if let Ok(path) = std::env::var(crate::setup::SK_PATH_ENV) {
        if !path.trim().is_empty() {
            return PathBuf::from(path);
        }
    }
    std::env::var("HOME")
        .map(|home| PathBuf::from(home).join(".scriptkit"))
        .unwrap_or_else(|_| PathBuf::from(".scriptkit"))
}

fn push_warning(report: &mut ReadArtifactReport, msg: String) {
    if report.warnings.len() >= MAX_WARNINGS {
        return;
    }
    report.warnings.push(msg);
}

fn extract_created_at(value: &serde_json::Value) -> Option<String> {
    let obj = value.as_object()?;
    // Shipped examples + scaffold use `createdAt`. Payload tempfiles use
    // `timestamp` (see `execute.rs`). Check both without privileging either.
    for key in ["createdAt", "timestamp"] {
        if let Some(v) = obj.get(key).and_then(|v| v.as_str()) {
            return Some(v.to_string());
        }
    }
    None
}

fn snippet_for_value(value: &serde_json::Value) -> String {
    let text = match value {
        serde_json::Value::Object(map) => {
            // Prefer human-readable fields in order: body, raw, target, then
            // the stringified object as a last resort. This keeps the snippet
            // meaningful for every shipped target shape.
            for key in ["body", "raw", "target", "url", "title"] {
                if let Some(v) = map.get(key).and_then(|v| v.as_str()) {
                    return truncate_snippet(v);
                }
            }
            serde_json::to_string(value).unwrap_or_default()
        }
        other => serde_json::to_string(other).unwrap_or_default(),
    };
    truncate_snippet(&text)
}

fn truncate_snippet(text: &str) -> String {
    let collapsed: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= MAX_SNIPPET_CHARS {
        return collapsed;
    }
    let mut out = String::with_capacity(MAX_SNIPPET_CHARS + 1);
    for (taken, ch) in collapsed.chars().enumerate() {
        if taken >= MAX_SNIPPET_CHARS.saturating_sub(1) {
            break;
        }
        out.push(ch);
    }
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_file(dir: &Path, name: &str, contents: &str) -> PathBuf {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("mkdir -p");
        }
        fs::write(&path, contents).expect("write artifact fixture");
        path
    }

    #[test]
    fn read_jsonl_artifact_returns_all_valid_entries() {
        let tmp = TempDir::new().expect("tempdir");
        let path = write_file(
            tmp.path(),
            "todos.jsonl",
            r#"{"body":"buy milk","createdAt":"2026-04-24T00:00:00Z"}
{"body":"walk dog","createdAt":"2026-04-24T00:01:00Z"}
"#,
        );
        let report = read_jsonl_artifact(&path, CaptureArtifactKind::Todo);
        assert_eq!(report.entries.len(), 2);
        assert_eq!(report.skipped, 0);
        assert!(report.warnings.is_empty());
        assert_eq!(report.entries[0].snippet, "buy milk");
        assert_eq!(
            report.entries[0].created_at.as_deref(),
            Some("2026-04-24T00:00:00Z")
        );
        assert_eq!(report.entries[0].line_number, Some(1));
        assert_eq!(report.entries[1].line_number, Some(2));
    }

    #[test]
    fn search_root_todos_reads_newest_first_and_matches_tags_due_and_body() {
        let tmp = TempDir::new().expect("tempdir");
        let base = tmp.path().join("menu-syntax");
        write_file(
            &base,
            "todos.jsonl",
            r#"{"body":"renew passport","tags":["errands"],"priority":1,"createdAt":"2026-05-19T10:00:00Z"}
{"body":"book design review","tags":["work"],"due":"Friday","createdAt":"2026-05-19T11:00:00Z"}
"#,
        );

        let hits = search_root_todos_in_sk_path(
            "work",
            RootTodoSectionOptions {
                max_results: 10,
                ..Default::default()
            },
            tmp.path(),
        );

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].title, "book design review");
        assert!(hits[0].subtitle.contains("#work"));
        assert!(hits[0].subtitle.contains("due Friday"));

        let hits =
            search_root_todos_in_sk_path("passport", RootTodoSectionOptions::default(), tmp.path());
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].priority, Some(1));
        assert_eq!(hits[0].line_number, Some(1));
    }

    #[test]
    fn search_root_todos_ignores_deleted_app_owned_rows() {
        let tmp = TempDir::new().expect("tempdir");
        let base = tmp.path().join("menu-syntax");
        write_file(
            &base,
            "todos.jsonl",
            r#"{"body":"old hidden task","status":"deleted","createdAt":"2026-05-19T10:00:00Z"}
{"body":"new hidden task","deletedAt":"2026-05-19T11:00:00Z","createdAt":"2026-05-19T11:00:00Z"}
{"body":"visible task","status":"open","createdAt":"2026-05-19T12:00:00Z"}
"#,
        );

        let hits =
            search_root_todos_in_sk_path("task", RootTodoSectionOptions::default(), tmp.path());

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].title, "visible task");
    }

    #[test]
    fn read_jsonl_artifact_skips_malformed_lines_with_warning() {
        let tmp = TempDir::new().expect("tempdir");
        let path = write_file(
            tmp.path(),
            "todos.jsonl",
            r#"{"body":"ok"}
this is not json
{"body":"also ok"}
{oops: true}
"#,
        );
        let report = read_jsonl_artifact(&path, CaptureArtifactKind::Todo);
        assert_eq!(report.entries.len(), 2, "only valid JSON lines surface");
        assert_eq!(report.skipped, 2, "each malformed line bumps skipped");
        assert!(
            report.warnings.iter().any(|w| w.contains("invalid JSON")),
            "should describe the failure"
        );
    }

    #[test]
    fn read_jsonl_artifact_handles_missing_file_gracefully() {
        let tmp = TempDir::new().expect("tempdir");
        let path = tmp.path().join("does-not-exist.jsonl");
        let report = read_jsonl_artifact(&path, CaptureArtifactKind::Todo);
        assert!(report.entries.is_empty());
        assert_eq!(report.skipped, 0);
        assert!(
            report.warnings.is_empty(),
            "missing file is not a warning — it just means the user hasn't captured yet"
        );
    }

    #[test]
    fn read_jsonl_artifact_ignores_blank_lines() {
        let tmp = TempDir::new().expect("tempdir");
        let path = write_file(
            tmp.path(),
            "todos.jsonl",
            "\n\n{\"body\":\"ok\"}\n  \n{\"body\":\"also\"}\n\n",
        );
        let report = read_jsonl_artifact(&path, CaptureArtifactKind::Todo);
        assert_eq!(report.entries.len(), 2);
        assert_eq!(report.skipped, 0);
    }

    #[test]
    fn read_jsonl_artifact_truncates_snippet_for_long_bodies() {
        let long_body: String = "x".repeat(MAX_SNIPPET_CHARS * 3);
        let line = format!(
            r#"{{"body":"{}","createdAt":"2026-04-24T00:00:00Z"}}"#,
            long_body
        );
        let tmp = TempDir::new().expect("tempdir");
        let path = write_file(tmp.path(), "todos.jsonl", &format!("{line}\n"));
        let report = read_jsonl_artifact(&path, CaptureArtifactKind::Todo);
        assert_eq!(report.entries.len(), 1);
        let snippet = &report.entries[0].snippet;
        assert!(
            snippet.chars().count() <= MAX_SNIPPET_CHARS,
            "snippet respects cap, got {} chars",
            snippet.chars().count()
        );
        assert!(
            snippet.ends_with('…'),
            "truncated snippet ends with ellipsis"
        );
    }

    #[test]
    fn snippet_falls_back_to_raw_when_body_is_missing() {
        let tmp = TempDir::new().expect("tempdir");
        let path = write_file(
            tmp.path(),
            "events.jsonl",
            r#"{"title":"Standup","raw":";cal standup tomorrow 3pm"}
"#,
        );
        let report = read_jsonl_artifact(&path, CaptureArtifactKind::CalendarEvent);
        assert_eq!(report.entries.len(), 1);
        assert_eq!(report.entries[0].snippet, ";cal standup tomorrow 3pm");
    }

    #[test]
    fn non_object_top_level_json_is_still_included() {
        // Unusual but legal JSONL — a scalar per line. Reader must not drop it.
        let tmp = TempDir::new().expect("tempdir");
        let path = write_file(tmp.path(), "notes.jsonl", "\"just a string\"\n42\n");
        let report = read_jsonl_artifact(&path, CaptureArtifactKind::Note);
        assert_eq!(report.entries.len(), 2);
        assert!(report.entries[0].snippet.contains("just a string"));
        assert_eq!(report.entries[1].snippet, "42");
    }

    #[test]
    fn read_payload_dir_returns_only_capture_v1_files() {
        let tmp = TempDir::new().expect("tempdir");
        let dir = tmp.path().join("payloads");
        fs::create_dir_all(&dir).expect("mkdir");
        fs::write(
            dir.join("capture_v1-abc.json"),
            r#"{"body":"hello","timestamp":"2026-04-24T01:00:00Z"}"#,
        )
        .unwrap();
        fs::write(dir.join("unrelated.json"), r#"{"unrelated":true}"#).unwrap();
        fs::write(dir.join("capture_v1-bad.json"), "not json").unwrap();
        let report = read_payload_dir(&dir);
        assert_eq!(report.entries.len(), 1, "only capture_v1-*.json succeeds");
        assert_eq!(report.skipped, 1, "bad payload file counts as skipped");
        assert_eq!(report.entries[0].kind, CaptureArtifactKind::Payload);
        assert!(report.entries[0].line_number.is_none());
        assert_eq!(
            report.entries[0].created_at.as_deref(),
            Some("2026-04-24T01:00:00Z")
        );
    }

    #[test]
    fn read_payload_dir_handles_missing_dir_gracefully() {
        let tmp = TempDir::new().expect("tempdir");
        let report = read_payload_dir(&tmp.path().join("never-existed"));
        assert!(report.entries.is_empty());
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn read_all_artifacts_combines_every_kind() {
        let tmp = TempDir::new().expect("tempdir");
        let sk = tmp.path().join(".scriptkit");
        let base = sk.join("menu-syntax");

        write_file(&base, "todos.jsonl", "{\"body\":\"t\"}\n");
        write_file(&base, "events.jsonl", "{\"body\":\"e\"}\n");
        write_file(&base, "notes.jsonl", "{\"body\":\"n\"}\n");
        write_file(&base, "drafts.jsonl", "{\"body\":\"d\"}\n");
        write_file(
            &base,
            "bookmarks.jsonl",
            "{\"body\":\"b\",\"url\":\"https://x\"}\n",
        );
        fs::create_dir_all(base.join("payloads")).unwrap();
        fs::write(
            base.join("payloads/capture_v1-1.json"),
            r#"{"body":"payload-body"}"#,
        )
        .unwrap();

        let report = read_all_artifacts(&sk);
        assert_eq!(report.entries.len(), 6);
        let kinds: Vec<_> = report.entries.iter().map(|e| e.kind).collect();
        assert_eq!(
            kinds,
            vec![
                CaptureArtifactKind::Todo,
                CaptureArtifactKind::CalendarEvent,
                CaptureArtifactKind::Note,
                CaptureArtifactKind::SocialDraft,
                CaptureArtifactKind::Bookmark,
                CaptureArtifactKind::Payload,
            ],
            "merge order matches BROWSER_ORDER then Payload last"
        );
    }

    #[test]
    fn read_all_artifacts_counts_warnings_across_files() {
        let tmp = TempDir::new().expect("tempdir");
        let sk = tmp.path().join(".scriptkit");
        let base = sk.join("menu-syntax");
        write_file(&base, "todos.jsonl", "bad json\n{\"body\":\"ok\"}\n");
        write_file(&base, "events.jsonl", "nope\n");
        let report = read_all_artifacts(&sk);
        assert_eq!(report.entries.len(), 1, "only valid lines surface");
        assert_eq!(report.skipped, 2, "dirty rows across files accumulate");
        assert_eq!(report.warnings.len(), 2);
    }

    #[test]
    fn warning_cap_prevents_unbounded_accumulation() {
        let tmp = TempDir::new().expect("tempdir");
        let mut buf = String::new();
        for _ in 0..(MAX_WARNINGS * 3) {
            buf.push_str("garbage\n");
        }
        let path = write_file(tmp.path(), "todos.jsonl", &buf);
        let report = read_jsonl_artifact(&path, CaptureArtifactKind::Todo);
        assert_eq!(report.entries.len(), 0);
        assert_eq!(
            report.warnings.len(),
            MAX_WARNINGS,
            "warnings must not grow unbounded"
        );
        assert!(
            report.skipped >= MAX_WARNINGS,
            "all dirty rows still count as skipped ({} rows)",
            report.skipped
        );
    }

    #[test]
    fn artifact_filename_for_matches_templates_and_shipped_examples() {
        assert_eq!(CaptureArtifactKind::Todo.filename(), "todos.jsonl");
        assert_eq!(
            CaptureArtifactKind::CalendarEvent.filename(),
            "events.jsonl"
        );
        assert_eq!(CaptureArtifactKind::Note.filename(), "notes.jsonl");
        assert_eq!(CaptureArtifactKind::SocialDraft.filename(), "drafts.jsonl");
        assert_eq!(CaptureArtifactKind::Bookmark.filename(), "bookmarks.jsonl");
        assert_eq!(CaptureArtifactKind::Payload.filename(), "payloads");
    }

    #[test]
    fn browser_order_excludes_payload() {
        assert!(
            !CaptureArtifactKind::BROWSER_ORDER
                .iter()
                .any(|k| *k == CaptureArtifactKind::Payload),
            "payload is retention-only, not in the user-facing browser order"
        );
    }
}
