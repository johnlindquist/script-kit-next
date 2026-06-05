//! Source-level contract pinning the Pass #21 Fix for the hide-path
//! teardown of the embedded-AI automation registry entry.
//!
//! Background: Run 9 Pass #20 (attacker probe, 33 actions × 4 categories,
//! commit `237ad3245`) filed `[?] attacker-hide-path-embedded-ai-registry-stale`
//! — after `triggerBuiltin tab-ai` then `{type:"hide"}`,
//! `listAutomationWindows` returned a stale `{id:"ai", kind:"ai",
//! visible:true, semanticSurface:"acpChat", parentWindowId:"main"}` entry
//! even though main's own entry had been correctly re-keyed to
//! `semanticSurface:"scriptList"` with `visible:false`. Root cause: the
//! four hide dispatchers called `reset_to_script_list(ctx)` +
//! `update_automation_semantic_surface("main", …)` but NEVER
//! `ensure_embedded_ai_window(false)` — so the `ai` child-entry that
//! `ensure_embedded_ai_window(true)` writes on tab-ai entry persisted
//! across hide.
//!
//! The Fix adds `ensure_embedded_ai_window(false)` to ALL FOUR hide
//! dispatcher sites, symmetric with the already-pinned
//! `close_acp_chat_to_script_list` teardown at
//! `src/app_impl/tab_ai_mode/mod.rs:3151`.
//!
//! The hide path has since been split: the dispatcher tears down stale child
//! automation entries before hiding, then schedules
//! `defer_reset_to_script_list_after_main_window_hidden` so the ScriptList
//! reset and main-surface re-key happen after the native hide turn. These tests
//! pin the current owner split instead of requiring the old inline
//! `reset_to_script_list` + `update_automation_semantic_surface` shape in
//! every dispatcher.
//!
//! These tests pin, for each of the 4 hide dispatcher files:
//! 1. The hide path calls `ensure_embedded_ai_window(false)` AT ALL.
//! 2. The dispatcher schedules the hidden ScriptList reset after enqueueing
//!    native hide.
//! 3. The hidden reset helper owns the ScriptList reset, main-surface re-key,
//!    and hidden visibility update.
//!
//! **Refactor threat**: a contributor refactoring one of the four hide
//! dispatchers to centralize the "reset view + re-key automation" block
//! into a helper (plausible consolidation — the four dispatchers already
//! share identical 30-line bodies) could easily drop the
//! `ensure_embedded_ai_window(false)` call from the extraction. The
//! pairing is code-only, not contract-tested except by this file. A
//! moved or deleted call in any one of the four sites would silently
//! regress the receipt for every hide-from-acpChat sequence on that
//! dispatcher path and reopen the Pass #20 anomaly. This contract
//! catches that edit at test time.

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
const LIFECYCLE_RESET: &str = include_str!("../src/app_impl/lifecycle_reset.rs");

#[test]
fn every_hide_site_calls_ensure_embedded_ai_window_false() {
    for (name, src) in HIDE_SITES {
        assert!(
            src.contains("ensure_embedded_ai_window(false)"),
            "Hide dispatcher {name} MUST call \
             `ensure_embedded_ai_window(false)` to tear down the `ai` \
             child registry entry when the main window hides from an \
             embedded-ACP view. Without this, `listAutomationWindows` \
             post-hide leaves a stale `{{id:\"ai\", visible:true, \
             semanticSurface:\"acpChat\"}}` entry that disagrees with \
             its parent main entry on both visibility and semantic \
             surface — the Pass #20 attacker anomaly \
             `attacker-hide-path-embedded-ai-registry-stale`."
        );
    }
}

#[test]
fn hide_sites_schedule_hidden_scriptlist_reset_after_native_hide() {
    for (name, src) in HIDE_SITES {
        let teardown_positions: Vec<usize> = src
            .match_indices("ensure_embedded_ai_window(false)")
            .map(|(idx, _)| idx)
            .collect();
        assert!(
            !teardown_positions.is_empty(),
            "{name} has no `ensure_embedded_ai_window(false)` call (covered \
             by every_hide_site_calls_ensure_embedded_ai_window_false — \
             fix that first)."
        );
        let hide_idx = src
            .find("defer_hide_main_window(")
            .unwrap_or_else(|| panic!("{name} must enqueue native main-window hide"));
        let reset_idx = src
            .find("defer_reset_to_script_list_after_main_window_hidden(")
            .unwrap_or_else(|| panic!("{name} must schedule hidden ScriptList reset"));
        assert!(
            hide_idx < reset_idx,
            "{name} must schedule the hidden ScriptList reset after \
             enqueueing native hide so the reset/re-key cannot render a \
             visible ScriptList frame while AppKit is closing the panel."
        );
    }
}

#[test]
fn hidden_reset_helper_owns_scriptlist_rekey_and_hidden_visibility() {
    let helper_start = LIFECYCLE_RESET
        .find("pub(crate) fn reset_hidden_main_window_to_script_list(")
        .expect("lifecycle_reset.rs must define hidden ScriptList reset helper");
    let helper_body =
        &LIFECYCLE_RESET[helper_start..(helper_start + 900).min(LIFECYCLE_RESET.len())];

    assert!(
        helper_body.contains("self.reset_to_script_list(cx);")
            && helper_body.contains("self.rekey_main_automation_surface_from_current_view();")
            && helper_body.contains("crate::windows::set_automation_visibility(\"main\", false);"),
        "hidden reset helper must own the post-hide ScriptList reset, \
         main automation surface re-key, and hidden visibility update"
    );
}
