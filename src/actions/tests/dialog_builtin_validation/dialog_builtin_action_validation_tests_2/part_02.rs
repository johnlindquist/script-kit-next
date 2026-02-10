
#[test]
fn notes_format_only_when_selected_and_not_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions_1 = get_notes_command_bar_actions(&info);
    let ids = action_ids(&actions_1);
    assert!(ids.contains(&"format"));

    let info_trash = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions_2 = get_notes_command_bar_actions(&info_trash);
    let ids_trash = action_ids(&actions_2);
    assert!(!ids_trash.contains(&"format"));
}

#[test]
fn notes_copy_section_only_when_selected_and_not_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions_1 = get_notes_command_bar_actions(&info);
    let ids = action_ids(&actions_1);
    assert!(ids.contains(&"copy_note_as"));
    assert!(ids.contains(&"copy_deeplink"));
    assert!(ids.contains(&"create_quicklink"));

    let info_no_sel = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions_2 = get_notes_command_bar_actions(&info_no_sel);
    let ids_no_sel = action_ids(&actions_2);
    assert!(!ids_no_sel.contains(&"copy_note_as"));
    assert!(!ids_no_sel.contains(&"copy_deeplink"));
    assert!(!ids_no_sel.contains(&"create_quicklink"));
}

#[test]
fn notes_export_only_when_selected_and_not_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions_1 = get_notes_command_bar_actions(&info);
    let ids = action_ids(&actions_1);
    assert!(ids.contains(&"export"));

    let info_no_sel = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions_2 = get_notes_command_bar_actions(&info_no_sel);
    let ids_no_sel = action_ids(&actions_2);
    assert!(!ids_no_sel.contains(&"export"));
}

// =========================================================================
// 8. Notes auto-sizing toggle — only when disabled
// =========================================================================

#[test]
fn notes_auto_sizing_only_when_disabled() {
    let info_disabled = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions_1 = get_notes_command_bar_actions(&info_disabled);
    let ids_disabled = action_ids(&actions_1);
    assert!(
        ids_disabled.contains(&"enable_auto_sizing"),
        "Should show enable_auto_sizing when disabled"
    );

    let info_enabled = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions_2 = get_notes_command_bar_actions(&info_enabled);
    let ids_enabled = action_ids(&actions_2);
    assert!(
        !ids_enabled.contains(&"enable_auto_sizing"),
        "Should NOT show enable_auto_sizing when already enabled"
    );
}

// =========================================================================
// 9. Chat conditional actions — copy_response / clear_conversation
// =========================================================================

#[test]
fn chat_copy_response_only_when_has_response() {
    let with_response = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions_1 = get_chat_context_actions(&with_response);
    let ids = action_ids(&actions_1);
    assert!(ids.contains(&"copy_response"));

    let without_response = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions_2 = get_chat_context_actions(&without_response);
    let ids_no = action_ids(&actions_2);
    assert!(!ids_no.contains(&"copy_response"));
}

#[test]
fn chat_clear_conversation_only_when_has_messages() {
    let with_messages = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions_1 = get_chat_context_actions(&with_messages);
    let ids = action_ids(&actions_1);
    assert!(ids.contains(&"clear_conversation"));

    let without_messages = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions_2 = get_chat_context_actions(&without_messages);
    let ids_no = action_ids(&actions_2);
    assert!(!ids_no.contains(&"clear_conversation"));
}

#[test]
fn chat_empty_models_still_has_continue_in_chat() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(
        actions.len(),
        1,
        "Empty chat should have only continue_in_chat"
    );
    assert_eq!(actions[0].id, "continue_in_chat");
}

#[test]
fn chat_full_context_has_all_actions() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".into()),
        available_models: vec![ChatModelInfo {
            id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: true,
        has_response: true,
    };
    let actions_tmp = get_chat_context_actions(&info);
    let ids = action_ids(&actions_tmp);
    assert!(ids.contains(&"select_model_gpt4"));
    assert!(ids.contains(&"continue_in_chat"));
    assert!(ids.contains(&"copy_response"));
    assert!(ids.contains(&"clear_conversation"));
}

// =========================================================================
// 10. Clipboard content-type-specific actions
// =========================================================================

#[test]
fn clipboard_image_has_ocr_action() {
    let entry = ClipboardEntryInfo {
        id: "img1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions_tmp = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions_tmp);
    assert!(
        ids.contains(&"clipboard_ocr"),
        "Image entry should have OCR action"
    );
}

#[test]
fn clipboard_text_has_no_ocr_action() {
    let entry = ClipboardEntryInfo {
        id: "txt1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions_tmp = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions_tmp);
    assert!(
        !ids.contains(&"clipboard_ocr"),
        "Text entry should NOT have OCR action"
    );
}

#[test]
fn clipboard_image_has_more_actions_than_text() {
    let text_entry = ClipboardEntryInfo {
        id: "t".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let image_entry = ClipboardEntryInfo {
        id: "i".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "".into(),
        image_dimensions: Some((10, 10)),
        frontmost_app_name: None,
    };
    let text_count = get_clipboard_history_context_actions(&text_entry).len();
    let image_count = get_clipboard_history_context_actions(&image_entry).len();
    assert!(
        image_count > text_count,
        "Image ({}) should have more actions than text ({})",
        image_count,
        text_count
    );
}

// =========================================================================
// 11. Clipboard pin/unpin mutual exclusivity
// =========================================================================

#[test]
fn clipboard_pinned_has_unpin_not_pin() {
    let entry = ClipboardEntryInfo {
        id: "p1".into(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "pinned".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions_tmp = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions_tmp);
    assert!(
        ids.contains(&"clipboard_unpin"),
        "Pinned entry should have unpin"
    );
    assert!(
        !ids.contains(&"clipboard_pin"),
        "Pinned entry should NOT have pin"
    );
}

#[test]
fn clipboard_unpinned_has_pin_not_unpin() {
    let entry = ClipboardEntryInfo {
        id: "u1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "unpinned".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions_tmp = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions_tmp);
    assert!(
        ids.contains(&"clipboard_pin"),
        "Unpinned entry should have pin"
    );
    assert!(
        !ids.contains(&"clipboard_unpin"),
        "Unpinned entry should NOT have unpin"
    );
}

// =========================================================================
// 12. Clipboard frontmost app name in paste title
// =========================================================================

#[test]
fn clipboard_paste_title_includes_app_name() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: Some("VS Code".into()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = find_action(&actions, "clipboard_paste").unwrap();
    assert!(
        paste.title.contains("VS Code"),
        "Paste title should include app name, got '{}'",
        paste.title
    );
}

#[test]
fn clipboard_paste_title_fallback_when_no_app() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = find_action(&actions, "clipboard_paste").unwrap();
    assert!(
        paste.title.contains("Active App"),
        "Paste title should say 'Active App' as fallback, got '{}'",
        paste.title
    );
}

// =========================================================================
// 13. Path context — is_dir differences
// =========================================================================

#[test]
fn path_directory_primary_is_open_directory() {
    let path = PathInfo::new("my-dir", "/my-dir", true);
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].id, "open_directory");
    assert!(actions[0].title.contains("my-dir"));
}

#[test]
fn path_file_primary_is_select_file() {
    let path = PathInfo::new("file.txt", "/file.txt", false);
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].id, "select_file");
    assert!(actions[0].title.contains("file.txt"));
}

#[test]
fn path_trash_description_varies_by_is_dir() {
    let dir_path = PathInfo::new("dir", "/dir", true);
    let dir_actions = get_path_context_actions(&dir_path);
    let dir_trash = find_action(&dir_actions, "move_to_trash").unwrap();
    assert!(
        dir_trash.description.as_ref().unwrap().contains("folder"),
        "Directory trash description should mention 'folder', got '{:?}'",
        dir_trash.description
    );

    let file_path = PathInfo::new("file.txt", "/file.txt", false);
    let file_actions = get_path_context_actions(&file_path);
    let file_trash = find_action(&file_actions, "move_to_trash").unwrap();
    assert!(
        file_trash.description.as_ref().unwrap().contains("file"),
        "File trash description should mention 'file', got '{:?}'",
        file_trash.description
    );
}

// =========================================================================
// 14. File context — is_dir differences
// =========================================================================

#[test]
fn file_directory_primary_is_open_directory() {
    let dir = FileInfo {
        path: "/my-dir".into(),
        name: "my-dir".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&dir);
    assert_eq!(actions[0].id, "open_directory");
}

#[test]
fn file_non_directory_primary_is_open_file() {
    let file = FileInfo {
        path: "/test.rs".into(),
        name: "test.rs".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    assert_eq!(actions[0].id, "open_file");
}

#[test]
fn file_directory_has_no_quick_look() {
    let dir = FileInfo {
        path: "/my-dir".into(),
        name: "my-dir".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions_tmp = get_file_context_actions(&dir);
    let ids = action_ids(&actions_tmp);
    assert!(
        !ids.contains(&"quick_look"),
        "Directories should not have quick_look"
    );
}

// =========================================================================
// 15. Agent-specific action invariants
// =========================================================================

#[test]
fn agent_has_edit_with_agent_title() {
    let mut agent = ScriptInfo::new("my-agent", "/path/agent.md");
    agent.is_agent = true;
    agent.is_script = false;
    let actions = get_script_context_actions(&agent);
    let edit = find_action(&actions, "edit_script").unwrap();
    assert!(
        edit.title.contains("Agent"),
        "Agent edit action should say 'Agent', got '{}'",
        edit.title
    );
}
