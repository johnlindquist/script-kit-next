
// ============================================================================
// Deeplink description formatting across script types
// ============================================================================

#[test]
fn deeplink_description_format_for_script() {
    let script = ScriptInfo::new("My Cool Script", "/path/script.ts");
    let actions = get_script_context_actions(&script);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert_eq!(
        dl.description.as_deref(),
        Some("Copy scriptkit://run/my-cool-script URL to clipboard")
    );
}

#[test]
fn deeplink_description_format_for_builtin() {
    let builtin = ScriptInfo::builtin("Clipboard History");
    let actions = get_script_context_actions(&builtin);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert_eq!(
        dl.description.as_deref(),
        Some("Copy scriptkit://run/clipboard-history URL to clipboard")
    );
}

#[test]
fn deeplink_description_format_for_scriptlet() {
    let scriptlet = ScriptInfo::scriptlet("Open GitHub", "/path/url.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&scriptlet, None);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert_eq!(
        dl.description.as_deref(),
        Some("Copy scriptkit://run/open-github URL to clipboard")
    );
}

// ============================================================================
// Agent flag interaction edge cases
// ============================================================================

#[test]
fn agent_with_is_script_false_gets_agent_actions() {
    let mut agent = ScriptInfo::new("My Agent", "/path/agent.md");
    agent.is_agent = true;
    agent.is_script = false;
    let actions = get_script_context_actions(&agent);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    // Agent-specific
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.title, "Edit Agent");
    assert!(ids.contains(&"reveal_in_finder"));
    assert!(ids.contains(&"copy_path"));
    assert!(ids.contains(&"copy_content"));

    // Must NOT have script-only actions
    assert!(!ids.contains(&"view_logs"));
}

#[test]
fn agent_with_all_flags_combined() {
    let mut agent = ScriptInfo::with_shortcut_and_alias(
        "Super Agent",
        "/path/agent.md",
        Some("cmd+shift+a".into()),
        Some("sa".into()),
    );
    agent.is_agent = true;
    agent.is_script = false;
    let agent = agent.with_frecency(true, Some("agent:super".into()));

    let actions = get_script_context_actions(&agent);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    // Has update/remove (not add) for shortcut and alias
    assert!(ids.contains(&"update_shortcut"));
    assert!(ids.contains(&"remove_shortcut"));
    assert!(ids.contains(&"update_alias"));
    assert!(ids.contains(&"remove_alias"));
    assert!(!ids.contains(&"add_shortcut"));
    assert!(!ids.contains(&"add_alias"));

    // Has frecency reset
    assert!(ids.contains(&"reset_ranking"));

    // Has agent actions
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.title, "Edit Agent");
    assert!(ids.contains(&"copy_deeplink"));
}

#[test]
fn agent_with_is_script_true_gets_script_actions_instead() {
    // If someone mistakenly sets both is_agent and is_script true,
    // is_script section fires first (before is_agent), so we get script actions
    let mut script = ScriptInfo::new("Weird Agent", "/path/weird.ts");
    script.is_agent = true;
    // is_script is already true from new()
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    // Gets BOTH script and agent actions (both branches fire)
    assert!(ids.contains(&"view_logs")); // script-only
                                         // Agent branch also adds edit_script with "Edit Agent" title
                                         // But script branch already added "Edit Script" - check that both exist
    let edit_actions: Vec<&str> = actions
        .iter()
        .filter(|a| a.id == "edit_script")
        .map(|a| a.title.as_str())
        .collect();
    assert_eq!(edit_actions.len(), 2); // One "Edit Script" from is_script, one "Edit Agent" from is_agent
}

// ============================================================================
// Notes command bar section correctness
// ============================================================================

#[test]
fn notes_command_bar_sections_are_correct() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);

    // Verify each action has the expected section
    let new_note = actions.iter().find(|a| a.id == "new_note").unwrap();
    assert_eq!(new_note.section.as_deref(), Some("Notes"));

    let duplicate = actions.iter().find(|a| a.id == "duplicate_note").unwrap();
    assert_eq!(duplicate.section.as_deref(), Some("Notes"));

    let browse = actions.iter().find(|a| a.id == "browse_notes").unwrap();
    assert_eq!(browse.section.as_deref(), Some("Notes"));

    let find = actions.iter().find(|a| a.id == "find_in_note").unwrap();
    assert_eq!(find.section.as_deref(), Some("Edit"));

    let format = actions.iter().find(|a| a.id == "format").unwrap();
    assert_eq!(format.section.as_deref(), Some("Edit"));

    let copy_as = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
    assert_eq!(copy_as.section.as_deref(), Some("Copy"));

    let export = actions.iter().find(|a| a.id == "export").unwrap();
    assert_eq!(export.section.as_deref(), Some("Export"));

    let auto_size = actions
        .iter()
        .find(|a| a.id == "enable_auto_sizing")
        .unwrap();
    assert_eq!(auto_size.section.as_deref(), Some("Settings"));
}

#[test]
fn notes_command_bar_auto_sizing_enabled_hides_toggle() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(!ids.contains(&"enable_auto_sizing"));
}

// ============================================================================
// AI command bar section ordering
// ============================================================================

#[test]
fn ai_command_bar_section_order_is_deterministic() {
    let actions = get_ai_command_bar_actions();
    let sections: Vec<&str> = actions
        .iter()
        .filter_map(|a| a.section.as_deref())
        .collect();

    // Find first occurrence of each section
    let response_idx = sections.iter().position(|s| *s == "Response").unwrap();
    let actions_idx = sections.iter().position(|s| *s == "Actions").unwrap();
    let attachments_idx = sections.iter().position(|s| *s == "Attachments").unwrap();
    let settings_idx = sections.iter().position(|s| *s == "Settings").unwrap();

    assert!(response_idx < actions_idx, "Response before Actions");
    assert!(actions_idx < attachments_idx, "Actions before Attachments");
    assert!(
        attachments_idx < settings_idx,
        "Attachments before Settings"
    );
}

#[test]
fn ai_command_bar_response_section_has_three_actions() {
    let actions = get_ai_command_bar_actions();
    let response_count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .count();
    assert_eq!(response_count, 3);
}

#[test]
fn ai_command_bar_actions_section_has_four_actions() {
    let actions = get_ai_command_bar_actions();
    let action_count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Actions"))
        .count();
    assert_eq!(action_count, 4);
}

#[test]
fn ai_command_bar_attachments_section_has_two_actions() {
    let actions = get_ai_command_bar_actions();
    let attach_count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Attachments"))
        .count();
    assert_eq!(attach_count, 2);
}

#[test]
fn ai_command_bar_settings_section_has_one_action() {
    let actions = get_ai_command_bar_actions();
    let settings_count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Settings"))
        .count();
    assert_eq!(settings_count, 1);
}

// ============================================================================
// Note switcher correctness
// ============================================================================

#[test]
fn note_switcher_multiple_notes_icon_assignment() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "a".into(),
            title: "Pinned Current".into(),
            char_count: 100,
            is_current: true,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "b".into(),
            title: "Pinned Not Current".into(),
            char_count: 50,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "c".into(),
            title: "Current Not Pinned".into(),
            char_count: 200,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "d".into(),
            title: "Neither".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];

    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions.len(), 4);

    // Pinned+current → StarFilled (pinned takes priority)
    assert_eq!(
        actions[0].icon,
        Some(crate::designs::icon_variations::IconName::StarFilled)
    );
    assert!(actions[0].title.starts_with("• ")); // current indicator

    // Pinned only → StarFilled
    assert_eq!(
        actions[1].icon,
        Some(crate::designs::icon_variations::IconName::StarFilled)
    );
    assert!(!actions[1].title.starts_with("• ")); // not current

    // Current only → Check
    assert_eq!(
        actions[2].icon,
        Some(crate::designs::icon_variations::IconName::Check)
    );
    assert!(actions[2].title.starts_with("• "));

    // Neither → File
    assert_eq!(
        actions[3].icon,
        Some(crate::designs::icon_variations::IconName::File)
    );
    assert!(!actions[3].title.starts_with("• "));
}

#[test]
fn note_switcher_char_count_description_formatting() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "a".into(),
            title: "Zero".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "b".into(),
            title: "One".into(),
            char_count: 1,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "c".into(),
            title: "Many".into(),
            char_count: 999,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];

    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("0 chars"));
    assert_eq!(actions[1].description.as_deref(), Some("1 char"));
    assert_eq!(actions[2].description.as_deref(), Some("999 chars"));
}

#[test]
fn note_switcher_id_format() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc-123-def".into(),
        title: "Test".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].id, "note_abc-123-def");
}

// ============================================================================
// Chat context model actions
// ============================================================================

#[test]
fn chat_context_with_multiple_models_marks_only_current() {
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
            ChatModelInfo {
                id: "gemini".into(),
                display_name: "Gemini".into(),
                provider: "Google".into(),
            },
        ],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);

    // Only GPT-4 should have checkmark
    let gpt4 = actions
        .iter()
        .find(|a| a.id == "select_model_gpt4")
        .unwrap();
    assert_eq!(gpt4.title, "GPT-4 ✓");

    let claude = actions
        .iter()
        .find(|a| a.id == "select_model_claude")
        .unwrap();
    assert_eq!(claude.title, "Claude");
    assert!(!claude.title.contains('✓'));

    let gemini = actions
        .iter()
        .find(|a| a.id == "select_model_gemini")
        .unwrap();
    assert_eq!(gemini.title, "Gemini");
    assert!(!gemini.title.contains('✓'));
}

#[test]
fn chat_context_model_description_shows_provider() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "model1".into(),
            display_name: "Model One".into(),
            provider: "Anthropic".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model = actions
        .iter()
        .find(|a| a.id == "select_model_model1")
        .unwrap();
    assert_eq!(model.description.as_deref(), Some("via Anthropic"));
}
