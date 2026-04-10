//! Integration tests: verify file-search AI chords route through the
//! new selection-or-query fallback path and advertise the updated copy.

use std::fs;

#[test]
fn file_search_key_handler_routes_cmd_enter_to_ai() {
    let source = fs::read_to_string("src/render_builtins/file_search.rs")
        .expect("Failed to read src/render_builtins/file_search.rs");

    assert!(
        source.contains("open_file_search_selection_or_query_in_tab_ai"),
        "file_search key handler must route through the selection-or-query helper"
    );
}

#[test]
fn file_search_view_is_eligible_for_shared_global_cmd_enter_route() {
    let source = fs::read_to_string("src/app_impl/tab_ai_mode.rs")
        .expect("Failed to read src/app_impl/tab_ai_mode.rs");

    assert!(
        source.contains("AppView::FileSearchView { .. }"),
        "FileSearchView must opt into the shared main-window Cmd+Enter ACP route"
    );
}

#[test]
fn file_search_plain_cmd_enter_attempts_shared_global_route_first() {
    let source = fs::read_to_string("src/render_builtins/file_search.rs")
        .expect("Failed to read src/render_builtins/file_search.rs");

    assert!(
        source.contains("file_search_cmd_enter_global_route_attempted"),
        "FileSearch plain Cmd+Enter must emit a structured log before using the shared route"
    );
    assert!(
        source.contains("this.try_route_global_cmd_enter_to_acp_context_capture(cx)"),
        "FileSearch plain Cmd+Enter must attempt the shared global ACP route first"
    );
}

#[test]
fn file_search_cmd_shift_enter_preserves_local_ai_path() {
    let source = fs::read_to_string("src/render_builtins/file_search.rs")
        .expect("Failed to read src/render_builtins/file_search.rs");

    assert!(
        source.contains("file_search_cmd_shift_enter_local_ai"),
        "FileSearch Cmd+Shift+Enter must keep a distinct local AI path"
    );
    assert!(
        source.contains("this.open_file_search_selection_or_query_in_tab_ai("),
        "FileSearch shift variant must preserve the existing selection/query helper"
    );
}

#[test]
fn file_search_view_is_eligible_for_shared_global_cmd_enter_route() {
    let source = fs::read_to_string("src/app_impl/tab_ai_mode.rs")
        .expect("Failed to read src/app_impl/tab_ai_mode.rs");

    assert!(
        source.contains("fn supports_global_cmd_enter_ai_entry(view: &AppView) -> bool"),
        "tab_ai_mode must define the shared global Cmd+Enter eligibility gate"
    );
    assert!(
        source.contains("| AppView::FileSearchView { .. }"),
        "FileSearchView must participate in the shared global Cmd+Enter ACP route"
    );
}

#[test]
fn tab_ai_mode_has_file_search_intent_builder() {
    let source = fs::read_to_string("src/app_impl/tab_ai_mode.rs")
        .expect("Failed to read src/app_impl/tab_ai_mode.rs");

    assert!(
        source.contains("build_file_search_ai_entry_intent"),
        "tab_ai_mode must define build_file_search_ai_entry_intent"
    );
    assert!(
        source.contains("\"directory\""),
        "intent builder must distinguish directories"
    );
    assert!(
        source.contains("\"file\""),
        "intent builder must distinguish files"
    );
}

#[test]
fn file_search_hints_advertise_real_ai_chords() {
    let source = fs::read_to_string("src/render_builtins/file_search.rs")
        .expect("Failed to read src/render_builtins/file_search.rs");

    assert!(
        source.contains("⌘↵ Explain"),
        "file search hints must advertise ⌘↵ Explain chord"
    );
    assert!(
        source.contains("⌘⇧↵ Plan"),
        "file search hints must advertise ⌘⇧↵ Plan chord"
    );
}
