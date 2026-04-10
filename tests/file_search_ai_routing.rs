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
    assert!(
        source.contains("⌘↵ / ⌘⇧↵"),
        "file_search key handler should document both explain and plan AI chords"
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
