//! Oracle-Session `protocol-builtin-boundary-refactor-plan` PR3:
//! golden-transcript tests for the pure `triggerBuiltin` resolver.
//!
//! The fixture at `tests/golden/trigger_builtin/basic.jsonl` has one
//! JSONL line per case:
//!
//! ```text
//! { "input": {...}, "expected": "Resolved::ClipboardHistory::NameAlias" }
//! ```
//!
//! The harness reads each line, feeds `input` through
//! [`resolve_trigger_builtin`], renders the outcome via
//! [`render_resolution`], and asserts byte-for-byte equality against
//! `expected`. The fixture is the spec. Adding a new resolver arm or
//! a new variant means adding a line here — the test fails loudly if
//! the outcome format drifts or a routing decision changes silently.
//!
//! Why golden-file instead of inline `assert_eq!` blocks:
//! 1. Adding a new case is one line of JSONL, not a copy-pasted
//!    Rust block — contributors reach for it more readily.
//! 2. Resolver regressions show up as a diff on a single data file.
//! 3. The same fixture can later be executed from the Bun side (by
//!    shelling out or importing into the kit-sdk harness) so Rust
//!    and TypeScript agree on routing without a shared test runner.

use script_kit_gpui::builtins::trigger_resolve::{render_resolution, resolve_trigger_builtin};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[derive(serde::Deserialize)]
struct GoldenCase {
    input: Value,
    expected: String,
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden/trigger_builtin")
        .join(name)
}

fn load_cases(name: &str) -> Vec<(usize, GoldenCase)> {
    let path = fixture_path(name);
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

#[test]
fn every_basic_case_matches_its_golden_line() {
    let cases = load_cases("basic.jsonl");
    assert!(
        !cases.is_empty(),
        "golden fixture must have at least one case"
    );

    let mut failures = Vec::new();
    for (line_no, case) in cases {
        let outcome = resolve_trigger_builtin(&case.input);
        let rendered = render_resolution(&outcome);
        if rendered != case.expected {
            failures.push(format!(
                "basic.jsonl:{line_no}: input={}\n    expected: {}\n    actual:   {}",
                case.input, case.expected, rendered
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "golden resolver transcripts drifted from expectations:\n{}\n\n\
         Update `tests/golden/trigger_builtin/basic.jsonl` if the new \
         rendering is intentional, or fix the resolver in \
         `src/builtins/trigger_resolve.rs` if the route changed by accident.",
        failures.join("\n\n")
    );
}

#[test]
fn fixture_has_at_least_one_case_per_resolved_via_arm() {
    // Guard against the fixture decaying into a narrow subset that
    // misses a resolution path. Every `ResolvedVia` variant must
    // appear at least once in `basic.jsonl`.
    let cases = load_cases("basic.jsonl");
    let mut seen_via: std::collections::BTreeSet<String> = Default::default();
    for (_line_no, case) in cases {
        let rendered = render_resolution(&resolve_trigger_builtin(&case.input));
        if let Some(rest) = rendered.strip_prefix("Resolved::") {
            if let Some((_variant, via)) = rest.split_once("::") {
                seen_via.insert(via.to_string());
            }
        }
    }
    for expected in [
        "BuiltinIdField",
        "NameAsCommandId",
        "NameAlias",
        "BothAgree",
    ] {
        assert!(
            seen_via.contains(expected),
            "basic.jsonl must have at least one case that resolves via {expected}; \
             saw: {:?}",
            seen_via
        );
    }
}

#[test]
fn fixture_covers_every_unresolved_arm() {
    let cases = load_cases("basic.jsonl");
    let mut saw_missing = false;
    let mut saw_unknown = false;
    let mut saw_conflict = false;
    for (_line_no, case) in cases {
        let rendered = render_resolution(&resolve_trigger_builtin(&case.input));
        if rendered == "MissingKey" {
            saw_missing = true;
        } else if rendered.starts_with("Unknown::") {
            saw_unknown = true;
        } else if rendered.starts_with("Conflict::") {
            saw_conflict = true;
        }
    }
    assert!(saw_missing, "fixture needs at least one MissingKey case");
    assert!(saw_unknown, "fixture needs at least one Unknown case");
    assert!(saw_conflict, "fixture needs at least one Conflict case");
}
