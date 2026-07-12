//! Verification-freshness verdicts over mdflow's eval trust ledger.
//!
//! mdflow persists eval outcomes to `~/.mdflow/eval-results.json`
//! (override: `MDFLOW_EVAL_RESULTS`), each entry carrying a content-bound
//! `verification` fingerprint over the flow's import graph, hooks, suite,
//! config, engine, and model. This module turns that ledger into per-flow
//! verdicts the GUI can show next to a flow row.
//!
//! The governing rule is fail-closed: **a verdict is never inherited — it is
//! bound to the exact content it judged, and anything less than a fresh,
//! fully-bound clean run is "unverified", not "passed".** Concretely:
//!
//! - No ledger, unreadable ledger, or no entry for the flow → [`EvalVerdict::Unverified`].
//! - Any failing case on the last run → [`EvalVerdict::Failing`].
//! - Any flaky case → [`EvalVerdict::Flaky`] (a sometimes-green gate is not green).
//! - A clean run with no `verification` fingerprint, with `currentClean`
//!   revoked by mdflow, or with the flow file modified after the run →
//!   [`EvalVerdict::Stale`] — approval must not survive edits to the thing
//!   it approved.
//! - Only a clean, fingerprint-bound run that predates no flow edit is
//!   [`EvalVerdict::Verified`].

use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerEntry {
    /// Absolute path of the flow the suite gates.
    pub flow: String,
    #[serde(default)]
    pub pass: u32,
    #[serde(default)]
    pub fail: u32,
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub last_run_at: Option<String>,
    #[serde(default)]
    pub last_clean_at: Option<String>,
    #[serde(default)]
    pub current_clean: Option<bool>,
    #[serde(default)]
    pub flaky: Option<u32>,
    #[serde(default)]
    pub inconclusive: Option<u32>,
    #[serde(default)]
    pub verification: Option<VerificationFingerprint>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationFingerprint {
    pub fingerprint: String,
    #[serde(default)]
    pub mdflow_version: Option<String>,
    #[serde(default)]
    pub engine: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
}

/// Per-flow verification verdict, ordered from best to worst trust.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalVerdict {
    /// Clean run, content-bound fingerprint, no flow edit since the run.
    Verified { last_clean_at: String },
    /// Was clean once, but the binding no longer holds.
    Stale { reason: String },
    /// The last run had at least one flaky case.
    Flaky,
    /// The last run had at least one failing case.
    Failing,
    /// No trustworthy signal at all. Never rendered as a pass.
    Unverified,
}

impl EvalVerdict {
    /// Short row-badge label. Deliberately never "Passed" for anything but a
    /// fresh bound run.
    pub fn badge_label(&self) -> &'static str {
        match self {
            EvalVerdict::Verified { .. } => "Verified",
            EvalVerdict::Stale { .. } => "Stale",
            EvalVerdict::Flaky => "Flaky",
            EvalVerdict::Failing => "Failing",
            EvalVerdict::Unverified => "Unverified",
        }
    }
}

/// Parsed ledger, keyed by flow path.
#[derive(Debug, Default, Clone)]
pub struct EvalLedger {
    by_flow: HashMap<String, LedgerEntry>,
}

pub fn ledger_path() -> PathBuf {
    if let Ok(custom) = std::env::var("MDFLOW_EVAL_RESULTS") {
        if !custom.is_empty() {
            return PathBuf::from(custom);
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
    PathBuf::from(home)
        .join(".mdflow")
        .join("eval-results.json")
}

impl EvalLedger {
    /// Read the on-disk ledger. Any read or parse failure yields an empty
    /// ledger — every flow then classifies as `Unverified`, never as passed.
    pub fn load_default() -> Self {
        match std::fs::read_to_string(ledger_path()) {
            Ok(raw) => Self::parse(&raw),
            Err(_) => Self::default(),
        }
    }

    /// Parse ledger JSON. Unparseable documents and unparseable entries are
    /// dropped (fail-closed), not guessed at. Duplicate entries for one flow
    /// (path key + `flow:<id>` key) keep the most recent `lastRunAt`.
    pub fn parse(raw: &str) -> Self {
        let Ok(doc) = serde_json::from_str::<HashMap<String, serde_json::Value>>(raw) else {
            return Self::default();
        };
        let mut by_flow: HashMap<String, LedgerEntry> = HashMap::new();
        for value in doc.into_values() {
            let Ok(entry) = serde_json::from_value::<LedgerEntry>(value) else {
                continue;
            };
            match by_flow.get(&entry.flow) {
                Some(existing)
                    if parse_ts(&existing.last_run_at) >= parse_ts(&entry.last_run_at) => {}
                _ => {
                    by_flow.insert(entry.flow.clone(), entry);
                }
            }
        }
        Self { by_flow }
    }

    /// The verdict for a flow, given its absolute path and file mtime in
    /// epoch milliseconds (the roster provides both).
    pub fn verdict_for(&self, flow_path: &str, flow_mtime_ms: u64) -> EvalVerdict {
        let Some(entry) = self.by_flow.get(flow_path) else {
            return EvalVerdict::Unverified;
        };
        if entry.fail > 0 {
            return EvalVerdict::Failing;
        }
        if entry.flaky.unwrap_or(0) > 0 {
            return EvalVerdict::Flaky;
        }
        if entry.total == 0 || entry.pass == 0 {
            // A suite that ran nothing proved nothing.
            return EvalVerdict::Unverified;
        }
        if entry.current_clean != Some(true) {
            return EvalVerdict::Stale {
                reason: "mdflow revoked the clean bit (content moved under the suite)".into(),
            };
        }
        if entry.verification.is_none() {
            return EvalVerdict::Stale {
                reason: "clean run has no content-bound fingerprint".into(),
            };
        }
        let Some(run_ms) = parse_ts(&entry.last_run_at) else {
            return EvalVerdict::Stale {
                reason: "clean run has no parseable lastRunAt".into(),
            };
        };
        if flow_mtime_ms > run_ms {
            return EvalVerdict::Stale {
                reason: "flow changed since judgment — re-run required".into(),
            };
        }
        EvalVerdict::Verified {
            last_clean_at: entry
                .last_clean_at
                .clone()
                .or_else(|| entry.last_run_at.clone())
                .unwrap_or_default(),
        }
    }
}

/// RFC 3339 timestamp → epoch milliseconds. `None` on absence or garbage.
fn parse_ts(value: &Option<String>) -> Option<u64> {
    let raw = value.as_deref()?;
    let parsed = DateTime::parse_from_rfc3339(raw).ok()?;
    u64::try_from(parsed.with_timezone(&Utc).timestamp_millis()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLOW: &str = "/repo/flows/clipboard.md";
    const RUN_TS: &str = "2026-07-09T17:35:43.958Z";

    fn run_ms() -> u64 {
        parse_ts(&Some(RUN_TS.to_string())).expect("test timestamp parses")
    }

    fn entry_json(overrides: &str) -> String {
        format!(
            r#"{{
              "/repo/flows/clipboard.eval.ts": {{
                "flow": "{FLOW}",
                "pass": 1, "fail": 0, "total": 1,
                "lastRunAt": "{RUN_TS}",
                "lastCleanAt": "{RUN_TS}",
                "currentClean": true,
                "verification": {{ "fingerprint": "a79db6aa", "engine": "codex" }}
                {overrides}
              }}
            }}"#
        )
    }

    #[test]
    fn fully_bound_clean_run_is_verified() {
        let ledger = EvalLedger::parse(&entry_json(""));
        assert_eq!(
            ledger.verdict_for(FLOW, run_ms() - 1),
            EvalVerdict::Verified {
                last_clean_at: RUN_TS.to_string()
            }
        );
    }

    #[test]
    fn missing_ledger_and_missing_entry_are_unverified_not_passed() {
        assert_eq!(
            EvalLedger::parse("").verdict_for(FLOW, 0),
            EvalVerdict::Unverified
        );
        assert_eq!(
            EvalLedger::parse("{not json").verdict_for(FLOW, 0),
            EvalVerdict::Unverified
        );
        let ledger = EvalLedger::parse(&entry_json(""));
        assert_eq!(
            ledger.verdict_for("/repo/flows/other.md", 0),
            EvalVerdict::Unverified
        );
    }

    #[test]
    fn flow_edited_after_judgment_goes_stale() {
        let ledger = EvalLedger::parse(&entry_json(""));
        let verdict = ledger.verdict_for(FLOW, run_ms() + 1);
        assert!(
            matches!(&verdict, EvalVerdict::Stale { reason } if reason.contains("changed since judgment")),
            "approval must not survive an edit to the flow it approved; got {verdict:?}"
        );
    }

    #[test]
    fn clean_run_without_fingerprint_is_stale_not_verified() {
        // Pre-4.1 ledger entries have lastCleanAt but no verification block:
        // green without binding is a wish, not a receipt.
        let raw = entry_json("").replace(
            r#""verification": { "fingerprint": "a79db6aa", "engine": "codex" }"#,
            r#""verification": null"#,
        );
        let verdict = EvalLedger::parse(&raw).verdict_for(FLOW, 0);
        assert!(
            matches!(&verdict, EvalVerdict::Stale { reason } if reason.contains("fingerprint"))
        );
    }

    #[test]
    fn revoked_clean_bit_is_stale() {
        let raw = entry_json("").replace(r#""currentClean": true"#, r#""currentClean": false"#);
        let verdict = EvalLedger::parse(&raw).verdict_for(FLOW, 0);
        assert!(matches!(verdict, EvalVerdict::Stale { .. }));
    }

    #[test]
    fn failures_and_flakes_are_never_averaged_away() {
        let failing = entry_json("").replace(r#""pass": 1, "fail": 0"#, r#""pass": 5, "fail": 1"#);
        assert_eq!(
            EvalLedger::parse(&failing).verdict_for(FLOW, 0),
            EvalVerdict::Failing
        );

        let flaky = entry_json(r#", "flaky": 1"#);
        assert_eq!(
            EvalLedger::parse(&flaky).verdict_for(FLOW, 0),
            EvalVerdict::Flaky
        );
    }

    #[test]
    fn empty_suite_proves_nothing() {
        let raw = entry_json("").replace(
            r#""pass": 1, "fail": 0, "total": 1"#,
            r#""pass": 0, "fail": 0, "total": 0"#,
        );
        assert_eq!(
            EvalLedger::parse(&raw).verdict_for(FLOW, 0),
            EvalVerdict::Unverified
        );
    }

    #[test]
    fn duplicate_entries_resolve_to_the_most_recent_run() {
        let raw = format!(
            r#"{{
              "/repo/flows/clipboard.eval.ts": {{
                "flow": "{FLOW}", "pass": 1, "fail": 0, "total": 1,
                "lastRunAt": "2026-07-01T00:00:00.000Z", "currentClean": true,
                "verification": {{ "fingerprint": "old" }}
              }},
              "flow:abc123": {{
                "flow": "{FLOW}", "pass": 0, "fail": 1, "total": 1,
                "lastRunAt": "{RUN_TS}"
              }}
            }}"#
        );
        // The newer run failed; the stale green must not shadow it.
        assert_eq!(
            EvalLedger::parse(&raw).verdict_for(FLOW, 0),
            EvalVerdict::Failing
        );
    }

    #[test]
    fn badge_labels_never_call_anything_else_passed() {
        for verdict in [
            EvalVerdict::Stale { reason: "x".into() },
            EvalVerdict::Flaky,
            EvalVerdict::Failing,
            EvalVerdict::Unverified,
        ] {
            assert_ne!(verdict.badge_label(), "Verified");
        }
    }
}
