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
fn tilde_mini_file_search_seeds_directory_rows_before_first_paint() {
    let source = fs::read_to_string("src/app_impl/filter_input_core.rs")
        .expect("Failed to read src/app_impl/filter_input_core.rs");

    assert!(
        source.contains("seed_file_search_directory_results_for_first_paint"),
        "fresh mini file-search entry must seed directory rows before switching surfaces"
    );
    assert!(
        source.contains("list_directory_with_options"),
        "first-paint seeding must use the real directory listing path"
    );
    assert!(
        source.contains("let seeded_initial_results"),
        "open_file_search_view must track whether first-paint rows were seeded"
    );
    assert!(
        source.contains("preserve_current_results_until_first_batch || seeded_initial_results"),
        "seeded rows must be preserved until the async directory stream replaces them"
    );
    assert!(
        source.contains("self.file_search_display_indices.len()"),
        "mini file-search sizing should use seeded display rows instead of a zero-row flash"
    );
    assert!(
        source.contains("self.current_view = AppView::FileSearchView")
            && source.contains("self.rekey_main_automation_surface_from_current_view();"),
        "file-search entry must re-key main automation surface after switching current_view"
    );
}

#[test]
fn filter_input_change_routes_script_list_mode_exit_entries() {
    let source = fs::read_to_string("src/app_impl/filter_input_change.rs")
        .expect("Failed to read src/app_impl/filter_input_change.rs");

    assert!(
        source.contains("Prompt-builder sigils (@, /, |, .) are handled by the Spine")
            && source.contains("stay in the main list"),
        "prompt-builder sigils must remain on the main-list Spine projection instead of switching AppView"
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
