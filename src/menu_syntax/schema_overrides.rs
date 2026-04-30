//! Story F (`grammar-schema-overrides-history`): pure ranking transform that
//! merges schema-declared enum values with free-form history pool rows for
//! the autocomplete popup. Schema enums always rank first in declared order;
//! history-only values appear after, marked so the UI can render them dimmed
//! ("previously used"). Values appearing in BOTH enum and history keep their
//! enum slot but carry the history count/recency for tie-breaking.
//!
//! See [[lat.md/menu-syntax#Menu Syntax#Schema Overrides History]].

use serde::{Deserialize, Serialize};

use crate::menu_syntax::history::ValueFrequency;
use crate::menu_syntax::payload::MenuSyntaxHandlerSpec;

/// Run 14 Pass 21 (`grammar-capture-key-enum-data-source`): walk a slice
/// of `capture.v1` handler specs and return the first non-empty
/// `kv_enums[key]` whose `targets` list contains `target` or `*`. Empty
/// vector when no spec matches — caller falls through to pure-history
/// ranking. Target match is case-insensitive on a trimmed slug.
pub fn capture_kv_enum_values_for_specs(
    target: &str,
    key: &str,
    specs: &[&MenuSyntaxHandlerSpec],
) -> Vec<String> {
    let target_lc = target.trim().to_ascii_lowercase();
    if target_lc.is_empty() || key.is_empty() {
        return Vec::new();
    }
    for spec in specs {
        if spec.family != "capture.v1" {
            continue;
        }
        let target_match = spec
            .targets
            .iter()
            .any(|t| t.trim() == "*" || t.trim().eq_ignore_ascii_case(&target_lc));
        if !target_match {
            continue;
        }
        if let Some(values) = spec.kv_enums.get(key) {
            if !values.is_empty() {
                return values.clone();
            }
        }
    }
    Vec::new()
}

/// Where a popup row came from. Drives UI emphasis: schema rows render
/// at full opacity; history-only rows render dimmed with a "previously used"
/// hint so the user can tell which values are blessed by the schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SlotValueSource {
    /// Declared in the schema only — never typed by the user.
    SchemaEnum,
    /// Declared in the schema AND seen in history. Render as schema; the
    /// `count`/`last_seen_ts` fields can drive a subtle "frequent" badge.
    SchemaEnumWithHistory,
    /// Free-form history value not present in the schema enum. Render dimmed.
    HistoryOnly,
}

/// One row produced by [`merge_enum_with_history`]. Order in the returned
/// `Vec` is the popup's render order (schema first in declared order, then
/// history-only sorted by recency-desc → count-desc — same shape as the
/// underlying value pool).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankedSlotValue {
    pub value: String,
    pub source: SlotValueSource,
    /// Zero when source is `SchemaEnum` (no history sighting).
    pub count: u64,
    /// Zero when source is `SchemaEnum` (no history sighting).
    pub last_seen_ts: u64,
}

/// Merge the schema-declared enum with the user's history pool.
///
/// - All `enum_values` appear first, in their declared order. Each enum row
///   is annotated `SchemaEnum` if absent from history, `SchemaEnumWithHistory`
///   if present (with the matching `count`/`last_seen_ts`).
/// - Remaining `history` rows (those not in the enum) appear after, in the
///   order they were given (callers pass `try_read_key_pool` output, which is
///   already recency-desc → count-desc per [[src/menu_syntax/history.rs#build_value_pool]]).
/// - Empty `enum_values` falls through to a pure-history list (every row is
///   `HistoryOnly`). This is the same shape `capture_history_picker` already
///   ships, so callers without a schema can use this transform uniformly.
pub fn merge_enum_with_history(
    enum_values: &[String],
    history: &[ValueFrequency],
) -> Vec<RankedSlotValue> {
    let mut out: Vec<RankedSlotValue> = Vec::with_capacity(enum_values.len() + history.len());

    for value in enum_values {
        let matched = history.iter().find(|h| &h.value == value);
        let (source, count, last_seen_ts) = match matched {
            Some(h) => (
                SlotValueSource::SchemaEnumWithHistory,
                h.count,
                h.last_seen_ts,
            ),
            None => (SlotValueSource::SchemaEnum, 0, 0),
        };
        out.push(RankedSlotValue {
            value: value.clone(),
            source,
            count,
            last_seen_ts,
        });
    }

    for entry in history {
        if enum_values.iter().any(|v| v == &entry.value) {
            continue;
        }
        out.push(RankedSlotValue {
            value: entry.value.clone(),
            source: SlotValueSource::HistoryOnly,
            count: entry.count,
            last_seen_ts: entry.last_seen_ts,
        });
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn capture_spec(target: &str, key: &str, values: &[&str]) -> MenuSyntaxHandlerSpec {
        let mut kv = BTreeMap::new();
        kv.insert(
            key.to_string(),
            values.iter().map(|s| s.to_string()).collect(),
        );
        MenuSyntaxHandlerSpec {
            family: "capture.v1".into(),
            targets: vec![target.into()],
            kv_enums: kv,
            ..Default::default()
        }
    }

    #[test]
    fn capture_kv_enum_values_returns_first_match() {
        let s = capture_spec("todo", "env", &["prod", "staging"]);
        let got = capture_kv_enum_values_for_specs("todo", "env", &[&s]);
        assert_eq!(got, vec!["prod".to_string(), "staging".to_string()]);
    }

    #[test]
    fn capture_kv_enum_values_matches_wildcard_target() {
        let s = capture_spec("*", "priority", &["P0", "P1", "P2"]);
        let got = capture_kv_enum_values_for_specs("anything", "priority", &[&s]);
        assert_eq!(
            got,
            vec!["P0".to_string(), "P1".to_string(), "P2".to_string()]
        );
    }

    #[test]
    fn capture_kv_enum_values_skips_command_family_and_unmatched_targets() {
        let cmd = MenuSyntaxHandlerSpec {
            family: "command.v1".into(),
            head: Some("deploy".into()),
            ..Default::default()
        };
        let unmatched = capture_spec("note", "env", &["should_be_ignored"]);
        let got = capture_kv_enum_values_for_specs("todo", "env", &[&cmd, &unmatched]);
        assert!(got.is_empty());
    }

    #[test]
    fn capture_kv_enum_values_returns_empty_for_empty_target_or_key() {
        let s = capture_spec("todo", "env", &["prod"]);
        assert!(capture_kv_enum_values_for_specs("", "env", &[&s]).is_empty());
        assert!(capture_kv_enum_values_for_specs("todo", "", &[&s]).is_empty());
    }

    fn vf(value: &str, count: u64, ts: u64) -> ValueFrequency {
        ValueFrequency {
            value: value.into(),
            count,
            last_seen_ts: ts,
        }
    }

    #[test]
    fn schema_enum_appears_first_in_declared_order() {
        let enum_values = vec!["prod".into(), "staging".into(), "dev".into()];
        let history = vec![vf("custom", 1, 100)];
        let merged = merge_enum_with_history(&enum_values, &history);
        assert_eq!(merged.len(), 4);
        assert_eq!(merged[0].value, "prod");
        assert_eq!(merged[0].source, SlotValueSource::SchemaEnum);
        assert_eq!(merged[1].value, "staging");
        assert_eq!(merged[2].value, "dev");
        assert_eq!(merged[3].value, "custom");
        assert_eq!(merged[3].source, SlotValueSource::HistoryOnly);
    }

    #[test]
    fn enum_value_with_history_carries_count_and_recency() {
        let enum_values = vec!["prod".into(), "staging".into()];
        let history = vec![vf("prod", 5, 200), vf("staging", 1, 100)];
        let merged = merge_enum_with_history(&enum_values, &history);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].value, "prod");
        assert_eq!(merged[0].source, SlotValueSource::SchemaEnumWithHistory);
        assert_eq!(merged[0].count, 5);
        assert_eq!(merged[0].last_seen_ts, 200);
        assert_eq!(merged[1].source, SlotValueSource::SchemaEnumWithHistory);
    }

    #[test]
    fn history_only_values_preserve_input_order() {
        // Caller passes recency-desc history; merge must preserve that order
        // for the history-only tail since the UI ranks by it.
        let enum_values = vec!["prod".into()];
        let history = vec![
            vf("most-recent", 1, 500),
            vf("middle", 3, 300),
            vf("oldest", 2, 100),
        ];
        let merged = merge_enum_with_history(&enum_values, &history);
        assert_eq!(merged.len(), 4);
        assert_eq!(merged[0].value, "prod");
        assert_eq!(merged[1].value, "most-recent");
        assert_eq!(merged[2].value, "middle");
        assert_eq!(merged[3].value, "oldest");
    }

    #[test]
    fn empty_enum_falls_through_to_pure_history() {
        let history = vec![vf("alpha", 2, 200), vf("beta", 1, 100)];
        let merged = merge_enum_with_history(&[], &history);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].value, "alpha");
        assert_eq!(merged[0].source, SlotValueSource::HistoryOnly);
        assert_eq!(merged[1].source, SlotValueSource::HistoryOnly);
    }

    #[test]
    fn empty_history_returns_pure_schema_enum() {
        let enum_values = vec!["prod".into(), "staging".into()];
        let merged = merge_enum_with_history(&enum_values, &[]);
        assert_eq!(merged.len(), 2);
        assert!(merged
            .iter()
            .all(|r| r.source == SlotValueSource::SchemaEnum));
        assert!(merged.iter().all(|r| r.count == 0 && r.last_seen_ts == 0));
    }

    #[test]
    fn duplicate_history_value_is_emitted_once_under_enum_slot() {
        // A value declared in the enum AND in history must not appear in
        // BOTH the enum block and the history-only tail. The story spec
        // explicitly rules out double-rendering of `prod` if it's both an
        // enum value and a frequent history value.
        let enum_values = vec!["prod".into()];
        let history = vec![vf("prod", 9, 900), vf("custom", 1, 100)];
        let merged = merge_enum_with_history(&enum_values, &history);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].value, "prod");
        assert_eq!(merged[0].source, SlotValueSource::SchemaEnumWithHistory);
        assert_eq!(merged[0].count, 9);
        assert_eq!(merged[1].value, "custom");
        assert_eq!(merged[1].source, SlotValueSource::HistoryOnly);
    }
}
