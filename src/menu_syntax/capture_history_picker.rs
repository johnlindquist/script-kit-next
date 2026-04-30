//! Run 14 Pass 9 — capture-composer history picker (data layer).
//!
//! Story D `grammar-popup-autocomplete-ui` (slice 1 of N): when the cursor
//! is parked on a bare `#` or a fresh `key:` inside an active capture
//! composer (e.g. `+todo Buy milk #` or `+cal Lunch start:`), this module
//! builds a `HistoryPickerSnapshot` from the per-target tag and key:value
//! pools shipped in Pass 6 and Pass 7. UI render + keyboard wiring +
//! `getState` exposure land in follow-up passes — this slice ships pure
//! detection + snapshot building + tests so the UI pass has a stable
//! contract to consume.
//!
//! The detection is intentionally narrow: only fires when the cursor is
//! exactly at the slot the user just opened (`#` with nothing after, or
//! `key:` with nothing after). Mid-word positions return `None` so the
//! popup never fights with regular typing.

use serde::{Deserialize, Serialize};

use super::history::{HistoryStore, TagFrequency, ValueFrequency};
use super::schema_overrides::{merge_enum_with_history, SlotValueSource};

/// What kind of history slot the cursor is parked on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum HistoryPickerKind {
    /// Cursor on a bare `#` token — show the tag pool for `target`.
    TagSlot,
    /// Cursor on a fresh `key:` token — show the value pool for `(target, key)`.
    KeySlot { key: String },
}

/// One row in the popup. `count` and `last_seen_ts` are passed through
/// from the underlying frequency aggregate so the UI can show "× N" or
/// "5 min ago" affordances later.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryPickerRow {
    pub value: String,
    pub count: u64,
    pub last_seen_ts: u64,
    /// Run 14 Pass 18: when populated by
    /// [`build_history_picker_snapshot_with_overrides`], names the row's
    /// origin so the UI can dim history-only rows or badge schema rows.
    /// Absent (`None`) for the legacy code path that pre-dates schema
    /// override merging — older callers and serializers see no shape change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SlotValueSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryPickerSnapshot {
    pub target: String,
    #[serde(flatten)]
    pub kind: HistoryPickerKind,
    pub rows: Vec<HistoryPickerRow>,
}

/// Detect whether `input[..cursor]` ends in a bare `#` or `key:` slot
/// inside an active capture body for `target`. Returns `None` when the
/// cursor is mid-word, before any slot trigger, or when no target is
/// active.
///
/// Examples (cursor `^`):
/// - `+todo Buy milk #^`            → `Some(TagSlot)`
/// - `+cal Lunch start:^`           → `Some(KeySlot { key: "start" })`
/// - `+todo Buy milk #errands^`     → `None` (mid-tag)
/// - `+todo #foo bar^`              → `None` (cursor in body, not slot)
/// - `+cal start: tomorrow^`        → `None` (value already typed)
pub fn detect_history_picker_context(input: &str, cursor: usize) -> Option<HistoryPickerKind> {
    if cursor == 0 || cursor > input.len() {
        return None;
    }
    let prefix = &input[..cursor];

    // Tag slot: prefix must end in `#` and the char before (if any) must
    // be whitespace or the start of input. `+todo Buy #` ✓; `+todo #foo`
    // mid-word, prefix ends in `o` not `#`, so naturally falls through.
    if let Some(rest) = prefix.strip_suffix('#') {
        let prev = rest.chars().next_back();
        if prev.map_or(true, |ch| ch.is_whitespace()) {
            return Some(HistoryPickerKind::TagSlot);
        }
    }

    // Key slot: prefix must end in `key:` where `key` is one ASCII word
    // since the last whitespace and `key:` is followed by NOTHING (the
    // value slot is empty).
    if prefix.ends_with(':') {
        let without_colon = &prefix[..prefix.len() - 1];
        // The char before `:` must be a key char — block `::` and ` :`.
        let last = without_colon.chars().next_back()?;
        if !is_key_char(last) {
            return None;
        }
        let key_start = without_colon
            .rfind(|ch: char| ch.is_whitespace() || ch == ':')
            .map(|i| i + 1)
            .unwrap_or(0);
        let key = &without_colon[key_start..];
        if key.is_empty() || !key.chars().all(is_key_char) {
            return None;
        }
        return Some(HistoryPickerKind::KeySlot {
            key: key.to_string(),
        });
    }

    None
}

fn is_key_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.'
}

/// Build a popup snapshot from the per-target history pools. Returns
/// `None` when `target` is empty OR when the resulting pool has no rows
/// (the UI should not render an empty popup).
pub fn build_history_picker_snapshot(
    target: &str,
    kind: &HistoryPickerKind,
    store: &HistoryStore,
) -> Option<HistoryPickerSnapshot> {
    if target.trim().is_empty() {
        return None;
    }
    let rows: Vec<HistoryPickerRow> = match kind {
        HistoryPickerKind::TagSlot => {
            let pool = store.try_read_tag_pool(target).ok()?;
            pool.into_iter().map(tag_row).collect()
        }
        HistoryPickerKind::KeySlot { key } => {
            let pool = store.try_read_key_pool(target, key).ok()?;
            pool.into_iter().map(value_row).collect()
        }
    };
    if rows.is_empty() {
        return None;
    }
    Some(HistoryPickerSnapshot {
        target: target.to_string(),
        kind: kind.clone(),
        rows,
    })
}

/// Compute a snapshot directly from `filter_text`, treating the cursor as
/// at end-of-input. Convenience for `getState` callers where the cursor
/// position is implicit in the just-typed input. Returns `None` when
/// no slot trigger fires, no active capture target can be parsed, or
/// the resulting pool is empty.
pub fn snapshot_from_filter_text(
    filter_text: &str,
    store: &HistoryStore,
) -> Option<HistoryPickerSnapshot> {
    use super::parse::parse;
    use super::payload::IncompleteKind;
    let kind = detect_history_picker_context(filter_text, filter_text.len())?;
    let target = match parse(filter_text) {
        super::parse::MenuSyntaxParse::Capture(inv) => inv.target,
        super::parse::MenuSyntaxParse::Incomplete(s) => match s.kind {
            IncompleteKind::MissingCaptureBody(t) => t,
            _ => return None,
        },
        _ => return None,
    };
    if target.is_empty() {
        return None;
    }
    build_history_picker_snapshot(&target, &kind, store)
}

/// Run 14 Pass 19: schema-aware companion to [`snapshot_from_filter_text`].
/// Resolves the active capture target + slot exactly like the legacy
/// helper, then asks `lookup_enum_values(target, key)` for the schema-
/// declared enum values for this `(target, key)` pair (only consulted
/// for KeySlot kinds). The returned snapshot routes through
/// [`build_history_picker_snapshot_with_overrides`] so each emitted
/// row carries a `Some(SlotValueSource)` discriminator. Empty lookup
/// output yields the same shape as the legacy path (source stays None).
pub fn snapshot_from_filter_text_with_overrides<F>(
    filter_text: &str,
    store: &HistoryStore,
    lookup_enum_values: F,
) -> Option<HistoryPickerSnapshot>
where
    F: FnOnce(&str, &str) -> Vec<String>,
{
    use super::parse::parse;
    use super::payload::IncompleteKind;
    let kind = detect_history_picker_context(filter_text, filter_text.len())?;
    let target = match parse(filter_text) {
        super::parse::MenuSyntaxParse::Capture(inv) => inv.target,
        super::parse::MenuSyntaxParse::Incomplete(s) => match s.kind {
            IncompleteKind::MissingCaptureBody(t) => t,
            _ => return None,
        },
        _ => return None,
    };
    if target.is_empty() {
        return None;
    }
    let key_enum_values: Vec<String> = match &kind {
        HistoryPickerKind::KeySlot { key } => lookup_enum_values(&target, key),
        HistoryPickerKind::TagSlot => Vec::new(),
    };
    build_history_picker_snapshot_with_overrides(&target, &kind, store, &key_enum_values)
}

fn tag_row(freq: TagFrequency) -> HistoryPickerRow {
    HistoryPickerRow {
        value: freq.tag,
        count: freq.count,
        last_seen_ts: freq.last_seen_ts,
        source: None,
    }
}

fn value_row(freq: ValueFrequency) -> HistoryPickerRow {
    HistoryPickerRow {
        value: freq.value,
        count: freq.count,
        last_seen_ts: freq.last_seen_ts,
        source: None,
    }
}

/// Like [`build_history_picker_snapshot`] but merges schema-declared enum
/// values into the KeySlot pool via
/// [`super::schema_overrides::merge_enum_with_history`]. Each emitted row
/// carries a `Some(SlotValueSource)` so the UI can render schema rows
/// blessed and history-only rows dimmed. TagSlot is unaffected (no enum
/// concept for free-form tags); empty `key_enum_values` falls through to
/// the legacy builder so the source field stays `None` for backward
/// compatibility.
pub fn build_history_picker_snapshot_with_overrides(
    target: &str,
    kind: &HistoryPickerKind,
    store: &HistoryStore,
    key_enum_values: &[String],
) -> Option<HistoryPickerSnapshot> {
    if target.trim().is_empty() {
        return None;
    }
    match kind {
        HistoryPickerKind::TagSlot => build_history_picker_snapshot(target, kind, store),
        HistoryPickerKind::KeySlot { key } => {
            if key_enum_values.is_empty() {
                return build_history_picker_snapshot(target, kind, store);
            }
            let pool = store.try_read_key_pool(target, key).ok()?;
            let merged = merge_enum_with_history(key_enum_values, &pool);
            if merged.is_empty() {
                return None;
            }
            let rows: Vec<HistoryPickerRow> = merged
                .into_iter()
                .map(|r| HistoryPickerRow {
                    value: r.value,
                    count: r.count,
                    last_seen_ts: r.last_seen_ts,
                    source: Some(r.source),
                })
                .collect();
            Some(HistoryPickerSnapshot {
                target: target.to_string(),
                kind: kind.clone(),
                rows,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_bare_hash_after_whitespace() {
        let input = ";todo Buy milk #";
        let kind = detect_history_picker_context(input, input.len()).unwrap();
        assert_eq!(kind, HistoryPickerKind::TagSlot);
    }

    #[test]
    fn detects_bare_hash_at_start_of_capture_body() {
        // `+todo #` — hash is the first non-whitespace char of the body.
        let input = ";todo #";
        let kind = detect_history_picker_context(input, input.len()).unwrap();
        assert_eq!(kind, HistoryPickerKind::TagSlot);
    }

    #[test]
    fn does_not_detect_mid_tag() {
        let input = ";todo Buy milk #errands";
        assert!(detect_history_picker_context(input, input.len()).is_none());
    }

    #[test]
    fn detects_fresh_key_slot() {
        let input = ";cal Lunch start:";
        let kind = detect_history_picker_context(input, input.len()).unwrap();
        assert_eq!(
            kind,
            HistoryPickerKind::KeySlot {
                key: "start".into()
            }
        );
    }

    #[test]
    fn does_not_detect_when_value_already_typed() {
        let input = ";cal start:tomorrow";
        assert!(detect_history_picker_context(input, input.len()).is_none());
    }

    #[test]
    fn does_not_detect_double_colon_or_space_colon() {
        assert!(detect_history_picker_context(";cal start::", 12).is_none());
        assert!(detect_history_picker_context(";cal :", 6).is_none());
    }

    #[test]
    fn snapshot_returns_none_when_no_history_exists() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = HistoryStore::new(tmp.path().join("menu-syntax"));
        assert!(
            build_history_picker_snapshot("todo", &HistoryPickerKind::TagSlot, &store).is_none()
        );
        assert!(build_history_picker_snapshot(
            "cal",
            &HistoryPickerKind::KeySlot {
                key: "start".into(),
            },
            &store
        )
        .is_none());
    }

    #[test]
    fn snapshot_ranks_tags_by_count_desc_then_recency() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = HistoryStore::new(tmp.path().join("menu-syntax"));
        store.record_tags_at("todo", &["errands"], 100).unwrap();
        store
            .record_tags_at("todo", &["errands", "client"], 200)
            .unwrap();
        let snap =
            build_history_picker_snapshot("todo", &HistoryPickerKind::TagSlot, &store).unwrap();
        assert_eq!(snap.target, "todo");
        assert_eq!(snap.rows.len(), 2);
        assert_eq!(snap.rows[0].value, "errands");
        assert_eq!(snap.rows[0].count, 2);
        assert_eq!(snap.rows[1].value, "client");
        assert_eq!(snap.rows[1].count, 1);
    }

    #[test]
    fn snapshot_returns_key_values_most_recent_first() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = HistoryStore::new(tmp.path().join("menu-syntax"));
        store
            .record_field_at("cal", "start", "friday 2pm", 100)
            .unwrap();
        store
            .record_field_at("cal", "start", "tomorrow 12:30pm", 200)
            .unwrap();
        let snap = build_history_picker_snapshot(
            "cal",
            &HistoryPickerKind::KeySlot {
                key: "start".into(),
            },
            &store,
        )
        .unwrap();
        assert_eq!(snap.rows.len(), 2);
        assert_eq!(snap.rows[0].value, "tomorrow 12:30pm");
        assert_eq!(snap.rows[1].value, "friday 2pm");
    }

    #[test]
    fn snapshot_from_filter_text_resolves_active_capture_target() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = HistoryStore::new(tmp.path().join("menu-syntax"));
        store.record_tags_at("todo", &["errands"], 100).unwrap();
        let snap = snapshot_from_filter_text(";todo Buy milk #", &store).unwrap();
        assert_eq!(snap.target, "todo");
        assert_eq!(snap.rows.len(), 1);
        assert_eq!(snap.rows[0].value, "errands");
    }

    #[test]
    fn snapshot_from_filter_text_returns_none_when_no_slot_trigger() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = HistoryStore::new(tmp.path().join("menu-syntax"));
        store.record_tags_at("todo", &["errands"], 100).unwrap();
        assert!(snapshot_from_filter_text(";todo Buy milk", &store).is_none());
    }

    #[test]
    fn with_overrides_emits_schema_first_and_history_only_dimmed() {
        // Pass 18 wire-through: when a key has declared enum values, the
        // popup ranks the enum first (in declared order) and tags
        // history-only values with `HistoryOnly` so the UI can dim them.
        let tmp = tempfile::TempDir::new().unwrap();
        let store = HistoryStore::new(tmp.path().join("menu-syntax"));
        store
            .record_field_at("deploy", "env", "custom", 100)
            .unwrap();
        let enum_values = vec!["prod".into(), "staging".into(), "dev".into()];
        let snap = build_history_picker_snapshot_with_overrides(
            "deploy",
            &HistoryPickerKind::KeySlot { key: "env".into() },
            &store,
            &enum_values,
        )
        .unwrap();
        assert_eq!(snap.rows.len(), 4);
        assert_eq!(snap.rows[0].value, "prod");
        assert_eq!(snap.rows[0].source, Some(SlotValueSource::SchemaEnum));
        assert_eq!(snap.rows[1].value, "staging");
        assert_eq!(snap.rows[2].value, "dev");
        assert_eq!(snap.rows[3].value, "custom");
        assert_eq!(snap.rows[3].source, Some(SlotValueSource::HistoryOnly));
    }

    #[test]
    fn with_overrides_empty_enum_falls_through_to_legacy_builder() {
        // Empty `key_enum_values` MUST behave exactly like the legacy
        // `build_history_picker_snapshot` so existing callers can adopt
        // the new function unchanged. In particular, the `source` field
        // stays `None` (skip-if-none) so the JSON shape is identical.
        let tmp = tempfile::TempDir::new().unwrap();
        let store = HistoryStore::new(tmp.path().join("menu-syntax"));
        store
            .record_field_at("cal", "start", "friday 2pm", 100)
            .unwrap();
        let snap = build_history_picker_snapshot_with_overrides(
            "cal",
            &HistoryPickerKind::KeySlot {
                key: "start".into(),
            },
            &store,
            &[],
        )
        .unwrap();
        assert_eq!(snap.rows.len(), 1);
        assert_eq!(snap.rows[0].value, "friday 2pm");
        assert!(snap.rows[0].source.is_none());
    }

    #[test]
    fn with_overrides_tag_slot_ignores_enum_values() {
        // Free-form tags have no enum concept, so even if a caller
        // accidentally passes enum_values for a TagSlot, we delegate to
        // the legacy tag builder and keep `source: None`.
        let tmp = tempfile::TempDir::new().unwrap();
        let store = HistoryStore::new(tmp.path().join("menu-syntax"));
        store.record_tags_at("todo", &["errands"], 100).unwrap();
        let snap = build_history_picker_snapshot_with_overrides(
            "todo",
            &HistoryPickerKind::TagSlot,
            &store,
            &["should".into(), "be".into(), "ignored".into()],
        )
        .unwrap();
        assert_eq!(snap.rows.len(), 1);
        assert_eq!(snap.rows[0].value, "errands");
        assert!(snap.rows[0].source.is_none());
    }

    #[test]
    fn with_overrides_pure_schema_when_history_empty() {
        // A schema with no history sightings still emits the schema rows
        // so the user can pick a blessed value on first use.
        let tmp = tempfile::TempDir::new().unwrap();
        let store = HistoryStore::new(tmp.path().join("menu-syntax"));
        let enum_values = vec!["prod".into(), "staging".into()];
        let snap = build_history_picker_snapshot_with_overrides(
            "deploy",
            &HistoryPickerKind::KeySlot { key: "env".into() },
            &store,
            &enum_values,
        )
        .unwrap();
        assert_eq!(snap.rows.len(), 2);
        assert!(snap
            .rows
            .iter()
            .all(|r| r.source == Some(SlotValueSource::SchemaEnum)));
    }

    #[test]
    fn snapshot_from_filter_text_with_overrides_passes_lookup_into_picker() {
        // Pass 19: a non-trivial lookup must propagate enum values into
        // the snapshot. Schema rows precede history-only rows; source
        // discriminators flow through.
        let tmp = tempfile::TempDir::new().unwrap();
        let store = HistoryStore::new(tmp.path().join("menu-syntax"));
        store.record_field_at("todo", "env", "custom", 100).unwrap();
        let lookup = |target: &str, key: &str| {
            assert_eq!(target, "todo");
            assert_eq!(key, "env");
            vec!["prod".into(), "staging".into()]
        };
        // `+todo` is a registered builtin capture target (todo/note/link/cal/social).
        // The popup parser resolves it; the env:-slot is just any key.
        let snap = snapshot_from_filter_text_with_overrides(";todo Roll out env:", &store, lookup)
            .unwrap();
        assert_eq!(snap.target, "todo");
        assert_eq!(snap.rows.len(), 3);
        assert_eq!(snap.rows[0].value, "prod");
        assert_eq!(snap.rows[0].source, Some(SlotValueSource::SchemaEnum));
        assert_eq!(snap.rows[1].value, "staging");
        assert_eq!(snap.rows[2].value, "custom");
        assert_eq!(snap.rows[2].source, Some(SlotValueSource::HistoryOnly));
    }

    #[test]
    fn snapshot_from_filter_text_with_overrides_skips_lookup_for_tag_slot() {
        // TagSlot kinds have no enum concept; the lookup must NOT be
        // called. Use a panicking lookup to assert this.
        let tmp = tempfile::TempDir::new().unwrap();
        let store = HistoryStore::new(tmp.path().join("menu-syntax"));
        store.record_tags_at("todo", &["errands"], 100).unwrap();
        let lookup = |_: &str, _: &str| -> Vec<String> {
            panic!("lookup must not be called for TagSlot");
        };
        let snap =
            snapshot_from_filter_text_with_overrides(";todo Buy milk #", &store, lookup).unwrap();
        assert_eq!(snap.target, "todo");
        assert_eq!(snap.rows.len(), 1);
        assert_eq!(snap.rows[0].value, "errands");
        assert!(snap.rows[0].source.is_none());
    }

    #[test]
    fn snapshot_from_filter_text_returns_none_when_no_active_target() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = HistoryStore::new(tmp.path().join("menu-syntax"));
        // No `+target ` prefix means no active capture target.
        assert!(snapshot_from_filter_text("Buy milk #", &store).is_none());
    }

    #[test]
    fn empty_target_returns_none() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = HistoryStore::new(tmp.path().join("menu-syntax"));
        store.record_tags_at("todo", &["errands"], 100).unwrap();
        assert!(build_history_picker_snapshot("", &HistoryPickerKind::TagSlot, &store).is_none());
        assert!(
            build_history_picker_snapshot("   ", &HistoryPickerKind::TagSlot, &store).is_none()
        );
    }
}
