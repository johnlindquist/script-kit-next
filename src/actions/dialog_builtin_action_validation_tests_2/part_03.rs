
#[test]
fn agent_has_reveal_and_copy_path() {
    let mut agent = ScriptInfo::new("my-agent", "/path/agent.md");
    agent.is_agent = true;
    agent.is_script = false;
    let actions_tmp = get_script_context_actions(&agent);
    let ids = action_ids(&actions_tmp);
    assert!(ids.contains(&"reveal_in_finder"));
    assert!(ids.contains(&"copy_path"));
    assert!(ids.contains(&"copy_content"));
}

#[test]
fn agent_lacks_view_logs() {
    let mut agent = ScriptInfo::new("my-agent", "/path/agent.md");
    agent.is_agent = true;
    agent.is_script = false;
    let actions_tmp = get_script_context_actions(&agent);
    let ids = action_ids(&actions_tmp);
    assert!(
        !ids.contains(&"view_logs"),
        "Agent should not have view_logs"
    );
}

// =========================================================================
// 16. Builtin lacks file-specific actions
// =========================================================================

#[test]
fn builtin_lacks_edit_view_logs_reveal_copy_path_copy_content() {
    let builtin = ScriptInfo::builtin("Clipboard History");
    let actions_tmp = get_script_context_actions(&builtin);
    let ids = action_ids(&actions_tmp);
    assert!(!ids.contains(&"edit_script"));
    assert!(!ids.contains(&"view_logs"));
    assert!(!ids.contains(&"reveal_in_finder"));
    assert!(!ids.contains(&"copy_path"));
    assert!(!ids.contains(&"copy_content"));
}

#[test]
fn builtin_has_run_shortcut_alias_deeplink() {
    let builtin = ScriptInfo::builtin("Clipboard History");
    let actions_tmp = get_script_context_actions(&builtin);
    let ids = action_ids(&actions_tmp);
    assert!(ids.contains(&"run_script"));
    assert!(ids.contains(&"add_shortcut"));
    assert!(ids.contains(&"add_alias"));
    assert!(ids.contains(&"copy_deeplink"));
}

// =========================================================================
// 17. Note switcher edge cases
// =========================================================================

#[test]
fn note_switcher_empty_shows_no_notes_placeholder() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "no_notes");
    assert!(actions[0].title.contains("No notes"));
    assert_eq!(actions[0].icon, Some(IconName::Plus));
}

#[test]
fn note_switcher_singular_character_count() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "one".into(),
        title: "One Char".into(),
        char_count: 1,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(
        actions[0].description.as_ref().unwrap().contains("1 char"),
        "Singular should be '1 char', got '{:?}'",
        actions[0].description
    );
    assert!(
        !actions[0].description.as_ref().unwrap().contains("chars"),
        "Singular should NOT contain 'chars'"
    );
}

#[test]
fn note_switcher_plural_character_count() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "many".into(),
        title: "Many Chars".into(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(
        actions[0]
            .description
            .as_ref()
            .unwrap()
            .contains("42 chars"),
        "Plural should be '42 chars', got '{:?}'",
        actions[0].description
    );
}

#[test]
fn note_switcher_zero_characters_plural() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "empty".into(),
        title: "Empty Note".into(),
        char_count: 0,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(
        actions[0].description.as_ref().unwrap().contains("0 chars"),
        "Zero should be '0 chars', got '{:?}'",
        actions[0].description
    );
}

#[test]
fn note_switcher_icon_hierarchy_pinned_over_current() {
    // Pinned + current = StarFilled (pinned wins)
    let notes = vec![NoteSwitcherNoteInfo {
        id: "both".into(),
        title: "Both".into(),
        char_count: 5,
        is_current: true,
        is_pinned: true,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn note_switcher_icon_current_only() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "cur".into(),
        title: "Current".into(),
        char_count: 5,
        is_current: true,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::Check));
}

#[test]
fn note_switcher_icon_default() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "plain".into(),
        title: "Plain".into(),
        char_count: 5,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::File));
}

#[test]
fn note_switcher_current_has_bullet_prefix() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "cur".into(),
            title: "Current Note".into(),
            char_count: 5,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "other".into(),
            title: "Other Note".into(),
            char_count: 3,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    assert!(
        actions[0].title.starts_with("• "),
        "Current note should have '• ' prefix, got '{}'",
        actions[0].title
    );
    assert!(
        !actions[1].title.starts_with("• "),
        "Non-current note should NOT have '• ' prefix"
    );
}

#[test]
fn note_switcher_all_have_notes_section() {
    let notes: Vec<NoteSwitcherNoteInfo> = (0..5)
        .map(|i| NoteSwitcherNoteInfo {
            id: format!("n{}", i),
            title: format!("Note {}", i),
            char_count: i * 10,
            is_current: i == 0,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        })
        .collect();
    for action in &get_note_switcher_actions(&notes) {
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
// 18. Score bonuses for description and shortcut matches
// =========================================================================

#[test]
fn score_description_only_match_returns_nonzero() {
    // Title doesn't match, but description contains the query
    let action = Action::new(
        "test",
        "Something Unrelated",
        Some("Opens the editor for you".into()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "editor");
    assert!(
        score > 0,
        "Description-only match should return nonzero score, got {}",
        score
    );
}

#[test]
fn score_shortcut_only_match_returns_nonzero() {
    let action = Action::new(
        "test",
        "Something Unrelated",
        None,
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘E");
    let score = ActionsDialog::score_action(&action, "⌘e");
    assert!(
        score > 0,
        "Shortcut-only match should return nonzero score, got {}",
        score
    );
}

#[test]
fn score_no_match_returns_zero() {
    let action = Action::new(
        "test",
        "Edit Script",
        Some("Open in editor".into()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘E");
    let score = ActionsDialog::score_action(&action, "zzzznotfound");
    assert_eq!(score, 0, "No match should return 0");
}

#[test]
fn score_title_plus_description_bonus_stacks() {
    let action = Action::new(
        "edit",
        "Edit Script",
        Some("Edit the script file".into()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "edit");
    // Should get prefix bonus (100) + description bonus (15) = 115
    assert!(
        score > 100,
        "Title + description match should stack bonuses, got {}",
        score
    );
}

// =========================================================================
// 19. New chat action ID format and empty sections
// =========================================================================

#[test]
fn new_chat_last_used_ids_are_indexed() {
    let last = vec![
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
    let actions = get_new_chat_actions(&last, &[], &[]);
    assert_eq!(actions[0].id, "last_used_0");
    assert_eq!(actions[1].id, "last_used_1");
}

#[test]
fn new_chat_preset_ids_use_preset_id() {
    let presets = vec![NewChatPresetInfo {
        id: "code-review".into(),
        name: "Code Review".into(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].id, "preset_code-review");
}

#[test]
fn new_chat_model_ids_are_indexed() {
    let models = vec![
        NewChatModelInfo {
            model_id: "claude".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        },
        NewChatModelInfo {
            model_id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "openai".into(),
            provider_display_name: "OpenAI".into(),
        },
    ];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].id, "model_0");
    assert_eq!(actions[1].id, "model_1");
}

#[test]
fn new_chat_empty_all_sections_returns_empty() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(
        actions.is_empty(),
        "All empty sections should return empty actions"
    );
}

#[test]
fn new_chat_model_descriptions_have_provider() {
    let models = vec![NewChatModelInfo {
        model_id: "claude".into(),
        display_name: "Claude".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(
        actions[0].description.as_deref(),
        Some("Anthropic"),
        "Model description should be provider_display_name"
    );
}

#[test]
fn new_chat_last_used_descriptions_have_provider() {
    let last = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&last, &[], &[]);
    assert_eq!(
        actions[0].description.as_deref(),
        Some("OpenAI"),
        "Last used description should be provider_display_name"
    );
}

#[test]
fn new_chat_presets_have_no_description() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert!(
        actions[0].description.is_none(),
        "Presets should have no description"
    );
}

// =========================================================================
// 20. Global actions always empty
// =========================================================================

#[test]
fn global_actions_always_returns_empty() {
    use super::builders::get_global_actions;
    let actions = get_global_actions();
    assert!(actions.is_empty(), "Global actions should always be empty");
}

// =========================================================================
// 21. Deeplink name edge cases
// =========================================================================

#[test]
fn deeplink_name_multiple_spaces_collapsed() {
    assert_eq!(to_deeplink_name("My   Script   Name"), "my-script-name");
}

#[test]
fn deeplink_name_underscores_become_hyphens() {
    assert_eq!(to_deeplink_name("my_script_name"), "my-script-name");
}

#[test]
fn deeplink_name_mixed_case_special_chars() {
    assert_eq!(to_deeplink_name("Hello (World) #1!"), "hello-world-1");
}

#[test]
fn deeplink_name_leading_trailing_special_chars() {
    assert_eq!(to_deeplink_name("---hello---"), "hello");
}
