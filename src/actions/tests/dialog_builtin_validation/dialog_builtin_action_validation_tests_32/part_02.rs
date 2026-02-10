
#[test]
fn batch32_chat_single_model_both_flags_produces_4_actions() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions.len(), 4, "1 model + continue + copy + clear = 4");
}

#[test]
fn batch32_chat_single_model_title_matches_display_name() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions[0].title, "GPT-4");
}

// ---------------------------------------------------------------------------
// 11. Chat context: has_response=true without has_messages
// ---------------------------------------------------------------------------

#[test]
fn batch32_chat_has_response_no_messages_has_copy_no_clear() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "copy_response"));
    assert!(!actions.iter().any(|a| a.id == "clear_conversation"));
}

#[test]
fn batch32_chat_has_messages_no_response_has_clear_no_copy() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "copy_response"));
    assert!(actions.iter().any(|a| a.id == "clear_conversation"));
}

#[test]
fn batch32_chat_no_flags_only_continue() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "continue_in_chat");
}

// ---------------------------------------------------------------------------
// 12. Notes command bar: find_in_note details
// ---------------------------------------------------------------------------

#[test]
fn batch32_notes_find_in_note_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let fin = actions.iter().find(|a| a.id == "find_in_note").unwrap();
    assert_eq!(fin.shortcut.as_deref(), Some("⌘F"));
}

#[test]
fn batch32_notes_find_in_note_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let fin = actions.iter().find(|a| a.id == "find_in_note").unwrap();
    assert_eq!(fin.icon, Some(IconName::MagnifyingGlass));
}

#[test]
fn batch32_notes_find_in_note_section() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let fin = actions.iter().find(|a| a.id == "find_in_note").unwrap();
    assert_eq!(fin.section.as_deref(), Some("Edit"));
}

#[test]
fn batch32_notes_find_in_note_absent_without_selection() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "find_in_note"));
}

// ---------------------------------------------------------------------------
// 13. Notes: trash view blocks all selection-dependent actions
// ---------------------------------------------------------------------------

#[test]
fn batch32_notes_trash_no_duplicate_note() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
}

#[test]
fn batch32_notes_trash_no_find_in_note() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "find_in_note"));
}

#[test]
fn batch32_notes_trash_no_format() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "format"));
}

#[test]
fn batch32_notes_trash_no_export() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "export"));
}

// ---------------------------------------------------------------------------
// 14. Note switcher: preview truncation with trim_end on > 60 chars
// ---------------------------------------------------------------------------

#[test]
fn batch32_note_switcher_61_char_preview_truncated_with_ellipsis() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "T".into(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "a".repeat(61),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(
        desc.ends_with('…'),
        "61-char preview should end with …, got: {}",
        desc
    );
}

#[test]
fn batch32_note_switcher_60_char_preview_no_truncation() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "T".into(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "b".repeat(60),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(
        !desc.ends_with('…'),
        "60-char preview should not be truncated, got: {}",
        desc
    );
    assert_eq!(desc.len(), 60);
}

#[test]
fn batch32_note_switcher_short_preview_no_truncation() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "T".into(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "hello world".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert_eq!(desc, "hello world");
}

// ---------------------------------------------------------------------------
// 15. Note switcher: title without current indicator has no bullet
// ---------------------------------------------------------------------------

#[test]
fn batch32_note_switcher_non_current_no_bullet() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "My Note".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].title, "My Note");
}

#[test]
fn batch32_note_switcher_current_has_bullet() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "My Note".into(),
        char_count: 10,
        is_current: true,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].title, "• My Note");
}

#[test]
fn batch32_note_switcher_current_pinned_icon_star_filled() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "My Note".into(),
        char_count: 10,
        is_current: true,
        is_pinned: true,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

// ---------------------------------------------------------------------------
// 16. New chat: last_used icon is always BoltFilled
// ---------------------------------------------------------------------------

#[test]
fn batch32_new_chat_last_used_icon_bolt_filled() {
    let last_used = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
}

#[test]
fn batch32_new_chat_last_used_section_is_last_used_settings() {
    let last_used = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
}

#[test]
fn batch32_new_chat_last_used_desc_is_provider_display_name() {
    let last_used = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].description.as_deref(), Some("OpenAI"));
}

// ---------------------------------------------------------------------------
// 17. New chat: model section always "Models" with Settings icon
// ---------------------------------------------------------------------------

#[test]
fn batch32_new_chat_model_section_is_models() {
    let models = vec![NewChatModelInfo {
        model_id: "claude".into(),
        display_name: "Claude".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].section.as_deref(), Some("Models"));
}

#[test]
fn batch32_new_chat_model_icon_is_settings() {
    let models = vec![NewChatModelInfo {
        model_id: "claude".into(),
        display_name: "Claude".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].icon, Some(IconName::Settings));
}

#[test]
fn batch32_new_chat_model_id_format() {
    let models = vec![NewChatModelInfo {
        model_id: "claude".into(),
        display_name: "Claude".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].id, "model_0");
}

#[test]
fn batch32_new_chat_preset_section_is_presets() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].section.as_deref(), Some("Presets"));
}

// ---------------------------------------------------------------------------
// 18. to_deeplink_name: tab, newline, and numbers-only input
// ---------------------------------------------------------------------------

#[test]
fn batch32_to_deeplink_name_tab_and_newline() {
    assert_eq!(to_deeplink_name("test\ttab\nnewline"), "test-tab-newline");
}

#[test]
fn batch32_to_deeplink_name_numbers_only() {
    assert_eq!(to_deeplink_name("12345"), "12345");
}

#[test]
fn batch32_to_deeplink_name_leading_trailing_hyphens() {
    assert_eq!(to_deeplink_name("--hello--"), "hello");
}

#[test]
fn batch32_to_deeplink_name_single_word() {
    assert_eq!(to_deeplink_name("hello"), "hello");
}

// ---------------------------------------------------------------------------
// 19. format_shortcut_hint (on ActionsDialog): key conversions
// ---------------------------------------------------------------------------

#[test]
fn batch32_format_shortcut_hint_cmd_e() {
    let result = ActionsDialog::format_shortcut_hint("cmd+e");
    assert_eq!(result, "⌘E");
}

#[test]
fn batch32_format_shortcut_hint_all_modifiers() {
    let result = ActionsDialog::format_shortcut_hint("cmd+shift+ctrl+alt+k");
    assert_eq!(result, "⌘⇧⌃⌥K");
}

#[test]
fn batch32_format_shortcut_hint_enter_alone() {
    let result = ActionsDialog::format_shortcut_hint("enter");
    assert_eq!(result, "↵");
}

#[test]
fn batch32_format_shortcut_hint_meta_alias() {
    let result = ActionsDialog::format_shortcut_hint("meta+c");
    assert_eq!(result, "⌘C");
}

// ---------------------------------------------------------------------------
// 20. parse_shortcut_keycaps: various inputs
// ---------------------------------------------------------------------------

#[test]
fn batch32_parse_shortcut_keycaps_single_letter() {
    let caps = ActionsDialog::parse_shortcut_keycaps("E");
    assert_eq!(caps, vec!["E"]);
}

#[test]
fn batch32_parse_shortcut_keycaps_cmd_enter() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘↵");
    assert_eq!(caps, vec!["⌘", "↵"]);
}

#[test]
fn batch32_parse_shortcut_keycaps_slash() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘/");
    assert_eq!(caps, vec!["⌘", "/"]);
}
