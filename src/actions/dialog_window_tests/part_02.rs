
// =========================================================================
// Note switcher: ordering, icons, multiple notes
// =========================================================================

#[test]
fn note_switcher_multiple_notes_ordering_preserved() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "aaa".to_string(),
            title: "First".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "bbb".to_string(),
            title: "Second".to_string(),
            char_count: 200,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "ccc".to_string(),
            title: "Third".to_string(),
            char_count: 50,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions.len(), 3);
    assert_eq!(actions[0].id, "note_aaa");
    assert_eq!(actions[1].id, "note_bbb");
    assert_eq!(actions[2].id, "note_ccc");
}

#[test]
fn note_switcher_icon_priority_pinned_over_current() {
    // A note that is BOTH pinned and current should show StarFilled (pinned wins)
    let notes = vec![NoteSwitcherNoteInfo {
        id: "x".to_string(),
        title: "Both".to_string(),
        char_count: 10,
        is_current: true,
        is_pinned: true,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn note_switcher_current_note_title_prefix() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "cur".to_string(),
        title: "My Note".to_string(),
        char_count: 5,
        is_current: true,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(
        actions[0].title.starts_with("• "),
        "Current note should have bullet prefix, got: {}",
        actions[0].title
    );
    assert!(actions[0].title.contains("My Note"));
}

#[test]
fn note_switcher_non_current_no_prefix() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "other".to_string(),
        title: "Other Note".to_string(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].title, "Other Note");
    assert!(!actions[0].title.starts_with("• "));
}

#[test]
fn note_switcher_char_count_description() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "z".to_string(),
            title: "Zero".to_string(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "o".to_string(),
            title: "One".to_string(),
            char_count: 1,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "m".to_string(),
            title: "Many".to_string(),
            char_count: 500,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(
        actions[0].description,
        Some("0 chars".to_string()),
        "Zero chars should be plural"
    );
    assert_eq!(
        actions[1].description,
        Some("1 char".to_string()),
        "One char should be singular"
    );
    assert_eq!(
        actions[2].description,
        Some("500 chars".to_string()),
        "Many chars should be plural"
    );
}

#[test]
fn note_switcher_all_have_notes_section() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "A".to_string(),
            char_count: 1,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "b".to_string(),
            title: "B".to_string(),
            char_count: 2,
            is_current: true,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    for action in &actions {
        assert!(
            action.section.as_deref() == Some("Recent")
                || action.section.as_deref() == Some("Pinned"),
            "Note switcher action '{}' should be in 'Recent' or 'Pinned' section, got {:?}",
            action.id,
            action.section
        );
    }
}

// =========================================================================
// New chat actions: section ordering
// =========================================================================

#[test]
fn new_chat_actions_section_ordering() {
    let last_used = vec![NewChatModelInfo {
        model_id: "lu1".to_string(),
        display_name: "Last Used Model".to_string(),
        provider: "test".to_string(),
        provider_display_name: "Test Provider".to_string(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "general".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "m1".to_string(),
        display_name: "Model One".to_string(),
        provider: "test".to_string(),
        provider_display_name: "Test".to_string(),
    }];

    let actions = get_new_chat_actions(&last_used, &presets, &models);
    assert_eq!(actions.len(), 3);

    // Ordering: Last Used, then Presets, then Models
    assert_eq!(
        actions[0].section,
        Some("Last Used Settings".to_string()),
        "First should be Last Used section"
    );
    assert_eq!(
        actions[1].section,
        Some("Presets".to_string()),
        "Second should be Presets section"
    );
    assert_eq!(
        actions[2].section,
        Some("Models".to_string()),
        "Third should be Models section"
    );
}

#[test]
fn new_chat_actions_all_have_icons() {
    let last_used = vec![NewChatModelInfo {
        model_id: "lu".to_string(),
        display_name: "LU".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "p1".to_string(),
        name: "P1".to_string(),
        icon: IconName::Code,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "m".to_string(),
        display_name: "M".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];

    let actions = get_new_chat_actions(&last_used, &presets, &models);
    for action in &actions {
        assert!(
            action.icon.is_some(),
            "New chat action '{}' should have icon",
            action.id
        );
    }
}

#[test]
fn new_chat_last_used_has_bolt_icon() {
    let last_used = vec![NewChatModelInfo {
        model_id: "lu".to_string(),
        display_name: "LU".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(
        actions[0].icon,
        Some(IconName::BoltFilled),
        "Last used entries should have BoltFilled icon"
    );
}

#[test]
fn new_chat_models_has_settings_icon() {
    let models = vec![NewChatModelInfo {
        model_id: "m".to_string(),
        display_name: "M".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(
        actions[0].icon,
        Some(IconName::Settings),
        "Model entries should have Settings icon"
    );
}

#[test]
fn new_chat_preset_uses_custom_icon() {
    let presets = vec![NewChatPresetInfo {
        id: "code".to_string(),
        name: "Code".to_string(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(
        actions[0].icon,
        Some(IconName::Code),
        "Preset should use its own icon"
    );
}

// =========================================================================
// Deeplink name edge cases
// =========================================================================

#[test]
fn deeplink_name_empty_string() {
    assert_eq!(to_deeplink_name(""), "");
}

#[test]
fn deeplink_name_all_special_chars() {
    assert_eq!(to_deeplink_name("!@#$%^&*()"), "");
}

#[test]
fn deeplink_name_leading_trailing_spaces() {
    assert_eq!(to_deeplink_name("  My Script  "), "my-script");
}

#[test]
fn deeplink_name_consecutive_separators() {
    assert_eq!(to_deeplink_name("a--b__c  d"), "a-b-c-d");
}

#[test]
fn deeplink_name_single_char() {
    assert_eq!(to_deeplink_name("X"), "x");
}

#[test]
fn deeplink_name_numbers() {
    assert_eq!(to_deeplink_name("Script 123"), "script-123");
}

// =========================================================================
// Agent with shortcut + alias + frecency
// =========================================================================

#[test]
fn agent_with_shortcut_alias_frecency() {
    let mut agent = ScriptInfo::with_shortcut_and_alias(
        "code-agent",
        "/agents/code.md",
        Some("cmd+shift+a".to_string()),
        Some("ca".to_string()),
    );
    agent.is_agent = true;
    agent.is_script = false;
    let agent = agent.with_frecency(true, Some("/agents/code.md".to_string()));

    let actions = get_script_context_actions(&agent);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    // Should have management actions for existing shortcut+alias
    assert!(ids.contains(&"update_shortcut"));
    assert!(ids.contains(&"remove_shortcut"));
    assert!(ids.contains(&"update_alias"));
    assert!(ids.contains(&"remove_alias"));
    assert!(ids.contains(&"reset_ranking"));

    // Agent-specific actions
    assert!(ids.contains(&"edit_script")); // title says "Edit Agent"
    assert!(ids.contains(&"reveal_in_finder"));
    assert!(ids.contains(&"copy_path"));
    assert!(ids.contains(&"copy_content"));
    assert!(ids.contains(&"copy_deeplink"));

    // Should NOT have script-only or add variants
    assert!(!ids.contains(&"view_logs"));
    assert!(!ids.contains(&"add_shortcut"));
    assert!(!ids.contains(&"add_alias"));

    // Verify edit title
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.title, "Edit Agent");
}

// =========================================================================
// All built-in action IDs use snake_case
// =========================================================================

#[test]
fn all_script_action_ids_are_snake_case() {
    let script =
        ScriptInfo::new("test", "/path/test.ts").with_frecency(true, Some("/path".to_string()));
    let actions = get_script_context_actions(&script);

    for action in &actions {
        assert!(
            !action.id.contains(' '),
            "Action ID '{}' should not contain spaces",
            action.id
        );
        assert!(
            !action.id.contains('-'),
            "Action ID '{}' should use underscores, not hyphens",
            action.id
        );
        assert_eq!(
            action.id,
            action.id.to_lowercase(),
            "Action ID '{}' should be lowercase",
            action.id
        );
    }
}

#[test]
fn all_clipboard_action_ids_are_snake_case() {
    let entry = ClipboardEntryInfo {
        id: "test".to_string(),
        content_type: ContentType::Image,
        pinned: true,
        preview: "img".to_string(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: Some("App".to_string()),
    };
    let actions = get_clipboard_history_context_actions(&entry);

    for action in &actions {
        assert!(
            !action.id.contains(' '),
            "Clipboard action ID '{}' should not contain spaces",
            action.id
        );
        assert_eq!(
            action.id,
            action.id.to_lowercase(),
            "Clipboard action ID '{}' should be lowercase",
            action.id
        );
    }
}
