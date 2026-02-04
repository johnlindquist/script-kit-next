//! Tests for action builder functions.
//!
//! Validates that built-in action builders produce correct actions with
//! expected IDs, titles, shortcuts, and conditional behavior across
//! multiple contexts: file search, path prompt, clipboard history,
//! chat prompt, notes command bar, and AI command bar.

use super::*;
use crate::clipboard_history::ContentType;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;

// ============================================================
// Helper: extract action IDs from a Vec<Action>
// ============================================================

fn action_ids(actions: &[Action]) -> Vec<&str> {
    actions.iter().map(|a| a.id.as_str()).collect()
}

fn find_action<'a>(actions: &'a [Action], id: &str) -> Option<&'a Action> {
    actions.iter().find(|a| a.id == id)
}

// ============================================================
// 1. File context actions: files vs directories
// ============================================================

#[test]
fn file_context_directory_primary_action_is_open_directory() {
    let dir = FileInfo {
        path: "/Users/test/Documents".into(),
        name: "Documents".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&dir);
    let ids = action_ids(&actions);

    assert_eq!(
        actions[0].id, "open_directory",
        "First action for a directory must be open_directory"
    );
    assert!(
        actions[0].title.contains("Documents"),
        "Title should include directory name"
    );
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));

    // Directories should NOT have quick_look
    assert!(
        !ids.contains(&"quick_look"),
        "Directories should not have Quick Look"
    );
}

#[test]
fn file_context_file_primary_action_is_open_file() {
    let file = FileInfo {
        path: "/Users/test/photo.png".into(),
        name: "photo.png".into(),
        file_type: FileType::Image,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let ids = action_ids(&actions);

    assert_eq!(
        actions[0].id, "open_file",
        "First action for a file must be open_file"
    );
    assert!(
        actions[0].title.contains("photo.png"),
        "Title should include filename"
    );

    // Files SHOULD have quick_look on macOS
    #[cfg(target_os = "macos")]
    assert!(
        ids.contains(&"quick_look"),
        "Files should have Quick Look on macOS"
    );
}

#[test]
fn file_context_common_actions_present_for_both() {
    let file = FileInfo {
        path: "/Users/test/readme.md".into(),
        name: "readme.md".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let dir = FileInfo {
        path: "/Users/test/src".into(),
        name: "src".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };

    for info in &[&file, &dir] {
        let actions = get_file_context_actions(info);
        let ids = action_ids(&actions);
        assert!(
            ids.contains(&"reveal_in_finder"),
            "{} should have reveal_in_finder",
            info.name
        );
        assert!(
            ids.contains(&"copy_path"),
            "{} should have copy_path",
            info.name
        );
        assert!(
            ids.contains(&"copy_filename"),
            "{} should have copy_filename",
            info.name
        );
    }
}

#[test]
fn file_context_macos_specific_actions() {
    let file = FileInfo {
        path: "/Users/test/data.csv".into(),
        name: "data.csv".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let ids = action_ids(&actions);

    #[cfg(target_os = "macos")]
    {
        assert!(ids.contains(&"open_with"), "macOS should have Open With...");
        assert!(ids.contains(&"show_info"), "macOS should have Get Info");
    }

    #[cfg(not(target_os = "macos"))]
    {
        assert!(
            !ids.contains(&"open_with"),
            "Non-macOS should not have Open With"
        );
        assert!(
            !ids.contains(&"show_info"),
            "Non-macOS should not have Get Info"
        );
    }
}

// ============================================================
// 2. Path context actions: files vs directories
// ============================================================

#[test]
fn path_context_directory_has_open_directory_as_primary() {
    let path = PathInfo::new("Projects", "/Users/test/Projects", true);
    let actions = get_path_context_actions(&path);

    assert_eq!(actions[0].id, "open_directory");
    assert!(actions[0].title.contains("Projects"));
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn path_context_file_has_select_file_as_primary() {
    let path = PathInfo::new("config.toml", "/Users/test/config.toml", false);
    let actions = get_path_context_actions(&path);

    assert_eq!(actions[0].id, "select_file");
    assert!(actions[0].title.contains("config.toml"));
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn path_context_all_common_actions_present() {
    let path = PathInfo::new("file.txt", "/tmp/file.txt", false);
    let actions = get_path_context_actions(&path);
    let ids = action_ids(&actions);

    let expected = [
        "copy_path",
        "open_in_finder",
        "open_in_editor",
        "open_in_terminal",
        "copy_filename",
        "move_to_trash",
    ];
    for &expected_id in &expected {
        assert!(
            ids.contains(&expected_id),
            "Path context should have {}",
            expected_id
        );
    }
}

#[test]
fn path_context_move_to_trash_description_matches_type() {
    let dir_path = PathInfo::new("build", "/tmp/build", true);
    let file_path = PathInfo::new("temp.log", "/tmp/temp.log", false);

    let dir_actions = get_path_context_actions(&dir_path);
    let file_actions = get_path_context_actions(&file_path);

    let dir_trash = find_action(&dir_actions, "move_to_trash").unwrap();
    let file_trash = find_action(&file_actions, "move_to_trash").unwrap();

    assert!(
        dir_trash.description.as_ref().unwrap().contains("folder"),
        "Directory trash description should mention 'folder'"
    );
    assert!(
        file_trash.description.as_ref().unwrap().contains("file"),
        "File trash description should mention 'file'"
    );
}

// ============================================================
// 3. Clipboard history actions: text vs image, pinned vs unpinned
// ============================================================

fn make_text_entry(pinned: bool) -> ClipboardEntryInfo {
    ClipboardEntryInfo {
        id: "entry-1".into(),
        content_type: ContentType::Text,
        pinned,
        preview: "Hello world".into(),
        image_dimensions: None,
        frontmost_app_name: Some("VS Code".into()),
    }
}

fn make_image_entry(pinned: bool) -> ClipboardEntryInfo {
    ClipboardEntryInfo {
        id: "entry-2".into(),
        content_type: ContentType::Image,
        pinned,
        preview: String::new(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    }
}

#[test]
fn clipboard_text_entry_has_core_actions() {
    let entry = make_text_entry(false);
    let actions = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions);

    let expected = [
        "clipboard_paste",
        "clipboard_copy",
        "clipboard_paste_keep_open",
        "clipboard_share",
        "clipboard_attach_to_ai",
        "clipboard_save_snippet",
        "clipboard_save_file",
        "clipboard_delete",
        "clipboard_delete_multiple",
        "clipboard_delete_all",
    ];
    for &expected_id in &expected {
        assert!(
            ids.contains(&expected_id),
            "Text entry should have {}",
            expected_id
        );
    }
}

#[test]
fn clipboard_paste_title_includes_app_name() {
    let entry = make_text_entry(false);
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = find_action(&actions, "clipboard_paste").unwrap();

    assert_eq!(paste.title, "Paste to VS Code");
}

#[test]
fn clipboard_paste_title_fallback_when_no_app() {
    let entry = make_image_entry(false);
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = find_action(&actions, "clipboard_paste").unwrap();

    assert_eq!(paste.title, "Paste to Active App");
}

#[test]
fn clipboard_image_entry_has_ocr_action() {
    let entry = make_image_entry(false);
    let actions = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions);

    assert!(
        ids.contains(&"clipboard_ocr"),
        "Image entries should have OCR action"
    );

    // macOS image-specific actions
    #[cfg(target_os = "macos")]
    {
        assert!(
            ids.contains(&"clipboard_open_with"),
            "macOS image should have Open With"
        );
        assert!(
            ids.contains(&"clipboard_annotate_cleanshot"),
            "macOS image should have CleanShot annotate"
        );
        assert!(
            ids.contains(&"clipboard_upload_cleanshot"),
            "macOS image should have CleanShot upload"
        );
    }
}

#[test]
fn clipboard_text_entry_lacks_image_specific_actions() {
    let entry = make_text_entry(false);
    let actions = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions);

    assert!(
        !ids.contains(&"clipboard_ocr"),
        "Text entries should NOT have OCR"
    );
    assert!(
        !ids.contains(&"clipboard_open_with"),
        "Text entries should NOT have Open With"
    );
    assert!(
        !ids.contains(&"clipboard_annotate_cleanshot"),
        "Text entries should NOT have CleanShot annotate"
    );
}

#[test]
fn clipboard_pinned_entry_shows_unpin() {
    let entry = make_text_entry(true);
    let actions = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions);

    assert!(
        ids.contains(&"clipboard_unpin"),
        "Pinned entry should show Unpin"
    );
    assert!(
        !ids.contains(&"clipboard_pin"),
        "Pinned entry should NOT show Pin"
    );
}

#[test]
fn clipboard_unpinned_entry_shows_pin() {
    let entry = make_text_entry(false);
    let actions = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions);

    assert!(
        ids.contains(&"clipboard_pin"),
        "Unpinned entry should show Pin"
    );
    assert!(
        !ids.contains(&"clipboard_unpin"),
        "Unpinned entry should NOT show Unpin"
    );
}

// ============================================================
// 4. Chat context actions: model selection, conditional actions
// ============================================================

#[test]
fn chat_context_model_selection_marks_current() {
    let info = ChatPromptInfo {
        current_model: Some("Claude Sonnet".into()),
        available_models: vec![
            ChatModelInfo {
                id: "claude-sonnet".into(),
                display_name: "Claude Sonnet".into(),
                provider: "Anthropic".into(),
            },
            ChatModelInfo {
                id: "gpt-4".into(),
                display_name: "GPT-4".into(),
                provider: "OpenAI".into(),
            },
        ],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);

    let sonnet = find_action(&actions, "select_model_claude-sonnet").unwrap();
    assert!(
        sonnet.title.contains("✓"),
        "Current model should have checkmark"
    );

    let gpt4 = find_action(&actions, "select_model_gpt-4").unwrap();
    assert!(
        !gpt4.title.contains("✓"),
        "Non-current model should not have checkmark"
    );
}

#[test]
fn chat_context_copy_response_only_when_has_response() {
    let with_response = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&with_response);
    let ids = action_ids(&actions);
    assert!(
        ids.contains(&"copy_response"),
        "Should have copy_response when has_response=true"
    );

    let without_response = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&without_response);
    let ids = action_ids(&actions);
    assert!(
        !ids.contains(&"copy_response"),
        "Should NOT have copy_response when has_response=false"
    );
}

#[test]
fn chat_context_clear_only_when_has_messages() {
    let with_msgs = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&with_msgs);
    let ids = action_ids(&actions);
    assert!(
        ids.contains(&"clear_conversation"),
        "Should have clear_conversation when has_messages=true"
    );

    let empty = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&empty);
    let ids = action_ids(&actions);
    assert!(
        !ids.contains(&"clear_conversation"),
        "Should NOT have clear_conversation when has_messages=false"
    );
}

#[test]
fn chat_context_continue_in_chat_always_present() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let ids = action_ids(&actions);
    assert!(
        ids.contains(&"continue_in_chat"),
        "continue_in_chat should always be present"
    );
}

// ============================================================
// 5. to_deeplink_name conversion
// ============================================================

#[test]
fn deeplink_name_lowercase_and_hyphenated() {
    assert_eq!(to_deeplink_name("My Script"), "my-script");
    assert_eq!(to_deeplink_name("Clipboard History"), "clipboard-history");
    assert_eq!(to_deeplink_name("hello_world"), "hello-world");
}

#[test]
fn deeplink_name_strips_special_chars() {
    assert_eq!(to_deeplink_name("hello@world!"), "hello-world");
    assert_eq!(to_deeplink_name("  spaced  out  "), "spaced-out");
    assert_eq!(to_deeplink_name("---dashes---"), "dashes");
}

#[test]
fn deeplink_name_preserves_alphanumeric() {
    assert_eq!(to_deeplink_name("script123"), "script123");
    assert_eq!(to_deeplink_name("ABC"), "abc");
}

// ============================================================
// 6. AI command bar: sections, icons, completeness
// ============================================================

#[test]
fn ai_command_bar_has_all_expected_actions() {
    let actions = get_ai_command_bar_actions();
    let ids = action_ids(&actions);

    let expected = [
        "copy_response",
        "copy_chat",
        "copy_last_code",
        "submit",
        "new_chat",
        "delete_chat",
        "add_attachment",
        "paste_image",
        "change_model",
    ];
    for &expected_id in &expected {
        assert!(
            ids.contains(&expected_id),
            "AI command bar should have {}",
            expected_id
        );
    }
}

#[test]
fn ai_command_bar_actions_have_sections() {
    let actions = get_ai_command_bar_actions();

    for action in &actions {
        assert!(
            action.section.is_some(),
            "AI command bar action '{}' should have a section",
            action.id
        );
    }

    // Verify correct section assignments
    let response_ids = ["copy_response", "copy_chat", "copy_last_code"];
    for id in &response_ids {
        let a = find_action(&actions, id).unwrap();
        assert_eq!(
            a.section.as_deref(),
            Some("Response"),
            "{} should be in Response section",
            id
        );
    }

    let action_ids_list = ["submit", "new_chat", "delete_chat"];
    for id in &action_ids_list {
        let a = find_action(&actions, id).unwrap();
        assert_eq!(
            a.section.as_deref(),
            Some("Actions"),
            "{} should be in Actions section",
            id
        );
    }
}

#[test]
fn ai_command_bar_actions_have_icons() {
    let actions = get_ai_command_bar_actions();

    for action in &actions {
        assert!(
            action.icon.is_some(),
            "AI command bar action '{}' should have an icon",
            action.id
        );
    }
}

// ============================================================
// 7. Notes command bar: conditional actions
// ============================================================

#[test]
fn notes_command_bar_minimal_when_no_selection() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids = action_ids(&actions);

    assert!(
        ids.contains(&"new_note"),
        "new_note should always be present"
    );
    assert!(
        ids.contains(&"browse_notes"),
        "browse_notes should always be present"
    );
    assert!(
        !ids.contains(&"duplicate_note"),
        "duplicate_note requires selection"
    );
    assert!(
        !ids.contains(&"find_in_note"),
        "find_in_note requires selection"
    );
    assert!(!ids.contains(&"export"), "export requires selection");
}

#[test]
fn notes_command_bar_full_when_selected() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids = action_ids(&actions);

    let expected = [
        "new_note",
        "duplicate_note",
        "browse_notes",
        "find_in_note",
        "format",
        "copy_note_as",
        "copy_deeplink",
        "create_quicklink",
        "export",
    ];
    for &expected_id in &expected {
        assert!(
            ids.contains(&expected_id),
            "Notes with selection should have {}",
            expected_id
        );
    }
}

#[test]
fn notes_command_bar_trash_view_suppresses_editing() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids = action_ids(&actions);

    // Trash view with selection should NOT show editing/copy actions
    assert!(
        !ids.contains(&"duplicate_note"),
        "Trash view should not have duplicate_note"
    );
    assert!(
        !ids.contains(&"find_in_note"),
        "Trash view should not have find_in_note"
    );
    assert!(
        !ids.contains(&"export"),
        "Trash view should not have export"
    );

    // But new_note and browse_notes should still be available
    assert!(ids.contains(&"new_note"));
    assert!(ids.contains(&"browse_notes"));
}

#[test]
fn notes_command_bar_auto_sizing_toggle() {
    let disabled = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&disabled);
    let ids = action_ids(&actions);
    assert!(
        ids.contains(&"enable_auto_sizing"),
        "Should show enable_auto_sizing when disabled"
    );

    let enabled = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&enabled);
    let ids = action_ids(&actions);
    assert!(
        !ids.contains(&"enable_auto_sizing"),
        "Should NOT show enable_auto_sizing when already enabled"
    );
}

// ============================================================
// 8. Scriptlet context actions with custom H3 actions
// ============================================================

#[test]
fn scriptlet_context_includes_shortcut_and_alias_dynamics() {
    // Scriptlet with shortcut + alias
    let info = ScriptInfo::scriptlet(
        "Quick Open",
        "/path/to/urls.md#quick-open",
        Some("cmd+o".into()),
        Some("qo".into()),
    );
    let actions = get_scriptlet_context_actions_with_custom(&info, None);
    let ids = action_ids(&actions);

    // Should have update/remove (not add) for both shortcut and alias
    assert!(ids.contains(&"update_shortcut"));
    assert!(ids.contains(&"remove_shortcut"));
    assert!(!ids.contains(&"add_shortcut"));

    assert!(ids.contains(&"update_alias"));
    assert!(ids.contains(&"remove_alias"));
    assert!(!ids.contains(&"add_alias"));

    // Should have scriptlet-specific actions
    assert!(ids.contains(&"edit_scriptlet"));
    assert!(ids.contains(&"reveal_scriptlet_in_finder"));
    assert!(ids.contains(&"copy_scriptlet_path"));
    assert!(ids.contains(&"copy_content"));
    assert!(ids.contains(&"copy_deeplink"));
}

#[test]
fn scriptlet_context_without_shortcut_shows_add() {
    let info = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&info, None);
    let ids = action_ids(&actions);

    assert!(ids.contains(&"add_shortcut"));
    assert!(ids.contains(&"add_alias"));
    assert!(!ids.contains(&"update_shortcut"));
    assert!(!ids.contains(&"remove_shortcut"));
}

// ============================================================
// 9. Action struct builder methods
// ============================================================

#[test]
fn action_with_shortcut_caches_lowercase() {
    let action =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧K");

    assert_eq!(action.shortcut, Some("⌘⇧K".into()));
    assert!(action.shortcut_lower.is_some());
}

#[test]
fn action_with_shortcut_opt_none_no_op() {
    let action =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut_opt(None);

    assert!(action.shortcut.is_none());
    assert!(action.shortcut_lower.is_none());
}

#[test]
fn action_caches_lowercase_title_and_description() {
    let action = Action::new(
        "test",
        "Copy Path",
        Some("Copy the FULL Path".into()),
        ActionCategory::ScriptContext,
    );

    assert_eq!(action.title_lower, "copy path");
    assert_eq!(
        action.description_lower.as_deref(),
        Some("copy the full path")
    );
}

#[test]
fn action_has_action_defaults_to_false() {
    let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
    assert!(
        !action.has_action,
        "Built-in actions should default has_action to false"
    );
}

// ============================================================
// 10. Global actions (currently empty, verify contract)
// ============================================================

#[test]
fn global_actions_are_empty() {
    let actions = get_global_actions();
    assert!(
        actions.is_empty(),
        "Global actions should be empty (Settings/Quit are in main menu)"
    );
}
