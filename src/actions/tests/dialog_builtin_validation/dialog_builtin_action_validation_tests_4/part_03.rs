
#[test]
fn format_shortcut_hint_cmd_enter() {
    let hint = ActionsDialog::format_shortcut_hint("cmd+enter");
    assert_eq!(hint, "⌘↵");
}

#[test]
fn format_shortcut_hint_ctrl_alt_delete() {
    let hint = ActionsDialog::format_shortcut_hint("ctrl+alt+delete");
    assert_eq!(hint, "⌃⌥⌫");
}

#[test]
fn format_shortcut_hint_shift_cmd_c() {
    let hint = ActionsDialog::format_shortcut_hint("shift+cmd+c");
    assert_eq!(hint, "⇧⌘C");
}

#[test]
fn format_shortcut_hint_option_variant() {
    let hint = ActionsDialog::format_shortcut_hint("option+a");
    assert_eq!(hint, "⌥A");
}

#[test]
fn format_shortcut_hint_command_variant() {
    let hint = ActionsDialog::format_shortcut_hint("command+s");
    assert_eq!(hint, "⌘S");
}

#[test]
fn format_shortcut_hint_arrowup_variant() {
    let hint = ActionsDialog::format_shortcut_hint("arrowup");
    assert_eq!(hint, "↑");
}

#[test]
fn format_shortcut_hint_arrowdown_variant() {
    let hint = ActionsDialog::format_shortcut_hint("arrowdown");
    assert_eq!(hint, "↓");
}

// =========================================================================
// 14. to_deeplink_name with CJK, emoji, RTL characters
// =========================================================================

#[test]
fn deeplink_name_ascii_basic() {
    assert_eq!(to_deeplink_name("Hello World"), "hello-world");
}

#[test]
fn deeplink_name_underscores_become_hyphens() {
    assert_eq!(to_deeplink_name("hello_world_test"), "hello-world-test");
}

#[test]
fn deeplink_name_special_chars_stripped() {
    assert_eq!(to_deeplink_name("test!@#$%^&*()"), "test");
}

#[test]
fn deeplink_name_multiple_spaces_collapsed() {
    assert_eq!(to_deeplink_name("foo   bar   baz"), "foo-bar-baz");
}

#[test]
fn deeplink_name_leading_trailing_stripped() {
    assert_eq!(to_deeplink_name("  hello  "), "hello");
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
fn deeplink_name_numbers_preserved() {
    assert_eq!(to_deeplink_name("Script 123"), "script-123");
}

#[test]
fn deeplink_name_mixed_case_lowered() {
    assert_eq!(to_deeplink_name("MyScript"), "myscript");
}

#[test]
fn deeplink_name_accented_chars() {
    // Accented characters are alphanumeric and should be preserved
    assert_eq!(to_deeplink_name("café résumé"), "café-résumé");
}

#[test]
fn deeplink_name_consecutive_hyphens_collapsed() {
    assert_eq!(to_deeplink_name("a--b"), "a-b");
}

// =========================================================================
// 15. Grouped items with realistic AI command bar data
// =========================================================================

#[test]
fn grouped_items_headers_style_produces_section_headers() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    let header_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
        .count();
    // Should have section headers for each section transition
    assert_eq!(
        header_count, 7,
        "AI command bar should have 7 section headers, got {}",
        header_count
    );
}

#[test]
fn grouped_items_none_style_produces_no_headers() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
    let header_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(header_count, 0, "None style should have 0 headers");
}

#[test]
fn grouped_items_separators_style_produces_no_headers() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    let header_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(header_count, 0, "Separators style should have 0 headers");
}

#[test]
fn grouped_items_empty_filtered_produces_empty() {
    let actions = get_ai_command_bar_actions();
    let grouped = build_grouped_items_static(&actions, &[], SectionStyle::Headers);
    assert!(grouped.is_empty());
}

#[test]
fn grouped_items_item_count_matches_filtered_count() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
    let item_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::Item(_)))
        .count();
    assert_eq!(item_count, filtered.len());
}

// =========================================================================
// 16. coerce_action_selection edge cases
// =========================================================================

#[test]
fn coerce_selection_empty_rows_returns_none() {
    let rows: Vec<GroupedActionItem> = vec![];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn coerce_selection_all_headers_returns_none() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".to_string()),
        GroupedActionItem::SectionHeader("B".to_string()),
        GroupedActionItem::SectionHeader("C".to_string()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn coerce_selection_on_item_returns_same_index() {
    let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    assert_eq!(coerce_action_selection(&rows, 1), Some(1));
}

#[test]
fn coerce_selection_on_header_skips_to_next_item() {
    let rows = vec![
        GroupedActionItem::SectionHeader("Response".to_string()),
        GroupedActionItem::Item(0),
        GroupedActionItem::Item(1),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn coerce_selection_on_last_header_searches_backward() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("End".to_string()),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn coerce_selection_out_of_bounds_clamps() {
    let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
    // Index 99 should be clamped to last valid index
    assert_eq!(coerce_action_selection(&rows, 99), Some(1));
}

// =========================================================================
// 17. Note switcher section assignment (Pinned vs Recent)
// =========================================================================

#[test]
fn note_switcher_pinned_note_has_pinned_section() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid-1".to_string(),
        title: "Pinned Note".to_string(),
        char_count: 100,
        is_current: false,
        is_pinned: true,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
}

#[test]
fn note_switcher_unpinned_note_has_recent_section() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid-2".to_string(),
        title: "Regular Note".to_string(),
        char_count: 50,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Recent"));
}

#[test]
fn note_switcher_mixed_pinned_and_recent() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "uuid-1".to_string(),
            title: "Pinned".to_string(),
            char_count: 10,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "uuid-2".to_string(),
            title: "Recent".to_string(),
            char_count: 20,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
    assert_eq!(actions[1].section.as_deref(), Some("Recent"));
}

#[test]
fn note_switcher_current_note_has_bullet_prefix() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid-1".to_string(),
        title: "My Note".to_string(),
        char_count: 42,
        is_current: true,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(
        actions[0].title.starts_with("• "),
        "Current note should have bullet prefix, got: {}",
        actions[0].title
    );
}

#[test]
fn note_switcher_non_current_note_no_bullet_prefix() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid-1".to_string(),
        title: "Other Note".to_string(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(!actions[0].title.starts_with("• "));
}

#[test]
fn note_switcher_icon_hierarchy_pinned_beats_current() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "uuid-1".to_string(),
            title: "Pinned+Current".to_string(),
            char_count: 10,
            is_current: true,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "uuid-2".to_string(),
            title: "Current Only".to_string(),
            char_count: 10,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "uuid-3".to_string(),
            title: "Pinned Only".to_string(),
            char_count: 10,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "uuid-4".to_string(),
            title: "Neither".to_string(),
            char_count: 10,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled)); // pinned+current → Star
    assert_eq!(actions[1].icon, Some(IconName::Check)); // current only → Check
    assert_eq!(actions[2].icon, Some(IconName::StarFilled)); // pinned only → Star
    assert_eq!(actions[3].icon, Some(IconName::File)); // neither → File
}

#[test]
fn note_switcher_char_count_singular() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid-1".to_string(),
        title: "Single Char Note".to_string(),
        char_count: 1,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("1 char"));
}

#[test]
fn note_switcher_char_count_plural() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid-1".to_string(),
        title: "Multi Char Note".to_string(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("42 chars"));
}

#[test]
fn note_switcher_char_count_zero() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid-1".to_string(),
        title: "Empty Note".to_string(),
        char_count: 0,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("0 chars"));
}

#[test]
fn note_switcher_empty_notes_shows_helpful_message() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "no_notes");
    assert_eq!(actions[0].title, "No notes yet");
    assert_eq!(actions[0].icon, Some(IconName::Plus));
}

#[test]
fn note_switcher_action_id_format() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc-123-def".to_string(),
        title: "Test".to_string(),
        char_count: 5,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].id, "note_abc-123-def");
}

// =========================================================================
// 18. Clipboard frontmost app edge cases
// =========================================================================

#[test]
fn clipboard_paste_title_with_empty_string_app_name() {
    let entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: Some("".to_string()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = find_action(&actions, "clipboard_paste").unwrap();
    // Even empty string gets formatted with "Paste to "
    assert_eq!(paste.title, "Paste to ");
}
