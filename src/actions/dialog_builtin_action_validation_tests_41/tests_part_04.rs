    #[test]
    fn chat_models_come_before_continue_in_chat() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![
                ChatModelInfo {
                    id: "a".to_string(),
                    display_name: "Model A".to_string(),
                    provider: "P".to_string(),
                },
                ChatModelInfo {
                    id: "b".to_string(),
                    display_name: "Model B".to_string(),
                    provider: "P".to_string(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let continue_idx = actions
            .iter()
            .position(|a| a.id == "continue_in_chat")
            .unwrap();
        let model_a_idx = actions
            .iter()
            .position(|a| a.id == "select_model_a")
            .unwrap();
        let model_b_idx = actions
            .iter()
            .position(|a| a.id == "select_model_b")
            .unwrap();
        assert!(model_a_idx < continue_idx);
        assert!(model_b_idx < continue_idx);
    }

    #[test]
    fn chat_models_preserve_order() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![
                ChatModelInfo {
                    id: "first".to_string(),
                    display_name: "First".to_string(),
                    provider: "P".to_string(),
                },
                ChatModelInfo {
                    id: "second".to_string(),
                    display_name: "Second".to_string(),
                    provider: "P".to_string(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let first_idx = actions
            .iter()
            .position(|a| a.id == "select_model_first")
            .unwrap();
        let second_idx = actions
            .iter()
            .position(|a| a.id == "select_model_second")
            .unwrap();
        assert!(first_idx < second_idx);
    }

    #[test]
    fn chat_both_messages_and_response_max_actions() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "m1".to_string(),
                display_name: "Model".to_string(),
                provider: "P".to_string(),
            }],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        // 1 model + continue + copy_response + clear_conversation = 4
        assert_eq!(actions.len(), 4);
    }

    #[test]
    fn chat_no_models_no_messages_minimal() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        // Only continue_in_chat
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "continue_in_chat");
    }

    // =========================================================================
    // 29. New chat: section ordering across last_used, presets, models
    // =========================================================================

    #[test]
    fn new_chat_section_ordering_last_used_first() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".to_string(),
            display_name: "Model 1".to_string(),
            provider: "p".to_string(),
            provider_display_name: "Provider".to_string(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "general".to_string(),
            name: "General".to_string(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m2".to_string(),
            display_name: "Model 2".to_string(),
            provider: "p".to_string(),
            provider_display_name: "Provider".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        // First action section should be Last Used Settings
        assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
    }

    #[test]
    fn new_chat_section_ordering_presets_second() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".to_string(),
            display_name: "Model 1".to_string(),
            provider: "p".to_string(),
            provider_display_name: "Provider".to_string(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "general".to_string(),
            name: "General".to_string(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m2".to_string(),
            display_name: "Model 2".to_string(),
            provider: "p".to_string(),
            provider_display_name: "Provider".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        assert_eq!(actions[1].section.as_deref(), Some("Presets"));
    }

    #[test]
    fn new_chat_section_ordering_models_last() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".to_string(),
            display_name: "Model 1".to_string(),
            provider: "p".to_string(),
            provider_display_name: "Provider".to_string(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "general".to_string(),
            name: "General".to_string(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m2".to_string(),
            display_name: "Model 2".to_string(),
            provider: "p".to_string(),
            provider_display_name: "Provider".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        assert_eq!(actions[2].section.as_deref(), Some("Models"));
    }

    #[test]
    fn new_chat_total_count_matches_input_sizes() {
        let last_used = vec![
            NewChatModelInfo {
                model_id: "m1".to_string(),
                display_name: "M1".to_string(),
                provider: "p".to_string(),
                provider_display_name: "P".to_string(),
            },
            NewChatModelInfo {
                model_id: "m2".to_string(),
                display_name: "M2".to_string(),
                provider: "p".to_string(),
                provider_display_name: "P".to_string(),
            },
        ];
        let presets = vec![NewChatPresetInfo {
            id: "g".to_string(),
            name: "General".to_string(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m3".to_string(),
            display_name: "M3".to_string(),
            provider: "p".to_string(),
            provider_display_name: "P".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        assert_eq!(actions.len(), 4); // 2 + 1 + 1
    }

    // =========================================================================
    // 30. count_section_headers: items without sections produce 0 headers
    // =========================================================================

    #[test]
    fn count_headers_no_sections_is_zero() {
        let actions = vec![
            Action::new("a", "Action A", None, ActionCategory::ScriptContext),
            Action::new("b", "Action B", None, ActionCategory::ScriptContext),
        ];
        let indices: Vec<usize> = (0..actions.len()).collect();
        assert_eq!(count_section_headers(&actions, &indices), 0);
    }

    #[test]
    fn count_headers_all_same_section_is_one() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Group"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Group"),
        ];
        let indices: Vec<usize> = (0..actions.len()).collect();
        assert_eq!(count_section_headers(&actions, &indices), 1);
    }

    #[test]
    fn count_headers_two_different_sections() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Alpha"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Beta"),
        ];
        let indices: Vec<usize> = (0..actions.len()).collect();
        assert_eq!(count_section_headers(&actions, &indices), 2);
    }

    #[test]
    fn count_headers_empty_indices() {
        let actions =
            vec![Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("X")];
        let indices: Vec<usize> = vec![];
        assert_eq!(count_section_headers(&actions, &indices), 0);
    }
