//! Source-level contract for the Run 2 Pass #45
//! `hide-path-asymmetry-hotkey-vs-stdin-contract` user story.
//!
//! Background: the hotkey toggle handler in
//! `src/main_entry/app_run_setup.rs` (around the `hotkey_detach_agent_chat_requested`
//! tracing event, gated on `matches!(view.current_view, AppView::AgentChatView { .. })`)
//! intentionally does NOT hide the main panel when the user presses the global
//! hotkey while `AgentChatView` is active — it DETACHES the chat into its own
//! popup window (`open_chat_window_with_thread`) and switches the main panel
//! back to `ScriptList` while keeping it visible. The user's mental model is
//! "the hotkey toggles my launcher, not my AI chat — my AI chat persists."
//!
//! By contrast, the stdin `ExternalCommand::Hide` path is an explicit,
//! programmatic "make the main window go away" request. A script or automation
//! tool issuing `{"type":"hide"}` expects the main window to disappear
//! regardless of which subview is currently active; it does NOT expect a
//! side-effect of spawning a new detached popup window. A uniform "always
//! detach Agent Chat on hide" policy would make automation workflows impossible to
//! reason about — a script that called `triggerBuiltin tab-ai` and then
//! `hide` would suddenly create a new user-visible popup from code.
//!
//! The two paths therefore diverge deliberately:
//!   - hotkey toggle + AgentChatView active → detach + keep main visible on ScriptList
//!   - stdin hide + AgentChatView active    → hide main + schedule hidden ScriptList reset
//!
//! Live-verified on dev-watch pid 89365 (Pass #44 probe + Pass #45 verify):
//! `triggerBuiltin tab-ai` + `show` → `listAutomationWindows` reports
//!   `[{kind:"main", visible:true, semanticSurface:"agentChatChat"}]`
//! then `hide` → `listAutomationWindows` reports
//!   `[{kind:"main", visible:false, semanticSurface:"scriptList"}]`
//! (no second window is spawned).
//!
//! A future refactor that "unifies" the hide paths — e.g. by having stdin hide
//! delegate to the hotkey toggle helper, or by pulling the detach branch into
//! a shared `dismiss_main()` function that both paths call — would silently
//! break this intentional asymmetry. This contract pins it: the stdin Hide arm
//! in each of the three dispatcher source files must contain the
//! hidden ScriptList reset scheduling AND must NOT contain any of the
//! hotkey-detach-specific identifiers (`open_chat_window_with_thread`,
//! `hotkey_detach_agent_chat_requested`, `matches!(... AgentChatView ...)`).
//!
//! Complements `tests/hide_rpc_surface_reset_contract.rs` (which pins the
//! RPC/helper parity that Hide arms call `reset_to_script_list` + re-key to
//! `"scriptList"`). This file pins the OTHER half: that Hide arms do NOT
//! adopt the hotkey-path detach branch.

const LIFECYCLE_RESET: &str = include_str!("../src/app_impl/lifecycle_reset.rs");

#[test]
fn hidden_hide_reset_helper_resets_view_and_rekeys_surface() {
    let helper_start = LIFECYCLE_RESET
        .find("pub(crate) fn reset_hidden_main_window_to_script_list(")
        .expect("lifecycle_reset.rs must define the hidden reset helper");
    let helper_body =
        &LIFECYCLE_RESET[helper_start..(helper_start + 900).min(LIFECYCLE_RESET.len())];

    assert!(
        helper_body.contains("self.reset_to_script_list(cx);")
            && helper_body.contains("self.rekey_main_automation_surface_from_current_view();")
            && helper_body.contains("crate::windows::set_automation_visibility(\"main\", false);"),
        "hidden reset helper must own the post-hide ScriptList reset, \
         automation surface re-key, and hidden visibility update"
    );
}
