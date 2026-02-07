    #[test]
    fn cat25_multiple_last_used_single_preset_single_model() {
        let last_used = vec![
            NewChatModelInfo {
                model_id: "m1".into(),
                display_name: "M1".into(),
                provider: "p1".into(),
                provider_display_name: "P1".into(),
            },
            NewChatModelInfo {
                model_id: "m2".into(),
                display_name: "M2".into(),
                provider: "p2".into(),
                provider_display_name: "P2".into(),
            },
        ];
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m3".into(),
            display_name: "M3".into(),
            provider: "p3".into(),
            provider_display_name: "P3".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        assert_eq!(actions.len(), 4); // 2 + 1 + 1
    }

    #[test]
    fn cat25_sections_are_correct() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "M1".into(),
            provider: "p1".into(),
            provider_display_name: "P1".into(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "gen".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m2".into(),
            display_name: "M2".into(),
            provider: "p2".into(),
            provider_display_name: "P2".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        let sections: Vec<&str> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        assert_eq!(sections, vec!["Last Used Settings", "Presets", "Models"]);
    }

    #[test]
    fn cat25_preset_has_no_description() {
        let presets = vec![NewChatPresetInfo {
            id: "gen".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert!(actions[0].description.is_none());
    }

    #[test]
    fn cat25_model_has_provider_description() {
        let models = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "M1".into(),
            provider: "p1".into(),
            provider_display_name: "Provider One".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].description.as_deref(), Some("Provider One"));
    }

    // ================================================================
    // Cat 26: Notes command bar browse_notes action details
    // ================================================================

    #[test]
    fn cat26_browse_notes_always_present() {
        let full = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let minimal = NotesInfo {
            has_selection: false,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let full_actions = get_notes_command_bar_actions(&full);
        let minimal_actions = get_notes_command_bar_actions(&minimal);
        assert!(full_actions.iter().any(|a| a.id == "browse_notes"));
        assert!(minimal_actions.iter().any(|a| a.id == "browse_notes"));
    }

    #[test]
    fn cat26_browse_notes_shortcut() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let bn = actions.iter().find(|a| a.id == "browse_notes").unwrap();
        assert_eq!(bn.shortcut.as_deref(), Some("⌘P"));
    }

    #[test]
    fn cat26_browse_notes_icon() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let bn = actions.iter().find(|a| a.id == "browse_notes").unwrap();
        assert_eq!(bn.icon, Some(IconName::FolderOpen));
    }

    #[test]
    fn cat26_browse_notes_section() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let bn = actions.iter().find(|a| a.id == "browse_notes").unwrap();
        assert_eq!(bn.section.as_deref(), Some("Notes"));
    }

    // ================================================================
    // Cat 27: Script context copy_deeplink description contains URL
    // ================================================================

    #[test]
    fn cat27_deeplink_description_contains_scriptkit_url() {
        let script = ScriptInfo::new("My Cool Script", "/path/script.ts");
        let actions = get_script_context_actions(&script);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert!(dl
            .description
            .as_ref()
            .unwrap()
            .contains("scriptkit://run/"));
    }

    #[test]
    fn cat27_deeplink_description_contains_deeplink_name() {
        let script = ScriptInfo::new("My Cool Script", "/path/script.ts");
        let actions = get_script_context_actions(&script);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert!(dl.description.as_ref().unwrap().contains("my-cool-script"));
    }

    #[test]
    fn cat27_deeplink_shortcut() {
        let script = ScriptInfo::new("Test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert_eq!(dl.shortcut.as_deref(), Some("⌘⇧D"));
    }

    #[test]
    fn cat27_scriptlet_deeplink_also_has_url() {
        let script = ScriptInfo::scriptlet("Open GitHub", "/path/url.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert!(dl
            .description
            .as_ref()
            .unwrap()
            .contains("scriptkit://run/open-github"));
    }

    // ================================================================
    // Cat 28: Cross-context all actions are ScriptContext category
    // ================================================================

    #[test]
    fn cat28_script_actions_all_script_context() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        for action in &get_script_context_actions(&script) {
            assert_eq!(
                action.category,
                ActionCategory::ScriptContext,
                "ID: {}",
                action.id
            );
        }
    }

    #[test]
    fn cat28_clipboard_actions_all_script_context() {
        let entry = ClipboardEntryInfo {
            id: "e".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "t".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        for action in &get_clipboard_history_context_actions(&entry) {
            assert_eq!(
                action.category,
                ActionCategory::ScriptContext,
                "ID: {}",
                action.id
            );
        }
    }

    #[test]
    fn cat28_ai_actions_all_script_context() {
        for action in &get_ai_command_bar_actions() {
            assert_eq!(
                action.category,
                ActionCategory::ScriptContext,
                "ID: {}",
                action.id
            );
        }
    }

    #[test]
    fn cat28_path_actions_all_script_context() {
        let info = PathInfo {
            path: "/a".into(),
            name: "a".into(),
            is_dir: true,
        };
        for action in &get_path_context_actions(&info) {
            assert_eq!(
                action.category,
                ActionCategory::ScriptContext,
                "ID: {}",
                action.id
            );
        }
    }

    #[test]
    fn cat28_file_actions_all_script_context() {
        let fi = FileInfo {
            path: "/test/f.txt".into(),
            name: "f.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        for action in &get_file_context_actions(&fi) {
            assert_eq!(
                action.category,
                ActionCategory::ScriptContext,
                "ID: {}",
                action.id
            );
        }
    }

    #[test]
    fn cat28_notes_actions_all_script_context() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for action in &get_notes_command_bar_actions(&info) {
            assert_eq!(
                action.category,
                ActionCategory::ScriptContext,
                "ID: {}",
                action.id
            );
        }
    }

    // ================================================================
    // Cat 29: Action with_icon chaining preserves all fields
    // ================================================================

    #[test]
    fn cat29_with_icon_preserves_shortcut() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘E")
            .with_icon(IconName::Star);
        assert_eq!(action.shortcut.as_deref(), Some("⌘E"));
        assert_eq!(action.icon, Some(IconName::Star));
    }

    #[test]
    fn cat29_with_section_preserves_icon() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_icon(IconName::Copy)
            .with_section("S");
        assert_eq!(action.icon, Some(IconName::Copy));
        assert_eq!(action.section.as_deref(), Some("S"));
    }

    #[test]
    fn cat29_full_chain_all_fields() {
        let action = Action::new(
            "id",
            "Title",
            Some("Desc".into()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘X")
        .with_icon(IconName::Trash)
        .with_section("Section");
        assert_eq!(action.id, "id");
        assert_eq!(action.title, "Title");
        assert_eq!(action.description.as_deref(), Some("Desc"));
        assert_eq!(action.shortcut.as_deref(), Some("⌘X"));
        assert_eq!(action.icon, Some(IconName::Trash));
        assert_eq!(action.section.as_deref(), Some("Section"));
        assert!(!action.has_action);
    }

    #[test]
    fn cat29_with_shortcut_opt_none_preserves_existing() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘A")
            .with_shortcut_opt(None);
        // with_shortcut_opt(None) does NOT clear existing shortcut
        assert_eq!(action.shortcut.as_deref(), Some("⌘A"));
    }

    #[test]
    fn cat29_with_shortcut_opt_some_sets() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘B".into()));
        assert_eq!(action.shortcut.as_deref(), Some("⌘B"));
    }

    // ================================================================
    // Cat 30: Script context action stability across flag combinations
    // ================================================================

    #[test]
    fn cat30_script_deterministic() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let a1 = action_ids(&get_script_context_actions(&script));
        let a2 = action_ids(&get_script_context_actions(&script));
        assert_eq!(a1, a2);
    }

    #[test]
    fn cat30_builtin_deterministic() {
        let builtin = ScriptInfo::builtin("Test");
        let a1 = action_ids(&get_script_context_actions(&builtin));
        let a2 = action_ids(&get_script_context_actions(&builtin));
        assert_eq!(a1, a2);
    }

    #[test]
    fn cat30_scriptlet_deterministic() {
        let scriptlet = ScriptInfo::scriptlet("Test", "/path.md", None, None);
        let a1 = action_ids(&get_script_context_actions(&scriptlet));
        let a2 = action_ids(&get_script_context_actions(&scriptlet));
        assert_eq!(a1, a2);
    }

    #[test]
    fn cat30_agent_deterministic() {
        let mut agent = ScriptInfo::new("Agent", "/path/agent.md");
        agent.is_script = false;
        agent.is_agent = true;
        let a1 = action_ids(&get_script_context_actions(&agent));
        let a2 = action_ids(&get_script_context_actions(&agent));
        assert_eq!(a1, a2);
    }

    #[test]
    fn cat30_frecency_flag_adds_exactly_one_action() {
        let base = ScriptInfo::new("test", "/path/test.ts");
        let with_frecency =
            ScriptInfo::new("test", "/path/test.ts").with_frecency(true, Some("/path".into()));
        let base_count = get_script_context_actions(&base).len();
        let frecency_count = get_script_context_actions(&with_frecency).len();
        assert_eq!(frecency_count, base_count + 1);
    }
