// Regression tests for file/clipboard tool action handlers:
// clipboard_share, clipboard_ocr, copy_content, clipboard_paste.

use crate::test_utils::read_source as read;

// ---------------------------------------------------------------------------
// clipboard_share — success path
// ---------------------------------------------------------------------------

#[test]
fn clipboard_share_handles_text_and_image_content_types() {
    let content = read("src/app_actions/handle_action.rs");

    let share_pos = content
        .find("\"clipboard_share\"")
        .expect("Expected clipboard_share action handler");
    let block = &content[share_pos..content.len().min(share_pos + 800)];

    assert!(
        block.contains("ContentType::Text"),
        "clipboard_share should handle Text content type"
    );
    assert!(
        block.contains("ContentType::Image"),
        "clipboard_share should handle Image content type"
    );
    assert!(
        block.contains("show_share_sheet"),
        "clipboard_share should call show_share_sheet for sharing"
    );
    assert!(
        block.contains("Share sheet opened"),
        "clipboard_share should show success HUD after opening share sheet"
    );
}

// ---------------------------------------------------------------------------
// clipboard_share — error paths
// ---------------------------------------------------------------------------

#[test]
fn clipboard_share_shows_error_when_no_entry_selected() {
    let content = read("src/app_actions/handle_action.rs");

    let share_pos = content
        .find("\"clipboard_share\"")
        .expect("Expected clipboard_share action handler");
    let block = &content[share_pos..content.len().min(share_pos + 300)];

    assert!(
        block.contains("No clipboard entry selected"),
        "clipboard_share should show error when no entry is selected"
    );
}

#[test]
fn clipboard_share_shows_error_when_content_unavailable() {
    let content = read("src/app_actions/handle_action.rs");

    let share_pos = content
        .find("\"clipboard_share\"")
        .expect("Expected clipboard_share action handler");
    let block = &content[share_pos..content.len().min(share_pos + 400)];

    assert!(
        block.contains("Clipboard entry content unavailable"),
        "clipboard_share should show error when entry content cannot be loaded"
    );
}

#[test]
fn clipboard_share_shows_error_when_image_decode_fails() {
    let content = read("src/app_actions/handle_action.rs");

    let share_pos = content
        .find("\"clipboard_share\"")
        .expect("Expected clipboard_share action handler");
    let block = &content[share_pos..content.len().min(share_pos + 800)];

    assert!(
        block.contains("Failed to decode clipboard image"),
        "clipboard_share should show error when image decoding fails"
    );
}

// ---------------------------------------------------------------------------
// clipboard_ocr — success path
// ---------------------------------------------------------------------------

#[test]
fn clipboard_ocr_guards_non_image_entries() {
    let content = read("src/app_actions/handle_action.rs");

    let ocr_pos = content
        .find("\"clipboard_ocr\"")
        .expect("Expected clipboard_ocr action handler");
    let block = &content[ocr_pos..content.len().min(ocr_pos + 400)];

    assert!(
        block.contains("ContentType::Image"),
        "clipboard_ocr should check that entry is an image"
    );
    assert!(
        block.contains("OCR is only available for images"),
        "clipboard_ocr should reject non-image entries with clear message"
    );
}

#[test]
fn clipboard_ocr_uses_cached_text_when_available() {
    let content = read("src/app_actions/handle_action.rs");

    let ocr_pos = content
        .find("\"clipboard_ocr\"")
        .expect("Expected clipboard_ocr action handler");
    let block = &content[ocr_pos..content.len().min(ocr_pos + 600)];

    assert!(
        block.contains("ocr_text"),
        "clipboard_ocr should check for cached OCR text"
    );
    assert!(
        block.contains("using cached OCR text"),
        "clipboard_ocr should log when using cached OCR text"
    );
    assert!(
        block.contains("Copied text from image"),
        "clipboard_ocr success should show copy feedback"
    );
}

// ---------------------------------------------------------------------------
// clipboard_ocr — error paths
// ---------------------------------------------------------------------------

#[test]
fn clipboard_ocr_shows_error_when_no_entry_selected() {
    let content = read("src/app_actions/handle_action.rs");

    let ocr_pos = content
        .find("\"clipboard_ocr\"")
        .expect("Expected clipboard_ocr action handler");
    let block = &content[ocr_pos..content.len().min(ocr_pos + 200)];

    assert!(
        block.contains("No clipboard entry selected"),
        "clipboard_ocr should show error when no entry is selected"
    );
}

// ---------------------------------------------------------------------------
// copy_content — success path
// ---------------------------------------------------------------------------

#[test]
fn copy_content_reads_file_and_copies_to_clipboard() {
    let content = read("src/app_actions/handle_action.rs");

    let copy_pos = content
        .find("\"copy_content\"")
        .expect("Expected copy_content action handler");
    let block = &content[copy_pos..content.len().min(copy_pos + 600)];

    assert!(
        block.contains("std::fs::read_to_string"),
        "copy_content should read file contents"
    );
    assert!(
        block.contains("copy_to_clipboard_with_feedback"),
        "copy_content should use copy_to_clipboard_with_feedback for consistent UX"
    );
    assert!(
        block.contains("Content copied to clipboard"),
        "copy_content should show success feedback after copying"
    );
}

// ---------------------------------------------------------------------------
// copy_content — error paths
// ---------------------------------------------------------------------------

#[test]
fn copy_content_shows_error_for_unsupported_item_types() {
    let content = read("src/app_actions/handle_action.rs");

    let copy_pos = content
        .find("\"copy_content\"")
        .expect("Expected copy_content action handler");
    let block = &content[copy_pos..content.len().min(copy_pos + 600)];

    assert!(
        block.contains("Cannot copy content for this item type"),
        "copy_content should show error for unsupported item types"
    );
}

#[test]
fn copy_content_shows_error_when_no_selection() {
    let content = read("src/app_actions/handle_action.rs");

    let copy_pos = content
        .find("\"copy_content\"")
        .expect("Expected copy_content action handler");
    let block = &content[copy_pos..content.len().min(copy_pos + 600)];

    assert!(
        block.contains("selection_required_message_for_action(action_id)"),
        "copy_content should use selection_required_message when nothing is selected"
    );
}

// ---------------------------------------------------------------------------
// clipboard_paste — success path
// ---------------------------------------------------------------------------

#[test]
fn clipboard_paste_copies_entry_and_hides_window() {
    let content = read("src/app_actions/handle_action.rs");

    let paste_pos = content
        .find("\"clipboard_paste\"")
        .expect("Expected clipboard_paste action handler");
    let block = &content[paste_pos..content.len().min(paste_pos + 400)];

    assert!(
        block.contains("copy_entry_to_clipboard"),
        "clipboard_paste should copy entry to system clipboard before pasting"
    );
    assert!(
        block.contains("hide_main_and_reset"),
        "clipboard_paste should hide the main window after paste"
    );
}

// ---------------------------------------------------------------------------
// clipboard_paste — error path
// ---------------------------------------------------------------------------

#[test]
fn clipboard_paste_shows_error_when_no_entry_selected() {
    let content = read("src/app_actions/handle_action.rs");

    let paste_pos = content
        .find("\"clipboard_paste\"")
        .expect("Expected clipboard_paste action handler");
    let block = &content[paste_pos..content.len().min(paste_pos + 200)];

    assert!(
        block.contains("No clipboard entry selected"),
        "clipboard_paste should show error when no entry is selected"
    );
}
