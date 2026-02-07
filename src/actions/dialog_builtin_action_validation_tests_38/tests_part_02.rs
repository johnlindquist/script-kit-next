    #[test]
    fn ai_bar_submit_shortcut() {
        let actions = get_ai_command_bar_actions();
        let submit = actions.iter().find(|a| a.id == "submit").unwrap();
        assert_eq!(submit.shortcut.as_deref(), Some("↵"));
    }

    #[test]
    fn ai_bar_submit_icon() {
        let actions = get_ai_command_bar_actions();
        let submit = actions.iter().find(|a| a.id == "submit").unwrap();
        assert_eq!(submit.icon, Some(IconName::ArrowUp));
    }

    #[test]
    fn ai_bar_submit_section_is_actions() {
        let actions = get_ai_command_bar_actions();
        let submit = actions.iter().find(|a| a.id == "submit").unwrap();
        assert_eq!(submit.section.as_deref(), Some("Actions"));
    }

    #[test]
    fn ai_bar_submit_desc_mentions_send() {
        let actions = get_ai_command_bar_actions();
        let submit = actions.iter().find(|a| a.id == "submit").unwrap();
        assert!(submit.description.as_ref().unwrap().contains("Send"));
    }

    // =========================================================================
    // 11. Chat context: empty available_models
    // =========================================================================

    #[test]
    fn chat_empty_models_still_has_continue() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "continue_in_chat"));
    }

    #[test]
    fn chat_empty_models_no_response_count_is_1() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions.len(), 1);
    }

    #[test]
    fn chat_model_id_uses_model_id_field() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "claude-3".into(),
                display_name: "Claude 3".into(),
                provider: "Anthropic".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "select_model_claude-3"));
    }

    #[test]
    fn chat_current_model_gets_check_mark() {
        let info = ChatPromptInfo {
            current_model: Some("Claude 3".into()),
            available_models: vec![ChatModelInfo {
                id: "claude-3".into(),
                display_name: "Claude 3".into(),
                provider: "Anthropic".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model = actions
            .iter()
            .find(|a| a.id == "select_model_claude-3")
            .unwrap();
        assert!(model.title.contains('✓'));
    }

    // =========================================================================
    // 12. New chat: last_used icon is BoltFilled
    // =========================================================================

    #[test]
    fn new_chat_last_used_icon_bolt_filled() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "Model 1".into(),
            provider: "p1".into(),
            provider_display_name: "Provider 1".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        let action = actions.iter().find(|a| a.id == "last_used_0").unwrap();
        assert_eq!(action.icon, Some(IconName::BoltFilled));
    }

    #[test]
    fn new_chat_last_used_section() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "Model 1".into(),
            provider: "p1".into(),
            provider_display_name: "Provider 1".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        let action = actions.iter().find(|a| a.id == "last_used_0").unwrap();
        assert_eq!(action.section.as_deref(), Some("Last Used Settings"));
    }

    #[test]
    fn new_chat_last_used_desc_is_provider_display() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "Model 1".into(),
            provider: "p1".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        let action = actions.iter().find(|a| a.id == "last_used_0").unwrap();
        assert_eq!(action.description.as_deref(), Some("Anthropic"));
    }

    #[test]
    fn new_chat_model_icon_settings() {
        let models = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "Model 1".into(),
            provider: "p1".into(),
            provider_display_name: "Provider 1".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        let action = actions.iter().find(|a| a.id == "model_0").unwrap();
        assert_eq!(action.icon, Some(IconName::Settings));
    }

    // =========================================================================
    // 13. New chat: preset section and desc
    // =========================================================================

    #[test]
    fn new_chat_preset_section_is_presets() {
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        let action = actions.iter().find(|a| a.id == "preset_general").unwrap();
        assert_eq!(action.section.as_deref(), Some("Presets"));
    }

    #[test]
    fn new_chat_preset_desc_is_none() {
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        let action = actions.iter().find(|a| a.id == "preset_general").unwrap();
        assert!(action.description.is_none());
    }

    #[test]
    fn new_chat_preset_icon_preserved() {
        let presets = vec![NewChatPresetInfo {
            id: "code".into(),
            name: "Code".into(),
            icon: IconName::Code,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        let action = actions.iter().find(|a| a.id == "preset_code").unwrap();
        assert_eq!(action.icon, Some(IconName::Code));
    }

    #[test]
    fn new_chat_model_section_is_models() {
        let models = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "M1".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        let action = actions.iter().find(|a| a.id == "model_0").unwrap();
        assert_eq!(action.section.as_deref(), Some("Models"));
    }

    // =========================================================================
    // 14. Note switcher: current note gets "• " prefix
    // =========================================================================

    #[test]
    fn note_switcher_current_note_has_bullet_prefix() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "My Note".into(),
            char_count: 42,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].title, "• My Note");
    }

    #[test]
    fn note_switcher_non_current_note_no_prefix() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "My Note".into(),
            char_count: 42,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].title, "My Note");
    }

    #[test]
    fn note_switcher_current_icon_is_check_when_not_pinned() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "Test".into(),
            char_count: 10,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::Check));
    }

    #[test]
    fn note_switcher_pinned_takes_priority_over_current() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "Test".into(),
            char_count: 10,
            is_current: true,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    // =========================================================================
    // 15. Note switcher: preview trimming at 60 chars
    // =========================================================================

    #[test]
    fn note_switcher_preview_exactly_60_not_truncated() {
        let preview: String = "A".repeat(60);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "T".into(),
            char_count: 60,
            is_current: false,
            is_pinned: false,
            preview,
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(!desc.contains('…'));
    }

    #[test]
    fn note_switcher_preview_61_chars_truncated() {
        let preview: String = "B".repeat(61);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "T".into(),
            char_count: 61,
            is_current: false,
            is_pinned: false,
            preview,
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains('…'));
    }

    #[test]
    fn note_switcher_preview_with_time_has_separator() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "T".into(),
            char_count: 10,
            is_current: false,
            is_pinned: false,
            preview: "Hello world".into(),
            relative_time: "2m ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains(" · "));
    }

    #[test]
    fn note_switcher_no_preview_no_time_shows_chars() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "T".into(),
            char_count: 99,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "99 chars");
    }

    // =========================================================================
    // 16. count_section_headers: edge cases
    // =========================================================================

    #[test]
    fn count_section_headers_empty_filtered() {
        let actions: Vec<Action> = vec![];
        let filtered: Vec<usize> = vec![];
        assert_eq!(count_section_headers(&actions, &filtered), 0);
    }

    #[test]
    fn count_section_headers_no_sections() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext),
            Action::new("b", "B", None, ActionCategory::ScriptContext),
        ];
        let filtered = vec![0, 1];
        assert_eq!(count_section_headers(&actions, &filtered), 0);
    }

    #[test]
    fn count_section_headers_all_same_section() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S"),
        ];
        let filtered = vec![0, 1];
        assert_eq!(count_section_headers(&actions, &filtered), 1);
    }

    #[test]
    fn count_section_headers_different_sections() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("X"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Y"),
            Action::new("c", "C", None, ActionCategory::ScriptContext).with_section("Z"),
        ];
        let filtered = vec![0, 1, 2];
        assert_eq!(count_section_headers(&actions, &filtered), 3);
    }

    // =========================================================================
    // 17. count_section_headers: mixed with and without sections
    // =========================================================================

    #[test]
    fn count_section_headers_mixed_some_none() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S"),
            Action::new("b", "B", None, ActionCategory::ScriptContext), // no section
            Action::new("c", "C", None, ActionCategory::ScriptContext).with_section("S"),
        ];
        let filtered = vec![0, 1, 2];
        // First S = 1 header; no section in b = skip; second S after None = new header
        assert_eq!(count_section_headers(&actions, &filtered), 2);
    }

    #[test]
    fn count_section_headers_filtered_subset() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("X"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Y"),
            Action::new("c", "C", None, ActionCategory::ScriptContext).with_section("X"),
        ];
        // Only index 0 and 2 (same section X, but separated by skipped Y)
        let filtered = vec![0, 2];
        assert_eq!(count_section_headers(&actions, &filtered), 1);
    }

    #[test]
    fn count_section_headers_single_action_with_section() {
        let actions =
            vec![Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S")];
        let filtered = vec![0];
        assert_eq!(count_section_headers(&actions, &filtered), 1);
    }

    #[test]
    fn count_section_headers_single_action_no_section() {
        let actions = vec![Action::new("a", "A", None, ActionCategory::ScriptContext)];
        let filtered = vec![0];
        assert_eq!(count_section_headers(&actions, &filtered), 0);
    }

    // =========================================================================
    // 18. WindowPosition enum variants and Default
    // =========================================================================

    #[test]
    fn window_position_default_is_bottom_right() {
        assert_eq!(WindowPosition::default(), WindowPosition::BottomRight);
    }

    #[test]
    fn window_position_bottom_right_variant_exists() {
        let _pos = WindowPosition::BottomRight;
    }

    #[test]
    fn window_position_top_right_variant_exists() {
        let _pos = WindowPosition::TopRight;
    }

    #[test]
    fn window_position_top_center_variant_exists() {
        let _pos = WindowPosition::TopCenter;
    }

    // =========================================================================
    // 19. ProtocolAction::with_value constructor
    // =========================================================================

