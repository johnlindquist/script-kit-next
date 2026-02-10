    #[test]
    fn chat_current_model_gets_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".to_string()),
            available_models: vec![
                ChatModelInfo {
                    id: "gpt-4".to_string(),
                    display_name: "GPT-4".to_string(),
                    provider: "OpenAI".to_string(),
                },
                ChatModelInfo {
                    id: "claude-3".to_string(),
                    display_name: "Claude 3".to_string(),
                    provider: "Anthropic".to_string(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let gpt = find_action(&actions, "select_model_gpt-4").unwrap();
        assert!(gpt.title.contains('✓'), "Current model should have ✓");
        let claude = find_action(&actions, "select_model_claude-3").unwrap();
        assert!(
            !claude.title.contains('✓'),
            "Non-current model should not have ✓"
        );
    }

    #[test]
    fn chat_continue_always_present() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(
            action_ids(&actions).contains(&"continue_in_chat"),
            "continue_in_chat should always be present"
        );
    }

    #[test]
    fn chat_copy_response_requires_has_response() {
        let without = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let with = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: true,
        };
        let without_actions = get_chat_context_actions(&without);
        assert!(!action_ids(&without_actions).contains(&"copy_response"));
        let with_actions = get_chat_context_actions(&with);
        assert!(action_ids(&with_actions).contains(&"copy_response"));
    }

    #[test]
    fn chat_clear_requires_has_messages() {
        let without = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let with = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let without_actions = get_chat_context_actions(&without);
        assert!(!action_ids(&without_actions).contains(&"clear_conversation"));
        let with_actions = get_chat_context_actions(&with);
        assert!(action_ids(&with_actions).contains(&"clear_conversation"));
    }

    // ============================================================
    // 6. Notes command bar icon presence
    // ============================================================

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
    // 7. New chat action ordering within each section
    // ============================================================

    #[test]
    fn new_chat_sections_appear_in_order() {
        let last_used = vec![NewChatModelInfo {
            model_id: "gpt-4".to_string(),
            display_name: "GPT-4".to_string(),
            provider: "openai".to_string(),
            provider_display_name: "OpenAI".to_string(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "general".to_string(),
            name: "General".to_string(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "claude-3".to_string(),
            display_name: "Claude 3".to_string(),
            provider: "anthropic".to_string(),
            provider_display_name: "Anthropic".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);

        // Find first index of each section
        let first_last_used = actions
            .iter()
            .position(|a| a.section.as_deref() == Some("Last Used Settings"));
        let first_preset = actions
            .iter()
            .position(|a| a.section.as_deref() == Some("Presets"));
        let first_model = actions
            .iter()
            .position(|a| a.section.as_deref() == Some("Models"));

        assert!(first_last_used.unwrap() < first_preset.unwrap());
        assert!(first_preset.unwrap() < first_model.unwrap());
    }

    #[test]
    fn new_chat_last_used_has_bolt_icon() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".to_string(),
            display_name: "Model 1".to_string(),
            provider: "p1".to_string(),
            provider_display_name: "Provider 1".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
    }

    #[test]
    fn new_chat_preset_uses_custom_icon() {
        let presets = vec![NewChatPresetInfo {
            id: "code".to_string(),
            name: "Code".to_string(),
            icon: IconName::Code,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].icon, Some(IconName::Code));
    }

    #[test]
    fn new_chat_model_uses_settings_icon() {
        let models = vec![NewChatModelInfo {
            model_id: "m1".to_string(),
            display_name: "M1".to_string(),
            provider: "p".to_string(),
            provider_display_name: "P".to_string(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].icon, Some(IconName::Settings));
    }

    #[test]
    fn new_chat_empty_inputs_empty_output() {
        let actions = get_new_chat_actions(&[], &[], &[]);
        assert!(actions.is_empty());
    }

    #[test]
    fn new_chat_id_format_indexed() {
        let lu = vec![
            NewChatModelInfo {
                model_id: "a".to_string(),
                display_name: "A".to_string(),
                provider: "p".to_string(),
                provider_display_name: "P".to_string(),
            },
            NewChatModelInfo {
                model_id: "b".to_string(),
                display_name: "B".to_string(),
                provider: "p".to_string(),
                provider_display_name: "P".to_string(),
            },
        ];
        let actions = get_new_chat_actions(&lu, &[], &[]);
        assert_eq!(actions[0].id, "last_used_0");
        assert_eq!(actions[1].id, "last_used_1");
    }

    #[test]
    fn new_chat_preset_id_format() {
        let presets = vec![NewChatPresetInfo {
            id: "writer".to_string(),
            name: "Writer".to_string(),
            icon: IconName::File,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].id, "preset_writer");
    }

    #[test]
    fn new_chat_model_id_format_indexed() {
        let models = vec![
            NewChatModelInfo {
                model_id: "x".to_string(),
                display_name: "X".to_string(),
                provider: "p".to_string(),
                provider_display_name: "P".to_string(),
            },
            NewChatModelInfo {
                model_id: "y".to_string(),
                display_name: "Y".to_string(),
                provider: "p".to_string(),
                provider_display_name: "P".to_string(),
            },
        ];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].id, "model_0");
        assert_eq!(actions[1].id, "model_1");
    }

    #[test]
    fn new_chat_last_used_has_provider_description() {
        let lu = vec![NewChatModelInfo {
            model_id: "m".to_string(),
            display_name: "M".to_string(),
            provider: "p".to_string(),
            provider_display_name: "ProviderName".to_string(),
        }];
        let actions = get_new_chat_actions(&lu, &[], &[]);
        assert_eq!(actions[0].description, Some("ProviderName".to_string()));
    }

    #[test]
    fn new_chat_preset_has_no_description() {
        let presets = vec![NewChatPresetInfo {
            id: "x".to_string(),
            name: "X".to_string(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].description, None);
    }

    // ============================================================
    // 8. Agent actions exclude view_logs
    // ============================================================

    #[test]
    fn agent_has_edit_agent_title() {
        let mut agent = ScriptInfo::new("My Agent", "/path/agent.md");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        let edit = find_action(&actions, "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    #[test]
    fn agent_lacks_view_logs() {
        let mut agent = ScriptInfo::new("My Agent", "/path/agent.md");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        assert!(
            !action_ids(&actions).contains(&"view_logs"),
            "Agent should not have view_logs"
        );
    }

    #[test]
    fn agent_has_reveal_and_copy() {
        let mut agent = ScriptInfo::new("My Agent", "/path/agent.md");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"reveal_in_finder"));
        assert!(ids.contains(&"copy_path"));
        assert!(ids.contains(&"copy_content"));
    }

    // ============================================================
    // 9. Script vs scriptlet action set symmetric difference
    // ============================================================

    #[test]
    fn script_has_actions_scriptlet_lacks() {
        let script = ScriptInfo::new("s", "/path/s.ts");
        let scriptlet = ScriptInfo::scriptlet("s", "/path/s.md", None, None);
        let s_actions = get_script_context_actions(&script);
        let s_ids: HashSet<&str> = action_ids(&s_actions).into_iter().collect();
        let sl_actions = get_script_context_actions(&scriptlet);
        let sl_ids: HashSet<&str> = action_ids(&sl_actions).into_iter().collect();
        // Script should have these that scriptlet lacks
        assert!(s_ids.contains("edit_script"));
        assert!(s_ids.contains("view_logs"));
        assert!(!sl_ids.contains("edit_script"));
        assert!(!sl_ids.contains("view_logs"));
    }

    #[test]
    fn scriptlet_has_actions_script_lacks() {
        let script = ScriptInfo::new("s", "/path/s.ts");
        let scriptlet = ScriptInfo::scriptlet("s", "/path/s.md", None, None);
        let s_actions = get_script_context_actions(&script);
        let s_ids: HashSet<&str> = action_ids(&s_actions).into_iter().collect();
        let sl_actions = get_script_context_actions(&scriptlet);
        let sl_ids: HashSet<&str> = action_ids(&sl_actions).into_iter().collect();
        // Scriptlet should have these that script lacks
        assert!(sl_ids.contains("edit_scriptlet"));
        assert!(sl_ids.contains("reveal_scriptlet_in_finder"));
        assert!(sl_ids.contains("copy_scriptlet_path"));
        assert!(!s_ids.contains("edit_scriptlet"));
        assert!(!s_ids.contains("reveal_scriptlet_in_finder"));
        assert!(!s_ids.contains("copy_scriptlet_path"));
    }

    #[test]
    fn script_and_scriptlet_share_common_ids() {
        let script = ScriptInfo::new("s", "/path/s.ts");
        let scriptlet = ScriptInfo::scriptlet("s", "/path/s.md", None, None);
        let s_actions = get_script_context_actions(&script);
        let s_ids: HashSet<&str> = action_ids(&s_actions).into_iter().collect();
        let sl_actions = get_script_context_actions(&scriptlet);
        let sl_ids: HashSet<&str> = action_ids(&sl_actions).into_iter().collect();
        // Both should have these common actions
        let common = [
            "run_script",
            "copy_deeplink",
            "add_shortcut",
            "add_alias",
            "copy_content",
        ];
        for id in &common {
            assert!(s_ids.contains(id), "Script should have {}", id);
            assert!(sl_ids.contains(id), "Scriptlet should have {}", id);
        }
    }

    // ============================================================
    // 10. Deeplink URL in description format
    // ============================================================

    #[test]
    fn deeplink_description_contains_url() {
        let script = ScriptInfo::new("My Cool Script", "/path/script.ts");
        let actions = get_script_context_actions(&script);
        let dl = find_action(&actions, "copy_deeplink").unwrap();
        let desc = dl.description.as_ref().unwrap();
        assert!(desc.contains("scriptkit://run/my-cool-script"));
    }

    #[test]
    fn deeplink_description_special_chars() {
        let script = ScriptInfo::new("Test!@#$Script", "/path/script.ts");
        let actions = get_script_context_actions(&script);
        let dl = find_action(&actions, "copy_deeplink").unwrap();
        let desc = dl.description.as_ref().unwrap();
        assert!(desc.contains("scriptkit://run/test-script"));
    }

    #[test]
    fn deeplink_scriptlet_context() {
        let script = ScriptInfo::scriptlet("Open GitHub", "/path.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let dl = find_action(&actions, "copy_deeplink").unwrap();
        let desc = dl.description.as_ref().unwrap();
        assert!(desc.contains("scriptkit://run/open-github"));
    }

    // ============================================================
    // 11. AI command bar shortcut uniqueness
    // ============================================================

    #[test]
    fn ai_command_bar_shortcuts_unique() {
        let actions = get_ai_command_bar_actions();
        let shortcuts: Vec<&str> = actions
            .iter()
            .filter_map(|a| a.shortcut.as_deref())
            .collect();
        let unique: HashSet<&str> = shortcuts.iter().copied().collect();
        assert_eq!(
            shortcuts.len(),
            unique.len(),
            "AI command bar shortcuts should be unique: {:?}",
            shortcuts
        );
    }

    #[test]
    fn ai_command_bar_exactly_12_actions() {
        let actions = get_ai_command_bar_actions();
        assert_eq!(actions.len(), 12);
    }

    #[test]
    fn ai_command_bar_all_have_icons() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                action.icon.is_some(),
                "AI action '{}' should have icon",
                action.id
            );
        }
    }

