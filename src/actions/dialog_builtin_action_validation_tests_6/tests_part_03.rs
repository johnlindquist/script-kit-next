    #[test]
    fn new_chat_model_has_provider_description() {
        let models = vec![NewChatModelInfo {
            model_id: "m".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].description.as_ref().unwrap(), "Anthropic");
    }

    // =========================================================================
    // 9. Score stacking (title+desc+shortcut all matching)
    // =========================================================================

    #[test]
    fn score_action_prefix_title_only() {
        let action = Action::new(
            "test",
            "Copy Path",
            Some("Copy the path".into()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "copy");
        assert!(score >= 100, "Prefix match: {}", score);
    }

    #[test]
    fn score_action_title_prefix_plus_description_match() {
        let action = Action::new(
            "test",
            "Copy Path",
            Some("Copy the path to clipboard".into()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "copy");
        // prefix(100) + description contains "copy"(15) = 115
        assert!(score >= 115, "Prefix + desc: {}", score);
    }

    #[test]
    fn score_action_title_prefix_plus_desc_plus_shortcut() {
        let action = Action::new(
            "test",
            "Copy Path",
            Some("Copy the path to clipboard".into()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘COPY");
        let score = ActionsDialog::score_action(&action, "copy");
        // prefix(100) + desc(15) + shortcut(10) = 125
        assert!(score >= 125, "Prefix + desc + shortcut: {}", score);
    }

    #[test]
    fn score_action_no_match_returns_zero() {
        let action = Action::new(
            "test",
            "Open File",
            Some("Open the file".into()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "xyz123");
        assert_eq!(score, 0);
    }

    #[test]
    fn score_action_contains_only_no_prefix() {
        let action = Action::new(
            "test",
            "Reset Copy Path",
            None,
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "copy");
        // Contains only: 50
        assert!(score >= 50, "Contains: {}", score);
        assert!(score < 100, "Should not be prefix: {}", score);
    }

    #[test]
    fn score_action_fuzzy_only() {
        let action = Action::new("test", "Extract Data", None, ActionCategory::ScriptContext);
        // "eda" matches E-x-t-r-a-c-t-D-A-t-a as subsequence e...d...a
        let score = ActionsDialog::score_action(&action, "eda");
        assert!(score >= 25, "Fuzzy match: {}", score);
        assert!(score < 50, "Should not be contains: {}", score);
    }

    #[test]
    fn score_action_description_only_match() {
        let action = Action::new(
            "test",
            "Open File",
            Some("Navigate to the editor".into()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "editor");
        // Description only: 15
        assert_eq!(score, 15, "Description-only match");
    }

    #[test]
    fn score_action_shortcut_only_match() {
        let action = Action::new(
            "test",
            "Open File",
            Some("Open the file".into()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘Z");
        let score = ActionsDialog::score_action(&action, "⌘z");
        // Shortcut contains: 10
        assert!(score >= 10, "Shortcut match: {}", score);
    }

    // =========================================================================
    // 10. build_grouped_items_static edge cases
    // =========================================================================

    #[test]
    fn grouped_items_same_section_no_duplicate_headers() {
        let actions = vec![
            Action::new("a1", "Action 1", None, ActionCategory::ScriptContext)
                .with_section("Response"),
            Action::new("a2", "Action 2", None, ActionCategory::ScriptContext)
                .with_section("Response"),
            Action::new("a3", "Action 3", None, ActionCategory::ScriptContext)
                .with_section("Response"),
        ];
        let filtered: Vec<usize> = (0..3).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Should be: 1 header + 3 items = 4
        assert_eq!(grouped.len(), 4);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 1, "Same section = 1 header");
    }

    #[test]
    fn grouped_items_alternating_sections_get_headers() {
        let actions = vec![
            Action::new("a1", "A1", None, ActionCategory::ScriptContext).with_section("Alpha"),
            Action::new("a2", "A2", None, ActionCategory::ScriptContext).with_section("Beta"),
            Action::new("a3", "A3", None, ActionCategory::ScriptContext).with_section("Alpha"),
        ];
        let filtered: Vec<usize> = (0..3).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Alpha header, A1, Beta header, A2, Alpha header again, A3 = 6
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 3, "Each section change = new header");
    }

    #[test]
    fn grouped_items_none_style_no_headers() {
        let actions = vec![
            Action::new("a1", "A1", None, ActionCategory::ScriptContext).with_section("Section"),
            Action::new("a2", "A2", None, ActionCategory::ScriptContext).with_section("Other"),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 0, "None style = no headers");
    }

    #[test]
    fn grouped_items_separators_style_no_headers() {
        let actions = vec![
            Action::new("a1", "A1", None, ActionCategory::ScriptContext).with_section("Section"),
            Action::new("a2", "A2", None, ActionCategory::ScriptContext).with_section("Other"),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 0, "Separators style = no headers");
    }

    #[test]
    fn grouped_items_empty_filtered_returns_empty() {
        let actions = vec![Action::new("a1", "A1", None, ActionCategory::ScriptContext)];
        let grouped = build_grouped_items_static(&actions, &[], SectionStyle::Headers);
        assert!(grouped.is_empty());
    }

    #[test]
    fn grouped_items_no_section_actions_with_headers_style() {
        let actions = vec![
            Action::new("a1", "A1", None, ActionCategory::ScriptContext),
            Action::new("a2", "A2", None, ActionCategory::ScriptContext),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // No sections means no headers, just items
        assert_eq!(grouped.len(), 2);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 0, "No sections = no headers");
    }

    // =========================================================================
    // 11. coerce_action_selection edge cases
    // =========================================================================

    #[test]
    fn coerce_selection_empty_rows_returns_none() {
        assert_eq!(coerce_action_selection(&[], 0), None);
    }

    #[test]
    fn coerce_selection_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".into()),
            GroupedActionItem::SectionHeader("B".into()),
            GroupedActionItem::SectionHeader("C".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
        assert_eq!(coerce_action_selection(&rows, 1), None);
        assert_eq!(coerce_action_selection(&rows, 2), None);
    }

    #[test]
    fn coerce_selection_on_item_returns_same_index() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".into()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(1));
    }

    #[test]
    fn coerce_selection_on_header_searches_down_first() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".into()),
            GroupedActionItem::Item(0),
            GroupedActionItem::Item(1),
        ];
        // Landing on header at 0, should go down to item at 1
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn coerce_selection_on_trailing_header_searches_up() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::Item(1),
            GroupedActionItem::SectionHeader("A".into()),
        ];
        // Landing on header at 2, no items below, should go up to item at 1
        assert_eq!(coerce_action_selection(&rows, 2), Some(1));
    }

    #[test]
    fn coerce_selection_out_of_bounds_clamped() {
        let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
        // Index 10 should be clamped to last index (1)
        assert_eq!(coerce_action_selection(&rows, 10), Some(1));
    }

    // =========================================================================
    // 12. ActionsDialog::format_shortcut_hint comprehensive
    // =========================================================================

    #[test]
    fn format_hint_cmd_c() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+c"), "⌘C");
    }

    #[test]
    fn format_hint_ctrl_shift_escape() {
        assert_eq!(
            ActionsDialog::format_shortcut_hint("ctrl+shift+escape"),
            "⌃⇧⎋"
        );
    }

    #[test]
    fn format_hint_alt_backspace() {
        assert_eq!(ActionsDialog::format_shortcut_hint("alt+backspace"), "⌥⌫");
    }

    #[test]
    fn format_hint_command_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("command+n"), "⌘N");
    }

    #[test]
    fn format_hint_meta_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("meta+n"), "⌘N");
    }

    #[test]
    fn format_hint_option_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("option+n"), "⌥N");
    }

    #[test]
    fn format_hint_control_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("control+x"), "⌃X");
    }

    #[test]
    fn format_hint_enter_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+enter"), "⌘↵");
    }

    #[test]
    fn format_hint_return_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+return"), "⌘↵");
    }

    #[test]
    fn format_hint_tab_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("shift+tab"), "⇧⇥");
    }

    #[test]
    fn format_hint_space_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+space"), "⌘␣");
    }

    #[test]
    fn format_hint_arrow_keys() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+up"), "⌘↑");
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+down"), "⌘↓");
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+left"), "⌘←");
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+right"), "⌘→");
    }

    #[test]
    fn format_hint_arrowup_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+arrowup"), "⌘↑");
    }

    #[test]
    fn format_hint_delete_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+delete"), "⌘⌫");
    }

    #[test]
    fn format_hint_esc_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("esc"), "⎋");
    }

    #[test]
    fn format_hint_super_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("super+k"), "⌘K");
    }

    #[test]
    fn format_hint_opt_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("opt+k"), "⌥K");
    }

    // =========================================================================
    // 13. Path context edge cases
    // =========================================================================

    #[test]
    fn path_context_dir_primary_title_includes_name() {
        let info = PathInfo {
            path: "/Users/test/Documents".into(),
            name: "Documents".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].title, "Open \"Documents\"");
        assert_eq!(actions[0].id, "open_directory");
    }

    #[test]
    fn path_context_file_primary_title_includes_name() {
        let info = PathInfo {
            path: "/Users/test/file.txt".into(),
            name: "file.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].title, "Select \"file.txt\"");
        assert_eq!(actions[0].id, "select_file");
    }

    #[test]
    fn path_context_trash_description_dir() {
        let info = PathInfo {
            path: "/Users/test/Documents".into(),
            name: "Documents".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
        assert_eq!(trash.description.as_ref().unwrap(), "Delete folder");
    }

    #[test]
    fn path_context_trash_description_file() {
        let info = PathInfo {
            path: "/Users/test/file.txt".into(),
            name: "file.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
        assert_eq!(trash.description.as_ref().unwrap(), "Delete file");
    }

    #[test]
    fn path_context_always_has_copy_path_and_copy_filename() {
        let info = PathInfo {
            path: "/Users/test/file.txt".into(),
            name: "file.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"copy_path"));
        assert!(ids.contains(&"copy_filename"));
    }

    #[test]
    fn path_context_has_open_in_editor_and_terminal() {
        let info = PathInfo {
            path: "/Users/test/file.txt".into(),
            name: "file.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"open_in_editor"));
        assert!(ids.contains(&"open_in_terminal"));
    }

