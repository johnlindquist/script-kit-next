const ARG_HELPERS: &str = include_str!("../src/render_prompts/arg/helpers.rs");
const MINI_RENDER: &str = include_str!("../src/render_prompts/mini.rs");
const STARTUP: &str = include_str!("../src/app_impl/startup.rs");
const STARTUP_NEW_PRELUDE: &str = include_str!("../src/app_impl/startup_new_prelude.rs");
const SELECTION_FALLBACK: &str = include_str!("../src/app_impl/selection_fallback.rs");
const SUBMIT_DIAGNOSTICS: &str = include_str!("../src/app_impl/submit_diagnostics.rs");
const FILTERING_CACHE: &str = include_str!("../src/app_impl/filtering_cache.rs");
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

fn source_function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing function signature: {signature}"));
    let after_signature = &source[start..];
    let open = after_signature
        .find('{')
        .unwrap_or_else(|| panic!("missing function body open: {signature}"));
    let body_start = start + open;
    let mut depth = 0usize;
    for (offset, ch) in source[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[body_start..=body_start + offset];
                }
            }
            _ => {}
        }
    }
    panic!("missing function body close: {signature}");
}

/// The braced body of the first `pattern => { ... }` match arm.
fn match_arm_body<'a>(source: &'a str, pattern: &str) -> &'a str {
    let start = source
        .find(pattern)
        .unwrap_or_else(|| panic!("missing match arm: {pattern}"));
    let arrow = source[start..]
        .find("=> {")
        .unwrap_or_else(|| panic!("missing match arm body: {pattern}"));
    source_function_body(&source[start + arrow..], "=> {")
}

fn source_window<'a>(source: &'a str, start: &str, len: usize) -> &'a str {
    let start_idx = source
        .find(start)
        .unwrap_or_else(|| panic!("missing source start: {start}"));
    let end_idx = (start_idx + len).min(source.len());
    &source[start_idx..end_idx]
}

#[test]
fn brain_inbox_enter_opens_chat_with_the_item_waiting_without_submit_or_resolve() {
    // Brain Inbox launcher rows are observations that require an explicit
    // user decision. Plain Enter opens the chat with the item waiting in the
    // line above the input, but must not submit a turn, fill the input, or
    // destructively resolve the inbox row.
    let execute_body = source_function_body(
        SELECTION_FALLBACK,
        "pub(crate) fn execute_root_brain_inbox_open",
    );
    assert!(
        execute_body.contains("self.one_window_open_chat(")
            && execute_body.contains("ChatEntry::quick()")
            && execute_body.contains(".with(Self::brain_inbox_context_part(&item))"),
        "Brain Inbox Enter should build context and open the chat with it waiting in the line"
    );
    assert!(
        !execute_body.contains("ChatEntry::sending(")
            && !execute_body.contains("ChatEntry::seeded(")
            && !execute_body.contains("resolve_root_brain_inbox_item")
            && !execute_body.contains("execute_root_note_open")
            && !execute_body.contains("resume_agent_chat_conversation_with_followup"),
        "Brain Inbox Enter must not send, fill the input, resolve, open notes directly, or use the auto-submit follow-up route"
    );

    let context_part_body = source_function_body(SELECTION_FALLBACK, "fn brain_inbox_context_part");
    assert!(
        context_part_body.contains("AiContextPart::TextBlock")
            && context_part_body
                .contains("text: crate::brain::response_prompt_for_inbox_item(item)")
            && context_part_body.contains("mime_type: Some(\"text/markdown\".to_string())"),
        "The full Brain Inbox prompt should be staged as a markdown TextBlock context part"
    );
}

#[test]
fn mini_prompt_enter_records_submit_owner_before_resetting_to_script_list() {
    let submit_fn = source_function_body(ARG_HELPERS, "fn submit_arg_prompt_from_current_state(");

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
            && SUBMIT_DIAGNOSTICS
                .contains("self.record_submit_diagnostic(owner, route, None, value, true);")
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
        let press_enter = match_arm_body(source, "InputEvent::PressEnter");
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
        source_function_body(SELECTION_FALLBACK, "pub(crate) fn execute_selected(");
    assert!(
        execute_selected
            .contains("should_consume_script_list_enter_after_submit(\"execute_selected\")")
            && execute_selected
                .contains("record_submit_diagnostic(\n                \"launcher\",")
            && execute_selected.find("should_consume_script_list_enter_after_submit")
                < execute_selected.find("execute_menu_syntax_command_invocation"),
        "execute_selected must be guarded and recorded before any ScriptList execution sink"
    );
    assert!(
        execute_selected.find("should_consume_script_list_enter_after_submit")
            < execute_selected.find("self.run_selected_row(observation, cx)"),
        "grouped rows, fallbacks included, must run only behind the execute_selected Enter echo guard"
    );
}

#[test]
fn script_list_submit_requires_live_filter_and_grouped_cache_domain() {
    let helper = source_function_body(
        SELECTION_FALLBACK,
        "pub(crate) fn live_script_list_flat_selection_for_submit",
    );
    assert!(
        helper.contains("self.resolved_main_menu_selected_subject()"),
        "ScriptList submit guard must resolve rows through the current-query resolver"
    );
    assert!(
        FILTERING_CACHE.contains("source-filters={grouped_source_filter_key}")
            && FILTERING_CACHE.contains("browser-tabs-gen={browser_tabs_generation}")
            && FILTERING_CACHE.contains("browser-history-gen={browser_history_generation}"),
        "Grouped cache domains may include source/current-app/generation fields, so submit guards must not require exact filter-only keys"
    );
    assert!(
        helper.contains("MainMenuRowSubject::SearchResult { flat_index }"),
        "ScriptList submit must resolve through the grouped visible-row cache"
    );
    assert!(
        helper.contains("main_list_submit_blocked.no_live_selected_row"),
        "No visible/live selected row must produce an explicit submit diagnostic"
    );
}

#[test]
fn execute_selected_uses_live_selection_guard_before_row_execution() {
    let run_row = source_function_body(SELECTION_FALLBACK, "pub(crate) fn run_selected_row(");
    assert!(
        run_row.contains("live_script_list_flat_selection_for_submit()"),
        "execute_selected must route visible-row submit through the live selection guard"
    );
    assert!(
        run_row.contains("ResolvedMainMenuSelection::Calculator { row, result }")
            && run_row.find("self.resolved_main_menu_selected_subject()")
                < run_row.find("one_window_open_calculator_answer"),
        "calculator rows must resolve through the same current-query resolver before running"
    );
    let guard_pos = run_row
        .find("live_script_list_flat_selection_for_submit()")
        .expect("run_selected_row must call live selection guard");
    let after_guard = &run_row[guard_pos..];
    assert!(
        after_guard.find("live_script_list_flat_selection_for_submit()")
            < after_guard.find("mark_opened_from_main_menu"),
        "live selection guard must run before any launcher row dispatch"
    );
}

#[test]
fn selected_main_list_search_result_owned_uses_live_selection_guard() {
    let selected_owned = source_function_body(
        SELECTION_FALLBACK,
        "pub(crate) fn selected_main_list_search_result_owned",
    );
    assert!(
        selected_owned.contains("self.resolved_main_menu_selected_subject()")
            && selected_owned.contains("row.eligibility.activatable"),
        "all owned ScriptList result lookups must share the live selection guard"
    );
}

#[test]
fn fallback_enter_cannot_execute_legacy_cached_fallback_without_grouped_row() {
    let run_row = source_function_body(SELECTION_FALLBACK, "pub(crate) fn run_selected_row(");
    let guard_pos = run_row
        .find("live_script_list_flat_selection_for_submit()")
        .expect("run_selected_row must call live selection guard");
    assert!(
        run_row[guard_pos..].contains("scripts::SearchResult::Fallback(fallback_match) =>")
            && run_row[guard_pos..]
                .contains("self.execute_fallback_item(&fallback_match.fallback, cx)"),
        "fallback Enter must execute only a live grouped fallback row"
    );
    assert!(
        !SELECTION_FALLBACK.contains("main_menu_fallback_state.selected_item().cloned()")
            && !SELECTION_FALLBACK.contains("execute_selected_fallback"),
        "fallback Enter must not execute legacy cached fallback state outside the visible grouped row domain"
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
    let submit_current_value =
        source_function_body(PROMPT_HANDLER, "fn submit_current_value(&mut self");
    assert!(
        submit_current_value.contains("self.submit_main_menu_selected_subject(cx)?")
            && submit_current_value.contains("self.submit_arg_prompt_from_current_state(&id, cx)")
            && source_function_body(ARG_HELPERS, "fn submit_arg_prompt_from_current_state(")
                .contains("record_submit_diagnostic"),
        "protocol-driven submit_current_value must record submit diagnostics"
    );
    assert!(
        PROMPT_HANDLER.contains("record_submit_diagnostic(\n                                            \"protocol\",\n                                            \"forceSubmit\""),
        "protocol forceSubmit must record submit diagnostics"
    );
    assert!(
        DEVTOOLS_FOCUS.contains("submitDiagnostics: diagnostic(state.submitDiagnostics ?? null)")
            && DEVTOOLS_ACT.contains("focus.inspect.submitDiagnostics")
            && DEVTOOLS_ACT.contains("isPromptEntityTargetReceipt")
            && DEVTOOLS_ACT.contains("sourceAfter.submitDiagnostics")
            && DEVTOOLS_ACT.contains("submitDiagnostics: diagnostic({\n      before: before.submitDiagnostics ?? null,\n      after: after.submitDiagnostics ?? null,"),
        "DevTools act/focus receipts must expose before/after submit diagnostics"
    );
}

/// WHY: an empty `@source:` colon mode (`@clipboard:` showing recents) must
/// never let a reflexive double-Enter attach a recent item the user never
/// chose. The old mechanism was a selectable "Type to search" guard row; the
/// current one renders recents UNARMED (no selected row, hint in ghost text +
/// section header) until an explicit Down/click. This locks the unarmed
/// mechanism: no fake guard row may return, Enter must be consumed while
/// suppressed, and Down must be the arming gesture.
#[test]
fn empty_spine_subsearch_prefixes_are_not_submit_armed() {
    const AGENT_HANDOFF: &str = include_str!("../src/app_impl/agent_handoff/mod.rs");
    const IMPL_MOVEMENT: &str = include_str!("../src/app_navigation/impl_movement.rs");

    let rich_classifier = source_window(FILTERING_CACHE, "fn active_rich_spine_subsearch(", 900);
    assert!(
        rich_classifier.contains("query.trim().to_string()"),
        "empty @file:/@clipboard:/@history: prefixes must keep using the rich path so unarmed recents render"
    );

    // The hint affordance must never be a selectable list row again: no
    // guard-row id and no consuming placeholder action anywhere in the
    // grouped-results builder.
    assert!(
        !FILTERING_CACHE.contains("empty-guard")
            && !FILTERING_CACHE.contains("AwaitContextSubsearchInput"),
        "the empty colon-mode hint must ride ghost text + the section header, not a selectable first row"
    );

    let spine_enter = source_window(AGENT_HANDOFF, "fn try_handle_spine_enter(", 1400);
    assert!(
        spine_enter.contains("spine_empty_subsearch_selection_suppressed()")
            && spine_enter.contains("empty_context_subsearch_enter_consumed"),
        "Enter in an unarmed empty colon mode must be consumed before any row acceptance or plan submission"
    );

    let down = source_function_body(IMPL_MOVEMENT, "fn move_selection_down(");
    let keyboard_select = source_function_body(IMPL_MOVEMENT, "fn set_selected_index(");
    assert!(
        down.contains("self.move_selection_by(1, cx)")
            && keyboard_select.contains("MainMenuSelectionOrigin::Keyboard"),
        "Down must be the explicit arming gesture for the unarmed empty colon mode"
    );
}
