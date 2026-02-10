    #[test]
    fn parse_keycaps_space_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("␣");
        assert_eq!(keycaps, vec!["␣"]);
    }

    #[test]
    fn parse_keycaps_all_modifiers() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⌃⌥⇧K");
        assert_eq!(keycaps, vec!["⌘", "⌃", "⌥", "⇧", "K"]);
    }

    #[test]
    fn parse_keycaps_arrow_keys() {
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("↑"), vec!["↑"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("↓"), vec!["↓"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("←"), vec!["←"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("→"), vec!["→"]);
    }

    // =========================================================================
    // 20. Agent-specific action validation
    // =========================================================================

    #[test]
    fn agent_has_edit_agent_title() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    #[test]
    fn agent_has_no_view_logs() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    #[test]
    fn agent_has_reveal_and_copy() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"reveal_in_finder"));
        assert!(ids.contains(&"copy_path"));
        assert!(ids.contains(&"copy_content"));
    }

    #[test]
    fn agent_edit_description_mentions_agent() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        let desc = edit.description.as_ref().unwrap().to_lowercase();
        assert!(desc.contains("agent"));
    }

    // =========================================================================
    // 21. New chat action details
    // =========================================================================

    #[test]
    fn new_chat_last_used_icon_is_bolt() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "Model 1".into(),
            provider: "p".into(),
            provider_display_name: "Provider".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
    }

    #[test]
    fn new_chat_preset_icon_matches_input() {
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].icon, Some(IconName::Star));
    }

    #[test]
    fn new_chat_model_icon_is_settings() {
        let models = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "Model 1".into(),
            provider: "p".into(),
            provider_display_name: "Provider".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].icon, Some(IconName::Settings));
    }

    #[test]
    fn new_chat_presets_have_no_description() {
        let presets = vec![NewChatPresetInfo {
            id: "code".into(),
            name: "Code".into(),
            icon: IconName::Code,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert!(actions[0].description.is_none());
    }

    #[test]
    fn new_chat_models_have_provider_description() {
        let models = vec![NewChatModelInfo {
            model_id: "claude".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].description.as_deref(), Some("Anthropic"));
    }

    #[test]
    fn new_chat_empty_all_returns_empty() {
        let actions = get_new_chat_actions(&[], &[], &[]);
        assert!(actions.is_empty());
    }

    // =========================================================================
    // 22. Action constructor edge cases
    // =========================================================================

    #[test]
    fn action_with_shortcut_opt_none_leaves_none() {
        let action =
            Action::new("t", "Test", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
        assert!(action.shortcut.is_none());
        assert!(action.shortcut_lower.is_none());
    }

    #[test]
    fn action_with_shortcut_opt_some_sets_both() {
        let action = Action::new("t", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘K".into()));
        assert_eq!(action.shortcut.as_deref(), Some("⌘K"));
        assert_eq!(action.shortcut_lower.as_deref(), Some("⌘k"));
    }

    #[test]
    fn action_title_lower_computed_on_creation() {
        let action = Action::new(
            "t",
            "My UPPERCASE Title",
            None,
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.title_lower, "my uppercase title");
    }

    #[test]
    fn action_description_lower_computed_on_creation() {
        let action = Action::new(
            "t",
            "T",
            Some("Description With CAPS".into()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(
            action.description_lower.as_deref(),
            Some("description with caps")
        );
    }

    #[test]
    fn action_no_description_has_none_lower() {
        let action = Action::new("t", "T", None, ActionCategory::ScriptContext);
        assert!(action.description_lower.is_none());
    }

    #[test]
    fn action_default_has_action_false() {
        let action = Action::new("t", "T", None, ActionCategory::ScriptContext);
        assert!(!action.has_action);
    }

    #[test]
    fn action_default_value_none() {
        let action = Action::new("t", "T", None, ActionCategory::ScriptContext);
        assert!(action.value.is_none());
    }

    #[test]
    fn action_default_icon_none() {
        let action = Action::new("t", "T", None, ActionCategory::ScriptContext);
        assert!(action.icon.is_none());
    }

    #[test]
    fn action_default_section_none() {
        let action = Action::new("t", "T", None, ActionCategory::ScriptContext);
        assert!(action.section.is_none());
    }

    #[test]
    fn action_with_icon_sets_icon() {
        let action =
            Action::new("t", "T", None, ActionCategory::ScriptContext).with_icon(IconName::Plus);
        assert_eq!(action.icon, Some(IconName::Plus));
    }

    #[test]
    fn action_with_section_sets_section() {
        let action =
            Action::new("t", "T", None, ActionCategory::ScriptContext).with_section("MySection");
        assert_eq!(action.section.as_deref(), Some("MySection"));
    }

    // =========================================================================
    // 23. ScriptInfo constructor validation
    // =========================================================================

    #[test]
    fn script_info_new_defaults() {
        let s = ScriptInfo::new("test", "/path");
        assert!(s.is_script);
        assert!(!s.is_scriptlet);
        assert!(!s.is_agent);
        assert_eq!(s.action_verb, "Run");
        assert!(s.shortcut.is_none());
        assert!(s.alias.is_none());
        assert!(!s.is_suggested);
        assert!(s.frecency_path.is_none());
    }

    #[test]
    fn script_info_builtin_has_empty_path() {
        let s = ScriptInfo::builtin("Test");
        assert!(s.path.is_empty());
        assert!(!s.is_script);
        assert!(!s.is_scriptlet);
        assert!(!s.is_agent);
    }

    #[test]
    fn script_info_scriptlet_sets_flags() {
        let s = ScriptInfo::scriptlet("Test", "/path.md", None, None);
        assert!(!s.is_script);
        assert!(s.is_scriptlet);
        assert!(!s.is_agent);
    }

    #[test]
    fn script_info_with_frecency_chaining() {
        let s = ScriptInfo::new("t", "/p").with_frecency(true, Some("/p".into()));
        assert!(s.is_suggested);
        assert_eq!(s.frecency_path.as_deref(), Some("/p"));
        // Original fields preserved
        assert!(s.is_script);
        assert_eq!(s.name, "t");
    }

    // =========================================================================
    // 24. Global actions always empty
    // =========================================================================

    #[test]
    fn global_actions_empty() {
        assert!(get_global_actions().is_empty());
    }

    // =========================================================================
    // 25. Ordering determinism (calling twice yields same result)
    // =========================================================================

    #[test]
    fn script_actions_deterministic() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions_1 = get_script_context_actions(&script);
        let actions_2 = get_script_context_actions(&script);
        let a1 = action_ids(&actions_1);
        let a2 = action_ids(&actions_2);
        assert_eq!(a1, a2);
    }

    #[test]
    fn clipboard_actions_deterministic() {
        let entry = ClipboardEntryInfo {
            id: "e".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "t".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions_1 = get_clipboard_history_context_actions(&entry);
        let actions_2 = get_clipboard_history_context_actions(&entry);
        let a1 = action_ids(&actions_1);
        let a2 = action_ids(&actions_2);
        assert_eq!(a1, a2);
    }

    #[test]
    fn notes_actions_deterministic() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions_1 = get_notes_command_bar_actions(&info);
        let actions_2 = get_notes_command_bar_actions(&info);
        let a1 = action_ids(&actions_1);
        let a2 = action_ids(&actions_2);
        assert_eq!(a1, a2);
    }

    #[test]
    fn ai_actions_deterministic() {
        let actions_1 = get_ai_command_bar_actions();
        let actions_2 = get_ai_command_bar_actions();
        let a1 = action_ids(&actions_1);
        let a2 = action_ids(&actions_2);
        assert_eq!(a1, a2);
    }
