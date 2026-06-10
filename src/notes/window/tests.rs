use super::{Note, NoteId, NotesApp};
use crate::ai::message_parts::AiContextPart;

#[test]
fn formatting_replacement_wraps_selected_text() {
    let value = "hello world";
    let selection = 6..11;

    let (replacement, new_selection) =
        NotesApp::formatting_replacement(value, selection.clone(), "**", "**");

    let new_value = format!(
        "{}{}{}",
        &value[..selection.start],
        replacement,
        &value[selection.end..]
    );

    assert_eq!(new_value, "hello **world**");
    assert_eq!(new_selection, 8..13);
}

#[test]
fn formatting_replacement_inserts_and_positions_cursor() {
    let value = "hello";
    let selection = 2..2;

    let (replacement, new_selection) =
        NotesApp::formatting_replacement(value, selection.clone(), "**", "**");

    let new_value = format!(
        "{}{}{}",
        &value[..selection.start],
        replacement,
        &value[selection.end..]
    );

    assert_eq!(new_value, "he****llo");
    assert_eq!(new_selection, 4..4);
}

#[test]
fn build_note_text_part_for_ai_prefers_selection_when_present() {
    let part = NotesApp::build_note_text_part_for_ai("Demo Note", "demo-id", "hello world", 6..11)
        .expect("Expected selected note part");

    assert_eq!(
        part,
        AiContextPart::TextBlock {
            label: "Selected Text".to_string(),
            source: "notes://demo-id#selection=6-11".to_string(),
            text: "world".to_string(),
            mime_type: Some("text/markdown".to_string()),
        }
    );
}

#[test]
fn build_note_text_part_for_ai_falls_back_to_full_note_when_selection_is_empty() {
    let part = NotesApp::build_note_text_part_for_ai("Demo Note", "demo-id", "hello world", 5..5)
        .expect("Expected full note part");

    assert_eq!(
        part,
        AiContextPart::TextBlock {
            label: "Demo Note".to_string(),
            source: "notes://demo-id".to_string(),
            text: "hello world".to_string(),
            mime_type: Some("text/markdown".to_string()),
        }
    );
}

#[test]
fn test_format_search_match_counter_uses_selected_position_when_available() {
    let counter = NotesApp::format_search_match_counter(Some((3, 8)), 8);
    assert_eq!(counter, "3/8");
}

#[test]
fn test_format_search_match_counter_uses_zero_when_selection_missing() {
    let counter = NotesApp::format_search_match_counter(None, 6);
    assert_eq!(counter, "0/6");
}

#[test]
fn test_resolve_selected_note_returns_none_when_selection_is_missing() {
    let notes = vec![Note::with_content("one"), Note::with_content("two")];

    let selected = NotesApp::resolve_selected_note(None, &notes);

    assert!(selected.is_none());
}

#[test]
fn test_resolve_selected_note_returns_none_when_selection_is_stale() {
    let notes = vec![Note::with_content("one"), Note::with_content("two")];

    let selected = NotesApp::resolve_selected_note(Some(NoteId::new()), &notes);

    assert!(selected.is_none());
}

#[test]
fn test_resolve_selected_note_returns_note_when_selection_exists() {
    let notes = vec![Note::with_content("one"), Note::with_content("two")];
    let selected_id = notes[1].id;

    let selected = NotesApp::resolve_selected_note(Some(selected_id), &notes);

    assert_eq!(
        selected.map(|(id, note)| (id, note.id)),
        Some((selected_id, selected_id))
    );
}

#[test]
fn test_cmd_f_dispatches_search_on_window_when_notes_shortcut_runs() {
    const KEYBOARD_SOURCE: &str = include_str!("keyboard.rs");
    assert!(
        KEYBOARD_SOURCE.contains("window.dispatch_action(Box::new(Search), cx);"),
        "Notes cmd+f shortcut should dispatch Search through the current window"
    );
    assert!(
        !KEYBOARD_SOURCE.contains("cx.dispatch_action(&Search);"),
        "Notes cmd+f shortcut should not dispatch Search through app context"
    );
}

#[test]
fn test_notes_keyboard_checks_ghost_acceptance_before_tab_indentation() {
    const KEYBOARD_SOURCE: &str = include_str!("keyboard.rs");
    let dialog_guard = KEYBOARD_SOURCE
        .find("if window.has_active_dialog(cx)")
        .expect("dialog guard should own first key routing");
    let indent = KEYBOARD_SOURCE
        .find("self.indent_at_cursor(window, cx)")
        .expect("plain Tab indentation should remain available");
    let ghost_accept = KEYBOARD_SOURCE[dialog_guard..indent]
        .find("NotesGhostAcceptMode::Word")
        .expect("plain Tab should try accepting one ghost word")
        + dialog_guard;

    assert!(
        dialog_guard < ghost_accept && ghost_accept < indent,
        "Notes keyboard must keep dialog routing first, then ghost acceptance, then Tab indentation fallback"
    );
}

#[test]
fn test_notes_keyboard_escape_dismisses_ghost_after_higher_priority_surfaces() {
    const KEYBOARD_SOURCE: &str = include_str!("keyboard.rs");
    let handle_key_down = KEYBOARD_SOURCE
        .find("pub(super) fn handle_key_down(")
        .expect("live keyboard handler should exist");
    let live_keyboard = &KEYBOARD_SOURCE[handle_key_down..];
    let dialog_guard = live_keyboard
        .find("if window.has_active_dialog(cx)")
        .expect("dialog guard should own first key routing");
    let command_bar = live_keyboard
        .find("if self.command_bar.is_open()")
        .expect("command bar should own escape before editor");
    let actions_panel = live_keyboard
        .find("if self.show_actions_panel && self.actions_panel.is_some()")
        .expect("actions panel should own escape before editor");
    let note_switcher = live_keyboard
        .find("if self.note_switcher.is_open()")
        .expect("note switcher should own escape before editor");
    let agent_chat_surface = live_keyboard[note_switcher..]
        .find("if self.surface_mode == NotesSurfaceMode::AgentChat")
        .expect("Agent Chat surface should own escape before editor")
        + note_switcher;
    let editor_escape = live_keyboard
        .find("if is_key_escape(key) {\n            cx.stop_propagation();")
        .expect("editor escape branch should exist");
    let ghost_dismiss = live_keyboard[editor_escape..]
        .find("if self.dismiss_notes_ghost(cx) {")
        .expect("editor escape should dismiss ghost before fallback closing behavior")
        + editor_escape;
    let search_close = live_keyboard[ghost_dismiss..]
        .find("if self.show_search {")
        .expect("escape fallback should still close search")
        + ghost_dismiss;

    assert!(
        dialog_guard < command_bar
            && command_bar < actions_panel
            && actions_panel < note_switcher
            && note_switcher < agent_chat_surface
            && agent_chat_surface < editor_escape
            && editor_escape < ghost_dismiss
            && ghost_dismiss < search_close,
        "Notes ghost escape must run only inside the editor escape branch after higher-priority surfaces and before editor fallback closing"
    );
}

#[test]
fn test_notes_keyboard_backtick_accepts_full_ghost_without_swallowing_plain_backtick() {
    const KEYBOARD_SOURCE: &str = include_str!("keyboard.rs");
    let backtick_branch = KEYBOARD_SOURCE
        .find("if is_key_backtick(key) && !modifiers.platform && !modifiers.control && !modifiers.alt")
        .expect("plain backtick should get a ghost-acceptance branch");
    let full_accept = KEYBOARD_SOURCE[backtick_branch..]
        .find("NotesGhostAcceptMode::Full")
        .expect("backtick branch should accept the full ghost suffix")
        + backtick_branch;
    let stop_propagation = KEYBOARD_SOURCE[full_accept..]
        .find("cx.stop_propagation();")
        .expect("handled ghost backtick should stop propagation")
        + full_accept;
    let propagate = KEYBOARD_SOURCE[stop_propagation..]
        .find("cx.propagate();")
        .expect("unhandled ghost backtick should propagate to normal text input")
        + stop_propagation;
    let tab_branch = KEYBOARD_SOURCE[propagate..]
        .find("if is_key_tab(key) && !modifiers.platform && !modifiers.control && !modifiers.alt")
        .expect("plain Tab branch should remain after backtick");
    let tab_branch = tab_branch + propagate;

    assert!(
        backtick_branch < full_accept
            && full_accept < stop_propagation
            && stop_propagation < propagate
            && propagate < tab_branch,
        "Backtick should stop propagation only when the full ghost suffix was accepted, otherwise normal text input can continue"
    );
}

#[test]
fn test_notes_render_wraps_editor_with_ghost_overlay() {
    const RENDER_BODY: &str = include_str!("render_editor_body.rs");
    for needle in [
        "notes_ghost_prediction",
        "render_notes_ghost_overlay",
        ".relative()",
        "Input::new(&self.editor_state)",
        "\"notes-ghost-autocomplete\"",
    ] {
        assert!(
            RENDER_BODY.contains(needle),
            "Notes editor render path must expose ghost overlay hook: {needle}"
        );
    }
}

#[test]
fn test_notes_automation_set_input_places_cursor_at_end_for_ghost_prefix() {
    const INIT_SOURCE: &str = include_str!("init.rs");
    let set_value = INIT_SOURCE
        .find("state.set_value(text.clone(), window, inner_cx);")
        .expect("automation setter should write the requested text");
    let set_selection = INIT_SOURCE
        .find("state.set_selection(text.len(), text.len(), window, inner_cx);")
        .expect("automation setter should park cursor at the end for editor-like input");

    assert!(
        set_value < set_selection,
        "Notes automation set-input must write text before moving the cursor to the end"
    );
}

#[test]
fn test_find_in_note_action_dispatches_search_on_window_when_action_executes() {
    const PANELS_SOURCE: &str = include_str!("panels.rs");
    assert!(
        PANELS_SOURCE.contains("window.dispatch_action(Box::new(Search), cx);"),
        "Notes Find in Note action should dispatch Search through the current window"
    );
    assert!(
        !PANELS_SOURCE.contains("cx.dispatch_action(&Search);"),
        "Notes Find in Note action should not dispatch Search through app context"
    );
}

#[test]
fn test_platform_arrow_shortcuts_only_run_note_navigation_when_editor_not_focused() {
    const KEYBOARD_SOURCE: &str = include_str!("keyboard.rs");
    assert!(
        KEYBOARD_SOURCE.contains("focus_handle(cx)")
            && KEYBOARD_SOURCE.contains(".is_focused(window);"),
        "Platform arrow shortcuts must check editor focus before note navigation"
    );
    assert!(
        KEYBOARD_SOURCE.contains("if !editor_is_focused {"),
        "Platform arrow shortcuts must skip note navigation when editor is focused"
    );
}

#[test]
fn test_show_selected_note_missing_feedback_notifies_after_feedback_state_update() {
    const NOTES_ACTIONS_SOURCE: &str = include_str!("notes_actions.rs");
    assert!(
        NOTES_ACTIONS_SOURCE.contains(
            "self.show_action_feedback(Self::SELECTED_NOTE_NOT_FOUND_FEEDBACK, true);\n        cx.notify();"
        ),
        "Missing-note feedback should notify after updating action feedback state"
    );
}

#[test]
fn test_duplicate_selected_note_sets_feedback_before_select_note() {
    const NOTES_ACTIONS_SOURCE: &str = include_str!("notes_actions.rs");
    let feedback_idx = NOTES_ACTIONS_SOURCE
        .find("self.show_action_feedback(\"Duplicated\", false);")
        .expect("Expected duplicate feedback call in notes_actions.rs");
    let select_idx = NOTES_ACTIONS_SOURCE
        .find("self.select_note(duplicate.id, window, cx);")
        .expect("Expected duplicate select_note call in notes_actions.rs");

    assert!(
        feedback_idx < select_idx,
        "Duplicate feedback should be set before select_note triggers notify"
    );
}

#[test]
fn test_copy_as_markdown_notifies_after_feedback_state_update() {
    const CLIPBOARD_OPS_SOURCE: &str = include_str!("clipboard_ops.rs");
    assert!(
        CLIPBOARD_OPS_SOURCE
            .contains("self.show_action_feedback(\"Copied\", false);\n        cx.notify();"),
        "Copy-as-markdown should notify after updating action feedback state"
    );
}

#[test]
fn test_notes_editor_disables_dynamic_code_editor_bottom_margin() {
    const INIT_SOURCE: &str = include_str!("init.rs");
    assert!(
        INIT_SOURCE.contains(
            ".code_editor(\"markdown\")\n                .code_editor_dynamic_bottom_margin(false)"
        ),
        "Notes editor should not reserve a large code-editor bottom scroll margin after trailing lines are deleted"
    );
}

#[test]
fn test_notes_keyboard_handles_named_bracket_keys_when_platform_navigation_shortcuts_run() {
    const KEYBOARD_SOURCE: &str = include_str!("keyboard.rs");
    assert!(
        KEYBOARD_SOURCE.contains("fn is_key_left_bracket(key: &str) -> bool"),
        "Notes keyboard should define a left bracket key helper"
    );
    assert!(
        KEYBOARD_SOURCE.contains("key == \"[\" || key.eq_ignore_ascii_case(\"bracketleft\")"),
        "Left bracket helper should match '[' and 'bracketleft'"
    );
    assert!(
        KEYBOARD_SOURCE.contains("fn is_key_right_bracket(key: &str) -> bool"),
        "Notes keyboard should define a right bracket key helper"
    );
    assert!(
        KEYBOARD_SOURCE.contains("key == \"]\" || key.eq_ignore_ascii_case(\"bracketright\")"),
        "Right bracket helper should match ']' and 'bracketright'"
    );
    assert!(
        KEYBOARD_SOURCE.contains("key if is_key_left_bracket(key) => {"),
        "Notes keyboard should use left bracket helper for navigate_back"
    );
    assert!(
        KEYBOARD_SOURCE.contains("key if is_key_right_bracket(key) => {"),
        "Notes keyboard should use right bracket helper for navigate_forward"
    );
}

#[test]
fn test_notes_keyboard_stops_propagation_when_escape_closes_actions_panel() {
    const KEYBOARD_SOURCE: &str = include_str!("keyboard.rs");
    let escape_branch =
        "if is_key_escape(key) || (modifiers.platform && key.eq_ignore_ascii_case(\"k\")) {";
    let branch_start = KEYBOARD_SOURCE
        .find(escape_branch)
        .expect("Expected actions panel escape branch in keyboard.rs");
    let branch_slice =
        &KEYBOARD_SOURCE[branch_start..(branch_start + 256).min(KEYBOARD_SOURCE.len())];

    let close_idx = branch_slice
        .find("self.close_actions_panel(window, cx);")
        .expect("Expected close_actions_panel call in actions panel escape branch");
    let stop_idx = branch_slice
        .find("cx.stop_propagation();")
        .expect("Expected cx.stop_propagation call in actions panel escape branch");
    let return_idx = branch_slice
        .find("return;")
        .expect("Expected return in actions panel escape branch");

    assert!(
        close_idx < stop_idx && stop_idx < return_idx,
        "Actions panel escape branch should stop propagation before returning"
    );
}

#[test]
fn test_notes_keyboard_stops_propagation_for_cmd_k_actions_toggle() {
    const KEYBOARD_SOURCE: &str = include_str!("keyboard.rs");
    let branch = "key if key.eq_ignore_ascii_case(\"k\") => {";
    let branch_start = KEYBOARD_SOURCE
        .find(branch)
        .expect("Expected cmd+k branch in keyboard.rs");
    let branch_slice =
        &KEYBOARD_SOURCE[branch_start..(branch_start + 512).min(KEYBOARD_SOURCE.len())];

    let open_idx = branch_slice
        .find("self.open_actions_panel(window, cx);")
        .expect("Expected open_actions_panel call in cmd+k branch");
    let stop_idx = branch_slice
        .find("cx.stop_propagation();")
        .expect("Expected cx.stop_propagation call in cmd+k branch");

    assert!(
        open_idx < stop_idx,
        "Cmd+K branch should stop propagation after toggling the actions panel"
    );
}

#[test]
fn test_notes_keyboard_stops_propagation_for_cmd_p_browse_toggle() {
    const KEYBOARD_SOURCE: &str = include_str!("keyboard.rs");
    let branch = "key if key.eq_ignore_ascii_case(\"p\") => {";
    let branch_start = KEYBOARD_SOURCE
        .find(branch)
        .expect("Expected cmd+p branch in keyboard.rs");
    let branch_slice =
        &KEYBOARD_SOURCE[branch_start..(branch_start + 640).min(KEYBOARD_SOURCE.len())];

    let open_idx = branch_slice
        .find("self.open_browse_panel(window, cx);")
        .expect("Expected open_browse_panel call in cmd+p branch");
    let stop_idx = branch_slice
        .rfind("cx.stop_propagation();")
        .expect("Expected cx.stop_propagation call in cmd+p branch");

    assert!(
        open_idx < stop_idx,
        "Cmd+P branch should stop propagation after toggling the browse panel"
    );
}

#[test]
fn test_notes_keyboard_stops_propagation_for_cmd_n_new_note() {
    const KEYBOARD_SOURCE: &str = include_str!("keyboard.rs");
    let branch = "key if key.eq_ignore_ascii_case(\"n\") => {";
    let branch_start = KEYBOARD_SOURCE
        .find(branch)
        .expect("Expected cmd+n branch in keyboard.rs");
    let branch_slice =
        &KEYBOARD_SOURCE[branch_start..(branch_start + 384).min(KEYBOARD_SOURCE.len())];

    let create_idx = branch_slice
        .find("self.create_note(window, cx);")
        .expect("Expected create_note call in cmd+n branch");
    let stop_idx = branch_slice
        .find("cx.stop_propagation();")
        .expect("Expected cx.stop_propagation call in cmd+n branch");

    assert!(
        create_idx < stop_idx,
        "Cmd+N branch should stop propagation after creating a note"
    );
}

#[test]
fn test_notes_auto_resize_height_allows_restored_height_above_default_max() {
    assert_eq!(
        NotesApp::resolve_auto_resize_height(108.0, 662.0, 600.0),
        662.0
    );
}

#[test]
fn test_notes_auto_resize_height_clamps_to_default_max_when_initial_height_is_smaller() {
    assert_eq!(
        NotesApp::resolve_auto_resize_height(900.0, 280.0, 600.0),
        600.0
    );
}

#[test]
fn test_notes_auto_resize_counts_soft_wrapped_display_lines() {
    const INIT_SOURCE: &str = include_str!("init.rs");
    assert!(
        INIT_SOURCE.contains("fn editor_display_line_count(")
            && INIT_SOURCE.contains("soft_wrapped_lines_len()")
            && !INIT_SOURCE.contains("note.content.lines().count()"),
        "Notes auto-resize must count the editor's soft-wrapped display lines, \
         not logical newline-separated lines, so wrapped paragraphs grow the window"
    );
}

#[test]
fn test_notes_keyboard_uses_cmd_shift_o_for_focused_note_mentions() {
    const KEYBOARD_SOURCE: &str = include_str!("keyboard.rs");
    assert!(
        KEYBOARD_SOURCE.contains("key if modifiers.shift && key.eq_ignore_ascii_case(\"o\") => {"),
        "Notes keyboard should reserve Cmd+Shift+O for focused note mention portal opens"
    );
    assert!(
        KEYBOARD_SOURCE.contains("self.open_focused_note_mention_portal(window, cx)"),
        "Cmd+Shift+O branch should route through the focused note mention portal helper"
    );
}

#[test]
fn test_note_switcher_selection_can_replace_active_note_mention() {
    const PANELS_SOURCE: &str = include_str!("panels.rs");
    assert!(
        PANELS_SOURCE.contains("self.replace_active_note_mention_with_note(note_id, window, cx)"),
        "Note switcher note selections should first try replacing an active note mention"
    );
}

#[test]
fn test_note_switcher_preview_includes_metadata_summary() {
    const INIT_SOURCE: &str = include_str!("init.rs");
    const PANELS_SOURCE: &str = include_str!("panels.rs");
    const NAVIGATION_SOURCE: &str = include_str!("navigation.rs");
    assert!(
        INIT_SOURCE.contains("preview: Self::note_switcher_preview(n),"),
        "Initial note switcher rows should use the metadata-aware preview helper"
    );
    assert!(
        PANELS_SOURCE.contains("preview: Self::note_switcher_preview(n),"),
        "Reopened note switcher rows should use the metadata-aware preview helper"
    );
    assert!(
        NAVIGATION_SOURCE.contains("storage::get_note_tags(note.id)")
            && NAVIGATION_SOURCE.contains("storage::get_note_aliases(note.id)")
            && NAVIGATION_SOURCE.contains("storage::get_note_backlink_count(note.id)")
            && NAVIGATION_SOURCE.contains("storage::get_note_outbound_link_count(note.id)"),
        "Notes navigation metadata paths should include tags, aliases, links, and backlinks"
    );
}

#[test]
fn test_notes_automation_state_includes_alias_metadata() {
    const NAVIGATION_SOURCE: &str = include_str!("navigation.rs");
    assert!(
        NAVIGATION_SOURCE
            .contains("\"aliases\": aliases.iter().take(8).cloned().collect::<Vec<_>>()")
            && NAVIGATION_SOURCE.contains("\"aliasCount\": aliases.len()"),
        "Notes automation selectedNote metadata should expose aliases for agent organization proof"
    );
}

#[test]
fn test_note_switcher_preview_metadata_summary_format() {
    let tags = vec![
        "planning".to_string(),
        "projects/script-kit".to_string(),
        "daily".to_string(),
        "overflow".to_string(),
    ];

    let preview = NotesApp::note_switcher_preview_from_metadata("Project body", &tags, 2, 1);

    assert_eq!(
        preview,
        "#planning · #projects/script-kit · #daily · +1 tags · 2 links · 1 backlink · Project body"
    );
}

#[test]
fn test_note_footer_preview_advertises_replace_shortcut() {
    const FOOTER_SOURCE: &str = include_str!("render_editor_footer.rs");
    const NAVIGATION_SOURCE: &str = include_str!("navigation.rs");
    assert!(
        FOOTER_SOURCE.contains("self.focused_note_mention_preview(cx)"),
        "Notes footer should derive focused note mention preview state"
    );
    assert!(
        NAVIGATION_SOURCE.contains("Cmd+Shift+O replace"),
        "Focused note mention preview should advertise the replace shortcut"
    );
}

#[test]
fn test_save_note_with_content_activates_existing_notes_window() {
    const WINDOW_OPS_SOURCE: &str = include_str!("window_ops.rs");
    let helper_start = WINDOW_OPS_SOURCE
        .find("pub fn save_note_with_content")
        .expect("Expected save_note_with_content helper in window_ops.rs");
    let helper_slice = &WINDOW_OPS_SOURCE[helper_start..];

    assert!(
        helper_slice.contains("window.activate_window();"),
        "save_note_with_content should activate the existing Notes window during Agent Chat handoff"
    );
}

#[test]
fn test_notes_window_registers_automation_parent_for_actions_popup() {
    const WINDOW_OPS_SOURCE: &str = include_str!("window_ops.rs");

    assert!(
        WINDOW_OPS_SOURCE.contains("upsert_runtime_window_handle(\"notes\""),
        "Notes window should register its runtime handle so shared actions popups can target it"
    );
    assert!(
        WINDOW_OPS_SOURCE.contains("AutomationWindowKind::Notes"),
        "Notes window should register itself as a Notes automation window"
    );
    assert!(
        WINDOW_OPS_SOURCE.contains("remove_automation_window(\"notes\")"),
        "Notes window close paths should clear its automation registration"
    );
}

#[test]
fn test_notes_window_supports_close_without_launcher_restore() {
    const WINDOW_OPS_SOURCE: &str = include_str!("window_ops.rs");

    assert!(
        WINDOW_OPS_SOURCE.contains("pub fn open_notes_window_without_launcher_restore"),
        "Notes window should expose a launcher-open helper that keeps the launcher hidden on close"
    );
    assert!(
        WINDOW_OPS_SOURCE.contains("restore_launcher_after_notes_close_if_needed(cx);"),
        "Notes close paths should route through the conditional launcher-restore helper"
    );
}

#[test]
fn test_notes_window_opts_out_of_app_hide() {
    const WINDOW_OPS_SOURCE: &str = include_str!("window_ops.rs");
    let configure_fn = WINDOW_OPS_SOURCE
        .split("fn configure_notes_as_floating_panel")
        .nth(1)
        .expect("configure_notes_as_floating_panel should exist");

    assert!(
        configure_fn.contains("setCanHide: false"),
        "Notes must opt out of app-level hide so main-window app-hide paths cannot conceal it"
    );
}

#[test]
fn test_main_menu_notes_entries_use_no_restore_helper() {
    const BUILTIN_SOURCE: &str = include_str!("../../app_execute/builtin_execution.rs");

    assert!(
        BUILTIN_SOURCE.contains("notes::open_notes_window_without_launcher_restore(cx)"),
        "Main-menu Notes entry should open Notes without restoring the launcher on close"
    );
}

#[test]
fn test_notes_keyboard_stops_propagation_at_start_of_global_escape_chain() {
    const KEYBOARD_SOURCE: &str = include_str!("keyboard.rs");
    assert!(
        KEYBOARD_SOURCE.contains("if is_key_escape(key) {\n            cx.stop_propagation();"),
        "Global escape chain should stop propagation before handling escape branches"
    );
}

#[test]
fn test_notes_agent_chat_escape_dismisses_local_popup_before_leaving_surface() {
    const KEYBOARD_SOURCE: &str = include_str!("keyboard.rs");
    let agent_chat_escape_branch = KEYBOARD_SOURCE
        .find("// In Agent Chat mode, intercept host-owned shortcuts before propagating to Agent Chat.")
        .and_then(|start| {
            KEYBOARD_SOURCE[start..]
                .find("if modifiers.platform {")
                .map(|end| &KEYBOARD_SOURCE[start..start + end])
        })
        .expect("Expected Notes Agent Chat keyboard branch");

    let dismiss_idx = agent_chat_escape_branch
        .find("chat.dismiss_escape_popup(cx)")
        .expect("Notes Agent Chat escape should check for local popup dismissal");
    let dismissed_branch_idx = agent_chat_escape_branch
        .find("if dismissed {")
        .expect("Notes Agent Chat escape should branch on dismissed popup state");
    let stop_idx = agent_chat_escape_branch
        .find("cx.stop_propagation();")
        .expect(
            "Notes Agent Chat escape should stop propagation after dismissing local popup state",
        );
    let return_idx = agent_chat_escape_branch
        .find("return;")
        .expect("Notes Agent Chat escape should return after dismissing local popup state");
    let switch_idx = agent_chat_escape_branch
        .find("self.switch_to_notes_surface(window, cx);")
        .expect("Notes Agent Chat escape should still fall back to leaving Agent Chat mode");

    assert!(
        dismiss_idx < switch_idx
            && dismiss_idx < dismissed_branch_idx
            && dismissed_branch_idx < stop_idx
            && stop_idx < return_idx
            && return_idx < switch_idx
            && agent_chat_escape_branch.contains("event = \"notes_agent_chat_escape_dismissed_local_popup\""),
        "Notes Agent Chat escape should dismiss local Agent Chat popup state before returning to the editor"
    );
}

#[test]
fn test_notes_actions_panel_uses_shared_disabled_opacity_constant() {
    const ACTIONS_PANEL_SOURCE: &str = include_str!("../actions_panel.rs");
    assert!(
        ACTIONS_PANEL_SOURCE.contains("use super::window::OPACITY_DISABLED;"),
        "Actions panel should use the shared Notes disabled opacity constant"
    );
    assert!(
        !ACTIONS_PANEL_SOURCE.contains("const OPACITY_DISABLED: f32"),
        "Actions panel should not define a duplicate disabled opacity constant"
    );
}

#[test]
fn test_notes_keyboard_delete_shortcut_routes_through_confirmation_helper() {
    const KEYBOARD_SOURCE: &str = include_str!("keyboard.rs");
    assert!(
        KEYBOARD_SOURCE.contains("pub(super) fn handle_platform_delete_shortcut")
            && KEYBOARD_SOURCE.contains("self.request_delete_selected_note(window, cx);"),
        "Notes keyboard delete shortcut should route through the confirmation helper"
    );
    assert!(
        KEYBOARD_SOURCE.contains("notes_delete_shortcut_requesting_confirmation"),
        "Delete shortcut helper should emit a structured confirmation-request log"
    );
}

#[test]
fn test_notes_keyboard_delete_shortcut_works_in_trash_view() {
    const KEYBOARD_SOURCE: &str = include_str!("keyboard.rs");
    // The trash-view guard was removed so the delete shortcut routes through
    // request_delete_selected_note which already handles both modes.
    assert!(
        !KEYBOARD_SOURCE.contains("trash_view_requires_dedicated_delete_flow"),
        "Delete shortcut should not block trash view — request_delete_selected_note handles both modes"
    );
}

#[test]
fn test_delete_dialog_cancel_restores_primary_focus_after_dialog() {
    const NOTES_SOURCE: &str = include_str!("notes.rs");
    assert!(
        NOTES_SOURCE.contains("this.restore_primary_focus_after_dialog(window, cx);"),
        "Cancel should restore focus after the dialog lifecycle completes"
    );
}

#[test]
fn test_delete_note_by_id_restores_editor_focus_via_focus_surface() {
    const NOTES_SOURCE: &str = include_str!("notes.rs");
    assert!(
        NOTES_SOURCE.contains("self.request_focus_surface(NotesFocusSurface::Editor, window, cx);"),
        "Confirmed delete should restore editor focus via the immediate focus-surface pattern"
    );
}

#[test]
fn test_delete_dialog_width_uses_viewport_size() {
    const NOTES_SOURCE: &str = include_str!("notes.rs");
    assert!(
        NOTES_SOURCE.contains("window.viewport_size().width"),
        "Dialog width should prefer viewport_size"
    );
    assert!(
        NOTES_SOURCE.contains("window.bounds().size.width"),
        "Dialog width should fall back to bounds when viewport width is zero"
    );
}

#[test]
fn test_on_search_change_uses_reload_helper_instead_of_silently_swallowing_errors() {
    const NOTES_SOURCE: &str = include_str!("notes.rs");
    assert!(
        NOTES_SOURCE.contains("pub(super) fn refresh_notes_for_search_query")
            && NOTES_SOURCE.contains("Failed to reload all notes while clearing the notes search"),
        "Notes search should use a dedicated helper with actionable reload errors"
    );
    assert!(
        !NOTES_SOURCE.contains("storage::get_all_notes().unwrap_or_default()"),
        "Notes search should not silently swallow errors when clearing the search query"
    );
    assert!(
        NOTES_SOURCE.contains("notes_search_refresh_started")
            && NOTES_SOURCE.contains("notes_search_refresh_completed")
            && NOTES_SOURCE.contains("notes_search_refresh_failed"),
        "Notes search refresh should emit structured logs for start, completion, and failure"
    );
}

#[test]
fn test_request_delete_selected_note_emits_structured_confirmation_logs() {
    const NOTES_SOURCE: &str = include_str!("notes.rs");
    assert!(
        NOTES_SOURCE.contains("notes_delete_confirmation_requested")
            && NOTES_SOURCE.contains("notes_delete_confirmation_opened")
            && NOTES_SOURCE.contains("notes_delete_cancelled"),
        "Delete confirmation flow should emit structured request/open/cancel logs"
    );
}

#[test]
fn test_request_delete_selected_note_log_includes_viability_fields() {
    const NOTES_SOURCE: &str = include_str!("notes.rs");
    assert!(
        NOTES_SOURCE.contains("viewport_viable,")
            && NOTES_SOURCE.contains("bounds_viable,")
            && NOTES_SOURCE.contains("source_width,")
            && NOTES_SOURCE.contains("dialog_width = dialog_width_value,"),
        "notes_delete_confirmation_requested log should include viewport_viable, bounds_viable, source_width, and dialog_width fields"
    );
}

#[test]
fn test_resolve_notes_delete_dialog_source_width_prefers_viewport() {
    assert_eq!(
        NotesApp::resolve_notes_delete_dialog_source_width(360.0, 520.0),
        360.0
    );
}

#[test]
fn test_resolve_notes_delete_dialog_source_width_falls_back_to_bounds() {
    assert_eq!(
        NotesApp::resolve_notes_delete_dialog_source_width(0.0, 520.0),
        520.0
    );
}

#[test]
fn test_resolve_notes_delete_dialog_source_width_uses_default_when_sizes_missing() {
    assert_eq!(
        NotesApp::resolve_notes_delete_dialog_source_width(0.0, 0.0),
        472.0
    );
}

#[test]
fn test_resolve_notes_delete_dialog_source_width_ignores_tiny_startup_sizes() {
    // Both viewport and bounds are tiny positive startup artifacts → default
    assert_eq!(
        NotesApp::resolve_notes_delete_dialog_source_width(0.0, 12.0),
        472.0
    );
    assert_eq!(
        NotesApp::resolve_notes_delete_dialog_source_width(8.0, 16.0),
        472.0
    );
}

#[test]
fn test_resolve_notes_delete_dialog_source_width_uses_viable_bounds_during_startup() {
    // Viewport zero but bounds is a real window size → use bounds
    assert_eq!(
        NotesApp::resolve_notes_delete_dialog_source_width(0.0, 320.0),
        320.0
    );
}

#[test]
fn test_resolve_notes_delete_dialog_source_width_uses_viable_viewport_over_tiny_bounds() {
    // Viewport is viable, bounds is tiny → use viewport
    assert_eq!(
        NotesApp::resolve_notes_delete_dialog_source_width(360.0, 12.0),
        360.0
    );
}

#[test]
fn test_delete_dialog_width_prefers_viewport_but_falls_back_when_zero() {
    const NOTES_SOURCE: &str = include_str!("notes.rs");
    assert!(
        NOTES_SOURCE.contains("window.viewport_size().width"),
        "Dialog width should still prefer viewport_size"
    );
    assert!(
        NOTES_SOURCE.contains("window.bounds().size.width"),
        "Dialog width should fall back to bounds when viewport width is unavailable"
    );
}

#[test]
fn test_permanent_delete_accepts_window_and_restores_selection_or_focus() {
    const NOTES_SOURCE: &str = include_str!("notes.rs");
    assert!(
        NOTES_SOURCE.contains("fn permanently_delete_note(")
            && NOTES_SOURCE.contains("window: &mut Window,"),
        "Permanent delete should accept window so it can restore editor state"
    );
    assert!(
        NOTES_SOURCE.contains("self.select_note_without_focus(next_note.id, window, cx);")
            && NOTES_SOURCE
                .contains("self.request_focus_surface(NotesFocusSurface::Editor, window, cx);"),
        "Permanent delete should update selection without early focus and restore via focus surface"
    );
}

#[test]
fn test_notes_render_does_not_apply_pending_focus_surface_in_render() {
    const RENDER_SOURCE: &str = include_str!("render.rs");
    assert!(
        !RENDER_SOURCE.contains("self.apply_pending_focus_surface(window, cx);"),
        "Notes render must stay read-only; apply focus outside render"
    );
}

#[test]
fn test_notes_agent_chat_focus_surface_targets_embedded_chat_focus_handle() {
    const FOCUS_SOURCE: &str = include_str!("focus.rs");
    assert!(
        FOCUS_SOURCE.contains("let focus_handle = agent_chat_entity.read(cx).focus_handle(cx);")
            && FOCUS_SOURCE.contains("window.focus(&focus_handle, cx);"),
        "Notes Agent Chat focus surface should focus the embedded Agent Chat view handle"
    );
}

#[test]
fn test_notes_agent_chat_actions_close_requests_embedded_chat_refocus() {
    const AGENT_CHAT_HOST_SOURCE: &str = include_str!("agent_chat_host.rs");
    assert!(
        AGENT_CHAT_HOST_SOURCE.contains(
            "self.request_focus_surface(focus::NotesFocusSurface::AgentChat, window, cx);"
        ) && AGENT_CHAT_HOST_SOURCE
            .contains("self.pending_focus_surface = Some(focus::NotesFocusSurface::AgentChat);"),
        "Closing the Notes-hosted Agent Chat actions popup should restore Agent Chat focus"
    );
    assert!(
        AGENT_CHAT_HOST_SOURCE
            .contains("app.close_notes_agent_chat_actions_via_host(\"dialog_on_close\", None, cx);"),
        "Dialog on_close should route Notes Agent Chat actions close through the shared host helper"
    );
}

#[test]
fn test_notes_agent_chat_host_routes_close_paths_through_host_helpers() {
    const AGENT_CHAT_HOST_SOURCE: &str = include_str!("agent_chat_host.rs");
    assert!(
        AGENT_CHAT_HOST_SOURCE.contains(
            "app.close_embedded_agent_chat_via_host(\"agent_chat_close_requested\", Some(window), cx);"
        ) && AGENT_CHAT_HOST_SOURCE.contains(
            "self.close_notes_agent_chat_actions_via_host(\"toggle_existing_window\", Some(window), cx);"
        ) && AGENT_CHAT_HOST_SOURCE
            .contains("app.close_notes_agent_chat_actions_via_host(\"dialog_on_close\", None, cx);")
            && AGENT_CHAT_HOST_SOURCE.contains(
                "app.close_embedded_agent_chat_via_host(\"agent_chat_action_close\", Some(window), cx);"
            )
            && AGENT_CHAT_HOST_SOURCE
                .contains("event = \"notes_agent_chat_action_cancel_consumed_after_on_close\""),
        "Notes embedded Agent Chat close and Agent Chat-actions close should route through shared host helpers"
    );
}

#[test]
fn test_notes_agent_chat_uses_shared_external_footer_renderer() {
    const AGENT_CHAT_HOST_SOURCE: &str = include_str!("agent_chat_host.rs");
    const RENDER_SOURCE: &str = include_str!("render.rs");
    const AGENT_CHAT_VIEW_SOURCE: &str = include_str!("../../ai/agent_chat/ui/view.rs");
    assert!(
        AGENT_CHAT_HOST_SOURCE.contains(
            "chat.set_footer_host(crate::ai::agent_chat::ui::view::AgentChatFooterHost::External);"
        ),
        "Notes-hosted Agent Chat should opt into the shared externally rendered footer"
    );
    assert!(
        RENDER_SOURCE.contains("view.build_external_host_footer(agent_chat_entity.downgrade(), cx)")
            && RENDER_SOURCE.contains(".when_some(agent_chat_footer, |d, footer| d.child(footer))"),
        "Notes Agent Chat surface should render the shared Agent Chat footer below the embedded chat view"
    );
    let footer_hints = &AGENT_CHAT_VIEW_SOURCE[AGENT_CHAT_VIEW_SOURCE
        .find("fn footer_hint_label(")
        .expect("Agent Chat footer should define shared hint labels")..];
    let run_pos = footer_hints
        .find("\"↵ Send\"")
        .expect("Agent Chat footer should render the Send hint");
    let actions_pos = footer_hints
        .find("\"⌘K Actions\"")
        .expect("Agent Chat footer should render the Actions hint");
    assert!(
        run_pos < actions_pos,
        "Notes-hosted Agent Chat should mirror the main-window Agent Chat footer labels and order"
    );
}

#[test]
fn test_delete_dialog_requests_dialog_focus_surface_before_opening() {
    const NOTES_SOURCE: &str = include_str!("notes.rs");
    assert!(
        NOTES_SOURCE.contains("self.request_focus_surface(NotesFocusSurface::Dialog, window, cx);"),
        "Delete flow should request the dialog focus surface before opening the confirm dialog"
    );
}

#[test]
fn test_confirmed_delete_updates_selection_without_early_editor_refocus() {
    const NOTES_SOURCE: &str = include_str!("notes.rs");
    assert!(
        NOTES_SOURCE.contains("self.select_note_without_focus(next_note.id, window, cx);")
            && NOTES_SOURCE
                .contains("self.request_focus_surface(NotesFocusSurface::Editor, window, cx);"),
        "Confirmed delete should update selection first and restore editor focus after dialog dismissal"
    );
}
