
#[test]
fn agent_with_shortcut_shows_update_not_add() {
    let mut agent = ScriptInfo::new("my-agent", "/path/agent.md");
    agent.is_agent = true;
    agent.is_script = false;
    agent.shortcut = Some("cmd+a".into());

    let actions = get_script_context_actions(&agent);
    let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();
    assert!(ids.contains("update_shortcut"));
    assert!(!ids.contains("add_shortcut"));
    assert!(ids.contains("edit_script")); // agent gets edit_script with title "Edit Agent"
}

// =========================================================================
// 13. Note switcher icon hierarchy for all is_current × is_pinned combos
// =========================================================================

#[test]
fn note_switcher_pinned_current_gets_star_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n1".into(),
        title: "Note 1".into(),
        char_count: 100,
        is_current: true,
        is_pinned: true,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn note_switcher_pinned_not_current_gets_star_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n2".into(),
        title: "Note 2".into(),
        char_count: 50,
        is_current: false,
        is_pinned: true,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn note_switcher_current_not_pinned_gets_check_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n3".into(),
        title: "Note 3".into(),
        char_count: 25,
        is_current: true,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::Check));
}

#[test]
fn note_switcher_neither_pinned_nor_current_gets_file_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n4".into(),
        title: "Note 4".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::File));
}

#[test]
fn note_switcher_current_note_has_bullet_prefix() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "a".into(),
            title: "Current Note".into(),
            char_count: 100,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "b".into(),
            title: "Other Note".into(),
            char_count: 50,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    assert!(
        actions[0].title.starts_with("• "),
        "Current note should have bullet prefix"
    );
    assert!(
        !actions[1].title.starts_with("• "),
        "Non-current note should not have bullet prefix"
    );
}

#[test]
fn note_switcher_char_count_plural() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "p0".into(),
            title: "Zero".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "p1".into(),
            title: "One".into(),
            char_count: 1,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "p2".into(),
            title: "Many".into(),
            char_count: 42,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("0 chars"));
    assert_eq!(actions[1].description.as_deref(), Some("1 char"));
    assert_eq!(actions[2].description.as_deref(), Some("42 chars"));
}

#[test]
fn note_switcher_empty_shows_no_notes_message() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "no_notes");
    assert_eq!(actions[0].title, "No notes yet");
    assert_eq!(actions[0].icon, Some(IconName::Plus));
}

// =========================================================================
// 14. to_deeplink_name with unicode / emoji edge cases
// =========================================================================

#[test]
fn deeplink_name_with_accented_chars() {
    // to_deeplink_name lowercases and replaces non-alphanumeric with hyphens,
    // but accented latin chars like 'é' are alphanumeric in Unicode
    assert_eq!(to_deeplink_name("café"), "café");
}

#[test]
fn deeplink_name_with_numbers() {
    assert_eq!(to_deeplink_name("Script123"), "script123");
}

#[test]
fn deeplink_name_empty_string() {
    assert_eq!(to_deeplink_name(""), "");
}

#[test]
fn deeplink_name_only_special_chars() {
    assert_eq!(to_deeplink_name("!@#$%"), "");
}

#[test]
fn deeplink_name_leading_trailing_spaces() {
    assert_eq!(to_deeplink_name("  hello  "), "hello");
}

#[test]
fn deeplink_name_consecutive_hyphens_collapsed() {
    assert_eq!(to_deeplink_name("a---b"), "a-b");
}

#[test]
fn deeplink_name_mixed_case_numbers_symbols() {
    assert_eq!(to_deeplink_name("My Script (v2.0)"), "my-script-v2-0");
}

// =========================================================================
// 15. Score stacking — title + description bonuses accumulate
// =========================================================================

#[test]
fn score_prefix_match_is_100() {
    let action = Action::new(
        "edit_script",
        "Edit Script",
        Some("Open in editor".to_string()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "edit");
    assert_eq!(
        score,
        100 + 15,
        "Prefix 'edit' should get 100 for title + 15 for description containing 'edit'"
    );
}

#[test]
fn score_contains_match_is_50() {
    let action = Action::new(
        "copy_path",
        "Copy Path",
        Some("Copy to clipboard".to_string()),
        ActionCategory::ScriptContext,
    );
    // "path" is contained but not a prefix
    let score = ActionsDialog::score_action(&action, "path");
    assert!(
        score >= 50,
        "Contains match should be at least 50, got {}",
        score
    );
}

#[test]
fn score_description_only_match() {
    let action = Action::new(
        "open_file",
        "Open File",
        Some("Launch with default application".to_string()),
        ActionCategory::ScriptContext,
    );
    // "launch" is in description but not title
    let score = ActionsDialog::score_action(&action, "launch");
    assert_eq!(score, 15, "'launch' only in description should give 15");
}

#[test]
fn score_shortcut_only_match() {
    let action = Action::new(
        "edit_script",
        "Edit Script",
        Some("Open in editor".to_string()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘E");
    // "⌘e" matches shortcut but not title or description
    let score = ActionsDialog::score_action(&action, "⌘e");
    assert!(
        score >= 10,
        "Shortcut match should give at least 10, got {}",
        score
    );
}

#[test]
fn score_no_match_is_zero() {
    let action = Action::new(
        "run_script",
        "Run Script",
        Some("Execute this item".to_string()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "zzzzz");
    assert_eq!(score, 0, "No match should score 0");
}

#[test]
fn score_prefix_plus_description_stack() {
    let action = Action::new(
        "copy_path",
        "Copy Path",
        Some("Copy the full path to clipboard".to_string()),
        ActionCategory::ScriptContext,
    );
    // "copy" is a prefix of title AND contained in description
    let score = ActionsDialog::score_action(&action, "copy");
    assert_eq!(
        score,
        100 + 15,
        "Prefix + description match should stack: 100 + 15 = 115, got {}",
        score
    );
}

// =========================================================================
// 16. File context primary title includes filename
// =========================================================================

#[test]
fn file_context_primary_title_includes_filename() {
    let file = FileInfo {
        path: "/Users/test/document.pdf".into(),
        name: "document.pdf".into(),
        file_type: FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    assert!(
        actions[0].title.contains("document.pdf"),
        "File primary title '{}' should include filename",
        actions[0].title
    );
}

#[test]
fn file_context_dir_primary_title_includes_dirname() {
    let file = FileInfo {
        path: "/Users/test/Documents".into(),
        name: "Documents".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&file);
    assert!(
        actions[0].title.contains("Documents"),
        "Dir primary title '{}' should include dirname",
        actions[0].title
    );
}

// =========================================================================
// 17. Chat model checkmark only on current model
// =========================================================================

#[test]
fn chat_model_checkmark_on_current_only() {
    let info = ChatPromptInfo {
        current_model: Some("Claude 3".to_string()),
        available_models: vec![
            ChatModelInfo {
                id: "claude-3".into(),
                display_name: "Claude 3".into(),
                provider: "Anthropic".into(),
            },
            ChatModelInfo {
                id: "gpt-4".into(),
                display_name: "GPT-4".into(),
                provider: "OpenAI".into(),
            },
            ChatModelInfo {
                id: "gemini".into(),
                display_name: "Gemini".into(),
                provider: "Google".into(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);

    // Claude 3 should have checkmark
    let claude = find_action(&actions, "select_model_claude-3").unwrap();
    assert!(
        claude.title.contains('✓'),
        "Current model should have checkmark"
    );

    // Others should not
    let gpt = find_action(&actions, "select_model_gpt-4").unwrap();
    assert!(
        !gpt.title.contains('✓'),
        "Non-current model should not have checkmark"
    );

    let gemini = find_action(&actions, "select_model_gemini").unwrap();
    assert!(
        !gemini.title.contains('✓'),
        "Non-current model should not have checkmark"
    );
}

#[test]
fn chat_no_current_model_no_checkmarks() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![
            ChatModelInfo {
                id: "m1".into(),
                display_name: "Model 1".into(),
                provider: "P1".into(),
            },
            ChatModelInfo {
                id: "m2".into(),
                display_name: "Model 2".into(),
                provider: "P2".into(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    for a in &actions {
        if a.id.starts_with("select_model_") {
            assert!(
                !a.title.contains('✓'),
                "No model should have checkmark when current_model is None"
            );
        }
    }
}

#[test]
fn chat_continue_in_chat_always_present() {
    // Even with no models, continue_in_chat should be present
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(
        actions.iter().any(|a| a.id == "continue_in_chat"),
        "continue_in_chat should always be present"
    );
}
