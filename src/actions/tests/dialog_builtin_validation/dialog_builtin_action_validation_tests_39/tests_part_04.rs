    #[test]
    fn new_chat_model_id_uses_index() {
        let models = vec![
            NewChatModelInfo {
                model_id: "claude".into(),
                display_name: "Claude".into(),
                provider: "anthropic".into(),
                provider_display_name: "Anthropic".into(),
            },
            NewChatModelInfo {
                model_id: "gpt4".into(),
                display_name: "GPT-4".into(),
                provider: "openai".into(),
                provider_display_name: "OpenAI".into(),
            },
        ];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].id, "model_0");
        assert_eq!(actions[1].id, "model_1");
    }

    #[test]
    fn new_chat_preset_id_uses_preset_id() {
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].id, "preset_general");
    }

    // =========================================================================
    // 27. Note switcher: singular vs plural char count
    // =========================================================================

    #[test]
    fn note_switcher_one_char_singular() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "Note".into(),
            char_count: 1,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].description.as_deref(), Some("1 char"));
    }

    #[test]
    fn note_switcher_zero_chars_plural() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "Note".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].description.as_deref(), Some("0 chars"));
    }

    #[test]
    fn note_switcher_many_chars_plural() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "Note".into(),
            char_count: 500,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].description.as_deref(), Some("500 chars"));
    }

    #[test]
    fn note_switcher_two_chars_plural() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "Note".into(),
            char_count: 2,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].description.as_deref(), Some("2 chars"));
    }

    // =========================================================================
    // 28. Note switcher: section assignment pinned vs recent
    // =========================================================================

    #[test]
    fn note_switcher_pinned_section() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "Note".into(),
            char_count: 10,
            is_current: false,
            is_pinned: true,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
    }

    #[test]
    fn note_switcher_unpinned_section() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "Note".into(),
            char_count: 10,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn note_switcher_mixed_sections() {
        let notes = vec![
            NoteSwitcherNoteInfo {
                id: "1".into(),
                title: "A".into(),
                char_count: 10,
                is_current: false,
                is_pinned: true,
                preview: "".into(),
                relative_time: "".into(),
            },
            NoteSwitcherNoteInfo {
                id: "2".into(),
                title: "B".into(),
                char_count: 20,
                is_current: false,
                is_pinned: false,
                preview: "".into(),
                relative_time: "".into(),
            },
        ];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
        assert_eq!(actions[1].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn note_switcher_current_pinned_still_pinned_section() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "Note".into(),
            char_count: 10,
            is_current: true,
            is_pinned: true,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
    }

    // =========================================================================
    // 29. coerce_action_selection: all headers returns None
    // =========================================================================

    #[test]
    fn coerce_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".into()),
            GroupedActionItem::SectionHeader("B".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn coerce_empty_returns_none() {
        let rows: Vec<GroupedActionItem> = vec![];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn coerce_header_then_item_returns_item_index() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".into()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn coerce_item_at_exact_index_returns_same() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("A".into()),
            GroupedActionItem::Item(1),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    }

    // =========================================================================
    // 30. build_grouped_items_static: filter_idx in Item matches enumerate order
    // =========================================================================

    #[test]
    fn build_grouped_items_no_sections_items_sequential() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext),
            Action::new("b", "B", None, ActionCategory::ScriptContext),
        ];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        assert_eq!(grouped.len(), 2);
        assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(1)));
    }

    #[test]
    fn build_grouped_items_with_headers_adds_section_header() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // S1 header + item(0) + S2 header + item(1) = 4
        assert_eq!(grouped.len(), 4);
        assert!(matches!(&grouped[0], GroupedActionItem::SectionHeader(s) if s == "S1"));
        assert!(matches!(grouped[1], GroupedActionItem::Item(0)));
        assert!(matches!(&grouped[2], GroupedActionItem::SectionHeader(s) if s == "S2"));
        assert!(matches!(grouped[3], GroupedActionItem::Item(1)));
    }

    #[test]
    fn build_grouped_items_separators_no_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        // No headers, just items
        assert_eq!(grouped.len(), 2);
        assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(1)));
    }

    #[test]
    fn build_grouped_items_empty_filtered() {
        let actions = vec![Action::new("a", "A", None, ActionCategory::ScriptContext)];
        let filtered: Vec<usize> = vec![];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(grouped.is_empty());
    }
