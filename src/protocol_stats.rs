//! Protocol-boundary counters.
//!
//! Atomic counters for the Rust↔Bun JSONL ingress and the built-in dispatch
//! path. Hooked by [`src/stdin_commands/mod.rs`] and
//! [`src/builtins/trigger_dispatch.rs`] so regressions surface as a visible
//! metric instead of only as log noise.
//!
//! Also provides a rate-limited emit helper so a hostile or buggy peer cannot
//! flood `app.log` with O(size_of_payload) lines — we log the first
//! occurrence and every 100th thereafter, keeping a single structured
//! breadcrumb without the noise.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct ProtocolStats {
    pub stdin_parse_failed_total: AtomicU64,
    pub stdin_command_too_large_total: AtomicU64,
    pub stdin_unsupported_protocol_version_total: AtomicU64,
    pub trigger_builtin_unknown_total: AtomicU64,
    pub trigger_builtin_deprecated_name_total: AtomicU64,
}

pub static PROTOCOL_STATS: ProtocolStats = ProtocolStats {
    stdin_parse_failed_total: AtomicU64::new(0),
    stdin_command_too_large_total: AtomicU64::new(0),
    stdin_unsupported_protocol_version_total: AtomicU64::new(0),
    trigger_builtin_unknown_total: AtomicU64::new(0),
    trigger_builtin_deprecated_name_total: AtomicU64::new(0),
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsSnapshot {
    pub stdin_parse_failed_total: u64,
    pub stdin_command_too_large_total: u64,
    pub stdin_unsupported_protocol_version_total: u64,
    pub trigger_builtin_unknown_total: u64,
    pub trigger_builtin_deprecated_name_total: u64,
}

/// Health thresholds (Oracle-Session
/// `protocol-builtin-boundary-refactor-plan` PR4).
///
/// A counter exceeding its threshold contributes a named flag to
/// [`ProtocolStatsHealth::flags`] and flips `ok` to `false`. These
/// thresholds are also emitted in the `kit://diagnostics/protocol-stats`
/// resource body so MCP consumers never need to hardcode them.
///
/// Zero-tolerance counters (`parse_failed`, `command_too_large`,
/// `unsupported_protocol_version`) fire on the first occurrence — a
/// single hit means the ingress contract has been violated. The
/// deprecated-name counter has generous headroom because Bun scripts
/// on older SDKs still emit it every call; only a flood indicates a
/// real regression.
pub const STDIN_PARSE_FAILED_THRESHOLD: u64 = 0;
pub const STDIN_COMMAND_TOO_LARGE_THRESHOLD: u64 = 0;
pub const STDIN_UNSUPPORTED_PROTOCOL_VERSION_THRESHOLD: u64 = 0;
pub const TRIGGER_BUILTIN_UNKNOWN_THRESHOLD: u64 = 10;
pub const TRIGGER_BUILTIN_DEPRECATED_NAME_THRESHOLD: u64 = 10_000;

/// A machine-readable health summary over a [`StatsSnapshot`]. The
/// menu-bar health chip and MCP diagnostics resource both key on
/// `ok`; `flags` enumerates every counter that crossed its threshold
/// so operators can jump straight at the regression.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolStatsHealth {
    pub ok: bool,
    pub flags: Vec<String>,
}

/// Thresholds surfaced to MCP clients alongside the snapshot so the
/// MCP consumer does not have to track the Rust constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolStatsThresholds {
    pub stdin_parse_failed: u64,
    pub stdin_command_too_large: u64,
    pub stdin_unsupported_protocol_version: u64,
    pub trigger_builtin_unknown: u64,
    pub trigger_builtin_deprecated_name: u64,
}

impl ProtocolStatsThresholds {
    pub const fn current() -> Self {
        Self {
            stdin_parse_failed: STDIN_PARSE_FAILED_THRESHOLD,
            stdin_command_too_large: STDIN_COMMAND_TOO_LARGE_THRESHOLD,
            stdin_unsupported_protocol_version: STDIN_UNSUPPORTED_PROTOCOL_VERSION_THRESHOLD,
            trigger_builtin_unknown: TRIGGER_BUILTIN_UNKNOWN_THRESHOLD,
            trigger_builtin_deprecated_name: TRIGGER_BUILTIN_DEPRECATED_NAME_THRESHOLD,
        }
    }
}

/// Combined report: counters + health + thresholds. Shape is pinned
/// by `tests/protocol_stats_report_contract.rs`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolStatsReport {
    pub snapshot: StatsSnapshot,
    pub health: ProtocolStatsHealth,
    pub thresholds: ProtocolStatsThresholds,
}

pub fn snapshot() -> StatsSnapshot {
    StatsSnapshot {
        stdin_parse_failed_total: PROTOCOL_STATS
            .stdin_parse_failed_total
            .load(Ordering::Relaxed),
        stdin_command_too_large_total: PROTOCOL_STATS
            .stdin_command_too_large_total
            .load(Ordering::Relaxed),
        stdin_unsupported_protocol_version_total: PROTOCOL_STATS
            .stdin_unsupported_protocol_version_total
            .load(Ordering::Relaxed),
        trigger_builtin_unknown_total: PROTOCOL_STATS
            .trigger_builtin_unknown_total
            .load(Ordering::Relaxed),
        trigger_builtin_deprecated_name_total: PROTOCOL_STATS
            .trigger_builtin_deprecated_name_total
            .load(Ordering::Relaxed),
    }
}

#[cfg(test)]
pub fn reset_for_test() {
    PROTOCOL_STATS
        .stdin_parse_failed_total
        .store(0, Ordering::Relaxed);
    PROTOCOL_STATS
        .stdin_command_too_large_total
        .store(0, Ordering::Relaxed);
    PROTOCOL_STATS
        .stdin_unsupported_protocol_version_total
        .store(0, Ordering::Relaxed);
    PROTOCOL_STATS
        .trigger_builtin_unknown_total
        .store(0, Ordering::Relaxed);
    PROTOCOL_STATS
        .trigger_builtin_deprecated_name_total
        .store(0, Ordering::Relaxed);
}

/// Increment a counter and return the new total. Use the return value with
/// [`should_log_occurrence`] to rate-limit floodable log lines.
#[inline]
pub fn increment(counter: &AtomicU64) -> u64 {
    counter.fetch_add(1, Ordering::Relaxed) + 1
}

/// Return `true` for the first occurrence and every 100th thereafter. Used
/// by callers that want to keep a structured breadcrumb without letting a
/// hostile peer spam `app.log`.
#[inline]
pub fn should_log_occurrence(total: u64) -> bool {
    total == 1 || total.is_multiple_of(100)
}

/// Compute the health summary over a snapshot. Pure — takes a
/// `StatsSnapshot` by reference and returns a fresh
/// [`ProtocolStatsHealth`]. Flags are appended in stable declaration
/// order so the MCP resource body is stable across reads.
pub fn health(snapshot: &StatsSnapshot) -> ProtocolStatsHealth {
    let mut flags: Vec<String> = Vec::new();
    if snapshot.stdin_parse_failed_total > STDIN_PARSE_FAILED_THRESHOLD {
        flags.push("stdin_parse_failed".to_string());
    }
    if snapshot.stdin_command_too_large_total > STDIN_COMMAND_TOO_LARGE_THRESHOLD {
        flags.push("stdin_command_too_large".to_string());
    }
    if snapshot.stdin_unsupported_protocol_version_total
        > STDIN_UNSUPPORTED_PROTOCOL_VERSION_THRESHOLD
    {
        flags.push("stdin_unsupported_protocol_version".to_string());
    }
    if snapshot.trigger_builtin_unknown_total > TRIGGER_BUILTIN_UNKNOWN_THRESHOLD {
        flags.push("trigger_builtin_unknown_flood".to_string());
    }
    if snapshot.trigger_builtin_deprecated_name_total > TRIGGER_BUILTIN_DEPRECATED_NAME_THRESHOLD {
        flags.push("trigger_builtin_deprecated_name_flood".to_string());
    }
    ProtocolStatsHealth {
        ok: flags.is_empty(),
        flags,
    }
}

/// Full report over the current live counters — the value
/// [`crate::mcp_resources`] hands out for the
/// `kit://diagnostics/protocol-stats` URI.
pub fn current_report() -> ProtocolStatsReport {
    let snapshot = snapshot();
    let health = health(&snapshot);
    ProtocolStatsReport {
        snapshot,
        health,
        thresholds: ProtocolStatsThresholds::current(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn should_log_occurrence_hits_first_and_every_hundredth() {
        assert!(should_log_occurrence(1));
        assert!(!should_log_occurrence(2));
        assert!(!should_log_occurrence(99));
        assert!(should_log_occurrence(100));
        assert!(!should_log_occurrence(101));
        assert!(should_log_occurrence(200));
        assert!(should_log_occurrence(1000));
    }

    #[test]
    fn health_ok_on_zero_counters() {
        let h = health(&zero_snapshot());
        assert!(h.ok);
        assert!(h.flags.is_empty());
    }

    #[test]
    fn health_flags_parse_failed_at_one() {
        let mut s = zero_snapshot();
        s.stdin_parse_failed_total = 1;
        let h = health(&s);
        assert!(!h.ok);
        assert_eq!(h.flags, vec!["stdin_parse_failed".to_string()]);
    }

    #[test]
    fn health_flags_too_large_at_one() {
        let mut s = zero_snapshot();
        s.stdin_command_too_large_total = 1;
        let h = health(&s);
        assert_eq!(h.flags, vec!["stdin_command_too_large".to_string()]);
    }

    #[test]
    fn health_flags_unsupported_version_at_one() {
        let mut s = zero_snapshot();
        s.stdin_unsupported_protocol_version_total = 1;
        let h = health(&s);
        assert_eq!(
            h.flags,
            vec!["stdin_unsupported_protocol_version".to_string()]
        );
    }

    #[test]
    fn health_flags_unknown_only_above_threshold() {
        let mut s = zero_snapshot();
        s.trigger_builtin_unknown_total = TRIGGER_BUILTIN_UNKNOWN_THRESHOLD;
        assert!(health(&s).ok);
        s.trigger_builtin_unknown_total = TRIGGER_BUILTIN_UNKNOWN_THRESHOLD + 1;
        let h = health(&s);
        assert!(!h.ok);
        assert_eq!(h.flags, vec!["trigger_builtin_unknown_flood".to_string()]);
    }

    #[test]
    fn health_flags_deprecated_name_only_above_threshold() {
        let mut s = zero_snapshot();
        s.trigger_builtin_deprecated_name_total = TRIGGER_BUILTIN_DEPRECATED_NAME_THRESHOLD;
        assert!(health(&s).ok);
        s.trigger_builtin_deprecated_name_total = TRIGGER_BUILTIN_DEPRECATED_NAME_THRESHOLD + 1;
        let h = health(&s);
        assert_eq!(
            h.flags,
            vec!["trigger_builtin_deprecated_name_flood".to_string()]
        );
    }

    #[test]
    fn health_flags_stable_declaration_order() {
        let s = StatsSnapshot {
            stdin_parse_failed_total: 1,
            stdin_command_too_large_total: 1,
            stdin_unsupported_protocol_version_total: 1,
            trigger_builtin_unknown_total: TRIGGER_BUILTIN_UNKNOWN_THRESHOLD + 1,
            trigger_builtin_deprecated_name_total: TRIGGER_BUILTIN_DEPRECATED_NAME_THRESHOLD + 1,
        };
        let h = health(&s);
        assert_eq!(
            h.flags,
            vec![
                "stdin_parse_failed".to_string(),
                "stdin_command_too_large".to_string(),
                "stdin_unsupported_protocol_version".to_string(),
                "trigger_builtin_unknown_flood".to_string(),
                "trigger_builtin_deprecated_name_flood".to_string(),
            ]
        );
    }

    #[test]
    fn current_report_embeds_thresholds() {
        reset_for_test();
        let report = current_report();
        assert!(report.health.ok);
        assert_eq!(report.thresholds, ProtocolStatsThresholds::current());
    }

    #[test]
    fn report_serializes_to_camel_case() {
        let report = ProtocolStatsReport {
            snapshot: zero_snapshot(),
            health: ProtocolStatsHealth {
                ok: true,
                flags: Vec::new(),
            },
            thresholds: ProtocolStatsThresholds::current(),
        };
        let json = serde_json::to_string(&report).unwrap();
        // camelCase field names must be on the wire.
        assert!(json.contains("\"stdinParseFailedTotal\""));
        assert!(json.contains("\"triggerBuiltinUnknownTotal\""));
        assert!(json.contains("\"triggerBuiltinUnknown\""));
        assert!(json.contains("\"flags\":[]"));
        assert!(json.contains("\"ok\":true"));
    }
}
