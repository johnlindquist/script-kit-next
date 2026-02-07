    #[test]
    fn script_without_alias_has_add_alias() {
        let info = ScriptInfo::new("my-script", "/s.ts");
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "add_alias"));
        assert!(!actions.iter().any(|a| a.id == "update_alias"));
        assert!(!actions.iter().any(|a| a.id == "remove_alias"));
    }

    // =========================================================================
    // 19. Script context: agent has edit_script with "Edit Agent" title, desc mentions agent
    // =========================================================================

    #[test]
    fn agent_edit_title_is_edit_agent() {
        let mut info = ScriptInfo::new("my-agent", "/a.md");
        info.is_agent = true;
        info.is_script = false;
        let actions = get_script_context_actions(&info);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    #[test]
    fn agent_edit_desc_mentions_agent() {
        let mut info = ScriptInfo::new("my-agent", "/a.md");
        info.is_agent = true;
        info.is_script = false;
        let actions = get_script_context_actions(&info);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert!(edit
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("agent"));
    }

    #[test]
    fn agent_has_reveal_in_finder() {
        let mut info = ScriptInfo::new("my-agent", "/a.md");
        info.is_agent = true;
        info.is_script = false;
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
    }

    #[test]
    fn agent_reveal_desc_mentions_agent() {
        let mut info = ScriptInfo::new("my-agent", "/a.md");
        info.is_agent = true;
        info.is_script = false;
        let actions = get_script_context_actions(&info);
        let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
        assert!(reveal
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("agent"));
    }

    // =========================================================================
    // 20. Script context: total action count varies by type
    // =========================================================================

    #[test]
    fn script_context_real_script_count() {
        // Real script: run + add_shortcut + add_alias + edit + view_logs + reveal + copy_path + copy_content + copy_deeplink = 9
        let info = ScriptInfo::new("test", "/test.ts");
        let actions = get_script_context_actions(&info);
        assert_eq!(actions.len(), 9);
    }

    #[test]
    fn script_context_builtin_count() {
        // Builtin: run + add_shortcut + add_alias + copy_deeplink = 4
        let info = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&info);
        assert_eq!(actions.len(), 4);
    }

    #[test]
    fn script_context_agent_count() {
        // Agent: run + add_shortcut + add_alias + edit + reveal + copy_path + copy_content + copy_deeplink = 8
        let mut info = ScriptInfo::new("my-agent", "/a.md");
        info.is_agent = true;
        info.is_script = false;
        let actions = get_script_context_actions(&info);
        assert_eq!(actions.len(), 8);
    }

    #[test]
    fn script_context_scriptlet_count() {
        // Scriptlet: run + add_shortcut + add_alias + edit_scriptlet + reveal + copy_path + copy_content + copy_deeplink = 8
        let info = ScriptInfo::scriptlet("Test Scriptlet", "/t.md", None, None);
        let actions = get_script_context_actions(&info);
        assert_eq!(actions.len(), 8);
    }

    // =========================================================================
    // 21. Scriptlet context: add_shortcut when no shortcut, add_alias when no alias
    // =========================================================================

    #[test]
    fn scriptlet_with_custom_no_shortcut_has_add_shortcut() {
        let script = ScriptInfo::scriptlet("Test", "/t.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(actions.iter().any(|a| a.id == "add_shortcut"));
        assert!(!actions.iter().any(|a| a.id == "update_shortcut"));
    }

    #[test]
    fn scriptlet_with_custom_has_shortcut_shows_update_remove() {
        let script = ScriptInfo::scriptlet("Test", "/t.md", Some("cmd+t".into()), None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(actions.iter().any(|a| a.id == "update_shortcut"));
        assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
        assert!(!actions.iter().any(|a| a.id == "add_shortcut"));
    }

    #[test]
    fn scriptlet_with_custom_no_alias_has_add_alias() {
        let script = ScriptInfo::scriptlet("Test", "/t.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(actions.iter().any(|a| a.id == "add_alias"));
        assert!(!actions.iter().any(|a| a.id == "update_alias"));
    }

    #[test]
    fn scriptlet_with_custom_has_alias_shows_update_remove() {
        let script = ScriptInfo::scriptlet("Test", "/t.md", None, Some("tst".into()));
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(actions.iter().any(|a| a.id == "update_alias"));
        assert!(actions.iter().any(|a| a.id == "remove_alias"));
        assert!(!actions.iter().any(|a| a.id == "add_alias"));
    }

    // =========================================================================
    // 22. Scriptlet context: reset_ranking only when is_suggested
    // =========================================================================

    #[test]
    fn scriptlet_with_custom_suggested_has_reset_ranking() {
        let script = ScriptInfo::scriptlet("Test", "/t.md", None, None)
            .with_frecency(true, Some("/t.md".into()));
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    }

    #[test]
    fn scriptlet_with_custom_not_suggested_no_reset_ranking() {
        let script = ScriptInfo::scriptlet("Test", "/t.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(!actions.iter().any(|a| a.id == "reset_ranking"));
    }

    #[test]
    fn scriptlet_with_custom_reset_ranking_is_last() {
        let script = ScriptInfo::scriptlet("Test", "/t.md", None, None)
            .with_frecency(true, Some("/t.md".into()));
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert_eq!(actions.last().unwrap().id, "reset_ranking");
    }

    #[test]
    fn scriptlet_with_custom_reset_ranking_has_no_shortcut() {
        let script = ScriptInfo::scriptlet("Test", "/t.md", None, None)
            .with_frecency(true, Some("/t.md".into()));
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let reset = actions.iter().find(|a| a.id == "reset_ranking").unwrap();
        assert!(reset.shortcut.is_none());
    }

    // =========================================================================
    // 23. AI bar: delete_chat shortcut and icon
    // =========================================================================

    #[test]
    fn ai_bar_delete_chat_shortcut() {
        let actions = get_ai_command_bar_actions();
        let delete = actions.iter().find(|a| a.id == "delete_chat").unwrap();
        assert_eq!(delete.shortcut.as_deref(), Some("⌘⌫"));
    }

    #[test]
    fn ai_bar_delete_chat_icon() {
        let actions = get_ai_command_bar_actions();
        let delete = actions.iter().find(|a| a.id == "delete_chat").unwrap();
        assert_eq!(delete.icon, Some(IconName::Trash));
    }

    #[test]
    fn ai_bar_delete_chat_section() {
        let actions = get_ai_command_bar_actions();
        let delete = actions.iter().find(|a| a.id == "delete_chat").unwrap();
        assert_eq!(delete.section.as_deref(), Some("Actions"));
    }

    #[test]
    fn ai_bar_delete_chat_desc_mentions_delete() {
        let actions = get_ai_command_bar_actions();
        let delete = actions.iter().find(|a| a.id == "delete_chat").unwrap();
        assert!(delete
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("delete"));
    }

    // =========================================================================
    // 24. AI bar: new_chat shortcut and icon
    // =========================================================================

    #[test]
    fn ai_bar_new_chat_shortcut() {
        let actions = get_ai_command_bar_actions();
        let nc = actions.iter().find(|a| a.id == "new_chat").unwrap();
        assert_eq!(nc.shortcut.as_deref(), Some("⌘N"));
    }

    #[test]
    fn ai_bar_new_chat_icon() {
        let actions = get_ai_command_bar_actions();
        let nc = actions.iter().find(|a| a.id == "new_chat").unwrap();
        assert_eq!(nc.icon, Some(IconName::Plus));
    }

    #[test]
    fn ai_bar_new_chat_section() {
        let actions = get_ai_command_bar_actions();
        let nc = actions.iter().find(|a| a.id == "new_chat").unwrap();
        assert_eq!(nc.section.as_deref(), Some("Actions"));
    }

    #[test]
    fn ai_bar_new_chat_desc_mentions_conversation() {
        let actions = get_ai_command_bar_actions();
        let nc = actions.iter().find(|a| a.id == "new_chat").unwrap();
        assert!(nc
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("conversation"));
    }

    // =========================================================================
    // 25. Notes: format action details
    // =========================================================================

    #[test]
    fn notes_format_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let format = actions.iter().find(|a| a.id == "format").unwrap();
        assert_eq!(format.shortcut.as_deref(), Some("⇧⌘T"));
    }

    #[test]
    fn notes_format_icon_code() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let format = actions.iter().find(|a| a.id == "format").unwrap();
        assert_eq!(format.icon, Some(IconName::Code));
    }

    #[test]
    fn notes_format_section_edit() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let format = actions.iter().find(|a| a.id == "format").unwrap();
        assert_eq!(format.section.as_deref(), Some("Edit"));
    }

    #[test]
    fn notes_format_absent_without_selection() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "format"));
    }

    // =========================================================================
    // 26. Notes: selection+trash yields subset of actions
    // =========================================================================

    #[test]
    fn notes_trash_view_has_new_note() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "new_note"));
    }

    #[test]
    fn notes_trash_view_no_duplicate() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
    }

    #[test]
    fn notes_trash_view_no_find_in_note() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "find_in_note"));
    }

    #[test]
    fn notes_trash_view_no_export() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "export"));
    }

    // =========================================================================
    // 27. Chat context: model with current_model gets checkmark
    // =========================================================================

    #[test]
    fn chat_current_model_has_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".to_string()),
            available_models: vec![ChatModelInfo {
                id: "gpt4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = actions
            .iter()
            .find(|a| a.id == "select_model_gpt4")
            .unwrap();
        assert!(model_action.title.contains("✓"));
    }

    #[test]
    fn chat_non_current_model_no_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".to_string()),
            available_models: vec![ChatModelInfo {
                id: "gpt4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
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
    fn chat_no_current_model_no_checkmark() {
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
                id: "claude".to_string(),
                display_name: "Claude".to_string(),
                provider: "Anthropic".to_string(),
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

    // =========================================================================
    // 28. Chat context: multiple models ordering
    // =========================================================================

