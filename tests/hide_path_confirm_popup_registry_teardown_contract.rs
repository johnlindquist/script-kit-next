//! Source-level contract pinning the Pass #25 Fix for the hide-path
//! teardown of the `confirm-popup` automation registry entry — the
//! third sibling in the `register_attached_popup` ↔
//! `remove_automation_window` defensive arc, after Pass #21
//! (`"ai"` / embedded AI) and Pass #23 (`"actions-dialog"`).
//!
//! Background: Run 9 Pass #24 attacker probe filed
//! `attacker-hide-path-confirm-popup-registry-stale` on static
//! evidence (live trigger deferred — opening a confirm popup
//! requires destructive Cmd+Shift+Delete on a script or notes
//! rename/create modal). The bypass shape mirrors Pass #23:
//! `register_attached_popup("confirm-popup", PromptPopup, ...)` at
//! `src/confirm/window.rs:705` upserts the registry entry;
//! `remove_automation_window("confirm-popup")` at
//! `src/confirm/window.rs:385` inside
//! `close_confirm_window(cx: &mut App)` is the sole production
//! removal path; grep of `src/main_sections` and `src/main_entry` for
//! `close_confirm_window` returns zero hits — none of the four Hide
//! dispatchers invoke it, so a confirm popup on screen when main
//! hides leaks its registry entry as `{visible:true,
//! parentWindowId:"main"}` even though AppKit has orderedOut the
//! child (structural sibling of Pass #23's
//! `inspectAutomationWindow("actions-dialog")` failure receipt).
//!
//! The `lifecycle_reset` path at `src/app_impl/lifecycle_reset.rs`
//! has no confirm-popup branch at all — a stronger class than the
//! `actions-dialog` sibling (which at least had a lifecycle_reset
//! path, bypassed only by stdin). So the Fix cannot rely on
//! routing hide through lifecycle_reset to pick up the teardown
//! opportunistically.
//!
//! The Fix adds
//! `crate::windows::remove_automation_window("confirm-popup")` to
//! ALL FOUR hide dispatcher sites, immediately after the Pass #23
//! `remove_automation_window("actions-dialog")` teardown so the
//! three sibling registry writes — embedded-AI (Pass #21),
//! actions-dialog (Pass #23), confirm-popup (this Pass #25) — stay
//! lexically co-located in each dispatcher body. A pure registry
//! call is sufficient because the Pass #23 root-cause work proved
//! AppKit tears down the OS child with main's hide; only the
//! registry HashMap entry persists.
//!
//! These tests pin, for each of the 4 hide dispatcher files:
//!   1. The hide path calls `remove_automation_window("confirm-popup")`
//!      AT ALL.
//!   2. That call appears AFTER the Pass #23
//!      `remove_automation_window("actions-dialog")` teardown, in
//!      lock-step (equal counts, each confirm-popup teardown paired
//!      with its preceding actions-dialog teardown by ordinal
//!      position).
//!   3. The gap between the two teardown lines stays under ~900
//!      bytes — matching the Pass #23 adjacency bound — and no
//!      function boundary appears between them.
//!
//! **Refactor threat**: a contributor consolidating the four hide
//! dispatchers into a shared
//! `reset_main_and_teardown_children(cx: &mut App)` helper
//! (increasingly plausible — each body now carries THREE sibling
//! teardown calls plus the view-reset and re-key work, pushing
//! past 40 near-identical lines per site) could easily drop any
//! one of the three teardowns during extraction while keeping
//! the others. This contract chains the Pass #21 + Pass #23 +
//! Pass #25 lock-steps into a 3-call adjacency so an extracted
//! helper that omits confirm-popup fails loudly on all four sites.
//! The Pass #21 contract
//! (`tests/hide_path_embedded_ai_registry_teardown_contract.rs`)
//! and Pass #23 contract
//! (`tests/hide_path_actions_dialog_registry_teardown_contract.rs`)
//! already defend the other two pairings; adding this file
//! completes the 3-way lock-step across all 4 sites = 36
//! assertions total across the three contract files.

const HIDE_SITES: &[(&str, &str)] = &[
    (
        "window_visibility",
        include_str!("../src/main_sections/window_visibility.rs"),
    ),
    (
        "runtime_stdin",
        include_str!("../src/main_entry/runtime_stdin.rs"),
    ),
    (
        "runtime_stdin_match_core",
        include_str!("../src/main_entry/runtime_stdin_match_core.rs"),
    ),
    (
        "app_run_setup",
        include_str!("../src/main_entry/app_run_setup.rs"),
    ),
];

// doc-anchor-removed: [[removed-docs window behavior#Embedded AI subview — addressable via  in the automation registry]]
#[test]
fn every_hide_site_removes_confirm_popup_registry_entry() {
    for (name, src) in HIDE_SITES {
        assert!(
            src.contains("crate::windows::remove_automation_window(\"confirm-popup\")"),
            "Hide dispatcher {name} MUST call \
             `crate::windows::remove_automation_window(\"confirm-popup\")` \
             to tear down the confirm-popup child registry entry when \
             the main window hides while a confirm dialog (script \
             removal, notes rename/create, etc.) is open. Without \
             this, `listAutomationWindows` post-hide reports a phantom \
             `{{id:\"confirm-popup\", kind:\"promptPopup\", \
             visible:true, parentWindowId:\"main\"}}` entry — the \
             Pass #24 attacker anomaly \
             `attacker-hide-path-confirm-popup-registry-stale`."
        );
    }
}

// doc-anchor-removed: [[removed-docs window behavior#Embedded AI subview — addressable via  in the automation registry]]
#[test]
fn confirm_popup_teardown_follows_actions_dialog_teardown_in_every_site() {
    for (name, src) in HIDE_SITES {
        let actions_positions: Vec<usize> = src
            .match_indices("crate::actions::close_actions_window(")
            .map(|(idx, _)| idx)
            .collect();
        let confirm_positions: Vec<usize> = src
            .match_indices("crate::windows::remove_automation_window(\"confirm-popup\")")
            .map(|(idx, _)| idx)
            .collect();
        assert!(
            !actions_positions.is_empty(),
            "{name} has no `crate::actions::close_actions_window(...)` \
             teardown — the Pass #29 upgrade (which replaced Pass #23's \
             bare registry op) should have failed first; fix that \
             before this one."
        );
        assert!(
            !confirm_positions.is_empty(),
            "{name} has no `remove_automation_window(\"confirm-popup\")` \
             teardown (covered by \
             every_hide_site_removes_confirm_popup_registry_entry — \
             fix that first)."
        );
        assert_eq!(
            actions_positions.len(),
            confirm_positions.len(),
            "{name} has {} close_actions_window calls but {} \
             confirm-popup teardowns — the sibling writes must appear \
             in lock-step, one confirm-popup teardown per \
             close_actions_window call. A mismatch means a hide arm \
             was added/removed without updating its partner.",
            actions_positions.len(),
            confirm_positions.len()
        );
        for (actions_idx, confirm_idx) in actions_positions.iter().zip(confirm_positions.iter()) {
            assert!(
                actions_idx < confirm_idx,
                "In {name}, a `remove_automation_window(\"confirm-popup\")` \
                 call at offset {confirm_idx} precedes its paired \
                 `crate::actions::close_actions_window(...)` call at \
                 offset {actions_idx}. The confirm-popup teardown must \
                 follow the actions-window close so the 3-sibling \
                 lock-step (embedded-AI → close_actions_window → \
                 confirm-popup) remains invariant."
            );
        }
    }
}

// doc-anchor-removed: [[removed-docs window behavior#Embedded AI subview — addressable via  in the automation registry]]
#[test]
fn confirm_popup_teardown_is_adjacent_to_actions_dialog_teardown() {
    for (name, src) in HIDE_SITES {
        let actions_positions: Vec<usize> = src
            .match_indices("crate::actions::close_actions_window(")
            .map(|(idx, _)| idx)
            .collect();
        let confirm_positions: Vec<usize> = src
            .match_indices("crate::windows::remove_automation_window(\"confirm-popup\")")
            .map(|(idx, _)| idx)
            .collect();
        assert_eq!(
            actions_positions.len(),
            confirm_positions.len(),
            "{name} lock-step mismatch (covered by \
             confirm_popup_teardown_follows_actions_dialog_teardown_in_every_site)."
        );
        for (actions_idx, confirm_idx) in actions_positions.iter().zip(confirm_positions.iter()) {
            let gap = confirm_idx.saturating_sub(*actions_idx);
            assert!(
                gap < 700,
                "In {name}, the gap between \
                 `crate::actions::close_actions_window(...)` (offset \
                 {actions_idx}) and the following \
                 `remove_automation_window(\"confirm-popup\")` is \
                 {gap} bytes — must stay under 700 so the sibling \
                 teardowns remain lexically co-located (same function \
                 body, no intervening block). A refactor that pushes \
                 them apart breaks this invariant."
            );
            let between = &src[*actions_idx..*confirm_idx];
            assert!(
                !between.contains("\n    pub fn ")
                    && !between.contains("\n    pub(crate) fn ")
                    && !between.contains("\n    fn "),
                "In {name}, a function boundary appears between the \
                 `crate::actions::close_actions_window(...)` call and \
                 the `remove_automation_window(\"confirm-popup\")` \
                 teardown. They must live in the same function body. \
                 Intervening text:\n{between}"
            );
        }
    }
}
