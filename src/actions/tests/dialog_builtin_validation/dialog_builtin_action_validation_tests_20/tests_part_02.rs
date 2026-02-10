    #[test]
    fn cat07_model_title_checkmark_current() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".into()),
            available_models: vec![
                ChatModelInfo {
                    id: "gpt-4".into(),
                    display_name: "GPT-4".into(),
                    provider: "OpenAI".into(),
                },
                ChatModelInfo {
                    id: "claude".into(),
                    display_name: "Claude".into(),
                    provider: "Anthropic".into(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let gpt4 = actions
            .iter()
            .find(|a| a.id == "select_model_gpt-4")
            .unwrap();
        assert!(gpt4.title.contains('✓'));
        let claude = actions
            .iter()
            .find(|a| a.id == "select_model_claude")
            .unwrap();
        assert!(!claude.title.contains('✓'));
    }

    #[test]
    fn cat07_model_description_via_provider() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "m".into(),
                display_name: "M".into(),
                provider: "Acme".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let m = actions.iter().find(|a| a.id == "select_model_m").unwrap();
        assert_eq!(m.description.as_ref().unwrap(), "via Acme");
    }

    // =========================================================================
    // Category 08: New chat last_used section icon is BoltFilled
    // Verifies icon and section assignments in get_new_chat_actions.
    // =========================================================================

    #[test]
    fn cat08_last_used_icon_bolt() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m".into(),
            display_name: "M".into(),
            provider: "P".into(),
            provider_display_name: "Provider".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
    }

    #[test]
    fn cat08_last_used_section_name() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m".into(),
            display_name: "M".into(),
            provider: "P".into(),
            provider_display_name: "Provider".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].section.as_ref().unwrap(), "Last Used Settings");
    }

    #[test]
    fn cat08_model_section_name() {
        let models = vec![NewChatModelInfo {
            model_id: "m".into(),
            display_name: "M".into(),
            provider: "P".into(),
            provider_display_name: "Provider".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].section.as_ref().unwrap(), "Models");
    }

    #[test]
    fn cat08_model_icon_settings() {
        let models = vec![NewChatModelInfo {
            model_id: "m".into(),
            display_name: "M".into(),
            provider: "P".into(),
            provider_display_name: "Provider".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].icon, Some(IconName::Settings));
    }

    #[test]
    fn cat08_preset_section_name() {
        let presets = vec![NewChatPresetInfo {
            id: "gen".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].section.as_ref().unwrap(), "Presets");
    }

    #[test]
    fn cat08_preset_icon_preserved() {
        let presets = vec![NewChatPresetInfo {
            id: "gen".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].icon, Some(IconName::Star));
    }

    // =========================================================================
    // Category 09: Note switcher description with preview + relative_time
    // Verifies the "preview · time" format and char count fallback.
    // =========================================================================

    #[test]
    fn cat09_preview_and_time_format() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "Note".into(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "Hello world".into(),
            relative_time: "2m ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains("Hello world"));
        assert!(desc.contains("2m ago"));
        assert!(desc.contains(" · "));
    }

    #[test]
    fn cat09_no_preview_uses_char_count() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "Note".into(),
            char_count: 42,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains("42 chars"));
    }

    #[test]
    fn cat09_char_count_singular() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "Note".into(),
            char_count: 1,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains("1 char"));
        assert!(!desc.contains("chars"));
    }

    #[test]
    fn cat09_preview_only_no_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "Note".into(),
            char_count: 5,
            is_current: false,
            is_pinned: false,
            preview: "Some text".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "Some text");
    }

    #[test]
    fn cat09_time_only_no_preview() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "Note".into(),
            char_count: 5,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "5d ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "5d ago");
    }

    // =========================================================================
    // Category 10: to_deeplink_name CJK and accented character preservation
    // Verifies Unicode alphanumeric chars are kept, not stripped.
    // =========================================================================

    #[test]
    fn cat10_cjk_chars_preserved() {
        let result = to_deeplink_name("测试脚本");
        assert!(result.contains("测试脚本"));
    }

    #[test]
    fn cat10_accented_chars_preserved() {
        let result = to_deeplink_name("Résumé Editor");
        assert!(result.contains("résumé"));
    }

    #[test]
    fn cat10_mixed_case_lowered() {
        assert_eq!(to_deeplink_name("MyScript"), "myscript");
    }

    #[test]
    fn cat10_mixed_special_and_alpha() {
        assert_eq!(to_deeplink_name("My -- Script!"), "my-script");
    }

    #[test]
    fn cat10_empty_string() {
        assert_eq!(to_deeplink_name(""), "");
    }

    // =========================================================================
    // Category 11: Action::new pre-computes lowercase fields correctly
    // =========================================================================

    #[test]
    fn cat11_title_lower_matches() {
        let action = Action::new("id", "Hello World", None, ActionCategory::ScriptContext);
        assert_eq!(action.title_lower, "hello world");
    }

    #[test]
    fn cat11_description_lower_matches() {
        let action = Action::new(
            "id",
            "T",
            Some("Hello DESC".to_string()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.description_lower, Some("hello desc".to_string()));
    }

    #[test]
    fn cat11_description_lower_none() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext);
        assert!(action.description_lower.is_none());
    }

    #[test]
    fn cat11_shortcut_lower_none_initially() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext);
        assert!(action.shortcut_lower.is_none());
    }

    #[test]
    fn cat11_shortcut_lower_set_after_with() {
        let action =
            Action::new("id", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
        assert_eq!(action.shortcut_lower, Some("⌘⇧c".to_string()));
    }

    // =========================================================================
    // Category 12: score_action scoring tiers via ActionsDialog
    // =========================================================================

    #[test]
    fn cat12_prefix_scores_100() {
        let action = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "edit");
        assert_eq!(score, 100);
    }

    #[test]
    fn cat12_contains_scores_50() {
        let action = Action::new("id", "Copy Edit Path", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "edit");
        assert_eq!(score, 50);
    }

    #[test]
    fn cat12_no_match_scores_0() {
        let action = Action::new("id", "Run Script", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "xyz");
        assert_eq!(score, 0);
    }

    #[test]
    fn cat12_desc_bonus_15() {
        let action = Action::new(
            "id",
            "Open File",
            Some("Edit in editor".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "editor");
        assert!(
            score >= 15,
            "Description match should add at least 15 points, got {}",
            score
        );
    }

    #[test]
    fn cat12_shortcut_bonus_10() {
        let action =
            Action::new("id", "Test", None, ActionCategory::ScriptContext).with_shortcut("⌘X");
        let score = ActionsDialog::score_action(&action, "⌘x");
        assert!(
            score >= 10,
            "Shortcut match should score 10+, got {}",
            score
        );
    }

    // =========================================================================
    // Category 13: fuzzy_match subsequence behavior
    // =========================================================================

    #[test]
    fn cat13_exact_match() {
        assert!(ActionsDialog::fuzzy_match("hello", "hello"));
    }

    #[test]
    fn cat13_subsequence_match() {
        assert!(ActionsDialog::fuzzy_match("hello world", "hlwrd"));
    }

    #[test]
    fn cat13_no_match() {
        assert!(!ActionsDialog::fuzzy_match("hello", "xyz"));
    }

    #[test]
    fn cat13_empty_needle_matches() {
        assert!(ActionsDialog::fuzzy_match("hello", ""));
    }

    #[test]
    fn cat13_empty_haystack_no_match() {
        assert!(!ActionsDialog::fuzzy_match("", "a"));
    }

    #[test]
    fn cat13_both_empty() {
        assert!(ActionsDialog::fuzzy_match("", ""));
    }

    #[test]
    fn cat13_needle_longer_fails() {
        assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
    }

    // =========================================================================
    // Category 14: parse_shortcut_keycaps splits correctly
    // =========================================================================

    #[test]
    fn cat14_cmd_c_two_caps() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘C");
        assert_eq!(caps.len(), 2);
        assert_eq!(caps[0], "⌘");
        assert_eq!(caps[1], "C");
    }

    #[test]
    fn cat14_cmd_shift_c_three_caps() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘⇧C");
        assert_eq!(caps.len(), 3);
    }

    #[test]
    fn cat14_enter_single() {
        let caps = ActionsDialog::parse_shortcut_keycaps("↵");
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0], "↵");
    }

    #[test]
    fn cat14_arrows() {
        let caps = ActionsDialog::parse_shortcut_keycaps("↑↓←→");
        assert_eq!(caps.len(), 4);
    }

    #[test]
    fn cat14_escape() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⎋");
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0], "⎋");
    }

    // =========================================================================
    // Category 15: build_grouped_items_static with Headers style
    // =========================================================================

    #[test]
    fn cat15_headers_add_section_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Should have: Header("S1"), Item(0), Header("S2"), Item(1)
        assert_eq!(grouped.len(), 4);
        assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(_)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[2], GroupedActionItem::SectionHeader(_)));
        assert!(matches!(grouped[3], GroupedActionItem::Item(1)));
    }

