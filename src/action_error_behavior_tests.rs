// Behavior tests for action handler error and feedback paths.
// Verifies that error paths use Toast (not HUD), copy actions do not
// hide the window on clipboard failure, SDK routing logs unknown actions,
// and file-search actions surface errors when the path is missing.

use std::fs;

fn read_handle_action() -> String {
    crate::test_utils::read_all_handle_action_sources()
}

fn read_sdk_actions() -> String {
    fs::read_to_string("src/app_actions/sdk_actions.rs")
        .expect("Failed to read src/app_actions/sdk_actions.rs")
}

// ---------------------------------------------------------------------------
// 1. Copy actions must NOT hide the window when the clipboard write fails.
//    hide_main_and_reset must only appear inside Ok(_) arms for copy_path,
//    copy_deeplink, and copy_content.
// ---------------------------------------------------------------------------

/// For copy_path: hide_main_and_reset must be inside the Ok branch, not after
/// the match block.
#[test]
fn copy_path_does_not_hide_window_on_clipboard_error() {
    let content = read_handle_action();

    // Find the copy_path action block
    let copy_path_start = content
        .find("\"copy_path\"")
        .expect("Expected handle_action/ to contain copy_path action");

    // Find the next action block to bound our search
    let search_region = &content[copy_path_start..];
    let next_action = search_region
        .find("\"copy_deeplink\"")
        .unwrap_or(search_region.len());
    let copy_path_block = &search_region[..next_action];

    // In the Err branch, there should NOT be hide_main_and_reset
    // Instead, the Err branch should use toast_manager (Toast::error)
    let err_sections: Vec<&str> = copy_path_block
        .split("Err(e)")
        .skip(1) // skip text before first Err
        .collect();

    assert!(
        !err_sections.is_empty(),
        "Expected copy_path to have Err(e) branches for clipboard failure"
    );

    for err_section in &err_sections {
        // Take a reasonable window after Err(e) — up to the next closing brace pair
        let window = &err_section[..err_section.len().min(300)];
        assert!(
            !window.contains("hide_main_and_reset"),
            "copy_path Err branch must NOT call hide_main_and_reset. Found:\n{}",
            window
        );
    }
}

/// For copy_deeplink: hide_main_and_reset must be inside the Ok branch only.
#[test]
fn copy_deeplink_does_not_hide_window_on_clipboard_error() {
    let content = read_handle_action();

    let deeplink_start = content
        .find("\"copy_deeplink\"")
        .expect("Expected handle_action/ to contain copy_deeplink action");

    let search_region = &content[deeplink_start..];
    // Bound: find next top-level action (the action after copy_deeplink)
    let next_action = search_region[20..]
        .find("\n            \"")
        .map(|pos| pos + 20)
        .unwrap_or(search_region.len());
    let deeplink_block = &search_region[..next_action];

    let err_sections: Vec<&str> = deeplink_block
        .split("Err(e)")
        .skip(1)
        .collect();

    assert!(
        !err_sections.is_empty(),
        "Expected copy_deeplink to have Err(e) branches for clipboard failure"
    );

    for err_section in &err_sections {
        let window = &err_section[..err_section.len().min(300)];
        assert!(
            !window.contains("hide_main_and_reset"),
            "copy_deeplink Err branch must NOT call hide_main_and_reset. Found:\n{}",
            window
        );
    }
}

/// For copy_content: hide_main_and_reset must be inside the Ok branch only.
#[test]
fn copy_content_does_not_hide_window_on_clipboard_error() {
    let content = read_handle_action();

    let copy_content_start = content
        .find("\"copy_content\"")
        .expect("Expected handle_action/ to contain copy_content action");

    let search_region = &content[copy_content_start..];
    let next_action = search_region[20..]
        .find("\n            \"")
        .map(|pos| pos + 20)
        .unwrap_or(search_region.len());
    let copy_content_block = &search_region[..next_action];

    let err_sections: Vec<&str> = copy_content_block
        .split("Err(e)")
        .skip(1)
        .collect();

    assert!(
        !err_sections.is_empty(),
        "Expected copy_content to have Err(e) branches for clipboard/file-read failure"
    );

    for err_section in &err_sections {
        let window = &err_section[..err_section.len().min(300)];
        assert!(
            !window.contains("hide_main_and_reset"),
            "copy_content Err branch must NOT call hide_main_and_reset. Found:\n{}",
            window
        );
    }
}

// ---------------------------------------------------------------------------
// 2. Error paths must use Toast::error(), not show_hud, for clipboard and
//    file-search failures.
// ---------------------------------------------------------------------------

/// Copy action clipboard errors must use Toast::error, not show_hud.
#[test]
fn copy_action_errors_use_toast_not_hud() {
    let content = read_handle_action();

    // Check copy_path error path
    let copy_path_start = content
        .find("\"copy_path\"")
        .expect("copy_path action not found");
    let next_action_offset = content[copy_path_start + 20..]
        .find("\n            \"")
        .map(|p| p + copy_path_start + 20)
        .unwrap_or(content.len());
    let copy_path_block = &content[copy_path_start..next_action_offset];

    // The error branch should contain Toast::error
    assert!(
        copy_path_block.contains("Toast::error(\"Failed to copy path\""),
        "copy_path clipboard error must use Toast::error, not show_hud"
    );

    // Check copy_deeplink error path
    let deeplink_start = content
        .find("\"copy_deeplink\"")
        .expect("copy_deeplink action not found");
    let next_offset = content[deeplink_start + 20..]
        .find("\n            \"")
        .map(|p| p + deeplink_start + 20)
        .unwrap_or(content.len());
    let deeplink_block = &content[deeplink_start..next_offset];

    assert!(
        deeplink_block.contains("Toast::error(\"Failed to copy deeplink\""),
        "copy_deeplink clipboard error must use Toast::error, not show_hud"
    );

    // Check copy_content error path
    let content_start = content
        .find("\"copy_content\"")
        .expect("copy_content action not found");
    let next_off = content[content_start + 20..]
        .find("\n            \"")
        .map(|p| p + content_start + 20)
        .unwrap_or(content.len());
    let content_block = &content[content_start..next_off];

    assert!(
        content_block.contains("Toast::error(\"Failed to copy content\""),
        "copy_content clipboard error must use Toast::error, not show_hud"
    );
    assert!(
        content_block.contains("Toast::error(") && content_block.contains("Failed to read file"),
        "copy_content file-read error must use Toast::error, not show_hud"
    );
}

/// File-search action errors must use Toast::error, not show_hud.
#[test]
fn file_search_action_errors_use_toast_not_hud() {
    let content = read_handle_action();

    // Find the file search action block (open_file | open_directory | ...)
    let fs_start = content
        .find("\"open_file\"\n            | \"open_directory\"")
        .or_else(|| content.find("\"open_file\""))
        .expect("File search action block not found");

    // Bound by copy_filename action
    let next_action = content[fs_start..]
        .find("\"copy_filename\"")
        .map(|p| p + fs_start)
        .unwrap_or(content.len());
    let fs_block = &content[fs_start..next_action];

    // The Err branch should use toast_manager.push(Toast::error(...))
    let err_pos = fs_block
        .find("Err(e) =>")
        .expect("File search action block must have an Err(e) branch");
    let after_err = &fs_block[err_pos..fs_block.len().min(err_pos + 1200)];

    assert!(
        after_err.contains("toast_manager.push("),
        "File search action Err branch must use toast_manager.push, not show_hud. Found:\n{}",
        after_err
    );
    assert!(
        after_err.contains("Toast::error("),
        "File search action Err branch must use Toast::error variant. Found:\n{}",
        after_err
    );
}

// ---------------------------------------------------------------------------
// 3. SDK routing must surface visible feedback on unknown action.
// ---------------------------------------------------------------------------

/// Unknown SDK actions must be logged via tracing::warn (not silent logging::log).
#[test]
fn sdk_routing_warns_on_unknown_action() {
    let content = read_sdk_actions();

    assert!(
        content.contains("sdk_unknown_action"),
        "SDK routing must log a tracing event for unknown actions"
    );
    assert!(
        content.contains("tracing::warn!"),
        "SDK routing unknown-action path must use tracing::warn, not logging::log"
    );
}

/// SDK routing must warn when no actions are registered.
#[test]
fn sdk_routing_warns_when_no_actions_registered() {
    let content = read_sdk_actions();

    assert!(
        content.contains("sdk_no_actions_registered"),
        "SDK routing must log when actions are triggered with no registered SDK actions"
    );
}

/// SDK routing must not use the legacy logging::log API.
#[test]
fn sdk_routing_uses_tracing_not_legacy_logging() {
    let content = read_sdk_actions();

    assert!(
        !content.contains("logging::log("),
        "sdk_actions.rs must use tracing:: instead of logging::log. Found legacy calls."
    );
}

// ---------------------------------------------------------------------------
// 4. File-search actions must show an error when file_search_actions_path
//    is None.
// ---------------------------------------------------------------------------

/// The file search action block must have an else branch that surfaces
/// visible feedback when no file path is available.
#[test]
fn file_search_actions_show_error_when_path_is_none() {
    let content = read_handle_action();

    // Find the file search action block
    let fs_start = content
        .find("\"open_file\"\n            | \"open_directory\"")
        .or_else(|| content.find("\"open_file\""))
        .expect("File search action block not found");

    // Bound by copy_filename action
    let next_action = content[fs_start..]
        .find("\"copy_filename\"")
        .map(|p| p + fs_start)
        .unwrap_or(content.len());
    let fs_block = &content[fs_start..next_action];

    // Must have an else branch for the if-let on file_search_actions_path
    assert!(
        fs_block.contains("} else {")
            && fs_block.contains("No file selected for this action"),
        "File search action block must show 'No file selected' error when path is None"
    );

    // The else branch must use Toast::error
    assert!(
        fs_block.contains("Toast::error(")
            && fs_block.contains("No file selected"),
        "File search path-is-None error must use Toast::error for visible feedback"
    );
}
