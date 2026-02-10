    #[test]
    fn cat14_all_arrows() {
        let caps = ActionsDialog::parse_shortcut_keycaps("↑↓←→");
        assert_eq!(caps, vec!["↑", "↓", "←", "→"]);
    }

    #[test]
    fn cat14_cmd_shift_delete() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘⇧⌫");
        assert_eq!(caps, vec!["⌘", "⇧", "⌫"]);
    }

    // =========================================================================
    // Category 15: score_action with empty search string
    // =========================================================================

    #[test]
    fn cat15_empty_search_matches_prefix() {
        let action = Action::new("test", "Test Action", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "");
        // Empty string is a prefix of everything
        assert_eq!(score, 100);
    }

    #[test]
    fn cat15_single_char_search() {
        let action = Action::new("test", "Test Action", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "t");
        assert_eq!(score, 100); // prefix match
    }

    #[test]
    fn cat15_no_match_returns_zero() {
        let action = Action::new("test", "Test Action", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "xyz");
        assert_eq!(score, 0);
    }

    #[test]
    fn cat15_description_bonus_stacking() {
        let action = Action::new(
            "test",
            "Test Action",
            Some("test description".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "test");
        // prefix(100) + description(15) = 115
        assert_eq!(score, 115);
    }

    #[test]
    fn cat15_shortcut_bonus_stacking() {
        let action = Action::new("test", "Test Action", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘T");
        let score = ActionsDialog::score_action(&action, "⌘t");
        // No title match for "⌘t", but shortcut match: 10
        assert_eq!(score, 10);
    }

    // =========================================================================
    // Category 16: fuzzy_match edge cases
    // =========================================================================

    #[test]
    fn cat16_repeated_chars_in_haystack() {
        // "aaa" in "banana" should match (b-a-n-a-n-a has three a's)
        assert!(ActionsDialog::fuzzy_match("banana", "aaa"));
    }

    #[test]
    fn cat16_repeated_chars_insufficient() {
        // "aaaa" in "banana" should fail (only 3 a's available)
        assert!(!ActionsDialog::fuzzy_match("banana", "aaaa"));
    }

    #[test]
    fn cat16_single_char_match() {
        assert!(ActionsDialog::fuzzy_match("hello", "h"));
    }

    #[test]
    fn cat16_single_char_no_match() {
        assert!(!ActionsDialog::fuzzy_match("hello", "z"));
    }

    #[test]
    fn cat16_full_string_match() {
        assert!(ActionsDialog::fuzzy_match("hello", "hello"));
    }

    // =========================================================================
    // Category 17: build_grouped_items_static — section change behavior
    // =========================================================================

    #[test]
    fn cat17_headers_style_adds_header_on_section_change() {
        let actions = vec![
            Action::new("a1", "Action 1", None, ActionCategory::ScriptContext)
                .with_section("Alpha"),
            Action::new("a2", "Action 2", None, ActionCategory::ScriptContext).with_section("Beta"),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Should be: Header("Alpha"), Item(0), Header("Beta"), Item(1)
        assert_eq!(grouped.len(), 4);
        assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(ref s) if s == "Alpha"));
        assert!(matches!(grouped[1], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[2], GroupedActionItem::SectionHeader(ref s) if s == "Beta"));
        assert!(matches!(grouped[3], GroupedActionItem::Item(1)));
    }

    #[test]
    fn cat17_headers_style_same_section_no_duplicate_header() {
        let actions = vec![
            Action::new("a1", "Action 1", None, ActionCategory::ScriptContext)
                .with_section("Alpha"),
            Action::new("a2", "Action 2", None, ActionCategory::ScriptContext)
                .with_section("Alpha"),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Should be: Header("Alpha"), Item(0), Item(1)
        assert_eq!(grouped.len(), 3);
        assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(ref s) if s == "Alpha"));
    }

    #[test]
    fn cat17_separators_style_no_headers() {
        let actions = vec![
            Action::new("a1", "Action 1", None, ActionCategory::ScriptContext)
                .with_section("Alpha"),
            Action::new("a2", "Action 2", None, ActionCategory::ScriptContext).with_section("Beta"),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        // No headers for separators mode
        assert_eq!(grouped.len(), 2);
        assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(1)));
    }

    #[test]
    fn cat17_none_style_no_headers() {
        let actions = vec![
            Action::new("a1", "Action 1", None, ActionCategory::ScriptContext)
                .with_section("Alpha"),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        assert_eq!(grouped.len(), 1);
        assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
    }

    #[test]
    fn cat17_empty_filtered_returns_empty() {
        let actions = vec![Action::new(
            "a1",
            "Action 1",
            None,
            ActionCategory::ScriptContext,
        )];
        let filtered: Vec<usize> = vec![];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(grouped.is_empty());
    }

    // =========================================================================
    // Category 18: coerce_action_selection — consecutive headers
    // =========================================================================

    #[test]
    fn cat18_two_consecutive_headers_then_item() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".to_string()),
            GroupedActionItem::SectionHeader("B".to_string()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(2));
    }

    #[test]
    fn cat18_header_at_end_searches_up() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("A".to_string()),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(0));
    }

    #[test]
    fn cat18_single_item_returns_itself() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    }

    #[test]
    fn cat18_index_beyond_bounds_clamped() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 99), Some(0));
    }

    #[test]
    fn cat18_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".to_string()),
            GroupedActionItem::SectionHeader("B".to_string()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    // =========================================================================
    // Category 19: CommandBarConfig main_menu_style values
    // =========================================================================

    #[test]
    fn cat19_main_menu_search_bottom() {
        let config = CommandBarConfig::main_menu_style();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Bottom);
    }

    #[test]
    fn cat19_main_menu_separators() {
        let config = CommandBarConfig::main_menu_style();
        assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
    }

    #[test]
    fn cat19_main_menu_anchor_bottom() {
        let config = CommandBarConfig::main_menu_style();
        assert_eq!(config.dialog_config.anchor, AnchorPosition::Bottom);
    }

    #[test]
    fn cat19_main_menu_no_icons() {
        let config = CommandBarConfig::main_menu_style();
        assert!(!config.dialog_config.show_icons);
    }

    #[test]
    fn cat19_main_menu_no_footer() {
        let config = CommandBarConfig::main_menu_style();
        assert!(!config.dialog_config.show_footer);
    }

    // =========================================================================
    // Category 20: CommandBarConfig ai_style values
    // =========================================================================

    #[test]
    fn cat20_ai_style_search_top() {
        let config = CommandBarConfig::ai_style();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
    }

    #[test]
    fn cat20_ai_style_headers() {
        let config = CommandBarConfig::ai_style();
        assert_eq!(config.dialog_config.section_style, SectionStyle::Headers);
    }

    #[test]
    fn cat20_ai_style_anchor_top() {
        let config = CommandBarConfig::ai_style();
        assert_eq!(config.dialog_config.anchor, AnchorPosition::Top);
    }

    #[test]
    fn cat20_ai_style_icons_enabled() {
        let config = CommandBarConfig::ai_style();
        assert!(config.dialog_config.show_icons);
    }

    #[test]
    fn cat20_ai_style_footer_enabled() {
        let config = CommandBarConfig::ai_style();
        assert!(config.dialog_config.show_footer);
    }

    // =========================================================================
    // Category 21: CommandBarConfig no_search values
    // =========================================================================

    #[test]
    fn cat21_no_search_hidden() {
        let config = CommandBarConfig::no_search();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Hidden);
    }

    #[test]
    fn cat21_no_search_separators() {
        let config = CommandBarConfig::no_search();
        assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
    }

    #[test]
    fn cat21_no_search_close_defaults_true() {
        let config = CommandBarConfig::no_search();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    // =========================================================================
    // Category 22: Action with_section sets section field
    // =========================================================================

    #[test]
    fn cat22_with_section_sets_field() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_section("MySection");
        assert_eq!(action.section, Some("MySection".to_string()));
    }

    #[test]
    fn cat22_no_section_by_default() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
        assert!(action.section.is_none());
    }

    #[test]
    fn cat22_with_section_preserves_shortcut() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘T")
            .with_section("MySection");
        assert_eq!(action.shortcut, Some("⌘T".to_string()));
        assert_eq!(action.section, Some("MySection".to_string()));
    }

    #[test]
    fn cat22_with_section_preserves_icon() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_icon(IconName::Star)
            .with_section("MySection");
        assert_eq!(action.icon, Some(IconName::Star));
        assert_eq!(action.section, Some("MySection".to_string()));
    }

    // =========================================================================
    // Category 23: ScriptInfo is_agent defaults and combined flags
    // =========================================================================

    #[test]
    fn cat23_new_is_agent_false() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        assert!(!script.is_agent);
    }

    #[test]
    fn cat23_builtin_is_agent_false() {
        let builtin = ScriptInfo::builtin("Clipboard History");
        assert!(!builtin.is_agent);
    }

    #[test]
    fn cat23_scriptlet_is_agent_false() {
        let scriptlet = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        assert!(!scriptlet.is_agent);
    }

    #[test]
    fn cat23_with_all_is_agent_false() {
        let script = ScriptInfo::with_all("Test", "/path", true, "Run", None, None);
        assert!(!script.is_agent);
    }

    #[test]
    fn cat23_agent_mutually_exclusive_with_script() {
        let mut script = ScriptInfo::new("Test", "/path");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        // Agent should NOT have view_logs (script-only)
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
        // Agent SHOULD have edit_script (with "Edit Agent" title)
        assert!(actions.iter().any(|a| a.id == "edit_script"));
    }

    // =========================================================================
    // Category 24: Clipboard save_snippet and save_file shortcuts
    // =========================================================================

    #[test]
    fn cat24_save_snippet_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let snippet = actions
            .iter()
            .find(|a| a.id == "clipboard_save_snippet")
            .unwrap();
        assert_eq!(snippet.shortcut.as_ref().unwrap(), "⇧⌘S");
    }

    #[test]
    fn cat24_save_file_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let save = actions
            .iter()
            .find(|a| a.id == "clipboard_save_file")
            .unwrap();
        assert_eq!(save.shortcut.as_ref().unwrap(), "⌥⇧⌘S");
    }

    #[test]
    fn cat24_save_snippet_title() {
        let entry = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let snippet = actions
            .iter()
            .find(|a| a.id == "clipboard_save_snippet")
            .unwrap();
        assert_eq!(snippet.title, "Save Text as Snippet");
    }

