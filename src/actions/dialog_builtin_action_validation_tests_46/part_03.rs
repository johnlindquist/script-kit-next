
#[test]
fn notes_copy_note_as_absent_no_selection() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "copy_note_as"));
}

#[test]
fn notes_copy_note_as_absent_in_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "copy_note_as"));
}

// =========== 23. Chat: copy_response conditional on has_response ===========

#[test]
fn chat_copy_response_present_when_has_response() {
    let info = ChatPromptInfo {
        current_model: Some("Claude".into()),
        available_models: vec![],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "copy_response"));
}

#[test]
fn chat_copy_response_absent_when_no_response() {
    let info = ChatPromptInfo {
        current_model: Some("Claude".into()),
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "copy_response"));
}

#[test]
fn chat_copy_response_shortcut() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let cr = actions.iter().find(|a| a.id == "copy_response").unwrap();
    assert_eq!(cr.shortcut.as_deref(), Some("⌘C"));
}

#[test]
fn chat_copy_response_title() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let cr = actions.iter().find(|a| a.id == "copy_response").unwrap();
    assert_eq!(cr.title, "Copy Last Response");
}

// =========== 24. Chat: clear_conversation conditional on has_messages ===========

#[test]
fn chat_clear_conversation_present_when_has_messages() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "clear_conversation"));
}

#[test]
fn chat_clear_conversation_absent_when_no_messages() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "clear_conversation"));
}

#[test]
fn chat_clear_conversation_shortcut() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let cc = actions
        .iter()
        .find(|a| a.id == "clear_conversation")
        .unwrap();
    assert_eq!(cc.shortcut.as_deref(), Some("⌘⌫"));
}

#[test]
fn chat_clear_conversation_title() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let cc = actions
        .iter()
        .find(|a| a.id == "clear_conversation")
        .unwrap();
    assert_eq!(cc.title, "Clear Conversation");
}

// =========== 25. New chat: empty inputs produce empty actions ===========

#[test]
fn new_chat_all_empty_produces_empty() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

#[test]
fn new_chat_only_last_used_count() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Claude".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions.len(), 1);
}

#[test]
fn new_chat_only_presets_count() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions.len(), 1);
}

#[test]
fn new_chat_only_models_count() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions.len(), 1);
}

// =========== 26. New chat: model ID format uses index ===========

#[test]
fn new_chat_model_id_format() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].id, "model_0");
}

#[test]
fn new_chat_last_used_id_format() {
    let last_used = vec![NewChatModelInfo {
        model_id: "claude".into(),
        display_name: "Claude".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].id, "last_used_0");
}

#[test]
fn new_chat_preset_id_format() {
    let presets = vec![NewChatPresetInfo {
        id: "code".into(),
        name: "Code".into(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].id, "preset_code");
}

#[test]
fn new_chat_combined_ordering() {
    let last_used = vec![NewChatModelInfo {
        model_id: "c".into(),
        display_name: "Claude".into(),
        provider: "a".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "gen".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "g".into(),
        display_name: "GPT-4".into(),
        provider: "o".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &presets, &models);
    assert_eq!(actions.len(), 3);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
    assert_eq!(actions[1].section.as_deref(), Some("Presets"));
    assert_eq!(actions[2].section.as_deref(), Some("Models"));
}

// =========== 27. Note switcher: empty notes produces "no notes yet" ===========

#[test]
fn note_switcher_empty_has_no_notes_message() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "no_notes");
}

#[test]
fn note_switcher_no_notes_title() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions[0].title, "No notes yet");
}

#[test]
fn note_switcher_no_notes_icon() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions[0].icon, Some(IconName::Plus));
}

#[test]
fn note_switcher_no_notes_desc_mentions_cmd_n() {
    let actions = get_note_switcher_actions(&[]);
    assert!(actions[0].description.as_ref().unwrap().contains("⌘N"));
}

// =========== 28. Note switcher: char count display when no preview ===========

#[test]
fn note_switcher_no_preview_shows_char_count_singular() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Note".into(),
        char_count: 1,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("1 char"));
}

#[test]
fn note_switcher_no_preview_shows_char_count_plural() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Note".into(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("42 chars"));
}

#[test]
fn note_switcher_no_preview_with_time_shows_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Note".into(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "5m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("5m ago"));
}

#[test]
fn note_switcher_with_preview_and_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Note".into(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "Hello world".into(),
        relative_time: "2d ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(
        actions[0].description.as_deref(),
        Some("Hello world · 2d ago")
    );
}

// =========== 29. coerce_action_selection: mixed headers and items ===========

#[test]
fn coerce_selection_first_header_then_items() {
    let rows = vec![
        GroupedActionItem::SectionHeader("H".into()),
        GroupedActionItem::Item(0),
        GroupedActionItem::Item(1),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn coerce_selection_item_between_headers() {
    let rows = vec![
        GroupedActionItem::SectionHeader("H1".into()),
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("H2".into()),
        GroupedActionItem::Item(1),
    ];
    assert_eq!(coerce_action_selection(&rows, 2), Some(3));
}

#[test]
fn coerce_selection_trailing_header_searches_up() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("H".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn coerce_selection_multiple_headers_between_items() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("H1".into()),
        GroupedActionItem::SectionHeader("H2".into()),
        GroupedActionItem::Item(1),
    ];
    // Index 1 is header, search down → finds Item(1) at index 3
    assert_eq!(coerce_action_selection(&rows, 1), Some(3));
}

// =========== 30. build_grouped_items_static: action count matches filtered ===========

#[test]
fn build_grouped_items_item_count_matches_filtered() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext),
        Action::new("b", "B", None, ActionCategory::ScriptContext),
        Action::new("c", "C", None, ActionCategory::ScriptContext),
    ];
    let filtered = vec![0usize, 1, 2];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
    let item_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::Item(_)))
        .count();
    assert_eq!(item_count, 3);
}

#[test]
fn build_grouped_items_headers_from_sections() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered = vec![0usize, 1];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    let header_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(header_count, 2);
}

#[test]
fn build_grouped_items_no_headers_with_none_style() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered = vec![0usize, 1];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
    let header_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(header_count, 0);
}

#[test]
fn build_grouped_items_same_section_one_header() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Same"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Same"),
        Action::new("c", "C", None, ActionCategory::ScriptContext).with_section("Same"),
    ];
    let filtered = vec![0usize, 1, 2];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    let header_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(header_count, 1);
}
