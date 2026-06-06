//! Oracle-Session `protocol-builtin-boundary-refactor-plan` PR2:
//! protocol-level field deprecation registry.
//!
//! When a field on a message is renamed or replaced, we land the
//! replacement first and keep the old field working, but mark it in
//! [`DEPRECATED_FIELDS`] with the version in which it was deprecated
//! and an optional version in which it will be removed. Inbound
//! parsers can then emit a single structured warning per
//! `(message_kind, field)` pair, and outbound serializers can refuse
//! to emit deprecated fields once the current version meets
//! `remove_in`.
//!
//! This module is a data-only registry. PR2b will wire it into
//! parse-graceful and the `kit://diagnostics/protocol-stats` MCP
//! resource so automation can tell whether a deprecated shape is
//! still reaching the Rust side from the field.
//!
//! The initial row — `triggerBuiltin.name -> builtinId` — is the
//! concrete motivator: the current SDK publishes
//! `{"type":"triggerBuiltin","name":"X"}` while new code publishes
//! `{"type":"triggerBuiltin","builtinId":"builtin/X"}`. Pinning the
//! rename here means future PRs can drop the old field by flipping a
//! single `remove_in` slot rather than scrubbing the repo.

use serde_json::Value;
use thiserror::Error;

use crate::protocol::version::ProtocolVersion;

/// One deprecated field on one message kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeprecatedField {
    /// The message `type` discriminant this row applies to (for
    /// example `"triggerBuiltin"`).
    pub kind: &'static str,
    /// The deprecated field name on the JSON message body.
    pub field: &'static str,
    /// The replacement field name callers should migrate to, if any.
    pub replacement: Option<&'static str>,
    /// The version that first emitted a deprecation warning for this
    /// field.
    pub deprecated_since: u16,
    /// The version at which the field is removed. If `Some(v)` and
    /// the current wire version is `>= v`, a parser may reject the
    /// message outright.
    pub remove_in: Option<u16>,
}

/// The canonical list. Ordering is preserved for stable diagnostics
/// output; add new rows at the end.
pub const DEPRECATED_FIELDS: &[DeprecatedField] = &[DeprecatedField {
    kind: "triggerBuiltin",
    field: "name",
    replacement: Some("builtinId"),
    deprecated_since: 2,
    remove_in: Some(3),
}];

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProtocolDeprecationError {
    #[error(
        "message `{kind}` carried removed field `{field}` at protocolVersion {current} (removed in v{removed_in}){replacement_hint}"
    )]
    FieldRemoved {
        kind: &'static str,
        field: &'static str,
        current: u16,
        removed_in: u16,
        replacement_hint: ReplacementHint,
    },
}

/// Renders the `; replacement is \`builtinId\`` hint suffix when a
/// replacement is registered, and nothing otherwise. Having this as
/// its own type lets the `Display` impl on `ProtocolDeprecationError`
/// format inline without smuggling `Option<&str>` through the enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplacementHint(pub Option<&'static str>);

impl std::fmt::Display for ReplacementHint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(r) => write!(f, "; replacement is `{r}`"),
            None => Ok(()),
        }
    }
}

/// One inbound deprecation observation. Lets the caller emit a
/// structured warning exactly once per `(kind, field)` pair without
/// this module needing to know about logging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolDeprecationWarning {
    pub kind: &'static str,
    pub field: &'static str,
    pub replacement: Option<&'static str>,
    pub deprecated_since: u16,
    pub remove_in: Option<u16>,
}

/// Look up a deprecation row for a given `(kind, field)` pair.
pub fn lookup(kind: &str, field: &str) -> Option<&'static DeprecatedField> {
    DEPRECATED_FIELDS
        .iter()
        .find(|row| row.kind == kind && row.field == field)
}

/// Walk every field of a message body and return the deprecation
/// observations that apply to the current version. Returns an error
/// if any field has already been removed at the current version.
pub fn validate_deprecations(
    kind: &str,
    body: &Value,
    current: ProtocolVersion,
) -> Result<Vec<ProtocolDeprecationWarning>, ProtocolDeprecationError> {
    let Some(obj) = body.as_object() else {
        return Ok(Vec::new());
    };
    let mut warnings = Vec::new();
    for row in DEPRECATED_FIELDS {
        if row.kind != kind {
            continue;
        }
        if !obj.contains_key(row.field) {
            continue;
        }
        if let Some(removed_in) = row.remove_in {
            if current.get() >= removed_in {
                return Err(ProtocolDeprecationError::FieldRemoved {
                    kind: row.kind,
                    field: row.field,
                    current: current.get(),
                    removed_in,
                    replacement_hint: ReplacementHint(row.replacement),
                });
            }
        }
        warnings.push(ProtocolDeprecationWarning {
            kind: row.kind,
            field: row.field,
            replacement: row.replacement,
            deprecated_since: row.deprecated_since,
            remove_in: row.remove_in,
        });
    }
    Ok(warnings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn lookup_finds_trigger_builtin_name() {
        let row = lookup("triggerBuiltin", "name").expect("row");
        assert_eq!(row.replacement, Some("builtinId"));
        assert_eq!(row.deprecated_since, 2);
        assert_eq!(row.remove_in, Some(3));
    }

    #[test]
    fn lookup_misses_unknown() {
        assert!(lookup("triggerBuiltin", "flibber").is_none());
        assert!(lookup("unknownKind", "name").is_none());
    }

    #[test]
    fn validate_ignores_non_object() {
        let warns = validate_deprecations(
            "triggerBuiltin",
            &json!("bare"),
            ProtocolVersion::default_legacy(),
        )
        .unwrap();
        assert!(warns.is_empty());
    }

    #[test]
    fn validate_returns_no_warnings_when_field_absent() {
        let body = json!({ "builtinId": "builtin/clipboard-history" });
        let warns =
            validate_deprecations("triggerBuiltin", &body, ProtocolVersion::current()).unwrap();
        assert!(warns.is_empty());
    }

    #[test]
    fn validate_warns_on_deprecated_field_at_current_version() {
        let body = json!({ "name": "clipboard-history" });
        let warns =
            validate_deprecations("triggerBuiltin", &body, ProtocolVersion::current()).unwrap();
        assert_eq!(warns.len(), 1);
        assert_eq!(warns[0].field, "name");
        assert_eq!(warns[0].replacement, Some("builtinId"));
    }

    #[test]
    fn validate_errors_when_removed_version_reached() {
        let body = json!({ "name": "clipboard-history" });
        let err = validate_deprecations("triggerBuiltin", &body, ProtocolVersion::from_raw(3))
            .unwrap_err();
        match err {
            ProtocolDeprecationError::FieldRemoved {
                kind,
                field,
                current,
                removed_in,
                ..
            } => {
                assert_eq!(kind, "triggerBuiltin");
                assert_eq!(field, "name");
                assert_eq!(current, 3);
                assert_eq!(removed_in, 3);
            }
        }
    }

    #[test]
    fn validate_ignores_unrelated_kinds() {
        let body = json!({ "name": "whatever" });
        let warns = validate_deprecations("arg", &body, ProtocolVersion::current()).unwrap();
        assert!(warns.is_empty());
    }

    #[test]
    fn replacement_hint_renders() {
        assert_eq!(
            format!("{}", ReplacementHint(Some("builtinId"))),
            "; replacement is `builtinId`"
        );
        assert_eq!(format!("{}", ReplacementHint(None)), "");
    }

    #[test]
    fn no_row_references_an_unknown_replacement_as_itself() {
        for row in DEPRECATED_FIELDS {
            if let Some(replacement) = row.replacement {
                assert_ne!(
                    replacement, row.field,
                    "row for `{}.{}` replaces the field with itself",
                    row.kind, row.field
                );
            }
        }
    }
}
