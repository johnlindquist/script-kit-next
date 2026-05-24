const ARG_HELPERS: &str = include_str!("../src/render_prompts/arg/helpers.rs");
const MINI_RENDER: &str = include_str!("../src/render_prompts/mini.rs");
const STARTUP: &str = include_str!("../src/app_impl/startup.rs");
const STARTUP_NEW_PRELUDE: &str = include_str!("../src/app_impl/startup_new_prelude.rs");
const SELECTION_FALLBACK: &str = include_str!("../src/app_impl/selection_fallback.rs");
const SUBMIT_DIAGNOSTICS: &str = include_str!("../src/app_impl/submit_diagnostics.rs");
const THEME_CHOOSER: &str = include_str!("../src/render_builtins/theme_chooser.rs");
const LIFECYCLE_RESET: &str = include_str!("../src/app_impl/lifecycle_reset.rs");
const PROTOCOL_QUERY_VARIANTS: &str = include_str!("../src/protocol/message/variants/query_ops.rs");
const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");
const DEVTOOLS_FOCUS: &str = include_str!("../scripts/devtools/focus.ts");
const DEVTOOLS_ACT: &str = include_str!("../scripts/devtools/act.ts");

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_idx = source
        .find(start)
        .unwrap_or_else(|| panic!("missing source start: {start}"));
    let end_idx = source[start_idx..]
        .find(end)
        .unwrap_or_else(|| panic!("missing source end after {start}: {end}"));
    &source[start_idx..start_idx + end_idx]
}

fn source_window<'a>(source: &'a str, start: &str, len: usize) -> &'a str {
    let start_idx = source
        .find(start)
        .unwrap_or_else(|| panic!("missing source start: {start}"));
    let end_idx = (start_idx + len).min(source.len());
    &source[start_idx..end_idx]
}

#[test]
fn mini_prompt_enter_records_submit_owner_before_resetting_to_script_list() {
    let submit_fn = source_between(
        ARG_HELPERS,
        "fn submit_arg_prompt_from_current_state(",
        "\n    fn is_valid_builtin_mic_selection",
    );

    assert!(
        MINI_RENDER.contains("this.submit_arg_prompt_from_current_state(&prompt_id, cx);")
            && MINI_RENDER.contains("cx.stop_propagation();"),
        "MiniPrompt Enter must submit through the shared arg submit helper and stop key propagation"
    );
    assert!(
        submit_fn.contains("record_submit_diagnostic")
            && submit_fn.contains("BUILTIN_SNAP_MODE_PROMPT_ID")
            && submit_fn.contains("handle_builtin_snap_mode_selection(&value, cx);"),
        "built-in snap mode prompt submit must record ownership before returning to ScriptList"
    );
}

#[test]
fn builtin_submit_transitions_that_return_to_script_list_arm_enter_echo_guard() {
    let theme_submit = source_between(
        THEME_CHOOSER,
        "pub(crate) fn submit_theme_chooser_from_input_enter(",
        "\n    fn theme_chooser_match_summary",
    );
    assert!(
        SUBMIT_DIAGNOSTICS.contains("fn record_return_to_script_list_submit")
            && SUBMIT_DIAGNOSTICS.contains("self.record_submit_diagnostic(owner, route, None, value, true);")
            && theme_submit.contains("record_return_to_script_list_submit")
            && theme_submit.contains("\"theme_chooser\"")
            && theme_submit.contains("\"submit_theme_chooser_from_input_enter\"")
            && theme_submit.find("record_return_to_script_list_submit")
                < theme_submit.find("self.go_back_or_close(window, cx);"),
        "Theme Designer Apply must use the return-to-ScriptList submit helper before returning to the main menu"
    );

    let reset_positions = source_between(
        LIFECYCLE_RESET,
        "pub(crate) fn reset_window_positions_to_default_main_menu(",
        "\n    pub(crate) fn cancel_script_execution",
    );
    assert!(
        reset_positions.contains("record_return_to_script_list_submit")
            && reset_positions.contains("\"reset_window_positions_to_default_main_menu\"")
            && reset_positions.find("record_return_to_script_list_submit")
                < reset_positions.find("self.reset_to_script_list(cx);"),
        "Reset Window Positions must use the return-to-ScriptList submit helper before resetting to the main menu"
    );
}

#[test]
fn snap_mode_submit_closes_main_window_through_named_transition() {
    let transition = source_between(
        ARG_HELPERS,
        "enum BuiltinPromptSubmitTransition",
        "\n#[derive(Clone, Copy)]\nstruct PromptRenderContext",
    );
    assert!(
        transition.contains("ReturnToScriptList")
            && transition.contains("CloseMainWindow")
            && transition.contains("app.close_and_reset_window(cx);"),
        "built-in prompt submits must use explicit lifecycle transitions"
    );

    let snap_handler = source_between(
        ARG_HELPERS,
        "fn handle_builtin_snap_mode_selection(",
        "\n    #[inline]\n    fn apply_arg_tab_completion",
    );
    assert!(
        snap_handler.contains("BuiltinPromptSubmitTransition::CloseMainWindow.apply(self, cx);"),
        "snap mode final submit must close the main window instead of returning visibly to ScriptList"
    );
    assert!(
        !snap_handler.contains("reset_to_script_list(cx)"),
        "snap mode final submit must not directly reset to ScriptList"
    );
}

#[test]
fn script_list_enter_refuses_immediate_prompt_submit_echo() {
    for (name, source) in [
        ("startup.rs", STARTUP),
        ("startup_new_prelude.rs", STARTUP_NEW_PRELUDE),
    ] {
        let press_enter = source_window(source, "InputEvent::PressEnter", 4200);
        assert!(
            press_enter.contains("submit_arg_prompt_from_current_state(&prompt_id, cx);"),
            "{name} must route Arg/MiniPrompt Enter to the shared prompt submit helper"
        );
        assert!(
            press_enter.contains("should_consume_script_list_enter_after_submit"),
            "{name} must check the submit ownership guard before ScriptList execution"
        );
        assert!(
            press_enter.find("should_consume_script_list_enter_after_submit")
                < press_enter.find("this.execute_selected(cx)"),
            "{name} must run the submit ownership guard before execute_selected"
        );
    }

    assert!(
        SUBMIT_DIAGNOSTICS.contains("script_list_enter_echo_consumed")
            && SUBMIT_DIAGNOSTICS.contains("ENTER_ECHO_GUARD_MS")
            && SUBMIT_DIAGNOSTICS.contains("pendingEnterGuardActive"),
        "submit diagnostics must log consumed ScriptList Enter echoes and expose receipts"
    );
    let execute_selected =
        source_window(SELECTION_FALLBACK, "pub(crate) fn execute_selected", 1400);
    assert!(
        execute_selected
            .contains("should_consume_script_list_enter_after_submit(\"execute_selected\")")
            && execute_selected
                .contains("record_submit_diagnostic(\n                \"launcher\",")
            && execute_selected.find("should_consume_script_list_enter_after_submit")
                < execute_selected.find("execute_menu_syntax_command_invocation"),
        "execute_selected must be guarded and recorded before any ScriptList execution sink"
    );
    let execute_selected_fallback =
        source_window(SELECTION_FALLBACK, "pub fn execute_selected_fallback", 1800);
    assert!(
        execute_selected_fallback.contains(
            "should_consume_script_list_enter_after_submit(\"execute_selected_fallback\")"
        ) && execute_selected_fallback
            .contains("record_submit_diagnostic(\n                \"launcher_fallback\","),
        "execute_selected_fallback must be guarded and recorded before fallback execution"
    );
}

#[test]
fn get_state_exposes_submit_diagnostics_receipt() {
    assert!(
        PROTOCOL_QUERY_VARIANTS.contains("submit_diagnostics: Option<serde_json::Value>")
            && PROTOCOL_QUERY_VARIANTS.contains("rename = \"submitDiagnostics\""),
        "StateResult must include submitDiagnostics for state-first submit proof"
    );
    assert!(
        PROMPT_HANDLER.contains("self.submit_diagnostics_snapshot()"),
        "main-window getState must forward submit diagnostics"
    );
    assert!(
        PROMPT_HANDLER.contains("record_submit_diagnostic(\n                    \"protocol\",\n                    \"submit_current_value\""),
        "protocol-driven submit_current_value must record submit diagnostics"
    );
    assert!(
        PROMPT_HANDLER.contains("record_submit_diagnostic(\n                                            \"protocol\",\n                                            \"forceSubmit\""),
        "protocol forceSubmit must record submit diagnostics"
    );
    assert!(
        DEVTOOLS_FOCUS.contains("submitDiagnostics: state.submitDiagnostics ?? null")
            && DEVTOOLS_ACT.contains("focus.inspect.submitDiagnostics")
            && DEVTOOLS_ACT.contains("isPromptEntityTargetReceipt")
            && DEVTOOLS_ACT.contains("sourceAfter.submitDiagnostics")
            && DEVTOOLS_ACT.contains("submitDiagnostics: {\n      before: before.submitDiagnostics ?? null,\n      after: after.submitDiagnostics ?? null,"),
        "DevTools act/focus receipts must expose before/after submit diagnostics"
    );
}
