//! Oracle-Session `protocol-builtin-boundary-refactor-plan` PR6:
//! golden-transcript tests for the pure `plan_trigger_builtin_route`
//! planner.
//!
//! The fixture at `tests/golden/trigger_builtin/routes.jsonl` contains
//! representative canonical commands and their serialized routes:
//!
//! ```text
//! { "input": "builtin/clipboard-history", "expected": "ShowFilterableView::ClipboardHistory" }
//! ```
//!
//! `input` is the canonical command id (stable, one per variant) and
//! `expected` is the string produced by
//! [`script_kit_gpui::routes::render_route`]. These examples defend
//! caller-visible route serialization; exhaustive routing is enforced
//! by the compiler rather than a case-count requirement.
//!
//! This mirrors the resolver-side golden harness added in PR3
//! (`tests/trigger_builtin_resolve_golden.rs`). Keeping the two in
//! the same `tests/golden/trigger_builtin/` directory lets a future
//! Bun-side harness import both files from one place.

use script_kit_gpui::builtins::trigger_registry::registry as trigger_registry;
use script_kit_gpui::routes::{parse_route, plan_trigger_builtin_route, render_route};
use std::fs;
use std::path::PathBuf;

#[derive(serde::Deserialize)]
struct GoldenCase {
    input: String,
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
fn every_route_case_matches_its_golden_line() {
    let cases = load_cases("routes.jsonl");
    assert!(
        !cases.is_empty(),
        "golden fixture `routes.jsonl` must have at least one case"
    );

    let registry = trigger_registry();
    let mut failures = Vec::new();
    for (line_no, case) in cases {
        let Some(id) = registry.resolve(&case.input) else {
            failures.push(format!(
                "routes.jsonl:{line_no}: input `{}` does not resolve via the trigger-builtin \
                 registry — the fixture expected a TriggerBuiltin but got nothing.",
                case.input
            ));
            continue;
        };
        let rendered = render_route(&plan_trigger_builtin_route(id));
        if rendered != case.expected {
            failures.push(format!(
                "routes.jsonl:{line_no}: input={}\n    expected: {}\n    actual:   {}",
                case.input, case.expected, rendered
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "golden route transcripts drifted from expectations:\n{}\n\n\
         Update `tests/golden/trigger_builtin/routes.jsonl` if the new \
         rendering is intentional, or fix the planner in \
         `src/app_impl/routes.rs` if the route changed by accident.",
        failures.join("\n\n")
    );
}

#[test]
fn every_fixture_expected_parses_back() {
    // PR7 wire-format contract: each golden `expected` string must
    // parse back into the same AppRoute the planner emits for that
    // `input`. If a Bun or MCP consumer serializes a route string and
    // Rust ingests it, the two sides must agree byte-for-byte.
    let cases = load_cases("routes.jsonl");
    let registry = trigger_registry();
    let mut failures = Vec::new();
    for (line_no, case) in cases {
        let Some(id) = registry.resolve(&case.input) else {
            failures.push(format!(
                "routes.jsonl:{line_no}: input `{}` does not resolve via the trigger-builtin \
                 registry — cannot verify parse_route round-trip.",
                case.input
            ));
            continue;
        };
        let planned = plan_trigger_builtin_route(id);
        match parse_route(&case.expected) {
            Some(parsed) if parsed == planned => {}
            Some(parsed) => failures.push(format!(
                "routes.jsonl:{line_no}: input={} expected={}\n    planner produced: {:?}\n    parse_route produced: {:?}",
                case.input, case.expected, planned, parsed
            )),
            None => failures.push(format!(
                "routes.jsonl:{line_no}: input={} expected={} — parse_route returned None, \
                 but the planner produces {:?} for this input.",
                case.input, case.expected, planned
            )),
        }
    }
    assert!(
        failures.is_empty(),
        "golden fixture expected strings failed to parse back into the planner's AppRoute:\n{}\n\n\
         If the wire format changed, update both `render_route` and `parse_route` in \
         `src/app_impl/routes.rs` together, then regenerate `routes.jsonl`.",
        failures.join("\n\n")
    );
}
