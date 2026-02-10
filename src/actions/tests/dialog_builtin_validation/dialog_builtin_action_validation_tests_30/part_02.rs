
// ---------------------------------------------------------------------------
// 9. Chat context: current model gets ✓ suffix
// ---------------------------------------------------------------------------
#[test]
fn batch30_chat_current_model_has_check() {
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
    let m = actions
        .iter()
        .find(|a| a.id == "select_model_gpt4")
        .unwrap();
    assert!(m.title.contains("✓"), "Current model title should have ✓");
}

#[test]
fn batch30_chat_non_current_model_no_check() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".into()),
        available_models: vec![ChatModelInfo {
            id: "claude".into(),
            display_name: "Claude".into(),
            provider: "Anthropic".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let m = actions
        .iter()
        .find(|a| a.id == "select_model_claude")
        .unwrap();
    assert!(!m.title.contains("✓"));
}

#[test]
fn batch30_chat_no_current_model_no_check() {
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
    let m = actions
        .iter()
        .find(|a| a.id == "select_model_gpt4")
        .unwrap();
    assert!(!m.title.contains("✓"));
}

#[test]
fn batch30_chat_model_desc_says_via_provider() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "c3".into(),
            display_name: "Claude 3".into(),
            provider: "Anthropic".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let m = actions.iter().find(|a| a.id == "select_model_c3").unwrap();
    assert_eq!(m.description.as_deref(), Some("via Anthropic"));
}

// ---------------------------------------------------------------------------
// 10. Notes command bar: new_note always present
// ---------------------------------------------------------------------------
#[test]
fn batch30_notes_new_note_present_full_mode() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "new_note"));
}

#[test]
fn batch30_notes_new_note_present_in_trash() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "new_note"));
}

#[test]
fn batch30_notes_new_note_shortcut_cmd_n() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let nn = actions.iter().find(|a| a.id == "new_note").unwrap();
    assert_eq!(nn.shortcut.as_deref(), Some("⌘N"));
}

#[test]
fn batch30_notes_new_note_icon_plus() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let nn = actions.iter().find(|a| a.id == "new_note").unwrap();
    assert_eq!(nn.icon, Some(IconName::Plus));
}

// ---------------------------------------------------------------------------
// 11. Notes command bar: full mode action count
// ---------------------------------------------------------------------------
#[test]
fn batch30_notes_full_mode_10_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note, duplicate_note, browse_notes, find_in_note, format,
    // copy_note_as, copy_deeplink, create_quicklink, export, enable_auto_sizing = 10
    assert_eq!(actions.len(), 10);
}

#[test]
fn batch30_notes_full_mode_auto_sizing_enabled_9_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    // same minus enable_auto_sizing = 9
    assert_eq!(actions.len(), 9);
}

#[test]
fn batch30_notes_no_selection_3_actions() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note, browse_notes, enable_auto_sizing = 3
    assert_eq!(actions.len(), 3);
}

#[test]
fn batch30_notes_trash_3_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note, browse_notes, enable_auto_sizing = 3
    assert_eq!(actions.len(), 3);
}

// ---------------------------------------------------------------------------
// 12. Note switcher: pinned note gets StarFilled icon
// ---------------------------------------------------------------------------
#[test]
fn batch30_note_switcher_pinned_icon_star_filled() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Pinned Note".into(),
        char_count: 50,
        is_current: false,
        is_pinned: true,
        preview: "Some preview".into(),
        relative_time: "1h ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn batch30_note_switcher_pinned_section() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "P".into(),
        char_count: 10,
        is_current: false,
        is_pinned: true,
        preview: "x".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
}

#[test]
fn batch30_note_switcher_pinned_and_current_icon_is_star() {
    // pinned trumps current for icon
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Both".into(),
        char_count: 10,
        is_current: true,
        is_pinned: true,
        preview: "x".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn batch30_note_switcher_regular_icon_file() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Regular".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "x".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::File));
}

// ---------------------------------------------------------------------------
// 13. Note switcher: preview truncation boundary at 60 chars
// ---------------------------------------------------------------------------
#[test]
fn batch30_note_switcher_60_chars_no_truncation() {
    let preview = "a".repeat(60);
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".into(),
        title: "T".into(),
        char_count: 60,
        is_current: false,
        is_pinned: false,
        preview,
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(!desc.contains("…"), "Exactly 60 chars should not truncate");
}

#[test]
fn batch30_note_switcher_61_chars_truncated() {
    let preview = "a".repeat(61);
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".into(),
        title: "T".into(),
        char_count: 61,
        is_current: false,
        is_pinned: false,
        preview,
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(desc.contains("…"), "61 chars should truncate with …");
}

#[test]
fn batch30_note_switcher_short_preview_no_truncation() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".into(),
        title: "T".into(),
        char_count: 5,
        is_current: false,
        is_pinned: false,
        preview: "hello".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(!desc.contains("…"));
    assert!(desc.contains("hello"));
}

// ---------------------------------------------------------------------------
// 14. Note switcher: empty preview falls back to char count
// ---------------------------------------------------------------------------
#[test]
fn batch30_note_switcher_empty_preview_empty_time_shows_chars() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".into(),
        title: "T".into(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert_eq!(desc, "42 chars");
}

#[test]
fn batch30_note_switcher_empty_preview_with_time_shows_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".into(),
        title: "T".into(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "3d ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert_eq!(desc, "3d ago");
}

#[test]
fn batch30_note_switcher_singular_char() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".into(),
        title: "T".into(),
        char_count: 1,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert_eq!(desc, "1 char");
}

#[test]
fn batch30_note_switcher_zero_chars() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".into(),
        title: "T".into(),
        char_count: 0,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert_eq!(desc, "0 chars");
}

// ---------------------------------------------------------------------------
// 15. New chat: empty inputs produce empty results
// ---------------------------------------------------------------------------
#[test]
fn batch30_new_chat_all_empty() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

#[test]
fn batch30_new_chat_only_models() {
    let models = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "p".into(),
        provider_display_name: "Provider".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Models"));
}

#[test]
fn batch30_new_chat_only_presets() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Presets"));
}

#[test]
fn batch30_new_chat_only_last_used() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "M1".into(),
        provider: "p".into(),
        provider_display_name: "Prov".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
}

// ---------------------------------------------------------------------------
// 16. New chat: section ordering is last_used → presets → models
// ---------------------------------------------------------------------------
#[test]
fn batch30_new_chat_section_ordering() {
    let last_used = vec![NewChatModelInfo {
        model_id: "lu".into(),
        display_name: "LU".into(),
        provider: "p".into(),
        provider_display_name: "P".into(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "gen".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "m".into(),
        display_name: "M".into(),
        provider: "p".into(),
        provider_display_name: "P".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &presets, &models);
    assert_eq!(actions.len(), 3);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
    assert_eq!(actions[1].section.as_deref(), Some("Presets"));
    assert_eq!(actions[2].section.as_deref(), Some("Models"));
}
