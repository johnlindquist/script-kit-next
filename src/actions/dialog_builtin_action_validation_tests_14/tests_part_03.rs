    #[test]
    fn cat11_grouped_items_headers_insert_for_each_section_change() {
        let actions = vec![
            Action::new("a1", "A1", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("a2", "A2", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("a3", "A3", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered: Vec<usize> = (0..3).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // S1 header, a1, a2, S2 header, a3
        assert_eq!(grouped.len(), 5);
        assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(_)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[2], GroupedActionItem::Item(1)));
        assert!(matches!(grouped[3], GroupedActionItem::SectionHeader(_)));
        assert!(matches!(grouped[4], GroupedActionItem::Item(2)));
    }

    #[test]
    fn cat11_grouped_items_separators_no_headers() {
        let actions = vec![
            Action::new("a1", "A1", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("a2", "A2", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        assert_eq!(grouped.len(), 2);
        assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(1)));
    }

    #[test]
    fn cat11_grouped_items_none_style_no_headers() {
        let actions = vec![
            Action::new("a1", "A1", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("a2", "A2", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        assert_eq!(grouped.len(), 2);
    }

    #[test]
    fn cat11_grouped_items_empty_input() {
        let actions: Vec<Action> = vec![];
        let filtered: Vec<usize> = vec![];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(grouped.is_empty());
    }

    #[test]
    fn cat11_grouped_items_no_section_no_header() {
        let actions = vec![
            Action::new("a1", "A1", None, ActionCategory::ScriptContext),
            Action::new("a2", "A2", None, ActionCategory::ScriptContext),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // No sections means no headers
        assert_eq!(grouped.len(), 2);
    }

    // =========================================================================
    // 12. coerce_action_selection with interleaved headers
    // =========================================================================

    #[test]
    fn cat12_coerce_empty_returns_none() {
        assert_eq!(coerce_action_selection(&[], 0), None);
    }

    #[test]
    fn cat12_coerce_on_item_returns_same() {
        let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
        assert_eq!(coerce_action_selection(&rows, 0), Some(0));
        assert_eq!(coerce_action_selection(&rows, 1), Some(1));
    }

    #[test]
    fn cat12_coerce_header_searches_down() {
        let rows = vec![
            GroupedActionItem::SectionHeader("H".into()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn cat12_coerce_trailing_header_searches_up() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("H".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(0));
    }

    #[test]
    fn cat12_coerce_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("H1".into()),
            GroupedActionItem::SectionHeader("H2".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn cat12_coerce_out_of_bounds_clamped() {
        let rows = vec![
            GroupedActionItem::SectionHeader("H".into()),
            GroupedActionItem::Item(0),
        ];
        // Index 99 should be clamped to last index (1), which is an Item
        assert_eq!(coerce_action_selection(&rows, 99), Some(1));
    }

    #[test]
    fn cat12_coerce_interleaved_headers() {
        let rows = vec![
            GroupedActionItem::SectionHeader("S1".into()),
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("S2".into()),
            GroupedActionItem::Item(1),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
        assert_eq!(coerce_action_selection(&rows, 2), Some(3));
    }

    // =========================================================================
    // 13. score_action stacking: title + description + shortcut all match
    // =========================================================================

    #[test]
    fn cat13_score_prefix_plus_desc_plus_shortcut() {
        let action = Action::new(
            "copy_path",
            "Copy Path",
            Some("Copy the full path".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧C");
        // "copy" matches: prefix title (100) + description (15) = 115
        let score = ActionsDialog::score_action(&action, "copy");
        assert!(
            score >= 115,
            "Expected >=115 for prefix+desc match, got {}",
            score
        );
    }

    #[test]
    fn cat13_score_contains_title_only() {
        let action = Action::new(
            "x",
            "Open Copy Path",
            Some("unrelated".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "copy");
        assert_eq!(score, 50);
    }

    #[test]
    fn cat13_score_fuzzy_title() {
        let action = Action::new(
            "x",
            "Create Proxy",
            Some("unrelated".to_string()),
            ActionCategory::ScriptContext,
        );
        // "cry" is a subsequence of "create proxy"
        let score = ActionsDialog::score_action(&action, "cry");
        assert!(score >= 25, "Expected fuzzy match >=25, got {}", score);
    }

    #[test]
    fn cat13_score_description_only() {
        let action = Action::new(
            "x",
            "Unrelated Title",
            Some("Copy the path to clipboard".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "clipboard");
        assert_eq!(score, 15);
    }

    #[test]
    fn cat13_score_no_match_zero() {
        let action = Action::new(
            "x",
            "Edit Script",
            Some("Open in editor".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘E");
        let score = ActionsDialog::score_action(&action, "zzzzz");
        assert_eq!(score, 0);
    }

    // =========================================================================
    // 14. fuzzy_match Unicode subsequence
    // =========================================================================

    #[test]
    fn cat14_fuzzy_match_ascii_subsequence() {
        assert!(ActionsDialog::fuzzy_match("hello world", "hlo"));
    }

    #[test]
    fn cat14_fuzzy_match_exact() {
        assert!(ActionsDialog::fuzzy_match("exact", "exact"));
    }

    #[test]
    fn cat14_fuzzy_match_empty_needle() {
        assert!(ActionsDialog::fuzzy_match("anything", ""));
    }

    #[test]
    fn cat14_fuzzy_match_no_match() {
        assert!(!ActionsDialog::fuzzy_match("abc", "xyz"));
    }

    #[test]
    fn cat14_fuzzy_match_needle_longer() {
        assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
    }

    #[test]
    fn cat14_fuzzy_match_both_empty() {
        assert!(ActionsDialog::fuzzy_match("", ""));
    }

    #[test]
    fn cat14_fuzzy_match_empty_haystack_nonempty_needle() {
        assert!(!ActionsDialog::fuzzy_match("", "a"));
    }

    // =========================================================================
    // 15. parse_shortcut_keycaps with slash and number inputs
    // =========================================================================

    #[test]
    fn cat15_parse_keycaps_cmd_slash() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘/");
        assert_eq!(keycaps, vec!["⌘", "/"]);
    }

    #[test]
    fn cat15_parse_keycaps_number() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘1");
        assert_eq!(keycaps, vec!["⌘", "1"]);
    }

    #[test]
    fn cat15_parse_keycaps_modifier_chain() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⇧⌥C");
        assert_eq!(keycaps, vec!["⌘", "⇧", "⌥", "C"]);
    }

    #[test]
    fn cat15_parse_keycaps_enter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("↵");
        assert_eq!(keycaps, vec!["↵"]);
    }

    #[test]
    fn cat15_parse_keycaps_escape() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⎋");
        assert_eq!(keycaps, vec!["⎋"]);
    }

    #[test]
    fn cat15_parse_keycaps_arrows() {
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("↑"), vec!["↑"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("↓"), vec!["↓"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("←"), vec!["←"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("→"), vec!["→"]);
    }

    #[test]
    fn cat15_parse_keycaps_lowercase_uppercased() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘c");
        assert_eq!(keycaps, vec!["⌘", "C"]);
    }

    // =========================================================================
    // 16. to_deeplink_name edge cases
    // =========================================================================

    #[test]
    fn cat16_deeplink_numeric_only() {
        assert_eq!(to_deeplink_name("12345"), "12345");
    }

    #[test]
    fn cat16_deeplink_all_special_returns_empty() {
        assert_eq!(to_deeplink_name("!@#$%^&*()"), "");
    }

    #[test]
    fn cat16_deeplink_mixed_case_lowered() {
        assert_eq!(to_deeplink_name("Hello World"), "hello-world");
    }

    #[test]
    fn cat16_deeplink_consecutive_specials_collapsed() {
        assert_eq!(to_deeplink_name("a!!b"), "a-b");
    }

    #[test]
    fn cat16_deeplink_leading_trailing_stripped() {
        assert_eq!(to_deeplink_name("---hello---"), "hello");
    }

    #[test]
    fn cat16_deeplink_unicode_preserved() {
        // Unicode alphanumeric chars (like CJK) should be preserved
        let result = to_deeplink_name("café");
        assert!(result.contains("caf"));
        assert!(result.contains("é"));
    }

    #[test]
    fn cat16_deeplink_underscores_become_hyphens() {
        assert_eq!(to_deeplink_name("hello_world"), "hello-world");
    }

    // =========================================================================
    // 17. Action with_shortcut_opt Some vs None
    // =========================================================================

    #[test]
    fn cat17_with_shortcut_opt_none_preserves_none() {
        let action =
            Action::new("x", "X", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
        assert!(action.shortcut.is_none());
        assert!(action.shortcut_lower.is_none());
    }

    #[test]
    fn cat17_with_shortcut_opt_some_sets_shortcut() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘C".to_string()));
        assert_eq!(action.shortcut, Some("⌘C".to_string()));
        assert!(action.shortcut_lower.is_some());
    }

    #[test]
    fn cat17_with_shortcut_sets_lower() {
        let action =
            Action::new("x", "X", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧K");
        assert_eq!(action.shortcut_lower, Some("⌘⇧k".to_string()));
    }

    #[test]
    fn cat17_action_new_no_shortcut_lower() {
        let action = Action::new("x", "Title", None, ActionCategory::ScriptContext);
        assert!(action.shortcut.is_none());
        assert!(action.shortcut_lower.is_none());
    }

    #[test]
    fn cat17_action_title_lower_cached() {
        let action = Action::new("x", "Hello World", None, ActionCategory::ScriptContext);
        assert_eq!(action.title_lower, "hello world");
    }

    #[test]
    fn cat17_action_description_lower_cached() {
        let action = Action::new(
            "x",
            "T",
            Some("Open in Editor".to_string()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.description_lower, Some("open in editor".to_string()));
    }

    // =========================================================================
    // 18. CommandBarConfig notes_style field completeness
    // =========================================================================

    #[test]
    fn cat18_notes_style_search_at_top() {
        let cfg = CommandBarConfig::notes_style();
        assert_eq!(cfg.dialog_config.search_position, SearchPosition::Top);
    }

    #[test]
    fn cat18_notes_style_separators() {
        let cfg = CommandBarConfig::notes_style();
        assert_eq!(cfg.dialog_config.section_style, SectionStyle::Separators);
    }

    #[test]
    fn cat18_notes_style_anchor_top() {
        let cfg = CommandBarConfig::notes_style();
        assert_eq!(cfg.dialog_config.anchor, AnchorPosition::Top);
    }

    #[test]
    fn cat18_notes_style_icons_enabled() {
        let cfg = CommandBarConfig::notes_style();
        assert!(cfg.dialog_config.show_icons);
    }

    #[test]
    fn cat18_notes_style_footer_enabled() {
        let cfg = CommandBarConfig::notes_style();
        assert!(cfg.dialog_config.show_footer);
    }

    #[test]
    fn cat18_notes_style_close_defaults_true() {
        let cfg = CommandBarConfig::notes_style();
        assert!(cfg.close_on_select);
        assert!(cfg.close_on_click_outside);
        assert!(cfg.close_on_escape);
    }

    #[test]
    fn cat18_ai_style_search_at_top_headers() {
        let cfg = CommandBarConfig::ai_style();
        assert_eq!(cfg.dialog_config.search_position, SearchPosition::Top);
        assert_eq!(cfg.dialog_config.section_style, SectionStyle::Headers);
    }

    #[test]
    fn cat18_main_menu_search_at_bottom() {
        let cfg = CommandBarConfig::main_menu_style();
        assert_eq!(cfg.dialog_config.search_position, SearchPosition::Bottom);
        assert_eq!(cfg.dialog_config.section_style, SectionStyle::Separators);
    }

    #[test]
    fn cat18_no_search_hidden() {
        let cfg = CommandBarConfig::no_search();
        assert_eq!(cfg.dialog_config.search_position, SearchPosition::Hidden);
    }

    // =========================================================================
    // 19. Clipboard destructive ordering invariant across pin states
    // =========================================================================

