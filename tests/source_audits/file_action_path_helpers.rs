//! Source audit: file actions in handle_action/files.rs use the shared
//! `resolve_file_action_path` helper with `extract_path_for_reveal` /
//! `extract_path_for_copy` extractors from `action_helpers`.
//!
//! This prevents regressions where individual file actions inline their own
//! path-extraction logic instead of routing through the canonical helpers.

use super::read_source as read;

fn files_content() -> String {
    read("src/app_actions/handle_action/files.rs")
}

// ---------------------------------------------------------------------------
// resolve_file_action_path — shared helper is used, not inline extraction
// ---------------------------------------------------------------------------

#[test]
fn files_handler_defines_resolve_file_action_path() {
    let content = files_content();
    assert!(
        content.contains("fn resolve_file_action_path"),
        "Expected files.rs to define the shared resolve_file_action_path helper"
    );
}

#[test]
fn reveal_in_finder_uses_resolve_file_action_path() {
    let content = files_content();
    let reveal_pos = content
        .find("\"reveal_in_finder\"")
        .expect("Expected reveal_in_finder action handler");
    let block = &content[reveal_pos..content.len().min(reveal_pos + 1500)];

    assert!(
        block.contains("self.resolve_file_action_path("),
        "reveal_in_finder must use resolve_file_action_path, not inline path extraction"
    );
    assert!(
        block.contains("extract_path_for_reveal"),
        "reveal_in_finder must pass extract_path_for_reveal as the extractor"
    );
}

#[test]
fn copy_path_uses_resolve_file_action_path() {
    let content = files_content();
    let copy_pos = content
        .find("\"copy_path\"")
        .expect("Expected copy_path action handler");
    let block = &content[copy_pos..content.len().min(copy_pos + 1500)];

    assert!(
        block.contains("self.resolve_file_action_path("),
        "copy_path must use resolve_file_action_path, not inline path extraction"
    );
    assert!(
        block.contains("extract_path_for_copy"),
        "copy_path must pass extract_path_for_copy as the extractor"
    );
}

// ---------------------------------------------------------------------------
// No inline path extraction — file actions must not bypass shared helpers
// ---------------------------------------------------------------------------

#[test]
fn file_actions_do_not_inline_search_result_matching_for_paths() {
    let content = files_content();

    // The handle_file_action function should not contain raw SearchResult
    // pattern matching for path extraction — that belongs in action_helpers.
    let fn_start = content
        .find("fn handle_file_action(")
        .expect("Expected handle_file_action function");
    let fn_body = &content[fn_start..];

    assert!(
        !fn_body.contains("SearchResult::Script(m) => Ok(m.script.path"),
        "handle_file_action must not inline SearchResult path extraction — use resolve_file_action_path"
    );
}

// ---------------------------------------------------------------------------
// resolve_file_action_path — consumes file_search_actions_path first
// ---------------------------------------------------------------------------

#[test]
fn resolve_file_action_path_prioritizes_file_search_actions_path() {
    let content = files_content();
    let fn_start = content
        .find("fn resolve_file_action_path")
        .expect("Expected resolve_file_action_path function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 1000)];

    assert!(
        fn_body.contains("self.file_search_actions_path.take()"),
        "resolve_file_action_path must consume file_search_actions_path (take) as first priority"
    );
}

#[test]
fn resolve_file_action_path_falls_back_to_selected_result() {
    let content = files_content();
    let fn_start = content
        .find("fn resolve_file_action_path")
        .expect("Expected resolve_file_action_path function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 1000)];

    assert!(
        fn_body.contains("self.get_selected_result()"),
        "resolve_file_action_path must fall back to get_selected_result when no file_search_actions_path"
    );
}
