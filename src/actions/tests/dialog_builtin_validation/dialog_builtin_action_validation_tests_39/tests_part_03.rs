    #[test]
    fn ai_bar_paste_image_section() {
        let actions = get_ai_command_bar_actions();
        let pi = actions.iter().find(|a| a.id == "paste_image").unwrap();
        assert_eq!(pi.section.as_deref(), Some("Attachments"));
    }

    #[test]
    fn ai_bar_paste_image_desc_mentions_clipboard() {
        let actions = get_ai_command_bar_actions();
        let pi = actions.iter().find(|a| a.id == "paste_image").unwrap();
        assert!(pi
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("clipboard"));
    }

    // =========================================================================
    // 20. AI bar: section ordering matches declaration order
    // =========================================================================

    #[test]
    fn ai_bar_first_section_is_response() {
        let actions = get_ai_command_bar_actions();
        let first_with_section = actions.iter().find(|a| a.section.is_some()).unwrap();
        assert_eq!(first_with_section.section.as_deref(), Some("Response"));
    }

    #[test]
    fn ai_bar_last_section_is_settings() {
        let actions = get_ai_command_bar_actions();
        let last = actions.last().unwrap();
        assert_eq!(last.section.as_deref(), Some("Settings"));
    }

    #[test]
    fn ai_bar_export_section_has_one_action() {
        let actions = get_ai_command_bar_actions();
        let export_count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Export"))
            .count();
        assert_eq!(export_count, 1);
    }

    #[test]
    fn ai_bar_attachments_section_has_two_actions() {
        let actions = get_ai_command_bar_actions();
        let att_count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Attachments"))
            .count();
        assert_eq!(att_count, 2);
    }

    // =========================================================================
    // 21. Notes: section distribution with selection + no trash + disabled auto
    // =========================================================================

    #[test]
    fn notes_full_selection_has_notes_section() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions
            .iter()
            .any(|a| a.section.as_deref() == Some("Notes")));
    }

    #[test]
    fn notes_full_selection_has_edit_section() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.section.as_deref() == Some("Edit")));
    }

    #[test]
    fn notes_full_selection_has_copy_section() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.section.as_deref() == Some("Copy")));
    }

    #[test]
    fn notes_full_selection_has_settings_when_auto_disabled() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions
            .iter()
            .any(|a| a.section.as_deref() == Some("Settings")));
    }

    // =========================================================================
    // 22. Notes: all actions have icons
    // =========================================================================

    #[test]
    fn notes_full_all_have_icons() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for action in &actions {
            assert!(action.icon.is_some(), "Action {} has no icon", action.id);
        }
    }

    #[test]
    fn notes_no_selection_all_have_icons() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for action in &actions {
            assert!(action.icon.is_some(), "Action {} has no icon", action.id);
        }
    }

    #[test]
    fn notes_trash_all_have_icons() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for action in &actions {
            assert!(action.icon.is_some(), "Action {} has no icon", action.id);
        }
    }

    #[test]
    fn notes_new_note_icon_is_plus() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let nn = actions.iter().find(|a| a.id == "new_note").unwrap();
        assert_eq!(nn.icon, Some(IconName::Plus));
    }

    // =========================================================================
    // 23. Chat context: model actions come before continue_in_chat
    // =========================================================================

    #[test]
    fn chat_model_actions_before_continue() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".into()),
            available_models: vec![
                ChatModelInfo {
                    id: "claude".into(),
                    display_name: "Claude".into(),
                    provider: "Anthropic".into(),
                },
                ChatModelInfo {
                    id: "gpt4".into(),
                    display_name: "GPT-4".into(),
                    provider: "OpenAI".into(),
                },
            ],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        let model_pos = actions
            .iter()
            .position(|a| a.id.starts_with("select_model_"))
            .unwrap();
        let continue_pos = actions
            .iter()
            .position(|a| a.id == "continue_in_chat")
            .unwrap();
        assert!(model_pos < continue_pos);
    }

    #[test]
    fn chat_all_model_actions_contiguous() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".into()),
            available_models: vec![
                ChatModelInfo {
                    id: "claude".into(),
                    display_name: "Claude".into(),
                    provider: "Anthropic".into(),
                },
                ChatModelInfo {
                    id: "gpt4".into(),
                    display_name: "GPT-4".into(),
                    provider: "OpenAI".into(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_indices: Vec<usize> = actions
            .iter()
            .enumerate()
            .filter(|(_, a)| a.id.starts_with("select_model_"))
            .map(|(i, _)| i)
            .collect();
        assert_eq!(model_indices, vec![0, 1]);
    }

    #[test]
    fn chat_continue_in_chat_always_after_models() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions[0].id, "continue_in_chat");
    }

    #[test]
    fn chat_copy_response_after_continue() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".into()),
            available_models: vec![],
            has_messages: false,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        let cont_pos = actions
            .iter()
            .position(|a| a.id == "continue_in_chat")
            .unwrap();
        let copy_pos = actions
            .iter()
            .position(|a| a.id == "copy_response")
            .unwrap();
        assert!(copy_pos > cont_pos);
    }

    // =========================================================================
    // 24. Chat context: current model marked with checkmark
    // =========================================================================

    #[test]
    fn chat_current_model_has_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".into()),
            available_models: vec![ChatModelInfo {
                id: "claude".into(),
                display_name: "Claude".into(),
                provider: "Anthropic".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = actions
            .iter()
            .find(|a| a.id == "select_model_claude")
            .unwrap();
        assert!(model_action.title.contains("✓"));
    }

    #[test]
    fn chat_non_current_model_no_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".into()),
            available_models: vec![ChatModelInfo {
                id: "gpt4".into(),
                display_name: "GPT-4".into(),
                provider: "OpenAI".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = actions
            .iter()
            .find(|a| a.id == "select_model_gpt4")
            .unwrap();
        assert!(!model_action.title.contains("✓"));
    }

    #[test]
    fn chat_model_desc_mentions_provider() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "claude".into(),
                display_name: "Claude".into(),
                provider: "Anthropic".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = actions
            .iter()
            .find(|a| a.id == "select_model_claude")
            .unwrap();
        assert!(model_action
            .description
            .as_ref()
            .unwrap()
            .contains("Anthropic"));
    }

    #[test]
    fn chat_model_no_shortcut() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "claude".into(),
                display_name: "Claude".into(),
                provider: "Anthropic".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = actions
            .iter()
            .find(|a| a.id == "select_model_claude")
            .unwrap();
        assert!(model_action.shortcut.is_none());
    }

    // =========================================================================
    // 25. New chat: last_used IDs use index format
    // =========================================================================

    #[test]
    fn new_chat_last_used_id_format() {
        let last_used = vec![NewChatModelInfo {
            model_id: "claude".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].id, "last_used_0");
    }

    #[test]
    fn new_chat_last_used_second_id() {
        let last_used = vec![
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
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[1].id, "last_used_1");
    }

    #[test]
    fn new_chat_last_used_desc_is_provider_display_name() {
        let last_used = vec![NewChatModelInfo {
            model_id: "claude".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].description.as_deref(), Some("Anthropic"));
    }

    #[test]
    fn new_chat_last_used_section() {
        let last_used = vec![NewChatModelInfo {
            model_id: "claude".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
    }

    // =========================================================================
    // 26. New chat: model section actions use Settings icon
    // =========================================================================

    #[test]
    fn new_chat_model_icon_is_settings() {
        let models = vec![NewChatModelInfo {
            model_id: "claude".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].icon, Some(IconName::Settings));
    }

    #[test]
    fn new_chat_model_section_is_models() {
        let models = vec![NewChatModelInfo {
            model_id: "claude".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].section.as_deref(), Some("Models"));
    }

