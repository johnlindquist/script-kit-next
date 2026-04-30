//! Source-level contract for the Run 2 Pass #45
//! `hide-path-asymmetry-hotkey-vs-stdin-contract` user story.
//!
//! Background: the hotkey toggle handler in
//! `src/main_entry/app_run_setup.rs` (around the `hotkey_detach_acp_requested`
//! tracing event, gated on `matches!(view.current_view, AppView::AcpChatView { .. })`)
//! intentionally does NOT hide the main panel when the user presses the global
//! hotkey while `AcpChatView` is active — it DETACHES the chat into its own
//! popup window (`open_chat_window_with_thread`) and switches the main panel
//! back to `ScriptList` while keeping it visible. The user's mental model is
//! "the hotkey toggles my launcher, not my AI chat — my AI chat persists."
//!
//! By contrast, the stdin `ExternalCommand::Hide` path is an explicit,
//! programmatic "make the main window go away" request. A script or automation
//! tool issuing `{"type":"hide"}` expects the main window to disappear
//! regardless of which subview is currently active; it does NOT expect a
//! side-effect of spawning a new detached popup window. A uniform "always
//! detach ACP on hide" policy would make automation workflows impossible to
//! reason about — a script that called `triggerBuiltin tab-ai` and then
//! `hide` would suddenly create a new user-visible popup from code.
//!
//! The two paths therefore diverge deliberately:
//!   - hotkey toggle + AcpChatView active → detach + keep main visible on ScriptList
//!   - stdin hide + AcpChatView active    → reset main to ScriptList + hide
//!
//! Live-verified on dev-watch pid 89365 (Pass #44 probe + Pass #45 verify):
//! `triggerBuiltin tab-ai` + `show` → `listAutomationWindows` reports
//!   `[{kind:"main", visible:true, semanticSurface:"acpChat"}]`
//! then `hide` → `listAutomationWindows` reports
//!   `[{kind:"main", visible:false, semanticSurface:"scriptList"}]`
//! (no second detached window, no `acpDetached` popup spawned).
//!
//! A future refactor that "unifies" the hide paths — e.g. by having stdin hide
//! delegate to the hotkey toggle helper, or by pulling the detach branch into
//! a shared `dismiss_main()` function that both paths call — would silently
//! break this intentional asymmetry. This contract pins it: the stdin Hide arm
//! in each of the three dispatcher source files must contain the
//! `reset_to_script_list` reset AND must NOT contain any of the
//! hotkey-detach-specific identifiers (`open_chat_window_with_thread`,
//! `hotkey_detach_acp_requested`, `matches!(... AcpChatView ...)`).
//!
//! Complements `tests/hide_rpc_surface_reset_contract.rs` (which pins the
//! RPC/helper parity that Hide arms call `reset_to_script_list` + re-key to
//! `"scriptList"`). This file pins the OTHER half: that Hide arms do NOT
//! adopt the hotkey-path detach branch.

const RUNTIME_STDIN_MATCH_CORE: &str =
    include_str!("../src/main_entry/runtime_stdin_match_core.rs");
const RUNTIME_STDIN: &str = include_str!("../src/main_entry/runtime_stdin.rs");
const APP_RUN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");

/// Extract the textual body of the `ExternalCommand::Hide` match arm from a
/// dispatcher source file. We slice from the `ExternalCommand::Hide {` header
/// up to the next `ExternalCommand::` head (which starts the following arm).
/// This is a deliberately conservative body — it captures EVERY byte between
/// the two markers, so any detach-related identifier living anywhere inside
/// the Hide arm will be caught, not just ones near the top.
fn hide_arm_body<'a>(src: &'a str, path: &str) -> &'a str {
    let start_marker = "ExternalCommand::Hide {";
    let start = src.find(start_marker).unwrap_or_else(|| {
        panic!(
            "{path}: could not locate `ExternalCommand::Hide {{` — the stdin \
             dispatcher must contain a Hide arm for this contract to apply. \
             If the Hide arm was removed or renamed, update this test; do \
             not silently delete the contract."
        )
    });
    // Find the next `ExternalCommand::` after the Hide arm head. The first
    // occurrence AFTER `start + start_marker.len()` is the next arm.
    let search_from = start + start_marker.len();
    let next_offset_rel = src[search_from..]
        .find("ExternalCommand::")
        .unwrap_or_else(|| {
            panic!(
                "{path}: could not locate the next `ExternalCommand::` arm \
                 after Hide — the dispatcher match block should have \
                 additional arms (e.g. SetFilter, TriggerBuiltin). If Hide \
                 is now the final arm, this helper needs to extend its \
                 boundary search to a terminating `}}` brace instead."
            )
        });
    let next_offset = search_from + next_offset_rel;
    &src[start..next_offset]
}

const DETACH_SENTINELS: &[&str] = &[
    // The detach call itself.
    "open_chat_window_with_thread",
    // The hotkey-path tracing event.
    "hotkey_detach_acp_requested",
    "hotkey_detach_acp_completed",
    "hotkey_detach_acp_aborted",
    "hotkey_detach_acp_failed",
    // An AcpChatView-specific branch in the Hide arm. If a future refactor
    // gates behavior on the current view being ACP, the detach asymmetry
    // has already leaked — flag it even before the actual detach call
    // appears.
    "AppView::AcpChatView",
];

fn assert_stdin_hide_arm_is_detach_free(src: &str, path: &str) {
    let body = hide_arm_body(src, path);

    assert!(
        body.contains("view.reset_to_script_list(ctx)"),
        "{path} `ExternalCommand::Hide` arm must call \
         `view.reset_to_script_list(ctx)` so a hide issued while in a subview \
         (e.g. `FileSearchView` / `AcpChatView`) resets the view BEFORE the \
         automation surface re-key, preventing a stale subview tag from \
         leaking across the next show. This is the positive half of the \
         contract — the asymmetry documented in this file only holds if the \
         reset itself is present."
    );

    assert!(
        body.contains("update_automation_semantic_surface") && body.contains("\"scriptList\""),
        "{path} `ExternalCommand::Hide` arm must re-key the automation \
         `semanticSurface` to `\"scriptList\"` via \
         `update_automation_semantic_surface(\"main\", Some(\"scriptList\".to_string()))` \
         (or equivalent) after the view reset. Without this, \
         `listAutomationWindows.windows[0].semanticSurface` could stay pinned \
         to the prior subview tag even though the view is already back to \
         ScriptList."
    );

    for sentinel in DETACH_SENTINELS {
        assert!(
            !body.contains(sentinel),
            "{path} `ExternalCommand::Hide` arm contains detach-path \
             sentinel `{sentinel}`. The stdin hide path is intentionally \
             asymmetric with the hotkey-toggle detach path: a programmatic \
             `{{\"type\":\"hide\"}}` request must HIDE the main window, not \
             spawn a detached ACP popup as a side effect. The hotkey-toggle \
             detach lives in `src/main_entry/app_run_setup.rs` around the \
             `hotkey_detach_acp_requested` tracing event and is gated on the \
             user pressing the global hotkey. If you need a stdin command \
             that detaches ACP, add a NEW explicit command (e.g. \
             `ExternalCommand::DetachAcpChat`) rather than folding detach \
             into `Hide` — the two user intents are different (\"toggle my \
             launcher, keep my chat alive\" vs. \"make this window go \
             away\") and must stay wired to two distinct commands."
        );
    }
}

// @lat: [[lat.md/acp-chat#ACP Chat#Detached window behavior]]
#[test]
fn runtime_stdin_match_core_hide_arm_has_no_detach_branch() {
    assert_stdin_hide_arm_is_detach_free(
        RUNTIME_STDIN_MATCH_CORE,
        "src/main_entry/runtime_stdin_match_core.rs",
    );
}

// @lat: [[lat.md/acp-chat#ACP Chat#Detached window behavior]]
#[test]
fn runtime_stdin_hide_arm_has_no_detach_branch() {
    assert_stdin_hide_arm_is_detach_free(RUNTIME_STDIN, "src/main_entry/runtime_stdin.rs");
}

// @lat: [[lat.md/acp-chat#ACP Chat#Detached window behavior]]
#[test]
fn app_run_setup_hide_arm_has_no_detach_branch() {
    assert_stdin_hide_arm_is_detach_free(APP_RUN_SETUP, "src/main_entry/app_run_setup.rs");
}
