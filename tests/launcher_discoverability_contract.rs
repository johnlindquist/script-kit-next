//! Launcher discoverability bar — intents must surface their affordance.
//!
//! Origin (2026-07-10): "I'm navigating the app and have _no idea_ how to
//! create a new flow. I would expect it to surface if I type 'new flow' or
//! 'flow new'… but it was nowhere." A feature the launcher cannot surface
//! from the words a user would actually type does not exist for that user.
//!
//! The bar is two-sided, in the SIDE/QUEST band tradition:
//! - RECALL: each intent phrasing must surface its affordance within a rank
//!   ceiling (a top result, not merely "somewhere in the list").
//! - PRECISION: unrelated intents must NOT surface the affordance near the
//!   top — discoverability must never be bought with rank spam. If a future
//!   keyword change busts the precision band, temper the keywords; do not
//!   widen the band.
//! - REFERENCE ANCHORS: known-good affordances ("new script") run through
//!   the same harness, so a harness regression fails visibly as an anchor
//!   failure rather than reading as an affordance failure.

use script_kit_gpui::builtins::get_builtin_entries;
use script_kit_gpui::config::BuiltInConfig;
use script_kit_gpui::scripts::fuzzy_search_builtins;

struct IntentCase {
    /// What a user would actually type in the main launcher input.
    intent: &'static str,
    /// The builtin id that must surface for that intent.
    affordance: &'static str,
    /// Band ceiling: 0-based rank in the builtin results must be <= this.
    max_rank: usize,
}

const RECALL_BAR: &[IntentCase] = &[
    // The create-flow affordance, in every phrasing from the original report.
    IntentCase {
        intent: "new flow",
        affordance: "builtin/new-flow",
        max_rank: 0,
    },
    IntentCase {
        intent: "flow new",
        affordance: "builtin/new-flow",
        max_rank: 2,
    },
    IntentCase {
        intent: "create flow",
        affordance: "builtin/new-flow",
        max_rank: 2,
    },
    IntentCase {
        intent: "create a flow",
        affordance: "builtin/new-flow",
        max_rank: 2,
    },
    // Reference anchors: pre-existing creation affordances through the same
    // harness. If these fail, suspect the harness or registry, not the bar.
    IntentCase {
        intent: "new script",
        affordance: "builtin/new-script",
        max_rank: 0,
    },
    IntentCase {
        intent: "new scriptlet",
        affordance: "builtin/new-extension",
        max_rank: 2,
    },
];

/// (intent, affordance that must NOT rank inside the precision window)
const PRECISION_BAR: &[(&str, &str)] = &[
    ("new script", "builtin/new-flow"),
    ("clipboard", "builtin/new-flow"),
    ("theme", "builtin/new-flow"),
    ("new flow", "builtin/new-script"),
];

/// Ranks inside this window are "surfaced"; precision cases must stay out.
const PRECISION_WINDOW: usize = 3;

fn rank_of(query: &str, affordance: &str) -> Option<usize> {
    let entries = get_builtin_entries(&BuiltInConfig::default());
    let results = fuzzy_search_builtins(&entries, query);
    results.iter().position(|m| m.entry.id == affordance)
}

#[test]
fn recall_bar_every_intent_surfaces_its_affordance_within_band() {
    let mut failures = Vec::new();
    for case in RECALL_BAR {
        match rank_of(case.intent, case.affordance) {
            None => failures.push(format!(
                "\"{}\" surfaces NO {} at all (the affordance is invisible for this intent)",
                case.intent, case.affordance
            )),
            Some(rank) if rank > case.max_rank => failures.push(format!(
                "\"{}\" ranks {} at {} — outside the top-{} band",
                case.intent,
                case.affordance,
                rank,
                case.max_rank + 1
            )),
            Some(_) => {}
        }
    }
    assert!(
        failures.is_empty(),
        "discoverability recall bar failed:\n  {}",
        failures.join("\n  ")
    );
}

#[test]
fn precision_bar_unrelated_intents_never_surface_the_affordance() {
    let mut failures = Vec::new();
    for (intent, affordance) in PRECISION_BAR {
        if let Some(rank) = rank_of(intent, affordance) {
            if rank < PRECISION_WINDOW {
                failures.push(format!(
                    "\"{intent}\" surfaces {affordance} at rank {rank} — rank spam inside the top-{PRECISION_WINDOW} window"
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "discoverability precision bar failed:\n  {}",
        failures.join("\n  ")
    );
}

#[test]
fn the_bar_exercises_both_sides_and_carries_anchors() {
    // A one-sided bar is a wish: recall must include multiple phrasings of
    // the motivating intent, at least one reference anchor, and a non-empty
    // precision side.
    let new_flow_phrasings = RECALL_BAR
        .iter()
        .filter(|c| c.affordance == "builtin/new-flow")
        .count();
    let anchors = RECALL_BAR
        .iter()
        .filter(|c| c.affordance != "builtin/new-flow")
        .count();
    assert!(
        new_flow_phrasings >= 3,
        "recall bar needs at least three phrasings of the motivating intent"
    );
    assert!(anchors >= 1, "recall bar needs a reference anchor");
    assert!(!PRECISION_BAR.is_empty(), "precision bar must not be empty");
}
