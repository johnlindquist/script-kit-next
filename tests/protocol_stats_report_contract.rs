//! Oracle-Session `protocol-builtin-boundary-refactor-plan` PR4:
//! contract tests for the `kit://diagnostics/protocol-stats` resource
//! and the [`ProtocolStatsReport`] shape.
//!
//! These tests are deliberately kept at the top level (not under
//! `source_audits/`) because the resource is a live JSON contract
//! that an MCP consumer can read today — a regression here is a
//! protocol break, not a source-style drift.

use script_kit_gpui::mcp_resources::{get_resource_definitions, read_resource};
use script_kit_gpui::protocol_stats::{
    current_report, health, ProtocolStatsHealth, ProtocolStatsReport, ProtocolStatsThresholds,
    StatsSnapshot,
};
use serde_json::Value;

const RESOURCE_URI: &str = "kit://diagnostics/protocol-stats";

fn zero_snapshot() -> StatsSnapshot {
    StatsSnapshot {
        stdin_parse_failed_total: 0,
        stdin_command_too_large_total: 0,
        stdin_unsupported_protocol_version_total: 0,
        trigger_builtin_unknown_total: 0,
        trigger_builtin_deprecated_name_total: 0,
    }
}

#[test]
fn resource_is_registered_in_definitions() {
    let defs = get_resource_definitions();
    let found = defs.iter().find(|r| r.uri == RESOURCE_URI);
    let resource = found.unwrap_or_else(|| panic!("resource not in definitions: {RESOURCE_URI}"));
    assert_eq!(resource.mime_type, "application/json");
    assert_eq!(resource.name, "Protocol Stats");
    assert!(
        resource.description.is_some(),
        "resource must carry a description so MCP clients can render it"
    );
}

#[test]
fn read_resource_returns_parseable_report() {
    let content = read_resource(RESOURCE_URI, &[], &[], None)
        .expect("kit://diagnostics/protocol-stats must be readable");
    assert_eq!(content.uri, RESOURCE_URI);
    assert_eq!(content.mime_type, "application/json");

    let report: ProtocolStatsReport = serde_json::from_str(&content.text)
        .expect("resource body must deserialize as ProtocolStatsReport");
    assert_eq!(report.thresholds, ProtocolStatsThresholds::current());
    // The snapshot may be nonzero in a shared-state test run, but health
    // should at least be internally consistent.
    let recomputed = health(&report.snapshot);
    assert_eq!(recomputed.ok, report.health.ok);
    assert_eq!(recomputed.flags, report.health.flags);
}

#[test]
fn resource_body_carries_expected_top_level_keys() {
    let content = read_resource(RESOURCE_URI, &[], &[], None).expect("resource read must succeed");
    let parsed: Value =
        serde_json::from_str(&content.text).expect("resource body must be valid JSON");
    let obj = parsed
        .as_object()
        .expect("resource body must be a JSON object");
    for key in ["snapshot", "health", "thresholds"] {
        assert!(
            obj.contains_key(key),
            "resource body must have top-level key `{key}`; got keys {:?}",
            obj.keys().collect::<Vec<_>>()
        );
    }
    let health = obj.get("health").and_then(|v| v.as_object()).unwrap();
    for key in ["ok", "flags"] {
        assert!(
            health.contains_key(key),
            "health object must expose `{key}`"
        );
    }
    let thresholds = obj.get("thresholds").and_then(|v| v.as_object()).unwrap();
    for key in [
        "stdinParseFailed",
        "stdinCommandTooLarge",
        "stdinUnsupportedProtocolVersion",
        "triggerBuiltinUnknown",
        "triggerBuiltinDeprecatedName",
    ] {
        assert!(
            thresholds.contains_key(key),
            "thresholds object must expose `{key}` (camelCase)"
        );
    }
}

#[test]
fn health_is_ok_when_all_counters_are_zero() {
    let h = health(&zero_snapshot());
    assert!(h.ok);
    assert!(h.flags.is_empty());
}

#[test]
fn health_flag_ordering_is_declaration_order() {
    // Fire every counter above its threshold; flags must come back in
    // the same order they are defined on StatsSnapshot.
    let s = StatsSnapshot {
        stdin_parse_failed_total: 9999,
        stdin_command_too_large_total: 9999,
        stdin_unsupported_protocol_version_total: 9999,
        trigger_builtin_unknown_total: 9999,
        trigger_builtin_deprecated_name_total: u64::MAX,
    };
    let h = health(&s);
    assert_eq!(
        h,
        ProtocolStatsHealth {
            ok: false,
            flags: vec![
                "stdin_parse_failed".to_string(),
                "stdin_command_too_large".to_string(),
                "stdin_unsupported_protocol_version".to_string(),
                "trigger_builtin_unknown_flood".to_string(),
                "trigger_builtin_deprecated_name_flood".to_string(),
            ]
        }
    );
}

#[test]
fn current_report_matches_explicit_health_call() {
    let report = current_report();
    let recomputed = health(&report.snapshot);
    assert_eq!(recomputed, report.health);
}
