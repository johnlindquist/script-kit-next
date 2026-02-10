
#[test]
fn batch21_chat_copy_response_shortcut() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let a = actions.iter().find(|a| a.id == "copy_response").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘C"));
}

// ============================================================
// 10. Chat context: continue_in_chat always after models
// ============================================================

#[test]
fn batch21_chat_continue_after_models() {
    let info = ChatPromptInfo {
        current_model: Some("gpt-4".into()),
        available_models: vec![
            ChatModelInfo {
                id: "gpt-4".into(),
                display_name: "GPT-4".into(),
                provider: "OpenAI".into(),
            },
            ChatModelInfo {
                id: "claude".into(),
                display_name: "Claude".into(),
                provider: "Anthropic".into(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model_last_pos = actions
        .iter()
        .rposition(|a| a.id.starts_with("select_model_"))
        .unwrap();
    let continue_pos = actions
        .iter()
        .position(|a| a.id == "continue_in_chat")
        .unwrap();
    assert!(continue_pos > model_last_pos);
}

#[test]
fn batch21_chat_continue_present_zero_models() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "continue_in_chat"));
}

#[test]
fn batch21_chat_continue_shortcut() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let a = actions.iter().find(|a| a.id == "continue_in_chat").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘↵"));
}

// ============================================================
// 11. Notes command bar: copy section actions
// ============================================================

#[test]
fn batch21_notes_copy_deeplink_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let a = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⇧⌘D"));
}

#[test]
fn batch21_notes_copy_deeplink_section_copy() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let a = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert_eq!(a.section.as_deref(), Some("Copy"));
}

#[test]
fn batch21_notes_create_quicklink_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let a = actions.iter().find(|a| a.id == "create_quicklink").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⇧⌘L"));
}

#[test]
fn batch21_notes_create_quicklink_icon_star() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let a = actions.iter().find(|a| a.id == "create_quicklink").unwrap();
    assert_eq!(a.icon, Some(IconName::Star));
}

#[test]
fn batch21_notes_copy_note_as_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let a = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⇧⌘C"));
}

// ============================================================
// 12. Notes command bar: enable_auto_sizing conditional
// ============================================================

#[test]
fn batch21_notes_auto_sizing_absent_when_enabled() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "enable_auto_sizing"));
}

#[test]
fn batch21_notes_auto_sizing_present_when_disabled() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "enable_auto_sizing"));
}

#[test]
fn batch21_notes_auto_sizing_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let a = actions
        .iter()
        .find(|a| a.id == "enable_auto_sizing")
        .unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘A"));
}

#[test]
fn batch21_notes_auto_sizing_section_settings() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let a = actions
        .iter()
        .find(|a| a.id == "enable_auto_sizing")
        .unwrap();
    assert_eq!(a.section.as_deref(), Some("Settings"));
}

// ============================================================
// 13. Note switcher: relative_time propagation
// ============================================================

#[test]
fn batch21_note_switcher_preview_and_time_joined() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 50,
        is_current: false,
        is_pinned: false,
        preview: "Hello world".into(),
        relative_time: "2m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_deref().unwrap();
    assert!(desc.contains("Hello world"));
    assert!(desc.contains("2m ago"));
    assert!(desc.contains(" · "));
}

#[test]
fn batch21_note_switcher_no_time_no_separator() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "Some text".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_deref().unwrap();
    assert_eq!(desc, "Some text");
    assert!(!desc.contains(" · "));
}

#[test]
fn batch21_note_switcher_no_preview_with_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 5,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "1h ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_deref().unwrap();
    assert_eq!(desc, "1h ago");
}

#[test]
fn batch21_note_switcher_no_preview_no_time_char_count() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_deref().unwrap();
    assert_eq!(desc, "42 chars");
}

#[test]
fn batch21_note_switcher_singular_char() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 1,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_deref().unwrap();
    assert_eq!(desc, "1 char");
}

// ============================================================
// 14. New chat actions: ID format patterns
// ============================================================

#[test]
fn batch21_new_chat_last_used_id_format() {
    let last_used = vec![NewChatModelInfo {
        model_id: "gpt-4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].id, "last_used_0");
}

#[test]
fn batch21_new_chat_preset_id_format() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].id, "preset_general");
}

#[test]
fn batch21_new_chat_model_id_format() {
    let models = vec![NewChatModelInfo {
        model_id: "claude-3".into(),
        display_name: "Claude 3".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].id, "model_0");
}

#[test]
fn batch21_new_chat_multiple_last_used_sequential_ids() {
    let last_used = vec![
        NewChatModelInfo {
            model_id: "a".into(),
            display_name: "A".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        },
        NewChatModelInfo {
            model_id: "b".into(),
            display_name: "B".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        },
    ];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].id, "last_used_0");
    assert_eq!(actions[1].id, "last_used_1");
}

#[test]
fn batch21_new_chat_empty_all_empty_result() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

// ============================================================
// 15. Clipboard context: clipboard_copy description
// ============================================================

#[test]
fn batch21_clipboard_copy_description_mentions_clipboard() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let a = actions.iter().find(|a| a.id == "clipboard_copy").unwrap();
    assert!(a
        .description
        .as_deref()
        .unwrap()
        .to_lowercase()
        .contains("clipboard"));
}

#[test]
fn batch21_clipboard_paste_description_mentions_clipboard() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let a = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
    assert!(a
        .description
        .as_deref()
        .unwrap()
        .to_lowercase()
        .contains("clipboard"));
}

#[test]
fn batch21_clipboard_paste_keep_open_desc_mentions_keep() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let a = actions
        .iter()
        .find(|a| a.id == "clipboard_paste_keep_open")
        .unwrap();
    assert!(a
        .description
        .as_deref()
        .unwrap()
        .to_lowercase()
        .contains("keep"));
}

#[test]
fn batch21_clipboard_delete_all_desc_mentions_pinned() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let a = actions
        .iter()
        .find(|a| a.id == "clipboard_delete_all")
        .unwrap();
    assert!(a
        .description
        .as_deref()
        .unwrap()
        .to_lowercase()
        .contains("pinned"));
}
