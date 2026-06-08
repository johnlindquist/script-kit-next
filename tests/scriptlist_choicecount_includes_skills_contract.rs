//! Source-level contract test for the Run 5 Pass #5 fix of
//! `[?] stateresult-visible-exceeds-total-count` (filed by Run 5 Pass #4
//! attacker-mode probe).
//!
//! Pins the invariant that `getState.choiceCount` in the `ScriptList`
//! view sums EVERY collection that `fuzzy_search_unified_all_with_skills`
//! searches over — so that `visibleChoiceCount` (= `filtered_results().len()`)
//! can never exceed `choiceCount`. Without this, any non-empty `skills`
//! (or future 6th collection) causes automation that relies on the
//! subset invariant to see nonsensical receipts.
//!
//! Two surfaces are pinned here because either drifting would silently
//! re-introduce the bug:
//!
//! (a) `src/prompt_handler/mod.rs` — the `choiceCount` sum in the
//!     `ScriptList` arm must include `self.skills.len()`.
//! (b) `src/app_impl/filtering_cache.rs` — the
//!     `fuzzy_search_unified_all_with_skills` call must pass exactly
//!     those 5 collections. If a contributor adds a 6th collection to
//!     the search (e.g. `agents`, `cached_windows`, fallback items),
//!     test (a) will still pass but this test (b) flags the drift so
//!     the contributor knows to also update the `choiceCount` sum.

const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");
const FILTERING_CACHE: &str = include_str!("../src/app_impl/filtering_cache.rs");

fn normalized(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn scriptlist_choice_count_includes_skills() {
    // The exact expression — if `+ self.skills.len()` is dropped during a
    // refactor, automation's documented "subset" invariant
    // (`visibleChoiceCount <= choiceCount`) silently breaks again.
    //
    // Refactor threat: A contributor consolidating the per-view state
    // tuple in `state_for_script_list` (or extracting a helper) could
    // drop one of the `.len()` adds and not notice — the unit tests for
    // search continue to pass because they don't exercise getState.
    // This contract ties the sum's shape to the test file so the drop
    // is caught at build time.
    let prompt_handler = normalized(PROMPT_HANDLER);
    assert!(
        prompt_handler.contains(
            "self.scripts.len() + self.scriptlets.len() + self.builtin_entries.len() + self.apps.len() + self.skills.len(),"
        ),
        "src/prompt_handler/mod.rs: the `choiceCount` sum in the \
         ScriptList arm of `state_for_script_list` (around line 1953) \
         must include `self.scripts.len() + self.scriptlets.len() + \
         self.builtin_entries.len() + self.apps.len() + \
         self.skills.len()` in that exact order. Dropping `self.skills.len()` \
         was the Run 5 Pass #4 attacker anomaly \
         `stateresult-visible-exceeds-total-count`: automation saw \
         `visibleChoiceCount > choiceCount` whenever any skill was \
         loaded. Add/reorder carefully and update this test in the same \
         commit."
    );
}

#[test]
fn search_entry_point_takes_exactly_five_collections() {
    // If the fuzzy-search entry point grows a 6th collection, the
    // `choiceCount` sum in `state_for_script_list` must grow to match
    // or the Pass #4 anomaly returns. This test flags the drift in the
    // commit that adds the 6th arg.
    //
    // Refactor threat: A contributor adding `agents: &[Agent]` (or
    // windows, fallbacks) as a search input would naturally update this
    // call site but might forget the sibling `choiceCount` sum. The
    // failure message here points them at the sum explicitly.
    let filtering_cache = normalized(FILTERING_CACHE);
    assert!(
        filtering_cache.contains(
            "scripts::fuzzy_search_unified_all_with_skills( &self.scripts, &self.scriptlets, &self.builtin_entries, &self.apps, &self.skills, search_text, )"
        ),
        "src/app_impl/filtering_cache.rs: `recompute_filtered_results` \
         must call `fuzzy_search_unified_all_with_skills` with exactly \
         the 5 collections `&self.scripts, &self.scriptlets, \
         &self.builtin_entries, &self.apps, &self.skills`, followed by \
         normalized search text. If a 6th collection is added here, also update \
         the `choiceCount` sum in `src/prompt_handler/mod.rs` (around \
         line 1953, in the `ScriptList` arm of the `match &self.current_view` \
         block) to include it — otherwise the \
         `visibleChoiceCount <= choiceCount` subset invariant breaks \
         for automation consumers."
    );
}

#[test]
fn tracing_event_fields_match_search_inputs() {
    // The `main_menu_filtered_results_recomputed` tracing event is the
    // diagnostic breadcrumb the Run 5 Pass #4 attacker probe used to
    // identify the root cause (its `skill_count=15` field exposed the
    // 15-item delta between `choiceCount` and `visibleChoiceCount`).
    // Keep the field set in lockstep with the search inputs so future
    // debugging stays possible.
    //
    // Refactor threat: Logging noise reduction could collapse these
    // fields into a single `total_count` and hide the per-collection
    // breakdown. Without the breakdown, the Pass #4 root-cause analysis
    // would have required running the app under a debugger.
    let required_fields = [
        "script_count = self.scripts.len()",
        "scriptlet_count = self.scriptlets.len()",
        "builtin_count = self.builtin_entries.len()",
        "app_count = self.apps.len()",
        "skill_count = self.skills.len()",
    ];
    for field in required_fields {
        assert!(
            FILTERING_CACHE.contains(field),
            "src/app_impl/filtering_cache.rs: the \
             `main_menu_filtered_results_recomputed` tracing event must \
             emit `{}` so future attacker probes can correlate \
             `choiceCount` vs `visibleChoiceCount` per collection. \
             Dropping this field would have made the Run 5 Pass #4 \
             anomaly root-cause analysis impossible without a debugger.",
            field
        );
    }
}
