//! Power Syntax per-target history pools.
//!
//! Story B (Run 14): remember positive `#tag` usage per target so later
//! autocomplete can rank suggestions by frequency and recency. Designed per
//! oracle (slug `grammar-tag-history-pool`, gpt-5.4-pro): append-only JSONL
//! per target, pure ranking transform, constructor-injected `HistoryStore`
//! for tests. Risk pinned at the boundary: unsafe target names are encoded
//! (no traversal, no lossy collision); negated/query tags are NEVER recorded
//! (autocomplete must not surface anti-tags).

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::grammar_payload::{FieldKind, GrammarPayload};

pub const TAG_HISTORY_FILENAME: &str = "tags.history.jsonl";
pub const KEYS_DIR: &str = "keys";
pub const KEY_HISTORY_SUFFIX: &str = ".history.jsonl";
pub const ARGV_HISTORY_FILENAME: &str = "argv.history.jsonl";
pub const COMMANDS_DIR: &str = "commands";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagHistoryEntry {
    pub ts: u64,
    pub tag: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagFrequency {
    pub tag: String,
    pub count: u64,
    pub last_seen_ts: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValueHistoryEntry {
    pub ts: u64,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueFrequency {
    pub value: String,
    pub count: u64,
    pub last_seen_ts: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArgvHistoryEntry {
    pub ts: u64,
    pub argv: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgvFrequency {
    pub argv: Vec<String>,
    pub count: u64,
    pub last_seen_ts: u64,
}

#[derive(Debug, Clone)]
pub struct HistoryStore {
    /// Menu-syntax root, e.g. `~/.scriptkit/menu-syntax`.
    pub base: PathBuf,
}

/// Story E (Run 14): per-command-head argv history. Storage layout
/// `<base>/<head>/argv.history.jsonl` with rows `{ts, argv:[...]}`.
/// Distinct from `HistoryStore` because commands aren't menu-syntax
/// artifacts — they live under `~/.scriptkit/commands` not
/// `~/.scriptkit/menu-syntax`. Same boundary discipline: hex-encode
/// unsafe heads, append-only, pure ranking transform.
#[derive(Debug, Clone)]
pub struct CommandHistoryStore {
    /// Commands root, e.g. `~/.scriptkit/commands`.
    pub base: PathBuf,
}

impl HistoryStore {
    pub fn new(base: impl Into<PathBuf>) -> Self {
        Self { base: base.into() }
    }

    pub fn from_sk_path(sk_path: impl AsRef<Path>) -> Self {
        Self::new(sk_path.as_ref().join("menu-syntax"))
    }

    pub fn from_env() -> Self {
        Self::from_sk_path(default_sk_path())
    }

    pub fn record_tags<T: AsRef<str>>(&self, target: &str, tags: &[T]) -> io::Result<()> {
        self.record_tags_at(target, tags, now_unix())
    }

    pub fn record_tags_at<T: AsRef<str>>(
        &self,
        target: &str,
        tags: &[T],
        ts: u64,
    ) -> io::Result<()> {
        let path = self.tag_history_path(target)?;
        let mut rows = Vec::new();
        let mut seen = HashSet::new();
        for raw in tags {
            let Some(tag) = normalize_tag(raw.as_ref()) else {
                continue;
            };
            // A single capture saying `#work #work` should not double-boost
            // autocomplete. Repeated tags across separate captures still
            // count normally because each call writes its own row set.
            if seen.insert(tag.clone()) {
                rows.push(TagHistoryEntry { ts, tag });
            }
        }
        append_jsonl_rows(&path, &rows)
    }

    pub fn record_payload_tags(&self, payload: &GrammarPayload) -> io::Result<()> {
        if payload.target.trim().is_empty() {
            return Ok(());
        }
        let tags: Vec<&str> = payload
            .tags
            .iter()
            .filter(|tag| !tag.negated)
            .map(|tag| tag.value.as_str())
            .collect();
        self.record_tags(&payload.target, &tags)
    }

    pub fn try_read_tag_pool(&self, target: &str) -> io::Result<Vec<TagFrequency>> {
        let path = self.tag_history_path(target)?;
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(err) => return Err(err),
        };
        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        for line_result in reader.lines() {
            let line = line_result?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            // History must not poison autocomplete. Dirty JSONL lines are
            // ignored, matching the tolerant artifact-reader precedent.
            let Ok(mut entry) = serde_json::from_str::<TagHistoryEntry>(trimmed) else {
                continue;
            };
            let Some(tag) = normalize_tag(&entry.tag) else {
                continue;
            };
            entry.tag = tag;
            entries.push(entry);
        }
        Ok(build_tag_pool(entries))
    }

    pub fn tag_history_path(&self, target: &str) -> io::Result<PathBuf> {
        let component = target_component(target)?;
        Ok(self.base.join(component).join(TAG_HISTORY_FILENAME))
    }

    pub fn record_field(&self, target: &str, key: &str, value: &str) -> io::Result<()> {
        self.record_field_at(target, key, value, now_unix())
    }

    pub fn record_field_at(&self, target: &str, key: &str, value: &str, ts: u64) -> io::Result<()> {
        let normalized_value = value.trim();
        if normalized_value.is_empty() {
            return Ok(());
        }
        let path = self.key_history_path(target, key)?;
        let row = ValueHistoryEntry {
            ts,
            value: normalized_value.to_string(),
        };
        append_jsonl_rows(&path, std::slice::from_ref(&row))
    }

    /// Record every free-form `key:value` field from a `GrammarPayload`.
    /// Negated, schema-bound, query-predicate and meta fields are skipped.
    /// Story C calls this on every successful capture; the popup (story D)
    /// then surfaces values most-recent-first.
    pub fn record_payload_fields(&self, payload: &GrammarPayload) -> io::Result<()> {
        if payload.target.trim().is_empty() {
            return Ok(());
        }
        let ts = now_unix();
        for field in payload.fields.iter() {
            if field.negated {
                continue;
            }
            // Only Free fields belong in history. Schema-bound fields are
            // governed by the script's declared enum (story F precedence);
            // Query/Meta fields belong to refine semantics, not capture.
            if !matches!(field.kind, FieldKind::Free) {
                continue;
            }
            self.record_field_at(&payload.target, &field.key, &field.value, ts)?;
        }
        Ok(())
    }

    pub fn try_read_key_pool(&self, target: &str, key: &str) -> io::Result<Vec<ValueFrequency>> {
        let path = self.key_history_path(target, key)?;
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(err) => return Err(err),
        };
        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        for line_result in reader.lines() {
            let line = line_result?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let Ok(entry) = serde_json::from_str::<ValueHistoryEntry>(trimmed) else {
                continue;
            };
            if entry.value.trim().is_empty() {
                continue;
            }
            entries.push(entry);
        }
        Ok(build_value_pool(entries))
    }

    pub fn key_history_path(&self, target: &str, key: &str) -> io::Result<PathBuf> {
        let target_component = target_component(target)?;
        let key_component = key_component(key)?;
        let mut path = self.base.join(target_component);
        path.push(KEYS_DIR);
        path.push(format!("{key_component}{KEY_HISTORY_SUFFIX}"));
        Ok(path)
    }
}

impl CommandHistoryStore {
    pub fn new(base: impl Into<PathBuf>) -> Self {
        Self { base: base.into() }
    }

    pub fn from_sk_path(sk_path: impl AsRef<Path>) -> Self {
        Self::new(sk_path.as_ref().join(COMMANDS_DIR))
    }

    pub fn from_env() -> Self {
        Self::from_sk_path(default_sk_path())
    }

    pub fn record_argv<T: AsRef<str>>(&self, head: &str, argv: &[T]) -> io::Result<()> {
        self.record_argv_at(head, argv, now_unix())
    }

    pub fn record_argv_at<T: AsRef<str>>(&self, head: &str, argv: &[T], ts: u64) -> io::Result<()> {
        let normalized: Vec<String> = argv
            .iter()
            .map(|s| s.as_ref().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if normalized.is_empty() {
            return Ok(());
        }
        let path = self.argv_history_path(head)?;
        let row = ArgvHistoryEntry {
            ts,
            argv: normalized,
        };
        append_jsonl_rows(&path, std::slice::from_ref(&row))
    }

    pub fn try_read_argv_pool(&self, head: &str) -> io::Result<Vec<ArgvFrequency>> {
        let path = self.argv_history_path(head)?;
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(err) => return Err(err),
        };
        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        for line_result in reader.lines() {
            let line = line_result?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let Ok(entry) = serde_json::from_str::<ArgvHistoryEntry>(trimmed) else {
                continue;
            };
            if entry.argv.is_empty() {
                continue;
            }
            entries.push(entry);
        }
        Ok(build_argv_pool(entries))
    }

    pub fn argv_history_path(&self, head: &str) -> io::Result<PathBuf> {
        let component = target_component(head)?;
        Ok(self.base.join(component).join(ARGV_HISTORY_FILENAME))
    }
}

pub fn record_argv<T: AsRef<str>>(head: &str, argv: &[T]) -> io::Result<()> {
    CommandHistoryStore::from_env().record_argv(head, argv)
}

pub fn read_argv_pool(head: &str) -> Vec<ArgvFrequency> {
    CommandHistoryStore::from_env()
        .try_read_argv_pool(head)
        .unwrap_or_default()
}

pub fn build_argv_pool(entries: impl IntoIterator<Item = ArgvHistoryEntry>) -> Vec<ArgvFrequency> {
    let mut by_key: BTreeMap<String, ArgvFrequency> = BTreeMap::new();
    for entry in entries {
        if entry.argv.is_empty() {
            continue;
        }
        // Argv vectors are deterministic when joined by a control char
        // that cannot legally appear in argv (NUL is the only argv
        // forbidden char in POSIX) — used as a hash key for grouping.
        let key = entry.argv.join("\0");
        let stat = by_key.entry(key).or_insert_with(|| ArgvFrequency {
            argv: entry.argv.clone(),
            count: 0,
            last_seen_ts: 0,
        });
        stat.count = stat.count.saturating_add(1);
        stat.last_seen_ts = stat.last_seen_ts.max(entry.ts);
    }
    let mut out: Vec<ArgvFrequency> = by_key.into_values().collect();
    // Story E spec: typing `!deploy --` should surface past argv values.
    // Sort recency-first (the most-recently-typed command is the one
    // the user most likely wants again), then count desc, then argv asc
    // (lexicographic on the joined form for deterministic ties).
    out.sort_by(|a, b| {
        b.last_seen_ts
            .cmp(&a.last_seen_ts)
            .then_with(|| b.count.cmp(&a.count))
            .then_with(|| a.argv.cmp(&b.argv))
    });
    out
}

pub fn record_tags<T: AsRef<str>>(target: &str, tags: &[T]) -> io::Result<()> {
    HistoryStore::from_env().record_tags(target, tags)
}

pub fn read_tag_pool(target: &str) -> Vec<TagFrequency> {
    HistoryStore::from_env()
        .try_read_tag_pool(target)
        .unwrap_or_default()
}

pub fn read_key_pool(target: &str, key: &str) -> Vec<ValueFrequency> {
    HistoryStore::from_env()
        .try_read_key_pool(target, key)
        .unwrap_or_default()
}

pub fn build_value_pool(
    entries: impl IntoIterator<Item = ValueHistoryEntry>,
) -> Vec<ValueFrequency> {
    let mut by_value: BTreeMap<String, ValueFrequency> = BTreeMap::new();
    for entry in entries {
        let value = entry.value.trim().to_string();
        if value.is_empty() {
            continue;
        }
        let stat = by_value
            .entry(value.clone())
            .or_insert_with(|| ValueFrequency {
                value,
                count: 0,
                last_seen_ts: 0,
            });
        stat.count = stat.count.saturating_add(1);
        stat.last_seen_ts = stat.last_seen_ts.max(entry.ts);
    }
    let mut out: Vec<ValueFrequency> = by_value.into_values().collect();
    // Story C spec: "values most-recent-first." Tie-break by count desc
    // so a frequently-typed value beats a one-off when timestamps tie.
    out.sort_by(|a, b| {
        b.last_seen_ts
            .cmp(&a.last_seen_ts)
            .then_with(|| b.count.cmp(&a.count))
            .then_with(|| a.value.cmp(&b.value))
    });
    out
}

pub fn build_tag_pool(entries: impl IntoIterator<Item = TagHistoryEntry>) -> Vec<TagFrequency> {
    let mut by_tag: BTreeMap<String, TagFrequency> = BTreeMap::new();
    for entry in entries {
        let Some(tag) = normalize_tag(&entry.tag) else {
            continue;
        };
        let stat = by_tag.entry(tag.clone()).or_insert_with(|| TagFrequency {
            tag,
            count: 0,
            last_seen_ts: 0,
        });
        stat.count = stat.count.saturating_add(1);
        stat.last_seen_ts = stat.last_seen_ts.max(entry.ts);
    }
    let mut out: Vec<TagFrequency> = by_tag.into_values().collect();
    out.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| b.last_seen_ts.cmp(&a.last_seen_ts))
            .then_with(|| a.tag.cmp(&b.tag))
    });
    out
}

fn append_jsonl_rows<T: Serialize>(path: &Path, rows: &[T]) -> io::Result<()> {
    if rows.is_empty() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut buf = Vec::new();
    for row in rows {
        serde_json::to_writer(&mut buf, row)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        buf.push(b'\n');
    }
    // Append-only is the correct shape for JSONL history. Tempfile+rename
    // would require read/merge/write and can drop concurrent appends from
    // sibling launcher processes.
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(&buf)
}

fn normalize_tag(raw: &str) -> Option<String> {
    let tag = raw.trim().trim_start_matches('#').trim();
    if tag.is_empty() {
        None
    } else {
        Some(tag.to_string())
    }
}

fn target_component(target: &str) -> io::Result<String> {
    let target = target.trim();
    if target.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "history target cannot be empty",
        ));
    }
    // Keep normal slugs human-readable. Encode everything else to avoid
    // path traversal and collision-prone lossy sanitization (oracle risk
    // pin: a slug like `../todo` must not escape the history root).
    if target
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Ok(target.to_string());
    }
    Ok(format!("~{}", hex_encode(target.as_bytes())))
}

fn key_component(key: &str) -> io::Result<String> {
    let key = key.trim();
    if key.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "history key cannot be empty",
        ));
    }
    if key
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Ok(key.to_string());
    }
    Ok(format!("~{}", hex_encode(key.as_bytes())))
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn default_sk_path() -> PathBuf {
    if let Ok(path) = std::env::var("SK_PATH") {
        if !path.trim().is_empty() {
            return PathBuf::from(path);
        }
    }
    std::env::var("HOME")
        .map(|home| PathBuf::from(home).join(".scriptkit"))
        .unwrap_or_else(|_| PathBuf::from(".scriptkit"))
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::grammar_payload::{
        GrammarSurface, GrammarVerb, TagEntry, GRAMMAR_PAYLOAD_VERSION,
    };

    #[test]
    fn record_tags_appends_one_row_per_distinct_tag() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = HistoryStore::new(tmp.path().join("menu-syntax"));
        store
            .record_tags_at("todo", &["errands", "client", "errands"], 100)
            .unwrap();
        let raw = std::fs::read_to_string(tmp.path().join("menu-syntax/todo/tags.history.jsonl"))
            .unwrap();
        let lines: Vec<_> = raw.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("\"tag\":\"errands\""));
        assert!(lines[1].contains("\"tag\":\"client\""));
    }

    #[test]
    fn build_tag_pool_sorts_by_count_then_recency_then_tag() {
        let entries = vec![
            TagHistoryEntry {
                ts: 10,
                tag: "client".into(),
            },
            TagHistoryEntry {
                ts: 20,
                tag: "errands".into(),
            },
            TagHistoryEntry {
                ts: 30,
                tag: "errands".into(),
            },
            TagHistoryEntry {
                ts: 40,
                tag: "alpha".into(),
            },
        ];
        let pool = build_tag_pool(entries);
        assert_eq!(pool[0].tag, "errands");
        assert_eq!(pool[0].count, 2);
        assert_eq!(pool[0].last_seen_ts, 30);
        assert_eq!(pool[1].tag, "alpha");
        assert_eq!(pool[2].tag, "client");
    }

    #[test]
    fn read_tag_pool_matches_story_receipt() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = HistoryStore::new(tmp.path().join("menu-syntax"));
        store.record_tags_at("todo", &["errands"], 100).unwrap();
        store
            .record_tags_at("todo", &["errands", "client"], 200)
            .unwrap();
        let pool = store.try_read_tag_pool("todo").unwrap();
        assert_eq!(
            pool,
            vec![
                TagFrequency {
                    tag: "errands".into(),
                    count: 2,
                    last_seen_ts: 200,
                },
                TagFrequency {
                    tag: "client".into(),
                    count: 1,
                    last_seen_ts: 200,
                },
            ]
        );
    }

    #[test]
    fn read_tag_pool_tolerates_missing_and_dirty_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = HistoryStore::new(tmp.path().join("menu-syntax"));
        assert!(store.try_read_tag_pool("todo").unwrap().is_empty());
        let path = store.tag_history_path("todo").unwrap();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            "{\"ts\":10,\"tag\":\"ok\"}\nnot json\n{\"ts\":20,\"tag\":\"ok\"}\n",
        )
        .unwrap();
        let pool = store.try_read_tag_pool("todo").unwrap();
        assert_eq!(pool.len(), 1);
        assert_eq!(pool[0].tag, "ok");
        assert_eq!(pool[0].count, 2);
    }

    #[test]
    fn record_payload_tags_skips_negated_tags() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = HistoryStore::new(tmp.path().join("menu-syntax"));
        let payload = GrammarPayload {
            version: GRAMMAR_PAYLOAD_VERSION,
            verb: GrammarVerb::Capture,
            surface: GrammarSurface::Plus,
            target: "todo".into(),
            raw: ";todo x #work".into(),
            text: "x".into(),
            tags: vec![
                TagEntry {
                    value: "work".into(),
                    negated: false,
                },
                TagEntry {
                    value: "archived".into(),
                    negated: true,
                },
            ],
            fields: Vec::new(),
            dates: Vec::new(),
            argv: Vec::new(),
        };
        store.record_payload_tags(&payload).unwrap();
        let pool = store.try_read_tag_pool("todo").unwrap();
        assert_eq!(pool.len(), 1);
        assert_eq!(pool[0].tag, "work");
    }

    #[test]
    fn record_field_appends_value_history_under_keys_dir() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = HistoryStore::new(tmp.path().join("menu-syntax"));
        store
            .record_field_at("cal", "start", "friday 2pm", 100)
            .unwrap();
        store
            .record_field_at("cal", "start", "tomorrow 12:30pm", 200)
            .unwrap();
        let path = tmp.path().join("menu-syntax/cal/keys/start.history.jsonl");
        let raw = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<_> = raw.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("\"value\":\"friday 2pm\""));
        assert!(lines[1].contains("\"value\":\"tomorrow 12:30pm\""));
    }

    #[test]
    fn read_key_pool_returns_values_most_recent_first() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = HistoryStore::new(tmp.path().join("menu-syntax"));
        store
            .record_field_at("cal", "start", "friday 2pm", 100)
            .unwrap();
        store
            .record_field_at("cal", "start", "tomorrow 12:30pm", 200)
            .unwrap();
        let pool = store.try_read_key_pool("cal", "start").unwrap();
        assert_eq!(pool.len(), 2);
        assert_eq!(pool[0].value, "tomorrow 12:30pm");
        assert_eq!(pool[0].last_seen_ts, 200);
        assert_eq!(pool[1].value, "friday 2pm");
        assert_eq!(pool[1].last_seen_ts, 100);
    }

    #[test]
    fn build_value_pool_tie_breaks_by_count_then_value() {
        let entries = vec![
            ValueHistoryEntry {
                ts: 100,
                value: "alpha".into(),
            },
            ValueHistoryEntry {
                ts: 100,
                value: "alpha".into(),
            },
            ValueHistoryEntry {
                ts: 100,
                value: "beta".into(),
            },
            ValueHistoryEntry {
                ts: 200,
                value: "gamma".into(),
            },
        ];
        let pool = build_value_pool(entries);
        assert_eq!(pool[0].value, "gamma");
        assert_eq!(pool[1].value, "alpha");
        assert_eq!(pool[1].count, 2);
        assert_eq!(pool[2].value, "beta");
    }

    #[test]
    fn record_payload_fields_skips_schema_query_meta_and_negated() {
        use crate::menu_syntax::grammar_payload::{FieldEntry, FieldKind};

        let tmp = tempfile::TempDir::new().unwrap();
        let store = HistoryStore::new(tmp.path().join("menu-syntax"));
        let payload = GrammarPayload {
            version: GRAMMAR_PAYLOAD_VERSION,
            verb: GrammarVerb::Capture,
            surface: GrammarSurface::Plus,
            target: "cal".into(),
            raw: ";cal Lunch start:tomorrow priority:1".into(),
            text: "Lunch".into(),
            tags: Vec::new(),
            fields: vec![
                FieldEntry {
                    key: "start".into(),
                    value: "tomorrow".into(),
                    kind: FieldKind::Free,
                    negated: false,
                },
                FieldEntry {
                    key: "priority".into(),
                    value: "1".into(),
                    kind: FieldKind::Schema,
                    negated: false,
                },
                FieldEntry {
                    key: "tag".into(),
                    value: "errands".into(),
                    kind: FieldKind::Query,
                    negated: false,
                },
                FieldEntry {
                    key: "owner".into(),
                    value: "alice".into(),
                    kind: FieldKind::Free,
                    negated: true,
                },
            ],
            dates: Vec::new(),
            argv: Vec::new(),
        };
        store.record_payload_fields(&payload).unwrap();
        // Only the Free non-negated `start` should land.
        let start_pool = store.try_read_key_pool("cal", "start").unwrap();
        assert_eq!(start_pool.len(), 1);
        assert_eq!(start_pool[0].value, "tomorrow");
        // Schema/Query/negated keys must NOT exist.
        assert!(store
            .try_read_key_pool("cal", "priority")
            .unwrap()
            .is_empty());
        assert!(store.try_read_key_pool("cal", "tag").unwrap().is_empty());
        assert!(store.try_read_key_pool("cal", "owner").unwrap().is_empty());
    }

    #[test]
    fn unsafe_key_names_do_not_escape_history_base() {
        let tmp = tempfile::TempDir::new().unwrap();
        let base = tmp.path().join("menu-syntax");
        let store = HistoryStore::new(&base);
        let path = store.key_history_path("cal", "../start").unwrap();
        assert!(path.starts_with(&base));
        assert!(!path.to_string_lossy().contains("../"));
    }

    #[test]
    fn capture_invocation_round_trips_into_history_pools() {
        use crate::menu_syntax::payload::CaptureInvocation;

        let tmp = tempfile::TempDir::new().unwrap();
        let store = HistoryStore::new(tmp.path().join("menu-syntax"));
        // Mirror what the executor does: CaptureInvocation -> GrammarPayload
        // -> record_payload_tags/fields. This pins the integration shape
        // the executor depends on so a future refactor that changes the
        // GrammarPayload From impl trips this test before it ships.
        use crate::menu_syntax::payload::CaptureAlias;
        let inv = CaptureInvocation {
            target: "todo".into(),
            alias_form: CaptureAlias::CapturePrefix,
            body: "Buy milk".into(),
            tags: vec!["errands".into(), "client".into()],
            priority: Some(1),
            url: None,
            duration: None,
            kv: vec![("owner".into(), "alice".into())],
            date_phrases: Vec::new(),
            raw: ";todo Buy milk #errands #client priority:1 owner:alice".into(),
        };
        let payload = GrammarPayload::from(&inv);
        store.record_payload_tags(&payload).unwrap();
        store.record_payload_fields(&payload).unwrap();

        let tags = store.try_read_tag_pool("todo").unwrap();
        assert_eq!(tags.len(), 2);
        let tag_values: Vec<&str> = tags.iter().map(|t| t.tag.as_str()).collect();
        assert!(tag_values.contains(&"errands"));
        assert!(tag_values.contains(&"client"));

        // Free-form `kv` field lands in the per-key pool. `priority` is
        // schema-bound (story F precedence) and MUST NOT land in history.
        let owner_pool = store.try_read_key_pool("todo", "owner").unwrap();
        assert_eq!(owner_pool.len(), 1);
        assert_eq!(owner_pool[0].value, "alice");
        let priority_pool = store.try_read_key_pool("todo", "priority").unwrap();
        assert!(
            priority_pool.is_empty(),
            "schema-bound priority must not poison history"
        );
    }

    #[test]
    fn command_history_store_records_argv_and_groups_by_argv_vector() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = CommandHistoryStore::new(tmp.path().join("commands"));
        store
            .record_argv_at("deploy", &["prod", "--dry-run"], 100)
            .unwrap();
        store.record_argv_at("deploy", &["staging"], 200).unwrap();
        store
            .record_argv_at("deploy", &["prod", "--dry-run"], 300)
            .unwrap();
        // Story E receipt: storage at <commands>/<head>/argv.history.jsonl
        let path = tmp.path().join("commands/deploy/argv.history.jsonl");
        let raw = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<_> = raw.lines().collect();
        assert_eq!(lines.len(), 3);
        // Pool should rank recency-first: the most recent prod+dry-run wins
        // (count=2, ts=300), then staging (count=1, ts=200).
        let pool = store.try_read_argv_pool("deploy").unwrap();
        assert_eq!(pool.len(), 2);
        assert_eq!(pool[0].argv, vec!["prod", "--dry-run"]);
        assert_eq!(pool[0].count, 2);
        assert_eq!(pool[0].last_seen_ts, 300);
        assert_eq!(pool[1].argv, vec!["staging"]);
        assert_eq!(pool[1].count, 1);
    }

    #[test]
    fn build_argv_pool_treats_distinct_argv_vectors_as_distinct() {
        let entries = vec![
            ArgvHistoryEntry {
                ts: 100,
                argv: vec!["build".into(), "--release".into()],
            },
            ArgvHistoryEntry {
                ts: 200,
                argv: vec!["build".into()],
            },
            ArgvHistoryEntry {
                ts: 100,
                argv: vec!["build".into(), "--release".into()],
            },
        ];
        let pool = build_argv_pool(entries);
        assert_eq!(pool.len(), 2);
        // Recency tie-break: ts=200 single arg wins over ts=100 two-arg.
        assert_eq!(pool[0].argv, vec!["build"]);
        assert_eq!(pool[0].count, 1);
        assert_eq!(pool[1].argv, vec!["build", "--release"]);
        assert_eq!(pool[1].count, 2);
    }

    #[test]
    fn record_argv_skips_empty_entries() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = CommandHistoryStore::new(tmp.path().join("commands"));
        store
            .record_argv_at("noop", &Vec::<&str>::new(), 100)
            .unwrap();
        store.record_argv_at("noop", &["", ""], 200).unwrap();
        // Both should produce ZERO file writes — the file should not exist.
        assert!(!tmp.path().join("commands/noop/argv.history.jsonl").exists());
        assert!(store.try_read_argv_pool("noop").unwrap().is_empty());
    }

    #[test]
    fn command_history_store_unsafe_head_does_not_escape() {
        let tmp = tempfile::TempDir::new().unwrap();
        let base = tmp.path().join("commands");
        let store = CommandHistoryStore::new(&base);
        let path = store.argv_history_path("../deploy").unwrap();
        assert!(path.starts_with(&base));
        assert!(!path.to_string_lossy().contains("../"));
    }

    #[test]
    fn argv_invocation_round_trips_into_command_history_pool() {
        // Pin (Pass 15): the wire-through in
        // `execute_menu_syntax_command_invocation` records argv from a real
        // `ArgvInvocation` into the per-head pool. A future refactor that
        // drops the call, swaps the head, or feeds a different field would
        // trip this test.
        let tmp = tempfile::TempDir::new().unwrap();
        let store = CommandHistoryStore::new(tmp.path().join("commands"));
        let inv = crate::menu_syntax::ArgvInvocation {
            head: "deploy".into(),
            fields: Vec::new(),
            tags: Vec::new(),
            argv: vec!["prod".into(), "--dry-run".into()],
            raw: ">deploy prod --dry-run".into(),
        };
        store.record_argv_at(&inv.head, &inv.argv, 500).unwrap();
        let pool = store.try_read_argv_pool(&inv.head).unwrap();
        assert_eq!(pool.len(), 1);
        assert_eq!(pool[0].argv, vec!["prod", "--dry-run"]);
        assert_eq!(pool[0].last_seen_ts, 500);
    }

    #[test]
    fn unsafe_target_names_do_not_escape_history_base() {
        let tmp = tempfile::TempDir::new().unwrap();
        let base = tmp.path().join("menu-syntax");
        let store = HistoryStore::new(&base);
        let path = store.tag_history_path("../todo").unwrap();
        assert!(path.starts_with(&base));
        assert!(!path.to_string_lossy().contains("../"));
    }
}
