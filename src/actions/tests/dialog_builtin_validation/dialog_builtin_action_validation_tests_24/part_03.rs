
#[test]
fn batch24_new_chat_preset_no_description() {
    let presets = vec![NewChatPresetInfo {
        id: "g".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert!(actions[0].description.is_none());
}

// ============================================================
// 18. Path context: exact action IDs in order for directory
// ============================================================

#[test]
fn batch24_path_dir_action_ids_ordered() {
    let p = PathInfo::new("Documents", "/Users/test/Documents", true);
    let actions = get_path_context_actions(&p);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "open_directory",
            "copy_path",
            "open_in_finder",
            "open_in_editor",
            "open_in_terminal",
            "copy_filename",
            "move_to_trash",
        ]
    );
}

#[test]
fn batch24_path_file_action_ids_ordered() {
    let p = PathInfo::new("file.txt", "/Users/test/file.txt", false);
    let actions = get_path_context_actions(&p);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "select_file",
            "copy_path",
            "open_in_finder",
            "open_in_editor",
            "open_in_terminal",
            "copy_filename",
            "move_to_trash",
        ]
    );
}

#[test]
fn batch24_path_always_7_actions() {
    let dir = PathInfo::new("d", "/d", true);
    let file = PathInfo::new("f", "/f", false);
    assert_eq!(get_path_context_actions(&dir).len(), 7);
    assert_eq!(get_path_context_actions(&file).len(), 7);
}

// ============================================================
// 19. Path context: shortcut assignments
// ============================================================

#[test]
fn batch24_path_copy_path_shortcut() {
    let p = PathInfo::new("f", "/f", false);
    let actions = get_path_context_actions(&p);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert_eq!(cp.shortcut.as_ref().unwrap(), "⌘⇧C");
}

#[test]
fn batch24_path_open_in_finder_shortcut() {
    let p = PathInfo::new("f", "/f", false);
    let actions = get_path_context_actions(&p);
    let f = actions.iter().find(|a| a.id == "open_in_finder").unwrap();
    assert_eq!(f.shortcut.as_ref().unwrap(), "⌘⇧F");
}

#[test]
fn batch24_path_open_in_terminal_shortcut() {
    let p = PathInfo::new("f", "/f", false);
    let actions = get_path_context_actions(&p);
    let t = actions.iter().find(|a| a.id == "open_in_terminal").unwrap();
    assert_eq!(t.shortcut.as_ref().unwrap(), "⌘T");
}

#[test]
fn batch24_path_copy_filename_no_shortcut() {
    let p = PathInfo::new("f", "/f", false);
    let actions = get_path_context_actions(&p);
    let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert!(cf.shortcut.is_none());
}

#[test]
fn batch24_path_move_to_trash_shortcut() {
    let p = PathInfo::new("f", "/f", false);
    let actions = get_path_context_actions(&p);
    let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert_eq!(trash.shortcut.as_ref().unwrap(), "⌘⌫");
}

// ============================================================
// 20. File context: macOS action count difference
// ============================================================

#[cfg(target_os = "macos")]
#[test]
fn batch24_file_context_macos_file_count() {
    let f = FileInfo {
        path: "/test/f.txt".to_string(),
        name: "f.txt".to_string(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&f);
    // open_file, reveal, quick_look, open_with, show_info, copy_path, copy_filename = 7
    assert_eq!(actions.len(), 7);
}

#[cfg(target_os = "macos")]
#[test]
fn batch24_file_context_macos_dir_count() {
    let f = FileInfo {
        path: "/test/d".to_string(),
        name: "d".to_string(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&f);
    // open_directory, reveal, open_with, show_info, copy_path, copy_filename = 6
    // (no quick_look for dirs)
    assert_eq!(actions.len(), 6);
}

// ============================================================
// 21. to_deeplink_name: additional edge cases
// ============================================================

#[test]
fn batch24_deeplink_numeric_only() {
    assert_eq!(to_deeplink_name("123"), "123");
}

#[test]
fn batch24_deeplink_single_char() {
    assert_eq!(to_deeplink_name("a"), "a");
}

#[test]
fn batch24_deeplink_all_special_empty() {
    assert_eq!(to_deeplink_name("!@#$%"), "");
}

#[test]
fn batch24_deeplink_mixed_unicode() {
    let result = to_deeplink_name("Café Script");
    assert!(result.contains("caf"));
    assert!(result.contains("script"));
}

#[test]
fn batch24_deeplink_underscores_to_hyphens() {
    assert_eq!(to_deeplink_name("hello_world"), "hello-world");
}

// ============================================================
// 22. format_shortcut_hint (dialog.rs version): alias coverage
// ============================================================

#[test]
fn batch24_format_hint_command_alias() {
    assert_eq!(ActionsDialog::format_shortcut_hint("command+c"), "⌘C");
}

#[test]
fn batch24_format_hint_meta_alias() {
    assert_eq!(ActionsDialog::format_shortcut_hint("meta+c"), "⌘C");
}

#[test]
fn batch24_format_hint_super_alias() {
    assert_eq!(ActionsDialog::format_shortcut_hint("super+c"), "⌘C");
}

#[test]
fn batch24_format_hint_control_alias() {
    assert_eq!(ActionsDialog::format_shortcut_hint("control+c"), "⌃C");
}

#[test]
fn batch24_format_hint_opt_alias() {
    assert_eq!(ActionsDialog::format_shortcut_hint("opt+c"), "⌥C");
}

#[test]
fn batch24_format_hint_option_alias() {
    assert_eq!(ActionsDialog::format_shortcut_hint("option+c"), "⌥C");
}

#[test]
fn batch24_format_hint_return_alias() {
    assert_eq!(ActionsDialog::format_shortcut_hint("cmd+return"), "⌘↵");
}

#[test]
fn batch24_format_hint_esc_alias() {
    assert_eq!(ActionsDialog::format_shortcut_hint("esc"), "⎋");
}

// ============================================================
// 23. parse_shortcut_keycaps: modifiers and special keys
// ============================================================

#[test]
fn batch24_keycaps_single_modifier() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘");
    assert_eq!(caps, vec!["⌘"]);
}

#[test]
fn batch24_keycaps_modifier_and_letter() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘C");
    assert_eq!(caps, vec!["⌘", "C"]);
}

#[test]
fn batch24_keycaps_all_modifiers() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌃⌥⇧⌘C");
    assert_eq!(caps, vec!["⌃", "⌥", "⇧", "⌘", "C"]);
}

#[test]
fn batch24_keycaps_arrows() {
    let caps = ActionsDialog::parse_shortcut_keycaps("↑↓←→");
    assert_eq!(caps, vec!["↑", "↓", "←", "→"]);
}

#[test]
fn batch24_keycaps_lowercase_uppercased() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘e");
    assert_eq!(caps, vec!["⌘", "E"]);
}

// ============================================================
// 24. score_action: scoring tiers with cached lowercase
// ============================================================

#[test]
fn batch24_score_prefix_match() {
    let action = Action::new(
        "id",
        "Edit Script",
        Some("Open editor".to_string()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "edit");
    assert!(score >= 100);
}

#[test]
fn batch24_score_contains_match() {
    let action = Action::new("id", "Copy Edit Path", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "edit");
    assert!(score >= 50);
    assert!(score < 100);
}

#[test]
fn batch24_score_no_match() {
    let action = Action::new("id", "Copy Path", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "xyz");
    assert_eq!(score, 0);
}

#[test]
fn batch24_score_description_bonus() {
    let action = Action::new(
        "id",
        "Open File",
        Some("Edit in editor".to_string()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "editor");
    assert!(score >= 15);
}

#[test]
fn batch24_score_shortcut_bonus() {
    let action =
        Action::new("id", "Open File", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
    let score = ActionsDialog::score_action(&action, "⌘e");
    assert!(score >= 10);
}

// ============================================================
// 25. fuzzy_match: edge cases
// ============================================================

#[test]
fn batch24_fuzzy_exact() {
    assert!(ActionsDialog::fuzzy_match("edit", "edit"));
}

#[test]
fn batch24_fuzzy_subsequence() {
    assert!(ActionsDialog::fuzzy_match("edit script", "esc"));
}

#[test]
fn batch24_fuzzy_no_match() {
    assert!(!ActionsDialog::fuzzy_match("abc", "xyz"));
}

#[test]
fn batch24_fuzzy_empty_needle() {
    assert!(ActionsDialog::fuzzy_match("abc", ""));
}

#[test]
fn batch24_fuzzy_needle_longer() {
    assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
}

// ============================================================
// 26. build_grouped_items_static: section style effects
// ============================================================

#[test]
fn batch24_grouped_headers_adds_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // S1 header, A item, S2 header, B item = 4
    assert_eq!(grouped.len(), 4);
}

#[test]
fn batch24_grouped_same_section_one_header() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S1"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // S1 header, A, B = 3
    assert_eq!(grouped.len(), 3);
}

#[test]
fn batch24_grouped_separators_no_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    // Just items, no headers
    assert_eq!(grouped.len(), 2);
}

#[test]
fn batch24_grouped_empty_filtered() {
    let actions = vec![Action::new("a", "A", None, ActionCategory::ScriptContext)];
    let filtered: Vec<usize> = vec![];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    assert!(grouped.is_empty());
}

// ============================================================
// 27. coerce_action_selection: header skipping
// ============================================================

#[test]
fn batch24_coerce_on_item_stays() {
    let rows = vec![
        GroupedActionItem::SectionHeader("S".to_string()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(1));
}

#[test]
fn batch24_coerce_header_skips_down() {
    let rows = vec![
        GroupedActionItem::SectionHeader("S".to_string()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn batch24_coerce_trailing_header_searches_up() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("S".to_string()),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn batch24_coerce_all_headers_none() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".to_string()),
        GroupedActionItem::SectionHeader("B".to_string()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn batch24_coerce_empty_none() {
    let rows: Vec<GroupedActionItem> = vec![];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

// ============================================================
// 28. CommandBarConfig preset field values
// ============================================================

#[test]
fn batch24_cmdbar_default_close_flags() {
    let config = CommandBarConfig::default();
    assert!(config.close_on_select);
    assert!(config.close_on_escape);
    assert!(config.close_on_click_outside);
}

#[test]
fn batch24_cmdbar_ai_style_search_top() {
    let config = CommandBarConfig::ai_style();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
    assert!(config.dialog_config.show_icons);
    assert!(config.dialog_config.show_footer);
}

#[test]
fn batch24_cmdbar_main_menu_search_bottom() {
    let config = CommandBarConfig::main_menu_style();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Bottom);
    assert!(!config.dialog_config.show_icons);
    assert!(!config.dialog_config.show_footer);
}
