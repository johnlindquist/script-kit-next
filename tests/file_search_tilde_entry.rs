//! Integration tests: verify the `~` tilde trigger routes ScriptList into
//! mini file search and that the render pipeline threads `FileSearchPresentation`.

use std::fs;

#[test]
fn filter_input_change_routes_tilde_into_mini_file_search() {
    let source = fs::read_to_string("src/app_impl/filter_input_change.rs")
        .expect("Failed to read src/app_impl/filter_input_change.rs");

    assert!(
        source.contains("special_entry_from_script_list_filter"),
        "filter_input_change must call the shared ScriptList special-entry classifier"
    );
    assert!(
        source.contains("FileSearchPresentation::Mini"),
        "filter_input_change must open Mini presentation on ~ trigger"
    );
    assert!(
        source.contains("open_file_search_view"),
        "filter_input_change must call open_file_search_view"
    );
}

#[test]
fn filter_input_change_routes_script_list_special_entries() {
    let source = fs::read_to_string("src/app_impl/filter_input_change.rs")
        .expect("Failed to read src/app_impl/filter_input_change.rs");

    assert!(
        source.contains("ScriptListSpecialEntry::AcpSlashPicker"),
        "filter_input_change must route '/' into ACP slash picker"
    );
    assert!(
        source.contains("open_tab_ai_acp_with_slash_picker(window, cx)"),
        "filter_input_change must call the ACP slash-picker helper"
    );
    assert!(
        source.contains("ScriptListSpecialEntry::AcpMentionPicker"),
        "filter_input_change must route '@' into the ACP mention picker"
    );
    assert!(
        source.contains("open_tab_ai_acp_with_mention_picker(window, cx)"),
        "filter_input_change must open ACP mention picker for '@'"
    );
    assert!(
        source.contains("ScriptListSpecialEntry::QuickTerminal"),
        "filter_input_change must route '>' into quick terminal"
    );
    assert!(
        source.contains("open_quick_terminal"),
        "filter_input_change must open quick terminal for '>'"
    );
    assert!(
        source.contains("ScriptListSpecialEntry::ActionsHelp"),
        "filter_input_change must route '?' into help/actions"
    );
    assert!(
        source.contains("toggle_actions(cx, window)"),
        "filter_input_change must open actions/help for '?'"
    );
}

#[test]
fn render_impl_threads_file_search_presentation() {
    let source = fs::read_to_string("src/main_sections/render_impl.rs")
        .expect("Failed to read src/main_sections/render_impl.rs");

    assert!(
        source.contains("presentation"),
        "render_impl must destructure FileSearchView.presentation"
    );
    assert!(
        source.contains("render_file_search"),
        "render_impl must call render_file_search with presentation"
    );
}
