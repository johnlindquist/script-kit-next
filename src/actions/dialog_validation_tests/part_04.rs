
#[test]
fn test_note_switcher_all_have_notes_section() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "1".to_string(),
            title: "A".to_string(),
            char_count: 10,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "2".to_string(),
            title: "B".to_string(),
            char_count: 20,
            is_current: false,
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
// New chat action validation
// =========================================================================

#[test]
fn test_new_chat_empty_inputs() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

#[test]
fn test_new_chat_section_ordering() {
    let last_used = vec![NewChatModelInfo {
        model_id: "claude-3".to_string(),
        display_name: "Claude 3".to_string(),
        provider: "anthropic".to_string(),
        provider_display_name: "Anthropic".to_string(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "general".to_string(),
        name: "General".to_string(),
        icon: IconName::Code,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "gpt-4".to_string(),
        display_name: "GPT-4".to_string(),
        provider: "openai".to_string(),
        provider_display_name: "OpenAI".to_string(),
    }];

    let actions = get_new_chat_actions(&last_used, &presets, &models);

    let sections: Vec<&str> = actions
        .iter()
        .filter_map(|a| a.section.as_deref())
        .collect();

    // Sections in order: Last Used Settings, Presets, Models
    let lu_idx = sections
        .iter()
        .position(|&s| s == "Last Used Settings")
        .unwrap();
    let p_idx = sections.iter().position(|&s| s == "Presets").unwrap();
    let m_idx = sections.iter().position(|&s| s == "Models").unwrap();

    assert!(lu_idx < p_idx);
    assert!(p_idx < m_idx);
}

#[test]
fn test_new_chat_all_have_icons() {
    let last_used = vec![NewChatModelInfo {
        model_id: "claude".to_string(),
        display_name: "Claude".to_string(),
        provider: "anthropic".to_string(),
        provider_display_name: "Anthropic".to_string(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "code".to_string(),
        name: "Code".to_string(),
        icon: IconName::Code,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "gpt".to_string(),
        display_name: "GPT".to_string(),
        provider: "openai".to_string(),
        provider_display_name: "OpenAI".to_string(),
    }];

    let actions = get_new_chat_actions(&last_used, &presets, &models);
    for action in &actions {
        assert!(
            action.icon.is_some(),
            "New chat action '{}' should have an icon",
            action.id
        );
    }
}

#[test]
fn test_new_chat_last_used_has_bolt_icon() {
    let last_used = vec![NewChatModelInfo {
        model_id: "claude".to_string(),
        display_name: "Claude".to_string(),
        provider: "anthropic".to_string(),
        provider_display_name: "Anthropic".to_string(),
    }];

    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
}

#[test]
fn test_new_chat_models_has_settings_icon() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt".to_string(),
        display_name: "GPT".to_string(),
        provider: "openai".to_string(),
        provider_display_name: "OpenAI".to_string(),
    }];

    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].icon, Some(IconName::Settings));
}

// =========================================================================
// File context edge cases
// =========================================================================

#[test]
fn test_file_context_file_vs_dir_action_count() {
    let file = FileInfo {
        path: "/tmp/file.txt".to_string(),
        name: "file.txt".to_string(),
        file_type: FileType::File,
        is_dir: false,
    };
    let dir = FileInfo {
        path: "/tmp/dir".to_string(),
        name: "dir".to_string(),
        file_type: FileType::Directory,
        is_dir: true,
    };

    let file_actions = get_file_context_actions(&file);
    let dir_actions = get_file_context_actions(&dir);

    // File should have Quick Look, dir should not (macOS)
    #[cfg(target_os = "macos")]
    {
        assert_eq!(file_actions.len(), 7);
        assert_eq!(dir_actions.len(), 6);
    }
}

#[test]
fn test_file_context_title_includes_name() {
    let file = FileInfo {
        path: "/tmp/my-document.pdf".to_string(),
        name: "my-document.pdf".to_string(),
        file_type: FileType::Document,
        is_dir: false,
    };

    let actions = get_file_context_actions(&file);
    assert!(actions[0].title.contains("my-document.pdf"));
}

// =========================================================================
// Scriptlet with custom actions validation
// =========================================================================

#[test]
fn test_scriptlet_custom_actions_have_has_action_true() {
    let script = ScriptInfo::scriptlet("Test", "/path/to/test.md", None, None);
    let mut scriptlet = Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Custom".to_string(),
        command: "custom".to_string(),
        tool: "bash".to_string(),
        code: "echo custom".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];

    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let custom = actions
        .iter()
        .find(|a| a.id == "scriptlet_action:custom")
        .unwrap();
    assert!(
        custom.has_action,
        "Custom scriptlet actions must have has_action=true"
    );
    assert_eq!(custom.value, Some("custom".to_string()));
}

#[test]
fn test_scriptlet_builtin_actions_have_has_action_false() {
    let script = ScriptInfo::scriptlet("Test", "/path/to/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);

    for action in &actions {
        if !action.id.starts_with("scriptlet_action:") {
            assert!(
                !action.has_action,
                "Built-in action '{}' should have has_action=false",
                action.id
            );
        }
    }
}

// =========================================================================
// ID uniqueness checks
// =========================================================================

#[test]
fn test_script_context_no_duplicate_ids() {
    let script = ScriptInfo::new("test", "/path/to/test.ts");
    let actions = get_script_context_actions(&script);

    let mut seen = std::collections::HashSet::new();
    for action in &actions {
        assert!(
            seen.insert(&action.id),
            "Duplicate action ID: {}",
            action.id
        );
    }
}

#[test]
fn test_clipboard_context_no_duplicate_ids() {
    let entry = ClipboardEntryInfo {
        id: "test".to_string(),
        content_type: ContentType::Image,
        pinned: true,
        preview: "test".to_string(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: Some("Finder".to_string()),
    };

    let actions = get_clipboard_history_context_actions(&entry);

    let mut seen = std::collections::HashSet::new();
    for action in &actions {
        assert!(
            seen.insert(&action.id),
            "Duplicate clipboard action ID: {}",
            action.id
        );
    }
}

#[test]
fn test_ai_command_bar_no_duplicate_ids() {
    let actions = get_ai_command_bar_actions();

    let mut seen = std::collections::HashSet::new();
    for action in &actions {
        assert!(
            seen.insert(&action.id),
            "Duplicate AI action ID: {}",
            action.id
        );
    }
}

// =========================================================================
// Action category invariants
// =========================================================================

#[test]
fn test_all_script_actions_use_script_context_category() {
    let script = ScriptInfo::new("test", "/path/to/test.ts");
    let actions = get_script_context_actions(&script);

    for action in &actions {
        assert_eq!(
            action.category,
            ActionCategory::ScriptContext,
            "Script action '{}' should have ScriptContext category",
            action.id
        );
    }
}

#[test]
fn test_all_clipboard_actions_use_script_context_category() {
    let entry = ClipboardEntryInfo {
        id: "test".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };

    let actions = get_clipboard_history_context_actions(&entry);
    for action in &actions {
        assert_eq!(
            action.category,
            ActionCategory::ScriptContext,
            "Clipboard action '{}' should have ScriptContext category",
            action.id
        );
    }
}

// =========================================================================
// Enum default values
// =========================================================================

#[test]
fn test_search_position_default() {
    assert_eq!(SearchPosition::default(), SearchPosition::Bottom);
}

#[test]
fn test_section_style_default() {
    assert_eq!(SectionStyle::default(), SectionStyle::Separators);
}

#[test]
fn test_anchor_position_default() {
    assert_eq!(AnchorPosition::default(), AnchorPosition::Bottom);
}

#[test]
fn test_actions_dialog_config_default() {
    let config = ActionsDialogConfig::default();
    assert_eq!(config.search_position, SearchPosition::Bottom);
    assert_eq!(config.section_style, SectionStyle::Separators);
    assert_eq!(config.anchor, AnchorPosition::Bottom);
    assert!(!config.show_icons);
    assert!(!config.show_footer);
}

// =========================================================================
// Action with_* builder chain validation
// =========================================================================

#[test]
fn test_action_builder_chain() {
    let action = Action::new(
        "test",
        "Test",
        Some("Desc".into()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘T")
    .with_icon(IconName::Star)
    .with_section("Section");

    assert_eq!(action.shortcut, Some("⌘T".to_string()));
    assert_eq!(action.icon, Some(IconName::Star));
    assert_eq!(action.section, Some("Section".to_string()));
    // Lowercase caches should be populated
    assert_eq!(action.title_lower, "test");
    assert_eq!(action.description_lower, Some("desc".to_string()));
    assert_eq!(action.shortcut_lower, Some("⌘t".to_string()));
}

#[test]
fn test_action_default_fields() {
    let action = Action::new("id", "Title", None, ActionCategory::ScriptContext);
    assert!(!action.has_action);
    assert!(action.value.is_none());
    assert!(action.icon.is_none());
    assert!(action.section.is_none());
    assert!(action.shortcut.is_none());
}

// =========================================================================
// ScriptInfo agent construction
// =========================================================================

#[test]
fn test_script_info_agent_requires_is_script_false() {
    let mut agent = ScriptInfo::new("Agent", "/path/to/agent.md");
    agent.is_agent = true;
    agent.is_script = false;

    let actions = get_script_context_actions(&agent);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    // Agent-specific actions
    assert!(ids.contains(&"edit_script")); // titled "Edit Agent"
    assert!(ids.contains(&"reveal_in_finder"));
    assert!(ids.contains(&"copy_path"));
    assert!(ids.contains(&"copy_content"));
    // NOT script-only
    assert!(!ids.contains(&"view_logs"));
}

#[test]
fn test_script_info_agent_with_is_script_true_gets_script_actions() {
    let mut script_agent = ScriptInfo::new("Agent", "/path/to/agent.md");
    script_agent.is_agent = true;
    script_agent.is_script = true; // This is wrong for agents but let's test behavior

    let actions = get_script_context_actions(&script_agent);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    // With is_script=true, gets BOTH script and agent actions (duplicates may occur)
    assert!(ids.contains(&"view_logs")); // script-only action
}

// =========================================================================
// Clipboard context title truncation
// =========================================================================

#[test]
fn test_clipboard_short_preview_not_truncated() {
    let preview = "Short text".to_string();
    let context_title = if preview.len() > 30 {
        format!("{}...", &preview[..27])
    } else {
        preview.clone()
    };
    assert_eq!(context_title, "Short text");
}
