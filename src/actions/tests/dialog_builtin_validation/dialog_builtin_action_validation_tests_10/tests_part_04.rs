    #[test]
    fn notes_cmd_bar_duplicate_note_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "duplicate_note").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘D"));
    }

    #[test]
    fn notes_cmd_bar_browse_notes_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "browse_notes").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘P"));
    }

    #[test]
    fn notes_cmd_bar_find_in_note_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "find_in_note").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘F"));
    }

    #[test]
    fn notes_cmd_bar_copy_note_as_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "copy_note_as").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⇧⌘C"));
    }

    #[test]
    fn notes_cmd_bar_export_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "export").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⇧⌘E"));
    }

    // ========================================
    // 20. Note switcher complex scenarios (5 tests)
    // ========================================

    #[test]
    fn note_switcher_10_notes_all_have_actions() {
        let notes: Vec<_> = (0..10)
            .map(|i| {
                make_note(
                    &format!("id-{}", i),
                    &format!("Note {}", i),
                    100,
                    false,
                    false,
                    "",
                    "",
                )
            })
            .collect();
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions.len(), 10);
    }

    #[test]
    fn note_switcher_pinned_section_label() {
        let note = make_note("1", "Pinned Note", 50, false, true, "", "");
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
    }

    #[test]
    fn note_switcher_recent_section_label() {
        let note = make_note("1", "Regular Note", 50, false, false, "", "");
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn note_switcher_mixed_pinned_and_recent_sections() {
        let notes = vec![
            make_note("1", "A", 10, false, true, "", ""),
            make_note("2", "B", 20, false, false, "", ""),
            make_note("3", "C", 30, false, true, "", ""),
        ];
        let actions = get_note_switcher_actions(&notes);
        let sections: Vec<_> = actions.iter().map(|a| a.section.as_deref()).collect();
        assert_eq!(
            sections,
            vec![Some("Pinned"), Some("Recent"), Some("Pinned")]
        );
    }

    #[test]
    fn note_switcher_current_note_has_bullet_prefix() {
        let note = make_note("1", "Current Note", 50, true, false, "", "");
        let actions = get_note_switcher_actions(&[note]);
        assert!(actions[0].title.starts_with("• "));
    }

    // ========================================
    // 21. build_grouped_items section header content (5 tests)
    // ========================================

    #[test]
    fn grouped_items_header_text_matches_section_name() {
        let actions = vec![
            make_action("a1", "Action 1", Some("Section A")),
            make_action("a2", "Action 2", Some("Section B")),
        ];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        match &grouped[0] {
            GroupedActionItem::SectionHeader(s) => assert_eq!(s, "Section A"),
            _ => panic!("Expected SectionHeader"),
        }
        match &grouped[2] {
            GroupedActionItem::SectionHeader(s) => assert_eq!(s, "Section B"),
            _ => panic!("Expected SectionHeader"),
        }
    }

    #[test]
    fn grouped_items_headers_count_matches_unique_sections() {
        let actions = vec![
            make_action("a1", "A1", Some("S1")),
            make_action("a2", "A2", Some("S1")),
            make_action("a3", "A3", Some("S2")),
            make_action("a4", "A4", Some("S3")),
        ];
        let filtered = vec![0, 1, 2, 3];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 3);
    }

    #[test]
    fn grouped_items_no_section_no_header() {
        let actions = vec![make_action("a1", "A1", None), make_action("a2", "A2", None)];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 0);
    }

    #[test]
    fn grouped_items_headers_precede_their_items() {
        let actions = vec![
            make_action("a1", "A1", Some("First")),
            make_action("a2", "A2", Some("Second")),
        ];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Expected: [Header("First"), Item(0), Header("Second"), Item(1)]
        assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(_)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[2], GroupedActionItem::SectionHeader(_)));
        assert!(matches!(grouped[3], GroupedActionItem::Item(1)));
    }

    #[test]
    fn grouped_items_separators_style_no_headers() {
        let actions = vec![
            make_action("a1", "A1", Some("S1")),
            make_action("a2", "A2", Some("S2")),
        ];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 0);
        assert_eq!(grouped.len(), 2); // Just Items
    }

    // ========================================
    // 22. coerce_action_selection edge cases (5 tests)
    // ========================================

    #[test]
    fn coerce_empty_returns_none() {
        assert_eq!(coerce_action_selection(&[], 0), None);
    }

    #[test]
    fn coerce_on_item_returns_same() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    }

    #[test]
    fn coerce_header_followed_by_item_goes_down() {
        let rows = vec![
            GroupedActionItem::SectionHeader("S".to_string()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn coerce_item_then_header_at_end() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("S".to_string()),
        ];
        // Index 1 is header, search down finds nothing, search up finds Item at 0
        assert_eq!(coerce_action_selection(&rows, 1), Some(0));
    }

    #[test]
    fn coerce_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".to_string()),
            GroupedActionItem::SectionHeader("B".to_string()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    // ========================================
    // 23. Score_action with cached lowercase (5 tests)
    // ========================================

    #[test]
    fn score_prefix_match_100() {
        let a = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
        assert!(ActionsDialog::score_action(&a, "edit") >= 100);
    }

    #[test]
    fn score_contains_match_50() {
        let a = Action::new("id", "Quick Edit", None, ActionCategory::ScriptContext);
        let s = ActionsDialog::score_action(&a, "edit");
        assert!((50..100).contains(&s));
    }

    #[test]
    fn score_fuzzy_match_25() {
        // "et" subsequence in "edit" => e...t
        let a = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
        let s = ActionsDialog::score_action(&a, "eit");
        // 'e','i','t' are subsequence of "edit script"
        assert!(s >= 25);
    }

    #[test]
    fn score_description_bonus_15() {
        let a = Action::new(
            "id",
            "Open File",
            Some("Launch editor".to_string()),
            ActionCategory::ScriptContext,
        );
        let s = ActionsDialog::score_action(&a, "editor");
        assert!(s >= 15);
    }

    #[test]
    fn score_no_match_zero() {
        let a = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
        assert_eq!(ActionsDialog::score_action(&a, "zzzzz"), 0);
    }

    // ========================================
    // 24. fuzzy_match edge cases (5 tests)
    // ========================================

    #[test]
    fn fuzzy_empty_needle_always_matches() {
        assert!(ActionsDialog::fuzzy_match("anything", ""));
    }

    #[test]
    fn fuzzy_empty_haystack_nonempty_needle_fails() {
        assert!(!ActionsDialog::fuzzy_match("", "a"));
    }

    #[test]
    fn fuzzy_exact_match() {
        assert!(ActionsDialog::fuzzy_match("hello", "hello"));
    }

    #[test]
    fn fuzzy_subsequence_match() {
        assert!(ActionsDialog::fuzzy_match("abcdef", "ace"));
    }

    #[test]
    fn fuzzy_no_match() {
        assert!(!ActionsDialog::fuzzy_match("abc", "z"));
    }

    // ========================================
    // 25. parse_shortcut_keycaps (6 tests)
    // ========================================

    #[test]
    fn keycaps_cmd_c() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘C");
        assert_eq!(caps, vec!["⌘", "C"]);
    }

    #[test]
    fn keycaps_cmd_shift_enter() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘⇧↵");
        assert_eq!(caps, vec!["⌘", "⇧", "↵"]);
    }

    #[test]
    fn keycaps_ctrl_x() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌃X");
        assert_eq!(caps, vec!["⌃", "X"]);
    }

    #[test]
    fn keycaps_space() {
        let caps = ActionsDialog::parse_shortcut_keycaps("␣");
        assert_eq!(caps, vec!["␣"]);
    }

    #[test]
    fn keycaps_arrows() {
        let caps = ActionsDialog::parse_shortcut_keycaps("↑↓←→");
        assert_eq!(caps, vec!["↑", "↓", "←", "→"]);
    }

    #[test]
    fn keycaps_lowercase_uppercased() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘c");
        assert_eq!(caps, vec!["⌘", "C"]);
    }

    // ========================================
    // 26. to_deeplink_name additional edge cases (4 tests)
    // ========================================

    #[test]
    fn deeplink_cjk_characters_preserved() {
        let result = to_deeplink_name("测试脚本");
        assert_eq!(result, "测试脚本");
    }

    #[test]
    fn deeplink_mixed_ascii_unicode() {
        let result = to_deeplink_name("My 脚本 Script");
        assert_eq!(result, "my-脚本-script");
    }

    #[test]
    fn deeplink_accented_preserved() {
        let result = to_deeplink_name("café résumé");
        assert_eq!(result, "café-résumé");
    }

    #[test]
    fn deeplink_emoji_stripped() {
        // Emoji are not alphanumeric, so they become hyphens
        let result = to_deeplink_name("Test 🚀 Script");
        // 🚀 becomes -, collapses with surrounding hyphens
        assert_eq!(result, "test-script");
    }

    // ========================================
    // 27. Clipboard exact shortcut values (6 tests)
    // ========================================

    #[test]
    fn clipboard_share_shortcut() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_share").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⇧⌘E"));
    }

    #[test]
    fn clipboard_attach_to_ai_shortcut() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_attach_to_ai").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌃⌘A"));
    }

    #[test]
    fn clipboard_pin_shortcut() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_pin").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⇧⌘P"));
    }

    #[test]
    fn clipboard_unpin_shortcut_same_as_pin() {
        let entry = make_clipboard_entry(ContentType::Text, true, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_unpin").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⇧⌘P"));
    }

    #[test]
    fn clipboard_save_snippet_shortcut() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_save_snippet").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⇧⌘S"));
    }

    #[test]
    fn clipboard_save_file_shortcut() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_save_file").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌥⇧⌘S"));
    }

    // ========================================
    // 28. Notes command bar section labels (4 tests)
    // ========================================
