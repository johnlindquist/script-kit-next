//! Oracle-Session `protocol-builtin-boundary-refactor-plan` PR6:
//! golden-transcript tests for the pure `plan_trigger_builtin_route`
//! planner.
//!
//! The fixture at `tests/golden/trigger_builtin/routes.jsonl` has one
//! JSONL line per [`TriggerBuiltin`] variant:
//!
//! ```text
//! { "input": "builtin/clipboard-history", "expected": "ShowFilterableView::ClipboardHistory" }
//! ```
//!
//! `input` is the canonical command id (stable, one per variant) and
//! `expected` is the string produced by
//! [`script_kit_gpui::routes::render_route`]. A silent planner rewire
//! or a `TriggerBuiltin` addition that forgets the planner will show
//! up as a fixture diff or a "fixture missing variant" failure —
//! whichever lands first.
//!
//! This mirrors the resolver-side golden harness added in PR3
//! (`tests/trigger_builtin_resolve_golden.rs`). Keeping the two in
//! the same `tests/golden/trigger_builtin/` directory lets a future
//! Bun-side harness import both files from one place.

use script_kit_gpui::builtins::trigger_registry::{registry as trigger_registry, TriggerBuiltin};
use script_kit_gpui::routes::{
    parse_route, plan_trigger_builtin_route, render_route, AppRoute, FilterableView,
};
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
fn fixture_covers_every_trigger_builtin_variant() {
    // Reverse coverage: every TriggerBuiltin::ALL variant must have
    // at least one line in `routes.jsonl`. This is the guard that
    // catches a new variant shipped without a golden case.
    let cases = load_cases("routes.jsonl");
    let registry = trigger_registry();
    let mut seen: std::collections::HashSet<TriggerBuiltin> = Default::default();
    for (_line_no, case) in cases {
        if let Some(id) = registry.resolve(&case.input) {
            seen.insert(id);
        }
    }
    for &id in TriggerBuiltin::ALL {
        assert!(
            seen.contains(&id),
            "routes.jsonl must have a case for {id:?} (command id `{}`); saw: {:?}",
            id.command_id(),
            seen
        );
    }
}

#[test]
fn fixture_covers_every_app_route_kind() {
    // Positive coverage on the output side. Every AppRoute variant
    // (and every FilterableView) must appear at least once so the
    // fixture cannot decay into e.g. "all filterable views, no
    // OpenTabAi".
    let cases = load_cases("routes.jsonl");
    let registry = trigger_registry();

    let mut saw_open_file_search = false;
    let mut saw_open_tab_ai = false;
    let mut saw_open_current_app_commands = false;
    let mut seen_views: std::collections::HashSet<FilterableView> = Default::default();

    for (_line_no, case) in cases {
        let Some(id) = registry.resolve(&case.input) else {
            continue;
        };
        match plan_trigger_builtin_route(id) {
            AppRoute::ShowFilterableView(v) => {
                seen_views.insert(v);
            }
            AppRoute::OpenFileSearch => saw_open_file_search = true,
            AppRoute::OpenTabAi => saw_open_tab_ai = true,
            AppRoute::OpenCurrentAppCommands => saw_open_current_app_commands = true,
        }
    }

    assert!(
        saw_open_file_search,
        "routes.jsonl needs at least one OpenFileSearch case"
    );
    assert!(
        saw_open_tab_ai,
        "routes.jsonl needs at least one OpenTabAi case"
    );
    assert!(
        saw_open_current_app_commands,
        "routes.jsonl needs at least one OpenCurrentAppCommands case"
    );
    for &v in FilterableView::ALL {
        assert!(
            seen_views.contains(&v),
            "routes.jsonl needs at least one ShowFilterableView::{} case; saw: {:?}",
            v.name(),
            seen_views
        );
    }
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

#[test]
fn fixture_has_exactly_one_case_per_variant() {
    // Stronger than `fixture_covers_every_trigger_builtin_variant`:
    // no duplicates either. The planner is a 1:1 mapping, so the
    // fixture should be too — otherwise a "works for one caller,
    // breaks for another" regression could hide.
    let cases = load_cases("routes.jsonl");
    let registry = trigger_registry();
    let mut counts: std::collections::HashMap<TriggerBuiltin, usize> = Default::default();
    for (_line_no, case) in cases {
        if let Some(id) = registry.resolve(&case.input) {
            *counts.entry(id).or_default() += 1;
        }
    }
    for &id in TriggerBuiltin::ALL {
        let n = counts.get(&id).copied().unwrap_or(0);
        assert_eq!(
            n, 1,
            "routes.jsonl must have exactly one case for {id:?}, got {n}"
        );
    }
}
