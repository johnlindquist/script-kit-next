// ============================================================
// 12. New chat actions (AI window new chat dropdown)
// ============================================================

#[test]
fn new_chat_actions_empty_inputs() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty(), "No inputs should produce no actions");
}

#[test]
fn new_chat_actions_last_used_section() {
    let last_used = vec![
        NewChatModelInfo {
            model_id: "claude-sonnet".into(),
            display_name: "Claude Sonnet".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        },
        NewChatModelInfo {
            model_id: "gpt-4".into(),
            display_name: "GPT-4".into(),
            provider: "openai".into(),
            provider_display_name: "OpenAI".into(),
        },
    ];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions.len(), 2);

    assert_eq!(actions[0].id, "last_used_0");
    assert_eq!(actions[0].title, "Claude Sonnet");
    assert_eq!(actions[0].description.as_deref(), Some("Anthropic"));
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));

    assert_eq!(actions[1].id, "last_used_1");
    assert_eq!(actions[1].title, "GPT-4");
    assert_eq!(actions[1].description.as_deref(), Some("OpenAI"));
}

#[test]
fn new_chat_actions_presets_section() {
    let presets = vec![
        NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: crate::designs::icon_variations::IconName::Settings,
        },
        NewChatPresetInfo {
            id: "code".into(),
            name: "Code".into(),
            icon: crate::designs::icon_variations::IconName::Code,
        },
    ];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions.len(), 2);

    assert_eq!(actions[0].id, "preset_general");
    assert_eq!(actions[0].title, "General");
    assert_eq!(actions[0].section.as_deref(), Some("Presets"));
    assert!(
        actions[0].description.is_none(),
        "Presets have no description"
    );

    assert_eq!(actions[1].id, "preset_code");
    assert_eq!(actions[1].title, "Code");
}

#[test]
fn new_chat_actions_models_section() {
    let models = vec![NewChatModelInfo {
        model_id: "opus".into(),
        display_name: "Claude Opus".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions.len(), 1);

    assert_eq!(actions[0].id, "model_0");
    assert_eq!(actions[0].title, "Claude Opus");
    assert_eq!(actions[0].description.as_deref(), Some("Anthropic"));
    assert_eq!(actions[0].section.as_deref(), Some("Models"));
}

#[test]
fn new_chat_actions_combined_ordering() {
    let last_used = vec![NewChatModelInfo {
        model_id: "last".into(),
        display_name: "Last".into(),
        provider: "p".into(),
        provider_display_name: "P".into(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "preset".into(),
        name: "Preset".into(),
        icon: crate::designs::icon_variations::IconName::Star,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "model".into(),
        display_name: "Model".into(),
        provider: "p".into(),
        provider_display_name: "P".into(),
    }];

    let actions = get_new_chat_actions(&last_used, &presets, &models);
    assert_eq!(actions.len(), 3);

    // Ordering: Last Used -> Presets -> Models
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
    assert_eq!(actions[1].section.as_deref(), Some("Presets"));
    assert_eq!(actions[2].section.as_deref(), Some("Models"));
}

#[test]
fn new_chat_actions_all_have_icons() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m".into(),
        display_name: "M".into(),
        provider: "p".into(),
        provider_display_name: "P".into(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "pr".into(),
        name: "PR".into(),
        icon: crate::designs::icon_variations::IconName::Code,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "mo".into(),
        display_name: "MO".into(),
        provider: "p".into(),
        provider_display_name: "P".into(),
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

// ============================================================
// 13. Agent-specific script context actions
// ============================================================

#[test]
fn agent_context_has_agent_actions() {
    let mut agent = ScriptInfo::new("My Agent", "/path/to/agent.claude.md");
    agent.is_agent = true;
    agent.is_script = false;

    let actions = get_script_context_actions(&agent);
    let ids = action_ids(&actions);

    // Agent should have: edit (as "Edit Agent"), reveal, copy_path, copy_content
    assert!(
        ids.contains(&"edit_script"),
        "Agent should have edit_script action"
    );
    assert!(ids.contains(&"reveal_in_finder"));
    assert!(ids.contains(&"copy_path"));
    assert!(ids.contains(&"copy_content"));

    // Verify edit title says "Edit Agent" not "Edit Script"
    let edit = find_action(&actions, "edit_script").unwrap();
    assert_eq!(edit.title, "Edit Agent");
    assert_eq!(
        edit.description.as_deref(),
        Some("Open the agent file in $EDITOR")
    );
}

#[test]
fn agent_context_no_script_only_actions() {
    let mut agent = ScriptInfo::new("Agent", "/path/to/agent.md");
    agent.is_agent = true;
    agent.is_script = false;

    let actions = get_script_context_actions(&agent);
    let ids = action_ids(&actions);

    // Agent should NOT have view_logs (script-only)
    assert!(
        !ids.contains(&"view_logs"),
        "Agent should not have view_logs"
    );
}

#[test]
fn agent_context_has_deeplink_and_shortcut() {
    let mut agent = ScriptInfo::new("Code Review Agent", "/path/to/agent.md");
    agent.is_agent = true;
    agent.is_script = false;

    let actions = get_script_context_actions(&agent);
    let ids = action_ids(&actions);

    // Should still have universal actions
    assert!(ids.contains(&"run_script"), "Agent should have run action");
    assert!(
        ids.contains(&"copy_deeplink"),
        "Agent should have copy_deeplink"
    );
    assert!(
        ids.contains(&"add_shortcut"),
        "Agent should have add_shortcut"
    );
    assert!(ids.contains(&"add_alias"), "Agent should have add_alias");
}

#[test]
fn agent_context_with_frecency_shows_reset() {
    let mut agent = ScriptInfo::new("Agent", "/path/to/agent.md");
    agent.is_agent = true;
    agent.is_script = false;
    let agent = agent.with_frecency(true, Some("agent:path".into()));

    let actions = get_script_context_actions(&agent);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"reset_ranking"));
}

#[test]
fn mixed_script_and_agent_flags_do_not_create_duplicate_action_ids() {
    let mut script = ScriptInfo::new("Mixed Flags", "/path/to/mixed.ts");
    script.is_agent = true;
    // Keep is_script=true to simulate accidental mixed flag state.

    let actions = get_script_context_actions(&script);

    let mut seen = HashSet::new();
    for action in &actions {
        assert!(
            seen.insert(action.id.as_str()),
            "duplicate action id generated: {}",
            action.id
        );
    }

    let edit_script_count = actions.iter().filter(|a| a.id == "edit_script").count();
    assert_eq!(edit_script_count, 1);
}
