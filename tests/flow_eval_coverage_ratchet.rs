//! Flow-eval coverage ratchet — the audit ledger for `flows/*.eval.ts`.
//!
//! `flows/README.md`'s own creed is "if a guardrail isn't covered by an
//! eval, it's a wish", yet most owner flows carry no eval suite. This
//! ratchet makes that debt visible and one-directional:
//!
//! - Every flow without a sibling `.eval.ts` must appear in the ledger
//!   below, so a NEW flow cannot land without either an eval suite or a
//!   deliberate, reviewable ledger entry.
//! - A ledger entry whose flow gains an eval suite must be REMOVED — the
//!   ledger only shrinks, never silently re-grows.
//!
//! This is a file-presence inventory, not a source audit: it reads no flow
//! prompt text and asserts nothing about suite contents (mdflow's own
//! `md eval --plan` owns that).

use std::collections::BTreeSet;
use std::path::Path;

/// Flows known to lack an eval suite as of 2026-07-11. Shrink-only: delete a
/// name the moment its `flows/<name>.eval.ts` lands. Never add to this list
/// to unblock a new flow — write the eval instead.
const UNCOVERED_LEDGER: &[&str] = &[
    "actions",
    "ai-core",
    "auditor",
    "build-doctor",
    "builtins",
    "components",
    "devex",
    "devtools",
    "escape",
    "execution",
    "gpui-vendor",
    "hotkeys",
    "launcher",
    "mcp",
    "migrate",
    "perf",
    "platform",
    "prompts",
    "release",
    "scout",
    "screenshots",
    "settings",
    "site",
    "terminal",
    "tests",
    "videos",
];

fn flow_names() -> BTreeSet<String> {
    let flows_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("flows");
    std::fs::read_dir(&flows_dir)
        .expect("flows/ directory exists")
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let base = name.strip_suffix(".md")?;
            if base.eq_ignore_ascii_case("readme") || base.ends_with(".eval") {
                return None;
            }
            Some(base.to_string())
        })
        .collect()
}

fn has_eval_suite(flow: &str) -> bool {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("flows")
        .join(format!("{flow}.eval.ts"))
        .exists()
}

#[test]
fn every_uncovered_flow_is_on_the_ledger() {
    let missing: Vec<String> = flow_names()
        .into_iter()
        .filter(|flow| !has_eval_suite(flow) && !UNCOVERED_LEDGER.contains(&flow.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "new flows must ship with an eval suite (flows/<name>.eval.ts); \
         these have neither a suite nor a ledger entry: {missing:?}"
    );
}

#[test]
fn the_ledger_only_shrinks() {
    let flows = flow_names();
    let mut covered_but_listed = Vec::new();
    let mut zombie_entries = Vec::new();
    for entry in UNCOVERED_LEDGER {
        if !flows.contains(*entry) {
            zombie_entries.push(*entry);
        } else if has_eval_suite(entry) {
            covered_but_listed.push(*entry);
        }
    }
    assert!(
        covered_but_listed.is_empty(),
        "these flows gained eval suites — delete them from UNCOVERED_LEDGER \
         so the ratchet locks the progress: {covered_but_listed:?}"
    );
    assert!(
        zombie_entries.is_empty(),
        "these ledger entries no longer name a flow — delete them: {zombie_entries:?}"
    );
}

#[test]
fn covered_flows_exist_and_the_ledger_is_not_vacuous() {
    let flows = flow_names();
    let covered: Vec<&String> = flows.iter().filter(|f| has_eval_suite(f)).collect();
    assert!(
        !covered.is_empty(),
        "at least one flow must carry an eval suite; an all-ledger corpus \
         means the eval harness itself has been abandoned"
    );
    assert!(
        !flows.is_empty(),
        "flows/ enumerated to nothing — the harness is looking at the wrong directory"
    );
}
