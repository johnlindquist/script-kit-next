    #[test]
    fn cat15_mixed_separators() {
        assert_eq!(to_deeplink_name("foo_bar baz-qux"), "foo-bar-baz-qux");
    }

    // ================================================================
    // Cat 16: Action::new pre-computes lowercase fields
    // ================================================================

    #[test]
    fn cat16_title_lower_cached() {
        let action = Action::new("id", "Hello World", None, ActionCategory::ScriptContext);
        assert_eq!(action.title_lower, "hello world");
    }

    #[test]
    fn cat16_description_lower_cached() {
        let action = Action::new(
            "id",
            "T",
            Some("Foo Bar".into()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.description_lower, Some("foo bar".into()));
    }

    #[test]
    fn cat16_description_none_lower_none() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext);
        assert!(action.description_lower.is_none());
    }

    #[test]
    fn cat16_shortcut_lower_none_initially() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext);
        assert!(action.shortcut_lower.is_none());
    }

    #[test]
    fn cat16_shortcut_lower_set_after_with_shortcut() {
        let action =
            Action::new("id", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
        assert_eq!(action.shortcut_lower.as_deref(), Some("⌘e"));
    }

    // ================================================================
    // Cat 17: ActionsDialog::format_shortcut_hint SDK-style shortcuts
    // ================================================================

    #[test]
    fn cat17_cmd_c() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+c"), "⌘C");
    }

    #[test]
    fn cat17_command_c() {
        assert_eq!(ActionsDialog::format_shortcut_hint("command+c"), "⌘C");
    }

    #[test]
    fn cat17_meta_c() {
        assert_eq!(ActionsDialog::format_shortcut_hint("meta+c"), "⌘C");
    }

    #[test]
    fn cat17_ctrl_alt_delete() {
        assert_eq!(
            ActionsDialog::format_shortcut_hint("ctrl+alt+delete"),
            "⌃⌥⌫"
        );
    }

    #[test]
    fn cat17_shift_enter() {
        assert_eq!(ActionsDialog::format_shortcut_hint("shift+enter"), "⇧↵");
    }

    #[test]
    fn cat17_option_space() {
        assert_eq!(ActionsDialog::format_shortcut_hint("option+space"), "⌥␣");
    }

    #[test]
    fn cat17_arrowup() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+arrowup"), "⌘↑");
    }

    // ================================================================
    // Cat 18: ActionsDialog::parse_shortcut_keycaps compound shortcuts
    // ================================================================

    #[test]
    fn cat18_cmd_enter_two_keycaps() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘↵");
        assert_eq!(caps, vec!["⌘", "↵"]);
    }

    #[test]
    fn cat18_cmd_shift_c_three_keycaps() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘⇧C");
        assert_eq!(caps, vec!["⌘", "⇧", "C"]);
    }

    #[test]
    fn cat18_single_letter_uppercased() {
        let caps = ActionsDialog::parse_shortcut_keycaps("a");
        assert_eq!(caps, vec!["A"]);
    }

    #[test]
    fn cat18_arrows() {
        let caps = ActionsDialog::parse_shortcut_keycaps("↑↓←→");
        assert_eq!(caps, vec!["↑", "↓", "←", "→"]);
    }

    #[test]
    fn cat18_empty_string() {
        let caps = ActionsDialog::parse_shortcut_keycaps("");
        assert!(caps.is_empty());
    }

    // ================================================================
    // Cat 19: ActionsDialog::score_action multi-field bonus stacking
    // ================================================================

    #[test]
    fn cat19_prefix_match_100() {
        let action = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
        assert_eq!(ActionsDialog::score_action(&action, "edit"), 100);
    }

    #[test]
    fn cat19_prefix_plus_description_115() {
        let action = Action::new(
            "id",
            "Edit Script",
            Some("Edit this script".into()),
            ActionCategory::ScriptContext,
        );
        // prefix(100) + description(15) = 115
        assert_eq!(ActionsDialog::score_action(&action, "edit"), 115);
    }

    #[test]
    fn cat19_contains_match_50() {
        let action = Action::new("id", "Script Editor", None, ActionCategory::ScriptContext);
        // "edit" is contained but not prefix in "script editor"
        assert_eq!(ActionsDialog::score_action(&action, "edit"), 50);
    }

    #[test]
    fn cat19_no_match_0() {
        let action = Action::new("id", "Run Script", None, ActionCategory::ScriptContext);
        assert_eq!(ActionsDialog::score_action(&action, "xyz"), 0);
    }

    #[test]
    fn cat19_shortcut_bonus_10() {
        let action = Action::new("id", "No Match Title", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘E");
        // "⌘e" matches shortcut_lower "⌘e" => +10
        assert_eq!(ActionsDialog::score_action(&action, "⌘e"), 10);
    }

    // ================================================================
    // Cat 20: ActionsDialog::fuzzy_match character ordering requirement
    // ================================================================

    #[test]
    fn cat20_correct_order_matches() {
        assert!(ActionsDialog::fuzzy_match("hello world", "hlo"));
    }

    #[test]
    fn cat20_wrong_order_no_match() {
        assert!(!ActionsDialog::fuzzy_match("hello world", "olh"));
    }

    #[test]
    fn cat20_exact_match() {
        assert!(ActionsDialog::fuzzy_match("abc", "abc"));
    }

    #[test]
    fn cat20_empty_needle_matches() {
        assert!(ActionsDialog::fuzzy_match("anything", ""));
    }

    #[test]
    fn cat20_empty_haystack_no_match() {
        assert!(!ActionsDialog::fuzzy_match("", "a"));
    }

    #[test]
    fn cat20_needle_longer_no_match() {
        assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
    }

    // ================================================================
    // Cat 21: build_grouped_items_static with no-section actions
    // ================================================================

    #[test]
    fn cat21_no_section_no_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext),
            Action::new("b", "B", None, ActionCategory::ScriptContext),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // No section set => no headers, just 2 items
        assert_eq!(grouped.len(), 2);
        assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(1)));
    }

    #[test]
    fn cat21_with_section_adds_header() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S"),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // 1 header + 2 items = 3
        assert_eq!(grouped.len(), 3);
        assert!(matches!(&grouped[0], GroupedActionItem::SectionHeader(s) if s == "S"));
    }

    #[test]
    fn cat21_separators_style_no_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        // No headers in Separators mode
        assert_eq!(grouped.len(), 2);
    }

    #[test]
    fn cat21_empty_input_empty_output() {
        let actions: Vec<Action> = vec![];
        let filtered: Vec<usize> = vec![];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(grouped.is_empty());
    }

    // ================================================================
    // Cat 22: coerce_action_selection alternating header-item pattern
    // ================================================================

    #[test]
    fn cat22_alternating_header_item_select_item() {
        let rows = vec![
            GroupedActionItem::SectionHeader("H1".into()),
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("H2".into()),
            GroupedActionItem::Item(1),
        ];
        // Index 0 is header => coerce to 1 (next item)
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn cat22_alternating_last_header_coerces_up() {
        let rows = vec![
            GroupedActionItem::SectionHeader("H1".into()),
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("H2".into()),
        ];
        // Index 2 is header, no items below => search up, find item at 1
        assert_eq!(coerce_action_selection(&rows, 2), Some(1));
    }

    #[test]
    fn cat22_item_at_index_stays() {
        let rows = vec![
            GroupedActionItem::SectionHeader("H".into()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(1));
    }

    #[test]
    fn cat22_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("H1".into()),
            GroupedActionItem::SectionHeader("H2".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn cat22_empty_returns_none() {
        let rows: Vec<GroupedActionItem> = vec![];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    // ================================================================
    // Cat 23: CommandBarConfig notes_style preset values
    // ================================================================

    #[test]
    fn cat23_notes_style_search_top() {
        let config = CommandBarConfig::notes_style();
        assert!(matches!(
            config.dialog_config.search_position,
            SearchPosition::Top
        ));
    }

    #[test]
    fn cat23_notes_style_separators() {
        let config = CommandBarConfig::notes_style();
        assert!(matches!(
            config.dialog_config.section_style,
            SectionStyle::Separators
        ));
    }

    #[test]
    fn cat23_notes_style_icons_enabled() {
        let config = CommandBarConfig::notes_style();
        assert!(config.dialog_config.show_icons);
    }

    #[test]
    fn cat23_notes_style_footer_enabled() {
        let config = CommandBarConfig::notes_style();
        assert!(config.dialog_config.show_footer);
    }

    #[test]
    fn cat23_notes_style_close_defaults_true() {
        let config = CommandBarConfig::notes_style();
        assert!(config.close_on_select);
        assert!(config.close_on_escape);
        assert!(config.close_on_click_outside);
    }

    // ================================================================
    // Cat 24: Clipboard unpin action title and description text
    // ================================================================

    #[test]
    fn cat24_unpin_title() {
        let entry = ClipboardEntryInfo {
            id: "e1".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let unpin = actions.iter().find(|a| a.id == "clipboard_unpin").unwrap();
        assert_eq!(unpin.title, "Unpin Entry");
    }

    #[test]
    fn cat24_unpin_description() {
        let entry = ClipboardEntryInfo {
            id: "e1".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let unpin = actions.iter().find(|a| a.id == "clipboard_unpin").unwrap();
        assert!(unpin.description.as_ref().unwrap().contains("pin"));
    }

    #[test]
    fn cat24_pin_title() {
        let entry = ClipboardEntryInfo {
            id: "e1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let pin = actions.iter().find(|a| a.id == "clipboard_pin").unwrap();
        assert_eq!(pin.title, "Pin Entry");
    }

    #[test]
    fn cat24_pin_unpin_same_shortcut() {
        let pinned_entry = ClipboardEntryInfo {
            id: "e1".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let unpinned_entry = ClipboardEntryInfo {
            id: "e2".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let pinned_actions = get_clipboard_history_context_actions(&pinned_entry);
        let unpinned_actions = get_clipboard_history_context_actions(&unpinned_entry);
        let unpin = pinned_actions
            .iter()
            .find(|a| a.id == "clipboard_unpin")
            .unwrap();
        let pin = unpinned_actions
            .iter()
            .find(|a| a.id == "clipboard_pin")
            .unwrap();
        assert_eq!(unpin.shortcut, pin.shortcut);
        assert_eq!(pin.shortcut.as_deref(), Some("⇧⌘P"));
    }

    // ================================================================
    // Cat 25: New chat actions mixed section sizes
    // ================================================================

