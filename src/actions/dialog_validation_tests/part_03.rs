
// =========================================================================
// coerce_action_selection edge cases
// =========================================================================

#[test]
fn test_coerce_empty_rows() {
    assert_eq!(coerce_action_selection(&[], 0), None);
}

#[test]
fn test_coerce_single_item() {
    let rows = vec![GroupedActionItem::Item(0)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
}

#[test]
fn test_coerce_single_header() {
    let rows = vec![GroupedActionItem::SectionHeader("Test".to_string())];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn test_coerce_header_then_item() {
    let rows = vec![
        GroupedActionItem::SectionHeader("Test".to_string()),
        GroupedActionItem::Item(0),
    ];
    // Landing on header (index 0) should search down to find item at index 1
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn test_coerce_item_then_header() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("Test".to_string()),
    ];
    // Landing on header (index 1) should search up to find item at index 0
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn test_coerce_clamps_beyond_bounds() {
    let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
    // Index 100 should be clamped to last valid
    assert_eq!(coerce_action_selection(&rows, 100), Some(1));
}

#[test]
fn test_coerce_consecutive_headers_at_start() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".to_string()),
        GroupedActionItem::SectionHeader("B".to_string()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(2));
    assert_eq!(coerce_action_selection(&rows, 1), Some(2));
}

// =========================================================================
// AI command bar action invariants
// =========================================================================

#[test]
fn test_ai_command_bar_all_have_icons() {
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        assert!(
            action.icon.is_some(),
            "AI command bar action '{}' should have an icon",
            action.id
        );
    }
}

#[test]
fn test_ai_command_bar_all_have_sections() {
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        assert!(
            action.section.is_some(),
            "AI command bar action '{}' should have a section",
            action.id
        );
    }
}

#[test]
fn test_ai_command_bar_section_ordering() {
    let actions = get_ai_command_bar_actions();
    let sections: Vec<&str> = actions
        .iter()
        .filter_map(|a| a.section.as_deref())
        .collect();

    // Sections should appear in order: Response, Actions, Attachments, Settings
    let first_response = sections.iter().position(|&s| s == "Response").unwrap();
    let first_actions = sections.iter().position(|&s| s == "Actions").unwrap();
    let first_attachments = sections.iter().position(|&s| s == "Attachments").unwrap();
    let first_settings = sections.iter().position(|&s| s == "Settings").unwrap();

    assert!(first_response < first_actions);
    assert!(first_actions < first_attachments);
    assert!(first_attachments < first_settings);
}

#[test]
fn test_ai_command_bar_has_expected_ids() {
    let actions = get_ai_command_bar_actions();
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    assert!(ids.contains(&"copy_response"));
    assert!(ids.contains(&"copy_chat"));
    assert!(ids.contains(&"copy_last_code"));
    assert!(ids.contains(&"submit"));
    assert!(ids.contains(&"new_chat"));
    assert!(ids.contains(&"delete_chat"));
    assert!(ids.contains(&"add_attachment"));
    assert!(ids.contains(&"paste_image"));
    assert!(ids.contains(&"change_model"));
    assert!(ids.contains(&"export_markdown"));
    assert!(ids.contains(&"branch_from_last"));
    assert!(ids.contains(&"toggle_shortcuts_help"));
    assert_eq!(ids.len(), 12);
}

// =========================================================================
// Chat context action variations
// =========================================================================

#[test]
fn test_chat_no_models_no_messages() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };

    let actions = get_chat_context_actions(&info);
    // Should only have continue_in_chat
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "continue_in_chat");
}

#[test]
fn test_chat_with_models_and_response() {
    let info = ChatPromptInfo {
        current_model: Some("Claude 3.5 Sonnet".to_string()),
        available_models: vec![
            ChatModelInfo {
                id: "claude-3-5-sonnet".to_string(),
                display_name: "Claude 3.5 Sonnet".to_string(),
                provider: "Anthropic".to_string(),
            },
            ChatModelInfo {
                id: "gpt-4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            },
        ],
        has_messages: true,
        has_response: true,
    };

    let actions = get_chat_context_actions(&info);
    // 2 models + continue_in_chat + copy_response + clear_conversation = 5
    assert_eq!(actions.len(), 5);

    // Current model should have checkmark
    let current = actions
        .iter()
        .find(|a| a.id == "select_model_claude-3-5-sonnet")
        .unwrap();
    assert!(current.title.contains("✓"));

    // Other model should not
    let other = actions
        .iter()
        .find(|a| a.id == "select_model_gpt-4")
        .unwrap();
    assert!(!other.title.contains("✓"));
}

#[test]
fn test_chat_messages_but_no_response() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };

    let actions = get_chat_context_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"clear_conversation"));
    assert!(!ids.contains(&"copy_response"));
}

// =========================================================================
// Notes command bar permutations
// =========================================================================

#[test]
fn test_notes_no_selection_no_trash() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };

    let actions = get_notes_command_bar_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    // new_note and browse_notes are always present
    assert!(ids.contains(&"new_note"));
    assert!(ids.contains(&"browse_notes"));
    // Selection-gated actions should NOT be present
    assert!(!ids.contains(&"duplicate_note"));
    assert!(!ids.contains(&"find_in_note"));
    assert!(!ids.contains(&"format"));
    assert!(!ids.contains(&"copy_note_as"));
    // Auto-sizing should be offered when disabled
    assert!(ids.contains(&"enable_auto_sizing"));
}

#[test]
fn test_notes_with_selection_not_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };

    let actions = get_notes_command_bar_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    assert!(ids.contains(&"new_note"));
    assert!(ids.contains(&"browse_notes"));
    assert!(ids.contains(&"duplicate_note"));
    assert!(ids.contains(&"find_in_note"));
    assert!(ids.contains(&"format"));
    assert!(ids.contains(&"copy_note_as"));
    assert!(ids.contains(&"copy_deeplink"));
    assert!(ids.contains(&"create_quicklink"));
    assert!(ids.contains(&"export"));
    // Auto-sizing already enabled -> should NOT show enable action
    assert!(!ids.contains(&"enable_auto_sizing"));
}

#[test]
fn test_notes_trash_view_suppresses_edit_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };

    let actions = get_notes_command_bar_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    // new_note always present
    assert!(ids.contains(&"new_note"));
    // Selection + trash view = no edit actions
    assert!(!ids.contains(&"duplicate_note"));
    assert!(!ids.contains(&"find_in_note"));
    assert!(!ids.contains(&"format"));
    assert!(!ids.contains(&"copy_note_as"));
}

#[test]
fn test_notes_all_actions_have_icons() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };

    let actions = get_notes_command_bar_actions(&info);
    for action in &actions {
        assert!(
            action.icon.is_some(),
            "Notes action '{}' should have an icon",
            action.id
        );
    }
}

#[test]
fn test_notes_all_actions_have_sections() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };

    let actions = get_notes_command_bar_actions(&info);
    for action in &actions {
        assert!(
            action.section.is_some(),
            "Notes action '{}' should have a section",
            action.id
        );
    }
}

// =========================================================================
// Note switcher edge cases
// =========================================================================

#[test]
fn test_note_switcher_empty_notes() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "no_notes");
    assert_eq!(actions[0].title, "No notes yet");
}

#[test]
fn test_note_switcher_current_note_bullet() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc-123".to_string(),
        title: "My Note".to_string(),
        char_count: 42,
        is_current: true,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];

    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].title, "• My Note");
}

#[test]
fn test_note_switcher_non_current_no_bullet() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc-123".to_string(),
        title: "My Note".to_string(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];

    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].title, "My Note");
}

#[test]
fn test_note_switcher_pinned_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc-123".to_string(),
        title: "Pinned Note".to_string(),
        char_count: 100,
        is_current: false,
        is_pinned: true,
        preview: String::new(),
        relative_time: String::new(),
    }];

    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn test_note_switcher_current_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc-123".to_string(),
        title: "Current Note".to_string(),
        char_count: 100,
        is_current: true,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];

    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::Check));
}

#[test]
fn test_note_switcher_pinned_priority_over_current() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc-123".to_string(),
        title: "Both".to_string(),
        char_count: 100,
        is_current: true,
        is_pinned: true,
        preview: String::new(),
        relative_time: String::new(),
    }];

    let actions = get_note_switcher_actions(&notes);
    // Pinned icon takes priority
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn test_note_switcher_char_count_singular() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc-123".to_string(),
        title: "Note".to_string(),
        char_count: 1,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];

    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description, Some("1 char".to_string()));
}

#[test]
fn test_note_switcher_char_count_plural() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc-123".to_string(),
        title: "Note".to_string(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];

    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description, Some("42 chars".to_string()));
}

#[test]
fn test_note_switcher_char_count_zero() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc-123".to_string(),
        title: "Empty".to_string(),
        char_count: 0,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];

    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description, Some("0 chars".to_string()));
}
