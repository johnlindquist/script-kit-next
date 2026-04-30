//! Source-level contract test for the `main-menu-cmd-enter-ai` user story
//! (Run 2 Pass #24).
//!
//! Pass #16 recorded the tool gap: `ExternalCommand::SimulateKey` had no
//! Cmd+Enter arm for `AppView::ScriptList`, so automation callers could
//! not exercise the universal "do this to that" AI entry point from the
//! main menu. The live GPUI handler at
//! `src/render_script_list/mod.rs:881-890` checks
//! `sk_is_key_enter(key) && event.keystroke.modifiers.platform && !shift
//! && !alt && !control` and calls
//! `try_route_global_cmd_enter_to_acp_context_capture(cx)`; without the
//! matching stdin arm, stdin callers fell through to the plain `enter`
//! case in the dispatcher and executed the selected item instead of
//! opening ACP with the item as explicit context.
//!
//! Pass #24 adds the arm in BOTH simulateKey dispatchers
//! (`src/main_entry/runtime_stdin_match_simulate_key.rs` — canonical —
//! and `src/main_entry/app_run_setup.rs` — embedded copy). This contract
//! pins the arm's presence + shape so a mechanical refactor of either
//! match block cannot silently drop it and regress the story.

const CANONICAL_SIMULATEKEY: &str =
    include_str!("../src/main_entry/runtime_stdin_match_simulate_key.rs");
const APP_RUN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn simulate_key_dispatchers_route_cmd_enter_in_scriptlist_to_acp_context_capture() {
    for (name, source) in [
        (
            "src/main_entry/runtime_stdin_match_simulate_key.rs",
            CANONICAL_SIMULATEKEY,
        ),
        ("src/main_entry/app_run_setup.rs", APP_RUN_SETUP),
    ] {
        // Find the ScriptList simulateKey arm. Both files have exactly
        // one `AppView::ScriptList =>` inside the simulateKey dispatcher;
        // the canonical file's only match is the dispatcher itself, and
        // `app_run_setup.rs` carries a match at line ~2023 inside the
        // triple-embedded copy. Grep for the shared anchor that SHOULD
        // immediately follow the arm head — the Cmd+K toggle-actions
        // call.
        let scriptlist_anchor = "// Main script list key handling\n                                        if has_cmd && key_lower == \"k\" {";
        let Some(anchor_idx) = source.find(scriptlist_anchor) else {
            panic!(
                "{name} is missing the `AppView::ScriptList` simulateKey arm's \
                 Cmd+K anchor (`// Main script list key handling` + \
                 `if has_cmd && key_lower == \"k\" {{`). Layout has shifted; \
                 update this test before the Cmd+Enter contract assertions \
                 can run."
            );
        };
        // Scope: from the Cmd+K anchor through the next 60 lines. The
        // Cmd+Enter arm must appear inside this region — it is the very
        // next `else if` branch after Cmd+K in both dispatchers.
        let arm_body: String = source[anchor_idx..]
            .lines()
            .take(60)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            arm_body.contains("has_cmd")
                && arm_body.contains("key_lower == \"enter\"")
                && arm_body.contains("!has_shift")
                && arm_body.contains("!_has_alt")
                && arm_body.contains("!_has_ctrl"),
            "{name} `AppView::ScriptList` simulateKey arm must gate the \
             Cmd+Enter routing on `has_cmd && key_lower == \"enter\" && \
             !has_shift && !_has_alt && !_has_ctrl`, mirroring the live \
             GPUI handler at `src/render_script_list/mod.rs:881-890`. \
             Loosening the modifier gate (e.g. letting Cmd+Shift+Enter \
             route to ACP too) would diverge the stdin path from the \
             real keybinding."
        );
        assert!(
            arm_body.contains("view.try_route_global_cmd_enter_to_acp_context_capture(ctx);"),
            "{name} `AppView::ScriptList` simulateKey arm must call \
             `view.try_route_global_cmd_enter_to_acp_context_capture(ctx)` \
             when Cmd+Enter is detected — this is the same function the \
             live GPUI handler invokes, so both paths share one routing \
             decision. Any replacement call would drift the stdin path \
             from the live keybinding."
        );
    }
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn simulate_key_cmd_enter_arm_precedes_plain_enter_in_scriptlist() {
    // Ordering contract: the Cmd+Enter arm MUST appear before the
    // plain-`enter` arm inside the ScriptList match block. If it were
    // reordered after the plain handler, `has_cmd && key_lower ==
    // "enter"` would still match but only after the plain handler
    // already fired `view.execute_selected(ctx)` — the story's ACP-open
    // path would never run. Both files use `else if` chaining rather
    // than a match on a composite key, so the branch order IS semantic.
    for (name, source) in [
        (
            "src/main_entry/runtime_stdin_match_simulate_key.rs",
            CANONICAL_SIMULATEKEY,
        ),
        ("src/main_entry/app_run_setup.rs", APP_RUN_SETUP),
    ] {
        let cmd_enter_idx = source
            .find("SimulateKey: Cmd+Enter - route to ACP context capture")
            .unwrap_or_else(|| {
                panic!(
                    "{name} missing the Cmd+Enter log line — either the arm \
                     was removed or the log string drifted. Re-check this \
                     contract after any renames."
                )
            });
        let plain_enter_idx = source
            .find("SimulateKey: Enter - execute selected")
            .unwrap_or_else(|| {
                panic!(
                    "{name} missing the plain-Enter log line — ScriptList \
                     execute-selected handler was renamed or removed; \
                     update this contract."
                )
            });
        assert!(
            cmd_enter_idx < plain_enter_idx,
            "{name} Cmd+Enter arm must appear BEFORE the plain-Enter arm \
             inside the ScriptList match block — got cmd_enter@{cmd_enter_idx}, \
             plain_enter@{plain_enter_idx}. Reversing the order would \
             execute the selected item before the Cmd+Enter gate could \
             route to ACP, silently regressing `main-menu-cmd-enter-ai`."
        );
    }
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn try_route_global_cmd_enter_to_acp_context_capture_still_defined() {
    // If the helper is renamed or removed, the stdin arms above would
    // fail to compile — but the contract is explicit about the function
    // name as part of the story's acceptance (it's referenced in
    // stories.md Pass #16 and is the single shared routing decision
    // between the live GPUI handler and the stdin simulateKey arm).
    // Pin the definition so any rename becomes a deliberate, multi-file
    // edit rather than an accidental stdin-only breakage.
    const TAB_AI_MODE: &str = include_str!("../src/app_impl/tab_ai_mode/mod.rs");
    assert!(
        TAB_AI_MODE.contains("pub(crate) fn try_route_global_cmd_enter_to_acp_context_capture("),
        "`src/app_impl/tab_ai_mode/mod.rs` must define \
         `try_route_global_cmd_enter_to_acp_context_capture` — the \
         shared routing helper for Cmd+Enter → ACP. Renaming it without \
         updating the simulateKey dispatchers and \
         `src/render_script_list/mod.rs:881-890` would silently diverge \
         the stdin and live-GPUI paths."
    );
}
