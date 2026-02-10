    #[test]
    fn command_bar_ai_style_close_flags_default() {
        let config = CommandBarConfig::ai_style();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    #[test]
    fn command_bar_main_menu_search_bottom() {
        let config = CommandBarConfig::main_menu_style();
        assert_eq!(
            config.dialog_config.search_position,
            crate::actions::types::SearchPosition::Bottom
        );
    }

    #[test]
    fn command_bar_ai_style_search_top() {
        let config = CommandBarConfig::ai_style();
        assert_eq!(
            config.dialog_config.search_position,
            crate::actions::types::SearchPosition::Top
        );
    }

    #[test]
    fn command_bar_no_search_hidden() {
        let config = CommandBarConfig::no_search();
        assert_eq!(
            config.dialog_config.search_position,
            crate::actions::types::SearchPosition::Hidden
        );
    }

    #[test]
    fn command_bar_notes_style_search_top_icons() {
        let config = CommandBarConfig::notes_style();
        assert_eq!(
            config.dialog_config.search_position,
            crate::actions::types::SearchPosition::Top
        );
        assert!(config.dialog_config.show_icons);
        assert!(config.dialog_config.show_footer);
    }

    // ============================================================
    // 21. Action constructor with empty strings
    // ============================================================

    #[test]
    fn action_empty_id_and_title() {
        let action = Action::new("", "", None, ActionCategory::ScriptContext);
        assert_eq!(action.id, "");
        assert_eq!(action.title, "");
        assert_eq!(action.title_lower, "");
        assert!(action.description.is_none());
        assert!(action.description_lower.is_none());
    }

    #[test]
    fn action_with_shortcut_sets_lower() {
        let action =
            Action::new("x", "X", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
        assert_eq!(action.shortcut_lower, Some("⌘⇧c".to_string()));
    }

    #[test]
    fn action_with_shortcut_opt_none_no_lower() {
        let action =
            Action::new("x", "X", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
        assert!(action.shortcut.is_none());
        assert!(action.shortcut_lower.is_none());
    }

    #[test]
    fn action_with_shortcut_opt_some_sets_lower() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘Z".to_string()));
        assert_eq!(action.shortcut, Some("⌘Z".to_string()));
        assert_eq!(action.shortcut_lower, Some("⌘z".to_string()));
    }

    // ============================================================
    // 22. ScriptInfo flag exclusivity
    // ============================================================

    #[test]
    fn script_info_scriptlet_is_not_script() {
        let scriptlet = ScriptInfo::scriptlet("X", "/p.md", None, None);
        assert!(scriptlet.is_scriptlet);
        assert!(!scriptlet.is_script);
        assert!(!scriptlet.is_agent);
    }

    #[test]
    fn script_info_agent_is_not_scriptlet() {
        let mut agent = ScriptInfo::new("A", "/a.md");
        agent.is_script = false;
        agent.is_agent = true;
        assert!(agent.is_agent);
        assert!(!agent.is_scriptlet);
        assert!(!agent.is_script);
    }

    #[test]
    fn script_info_builtin_is_none_of_the_above() {
        let builtin = ScriptInfo::builtin("Clipboard");
        assert!(!builtin.is_script);
        assert!(!builtin.is_scriptlet);
        assert!(!builtin.is_agent);
    }

    // ============================================================
    // 23. Notes command bar action count bounds per flag state
    // ============================================================

    #[test]
    fn notes_minimal_count() {
        // No selection, no auto-sizing disabled → only new_note + browse_notes + enable_auto_sizing
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        // new_note + browse_notes + enable_auto_sizing = 3
        assert_eq!(
            actions.len(),
            3,
            "Minimal notes actions: {:?}",
            action_ids(&actions)
        );
    }

    #[test]
    fn notes_minimal_auto_sizing_enabled() {
        // No selection, auto-sizing already enabled
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        // new_note + browse_notes = 2
        assert_eq!(
            actions.len(),
            2,
            "Minimal with auto: {:?}",
            action_ids(&actions)
        );
    }

    #[test]
    fn notes_full_feature_count() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        // new_note + duplicate + browse_notes + find + format + copy_note_as + copy_deeplink
        // + create_quicklink + export + enable_auto_sizing = 10
        assert_eq!(
            actions.len(),
            10,
            "Full feature: {:?}",
            action_ids(&actions)
        );
    }

    #[test]
    fn notes_trash_hides_editing() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        assert!(!ids.contains(&"duplicate_note"));
        assert!(!ids.contains(&"find_in_note"));
        assert!(!ids.contains(&"format"));
        assert!(!ids.contains(&"copy_note_as"));
        assert!(!ids.contains(&"export"));
    }

    // ============================================================
    // 24. Chat model display_name in title
    // ============================================================

    #[test]
    fn chat_model_display_name_in_title() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "model-x".to_string(),
                display_name: "Model X Ultra".to_string(),
                provider: "Acme".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = find_action(&actions, "select_model_model-x").unwrap();
        assert_eq!(model_action.title, "Model X Ultra");
    }

    #[test]
    fn chat_model_provider_in_description() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "m".to_string(),
                display_name: "M".to_string(),
                provider: "Acme Corp".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = find_action(&actions, "select_model_m").unwrap();
        assert_eq!(model_action.description, Some("via Acme Corp".to_string()));
    }

    // ============================================================
    // 25. New chat model_id in action ID
    // ============================================================

    #[test]
    fn new_chat_model_section_name() {
        let models = vec![NewChatModelInfo {
            model_id: "abc-123".to_string(),
            display_name: "ABC 123".to_string(),
            provider: "p".to_string(),
            provider_display_name: "P".to_string(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].section.as_deref(), Some("Models"));
    }

    // ============================================================
    // 26. Clipboard delete_all description mentions "pinned"
    // ============================================================

    #[test]
    fn clipboard_delete_all_mentions_pinned() {
        let entry = make_text_entry(false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let delete_all = find_action(&actions, "clipboard_delete_all").unwrap();
        assert!(
            delete_all
                .description
                .as_ref()
                .unwrap()
                .to_lowercase()
                .contains("pinned"),
            "delete_all description should mention pinned: {:?}",
            delete_all.description
        );
    }

    // ============================================================
    // 27. File context all actions have ScriptContext category
    // ============================================================

    #[test]
    fn file_all_script_context_category() {
        let file_info = FileInfo {
            path: "/test/file.rs".to_string(),
            name: "file.rs".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        for action in &actions {
            assert_eq!(
                action.category,
                ActionCategory::ScriptContext,
                "File action '{}' should be ScriptContext",
                action.id
            );
        }
    }

    #[test]
    fn file_dir_all_script_context_category() {
        let file_info = FileInfo {
            path: "/test/dir".to_string(),
            name: "dir".to_string(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&file_info);
        for action in &actions {
            assert_eq!(
                action.category,
                ActionCategory::ScriptContext,
                "File dir action '{}' should be ScriptContext",
                action.id
            );
        }
    }

    // ============================================================
    // 28. Path context copy_path and copy_filename always present
    // ============================================================

    #[test]
    fn path_always_has_copy_path_and_filename() {
        for is_dir in [true, false] {
            let path = PathInfo {
                path: "/test/item".to_string(),
                name: "item".to_string(),
                is_dir,
            };
            let actions = get_path_context_actions(&path);
            let ids = action_ids(&actions);
            assert!(
                ids.contains(&"copy_path"),
                "Path (is_dir={}) should have copy_path",
                is_dir
            );
            assert!(
                ids.contains(&"copy_filename"),
                "Path (is_dir={}) should have copy_filename",
                is_dir
            );
        }
    }

    #[test]
    fn path_always_has_open_in_finder_editor_terminal() {
        for is_dir in [true, false] {
            let path = PathInfo {
                path: "/test/x".to_string(),
                name: "x".to_string(),
                is_dir,
            };
            let actions = get_path_context_actions(&path);
            let ids = action_ids(&actions);
            assert!(ids.contains(&"open_in_finder"));
            assert!(ids.contains(&"open_in_editor"));
            assert!(ids.contains(&"open_in_terminal"));
        }
    }

    // ============================================================
    // 29. Cross-context ID namespace separation
    // ============================================================

    #[test]
    fn clipboard_ids_not_in_script_context() {
        let clip = make_text_entry(false, None);
        let script = ScriptInfo::new("s", "/s.ts");
        let clip_actions = get_clipboard_history_context_actions(&clip);
        let clip_ids: HashSet<&str> = action_ids(&clip_actions).into_iter().collect();
        let script_actions = get_script_context_actions(&script);
        let script_ids: HashSet<&str> = action_ids(&script_actions).into_iter().collect();
        let overlap: Vec<&&str> = clip_ids.intersection(&script_ids).collect();
        assert!(
            overlap.is_empty(),
            "Clipboard and script IDs should not overlap: {:?}",
            overlap
        );
    }

    #[test]
    fn file_ids_not_in_clipboard_context() {
        let file = FileInfo {
            path: "/f.txt".to_string(),
            name: "f.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let clip = make_text_entry(false, None);
        let file_actions = get_file_context_actions(&file);
        let file_ids: HashSet<&str> = action_ids(&file_actions).into_iter().collect();
        let clip_actions = get_clipboard_history_context_actions(&clip);
        let clip_ids: HashSet<&str> = action_ids(&clip_actions).into_iter().collect();
        let overlap: Vec<&&str> = file_ids.intersection(&clip_ids).collect();
        assert!(
            overlap.is_empty(),
            "File and clipboard IDs should not overlap: {:?}",
            overlap
        );
    }

    #[test]
    fn ai_ids_not_in_notes_context() {
        let ai_actions = get_ai_command_bar_actions();
        let ai_ids: HashSet<&str> = action_ids(&ai_actions).into_iter().collect();
        let notes_info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let notes_actions = get_notes_command_bar_actions(&notes_info);
        let notes_ids: HashSet<&str> = action_ids(&notes_actions).into_iter().collect();
        // copy_deeplink can exist in both contexts, but the rest should not overlap
        // Actually checking: AI actions should be distinct from notes actions
        let overlap: Vec<&&str> = ai_ids.intersection(&notes_ids).collect();
        // copy_deeplink exists in notes. Let's check what AI has - it has copy_response, copy_chat etc.
        // They should not overlap
        assert!(
            overlap.is_empty(),
            "AI and notes IDs should not overlap: {:?}",
            overlap
        );
    }

    // ============================================================
    // 30. Action title_lower invariant across all builder functions
    // ============================================================

    #[test]
    fn title_lower_matches_title_for_script() {
        let script = ScriptInfo::new("My Script", "/path/s.ts");
        for action in &get_script_context_actions(&script) {
            assert_eq!(
                action.title_lower,
                action.title.to_lowercase(),
                "title_lower mismatch for '{}'",
                action.id
            );
        }
    }

    #[test]
    fn title_lower_matches_title_for_clipboard() {
        let entry = make_text_entry(false, Some("VS Code"));
        for action in &get_clipboard_history_context_actions(&entry) {
            assert_eq!(
                action.title_lower,
                action.title.to_lowercase(),
                "title_lower mismatch for '{}'",
                action.id
            );
        }
    }

