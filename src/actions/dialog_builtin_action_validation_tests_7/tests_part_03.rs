    #[test]
    fn ai_command_bar_all_have_sections() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                action.section.is_some(),
                "AI action '{}' should have section",
                action.id
            );
        }
    }

    #[test]
    fn ai_command_bar_section_order() {
        let actions = get_ai_command_bar_actions();
        let sections: Vec<&str> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        // Verify order: Response before Actions before Attachments before Settings
        let first_response = sections.iter().position(|&s| s == "Response").unwrap();
        let first_actions = sections.iter().position(|&s| s == "Actions").unwrap();
        let first_attachments = sections.iter().position(|&s| s == "Attachments").unwrap();
        let first_settings = sections.iter().position(|&s| s == "Settings").unwrap();
        assert!(first_response < first_actions);
        assert!(first_actions < first_attachments);
        assert!(first_attachments < first_settings);
    }

    // ============================================================
    // 12. Notes command bar shortcut uniqueness
    // ============================================================

    #[test]
    fn notes_command_bar_shortcuts_unique() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let shortcuts: Vec<&str> = actions
            .iter()
            .filter_map(|a| a.shortcut.as_deref())
            .collect();
        let unique: HashSet<&str> = shortcuts.iter().copied().collect();
        assert_eq!(
            shortcuts.len(),
            unique.len(),
            "Notes command bar shortcuts should be unique: {:?}",
            shortcuts
        );
    }

    // ============================================================
    // 13. Path context action ordering
    // ============================================================

    #[test]
    fn path_dir_primary_first() {
        let path = PathInfo {
            path: "/test/dir".to_string(),
            name: "dir".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path);
        assert_eq!(actions[0].id, "open_directory");
    }

    #[test]
    fn path_file_primary_first() {
        let path = PathInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        assert_eq!(actions[0].id, "select_file");
    }

    #[test]
    fn path_trash_last() {
        let path = PathInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        assert_eq!(
            actions.last().unwrap().id,
            "move_to_trash",
            "Trash should be last action"
        );
    }

    #[test]
    fn path_dir_trash_says_folder() {
        let path = PathInfo {
            path: "/test/dir".to_string(),
            name: "dir".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path);
        let trash = find_action(&actions, "move_to_trash").unwrap();
        assert!(
            trash.description.as_ref().unwrap().contains("folder"),
            "Dir trash should say 'folder'"
        );
    }

    #[test]
    fn path_file_trash_says_file() {
        let path = PathInfo {
            path: "/test/f.txt".to_string(),
            name: "f.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        let trash = find_action(&actions, "move_to_trash").unwrap();
        assert!(
            trash.description.as_ref().unwrap().contains("file"),
            "File trash should say 'file'"
        );
    }

    // ============================================================
    // 14. Clipboard action shortcut format
    // ============================================================

    #[test]
    fn clipboard_all_shortcuts_use_symbols() {
        let entry = make_text_entry(false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        for action in &actions {
            if let Some(ref shortcut) = action.shortcut {
                // Should not contain text like "cmd", "ctrl" etc.
                assert!(
                    !shortcut.contains("cmd"),
                    "Shortcut '{}' should use symbols not text",
                    shortcut
                );
                assert!(
                    !shortcut.contains("shift"),
                    "Shortcut '{}' should use symbols not text",
                    shortcut
                );
            }
        }
    }

    // ============================================================
    // 15. score_action with whitespace-only query
    // ============================================================

    #[test]
    fn score_whitespace_query() {
        let action = Action::new(
            "test",
            "Test Action With Spaces",
            Some("Description with spaces".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, " ");
        // " " is a contains match on the title " " appears after words
        assert!(score > 0, "Space should match title containing spaces");
    }

    // ============================================================
    // 16. fuzzy_match with repeated characters
    // ============================================================

    #[test]
    fn fuzzy_repeated_chars_in_needle() {
        assert!(
            ActionsDialog::fuzzy_match("aabbcc", "abc"),
            "Should match subsequence with repeated chars in haystack"
        );
    }

    #[test]
    fn fuzzy_repeated_chars_in_both() {
        assert!(
            ActionsDialog::fuzzy_match("aabbcc", "aabb"),
            "Should match when both have repeated chars"
        );
    }

    #[test]
    fn fuzzy_needle_longer_than_haystack() {
        assert!(
            !ActionsDialog::fuzzy_match("ab", "abc"),
            "Needle longer than haystack should not match"
        );
    }

    #[test]
    fn fuzzy_exact_match() {
        assert!(
            ActionsDialog::fuzzy_match("hello", "hello"),
            "Exact match is a valid subsequence"
        );
    }

    #[test]
    fn fuzzy_empty_needle_always_matches() {
        assert!(
            ActionsDialog::fuzzy_match("anything", ""),
            "Empty needle should match everything"
        );
    }

    #[test]
    fn fuzzy_empty_haystack_empty_needle() {
        assert!(
            ActionsDialog::fuzzy_match("", ""),
            "Both empty should match"
        );
    }

    #[test]
    fn fuzzy_empty_haystack_nonempty_needle() {
        assert!(
            !ActionsDialog::fuzzy_match("", "a"),
            "Non-empty needle with empty haystack should not match"
        );
    }

    // ============================================================
    // 17. build_grouped_items_static edge cases
    // ============================================================

    #[test]
    fn grouped_single_item_no_section_headers_style() {
        let actions = vec![make_action("a1", "Action 1", Some("Sec"))];
        let filtered = vec![0usize];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Should have 1 header + 1 item = 2
        assert_eq!(grouped.len(), 2);
        assert!(matches!(&grouped[0], GroupedActionItem::SectionHeader(s) if s == "Sec"));
        assert!(matches!(&grouped[1], GroupedActionItem::Item(0)));
    }

    #[test]
    fn grouped_empty_filtered() {
        let actions = vec![make_action("a1", "Action 1", None)];
        let filtered: Vec<usize> = vec![];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(grouped.is_empty());
    }

    #[test]
    fn grouped_none_style_no_headers() {
        let actions = vec![
            make_action("a1", "A1", Some("Sec1")),
            make_action("a2", "A2", Some("Sec2")),
        ];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        // None style should have no section headers
        assert_eq!(grouped.len(), 2);
        for item in &grouped {
            assert!(
                matches!(item, GroupedActionItem::Item(_)),
                "None style should have no headers"
            );
        }
    }

    #[test]
    fn grouped_separators_style_no_headers() {
        let actions = vec![
            make_action("a1", "A1", Some("Sec1")),
            make_action("a2", "A2", Some("Sec2")),
        ];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        assert_eq!(grouped.len(), 2);
        for item in &grouped {
            assert!(
                matches!(item, GroupedActionItem::Item(_)),
                "Separators style should have no headers"
            );
        }
    }

    #[test]
    fn grouped_same_section_no_duplicate_header() {
        let actions = vec![
            make_action("a1", "A1", Some("Same")),
            make_action("a2", "A2", Some("Same")),
            make_action("a3", "A3", Some("Same")),
        ];
        let filtered = vec![0, 1, 2];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        let header_count = grouped
            .iter()
            .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 1, "Same section should produce only 1 header");
    }

    #[test]
    fn grouped_alternating_sections_produce_headers() {
        let actions = vec![
            make_action("a1", "A1", Some("A")),
            make_action("a2", "A2", Some("B")),
            make_action("a3", "A3", Some("A")),
        ];
        let filtered = vec![0, 1, 2];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        let header_count = grouped
            .iter()
            .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
            .count();
        // A -> B -> A = 3 section changes
        assert_eq!(
            header_count, 3,
            "Alternating sections should produce 3 headers"
        );
    }

    // ============================================================
    // 18. coerce_action_selection edge cases
    // ============================================================

    #[test]
    fn coerce_empty_returns_none() {
        assert_eq!(coerce_action_selection(&[], 0), None);
    }

    #[test]
    fn coerce_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".to_string()),
            GroupedActionItem::SectionHeader("B".to_string()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn coerce_on_item_returns_same() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".to_string()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(1));
    }

    #[test]
    fn coerce_header_searches_down() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".to_string()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn coerce_trailing_header_searches_up() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("A".to_string()),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(0));
    }

    #[test]
    fn coerce_alternating_header_item() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".to_string()),
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("B".to_string()),
            GroupedActionItem::Item(1),
            GroupedActionItem::SectionHeader("C".to_string()),
            GroupedActionItem::Item(2),
        ];
        // On header at 0 -> should find Item at 1
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
        // On header at 2 -> should find Item at 3
        assert_eq!(coerce_action_selection(&rows, 2), Some(3));
        // On header at 4 -> should find Item at 5
        assert_eq!(coerce_action_selection(&rows, 4), Some(5));
    }

    #[test]
    fn coerce_out_of_bounds_clamped() {
        let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
        // Index 99 should clamp to len-1 = 1
        assert_eq!(coerce_action_selection(&rows, 99), Some(1));
    }

    // ============================================================
    // 19. parse_shortcut_keycaps edge cases
    // ============================================================

    #[test]
    fn keycaps_empty_string() {
        let result = ActionsDialog::parse_shortcut_keycaps("");
        assert!(result.is_empty());
    }

    #[test]
    fn keycaps_single_modifier() {
        let result = ActionsDialog::parse_shortcut_keycaps("⌘");
        assert_eq!(result, vec!["⌘"]);
    }

    #[test]
    fn keycaps_modifier_plus_letter() {
        let result = ActionsDialog::parse_shortcut_keycaps("⌘C");
        assert_eq!(result, vec!["⌘", "C"]);
    }

    #[test]
    fn keycaps_all_modifiers() {
        let result = ActionsDialog::parse_shortcut_keycaps("⌘⌃⌥⇧");
        assert_eq!(result, vec!["⌘", "⌃", "⌥", "⇧"]);
    }

    #[test]
    fn keycaps_special_keys() {
        let result = ActionsDialog::parse_shortcut_keycaps("↵⎋⇥⌫␣↑↓←→");
        assert_eq!(result, vec!["↵", "⎋", "⇥", "⌫", "␣", "↑", "↓", "←", "→"]);
    }

    #[test]
    fn keycaps_lowercase_uppercased() {
        let result = ActionsDialog::parse_shortcut_keycaps("⌘a");
        assert_eq!(result, vec!["⌘", "A"]);
    }

    // ============================================================
    // 20. CommandBarConfig close flags independence
    // ============================================================

    #[test]
    fn command_bar_default_all_close_true() {
        let config = CommandBarConfig::default();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

