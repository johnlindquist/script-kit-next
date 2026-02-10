
#[test]
fn notes_duplicate_note_absent_in_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
}

#[test]
fn notes_duplicate_note_absent_without_selection() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
}

// =========== 20. Notes: copy_note_as details ===========

#[test]
fn notes_copy_note_as_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let cna = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
    assert_eq!(cna.shortcut, Some("⇧⌘C".to_string()));
}

#[test]
fn notes_copy_note_as_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let cna = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
    assert_eq!(cna.icon, Some(IconName::Copy));
}

#[test]
fn notes_copy_note_as_section() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let cna = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
    assert_eq!(cna.section, Some("Copy".to_string()));
}

#[test]
fn notes_copy_note_as_absent_in_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "copy_note_as"));
}

// =========== 21. Notes: total action count varies by state ===========

#[test]
fn notes_full_selection_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note + duplicate + browse + find + format + copy_note_as + copy_deeplink + create_quicklink + export + enable_auto_sizing = 10
    assert_eq!(actions.len(), 10);
}

#[test]
fn notes_no_selection_count() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note + browse + enable_auto_sizing = 3
    assert_eq!(actions.len(), 3);
}

#[test]
fn notes_trash_selection_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note + browse + enable_auto_sizing = 3
    assert_eq!(actions.len(), 3);
}

#[test]
fn notes_full_selection_auto_sizing_enabled_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    // 10 minus enable_auto_sizing = 9
    assert_eq!(actions.len(), 9);
}

// =========== 22. Chat context: no models produces only continue_in_chat ===========

#[test]
fn chat_no_models_no_messages_single_action() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions.len(), 1);
}

#[test]
fn chat_no_models_single_is_continue() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions[0].id, "continue_in_chat");
}

#[test]
fn chat_with_messages_adds_clear() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions.len(), 2);
}

#[test]
fn chat_with_response_adds_copy() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions.len(), 2);
}

// =========== 23. Chat context: model IDs use select_model_{model.id} ===========

#[test]
fn chat_model_id_format() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "claude-3-opus".into(),
            display_name: "Claude 3 Opus".into(),
            provider: "Anthropic".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions[0].id, "select_model_claude-3-opus");
}

#[test]
fn chat_model_title_is_display_name() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "gpt-4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions[0].title, "GPT-4");
}

#[test]
fn chat_model_desc_via_provider() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "gpt-4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions[0].description, Some("via OpenAI".to_string()));
}

#[test]
fn chat_current_model_gets_checkmark() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".to_string()),
        available_models: vec![ChatModelInfo {
            id: "gpt-4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions[0].title.contains('✓'));
}

// =========== 24. New chat: last_used section and icon ===========

#[test]
fn new_chat_last_used_section() {
    let lu = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "p".into(),
        provider_display_name: "Provider 1".into(),
    }];
    let actions = get_new_chat_actions(&lu, &[], &[]);
    assert_eq!(actions[0].section, Some("Last Used Settings".to_string()));
}

#[test]
fn new_chat_last_used_icon_bolt() {
    let lu = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "p".into(),
        provider_display_name: "Provider 1".into(),
    }];
    let actions = get_new_chat_actions(&lu, &[], &[]);
    assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
}

#[test]
fn new_chat_last_used_desc_is_provider() {
    let lu = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "p".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&lu, &[], &[]);
    assert_eq!(actions[0].description, Some("Anthropic".to_string()));
}

#[test]
fn new_chat_last_used_id_format() {
    let lu = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "p".into(),
        provider_display_name: "P".into(),
    }];
    let actions = get_new_chat_actions(&lu, &[], &[]);
    assert_eq!(actions[0].id, "last_used_0");
}

// =========== 25. New chat: preset section and icon ===========

#[test]
fn new_chat_preset_section() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].section, Some("Presets".to_string()));
}

#[test]
fn new_chat_preset_icon_preserved() {
    let presets = vec![NewChatPresetInfo {
        id: "code".into(),
        name: "Code".into(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].icon, Some(IconName::Code));
}

#[test]
fn new_chat_preset_id_format() {
    let presets = vec![NewChatPresetInfo {
        id: "writer".into(),
        name: "Writer".into(),
        icon: IconName::File,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].id, "preset_writer");
}

#[test]
fn new_chat_preset_desc_none() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert!(actions[0].description.is_none());
}

// =========== 26. Note switcher: current note has bullet prefix ===========

#[test]
fn note_switcher_current_has_bullet() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "My Note".into(),
        char_count: 42,
        is_current: true,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(actions[0].title.starts_with("• "));
}

#[test]
fn note_switcher_non_current_no_bullet() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "My Note".into(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(!actions[0].title.starts_with("• "));
}

#[test]
fn note_switcher_current_icon_check() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "My Note".into(),
        char_count: 42,
        is_current: true,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::Check));
}

#[test]
fn note_switcher_pinned_current_icon_star() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "My Note".into(),
        char_count: 42,
        is_current: true,
        is_pinned: true,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    // pinned takes priority over current for icon
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

// =========== 27. Note switcher: preview truncation at 60 chars ===========

#[test]
fn note_switcher_short_preview_not_truncated() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "T".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "Short preview".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description, Some("Short preview".to_string()));
}

#[test]
fn note_switcher_long_preview_truncated_with_ellipsis() {
    let long_preview = "a".repeat(80);
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "T".into(),
        char_count: 80,
        is_current: false,
        is_pinned: false,
        preview: long_preview,
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(desc.ends_with('…'));
    // 60 'a's + ellipsis
    assert_eq!(desc.chars().count(), 61);
}

#[test]
fn note_switcher_preview_with_time_separator() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "T".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "Hello world".into(),
        relative_time: "2m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(desc.contains(" · "));
    assert!(desc.contains("2m ago"));
}
