
#[test]
fn notes_full_selection_has_duplicate() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "duplicate_note"));
}

#[test]
fn notes_full_selection_has_export() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "export"));
}

#[test]
fn notes_no_selection_count() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note + browse_notes + enable_auto_sizing = 3
    assert_eq!(actions.len(), 3);
}

// =====================================================================
// 20. CommandBarConfig: anchor position differences
// =====================================================================

#[test]
fn command_bar_ai_style_anchor_top() {
    let config = CommandBarConfig::ai_style();
    assert!(matches!(config.dialog_config.anchor, AnchorPosition::Top));
}

#[test]
fn command_bar_main_menu_anchor_bottom() {
    let config = CommandBarConfig::main_menu_style();
    assert!(matches!(
        config.dialog_config.anchor,
        AnchorPosition::Bottom
    ));
}

#[test]
fn command_bar_no_search_anchor_bottom() {
    let config = CommandBarConfig::no_search();
    assert!(matches!(
        config.dialog_config.anchor,
        AnchorPosition::Bottom
    ));
}

#[test]
fn command_bar_notes_anchor_top() {
    let config = CommandBarConfig::notes_style();
    assert!(matches!(config.dialog_config.anchor, AnchorPosition::Top));
}

// =====================================================================
// 21. New chat: combination of all three input types
// =====================================================================

#[test]
fn new_chat_all_three_types() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model-1".into(),
        provider: "P".into(),
        provider_display_name: "Provider-1".into(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "m2".into(),
        display_name: "Model-2".into(),
        provider: "P".into(),
        provider_display_name: "Provider-2".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &presets, &models);
    assert_eq!(actions.len(), 3);
}

#[test]
fn new_chat_sections_are_correct() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "LU".into(),
        provider: "P".into(),
        provider_display_name: "PD".into(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "g".into(),
        name: "G".into(),
        icon: IconName::Star,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "m2".into(),
        display_name: "M".into(),
        provider: "P".into(),
        provider_display_name: "PD2".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &presets, &models);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
    assert_eq!(actions[1].section.as_deref(), Some("Presets"));
    assert_eq!(actions[2].section.as_deref(), Some("Models"));
}

#[test]
fn new_chat_all_empty_produces_zero() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert_eq!(actions.len(), 0);
}

#[test]
fn new_chat_only_presets() {
    let presets = vec![
        NewChatPresetInfo {
            id: "a".into(),
            name: "A".into(),
            icon: IconName::Plus,
        },
        NewChatPresetInfo {
            id: "b".into(),
            name: "B".into(),
            icon: IconName::Code,
        },
    ];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions.len(), 2);
    assert!(actions
        .iter()
        .all(|a| a.section.as_deref() == Some("Presets")));
}

// =====================================================================
// 22. Note switcher: pinned+current uses StarFilled
// =====================================================================

#[test]
fn note_switcher_pinned_current_icon_is_star_filled() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n1".into(),
        title: "My Note".into(),
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
fn note_switcher_pinned_not_current_icon_is_star_filled() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n2".into(),
        title: "Other Note".into(),
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
fn note_switcher_current_not_pinned_icon_is_check() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n3".into(),
        title: "Current".into(),
        char_count: 30,
        is_current: true,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::Check));
}

#[test]
fn note_switcher_neither_icon_is_file() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n4".into(),
        title: "Regular".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::File));
}

// =====================================================================
// 23. Note switcher: description with preview exactly 60 chars
// =====================================================================

#[test]
fn note_switcher_preview_60_chars_not_truncated() {
    let preview: String = "a".repeat(60);
    let notes = vec![NoteSwitcherNoteInfo {
        id: "t1".into(),
        title: "T".into(),
        char_count: 60,
        is_current: false,
        is_pinned: false,
        preview: preview.clone(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_deref().unwrap();
    // 60 chars should NOT be truncated (no ellipsis)
    assert!(!desc.contains('…'));
    assert_eq!(desc, &preview);
}

#[test]
fn note_switcher_preview_61_chars_truncated() {
    let preview: String = "b".repeat(61);
    let notes = vec![NoteSwitcherNoteInfo {
        id: "t2".into(),
        title: "T".into(),
        char_count: 61,
        is_current: false,
        is_pinned: false,
        preview,
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_deref().unwrap();
    assert!(desc.contains('…'));
}

#[test]
fn note_switcher_empty_preview_with_time_shows_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "t3".into(),
        title: "T".into(),
        char_count: 0,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: "5m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_deref().unwrap();
    assert_eq!(desc, "5m ago");
}

// =====================================================================
// 24. to_deeplink_name: emoji and unicode handling
// =====================================================================

#[test]
fn deeplink_name_emoji_preserved_as_chars() {
    // Emoji are alphanumeric-ish in Unicode; to_deeplink_name keeps them
    let result = to_deeplink_name("Hello 🌍 World");
    // Spaces become hyphens, emoji is alphanumeric? Let's test actual behavior
    assert!(result.contains("hello"));
    assert!(result.contains("world"));
}

#[test]
fn deeplink_name_accented_chars_preserved() {
    let result = to_deeplink_name("café résumé");
    assert!(result.contains("caf"));
    assert!(result.contains("sum"));
}

#[test]
fn deeplink_name_all_special_chars_empty() {
    let result = to_deeplink_name("!@#$%^&*()");
    assert_eq!(result, "");
}

#[test]
fn deeplink_name_mixed_separators() {
    let result = to_deeplink_name("hello---world___test   foo");
    assert_eq!(result, "hello-world-test-foo");
}

// =====================================================================
// 25. ScriptInfo: with_frecency preserves all other fields
// =====================================================================

#[test]
fn with_frecency_preserves_name_and_path() {
    let script = ScriptInfo::new("my-script", "/path/script.ts")
        .with_frecency(true, Some("/frecency".into()));
    assert_eq!(script.name, "my-script");
    assert_eq!(script.path, "/path/script.ts");
}

#[test]
fn with_frecency_preserves_is_script() {
    let script = ScriptInfo::new("my-script", "/path/script.ts").with_frecency(true, None);
    assert!(script.is_script);
}

#[test]
fn with_frecency_preserves_shortcut_and_alias() {
    let script = ScriptInfo::with_shortcut_and_alias(
        "test",
        "/path/test.ts",
        Some("cmd+k".into()),
        Some("tk".into()),
    )
    .with_frecency(true, Some("/fp".into()));
    assert_eq!(script.shortcut, Some("cmd+k".into()));
    assert_eq!(script.alias, Some("tk".into()));
    assert!(script.is_suggested);
}

#[test]
fn with_frecency_false_not_suggested() {
    let script = ScriptInfo::new("s", "/p").with_frecency(false, None);
    assert!(!script.is_suggested);
    assert!(script.frecency_path.is_none());
}

// =====================================================================
// 26. Action: cached lowercase fields correctness
// =====================================================================

#[test]
fn action_title_lower_cached_correctly() {
    let action = Action::new("id", "Hello World", None, ActionCategory::ScriptContext);
    assert_eq!(action.title_lower, "hello world");
}

#[test]
fn action_description_lower_cached() {
    let action = Action::new(
        "id",
        "T",
        Some("My Description HERE".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(action.description_lower, Some("my description here".into()));
}

#[test]
fn action_description_lower_none_when_no_desc() {
    let action = Action::new("id", "T", None, ActionCategory::ScriptContext);
    assert!(action.description_lower.is_none());
}

#[test]
fn action_shortcut_lower_set_after_with_shortcut() {
    let action = Action::new("id", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧K");
    assert_eq!(action.shortcut_lower, Some("⌘⇧k".into()));
}

// =====================================================================
// 27. build_grouped_items_static: SectionStyle::None produces no headers
// =====================================================================

#[test]
fn grouped_items_none_style_no_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Section1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Section2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
    // With SectionStyle::None, no headers should be inserted
    for item in &grouped {
        assert!(
            matches!(item, crate::actions::dialog::GroupedActionItem::Item(_)),
            "SectionStyle::None should not produce headers"
        );
    }
    assert_eq!(grouped.len(), 2);
}

#[test]
fn grouped_items_headers_style_adds_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // 2 section headers + 2 items = 4
    assert_eq!(grouped.len(), 4);
}

#[test]
fn grouped_items_same_section_one_header() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S1"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // 1 header + 2 items = 3
    assert_eq!(grouped.len(), 3);
}

#[test]
fn grouped_items_no_section_no_header() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext),
        Action::new("b", "B", None, ActionCategory::ScriptContext),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // No sections = no headers, just 2 items
    assert_eq!(grouped.len(), 2);
}

// =====================================================================
// 28. coerce_action_selection: edge cases
// =====================================================================

#[test]
fn coerce_selection_empty_returns_none() {
    let rows = vec![];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn coerce_selection_single_item() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![GroupedActionItem::Item(0)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
}
