    #[test]
    fn cat14_score_contains_match_50() {
        let action = Action::new(
            "copy_path",
            "Copy Path",
            Some("Copy to clipboard".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "path");
        assert!(
            (50..100).contains(&score),
            "Contains match should be 50-99, got {}",
            score
        );
    }

    #[test]
    fn cat14_score_no_match_zero() {
        let action = Action::new(
            "edit_script",
            "Edit Script",
            None,
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "zzz");
        assert_eq!(score, 0, "No match should be 0, got {}", score);
    }

    #[test]
    fn cat14_score_empty_query_prefix() {
        let action = Action::new(
            "edit_script",
            "Edit Script",
            None,
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "");
        assert!(
            score >= 100,
            "Empty query is prefix match (100+), got {}",
            score
        );
    }

    #[test]
    fn cat14_score_description_bonus() {
        let action = Action::new(
            "open_file",
            "Open File",
            Some("Edit the file in your editor".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "editor");
        assert!(
            score >= 15,
            "Description match should give 15+, got {}",
            score
        );
    }

    // =========================================================================
    // Category 15: fuzzy_match subsequence correctness
    // Tests the fuzzy matching helper function.
    // =========================================================================

    #[test]
    fn cat15_fuzzy_exact_match() {
        assert!(ActionsDialog::fuzzy_match("hello", "hello"));
    }

    #[test]
    fn cat15_fuzzy_subsequence_match() {
        assert!(ActionsDialog::fuzzy_match("hello world", "hlwrd"));
    }

    #[test]
    fn cat15_fuzzy_no_match() {
        assert!(!ActionsDialog::fuzzy_match("hello", "xyz"));
    }

    #[test]
    fn cat15_fuzzy_empty_needle() {
        assert!(ActionsDialog::fuzzy_match("hello", ""));
    }

    #[test]
    fn cat15_fuzzy_empty_haystack() {
        assert!(!ActionsDialog::fuzzy_match("", "a"));
    }

    #[test]
    fn cat15_fuzzy_needle_longer_than_haystack() {
        assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
    }

    #[test]
    fn cat15_fuzzy_both_empty() {
        assert!(ActionsDialog::fuzzy_match("", ""));
    }

    // =========================================================================
    // Category 16: build_grouped_items_static section style behavior
    // Tests that different section styles produce correct GroupedActionItem layouts.
    // =========================================================================

    #[test]
    fn cat16_grouped_headers_style_adds_section_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered = vec![0, 1];
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Should have: Header(S1), Item(0), Header(S2), Item(1)
        assert_eq!(items.len(), 4);
        assert!(matches!(items[0], GroupedActionItem::SectionHeader(_)));
        assert!(matches!(items[1], GroupedActionItem::Item(0)));
        assert!(matches!(items[2], GroupedActionItem::SectionHeader(_)));
        assert!(matches!(items[3], GroupedActionItem::Item(1)));
    }

    #[test]
    fn cat16_grouped_separators_style_no_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered = vec![0, 1];
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        // No headers, just items
        assert_eq!(items.len(), 2);
        assert!(matches!(items[0], GroupedActionItem::Item(0)));
        assert!(matches!(items[1], GroupedActionItem::Item(1)));
    }

    #[test]
    fn cat16_grouped_none_style_no_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered = vec![0, 1];
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        assert_eq!(items.len(), 2);
        assert!(matches!(items[0], GroupedActionItem::Item(0)));
    }

    #[test]
    fn cat16_grouped_empty_filtered_returns_empty() {
        let actions =
            vec![Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1")];
        let filtered: Vec<usize> = vec![];
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(items.is_empty());
    }

    #[test]
    fn cat16_grouped_same_section_one_header() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S1"),
        ];
        let filtered = vec![0, 1];
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Header(S1), Item(0), Item(1) — no duplicate header
        assert_eq!(items.len(), 3);
        let header_count = items
            .iter()
            .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 1);
    }

    // =========================================================================
    // Category 17: coerce_action_selection header skipping
    // Tests that selection correctly skips over section headers.
    // =========================================================================

    #[test]
    fn cat17_coerce_on_item_stays() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    }

    #[test]
    fn cat17_coerce_on_header_moves_down() {
        let rows = vec![
            GroupedActionItem::SectionHeader("S1".to_string()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn cat17_coerce_trailing_header_moves_up() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("S1".to_string()),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(0));
    }

    #[test]
    fn cat17_coerce_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("S1".to_string()),
            GroupedActionItem::SectionHeader("S2".to_string()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn cat17_coerce_empty_returns_none() {
        let rows: Vec<GroupedActionItem> = vec![];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    // =========================================================================
    // Category 18: CommandBarConfig preset defaults — close flags
    // Validates that all presets have consistent close behavior defaults.
    // =========================================================================

    #[test]
    fn cat18_default_config_close_on_select() {
        let config = CommandBarConfig::default();
        assert!(config.close_on_select);
    }

    #[test]
    fn cat18_default_config_close_on_escape() {
        let config = CommandBarConfig::default();
        assert!(config.close_on_escape);
    }

    #[test]
    fn cat18_ai_style_close_on_select() {
        let config = CommandBarConfig::ai_style();
        assert!(config.close_on_select);
    }

    #[test]
    fn cat18_main_menu_close_on_select() {
        let config = CommandBarConfig::main_menu_style();
        assert!(config.close_on_select);
    }

    #[test]
    fn cat18_notes_style_close_on_select() {
        let config = CommandBarConfig::notes_style();
        assert!(config.close_on_select);
    }

    #[test]
    fn cat18_no_search_close_on_escape() {
        let config = CommandBarConfig::no_search();
        assert!(config.close_on_escape);
    }

    // =========================================================================
    // Category 19: Action lowercase caching correctness
    // Verifies that title_lower, description_lower, and shortcut_lower
    // are correctly pre-computed when constructing an Action.
    // =========================================================================

    #[test]
    fn cat19_title_lower_computed() {
        let action = Action::new("test", "Edit Script", None, ActionCategory::ScriptContext);
        assert_eq!(action.title_lower, "edit script");
    }

    #[test]
    fn cat19_description_lower_computed() {
        let action = Action::new(
            "test",
            "Test",
            Some("Open In Editor".to_string()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.description_lower, Some("open in editor".to_string()));
    }

    #[test]
    fn cat19_description_lower_none_when_no_desc() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
        assert!(action.description_lower.is_none());
    }

    #[test]
    fn cat19_shortcut_lower_none_initially() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
        assert!(action.shortcut_lower.is_none());
    }

    #[test]
    fn cat19_shortcut_lower_set_after_with_shortcut() {
        let action =
            Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
        assert_eq!(action.shortcut_lower, Some("⌘e".to_string()));
    }

    #[test]
    fn cat19_unicode_title_lower() {
        let action = Action::new("test", "ÜBER SCRIPT", None, ActionCategory::ScriptContext);
        assert_eq!(action.title_lower, "über script");
    }

    // =========================================================================
    // Category 20: Action builder chaining — field preservation
    // Verifies that chaining with_shortcut, with_icon, with_section
    // preserves previously set fields.
    // =========================================================================

    #[test]
    fn cat20_with_shortcut_then_icon_preserves_shortcut() {
        let action = Action::new("t", "T", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘E")
            .with_icon(IconName::Copy);
        assert_eq!(action.shortcut.as_deref(), Some("⌘E"));
        assert_eq!(action.icon, Some(IconName::Copy));
    }

    #[test]
    fn cat20_with_icon_then_section_preserves_icon() {
        let action = Action::new("t", "T", None, ActionCategory::ScriptContext)
            .with_icon(IconName::Star)
            .with_section("Help");
        assert_eq!(action.icon, Some(IconName::Star));
        assert_eq!(action.section.as_deref(), Some("Help"));
    }

    #[test]
    fn cat20_full_chain_preserves_all() {
        let action = Action::new(
            "t",
            "T",
            Some("Desc".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘E")
        .with_icon(IconName::Settings)
        .with_section("Settings");
        assert_eq!(action.shortcut.as_deref(), Some("⌘E"));
        assert_eq!(action.icon, Some(IconName::Settings));
        assert_eq!(action.section.as_deref(), Some("Settings"));
        assert_eq!(action.description.as_deref(), Some("Desc"));
    }

    #[test]
    fn cat20_with_shortcut_opt_none_preserves_existing() {
        let action = Action::new("t", "T", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘E")
            .with_shortcut_opt(None);
        // with_shortcut_opt(None) preserves the existing shortcut
        assert_eq!(action.shortcut.as_deref(), Some("⌘E"));
    }

    #[test]
    fn cat20_with_shortcut_opt_some_sets() {
        let action = Action::new("t", "T", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘F".to_string()));
        assert_eq!(action.shortcut.as_deref(), Some("⌘F"));
    }

    // =========================================================================
    // Category 21: ScriptInfo constructor defaults and mutability
    // Validates default field values across constructors and mutability of flags.
    // =========================================================================

    #[test]
    fn cat21_new_defaults() {
        let s = ScriptInfo::new("test", "/path/test.ts");
        assert!(s.is_script);
        assert!(!s.is_scriptlet);
        assert!(!s.is_agent);
        assert_eq!(s.action_verb, "Run");
        assert!(s.shortcut.is_none());
        assert!(s.alias.is_none());
        assert!(!s.is_suggested);
    }

    #[test]
    fn cat21_builtin_defaults() {
        let s = ScriptInfo::builtin("Test");
        assert!(!s.is_script);
        assert!(!s.is_scriptlet);
        assert!(!s.is_agent);
        assert!(s.path.is_empty());
    }

    #[test]
    fn cat21_scriptlet_defaults() {
        let s = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        assert!(!s.is_script);
        assert!(s.is_scriptlet);
        assert!(!s.is_agent);
    }

    #[test]
    fn cat21_agent_via_mutation() {
        let mut s = ScriptInfo::new("Agent", "/path/agent.md");
        s.is_agent = true;
        s.is_script = false;
        assert!(s.is_agent);
        assert!(!s.is_script);
    }

    #[test]
    fn cat21_with_frecency_builder() {
        let s = ScriptInfo::new("test", "/path/test.ts")
            .with_frecency(true, Some("/frecency".to_string()));
        assert!(s.is_suggested);
        assert_eq!(s.frecency_path, Some("/frecency".to_string()));
    }

    // =========================================================================
    // Category 22: Clipboard pin/unpin toggle — exact action details
    // Validates that pin/unpin toggle produces correct titles/descriptions.
    // =========================================================================

    #[test]
    fn cat22_unpinned_shows_pin() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_pin"));
        assert!(!actions.iter().any(|a| a.id == "clipboard_unpin"));
    }

    #[test]
    fn cat22_pinned_shows_unpin() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(!actions.iter().any(|a| a.id == "clipboard_pin"));
        assert!(actions.iter().any(|a| a.id == "clipboard_unpin"));
    }

