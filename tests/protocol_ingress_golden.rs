//! Oracle-Session `protocol-builtin-boundary-engineering-plan` Pass #2:
//! golden-transcript tests for the pure ingress observer at
//! [`script_kit_gpui::protocol::ingress::observe_ingress`].
//!
//! Why this test exists: the inline `#[cfg(test)]` suite inside
//! `src/protocol/ingress.rs` already pins the happy/sad paths case by
//! case in Rust, but there is no external fixture that a reviewer can
//! extend without touching Rust. This file complements Pass #1's
//! `tests/mcp_protocol_golden.rs` by applying the same shape-match
//! layer one level down — at the protocol-ingress seam, before any
//! typed-message deserialization.
//!
//! The fixture at `tests/golden/protocol/ingress_observations.jsonl`
//! has one case per line. Each case is `{name, line, expect}` where
//! `expect` is either:
//!   - `{"ok": {kind, version, warningsFor[], hasDeprecationError}}`
//!     (success observation — `kind` is nullable, `warningsFor` is an
//!     order-independent set of deprecated field names), or
//!   - `{"error": "InvalidJson" | "NotObject" | "Version"}` (the
//!     discriminating name of the `IngressObserveError` variant).
//!
//! Three invariants are pinned:
//!
//! 1. Every fixture case matches its expected shape (round-trip
//!    through `observe_ingress`).
//! 2. The fixture covers every `IngressObserveError` variant. A
//!    refactor that silently collapses two error classes still fails
//!    this test even if individual success cases still pass.
//! 3. The fixture exercises every row in `DEPRECATED_FIELDS`. If a
//!    new deprecation is added without a fixture line, the golden
//!    suite silently gives it zero coverage — this test pins the
//!    lower bound so the fixture grows with the table.

use script_kit_gpui::protocol::deprecations::DEPRECATED_FIELDS;
use script_kit_gpui::protocol::ingress::{observe_ingress, IngressObserveError};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct GoldenCase {
    name: String,
    line: String,
    expect: Expectation,
}

#[derive(Debug, Deserialize)]
enum Expectation {
    #[serde(rename = "ok")]
    Ok(OkShape),
    #[serde(rename = "error")]
    Error(ErrorVariant),
}

#[derive(Debug, Deserialize)]
struct OkShape {
    kind: Option<String>,
    version: u16,
    #[serde(rename = "warningsFor")]
    warnings_for: Vec<String>,
    #[serde(rename = "hasDeprecationError")]
    has_deprecation_error: bool,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum ErrorVariant {
    InvalidJson,
    NotObject,
    Version,
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden/protocol/ingress_observations.jsonl")
}

fn load_cases() -> Vec<(usize, GoldenCase)> {
    let path = fixture_path();
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read golden fixture {}: {e}", path.display()));
    let mut cases = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parsed: GoldenCase = serde_json::from_str(trimmed).unwrap_or_else(|e| {
            panic!(
                "fixture {}:{} failed to parse as GoldenCase: {e}\n  line: {trimmed}",
                path.display(),
                idx + 1
            )
        });
        cases.push((idx + 1, parsed));
    }
    cases
}

fn classify_error(err: &IngressObserveError) -> ErrorVariant {
    match err {
        IngressObserveError::InvalidJson(_) => ErrorVariant::InvalidJson,
        IngressObserveError::NotObject => ErrorVariant::NotObject,
        IngressObserveError::Version(_) => ErrorVariant::Version,
    }
}

fn assert_case(line_no: usize, case: &GoldenCase) -> Result<(), String> {
    let actual = observe_ingress(&case.line);
    match (&case.expect, actual) {
        (Expectation::Ok(shape), Ok(obs)) => {
            if obs.kind.as_deref() != shape.kind.as_deref() {
                return Err(format!(
                    "ingress_observations.jsonl:{line_no}: `{}` expected kind {:?}, got {:?}",
                    case.name, shape.kind, obs.kind
                ));
            }
            if obs.version.get() != shape.version {
                return Err(format!(
                    "ingress_observations.jsonl:{line_no}: `{}` expected version {}, got {}",
                    case.name,
                    shape.version,
                    obs.version.get()
                ));
            }
            let actual_warnings: BTreeSet<&str> = obs.warnings.iter().map(|w| w.field).collect();
            let expected_warnings: BTreeSet<&str> =
                shape.warnings_for.iter().map(String::as_str).collect();
            if actual_warnings != expected_warnings {
                return Err(format!(
                    "ingress_observations.jsonl:{line_no}: `{}` expected warnings {:?}, got {:?}",
                    case.name, expected_warnings, actual_warnings
                ));
            }
            let has_err = obs.deprecation_error.is_some();
            if has_err != shape.has_deprecation_error {
                return Err(format!(
                    "ingress_observations.jsonl:{line_no}: `{}` expected hasDeprecationError={}, got {}",
                    case.name, shape.has_deprecation_error, has_err
                ));
            }
            Ok(())
        }
        (Expectation::Error(expected), Err(err)) => {
            let got = classify_error(&err);
            if got != *expected {
                return Err(format!(
                    "ingress_observations.jsonl:{line_no}: `{}` expected error {:?}, got {:?} (inner: {err})",
                    case.name, expected, got
                ));
            }
            Ok(())
        }
        (Expectation::Ok(_), Err(err)) => Err(format!(
            "ingress_observations.jsonl:{line_no}: `{}` expected ok, got error: {err}",
            case.name
        )),
        (Expectation::Error(expected), Ok(obs)) => Err(format!(
            "ingress_observations.jsonl:{line_no}: `{}` expected error {:?}, got ok: {:?}",
            case.name, expected, obs
        )),
    }
}

#[test]
fn every_ingress_observation_case_matches_shape() {
    let cases = load_cases();
    assert!(
        !cases.is_empty(),
        "golden fixture `ingress_observations.jsonl` must have at least one case"
    );

    let mut failures = Vec::new();
    for (line_no, case) in &cases {
        if let Err(msg) = assert_case(*line_no, case) {
            failures.push(msg);
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} ingress golden cases failed:\n  - {}",
        failures.len(),
        cases.len(),
        failures.join("\n  - ")
    );
}

#[test]
fn fixture_covers_every_observe_error_variant() {
    // Refactor guard: a future cleanup that collapses `InvalidJson` into
    // `NotObject` (or vice versa) must still fail at least one shape
    // assertion. The easiest way to guarantee that is to pin that the
    // fixture actually exercises all three error classes today.
    let cases = load_cases();
    let seen: BTreeSet<ErrorVariant> = cases
        .iter()
        .filter_map(|(_, c)| match &c.expect {
            Expectation::Error(v) => Some(*v),
            Expectation::Ok(_) => None,
        })
        .collect();

    for required in [
        ErrorVariant::InvalidJson,
        ErrorVariant::NotObject,
        ErrorVariant::Version,
    ] {
        assert!(
            seen.contains(&required),
            "ingress_observations.jsonl must exercise error variant {required:?}; \
             saw {seen:?}"
        );
    }
}

#[test]
fn fixture_covers_every_deprecation_row() {
    // Shape invariant: every row in `DEPRECATED_FIELDS` must be
    // exercised by at least one fixture line that carries the
    // deprecated field on the deprecated kind. This keeps the fixture
    // honest — adding a new deprecation without a fixture case would
    // ship with zero transcript coverage for it.
    let cases = load_cases();

    let mut covered: BTreeSet<(&'static str, &'static str)> = BTreeSet::new();
    for (_, case) in &cases {
        let Expectation::Ok(shape) = &case.expect else {
            continue;
        };
        let Some(kind) = shape.kind.as_deref() else {
            continue;
        };
        for warned_field in &shape.warnings_for {
            for row in DEPRECATED_FIELDS {
                if row.kind == kind && row.field == warned_field.as_str() {
                    covered.insert((row.kind, row.field));
                }
            }
        }
    }

    for row in DEPRECATED_FIELDS {
        assert!(
            covered.contains(&(row.kind, row.field)),
            "ingress_observations.jsonl must have a fixture line that warns on \
             deprecated field `{}.{}` (deprecated_since v{}, remove_in {:?}); \
             covered rows so far: {:?}",
            row.kind,
            row.field,
            row.deprecated_since,
            row.remove_in,
            covered
        );
    }
}
