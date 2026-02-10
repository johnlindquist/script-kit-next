    #[test]
    fn cat13_score_empty_query_returns_prefix_match() {
        let action = Action::new(
            "test",
            "Edit Script",
            Some("Open in editor".to_string()),
            ActionCategory::ScriptContext,
        );
        // Empty string is a prefix of everything
        let score = ActionsDialog::score_action(&action, "");
        assert!(
            score >= 100,
            "empty query is prefix of any title: {}",
            score
        );
    }

    #[test]
    fn cat13_score_no_match_returns_zero() {
        let action = Action::new("test", "Edit Script", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "zzzzz");
        assert_eq!(score, 0);
    }

    #[test]
    fn cat13_score_prefix_beats_contains() {
        let action = Action::new("test", "Edit Script", None, ActionCategory::ScriptContext);
        let prefix_score = ActionsDialog::score_action(&action, "edit");
        let contains_score = ActionsDialog::score_action(&action, "script");
        assert!(
            prefix_score > contains_score,
            "prefix {} > contains {}",
            prefix_score,
            contains_score
        );
    }

    #[test]
    fn cat13_score_description_bonus() {
        let action = Action::new(
            "test",
            "Open File",
            Some("Open in the default editor".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "editor");
        assert!(
            score >= 15,
            "description match gives at least 15: {}",
            score
        );
    }

    #[test]
    fn cat13_score_shortcut_bonus() {
        let action =
            Action::new("test", "Submit", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
        let score = ActionsDialog::score_action(&action, "⌘e");
        assert!(score >= 10, "shortcut match gives at least 10: {}", score);
    }

    // =========================================================================
    // cat14: fuzzy_match case sensitivity
    // =========================================================================

    #[test]
    fn cat14_fuzzy_exact_match() {
        assert!(ActionsDialog::fuzzy_match("hello", "hello"));
    }

    #[test]
    fn cat14_fuzzy_subsequence() {
        assert!(ActionsDialog::fuzzy_match("hello world", "hwd"));
    }

    #[test]
    fn cat14_fuzzy_no_match() {
        assert!(!ActionsDialog::fuzzy_match("hello", "xyz"));
    }

    #[test]
    fn cat14_fuzzy_empty_needle() {
        assert!(ActionsDialog::fuzzy_match("hello", ""));
    }

    #[test]
    fn cat14_fuzzy_empty_haystack() {
        assert!(!ActionsDialog::fuzzy_match("", "a"));
    }

    #[test]
    fn cat14_fuzzy_both_empty() {
        assert!(ActionsDialog::fuzzy_match("", ""));
    }

    #[test]
    fn cat14_fuzzy_needle_longer() {
        assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
    }

    // =========================================================================
    // cat15: build_grouped_items_static single-item input
    // =========================================================================

    #[test]
    fn cat15_grouped_single_item_headers() {
        let actions = vec![
            Action::new("a", "Action A", None, ActionCategory::ScriptContext).with_section("Sec"),
        ];
        let filtered = vec![0usize];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert_eq!(grouped.len(), 2, "1 header + 1 item");
        assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(_)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(0)));
    }

    #[test]
    fn cat15_grouped_single_item_separators() {
        let actions = vec![Action::new(
            "a",
            "Action A",
            None,
            ActionCategory::ScriptContext,
        )];
        let filtered = vec![0usize];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        assert_eq!(grouped.len(), 1, "no header for separators");
        assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
    }

    #[test]
    fn cat15_grouped_single_item_none_style() {
        let actions = vec![Action::new(
            "a",
            "Action A",
            None,
            ActionCategory::ScriptContext,
        )];
        let filtered = vec![0usize];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        assert_eq!(grouped.len(), 1);
    }

    #[test]
    fn cat15_grouped_empty_returns_empty() {
        let actions: Vec<Action> = vec![];
        let filtered: Vec<usize> = vec![];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(grouped.is_empty());
    }

    #[test]
    fn cat15_grouped_same_section_no_duplicate_header() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S"),
        ];
        let filtered = vec![0usize, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 1, "same section = 1 header");
    }

    // =========================================================================
    // cat16: coerce_action_selection single-item input
    // =========================================================================

    #[test]
    fn cat16_coerce_single_item() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    }

    #[test]
    fn cat16_coerce_single_header() {
        let rows = vec![GroupedActionItem::SectionHeader("S".into())];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn cat16_coerce_header_then_item() {
        let rows = vec![
            GroupedActionItem::SectionHeader("S".into()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn cat16_coerce_item_then_header() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("S".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(0));
    }

    #[test]
    fn cat16_coerce_empty() {
        let rows: Vec<GroupedActionItem> = vec![];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn cat16_coerce_out_of_bounds_clamps() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 999), Some(0));
    }

    // =========================================================================
    // cat17: CommandBarConfig close flag independence
    // =========================================================================

    #[test]
    fn cat17_default_all_close_true() {
        let config = CommandBarConfig::default();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    #[test]
    fn cat17_ai_style_close_flags() {
        let config = CommandBarConfig::ai_style();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    #[test]
    fn cat17_main_menu_close_flags() {
        let config = CommandBarConfig::main_menu_style();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    #[test]
    fn cat17_no_search_close_flags() {
        let config = CommandBarConfig::no_search();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    // =========================================================================
    // cat18: Action::new description_lower None when description is None
    // =========================================================================

    #[test]
    fn cat18_action_no_description_lower_none() {
        let action = Action::new("id", "Title", None, ActionCategory::ScriptContext);
        assert!(action.description_lower.is_none());
    }

    #[test]
    fn cat18_action_with_description_lower_set() {
        let action = Action::new(
            "id",
            "Title",
            Some("Hello World".to_string()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.description_lower.as_deref(), Some("hello world"));
    }

    #[test]
    fn cat18_action_title_lower_cached() {
        let action = Action::new("id", "My Title", None, ActionCategory::ScriptContext);
        assert_eq!(action.title_lower, "my title");
    }

    #[test]
    fn cat18_action_shortcut_lower_none_initially() {
        let action = Action::new("id", "Title", None, ActionCategory::ScriptContext);
        assert!(action.shortcut_lower.is_none());
    }

    #[test]
    fn cat18_action_shortcut_lower_set_after_with_shortcut() {
        let action =
            Action::new("id", "Title", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
        assert_eq!(action.shortcut_lower.as_deref(), Some("⌘e"));
    }

    // =========================================================================
    // cat19: Action builder chain ordering (icon, section, shortcut)
    // =========================================================================

    #[test]
    fn cat19_icon_then_section() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_icon(IconName::Star)
            .with_section("Sec");
        assert_eq!(action.icon, Some(IconName::Star));
        assert_eq!(action.section.as_deref(), Some("Sec"));
    }

    #[test]
    fn cat19_section_then_icon() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_section("Sec")
            .with_icon(IconName::Star);
        assert_eq!(action.icon, Some(IconName::Star));
        assert_eq!(action.section.as_deref(), Some("Sec"));
    }

    #[test]
    fn cat19_shortcut_then_icon_preserves_shortcut() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘K")
            .with_icon(IconName::Settings);
        assert_eq!(action.shortcut.as_deref(), Some("⌘K"));
        assert_eq!(action.icon, Some(IconName::Settings));
    }

    #[test]
    fn cat19_full_chain() {
        let action = Action::new(
            "id",
            "T",
            Some("desc".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘X")
        .with_icon(IconName::Trash)
        .with_section("Danger");
        assert_eq!(action.shortcut.as_deref(), Some("⌘X"));
        assert_eq!(action.icon, Some(IconName::Trash));
        assert_eq!(action.section.as_deref(), Some("Danger"));
        assert_eq!(action.description.as_deref(), Some("desc"));
    }

    // =========================================================================
    // cat20: ScriptInfo with_action_verb preserves defaults
    // =========================================================================

    #[test]
    fn cat20_with_action_verb_preserves_not_scriptlet() {
        let info = ScriptInfo::with_action_verb("App", "/app", false, "Launch");
        assert!(!info.is_scriptlet);
        assert!(!info.is_agent);
        assert!(info.shortcut.is_none());
        assert!(info.alias.is_none());
        assert!(!info.is_suggested);
    }

    #[test]
    fn cat20_with_action_verb_sets_verb() {
        let info = ScriptInfo::with_action_verb("Win", "/win", false, "Switch to");
        assert_eq!(info.action_verb, "Switch to");
    }

    #[test]
    fn cat20_with_action_verb_name_and_path() {
        let info =
            ScriptInfo::with_action_verb("Safari", "/Applications/Safari.app", false, "Launch");
        assert_eq!(info.name, "Safari");
        assert_eq!(info.path, "/Applications/Safari.app");
    }

    #[test]
    fn cat20_with_action_verb_is_script_flag() {
        let info_true = ScriptInfo::with_action_verb("S", "/s", true, "Run");
        assert!(info_true.is_script);
        let info_false = ScriptInfo::with_action_verb("S", "/s", false, "Run");
        assert!(!info_false.is_script);
    }

    // =========================================================================
    // cat21: Script context agent flag produces edit with "Edit Agent" title
    // =========================================================================

    #[test]
    fn cat21_agent_flag_produces_edit_agent() {
        let mut script = ScriptInfo::new("Bot", "/bot.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert!(edit.title.contains("Agent"));
    }

    #[test]
    fn cat21_agent_has_copy_content() {
        let mut script = ScriptInfo::new("Bot", "/bot.md");
        script.is_script = false;
        script.is_agent = true;
        let ids = action_ids(&get_script_context_actions(&script));
        assert!(ids.contains(&"copy_content".to_string()));
    }

    #[test]
    fn cat21_agent_edit_shortcut() {
        let mut script = ScriptInfo::new("Bot", "/bot.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.shortcut.as_deref(), Some("⌘E"));
    }

    #[test]
    fn cat21_agent_reveal_shortcut() {
        let mut script = ScriptInfo::new("Bot", "/bot.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
        assert_eq!(reveal.shortcut.as_deref(), Some("⌘⇧F"));
    }

    // =========================================================================
    // cat22: Cross-context shortcut format uses Unicode symbols
    // =========================================================================

    #[test]
    fn cat22_script_shortcuts_use_unicode() {
        let script = ScriptInfo::new("test", "/p/test.ts");
        let actions = get_script_context_actions(&script);
        for action in &actions {
            if let Some(ref s) = action.shortcut {
                // All shortcuts should contain Unicode symbols, not "cmd" / "shift" etc.
                assert!(
                    !s.contains("cmd") && !s.contains("shift") && !s.contains("ctrl"),
                    "Shortcut '{}' on action '{}' should use Unicode symbols",
                    s,
                    action.id
                );
            }
        }
    }

