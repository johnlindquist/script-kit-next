
#[test]
fn test_score_action_combined_bonuses() {
    // Action where title prefix AND description match
    let action = Action::new(
        "copy_path",
        "Copy Path",
        Some("Copy the full path to clipboard".to_string()),
        ActionCategory::ScriptContext,
    );

    let score = ActionsDialog::score_action(&action, "copy");
    // Prefix match (100) + description contains "copy" (15) = 115
    assert_eq!(score, 115, "Combined prefix + desc should be 115");
}

#[test]
fn test_score_action_fuzzy_match() {
    let action = Action::new(
        "reveal_in_finder",
        "Reveal in Finder",
        None,
        ActionCategory::ScriptContext,
    );

    // "rvf" is a fuzzy match for "reveal in finder" (r-e-v-e-a-l-i-n-f-i-n-d-e-r)
    let fuzzy = ActionsDialog::fuzzy_match(&action.title_lower, "rvf");
    assert!(fuzzy, "rvf should fuzzy-match 'reveal in finder'");

    let score = ActionsDialog::score_action(&action, "rvf");
    assert_eq!(score, 25, "Fuzzy match should score 25");
}

#[test]
fn test_score_action_no_match() {
    let action = Action::new(
        "edit_script",
        "Edit Script",
        None,
        ActionCategory::ScriptContext,
    );

    let score = ActionsDialog::score_action(&action, "zzz");
    assert_eq!(score, 0, "No match should score 0");
}

#[test]
fn test_fuzzy_match_empty_needle() {
    assert!(
        ActionsDialog::fuzzy_match("anything", ""),
        "Empty needle matches everything"
    );
}

#[test]
fn test_fuzzy_match_needle_longer_than_haystack() {
    assert!(
        !ActionsDialog::fuzzy_match("ab", "abc"),
        "Needle longer than haystack should not match"
    );
}

#[test]
fn test_fuzzy_match_exact() {
    assert!(ActionsDialog::fuzzy_match("hello", "hello"));
}

// =========================================================================
// Constants consistency validation
// =========================================================================

#[test]
fn test_constants_positive_and_reasonable() {
    // Use a runtime identity function to prevent clippy constant-value lint
    fn val(x: f32) -> f32 {
        x
    }
    assert!(val(POPUP_WIDTH) > 0.0 && val(POPUP_WIDTH) < 1000.0);
    assert!(val(POPUP_MAX_HEIGHT) > 0.0 && val(POPUP_MAX_HEIGHT) < 2000.0);
    assert!(val(ACTION_ITEM_HEIGHT) > 0.0 && val(ACTION_ITEM_HEIGHT) < 100.0);
    assert!(val(SEARCH_INPUT_HEIGHT) > 0.0 && val(SEARCH_INPUT_HEIGHT) < 100.0);
    assert!(val(HEADER_HEIGHT) > 0.0 && val(HEADER_HEIGHT) < 100.0);
    assert!(val(SECTION_HEADER_HEIGHT) > 0.0 && val(SECTION_HEADER_HEIGHT) < 100.0);
    assert!(val(ACTION_ROW_INSET) >= 0.0 && val(ACTION_ROW_INSET) < 50.0);
    assert!(val(SELECTION_RADIUS) >= 0.0 && val(SELECTION_RADIUS) < 50.0);
    assert!(val(KEYCAP_MIN_WIDTH) > 0.0 && val(KEYCAP_MIN_WIDTH) < 100.0);
    assert!(val(KEYCAP_HEIGHT) > 0.0 && val(KEYCAP_HEIGHT) < 100.0);
}

#[test]
fn test_popup_can_fit_at_least_5_items() {
    let max_items_height = POPUP_MAX_HEIGHT - SEARCH_INPUT_HEIGHT;
    let max_items = (max_items_height / ACTION_ITEM_HEIGHT) as usize;
    assert!(
        max_items >= 5,
        "Popup should fit at least 5 items, fits {}",
        max_items
    );
}

#[test]
fn test_section_header_shorter_than_action_item() {
    fn val(x: f32) -> f32 {
        x
    }
    assert!(
        val(SECTION_HEADER_HEIGHT) < val(ACTION_ITEM_HEIGHT),
        "Section headers should be shorter than action items"
    );
}

// =========================================================================
// CommandBarConfig presets field validation
// =========================================================================

#[test]
fn test_command_bar_config_default_values() {
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
fn test_command_bar_config_ai_style_values() {
    let config = CommandBarConfig::ai_style();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
    assert_eq!(config.dialog_config.section_style, SectionStyle::Headers);
    assert_eq!(config.dialog_config.anchor, AnchorPosition::Top);
    assert!(config.dialog_config.show_icons);
    assert!(config.dialog_config.show_footer);
}

#[test]
fn test_command_bar_config_main_menu_style() {
    let config = CommandBarConfig::main_menu_style();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Bottom);
    assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
    assert_eq!(config.dialog_config.anchor, AnchorPosition::Bottom);
    assert!(!config.dialog_config.show_icons);
    assert!(!config.dialog_config.show_footer);
}

#[test]
fn test_command_bar_config_no_search() {
    let config = CommandBarConfig::no_search();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Hidden);
}

// =========================================================================
// Agent-specific action validation
// =========================================================================

#[test]
fn test_agent_actions_edit_title_is_edit_agent() {
    let mut script = ScriptInfo::new("My Agent", "/path/to/agent.claude.md");
    script.is_agent = true;
    script.is_script = false; // Agents set is_script=false

    let actions = get_script_context_actions(&script);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.title, "Edit Agent");
}

#[test]
fn test_agent_has_no_view_logs() {
    let mut script = ScriptInfo::new("My Agent", "/path/to/agent.claude.md");
    script.is_agent = true;
    script.is_script = false;

    let actions = get_script_context_actions(&script);
    assert!(
        !actions.iter().any(|a| a.id == "view_logs"),
        "Agents should not have view_logs"
    );
}

#[test]
fn test_agent_has_reveal_and_copy() {
    let mut script = ScriptInfo::new("My Agent", "/path/to/agent.claude.md");
    script.is_agent = true;
    script.is_script = false;

    let actions = get_script_context_actions(&script);
    assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
    assert!(actions.iter().any(|a| a.id == "copy_path"));
    assert!(actions.iter().any(|a| a.id == "copy_content"));
}

#[test]
fn test_agent_with_shortcut_and_alias() {
    let mut script = ScriptInfo::with_shortcut_and_alias(
        "My Agent",
        "/path/to/agent.claude.md",
        Some("cmd+shift+a".to_string()),
        Some("ag".to_string()),
    );
    script.is_agent = true;
    script.is_script = false;

    let actions = get_script_context_actions(&script);

    // Should have update/remove pairs
    assert!(actions.iter().any(|a| a.id == "update_shortcut"));
    assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
    assert!(actions.iter().any(|a| a.id == "update_alias"));
    assert!(actions.iter().any(|a| a.id == "remove_alias"));
    // Should NOT have add
    assert!(!actions.iter().any(|a| a.id == "add_shortcut"));
    assert!(!actions.iter().any(|a| a.id == "add_alias"));
}

#[test]
fn test_agent_with_frecency_has_reset_ranking() {
    let mut script = ScriptInfo::new("My Agent", "/path/to/agent.claude.md")
        .with_frecency(true, Some("agent:/path/to/agent.claude.md".to_string()));
    script.is_agent = true;
    script.is_script = false;

    let actions = get_script_context_actions(&script);
    assert!(actions.iter().any(|a| a.id == "reset_ranking"));
}

// =========================================================================
// Clipboard destructive action ordering invariants
// =========================================================================

#[test]
fn test_clipboard_destructive_actions_always_last() {
    let entry = ClipboardEntryInfo {
        id: "test".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "Test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };

    let actions = get_clipboard_history_context_actions(&entry);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    let delete_idx = ids.iter().position(|&id| id == "clipboard_delete").unwrap();
    let delete_multi_idx = ids
        .iter()
        .position(|&id| id == "clipboard_delete_multiple")
        .unwrap();
    let delete_all_idx = ids
        .iter()
        .position(|&id| id == "clipboard_delete_all")
        .unwrap();

    // All destructive actions should be at the end
    let non_destructive_count = actions.len() - 3;
    assert!(
        delete_idx >= non_destructive_count,
        "clipboard_delete should be in last 3"
    );
    assert!(
        delete_multi_idx >= non_destructive_count,
        "clipboard_delete_multiple should be in last 3"
    );
    assert!(
        delete_all_idx >= non_destructive_count,
        "clipboard_delete_all should be in last 3"
    );
}

#[test]
fn test_clipboard_paste_always_first() {
    let entry = ClipboardEntryInfo {
        id: "test".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "Test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };

    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[0].id, "clipboard_paste");
}

#[test]
fn test_clipboard_copy_always_second() {
    let entry = ClipboardEntryInfo {
        id: "test".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "Test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };

    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[1].id, "clipboard_copy");
}

// =========================================================================
// Path context action validation
// =========================================================================

#[test]
fn test_path_context_directory_primary_action() {
    let path = PathInfo {
        path: "/Users/test/Documents".to_string(),
        name: "Documents".to_string(),
        is_dir: true,
    };

    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].id, "open_directory");
    assert!(actions[0].title.contains("Documents"));
}

#[test]
fn test_path_context_file_primary_action() {
    let path = PathInfo {
        path: "/Users/test/readme.md".to_string(),
        name: "readme.md".to_string(),
        is_dir: false,
    };

    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].id, "select_file");
    assert!(actions[0].title.contains("readme.md"));
}

#[test]
fn test_path_context_trash_description_varies() {
    let dir_path = PathInfo {
        path: "/tmp/dir".to_string(),
        name: "dir".to_string(),
        is_dir: true,
    };
    let file_path = PathInfo {
        path: "/tmp/file.txt".to_string(),
        name: "file.txt".to_string(),
        is_dir: false,
    };

    let dir_actions = get_path_context_actions(&dir_path);
    let file_actions = get_path_context_actions(&file_path);

    let dir_trash = dir_actions
        .iter()
        .find(|a| a.id == "move_to_trash")
        .unwrap();
    let file_trash = file_actions
        .iter()
        .find(|a| a.id == "move_to_trash")
        .unwrap();

    assert!(
        dir_trash.description.as_ref().unwrap().contains("folder"),
        "Directory trash should say 'folder'"
    );
    assert!(
        file_trash.description.as_ref().unwrap().contains("file"),
        "File trash should say 'file'"
    );
}

#[test]
fn test_path_context_common_actions_present() {
    let path = PathInfo {
        path: "/tmp/test".to_string(),
        name: "test".to_string(),
        is_dir: false,
    };

    let actions = get_path_context_actions(&path);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    assert!(ids.contains(&"copy_path"));
    assert!(ids.contains(&"open_in_finder"));
    assert!(ids.contains(&"open_in_editor"));
    assert!(ids.contains(&"open_in_terminal"));
    assert!(ids.contains(&"copy_filename"));
    assert!(ids.contains(&"move_to_trash"));
}

// =========================================================================
// build_grouped_items_static edge cases
// =========================================================================

#[test]
fn test_build_grouped_items_empty_actions() {
    let actions: Vec<Action> = vec![];
    let filtered: Vec<usize> = vec![];
    let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    assert!(items.is_empty());
}

#[test]
fn test_build_grouped_items_headers_style_adds_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Group1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Group1"),
        Action::new("c", "C", None, ActionCategory::ScriptContext).with_section("Group2"),
    ];
    let filtered = vec![0, 1, 2];
    let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);

    // Should have: Header("Group1"), Item(0), Item(1), Header("Group2"), Item(2)
    assert_eq!(items.len(), 5);
    assert!(matches!(&items[0], GroupedActionItem::SectionHeader(s) if s == "Group1"));
    assert!(matches!(&items[1], GroupedActionItem::Item(0)));
    assert!(matches!(&items[2], GroupedActionItem::Item(1)));
    assert!(matches!(&items[3], GroupedActionItem::SectionHeader(s) if s == "Group2"));
    assert!(matches!(&items[4], GroupedActionItem::Item(2)));
}

#[test]
fn test_build_grouped_items_separators_style_no_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Group1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Group2"),
    ];
    let filtered = vec![0, 1];
    let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);

    // Separators style should NOT add headers
    assert_eq!(items.len(), 2);
    assert!(matches!(&items[0], GroupedActionItem::Item(0)));
    assert!(matches!(&items[1], GroupedActionItem::Item(1)));
}

#[test]
fn test_build_grouped_items_none_style_no_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Group1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Group2"),
    ];
    let filtered = vec![0, 1];
    let items = build_grouped_items_static(&actions, &filtered, SectionStyle::None);

    assert_eq!(items.len(), 2);
    assert!(matches!(&items[0], GroupedActionItem::Item(0)));
    assert!(matches!(&items[1], GroupedActionItem::Item(1)));
}
