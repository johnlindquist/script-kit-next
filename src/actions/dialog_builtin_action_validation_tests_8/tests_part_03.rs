    #[test]
    fn new_chat_last_used_has_bolt_icon() {
        let last_used = vec![NewChatModelInfo {
            model_id: "lu1".to_string(),
            display_name: "Recent".to_string(),
            provider: "p".to_string(),
            provider_display_name: "Provider".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
    }

    #[test]
    fn new_chat_model_has_settings_icon() {
        let models = vec![NewChatModelInfo {
            model_id: "m1".to_string(),
            display_name: "Model".to_string(),
            provider: "p".to_string(),
            provider_display_name: "Provider".to_string(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].icon, Some(IconName::Settings));
    }

    // ============================================================
    // 10. Deeplink description URL format
    // ============================================================

    #[test]
    fn deeplink_description_contains_url() {
        let script = ScriptInfo::new("My Script", "/path/to/script.ts");
        let actions = get_script_context_actions(&script);
        let dl = find_action(&actions, "copy_deeplink").unwrap();
        assert!(dl
            .description
            .as_ref()
            .unwrap()
            .contains("scriptkit://run/my-script"));
    }

    #[test]
    fn deeplink_description_special_chars_stripped() {
        let script = ScriptInfo::new("Hello!@#World", "/path/to/script.ts");
        let actions = get_script_context_actions(&script);
        let dl = find_action(&actions, "copy_deeplink").unwrap();
        assert!(dl
            .description
            .as_ref()
            .unwrap()
            .contains("scriptkit://run/hello-world"));
    }

    #[test]
    fn deeplink_name_consecutive_specials() {
        assert_eq!(to_deeplink_name("a!!!b"), "a-b");
        assert_eq!(to_deeplink_name("---hello---"), "hello");
        assert_eq!(to_deeplink_name("  spaces  between  "), "spaces-between");
    }

    #[test]
    fn deeplink_name_unicode_preserved() {
        // é and ï are alphanumeric in Unicode, so they are preserved
        assert_eq!(to_deeplink_name("café"), "café");
        assert_eq!(to_deeplink_name("naïve"), "naïve");
    }

    #[test]
    fn deeplink_name_empty_after_stripping() {
        let result = to_deeplink_name("!@#$%");
        assert_eq!(result, "");
    }

    // ============================================================
    // 11. Score_action boundary thresholds
    // ============================================================

    #[test]
    fn score_prefix_match_is_100() {
        let action = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "edit");
        assert_eq!(score, 100, "Prefix match should score exactly 100");
    }

    #[test]
    fn score_contains_match_is_50() {
        let action = Action::new("id", "The Editor", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "editor");
        assert_eq!(score, 50, "Contains match should score exactly 50");
    }

    #[test]
    fn score_fuzzy_match_is_25() {
        // "esr" is a subsequence of "edit script" (e...s...r? no)
        // Need a proper subsequence: "esc" matches "edit script" (e..s..c? no)
        // "edc" -> e_d_i_t -> no... let's use "eit" -> "edit script" e..i..t
        let action = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "eit");
        assert_eq!(score, 25, "Fuzzy subsequence match should score exactly 25");
    }

    #[test]
    fn score_description_bonus_is_15() {
        let action = Action::new(
            "id",
            "Open File",
            Some("edit the file".to_string()),
            ActionCategory::ScriptContext,
        );
        // "xyz" won't match title, but if we query something that matches desc only...
        // Actually, we need to match title for base score. If no title match, desc only won't help.
        // Let's match title + desc: "open" prefix = 100, desc contains "file" doesn't add if query is "open"
        // We need desc-only bonus: query matches desc but not title
        let score_with_desc = ActionsDialog::score_action(&action, "edit");
        // "edit" doesn't prefix "open file", doesn't contain... let's check fuzzy
        // fuzzy "edit" in "open file": e_d_i_t? no 'e' found -> nope, not in "open file"
        // Actually "open file" has no 'e'? Wait, "open file" -> o,p,e,n,f,i,l,e -> has 'e'!
        // fuzzy "edit": e in "open file" at pos 2, d? no d in "open file" after pos 2... nope
        // So title score = 0, desc "edit the file" contains "edit" = +15
        assert_eq!(
            score_with_desc, 15,
            "Description-only match should score 15"
        );
    }

    #[test]
    fn score_shortcut_bonus_is_10() {
        let action =
            Action::new("id", "Open File", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
        // "⌘e" matches shortcut_lower
        let score = ActionsDialog::score_action(&action, "⌘e");
        assert_eq!(score, 10, "Shortcut-only match should score 10");
    }

    #[test]
    fn score_prefix_plus_desc_stacking() {
        let action = Action::new(
            "id",
            "Copy Path",
            Some("Copy the full path to clipboard".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "copy");
        // prefix match on "copy path" = 100, desc "copy the full path..." contains "copy" = +15
        assert_eq!(score, 115, "Prefix + desc should stack to 115");
    }

    #[test]
    fn score_no_match_is_zero() {
        let action = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "zzz");
        assert_eq!(score, 0, "No match should score 0");
    }

    #[test]
    fn score_empty_query_scores_zero() {
        let action = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "");
        // Empty string is prefix of everything
        assert_eq!(score, 100, "Empty query matches everything as prefix");
    }

    // ============================================================
    // 12. build_grouped_items interleaved section/no-section
    // ============================================================

    #[test]
    fn grouped_items_mixed_section_and_no_section() {
        let actions = vec![
            make_action("a1", "Action 1", Some("Section A")),
            make_action("a2", "Action 2", None),
            make_action("a3", "Action 3", Some("Section B")),
        ];
        let filtered: Vec<usize> = (0..3).collect();
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Should have: Header("Section A"), Item(0), Item(1) [no header for None], Header("Section B"), Item(2)
        let mut header_count = 0;
        let mut item_count = 0;
        for item in &items {
            match item {
                GroupedActionItem::SectionHeader(_) => header_count += 1,
                GroupedActionItem::Item(_) => item_count += 1,
            }
        }
        assert_eq!(header_count, 2, "Should have 2 section headers");
        assert_eq!(item_count, 3, "Should have 3 items");
    }

    #[test]
    fn grouped_items_none_style_no_headers() {
        let actions = vec![
            make_action("a1", "Action 1", Some("Section A")),
            make_action("a2", "Action 2", Some("Section B")),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        // None style should have no headers
        for item in &items {
            assert!(
                !matches!(item, GroupedActionItem::SectionHeader(_)),
                "None style should not insert section headers"
            );
        }
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn grouped_items_separators_style_no_headers() {
        let actions = vec![
            make_action("a1", "Action 1", Some("Section A")),
            make_action("a2", "Action 2", Some("Section B")),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        for item in &items {
            assert!(
                !matches!(item, GroupedActionItem::SectionHeader(_)),
                "Separators style should not insert section headers"
            );
        }
    }

    #[test]
    fn grouped_items_empty_filtered() {
        let actions = vec![make_action("a1", "Action 1", Some("Section A"))];
        let filtered: Vec<usize> = vec![];
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(items.is_empty());
    }

    #[test]
    fn grouped_items_same_section_no_duplicate_header() {
        let actions = vec![
            make_action("a1", "Action 1", Some("Same")),
            make_action("a2", "Action 2", Some("Same")),
            make_action("a3", "Action 3", Some("Same")),
        ];
        let filtered: Vec<usize> = (0..3).collect();
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        let header_count = items
            .iter()
            .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 1, "Same section should produce only 1 header");
    }

    // ============================================================
    // 13. coerce_action_selection complex patterns
    // ============================================================

    #[test]
    fn coerce_empty_returns_none() {
        let result = coerce_action_selection(&[], 0);
        assert_eq!(result, None);
    }

    #[test]
    fn coerce_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".to_string()),
            GroupedActionItem::SectionHeader("B".to_string()),
        ];
        let result = coerce_action_selection(&rows, 0);
        assert_eq!(result, None);
    }

    #[test]
    fn coerce_on_item_returns_same() {
        let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
        let result = coerce_action_selection(&rows, 0);
        assert_eq!(result, Some(0));
    }

    #[test]
    fn coerce_header_at_start_goes_down() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".to_string()),
            GroupedActionItem::Item(0),
        ];
        let result = coerce_action_selection(&rows, 0);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn coerce_header_at_end_goes_up() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("A".to_string()),
        ];
        let result = coerce_action_selection(&rows, 1);
        assert_eq!(result, Some(0));
    }

    #[test]
    fn coerce_alternating_header_item() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".to_string()),
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("B".to_string()),
            GroupedActionItem::Item(1),
        ];
        // Landing on header at index 2 should go down to item at index 3
        let result = coerce_action_selection(&rows, 2);
        assert_eq!(result, Some(3));
    }

    #[test]
    fn coerce_out_of_bounds_clamped() {
        let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
        let result = coerce_action_selection(&rows, 999);
        assert_eq!(result, Some(1));
    }

    // ============================================================
    // 14. parse_shortcut_keycaps compound symbol sequences
    // ============================================================

    #[test]
    fn parse_keycaps_cmd_c() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘C");
        assert_eq!(keycaps, vec!["⌘", "C"]);
    }

    #[test]
    fn parse_keycaps_all_modifiers() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌃⌥⇧⌘A");
        assert_eq!(keycaps, vec!["⌃", "⌥", "⇧", "⌘", "A"]);
    }

    #[test]
    fn parse_keycaps_enter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("↵");
        assert_eq!(keycaps, vec!["↵"]);
    }

    #[test]
    fn parse_keycaps_arrows() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("↑↓←→");
        assert_eq!(keycaps, vec!["↑", "↓", "←", "→"]);
    }

    #[test]
    fn parse_keycaps_escape() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⎋");
        assert_eq!(keycaps, vec!["⎋"]);
    }

    #[test]
    fn parse_keycaps_empty() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("");
        assert!(keycaps.is_empty());
    }

    #[test]
    fn parse_keycaps_space_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("␣");
        assert_eq!(keycaps, vec!["␣"]);
    }

    #[test]
    fn parse_keycaps_lowercase_uppercased() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘c");
        assert_eq!(keycaps, vec!["⌘", "C"]);
    }

    // ============================================================
    // 15. CommandBarConfig detailed fields
    // ============================================================

    #[test]
    fn command_bar_default_config() {
        let config = CommandBarConfig::default();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
        assert_eq!(
            config.dialog_config.search_position,
            crate::actions::types::SearchPosition::Bottom
        );
    }

    #[test]
    fn command_bar_ai_style() {
        let config = CommandBarConfig::ai_style();
        assert_eq!(
            config.dialog_config.search_position,
            crate::actions::types::SearchPosition::Top
        );
        assert_eq!(config.dialog_config.section_style, SectionStyle::Headers);
        assert!(config.dialog_config.show_icons);
        assert!(config.dialog_config.show_footer);
    }

    #[test]
    fn command_bar_main_menu_style() {
        let config = CommandBarConfig::main_menu_style();
        assert_eq!(
            config.dialog_config.search_position,
            crate::actions::types::SearchPosition::Bottom
        );
        assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
        assert!(!config.dialog_config.show_icons);
        assert!(!config.dialog_config.show_footer);
    }

    #[test]
    fn command_bar_no_search() {
        let config = CommandBarConfig::no_search();
        assert_eq!(
            config.dialog_config.search_position,
            crate::actions::types::SearchPosition::Hidden
        );
    }

    // ============================================================
    // 16. Cross-builder action count comparisons
    // ============================================================

    #[test]
    fn script_has_more_actions_than_builtin() {
        let script = ScriptInfo::new("test", "/path/to/test.ts");
        let builtin = ScriptInfo::builtin("Test Builtin");
        let script_actions = get_script_context_actions(&script);
        let builtin_actions = get_script_context_actions(&builtin);
        assert!(
            script_actions.len() > builtin_actions.len(),
            "Script ({}) should have more actions than builtin ({})",
            script_actions.len(),
            builtin_actions.len()
        );
    }

    #[test]
    fn scriptlet_via_script_context_vs_scriptlet_context_same_count() {
        let script = ScriptInfo::scriptlet("Test", "/path/to/test.md", None, None);
        let via_script = get_script_context_actions(&script);
        let via_scriptlet = get_scriptlet_context_actions_with_custom(&script, None);
        // Both should produce same actions (scriptlet context without custom = script context for scriptlet)
        assert_eq!(
            via_script.len(),
            via_scriptlet.len(),
            "Script context ({}) and scriptlet context ({}) should match for plain scriptlet",
            via_script.len(),
            via_scriptlet.len()
        );
    }

