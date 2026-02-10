    #[test]
    fn note_switcher_regular_has_file_icon() {
        let notes = vec![make_note(
            "id1",
            "Regular",
            50,
            false,
            false,
            "regular content",
            "3d ago",
        )];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::File));
        assert_eq!(actions[0].section, Some("Recent".to_string()));
    }

    #[test]
    fn note_switcher_pinned_overrides_current_icon() {
        // When both pinned and current, pinned wins for icon
        let notes = vec![make_note(
            "id1",
            "Both",
            50,
            true,
            true,
            "both flags",
            "1m ago",
        )];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(
            actions[0].icon,
            Some(IconName::StarFilled),
            "Pinned should override current for icon"
        );
        assert_eq!(actions[0].section, Some("Pinned".to_string()));
    }

    #[test]
    fn note_switcher_empty_returns_placeholder() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "no_notes");
        assert_eq!(actions[0].title, "No notes yet");
    }

    #[test]
    fn note_switcher_id_format() {
        let notes = vec![make_note("abc-123-def", "Test", 10, false, false, "", "")];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].id, "note_abc-123-def");
    }

    // ============================================================
    // 6. Chat context partial state combinations
    // ============================================================

    #[test]
    fn chat_no_models_no_messages_no_response() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        // Should still have continue_in_chat
        assert!(actions.iter().any(|a| a.id == "continue_in_chat"));
        // Should NOT have copy_response or clear
        assert!(!actions.iter().any(|a| a.id == "copy_response"));
        assert!(!actions.iter().any(|a| a.id == "clear_conversation"));
    }

    #[test]
    fn chat_has_response_but_no_messages() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "copy_response"));
        assert!(!actions.iter().any(|a| a.id == "clear_conversation"));
    }

    #[test]
    fn chat_has_messages_but_no_response() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "copy_response"));
        assert!(actions.iter().any(|a| a.id == "clear_conversation"));
    }

    #[test]
    fn chat_all_flags_true() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".to_string()),
            available_models: vec![ChatModelInfo {
                id: "claude-3".to_string(),
                display_name: "Claude".to_string(),
                provider: "Anthropic".to_string(),
            }],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "continue_in_chat"));
        assert!(actions.iter().any(|a| a.id == "copy_response"));
        assert!(actions.iter().any(|a| a.id == "clear_conversation"));
        // Model should have checkmark
        let model_action = actions
            .iter()
            .find(|a| a.id == "select_model_claude-3")
            .unwrap();
        assert!(model_action.title.contains('✓'));
    }

    #[test]
    fn chat_checkmark_only_on_exact_display_name_match() {
        let info = ChatPromptInfo {
            current_model: Some("Claude 3.5".to_string()),
            available_models: vec![
                ChatModelInfo {
                    id: "claude-3".to_string(),
                    display_name: "Claude 3".to_string(),
                    provider: "Anthropic".to_string(),
                },
                ChatModelInfo {
                    id: "claude-35".to_string(),
                    display_name: "Claude 3.5".to_string(),
                    provider: "Anthropic".to_string(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let claude3 = find_action(&actions, "select_model_claude-3").unwrap();
        let claude35 = find_action(&actions, "select_model_claude-35").unwrap();

        assert!(
            !claude3.title.contains('✓'),
            "Claude 3 should NOT have checkmark when current is Claude 3.5"
        );
        assert!(
            claude35.title.contains('✓'),
            "Claude 3.5 should have checkmark"
        );
    }

    #[test]
    fn chat_model_description_includes_provider() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "gpt4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model = find_action(&actions, "select_model_gpt4").unwrap();
        assert!(
            model.description.as_ref().unwrap().contains("OpenAI"),
            "Model description should contain provider"
        );
    }

    // ============================================================
    // 7. AI command bar description keyword validation
    // ============================================================

    #[test]
    fn ai_command_bar_copy_response_desc() {
        let actions = get_ai_command_bar_actions();
        let action = find_action(&actions, "copy_response").unwrap();
        assert!(action
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("response"));
    }

    #[test]
    fn ai_command_bar_copy_chat_desc() {
        let actions = get_ai_command_bar_actions();
        let action = find_action(&actions, "copy_chat").unwrap();
        assert!(action
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("conversation"));
    }

    #[test]
    fn ai_command_bar_copy_last_code_desc() {
        let actions = get_ai_command_bar_actions();
        let action = find_action(&actions, "copy_last_code").unwrap();
        assert!(action
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("code"));
    }

    #[test]
    fn ai_command_bar_new_chat_desc() {
        let actions = get_ai_command_bar_actions();
        let action = find_action(&actions, "new_chat").unwrap();
        assert!(action
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("new"));
    }

    #[test]
    fn ai_command_bar_change_model_desc() {
        let actions = get_ai_command_bar_actions();
        let action = find_action(&actions, "change_model").unwrap();
        assert!(action
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("model"));
    }

    #[test]
    fn ai_command_bar_delete_chat_desc() {
        let actions = get_ai_command_bar_actions();
        let action = find_action(&actions, "delete_chat").unwrap();
        assert!(action
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("delete"));
    }

    #[test]
    fn ai_command_bar_submit_desc() {
        let actions = get_ai_command_bar_actions();
        let action = find_action(&actions, "submit").unwrap();
        assert!(action
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("send"));
    }

    // ============================================================
    // 8. Notes command bar section label transitions
    // ============================================================

    #[test]
    fn notes_full_feature_sections() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let sections: Vec<_> = actions.iter().filter_map(|a| a.section.as_ref()).collect();
        // Should have Notes, Edit, Copy, Export, Settings sections
        assert!(sections.contains(&&"Notes".to_string()));
        assert!(sections.contains(&&"Edit".to_string()));
        assert!(sections.contains(&&"Copy".to_string()));
        assert!(sections.contains(&&"Export".to_string()));
        assert!(sections.contains(&&"Settings".to_string()));
    }

    #[test]
    fn notes_minimal_sections() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let sections: Vec<_> = actions
            .iter()
            .filter_map(|a| a.section.as_ref())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        // Should only have Notes section (no selection means no Edit/Copy/Export/Settings)
        assert_eq!(
            sections.len(),
            1,
            "Minimal config should have 1 section, got {:?}",
            sections
        );
    }

    #[test]
    fn notes_trash_view_hides_edit_actions() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        // Trash view should hide editing actions even with selection
        assert!(!ids.contains(&"duplicate_note"));
        assert!(!ids.contains(&"find_in_note"));
        assert!(!ids.contains(&"format"));
        assert!(!ids.contains(&"copy_note_as"));
        assert!(!ids.contains(&"export"));
    }

    #[test]
    fn notes_all_actions_have_icons() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for action in &actions {
            assert!(
                action.icon.is_some(),
                "Notes action '{}' should have an icon",
                action.id
            );
        }
    }

    #[test]
    fn notes_all_actions_have_sections() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for action in &actions {
            assert!(
                action.section.is_some(),
                "Notes action '{}' should have a section",
                action.id
            );
        }
    }

    // ============================================================
    // 9. New chat duplicate providers
    // ============================================================

    #[test]
    fn new_chat_multiple_models_same_provider() {
        let models = vec![
            NewChatModelInfo {
                model_id: "claude-3".to_string(),
                display_name: "Claude 3".to_string(),
                provider: "anthropic".to_string(),
                provider_display_name: "Anthropic".to_string(),
            },
            NewChatModelInfo {
                model_id: "claude-35".to_string(),
                display_name: "Claude 3.5".to_string(),
                provider: "anthropic".to_string(),
                provider_display_name: "Anthropic".to_string(),
            },
        ];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions.len(), 2);
        // Both should have Models section
        assert_eq!(actions[0].section, Some("Models".to_string()));
        assert_eq!(actions[1].section, Some("Models".to_string()));
        // Both should show Anthropic as provider in description
        for action in &actions {
            assert_eq!(
                action.description,
                Some("Anthropic".to_string()),
                "Model action should have provider in description"
            );
        }
    }

    #[test]
    fn new_chat_empty_inputs_return_empty() {
        let actions = get_new_chat_actions(&[], &[], &[]);
        assert!(actions.is_empty());
    }

    #[test]
    fn new_chat_section_ordering() {
        let last_used = vec![NewChatModelInfo {
            model_id: "lu1".to_string(),
            display_name: "Last Used 1".to_string(),
            provider: "p".to_string(),
            provider_display_name: "Provider".to_string(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "general".to_string(),
            name: "General".to_string(),
            icon: IconName::BoltFilled,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m1".to_string(),
            display_name: "Model 1".to_string(),
            provider: "p".to_string(),
            provider_display_name: "Provider".to_string(),
        }];

        let actions = get_new_chat_actions(&last_used, &presets, &models);
        let lu_idx = actions
            .iter()
            .position(|a| a.section == Some("Last Used Settings".to_string()))
            .unwrap();
        let preset_idx = actions
            .iter()
            .position(|a| a.section == Some("Presets".to_string()))
            .unwrap();
        let model_idx = actions
            .iter()
            .position(|a| a.section == Some("Models".to_string()))
            .unwrap();
        assert!(lu_idx < preset_idx, "Last Used before Presets");
        assert!(preset_idx < model_idx, "Presets before Models");
    }

    #[test]
    fn new_chat_preset_has_no_description() {
        let presets = vec![NewChatPresetInfo {
            id: "code".to_string(),
            name: "Code".to_string(),
            icon: IconName::Code,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(
            actions[0].description, None,
            "Presets should have no description"
        );
    }

