
#[test]
fn batch22_clipboard_text_no_ocr() {
    let entry = ClipboardEntryInfo {
        id: "3".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_ocr"));
}

#[test]
fn batch22_clipboard_image_more_actions_than_text() {
    let text_entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let img_entry = ClipboardEntryInfo {
        id: "2".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: String::new(),
        image_dimensions: Some((50, 50)),
        frontmost_app_name: None,
    };
    let t = get_clipboard_history_context_actions(&text_entry).len();
    let i = get_clipboard_history_context_actions(&img_entry).len();
    assert!(
        i > t,
        "Image {} should have more actions than text {}",
        i,
        t
    );
}

// ============================================================
// 11. Clipboard context: pin/unpin toggle based on pinned state
// ============================================================

#[test]
fn batch22_clipboard_unpinned_shows_pin() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_pin"));
    assert!(!actions.iter().any(|a| a.id == "clipboard_unpin"));
}

#[test]
fn batch22_clipboard_pinned_shows_unpin() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_unpin"));
    assert!(!actions.iter().any(|a| a.id == "clipboard_pin"));
}

#[test]
fn batch22_clipboard_pin_unpin_same_shortcut() {
    let pinned_entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let unpinned_entry = ClipboardEntryInfo {
        id: "2".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let pa = get_clipboard_history_context_actions(&pinned_entry);
    let ua = get_clipboard_history_context_actions(&unpinned_entry);
    let unpin_shortcut = pa
        .iter()
        .find(|a| a.id == "clipboard_unpin")
        .unwrap()
        .shortcut
        .as_deref();
    let pin_shortcut = ua
        .iter()
        .find(|a| a.id == "clipboard_pin")
        .unwrap()
        .shortcut
        .as_deref();
    assert_eq!(unpin_shortcut, pin_shortcut);
    assert_eq!(pin_shortcut, Some("⇧⌘P"));
}

// ============================================================
// 12. Chat context: model count affects total action count
// ============================================================

#[test]
fn batch22_chat_zero_models_no_flags() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    // Just continue_in_chat
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "continue_in_chat");
}

#[test]
fn batch22_chat_two_models_both_flags() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".into()),
        available_models: vec![
            ChatModelInfo {
                id: "gpt4".into(),
                display_name: "GPT-4".into(),
                provider: "OpenAI".into(),
            },
            ChatModelInfo {
                id: "claude".into(),
                display_name: "Claude".into(),
                provider: "Anthropic".into(),
            },
        ],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    // 2 models + continue + copy_response + clear_conversation = 5
    assert_eq!(actions.len(), 5);
}

#[test]
fn batch22_chat_current_model_checkmark() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".into()),
        available_models: vec![ChatModelInfo {
            id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model_action = actions
        .iter()
        .find(|a| a.id == "select_model_gpt4")
        .unwrap();
    assert!(model_action.title.contains('✓'));
}

#[test]
fn batch22_chat_non_current_model_no_checkmark() {
    let info = ChatPromptInfo {
        current_model: Some("Other".into()),
        available_models: vec![ChatModelInfo {
            id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model_action = actions
        .iter()
        .find(|a| a.id == "select_model_gpt4")
        .unwrap();
    assert!(!model_action.title.contains('✓'));
}

// ============================================================
// 13. AI command bar: every action has an icon
// ============================================================

#[test]
fn batch22_ai_command_bar_all_have_icons() {
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        assert!(
            action.icon.is_some(),
            "AI action '{}' should have an icon",
            action.id
        );
    }
}

#[test]
fn batch22_ai_command_bar_all_have_sections() {
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        assert!(
            action.section.is_some(),
            "AI action '{}' should have a section",
            action.id
        );
    }
}

#[test]
fn batch22_ai_command_bar_total_is_12() {
    let actions = get_ai_command_bar_actions();
    assert_eq!(actions.len(), 12);
}

#[test]
fn batch22_ai_export_markdown_icon_is_filecode() {
    let actions = get_ai_command_bar_actions();
    let export = actions.iter().find(|a| a.id == "export_markdown").unwrap();
    assert_eq!(export.icon, Some(IconName::FileCode));
}

// ============================================================
// 14. Notes command bar: trash mode removes most actions
// ============================================================

#[test]
fn batch22_notes_trash_mode_minimal() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // Trash mode: new_note, browse_notes, enable_auto_sizing = 3
    assert_eq!(actions.len(), 3);
}

#[test]
fn batch22_notes_full_mode_max_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // Full: new+dup+browse+find+format+copy_note_as+copy_deeplink+create_quicklink+export+auto_sizing = 10
    assert_eq!(actions.len(), 10);
}

#[test]
fn batch22_notes_auto_sizing_enabled_removes_one() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    // Same as full minus enable_auto_sizing = 9
    assert_eq!(actions.len(), 9);
}

#[test]
fn batch22_notes_no_selection_no_trash() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note + browse_notes + enable_auto_sizing = 3
    assert_eq!(actions.len(), 3);
}

// ============================================================
// 15. New chat actions: section assignment and ID patterns
// ============================================================

#[test]
fn batch22_new_chat_last_used_section() {
    let last_used = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "OpenAI".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
}

#[test]
fn batch22_new_chat_preset_section() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].section.as_deref(), Some("Presets"));
}

#[test]
fn batch22_new_chat_model_section() {
    let models = vec![NewChatModelInfo {
        model_id: "claude".into(),
        display_name: "Claude".into(),
        provider: "Anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].section.as_deref(), Some("Models"));
}

#[test]
fn batch22_new_chat_id_patterns() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "M1".into(),
        provider: "P".into(),
        provider_display_name: "P".into(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "code".into(),
        name: "Code".into(),
        icon: IconName::Code,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "m2".into(),
        display_name: "M2".into(),
        provider: "P".into(),
        provider_display_name: "P".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &presets, &models);
    assert_eq!(actions[0].id, "last_used_0");
    assert_eq!(actions[1].id, "preset_code");
    assert_eq!(actions[2].id, "model_0");
}

#[test]
fn batch22_new_chat_empty_all_returns_empty() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

// ============================================================
// 16. Note switcher: icon priority hierarchy
// ============================================================

#[test]
fn batch22_note_switcher_pinned_icon_starfilled() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 10,
        is_current: false,
        is_pinned: true,
        preview: "p".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn batch22_note_switcher_current_icon_check() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 10,
        is_current: true,
        is_pinned: false,
        preview: "p".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::Check));
}

#[test]
fn batch22_note_switcher_regular_icon_file() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "p".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::File));
}

#[test]
fn batch22_note_switcher_pinned_trumps_current() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 10,
        is_current: true,
        is_pinned: true,
        preview: "p".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

// ============================================================
// 17. Note switcher: description format (preview+time, char count)
// ============================================================

#[test]
fn batch22_note_switcher_preview_plus_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "Hello world".into(),
        relative_time: "2m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(desc.contains("Hello world"));
    assert!(desc.contains("2m ago"));
    assert!(desc.contains(" · "));
}
