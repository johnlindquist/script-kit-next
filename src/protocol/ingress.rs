//! Oracle-Session `protocol-builtin-boundary-refactor-plan` PR8:
//! additive pure ingress observer that composes the existing
//! [`crate::protocol::version::read_wire_version`] and
//! [`crate::protocol::deprecations::validate_deprecations`] helpers
//! into one structured view over an inbound JSONL line.
//!
//! PR8 landed [`ObservedIngress`] / [`observe_ingress`] /
//! [`observe_value`] as pure plumbing; PR8b added the first side-
//! effecting wrapper, [`record_unsupported_version`], and wired it
//! into [`crate::protocol::io`]'s per-line reader loop so the
//! `stdin_unsupported_protocol_version_total` counter bumps the
//! first time a peer sends an out-of-range envelope. The pure
//! entry points remain available for follow-up wiring passes (for
//! example to expose deprecation warnings from the slow path of
//! `parse_message_graceful` without re-deriving the shape each
//! time). This split — landing pure helpers first, wiring each
//! counter in a second pass — mirrors PR5b/PR5c earlier in the
//! same Oracle session.

use serde_json::Value;
use thiserror::Error;

use crate::protocol::deprecations::{
    validate_deprecations, ProtocolDeprecationError, ProtocolDeprecationWarning,
};
use crate::protocol::version::{read_wire_version, ProtocolVersion, ProtocolVersionError};

/// Structured observation over an inbound JSONL line *after* it has
/// parsed to a JSON value. Carries the envelope version, any
/// deprecation warnings that applied at that version, and an
/// optional fatal deprecation error (a removed field at a too-new
/// version).
///
/// Missing `type` yields `kind = None` with empty warnings — the
/// deprecation table is keyed on message kind, so an un-tagged body
/// cannot trigger field-level deprecations. Callers that require a
/// tagged `type` should check `kind.is_some()` before acting.
#[derive(Debug, PartialEq, Eq)]
pub struct ObservedIngress {
    pub kind: Option<String>,
    pub version: ProtocolVersion,
    pub warnings: Vec<ProtocolDeprecationWarning>,
    pub deprecation_error: Option<ProtocolDeprecationError>,
}

/// Anything that can go wrong *before* the deprecation walk can run:
/// JSON parse failure, non-object root, or an unsupported
/// `protocolVersion` envelope. Keeping these separate from
/// [`ProtocolDeprecationError`] means a caller that only wants the
/// wire-version health check can ignore the deprecation outcome.
#[derive(Debug, PartialEq, Eq, Error)]
pub enum IngressObserveError {
    #[error("failed to parse line as JSON: {0}")]
    InvalidJson(String),
    #[error("message root is not a JSON object")]
    NotObject,
    #[error(transparent)]
    Version(#[from] ProtocolVersionError),
}

/// Parse `line` as JSON, then run [`observe_value`].
pub fn observe_ingress(line: &str) -> Result<ObservedIngress, IngressObserveError> {
    let value: Value =
        serde_json::from_str(line).map_err(|e| IngressObserveError::InvalidJson(e.to_string()))?;
    observe_value(&value)
}

/// Side-effecting ingress helper wired into the stdin reader
/// ([`crate::protocol::io`]) as the pre-flight version check. Oracle-Session
/// `protocol-builtin-boundary-refactor-plan` PR8b.
///
/// Bumps [`crate::protocol_stats::PROTOCOL_STATS`]`.stdin_unsupported_protocol_version_total`
/// *iff* the line's envelope parses to an out-of-range version
/// ([`ProtocolVersionError::Unsupported`]). Emits exactly one
/// rate-limited `tracing::warn!` per `(stdin_unsupported_protocol_version, found)`
/// key per 30s window, following the safe-user-value contract
/// established in Oracle-Session `logging-observability-next-pass` PR1.
///
/// Fire-and-forget — no return, no caller-visible behavior change. The
/// stats counter has a zero-tolerance threshold
/// ([`crate::protocol_stats::STDIN_UNSUPPORTED_PROTOCOL_VERSION_THRESHOLD`]),
/// so the MCP `kit://diagnostics/protocol-stats` health flag flips to
/// `!ok` the first time an unsupported envelope arrives.
///
/// Intentionally narrow: does not bump counters for bad-JSON,
/// non-object root, or deprecation warnings. Those paths either
/// cannot be version-classified or are already counted elsewhere
/// (the deprecation-name counter is bumped at BuiltinRef resolution).
pub fn record_unsupported_version(line: &str) {
    let Err(err) = observe_ingress(line) else {
        return;
    };
    let IngressObserveError::Version(ProtocolVersionError::Unsupported { found }) = err else {
        return;
    };
    let total = crate::protocol_stats::increment(
        &crate::protocol_stats::PROTOCOL_STATS.stdin_unsupported_protocol_version_total,
    );
    let rate =
        crate::logging::log_rate_limit("stdin_unsupported_protocol_version", &found.to_string());
    if !rate.emit {
        return;
    }
    tracing::warn!(
        category = "STDIN",
        event_type = "stdin_unsupported_protocol_version",
        found_version = found,
        suppressed = rate.suppressed,
        occurrences_total = total,
        "stdin JSONL line carries unsupported protocolVersion — counter bumped, message still dispatched"
    );
}

/// Variant for callers that already parsed the JSON (for example the
/// classification slow path inside
/// [`crate::protocol::io`]). Skips the string parse and composes
/// `read_wire_version` + `validate_deprecations` over the same value.
pub fn observe_value(value: &Value) -> Result<ObservedIngress, IngressObserveError> {
    let Some(obj) = value.as_object() else {
        return Err(IngressObserveError::NotObject);
    };
    let version = read_wire_version(value)?;
    let kind = obj
        .get("type")
        .and_then(|t| t.as_str())
        .map(|s| s.to_string());
    let (warnings, deprecation_error) = match kind.as_deref() {
        Some(k) => match validate_deprecations(k, value, version) {
            Ok(warns) => (warns, None),
            Err(err) => (Vec::new(), Some(err)),
        },
        None => (Vec::new(), None),
    };
    Ok(ObservedIngress {
        kind,
        version,
        warnings,
        deprecation_error,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::version::CURRENT_PROTOCOL_VERSION;

    #[test]
    fn well_formed_v2_trigger_builtin_has_no_warnings() {
        let line = r#"{"type":"triggerBuiltin","builtinId":"builtin/clipboard-history","protocolVersion":2}"#;
        let obs = observe_ingress(line).unwrap();
        assert_eq!(obs.kind.as_deref(), Some("triggerBuiltin"));
        assert_eq!(obs.version.get(), CURRENT_PROTOCOL_VERSION);
        assert!(obs.warnings.is_empty());
        assert!(obs.deprecation_error.is_none());
    }

    #[test]
    fn legacy_name_on_v1_warns_but_does_not_error() {
        let line = r#"{"type":"triggerBuiltin","name":"clipboardHistory"}"#;
        let obs = observe_ingress(line).unwrap();
        assert_eq!(obs.version, ProtocolVersion::default_legacy());
        assert_eq!(obs.warnings.len(), 1);
        assert_eq!(obs.warnings[0].field, "name");
        assert_eq!(obs.warnings[0].replacement, Some("builtinId"));
        assert!(obs.deprecation_error.is_none());
    }

    #[test]
    fn legacy_name_on_v2_still_warns() {
        let line = r#"{"type":"triggerBuiltin","name":"clipboardHistory","protocolVersion":2}"#;
        let obs = observe_ingress(line).unwrap();
        assert_eq!(obs.warnings.len(), 1);
        assert!(obs.deprecation_error.is_none());
    }

    #[test]
    fn invalid_json_is_invalid_json_error() {
        let err = observe_ingress("{not json").unwrap_err();
        assert!(matches!(err, IngressObserveError::InvalidJson(_)));
    }

    #[test]
    fn non_object_root_is_not_object_error() {
        let err = observe_ingress("\"bare string\"").unwrap_err();
        assert!(matches!(err, IngressObserveError::NotObject));
    }

    #[test]
    fn unsupported_version_is_version_error() {
        let line = r#"{"type":"arg","protocolVersion":999}"#;
        let err = observe_ingress(line).unwrap_err();
        match err {
            IngressObserveError::Version(ProtocolVersionError::Unsupported { found }) => {
                assert_eq!(found, 999);
            }
            other => panic!("expected Unsupported version, got {other:?}"),
        }
    }

    #[test]
    fn missing_type_returns_none_kind_with_no_warnings() {
        let line = r#"{"protocolVersion":2}"#;
        let obs = observe_ingress(line).unwrap();
        assert!(obs.kind.is_none());
        assert!(obs.warnings.is_empty());
        assert!(obs.deprecation_error.is_none());
    }

    #[test]
    fn unrelated_kind_never_warns() {
        let line = r#"{"type":"arg","name":"whatever"}"#;
        let obs = observe_ingress(line).unwrap();
        assert_eq!(obs.kind.as_deref(), Some("arg"));
        assert!(obs.warnings.is_empty());
    }

    #[test]
    fn observe_value_matches_observe_ingress() {
        // Composition sanity: `observe_value` and `observe_ingress`
        // must agree for every well-formed line. Keeps the two
        // entry points from silently diverging.
        let line = r#"{"type":"triggerBuiltin","name":"clipboardHistory","protocolVersion":2}"#;
        let from_line = observe_ingress(line).unwrap();
        let value: Value = serde_json::from_str(line).unwrap();
        let from_value = observe_value(&value).unwrap();
        assert_eq!(from_line, from_value);
    }

    mod record {
        //! PR8b counter-bumping tests. These touch the global
        //! `PROTOCOL_STATS` so they reset the counter first and
        //! serialize via a process-wide mutex — concurrent
        //! `cargo test --lib` threads would otherwise race on the
        //! atomic and produce flaky deltas.
        use super::super::*;
        use crate::protocol_stats::{self, PROTOCOL_STATS};
        use std::sync::atomic::Ordering;
        use std::sync::Mutex;

        static TEST_LOCK: Mutex<()> = Mutex::new(());

        fn reset_and_snapshot() -> u64 {
            protocol_stats::reset_for_test();
            PROTOCOL_STATS
                .stdin_unsupported_protocol_version_total
                .load(Ordering::Relaxed)
        }

        #[test]
        fn valid_v2_line_does_not_bump() {
            let _guard = TEST_LOCK.lock().unwrap();
            let before = reset_and_snapshot();
            record_unsupported_version(
                r#"{"type":"triggerBuiltin","builtinId":"builtin/clipboard-history","protocolVersion":2}"#,
            );
            let after = PROTOCOL_STATS
                .stdin_unsupported_protocol_version_total
                .load(Ordering::Relaxed);
            assert_eq!(after, before, "valid v2 must not bump the counter");
        }

        #[test]
        fn legacy_no_envelope_does_not_bump() {
            let _guard = TEST_LOCK.lock().unwrap();
            let before = reset_and_snapshot();
            record_unsupported_version(r#"{"type":"triggerBuiltin","name":"clipboardHistory"}"#);
            let after = PROTOCOL_STATS
                .stdin_unsupported_protocol_version_total
                .load(Ordering::Relaxed);
            assert_eq!(
                after, before,
                "absent protocolVersion implies v1 and must not bump the counter"
            );
        }

        #[test]
        fn unsupported_version_bumps_counter_once() {
            let _guard = TEST_LOCK.lock().unwrap();
            reset_and_snapshot();
            record_unsupported_version(r#"{"type":"arg","protocolVersion":999}"#);
            let after = PROTOCOL_STATS
                .stdin_unsupported_protocol_version_total
                .load(Ordering::Relaxed);
            assert_eq!(after, 1, "one unsupported line must bump the counter by 1");
        }

        #[test]
        fn malformed_json_does_not_bump() {
            let _guard = TEST_LOCK.lock().unwrap();
            let before = reset_and_snapshot();
            record_unsupported_version("{not json");
            let after = PROTOCOL_STATS
                .stdin_unsupported_protocol_version_total
                .load(Ordering::Relaxed);
            assert_eq!(
                after, before,
                "bad JSON is a structural failure, not a version violation"
            );
        }

        #[test]
        fn non_object_root_does_not_bump() {
            let _guard = TEST_LOCK.lock().unwrap();
            let before = reset_and_snapshot();
            record_unsupported_version("\"bare string\"");
            let after = PROTOCOL_STATS
                .stdin_unsupported_protocol_version_total
                .load(Ordering::Relaxed);
            assert_eq!(after, before);
        }

        #[test]
        fn non_integer_version_does_not_bump_this_counter() {
            // InvalidType is a shape error, not an unsupported-range
            // error — it's classified upstream and not this counter's
            // responsibility.
            let _guard = TEST_LOCK.lock().unwrap();
            let before = reset_and_snapshot();
            record_unsupported_version(r#"{"type":"arg","protocolVersion":"two"}"#);
            let after = PROTOCOL_STATS
                .stdin_unsupported_protocol_version_total
                .load(Ordering::Relaxed);
            assert_eq!(after, before);
        }
    }
}
