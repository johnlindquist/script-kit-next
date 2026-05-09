use std::fs;

#[test]
fn capture_composer_suppresses_main_list_before_capture_grouping() {
    let source = fs::read_to_string("src/app_impl/filtering_cache.rs")
        .expect("Failed to read src/app_impl/filtering_cache.rs");

    let ownership_gate = source
        .find("capture_composer_owns_input_for(raw_filter_text)")
        .expect("filtering cache must gate capture composer ownership");
    let capture_grouping = source
        .find("capture_for(raw_filter_text)")
        .expect("filtering cache must keep capture grouping for non-composer paths");

    assert!(
        ownership_gate < capture_grouping,
        "capture composer ownership must blank the main list before capture handler grouping can run"
    );
    assert!(
        source.contains("capture-handler rows"),
        "the suppression comment should document hidden capture-handler rows"
    );
}

#[test]
fn capture_composer_renders_hint_surface_instead_of_empty_state() {
    let source = fs::read_to_string("src/render_script_list/mod.rs")
        .expect("Failed to read src/render_script_list/mod.rs");

    assert!(
        source.contains("let menu_syntax_owns_main_list =")
            && source.contains("capture_composer_owns_input_for(&filter_text_for_render)")
            && source.contains("if menu_syntax_owns_main_list")
            && source.contains("menu_syntax_main_hint_snapshot(&filter_text_for_render, false)")
            && source.contains("render_menu_syntax_main_hint"),
        "menu syntax ownership should render the read-only grammar hint surface, not No results messaging"
    );

    let ownership_branch = source
        .find("if menu_syntax_owns_main_list")
        .expect("render should branch on menu syntax ownership");
    let empty_state_branch = source
        .find("else if item_count == 0")
        .expect("render should keep the normal empty state for ordinary searches");
    assert!(
        ownership_branch < empty_state_branch,
        "menu syntax ownership must hide stale grouped rows before item_count decides which list to render"
    );
}

#[test]
fn live_menu_syntax_ownership_bypasses_debounced_grouped_cache() {
    let source = fs::read_to_string("src/app_impl/filtering_cache.rs")
        .expect("Failed to read src/app_impl/filtering_cache.rs");

    let live_gate = source
        .find("let live_menu_syntax_owns_main_list =")
        .expect("grouped cache must check live menu syntax ownership");
    let cache_hit = source
        .find(".has_grouped_results_for(&self.computed_filter_text)")
        .expect("grouped cache should still key ordinary rows by computed filter text");

    assert!(
        live_gate < cache_hit,
        "live menu syntax ownership must bypass stale computed-filter cache hits"
    );
    assert!(
        source.contains("live_filter_text != computed_filter_text")
            && source.contains("Arc::<[GroupedListItem]>::from(Vec::new())"),
        "the live ownership gate should return a transient empty result without storing stale rows"
    );
    assert!(
        source.contains("pub(crate) fn get_filtered_results_cached")
            && source.contains(".store_filtered_results(self.filter_text.clone(), Vec::new())"),
        "mutable filtered-cache refreshes used by stdin setFilter must not repopulate fuzzy rows while menu syntax owns input"
    );
}

#[test]
fn refine_popup_does_not_blank_launcher_results() {
    let popup = fs::read_to_string("src/app_impl/menu_syntax_trigger_popup.rs")
        .expect("Failed to read src/app_impl/menu_syntax_trigger_popup.rs");
    let filtering_cache = fs::read_to_string("src/app_impl/filtering_cache.rs")
        .expect("Failed to read src/app_impl/filtering_cache.rs");
    let render = fs::read_to_string("src/render_script_list/mod.rs")
        .expect("Failed to read src/render_script_list/mod.rs");

    assert!(
        popup.contains("fn owns_main_list(&self) -> bool")
            && popup.contains("TriggerPickerMode::Capture | TriggerPickerMode::Command"),
        "only capture and command trigger popups should blank the main launcher list"
    );
    assert!(
        filtering_cache.contains("self.menu_syntax_trigger_popup_state.owns_main_list()")
            && render.contains("self.menu_syntax_trigger_popup_state.owns_main_list()"),
        "refine (`:`) popup snapshots should not suppress structured search results"
    );
    assert!(
        filtering_cache.contains("free_text_for_search(&self.menu_syntax_mode, filter_text)")
            && filtering_cache.contains("apply_advanced_query(results, query)"),
        "automation state counts should use the same advanced-query search text and predicates as rendered grouping"
    );
}

#[test]
fn capture_target_picker_closes_when_body_composition_starts() {
    let trigger_picker = fs::read_to_string("src/menu_syntax/trigger_picker.rs")
        .expect("Failed to read src/menu_syntax/trigger_picker.rs");
    let keys = fs::read_to_string("src/menu_syntax/trigger_picker_keys.rs")
        .expect("Failed to read src/menu_syntax/trigger_picker_keys.rs");

    assert!(
        trigger_picker
            .contains("capture_body_boundary_has_started_with_targets(input, &capture_targets)")
            && trigger_picker.contains("return None;"),
        "target picker should stop producing snapshots after ;target body composition starts"
    );
    assert!(
        trigger_picker.contains("keep_open: false"),
        "capture target InsertToken should commit ;target and close the picker"
    );
    assert!(
        !trigger_picker.contains("rows.extend(capture_handler_rows"),
        "capture handlers must not render inside the target picker"
    );
    assert!(
        keys.contains("keep_open: *keep_open"),
        "open-value qualifier rows should be able to keep the popup open on Accept or Apply"
    );
}

#[test]
fn popup_highlights_use_live_filter_snapshot_text() {
    let filter_input_change = fs::read_to_string("src/app_impl/filter_input_change.rs")
        .expect("Failed to read src/app_impl/filter_input_change.rs");
    let popup_window = fs::read_to_string("src/app_impl/menu_syntax_trigger_popup_window.rs")
        .expect("Failed to read src/app_impl/menu_syntax_trigger_popup_window.rs");

    assert!(
        filter_input_change.contains("sync_menu_syntax_trigger_popup_window_for_filter")
            && filter_input_change.contains("new_text.clone()"),
        "filter changes should pass the just-typed text into the popup snapshot"
    );
    assert!(
        popup_window.contains("raw_filter_text,")
            && popup_window.contains(
                "trigger_popup_row_highlight_indices(row, &self.snapshot.raw_filter_text)"
            ),
        "popup rows should render highlights from the snapshot raw filter"
    );
}

#[test]
fn popup_footer_rows_and_visible_page_follow_inline_picker_contracts() {
    let popup_window = fs::read_to_string("src/app_impl/menu_syntax_trigger_popup_window.rs")
        .expect("Failed to read src/app_impl/menu_syntax_trigger_popup_window.rs");

    assert!(
        popup_window.contains("fn set_snapshot(&mut self, mut snapshot:")
            && popup_window.contains("snapshot.visible_start = self.visible_range().start;")
            && popup_window.contains("visible_start: self.menu_syntax_trigger_popup_state.visible_start")
            && popup_window.contains("trigger_popup_visible_start_for_selection(")
            && popup_window.contains("inline_dropdown_visible_range_from_start("),
        "menu-syntax popup updates should preserve the current visible page through the shared inline-dropdown range helper"
    );
    assert!(
        !popup_window.contains("|| is_footer")
            && !popup_window.contains("let is_footer = matches!(row.kind"),
        "enabled footer action rows should use the same click-to-accept path as other menu-syntax popup rows"
    );
    assert!(
        popup_window.contains("CONTEXT_PICKER_SYNOPSIS_HEIGHT")
            && popup_window.contains("selected_row_has_synopsis")
            && popup_window.contains("row.detail.is_some() || row.example.is_some()"),
        "menu-syntax popup window height should reserve shared inline-dropdown synopsis space when the selected row renders synopsis"
    );
    assert!(
        popup_window.contains("MENU_SYNTAX_TRIGGER_POPUP_AUTOMATION_ID")
            && popup_window.contains("register_attached_popup(")
            && popup_window.contains("menuSyntaxTriggerPopup")
            && popup_window.contains("set_automation_bounds(")
            && popup_window.contains("remove_automation_window(MENU_SYNTAX_TRIGGER_POPUP_AUTOMATION_ID)"),
        "menu-syntax popup should register as an attached automation popup so screenshots and layout probes can target it directly"
    );
}

#[test]
fn stdin_setfilter_runs_menu_syntax_popup_state_machine() {
    let updates = fs::read_to_string("src/app_impl/filter_input_updates.rs")
        .expect("Failed to read src/app_impl/filter_input_updates.rs");
    let popup_window = fs::read_to_string("src/app_impl/menu_syntax_trigger_popup_window.rs")
        .expect("Failed to read src/app_impl/menu_syntax_trigger_popup_window.rs");

    assert!(
        updates.contains("self.run_menu_syntax_trigger_popup_state_machine(&text, window, cx);")
            && updates.contains("self.invalidate_grouped_cache();"),
        "programmatic setFilter must run the same menu-syntax popup state machine as real typing and invalidate stale grouped rows"
    );
    assert!(
        popup_window.contains("pub(crate) fn run_menu_syntax_trigger_popup_state_machine"),
        "the popup state-machine helper should be shared by keyboard input and setFilter"
    );
}

#[test]
fn launcher_input_accents_power_syntax_prefixes() {
    let render = fs::read_to_string("src/render_script_list/mod.rs")
        .expect("Failed to read src/render_script_list/mod.rs");
    let input_state = fs::read_to_string("vendor/gpui-component/crates/ui/src/input/state.rs")
        .expect("Failed to read vendored input state");
    let input_element = fs::read_to_string("vendor/gpui-component/crates/ui/src/input/element.rs")
        .expect("Failed to read vendored input element");

    assert!(
        render.contains("input_spans_for_input_with_targets(")
            && render.contains("state.set_highlight_ranges(input_highlight_ranges)"),
        "ScriptList input should accent the parsed power-syntax prefix span"
    );
    assert!(
        input_state.contains("highlight_ranges: Vec<(Range<usize>, Hsla)>")
            && input_state.contains("pub fn set_highlight_ranges"),
        "the input component should expose plain-text highlight ranges"
    );
    assert!(
        input_element.contains("fn custom_highlight_styles")
            && input_element.contains("state.highlight_ranges"),
        "plain input highlight ranges should be converted into text runs"
    );
}

#[test]
fn trailing_space_alias_execution_is_disabled_for_menu_syntax() {
    let source = fs::read_to_string("src/app_impl/filter_input_change.rs")
        .expect("Failed to read src/app_impl/filter_input_change.rs");

    assert!(
        source.contains(
            "!self.menu_syntax_mode.is_menu_syntax_for(&new_text) && new_text.ends_with(' ')"
        ),
        "alias auto-run on trailing space must not fire while menu syntax owns input"
    );
}

#[test]
fn enter_in_capture_composer_executes_capture_not_main_selection() {
    let source = fs::read_to_string("src/app_impl/selection_fallback.rs")
        .expect("Failed to read src/app_impl/selection_fallback.rs");

    let capture_route = source
        .find("self.menu_syntax_mode.capture_for(&self.filter_text)")
        .expect("execute_selected must first check for capture composer input");
    let grouped_results = source
        .find("self.get_grouped_results_cached()")
        .expect("execute_selected still resolves main-list rows for normal search");

    assert!(
        capture_route < grouped_results,
        "Enter in capture composer must route to capture execution before main-list selection"
    );
    assert!(
        source.contains("rank_scripts_handling_capture")
            && source.contains("execute_menu_syntax_capture_script"),
        "capture composer Enter should execute the ranked capture handler"
    );
}

#[test]
fn command_invocation_suppresses_main_list_and_routes_without_shell() {
    let filtering_cache = fs::read_to_string("src/app_impl/filtering_cache.rs")
        .expect("Failed to read src/app_impl/filtering_cache.rs");
    let selection = fs::read_to_string("src/app_impl/selection_fallback.rs")
        .expect("Failed to read src/app_impl/selection_fallback.rs");
    let execution = fs::read_to_string("src/app_execute/menu_syntax_execution.rs")
        .expect("Failed to read src/app_execute/menu_syntax_execution.rs");

    assert!(
        filtering_cache.contains("command_owns_input_for(filter_text)")
            && filtering_cache.contains("command_owns_input_for(raw_filter_text)"),
        "`!` command composition must blank normal launcher results"
    );

    let command_route = selection
        .find("self.menu_syntax_mode.command_for(&self.filter_text)")
        .expect("execute_selected must first check command invocation input");
    let grouped_results = selection
        .find("self.get_grouped_results_cached()")
        .expect("execute_selected still resolves main-list rows for normal search");
    assert!(
        command_route < grouped_results,
        "Enter in ! command mode must route before main-list selection"
    );

    assert!(
        execution.contains("execute_menu_syntax_command_invocation")
            && execution.contains("script_command_head")
            && execution.contains("scriptlet_command_head")
            && execution.contains("execute_interactive_with_env_and_args")
            && execution.contains("execute_scriptlet_with_env_and_args")
            && execution.contains("menu_syntax_command_ambiguous")
            && !execution.contains("Command::new(&invocation.head)")
            && !execution.contains("Command::new(invocation.head"),
        "`!` must resolve registered Script Kit commands, pass env to scripts/scriptlets, reject ambiguous heads, and never spawn typed text as shell"
    );
}

#[test]
fn power_syntax_tags_and_command_picker_are_first_class() {
    let query = fs::read_to_string("src/menu_syntax/query.rs")
        .expect("Failed to read src/menu_syntax/query.rs");
    let trigger_picker = fs::read_to_string("src/menu_syntax/trigger_picker.rs")
        .expect("Failed to read src/menu_syntax/trigger_picker.rs");
    let popup = fs::read_to_string("src/app_impl/menu_syntax_trigger_popup.rs")
        .expect("Failed to read src/app_impl/menu_syntax_trigger_popup.rs");

    assert!(
        query.contains("strip_prefix('#')")
            && query.contains("Predicate::Tag")
            && query.contains("\"tag\""),
        "`:#tag` and `:tag:value` should parse as refine tag predicates"
    );
    assert!(
        trigger_picker.contains("title: \"Filter by tag\"")
            && trigger_picker.contains("insert: \":#\"")
            && trigger_picker.contains("insert: \":tag:\"")
            && trigger_picker.contains("keep_open: true"),
        "`:` popup should teach both :#tag sugar and canonical tag: filters, keeping the popup open for a tag name"
    );
    assert!(
        trigger_picker.contains("TriggerPickerMode::Command")
            && trigger_picker.contains("build_command_snapshot")
            && trigger_picker.contains("script_command_head")
            && trigger_picker.contains("scriptlet_command_head"),
        "`!` should have a real trigger-picker mode backed by registered commands"
    );
    assert!(
        trigger_picker.contains("bang_command_snapshot(input, ctx)")
            && popup.contains("trigger != ';' && trigger != ':'"),
        "command trigger rows should be handled by the trigger picker while popup partial filtering stays scoped to text-composer triggers"
    );
}

#[test]
fn registered_capture_targets_extend_parser_popup_and_input_highlight() {
    let filter_input_core = fs::read_to_string("src/app_impl/filter_input_core.rs")
        .expect("Failed to read src/app_impl/filter_input_core.rs");
    let parse = fs::read_to_string("src/menu_syntax/parse.rs")
        .expect("Failed to read src/menu_syntax/parse.rs");
    let trigger_picker = fs::read_to_string("src/menu_syntax/trigger_picker.rs")
        .expect("Failed to read src/menu_syntax/trigger_picker.rs");
    let mode = fs::read_to_string("src/menu_syntax/mode.rs")
        .expect("Failed to read src/menu_syntax/mode.rs");

    assert!(
        filter_input_core.contains("registered_capture_targets_from_scripts(&self.scripts)")
            && filter_input_core.contains("from_input_with_capture_targets"),
        "ScriptList should parse ;target with registered capture targets from metadata"
    );
    assert!(
        parse.contains("parse_with_capture_targets")
            && parse.contains("is_capture_target_registered"),
        "the parser should distinguish unknown ;text search from registered ;target capture"
    );
    assert!(
        trigger_picker.contains("registered_capture_targets(ctx)")
            && trigger_picker.contains("capture_target_catalog(ctx)"),
        "the popup should show registered capture target rows"
    );
    assert!(
        mode.contains("prefix_span_for_input_with_targets")
            && mode.contains("capture_body_boundary_has_started_with_targets"),
        "registered targets should get composer ownership and input accent spans"
    );
}

#[test]
fn menu_syntax_hint_surface_is_not_grouped_results() {
    let render = fs::read_to_string("src/render_script_list/mod.rs")
        .expect("Failed to read src/render_script_list/mod.rs");
    let filtering_cache = fs::read_to_string("src/app_impl/filtering_cache.rs")
        .expect("Failed to read src/app_impl/filtering_cache.rs");
    let main_hint = fs::read_to_string("src/menu_syntax/main_hint.rs")
        .expect("Failed to read src/menu_syntax/main_hint.rs");

    assert!(
        render.contains("fn render_menu_syntax_main_hint(")
            && render.contains("MenuSyntaxMainHintSnapshot")
            && render.contains("menu_syntax_main_hint_snapshot(&filter_text_for_render, true)"),
        "the grammar hint should be a rendered snapshot for owned states and advanced-query empty states"
    );
    assert!(
        filtering_cache.contains("Arc::<[GroupedListItem]>::from(Vec::new())")
            && !render.contains("build_menu_syntax_hint_results("),
        "grammar hints must not repopulate the selectable GroupedListItem list"
    );
    assert!(
        render.contains(".h_full()")
            && render.contains("hint.examples")
            && render.contains("advanced_query_guide_hint")
            && render.contains("MenuSyntaxMainHintKind::AdvancedQueryGuide")
            && render.contains("Plain #tag is launcher search"),
        "hint cards should fill the list area, render persistent examples, and nudge top-level #tag search toward :#tag or ; capture labels"
    );
    assert!(
        main_hint.contains("AdvancedQueryGuide")
            && main_hint.contains("Filter by tag")
            && main_hint.contains("Refine launcher search"),
        "bare/partial `:` states should get guide hints before they become zero-result structured queries"
    );
}

#[test]
fn state_result_exposes_menu_syntax_main_hint_for_agentic_tests() {
    let variants = fs::read_to_string("src/protocol/message/variants/query_ops.rs")
        .expect("Failed to read src/protocol/message/variants/query_ops.rs");
    let constructors = fs::read_to_string("src/protocol/message/constructors/query_ops.rs")
        .expect("Failed to read src/protocol/message/constructors/query_ops.rs");
    let prompt_handler = fs::read_to_string("src/prompt_handler/mod.rs")
        .expect("Failed to read src/prompt_handler/mod.rs");

    assert!(
        variants.contains("menuSyntaxMainHint") && variants.contains("MenuSyntaxMainHintSnapshot"),
        "stateResult should expose the same grammar hint snapshot the main menu renders"
    );
    assert!(
        constructors.contains("menu_syntax_main_hint: Option<crate::menu_syntax::MenuSyntaxMainHintSnapshot>")
            && constructors.contains("            menu_syntax_main_hint,"),
        "Message::state_result should forward menu_syntax_main_hint without ad hoc protocol construction"
    );
    assert!(
        prompt_handler.contains("self.menu_syntax_main_hint_snapshot(")
            && prompt_handler.contains("advanced_query_results_empty"),
        "getState should compute owned-state and structured-empty grammar hints for automation"
    );
}

#[test]
fn capture_composer_hint_explains_handler_ranking_in_state_hint_surface() {
    let main_hint = fs::read_to_string("src/menu_syntax/main_hint.rs")
        .expect("Failed to read src/menu_syntax/main_hint.rs");
    let handler_index = fs::read_to_string("src/menu_syntax/handler_index.rs")
        .expect("Failed to read src/menu_syntax/handler_index.rs");

    assert!(
        handler_index.contains("explain_capture_handler_ranking")
            && handler_index.contains("rank_handlers_for_target")
            && handler_index.contains("dedupe_ranked_handlers_by_path"),
        "handler ranking explanation should be backed by the existing ranker and execution dedupe"
    );
    assert!(
        main_hint.contains("explain_capture_handler_ranking")
            && main_hint.contains("Handler")
            && main_hint.contains("Why selected")
            && main_hint.contains("Handler conflict"),
        "capture composer hint should expose handler ranking through MenuSyntaxMainHintSnapshot rows"
    );
}
