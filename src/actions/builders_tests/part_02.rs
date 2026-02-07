// ============================================================
// 4. Chat context actions: model selection, conditional actions
// ============================================================

#[test]
fn chat_context_model_selection_marks_current() {
    let info = ChatPromptInfo {
        current_model: Some("Claude Sonnet".into()),
        available_models: vec![
            ChatModelInfo {
                id: "claude-sonnet".into(),
                display_name: "Claude Sonnet".into(),
                provider: "Anthropic".into(),
            },
            ChatModelInfo {
                id: "gpt-4".into(),
                display_name: "GPT-4".into(),
                provider: "OpenAI".into(),
            },
        ],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);

    let sonnet = find_action(&actions, "select_model_claude-sonnet").unwrap();
    assert!(
        sonnet.title.contains("✓"),
        "Current model should have checkmark"
    );

    let gpt4 = find_action(&actions, "select_model_gpt-4").unwrap();
    assert!(
        !gpt4.title.contains("✓"),
        "Non-current model should not have checkmark"
    );
}

#[test]
fn chat_context_copy_response_only_when_has_response() {
    let with_response = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&with_response);
    let ids = action_ids(&actions);
    assert!(
        ids.contains(&"copy_response"),
        "Should have copy_response when has_response=true"
    );

    let without_response = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&without_response);
    let ids = action_ids(&actions);
    assert!(
        !ids.contains(&"copy_response"),
        "Should NOT have copy_response when has_response=false"
    );
}

#[test]
fn chat_context_clear_only_when_has_messages() {
    let with_msgs = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&with_msgs);
    let ids = action_ids(&actions);
    assert!(
        ids.contains(&"clear_conversation"),
        "Should have clear_conversation when has_messages=true"
    );

    let empty = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&empty);
    let ids = action_ids(&actions);
    assert!(
        !ids.contains(&"clear_conversation"),
        "Should NOT have clear_conversation when has_messages=false"
    );
}

#[test]
fn chat_context_continue_in_chat_always_present() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let ids = action_ids(&actions);
    assert!(
        ids.contains(&"continue_in_chat"),
        "continue_in_chat should always be present"
    );
}

// ============================================================
// 5. to_deeplink_name conversion
// ============================================================

#[test]
fn deeplink_name_lowercase_and_hyphenated() {
    assert_eq!(to_deeplink_name("My Script"), "my-script");
    assert_eq!(to_deeplink_name("Clipboard History"), "clipboard-history");
    assert_eq!(to_deeplink_name("hello_world"), "hello-world");
}

#[test]
fn deeplink_name_strips_special_chars() {
    assert_eq!(to_deeplink_name("hello@world!"), "hello-world");
    assert_eq!(to_deeplink_name("  spaced  out  "), "spaced-out");
    assert_eq!(to_deeplink_name("---dashes---"), "dashes");
}

#[test]
fn deeplink_name_preserves_alphanumeric() {
    assert_eq!(to_deeplink_name("script123"), "script123");
    assert_eq!(to_deeplink_name("ABC"), "abc");
}

// ============================================================
// 6. Script context actions: sections + destructive ordering
// ============================================================

#[test]
fn script_context_actions_use_sectioned_grouping() {
    let script = ScriptInfo::new("my-script", "/path/to/my-script.ts");
    let actions = get_script_context_actions(&script);

    let run = find_action(&actions, "run_script").expect("missing run_script");
    assert_eq!(run.section.as_deref(), Some("Actions"));

    let add_shortcut = find_action(&actions, "add_shortcut").expect("missing add_shortcut");
    assert_eq!(add_shortcut.section.as_deref(), Some("Edit"));

    let add_alias = find_action(&actions, "add_alias").expect("missing add_alias");
    assert_eq!(add_alias.section.as_deref(), Some("Edit"));

    let edit_script = find_action(&actions, "edit_script").expect("missing edit_script");
    assert_eq!(edit_script.section.as_deref(), Some("Edit"));

    let copy_path = find_action(&actions, "copy_path").expect("missing copy_path");
    assert_eq!(copy_path.section.as_deref(), Some("Share"));

    let copy_deeplink = find_action(&actions, "copy_deeplink").expect("missing copy_deeplink");
    assert_eq!(copy_deeplink.section.as_deref(), Some("Share"));
}

#[test]
fn script_context_destructive_actions_are_last_and_marked() {
    let script = ScriptInfo::with_shortcut_and_alias(
        "my-script",
        "/path/to/my-script.ts",
        Some("cmd+shift+m".to_string()),
        Some("ms".to_string()),
    )
    .with_frecency(true, Some("my-script:/path/to/my-script.ts".to_string()));

    let actions = get_script_context_actions(&script);

    let first_destructive_index = actions
        .iter()
        .position(|a| a.section.as_deref() == Some("Destructive"))
        .expect("expected destructive section");

    for action in actions.iter().skip(first_destructive_index) {
        assert_eq!(
            action.section.as_deref(),
            Some("Destructive"),
            "all trailing actions should be in Destructive section"
        );
    }

    let remove_shortcut =
        find_action(&actions, "remove_shortcut").expect("missing remove_shortcut");
    assert_eq!(remove_shortcut.section.as_deref(), Some("Destructive"));
    assert_eq!(remove_shortcut.shortcut.as_deref(), Some("⌘⌥K"));

    let remove_alias = find_action(&actions, "remove_alias").expect("missing remove_alias");
    assert_eq!(remove_alias.section.as_deref(), Some("Destructive"));
    assert_eq!(remove_alias.shortcut.as_deref(), Some("⌘⌥A"));

    let reset_ranking = find_action(&actions, "reset_ranking").expect("missing reset_ranking");
    assert_eq!(reset_ranking.section.as_deref(), Some("Destructive"));
    assert!(
        reset_ranking.shortcut.is_some(),
        "destructive action should include keyboard hint"
    );
}

// ============================================================
// 7. AI command bar: sections, icons, completeness
// ============================================================

#[test]
fn ai_command_bar_has_all_expected_actions() {
    let actions = get_ai_command_bar_actions();
    let ids = action_ids(&actions);

    let expected = [
        "copy_response",
        "copy_chat",
        "copy_last_code",
        "submit",
        "new_chat",
        "delete_chat",
        "add_attachment",
        "paste_image",
        "change_model",
    ];
    for &expected_id in &expected {
        assert!(
            ids.contains(&expected_id),
            "AI command bar should have {}",
            expected_id
        );
    }
}

#[test]
fn ai_command_bar_actions_have_sections() {
    let actions = get_ai_command_bar_actions();

    for action in &actions {
        assert!(
            action.section.is_some(),
            "AI command bar action '{}' should have a section",
            action.id
        );
    }

    // Verify correct section assignments
    let response_ids = ["copy_response", "copy_chat", "copy_last_code"];
    for id in &response_ids {
        let a = find_action(&actions, id).unwrap();
        assert_eq!(
            a.section.as_deref(),
            Some("Response"),
            "{} should be in Response section",
            id
        );
    }

    let action_ids_list = ["submit", "new_chat", "delete_chat"];
    for id in &action_ids_list {
        let a = find_action(&actions, id).unwrap();
        assert_eq!(
            a.section.as_deref(),
            Some("Actions"),
            "{} should be in Actions section",
            id
        );
    }
}

#[test]
fn ai_command_bar_actions_have_icons() {
    let actions = get_ai_command_bar_actions();

    for action in &actions {
        assert!(
            action.icon.is_some(),
            "AI command bar action '{}' should have an icon",
            action.id
        );
    }
}

// ============================================================
// 7. Notes command bar: conditional actions
// ============================================================

#[test]
fn notes_command_bar_minimal_when_no_selection() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids = action_ids(&actions);

    assert!(
        ids.contains(&"new_note"),
        "new_note should always be present"
    );
    assert!(
        ids.contains(&"browse_notes"),
        "browse_notes should always be present"
    );
    assert!(
        !ids.contains(&"duplicate_note"),
        "duplicate_note requires selection"
    );
    assert!(
        !ids.contains(&"find_in_note"),
        "find_in_note requires selection"
    );
    assert!(!ids.contains(&"export"), "export requires selection");
}

#[test]
fn notes_command_bar_full_when_selected() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids = action_ids(&actions);

    let expected = [
        "new_note",
        "duplicate_note",
        "browse_notes",
        "find_in_note",
        "format",
        "copy_note_as",
        "copy_deeplink",
        "create_quicklink",
        "export",
    ];
    for &expected_id in &expected {
        assert!(
            ids.contains(&expected_id),
            "Notes with selection should have {}",
            expected_id
        );
    }
}

#[test]
fn notes_command_bar_trash_view_suppresses_editing() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids = action_ids(&actions);

    // Trash view with selection should NOT show editing/copy actions
    assert!(
        !ids.contains(&"duplicate_note"),
        "Trash view should not have duplicate_note"
    );
    assert!(
        !ids.contains(&"find_in_note"),
        "Trash view should not have find_in_note"
    );
    assert!(
        !ids.contains(&"export"),
        "Trash view should not have export"
    );

    // But new_note and browse_notes should still be available
    assert!(ids.contains(&"new_note"));
    assert!(ids.contains(&"browse_notes"));
}

#[test]
fn notes_command_bar_auto_sizing_toggle() {
    let disabled = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&disabled);
    let ids = action_ids(&actions);
    assert!(
        ids.contains(&"enable_auto_sizing"),
        "Should show enable_auto_sizing when disabled"
    );

    let enabled = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&enabled);
    let ids = action_ids(&actions);
    assert!(
        !ids.contains(&"enable_auto_sizing"),
        "Should NOT show enable_auto_sizing when already enabled"
    );
}

// ============================================================
