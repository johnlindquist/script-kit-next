    #[test]
    fn cat13_deeplink_name_special_chars_collapsed() {
        assert_eq!(to_deeplink_name("a!!b"), "a-b");
    }

    #[test]
    fn cat13_deeplink_name_leading_trailing_stripped() {
        assert_eq!(to_deeplink_name("  hello  "), "hello");
    }

    #[test]
    fn cat13_deeplink_name_unicode_preserved() {
        let result = to_deeplink_name("café");
        assert!(
            result.contains("caf"),
            "Should contain ascii part: {}",
            result
        );
        assert!(result.contains("é"), "Should preserve unicode: {}", result);
    }

    // =========================================================================
    // 14. Notes command bar create_quicklink and export actions
    // =========================================================================

    #[test]
    fn cat14_full_feature_has_create_quicklink() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "create_quicklink"));
    }

    #[test]
    fn cat14_create_quicklink_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = actions.iter().find(|a| a.id == "create_quicklink").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⇧⌘L"));
    }

    #[test]
    fn cat14_create_quicklink_icon_is_star() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = actions.iter().find(|a| a.id == "create_quicklink").unwrap();
        assert_eq!(a.icon, Some(IconName::Star));
    }

    #[test]
    fn cat14_export_action_present() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "export"));
    }

    #[test]
    fn cat14_export_section_is_export() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = actions.iter().find(|a| a.id == "export").unwrap();
        assert_eq!(a.section.as_deref(), Some("Export"));
    }

    #[test]
    fn cat14_trash_view_no_quicklink_no_export() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "create_quicklink"));
        assert!(!actions.iter().any(|a| a.id == "export"));
    }

    #[test]
    fn cat14_no_selection_no_quicklink_no_export() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "create_quicklink"));
        assert!(!actions.iter().any(|a| a.id == "export"));
    }

    // =========================================================================
    // 15. Action::new lowercase caching correctness
    // =========================================================================

    #[test]
    fn cat15_title_lower_is_lowercase() {
        let a = Action::new("id", "Hello World", None, ActionCategory::ScriptContext);
        assert_eq!(a.title_lower, "hello world");
    }

    #[test]
    fn cat15_description_lower_is_lowercase() {
        let a = Action::new(
            "id",
            "T",
            Some("Foo BAR".into()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(a.description_lower, Some("foo bar".into()));
    }

    #[test]
    fn cat15_description_lower_none_when_no_description() {
        let a = Action::new("id", "T", None, ActionCategory::ScriptContext);
        assert!(a.description_lower.is_none());
    }

    #[test]
    fn cat15_shortcut_lower_none_initially() {
        let a = Action::new("id", "T", None, ActionCategory::ScriptContext);
        assert!(a.shortcut_lower.is_none());
    }

    #[test]
    fn cat15_shortcut_lower_set_after_with_shortcut() {
        let a = Action::new("id", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
        assert_eq!(a.shortcut_lower, Some("⌘e".into()));
    }

    #[test]
    fn cat15_with_shortcut_opt_none_does_not_set() {
        let a = Action::new("id", "T", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
        assert!(a.shortcut_lower.is_none());
        assert!(a.shortcut.is_none());
    }

    #[test]
    fn cat15_with_shortcut_opt_some_sets() {
        let a = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘X".into()));
        assert_eq!(a.shortcut, Some("⌘X".into()));
        assert_eq!(a.shortcut_lower, Some("⌘x".into()));
    }

    // =========================================================================
    // 16. parse_shortcut_keycaps for special symbols
    // =========================================================================

    #[test]
    fn cat16_parse_cmd_c() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘C");
        assert_eq!(caps, vec!["⌘", "C"]);
    }

    #[test]
    fn cat16_parse_all_modifiers() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘⌃⌥⇧X");
        assert_eq!(caps, vec!["⌘", "⌃", "⌥", "⇧", "X"]);
    }

    #[test]
    fn cat16_parse_enter() {
        let caps = ActionsDialog::parse_shortcut_keycaps("↵");
        assert_eq!(caps, vec!["↵"]);
    }

    #[test]
    fn cat16_parse_escape() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⎋");
        assert_eq!(caps, vec!["⎋"]);
    }

    #[test]
    fn cat16_parse_arrows() {
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("↑"), vec!["↑"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("↓"), vec!["↓"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("←"), vec!["←"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("→"), vec!["→"]);
    }

    #[test]
    fn cat16_parse_space() {
        let caps = ActionsDialog::parse_shortcut_keycaps("␣");
        assert_eq!(caps, vec!["␣"]);
    }

    #[test]
    fn cat16_parse_tab() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⇥");
        assert_eq!(caps, vec!["⇥"]);
    }

    #[test]
    fn cat16_parse_backspace() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌫");
        assert_eq!(caps, vec!["⌫"]);
    }

    #[test]
    fn cat16_parse_lowercase_uppercased() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘a");
        assert_eq!(caps, vec!["⌘", "A"]);
    }

    #[test]
    fn cat16_parse_empty() {
        let caps = ActionsDialog::parse_shortcut_keycaps("");
        assert!(caps.is_empty());
    }

    // =========================================================================
    // 17. score_action boundary thresholds
    // =========================================================================

    #[test]
    fn cat17_prefix_match_100() {
        let a = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
        assert_eq!(ActionsDialog::score_action(&a, "edit"), 100);
    }

    #[test]
    fn cat17_contains_match_50() {
        let a = Action::new("id", "Copy Edit Path", None, ActionCategory::ScriptContext);
        assert_eq!(ActionsDialog::score_action(&a, "edit"), 50);
    }

    #[test]
    fn cat17_fuzzy_match_25() {
        let a = Action::new(
            "id",
            "Reveal in Finder",
            None,
            ActionCategory::ScriptContext,
        );
        // "rvf" is a subsequence of "reveal in finder" (r-e-v-e-a-l-_-i-n-_-f)
        assert_eq!(ActionsDialog::score_action(&a, "rvf"), 25);
    }

    #[test]
    fn cat17_description_bonus_15() {
        let a = Action::new(
            "id",
            "Open",
            Some("Edit file in editor".into()),
            ActionCategory::ScriptContext,
        );
        // "editor" not in title but in description
        assert_eq!(ActionsDialog::score_action(&a, "editor"), 15);
    }

    #[test]
    fn cat17_shortcut_bonus_10() {
        let a = Action::new("id", "Open", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
        // "⌘e" is in shortcut_lower
        assert_eq!(ActionsDialog::score_action(&a, "⌘e"), 10);
    }

    #[test]
    fn cat17_no_match_0() {
        let a = Action::new("id", "Open", None, ActionCategory::ScriptContext);
        assert_eq!(ActionsDialog::score_action(&a, "xyz"), 0);
    }

    #[test]
    fn cat17_prefix_plus_description_115() {
        let a = Action::new(
            "id",
            "Edit Script",
            Some("Edit the script file".into()),
            ActionCategory::ScriptContext,
        );
        // "edit" is prefix (100) + description contains "edit" (15)
        assert_eq!(ActionsDialog::score_action(&a, "edit"), 115);
    }

    // =========================================================================
    // 18. fuzzy_match edge cases
    // =========================================================================

    #[test]
    fn cat18_empty_needle_true() {
        assert!(ActionsDialog::fuzzy_match("anything", ""));
    }

    #[test]
    fn cat18_empty_haystack_false() {
        assert!(!ActionsDialog::fuzzy_match("", "a"));
    }

    #[test]
    fn cat18_both_empty_true() {
        assert!(ActionsDialog::fuzzy_match("", ""));
    }

    #[test]
    fn cat18_exact_match() {
        assert!(ActionsDialog::fuzzy_match("abc", "abc"));
    }

    #[test]
    fn cat18_subsequence() {
        assert!(ActionsDialog::fuzzy_match("abcdef", "ace"));
    }

    #[test]
    fn cat18_no_match() {
        assert!(!ActionsDialog::fuzzy_match("abc", "xyz"));
    }

    #[test]
    fn cat18_needle_longer_than_haystack() {
        assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
    }

    // =========================================================================
    // 19. build_grouped_items_static
    // =========================================================================

    #[test]
    fn cat19_empty_filtered_empty_grouped() {
        let actions: Vec<Action> = vec![];
        let filtered: Vec<usize> = vec![];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(grouped.is_empty());
    }

    #[test]
    fn cat19_headers_inserts_section_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Should have: S1 header, item 0, S2 header, item 1
        assert_eq!(grouped.len(), 4);
        assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(_)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[2], GroupedActionItem::SectionHeader(_)));
        assert!(matches!(grouped[3], GroupedActionItem::Item(1)));
    }

    #[test]
    fn cat19_separators_no_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        // No section headers
        assert_eq!(grouped.len(), 2);
        assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(1)));
    }

    #[test]
    fn cat19_none_style_no_headers() {
        let actions =
            vec![Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1")];
        let filtered = vec![0];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        assert_eq!(grouped.len(), 1);
        assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
    }

    #[test]
    fn cat19_same_section_no_duplicate_header() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S1"),
        ];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // S1 header, item 0, item 1 (no second header)
        assert_eq!(grouped.len(), 3);
    }

    // =========================================================================
    // 20. coerce_action_selection edge cases
    // =========================================================================

    #[test]
    fn cat20_empty_returns_none() {
        assert_eq!(coerce_action_selection(&[], 0), None);
    }

    #[test]
    fn cat20_on_item_returns_same() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    }

    #[test]
    fn cat20_header_searches_down() {
        let rows = vec![
            GroupedActionItem::SectionHeader("S".into()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn cat20_trailing_header_searches_up() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("S".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(0));
    }

    #[test]
    fn cat20_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".into()),
            GroupedActionItem::SectionHeader("B".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn cat20_out_of_bounds_clamped() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 100), Some(0));
    }

    // =========================================================================
    // 21. New chat actions structure
    // =========================================================================

