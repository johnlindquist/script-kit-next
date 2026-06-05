//! Source-level contract pinning the Run 9 Pass #23 + Pass #29 Fix for
//! the hide-path teardown of the actions-dialog popup.
//!
//! ## History
//!
//! **Pass #23** (commit `a1349de4d`): discovered the bare registry
//! entry `{id:"actions-dialog", visible:true, parentWindowId:"main"}`
//! persisted across hide — the OS child was torn down with main but the
//! automation registry lied. Fix: added
//! `crate::windows::remove_automation_window("actions-dialog")` to all
//! four hide dispatchers. This closed the `listAutomationWindows`
//! falsifier but left a deeper bug alive.
//!
//! **Pass #29** (this pass, `cmd-k-on-unfocused-clipboard-pops-overlay-not-actions`):
//! discovered Pass #23's fix was structurally under-specified. The
//! `ACTIONS_WINDOW` static (`Mutex<Option<WindowHandle<ActionsWindow>>>`
//! at `src/actions/window.rs`) kept holding a stale handle after hide
//! because the bare `remove_automation_window` call only touched the
//! registry map. Reproduction sequence (app.log + `listAutomationWindows`
//! receipts on session pid 47963):
//!   1. `simulateKey cmd+k` on main opens the popup — `ACTIONS_WINDOW`
//!      static gets `Some(handle)`; registry gains `actions-dialog`.
//!   2. `{type:"hide"}` — Pass #23 clears the registry, but the static
//!      keeps `Some(handle)` (the handle itself is to a window whose
//!      NSWindow was torn down with main, so the OS side is dead but
//!      the Rust `Option` is stale-non-None).
//!   3. `triggerBuiltin clipboardHistory` + `show` (unfocused: NSPanel
//!      non-activating style, `isKeyWindow actual=false`).
//!   4. `simulateKey cmd+k` routes through the clipboard view's Cmd+K
//!      handler at `src/render_builtins/clipboard.rs:183-189`, which
//!      calls `toggle_clipboard_actions`. That function at
//!      `src/render_builtins/actions.rs:333` reads
//!      `if self.show_actions_popup || is_actions_window_open()` —
//!      `is_actions_window_open()` returns `true` from the stale static,
//!      so the CLOSE branch fires: `close_actions_window` + `pop_focus_overlay`
//!      pop whichever overlay is on top of the FocusCoordinator stack
//!      (the ClipboardHistoryView itself, since no actions popup was
//!      actually open) → user-observable as "Cmd+K on the clipboard
//!      history view dumps me back to the main menu instead of opening
//!      the clipboard actions dialog".
//!
//! Fix: upgrade all four hide dispatchers from the bare
//! `remove_automation_window("actions-dialog")` call (Pass #23) to the
//! full `crate::actions::close_actions_window(cx)` call. The latter
//! clears BOTH the `ACTIONS_WINDOW` static AND the registry entry (its
//! first line is `crate::windows::remove_automation_window("actions-dialog")`),
//! so the upgrade is strictly stronger than the pre-#29 call. The
//! registry-lie invariant that Pass #23 pinned is preserved; the
//! `is_actions_window_open()=true` invariant that Pass #29 uncovered is
//! newly pinned.
//!
//! These tests pin, for each of the 4 hide dispatcher files:
//!   1. The hide path calls `crate::actions::close_actions_window(` —
//!      the Pass #29 full teardown. Grep for the fully-qualified call
//!      prefix so a contributor renaming the variable from `ctx`/`cx`
//!      doesn't silently break the pin.
//!   2. That call is AFTER `ensure_embedded_ai_window(false)` (the
//!      Pass #21 teardown) — the two sibling writes must stay adjacent
//!      so neither drifts independently under a refactor.
//!   3. The gap between the two teardown lines stays under ~1100 bytes —
//!      any growth beyond that means they are no longer co-located and
//!      the lock-step pairing has weakened. (Widened from 900 to 1100
//!      because the Pass #29 explanation comment is denser than the
//!      Pass #23 comment it replaces.)
//!
//! **Refactor threat**: a contributor consolidating the four hide
//! dispatchers into a shared "reset-main-and-teardown-children" helper
//! (plausible — the bodies are already 30+ lines of near-identical
//! sibling teardown) could easily drop the
//! `crate::actions::close_actions_window(cx)` line from the extraction
//! while keeping the louder `ensure_embedded_ai_window(false)` call.
//! Or — more dangerous — could "optimize" the upgrade back down to
//! `remove_automation_window("actions-dialog")` on the grounds that the
//! OS child is already gone, not realizing that the `ACTIONS_WINDOW`
//! static remains stale and breaks the Cmd+K route on subsequent
//! unfocused shows. This contract fails loudly on either mutation.

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

const CLOSE_ACTIONS_CALL_PREFIX: &str = "crate::actions::close_actions_window(";
const LEGACY_BARE_REGISTRY_CALL: &str =
    "crate::windows::remove_automation_window(\"actions-dialog\")";

#[test]
fn every_hide_site_closes_actions_window_fully() {
    for (name, src) in HIDE_SITES {
        assert!(
            src.contains(CLOSE_ACTIONS_CALL_PREFIX),
            "Hide dispatcher {name} MUST call \
             `crate::actions::close_actions_window(...)` to fully tear \
             down the actions-dialog popup when main hides. A bare \
             `remove_automation_window(\"actions-dialog\")` (the \
             pre-Pass-#29 call) is insufficient: it cleans the \
             automation registry but leaves the `ACTIONS_WINDOW` static \
             `Mutex<Option<WindowHandle>>` holding a stale \
             `Some(handle)`. On a subsequent unfocused \
             `simulateKey cmd+k` against any built-in view, \
             `is_actions_window_open()` reads that stale `Some` and \
             routes the Cmd+K through the CLOSE branch of \
             `toggle_clipboard_actions` (or any sibling toggle), \
             popping whichever overlay was top instead of opening the \
             actions dialog. Filed as \
             `cmd-k-on-unfocused-clipboard-pops-overlay-not-actions` \
             (Run 9 Pass #28 Reproduce / Pass #29 Fix)."
        );
    }
}

#[test]
fn no_hide_site_uses_legacy_bare_registry_teardown() {
    for (name, src) in HIDE_SITES {
        assert!(
            !src.contains(LEGACY_BARE_REGISTRY_CALL),
            "Hide dispatcher {name} contains a bare \
             `{LEGACY_BARE_REGISTRY_CALL}` call — this is the Pass #23 \
             shape that Pass #29 upgraded to full \
             `crate::actions::close_actions_window(...)`. Reverting \
             re-introduces the Pass #28 repro \
             `cmd-k-on-unfocused-clipboard-pops-overlay-not-actions`: \
             the `ACTIONS_WINDOW` static stays \
             stale-`Some(handle)` after hide, so a later Cmd+K on an \
             unfocused built-in view takes the CLOSE branch and pops \
             the wrong overlay. The full `close_actions_window(cx)` \
             call is strictly stronger — it runs \
             `remove_automation_window(\"actions-dialog\")` internally \
             on its first line AND clears the static."
        );
    }
}

#[test]
fn actions_dialog_teardown_follows_embedded_ai_teardown_in_every_site() {
    for (name, src) in HIDE_SITES {
        let ai_positions: Vec<usize> = src
            .match_indices("crate::windows::ensure_embedded_ai_window(false)")
            .map(|(idx, _)| idx)
            .collect();
        let actions_positions: Vec<usize> = src
            .match_indices(CLOSE_ACTIONS_CALL_PREFIX)
            .map(|(idx, _)| idx)
            .collect();
        assert!(
            !ai_positions.is_empty(),
            "{name} has no `ensure_embedded_ai_window(false)` teardown \
             — the Pass #21 contract should have failed first; fix that \
             before this one."
        );
        assert!(
            !actions_positions.is_empty(),
            "{name} has no `{CLOSE_ACTIONS_CALL_PREFIX}` teardown \
             (covered by every_hide_site_closes_actions_window_fully — \
             fix that first)."
        );
        assert_eq!(
            ai_positions.len(),
            actions_positions.len(),
            "{name} has {} embedded-AI teardowns but {} \
             close_actions_window calls — the two sibling writes must \
             appear in lock-step, one close_actions_window per \
             embedded-AI teardown. A mismatch means a hide arm was \
             added/removed without updating its partner.",
            ai_positions.len(),
            actions_positions.len()
        );
        for (ai_idx, actions_idx) in ai_positions.iter().zip(actions_positions.iter()) {
            assert!(
                ai_idx < actions_idx,
                "In {name}, a `{CLOSE_ACTIONS_CALL_PREFIX}` call at \
                 offset {actions_idx} precedes its paired \
                 `ensure_embedded_ai_window(false)` call at offset \
                 {ai_idx}. The actions-window close must follow the \
                 embedded-AI teardown so the two sibling writes remain \
                 in lock-step and a future refactor cannot split them \
                 silently."
            );
        }
    }
}

#[test]
fn actions_dialog_teardown_is_adjacent_to_embedded_ai_teardown() {
    for (name, src) in HIDE_SITES {
        let ai_positions: Vec<usize> = src
            .match_indices("crate::windows::ensure_embedded_ai_window(false)")
            .map(|(idx, _)| idx)
            .collect();
        let actions_positions: Vec<usize> = src
            .match_indices(CLOSE_ACTIONS_CALL_PREFIX)
            .map(|(idx, _)| idx)
            .collect();
        assert_eq!(
            ai_positions.len(),
            actions_positions.len(),
            "{name} lock-step mismatch (covered by \
             actions_dialog_teardown_follows_embedded_ai_teardown_in_every_site)."
        );
        for (ai_idx, actions_idx) in ai_positions.iter().zip(actions_positions.iter()) {
            let gap = actions_idx.saturating_sub(*ai_idx);
            assert!(
                gap < 1100,
                "In {name}, the gap between \
                 `ensure_embedded_ai_window(false)` (offset {ai_idx}) \
                 and the following `{CLOSE_ACTIONS_CALL_PREFIX}` is \
                 {gap} bytes — must stay under 1100 so the two sibling \
                 teardowns remain lexically co-located. A refactor \
                 that pushes them apart breaks this invariant."
            );
            let between = &src[*ai_idx..*actions_idx];
            assert!(
                !between.contains("\n    pub fn ")
                    && !between.contains("\n    pub(crate) fn ")
                    && !between.contains("\n    fn "),
                "In {name}, a function boundary appears between the \
                 `ensure_embedded_ai_window(false)` teardown and the \
                 `{CLOSE_ACTIONS_CALL_PREFIX}` call. They must live in \
                 the same function body. Intervening text:\n{between}"
            );
        }
    }
}
