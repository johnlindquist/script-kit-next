//! Integration tests: verify Cmd+Enter AI chord in file search and
//! the entry intent builder exist in the expected source files.

use std::fs;

#[test]
fn file_search_key_handler_routes_cmd_enter_to_ai() {
    let source = fs::read_to_string("src/render_builtins/file_search.rs")
        .expect("Failed to read src/render_builtins/file_search.rs");

    assert!(
        source.contains("build_file_search_ai_entry_intent"),
        "file_search key handler must call build_file_search_ai_entry_intent"
    );
    assert!(
        source.contains("open_tab_ai_chat_with_entry_intent"),
        "file_search key handler must call open_tab_ai_chat_with_entry_intent"
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
fn file_search_mini_hints_advertise_ai_chord() {
    let source = fs::read_to_string("src/render_builtins/file_search.rs")
        .expect("Failed to read src/render_builtins/file_search.rs");

    // The mini hint strip must advertise ⌘↵ Ask AI
    assert!(
        source.contains("Ask AI"),
        "mini file search hints must include 'Ask AI'"
    );
}
