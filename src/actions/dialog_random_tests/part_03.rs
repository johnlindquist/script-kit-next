
#[test]
fn notes_duplicate_only_when_selected_and_not_trash() {
    assert!(notes_action_ids(true, false, false).contains(&"duplicate_note".to_string()));
    assert!(!notes_action_ids(false, false, false).contains(&"duplicate_note".to_string()));
    assert!(!notes_action_ids(true, true, false).contains(&"duplicate_note".to_string()));
}

#[test]
fn notes_edit_actions_only_when_selected_and_not_trash() {
    let with = notes_action_ids(true, false, false);
    assert!(with.contains(&"find_in_note".to_string()));
    assert!(with.contains(&"format".to_string()));

    let without_sel = notes_action_ids(false, false, false);
    assert!(!without_sel.contains(&"find_in_note".to_string()));

    let trash = notes_action_ids(true, true, false);
    assert!(!trash.contains(&"find_in_note".to_string()));
}

#[test]
fn notes_auto_sizing_toggle() {
    // auto_sizing disabled → show enable action
    assert!(notes_action_ids(false, false, false).contains(&"enable_auto_sizing".to_string()));
    // auto_sizing enabled → no enable action
    assert!(!notes_action_ids(false, false, true).contains(&"enable_auto_sizing".to_string()));
}

#[test]
fn notes_all_actions_have_icons() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    for action in &actions {
        assert!(action.icon.is_some(), "Action '{}' missing icon", action.id);
    }
}

#[test]
fn notes_all_actions_have_sections() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    for action in &actions {
        assert!(
            action.section.is_some(),
            "Action '{}' missing section",
            action.id
        );
    }
}

// =========================================================================
// 14. AI command bar actions
// =========================================================================

#[test]
fn ai_command_bar_all_twelve_ids() {
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
        "export_markdown",
        "branch_from_last",
        "toggle_shortcuts_help",
    ];
    for id in &expected {
        assert!(ids.contains(id), "Missing AI action: {}", id);
    }
    assert_eq!(actions.len(), 12);
}

#[test]
fn ai_command_bar_section_ordering() {
    let actions = get_ai_command_bar_actions();
    let sections: Vec<&str> = actions
        .iter()
        .filter_map(|a| a.section.as_deref())
        .collect();
    // Response comes before Actions, Actions before Attachments, Attachments before Settings
    let resp_idx = sections.iter().position(|&s| s == "Response").unwrap();
    let act_idx = sections.iter().position(|&s| s == "Actions").unwrap();
    let att_idx = sections.iter().position(|&s| s == "Attachments").unwrap();
    let set_idx = sections.iter().position(|&s| s == "Settings").unwrap();
    assert!(resp_idx < act_idx);
    assert!(act_idx < att_idx);
    assert!(att_idx < set_idx);
}

#[test]
fn ai_command_bar_all_have_icons() {
    for action in &get_ai_command_bar_actions() {
        assert!(
            action.icon.is_some(),
            "AI action '{}' missing icon",
            action.id
        );
    }
}

// =========================================================================
// 15. Note switcher actions
// =========================================================================

#[test]
fn note_switcher_empty_shows_no_notes() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "no_notes");
    assert_eq!(actions[0].title, "No notes yet");
}

#[test]
fn note_switcher_current_has_bullet() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".to_string(),
        title: "My Note".to_string(),
        char_count: 42,
        is_current: true,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(actions[0].title.starts_with("• "));
}

#[test]
fn note_switcher_non_current_no_bullet() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".to_string(),
        title: "My Note".to_string(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(!actions[0].title.starts_with("• "));
}

#[test]
fn note_switcher_pinned_gets_star_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".to_string(),
        title: "Pinned".to_string(),
        char_count: 10,
        is_current: false,
        is_pinned: true,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn note_switcher_current_gets_check_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".to_string(),
        title: "Current".to_string(),
        char_count: 10,
        is_current: true,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::Check));
}

#[test]
fn note_switcher_pinned_priority_over_current() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".to_string(),
        title: "Both".to_string(),
        char_count: 10,
        is_current: true,
        is_pinned: true,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    // Pinned takes priority: StarFilled
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn note_switcher_char_count_singular() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".to_string(),
        title: "Single".to_string(),
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
fn note_switcher_char_count_plural() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".to_string(),
        title: "Multi".to_string(),
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
fn note_switcher_char_count_zero() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".to_string(),
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

#[test]
fn note_switcher_all_have_notes_section() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "1".to_string(),
            title: "A".to_string(),
            char_count: 1,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "2".to_string(),
            title: "B".to_string(),
            char_count: 2,
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
// 16. New chat actions
// =========================================================================

#[test]
fn new_chat_empty_inputs() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

#[test]
fn new_chat_section_ordering() {
    let last_used = vec![NewChatModelInfo {
        model_id: "c3".to_string(),
        display_name: "Claude 3".to_string(),
        provider: "anthropic".to_string(),
        provider_display_name: "Anthropic".to_string(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "gen".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "g4".to_string(),
        display_name: "GPT-4".to_string(),
        provider: "openai".to_string(),
        provider_display_name: "OpenAI".to_string(),
    }];
    let actions = get_new_chat_actions(&last_used, &presets, &models);
    let sections: Vec<&str> = actions
        .iter()
        .filter_map(|a| a.section.as_deref())
        .collect();
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
fn new_chat_all_have_icons() {
    let last_used = vec![NewChatModelInfo {
        model_id: "c3".to_string(),
        display_name: "Claude 3".to_string(),
        provider: "anthropic".to_string(),
        provider_display_name: "Anthropic".to_string(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "gen".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "g4".to_string(),
        display_name: "GPT-4".to_string(),
        provider: "openai".to_string(),
        provider_display_name: "OpenAI".to_string(),
    }];
    let actions = get_new_chat_actions(&last_used, &presets, &models);
    for action in &actions {
        assert!(
            action.icon.is_some(),
            "New chat action '{}' missing icon",
            action.id
        );
    }
}

#[test]
fn new_chat_last_used_has_bolt_icon() {
    let last_used = vec![NewChatModelInfo {
        model_id: "c3".to_string(),
        display_name: "Claude 3".to_string(),
        provider: "anthropic".to_string(),
        provider_display_name: "Anthropic".to_string(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
}

#[test]
fn new_chat_models_has_settings_icon() {
    let models = vec![NewChatModelInfo {
        model_id: "g4".to_string(),
        display_name: "GPT-4".to_string(),
        provider: "openai".to_string(),
        provider_display_name: "OpenAI".to_string(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].icon, Some(IconName::Settings));
}

// =========================================================================
// 17. CommandBarConfig presets
// =========================================================================

#[test]
fn command_bar_default_config() {
    let config = CommandBarConfig::default();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Bottom);
    assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
    assert_eq!(config.dialog_config.anchor, AnchorPosition::Bottom);
    assert!(!config.dialog_config.show_icons);
    assert!(!config.dialog_config.show_footer);
    assert!(config.close_on_select);
    assert!(config.close_on_click_outside);
    assert!(config.close_on_escape);
}

#[test]
fn command_bar_ai_style() {
    let config = CommandBarConfig::ai_style();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
    assert_eq!(config.dialog_config.section_style, SectionStyle::Headers);
    assert_eq!(config.dialog_config.anchor, AnchorPosition::Top);
    assert!(config.dialog_config.show_icons);
    assert!(config.dialog_config.show_footer);
}

#[test]
fn command_bar_notes_style() {
    let config = CommandBarConfig::notes_style();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
    assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
    assert_eq!(config.dialog_config.anchor, AnchorPosition::Top);
    assert!(config.dialog_config.show_icons);
    assert!(config.dialog_config.show_footer);
}

#[test]
fn command_bar_no_search() {
    let config = CommandBarConfig::no_search();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Hidden);
}

#[test]
fn command_bar_main_menu_style() {
    let config = CommandBarConfig::main_menu_style();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Bottom);
    assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
    assert!(!config.dialog_config.show_icons);
    assert!(!config.dialog_config.show_footer);
}

// =========================================================================
// 18. Action lowercase caching
// =========================================================================

#[test]
fn action_title_lower_is_cached() {
    let action = Action::new(
        "test",
        "UPPERCASE TITLE",
        None,
        ActionCategory::ScriptContext,
    );
    assert_eq!(action.title_lower, "uppercase title");
}
