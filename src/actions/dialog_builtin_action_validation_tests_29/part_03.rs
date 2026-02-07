
#[test]
fn cat29_20_score_prefix_plus_shortcut_bonus() {
    let action = Action::new(
        "edit_script",
        "Edit Script",
        None,
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘E");
    // search for "e" — prefix match on title "edit script" (100) + shortcut "⌘e" contains "e" (10)
    let score = super::dialog::ActionsDialog::score_action(&action, "e");
    assert!(score >= 110);
}

#[test]
fn cat29_20_score_all_three_bonuses() {
    let action = Action::new(
        "edit_script",
        "Edit Script",
        Some("Edit the file".to_string()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘E");
    // "edit" → prefix(100) + desc contains "edit"(15) + shortcut doesn't contain "edit"
    let score = super::dialog::ActionsDialog::score_action(&action, "edit");
    assert!(score >= 115);
}

// =============================================================================
// Category 21: fuzzy_match — case sensitivity and edge cases
// =============================================================================

#[test]
fn cat29_21_fuzzy_match_exact() {
    assert!(super::dialog::ActionsDialog::fuzzy_match("hello", "hello"));
}

#[test]
fn cat29_21_fuzzy_match_subsequence() {
    assert!(super::dialog::ActionsDialog::fuzzy_match(
        "hello world",
        "hwd"
    ));
}

#[test]
fn cat29_21_fuzzy_match_no_match() {
    assert!(!super::dialog::ActionsDialog::fuzzy_match("hello", "xyz"));
}

#[test]
fn cat29_21_fuzzy_match_empty_needle_matches() {
    assert!(super::dialog::ActionsDialog::fuzzy_match("anything", ""));
}

#[test]
fn cat29_21_fuzzy_match_needle_longer_fails() {
    assert!(!super::dialog::ActionsDialog::fuzzy_match("hi", "hello"));
}

// =============================================================================
// Category 22: build_grouped_items_static — section headers with Headers style
// =============================================================================

#[test]
fn cat29_22_grouped_items_headers_adds_section_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Group1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Group2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // Should have: Header("Group1"), Item(0), Header("Group2"), Item(1)
    assert_eq!(grouped.len(), 4);
    assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(_)));
    assert!(matches!(grouped[1], GroupedActionItem::Item(0)));
    assert!(matches!(grouped[2], GroupedActionItem::SectionHeader(_)));
    assert!(matches!(grouped[3], GroupedActionItem::Item(1)));
}

#[test]
fn cat29_22_grouped_items_separators_no_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Group1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Group2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    // Should have just items, no headers
    assert_eq!(grouped.len(), 2);
    assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
    assert!(matches!(grouped[1], GroupedActionItem::Item(1)));
}

#[test]
fn cat29_22_grouped_items_same_section_one_header() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Same"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Same"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // Should have: Header("Same"), Item(0), Item(1) — only one header
    assert_eq!(grouped.len(), 3);
    assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(_)));
}

#[test]
fn cat29_22_grouped_items_empty_returns_empty() {
    let actions: Vec<Action> = vec![];
    let filtered: Vec<usize> = vec![];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    assert!(grouped.is_empty());
}

// =============================================================================
// Category 23: coerce_action_selection — header skipping behavior
// =============================================================================

#[test]
fn cat29_23_coerce_on_item_stays() {
    let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
}

#[test]
fn cat29_23_coerce_on_header_jumps_down() {
    let rows = vec![
        GroupedActionItem::SectionHeader("H".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn cat29_23_coerce_trailing_header_jumps_up() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("H".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn cat29_23_coerce_all_headers_returns_none() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".into()),
        GroupedActionItem::SectionHeader("B".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn cat29_23_coerce_empty_returns_none() {
    let rows: Vec<GroupedActionItem> = vec![];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

// =============================================================================
// Category 24: CommandBarConfig — all presets preserve close defaults
// =============================================================================

#[test]
fn cat29_24_ai_style_close_on_select() {
    let config = CommandBarConfig::ai_style();
    assert!(config.close_on_select);
}

#[test]
fn cat29_24_main_menu_close_on_escape() {
    let config = CommandBarConfig::main_menu_style();
    assert!(config.close_on_escape);
}

#[test]
fn cat29_24_no_search_close_on_click_outside() {
    let config = CommandBarConfig::no_search();
    assert!(config.close_on_click_outside);
}

#[test]
fn cat29_24_notes_style_close_defaults() {
    let config = CommandBarConfig::notes_style();
    assert!(config.close_on_select);
    assert!(config.close_on_escape);
    assert!(config.close_on_click_outside);
}

// =============================================================================
// Category 25: CommandBarConfig — show_icons and show_footer combinations
// =============================================================================

#[test]
fn cat29_25_ai_style_has_icons_and_footer() {
    let config = CommandBarConfig::ai_style();
    assert!(config.dialog_config.show_icons);
    assert!(config.dialog_config.show_footer);
}

#[test]
fn cat29_25_main_menu_no_icons_no_footer() {
    let config = CommandBarConfig::main_menu_style();
    assert!(!config.dialog_config.show_icons);
    assert!(!config.dialog_config.show_footer);
}

#[test]
fn cat29_25_notes_style_has_icons_and_footer() {
    let config = CommandBarConfig::notes_style();
    assert!(config.dialog_config.show_icons);
    assert!(config.dialog_config.show_footer);
}

#[test]
fn cat29_25_no_search_no_icons_no_footer() {
    let config = CommandBarConfig::no_search();
    assert!(!config.dialog_config.show_icons);
    assert!(!config.dialog_config.show_footer);
}

// =============================================================================
// Category 26: New chat — empty inputs produce empty actions
// =============================================================================

#[test]
fn cat29_26_new_chat_all_empty() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

#[test]
fn cat29_26_new_chat_only_last_used() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "P".into(),
        provider_display_name: "Provider".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
}

#[test]
fn cat29_26_new_chat_only_presets() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Presets"));
}

#[test]
fn cat29_26_new_chat_only_models() {
    let models = vec![NewChatModelInfo {
        model_id: "m2".into(),
        display_name: "Model 2".into(),
        provider: "P".into(),
        provider_display_name: "Provider".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Models"));
}

// =============================================================================
// Category 27: Scriptlet context with_custom — reset_ranking is always last
// =============================================================================

#[test]
fn cat29_27_scriptlet_frecency_reset_ranking_last() {
    let script =
        ScriptInfo::scriptlet("S", "/p.md", None, None).with_frecency(true, Some("s".into()));
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let last = actions.last().unwrap();
    assert_eq!(last.id, "reset_ranking");
}

#[test]
fn cat29_27_script_frecency_reset_ranking_last() {
    let script = ScriptInfo::new("S", "/p.ts").with_frecency(true, Some("s".into()));
    let actions = get_script_context_actions(&script);
    let last = actions.last().unwrap();
    assert_eq!(last.id, "reset_ranking");
}

#[test]
fn cat29_27_builtin_frecency_reset_ranking_last() {
    let script = ScriptInfo::builtin("CH").with_frecency(true, Some("ch".into()));
    let actions = get_script_context_actions(&script);
    let last = actions.last().unwrap();
    assert_eq!(last.id, "reset_ranking");
}

// =============================================================================
// Category 28: Cross-context — all built-in actions have ActionCategory::ScriptContext
// =============================================================================

#[test]
fn cat29_28_script_all_script_context() {
    let script = ScriptInfo::new("S", "/p.ts");
    let actions = get_script_context_actions(&script);
    for a in &actions {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "action {} has wrong category",
            a.id
        );
    }
}

#[test]
fn cat29_28_clipboard_all_script_context() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    for a in &actions {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "action {} has wrong category",
            a.id
        );
    }
}

#[test]
fn cat29_28_ai_all_script_context() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "action {} has wrong category",
            a.id
        );
    }
}

#[test]
fn cat29_28_path_all_script_context() {
    let pi = PathInfo {
        path: "/p".into(),
        name: "p".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    for a in &actions {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "action {} has wrong category",
            a.id
        );
    }
}

// =============================================================================
// Category 29: Cross-context — first action ID is always the primary action
// =============================================================================

#[test]
fn cat29_29_script_first_is_run_script() {
    let script = ScriptInfo::new("S", "/p.ts");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn cat29_29_clipboard_first_is_paste() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[0].id, "clipboard_paste");
}

#[test]
fn cat29_29_path_file_first_is_select_file() {
    let pi = PathInfo {
        path: "/p/f.txt".into(),
        name: "f.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    assert_eq!(actions[0].id, "select_file");
}

#[test]
fn cat29_29_path_dir_first_is_open_directory() {
    let pi = PathInfo {
        path: "/p/dir".into(),
        name: "dir".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    assert_eq!(actions[0].id, "open_directory");
}

#[test]
fn cat29_29_file_first_is_open() {
    let fi = FileInfo {
        path: "/p/f.txt".into(),
        name: "f.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    assert_eq!(actions[0].id, "open_file");
}

// =============================================================================
// Category 30: Action builder — chaining preserves all fields correctly
// =============================================================================

#[test]
fn cat29_30_action_new_defaults() {
    let a = Action::new(
        "id",
        "Title",
        Some("Desc".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(a.id, "id");
    assert_eq!(a.title, "Title");
    assert_eq!(a.description.as_deref(), Some("Desc"));
    assert!(!a.has_action);
    assert!(a.shortcut.is_none());
    assert!(a.icon.is_none());
    assert!(a.section.is_none());
    assert!(a.value.is_none());
}
