    #[test]
    fn ai_command_bar_section_order_correct() {
        let actions = get_ai_command_bar_actions();
        let sections: Vec<&str> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        // Order: Response(3), Actions(3), Attachments(2), Export(1), Actions(1), Help(1), Settings(1)
        let unique_order: Vec<&str> = {
            let mut result = vec![];
            let mut prev: Option<&str> = None;
            for s in &sections {
                if prev != Some(s) {
                    result.push(*s);
                    prev = Some(s);
                }
            }
            result
        };
        assert_eq!(
            unique_order,
            vec![
                "Response",
                "Actions",
                "Attachments",
                "Export",
                "Actions",
                "Help",
                "Settings"
            ]
        );
    }

    // =========================================================================
    // 20. fuzzy_match edge cases
    // =========================================================================

    #[test]
    fn fuzzy_match_empty_needle_matches_anything() {
        assert!(ActionsDialog::fuzzy_match("hello", ""));
    }

    #[test]
    fn fuzzy_match_empty_haystack_no_match() {
        assert!(!ActionsDialog::fuzzy_match("", "a"));
    }

    #[test]
    fn fuzzy_match_both_empty_matches() {
        assert!(ActionsDialog::fuzzy_match("", ""));
    }

    #[test]
    fn fuzzy_match_exact_match() {
        assert!(ActionsDialog::fuzzy_match("hello", "hello"));
    }

    #[test]
    fn fuzzy_match_subsequence() {
        assert!(ActionsDialog::fuzzy_match("hello world", "hlwrd"));
    }

    #[test]
    fn fuzzy_match_no_subsequence() {
        assert!(!ActionsDialog::fuzzy_match("hello", "xyz"));
    }

    #[test]
    fn fuzzy_match_needle_longer_than_haystack() {
        assert!(!ActionsDialog::fuzzy_match("hi", "hello"));
    }

    #[test]
    fn fuzzy_match_single_char() {
        assert!(ActionsDialog::fuzzy_match("hello", "h"));
        assert!(ActionsDialog::fuzzy_match("hello", "o"));
        assert!(!ActionsDialog::fuzzy_match("hello", "z"));
    }

    // =========================================================================
    // 21. parse_shortcut_keycaps edge cases
    // =========================================================================

    #[test]
    fn parse_keycaps_modifier_plus_letter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘C");
        assert_eq!(keycaps, vec!["⌘", "C"]);
    }

    #[test]
    fn parse_keycaps_two_modifiers() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⇧C");
        assert_eq!(keycaps, vec!["⌘", "⇧", "C"]);
    }

    #[test]
    fn parse_keycaps_enter_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("↵");
        assert_eq!(keycaps, vec!["↵"]);
    }

    #[test]
    fn parse_keycaps_arrow_keys() {
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("↑"), vec!["↑"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("↓"), vec!["↓"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("←"), vec!["←"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("→"), vec!["→"]);
    }

    #[test]
    fn parse_keycaps_escape_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⎋");
        assert_eq!(keycaps, vec!["⎋"]);
    }

    #[test]
    fn parse_keycaps_space_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("␣");
        assert_eq!(keycaps, vec!["␣"]);
    }

    #[test]
    fn parse_keycaps_lowercase_uppercased() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘e");
        assert_eq!(keycaps, vec!["⌘", "E"]);
    }

    // =========================================================================
    // 22. to_deeplink_name edge cases
    // =========================================================================

    #[test]
    fn deeplink_name_basic() {
        assert_eq!(to_deeplink_name("My Script"), "my-script");
    }

    #[test]
    fn deeplink_name_underscores_to_hyphens() {
        assert_eq!(to_deeplink_name("hello_world"), "hello-world");
    }

    #[test]
    fn deeplink_name_special_chars_stripped() {
        assert_eq!(to_deeplink_name("Hello!@#$World"), "hello-world");
    }

    #[test]
    fn deeplink_name_multiple_spaces_collapsed() {
        assert_eq!(to_deeplink_name("My   Script"), "my-script");
    }

    #[test]
    fn deeplink_name_leading_trailing_stripped() {
        assert_eq!(to_deeplink_name("  My Script  "), "my-script");
    }

    #[test]
    fn deeplink_name_numbers_preserved() {
        assert_eq!(to_deeplink_name("Script 123"), "script-123");
    }

    #[test]
    fn deeplink_name_all_special_chars_empty() {
        assert_eq!(to_deeplink_name("!@#$%^&*"), "");
    }

    #[test]
    fn deeplink_name_already_hyphenated() {
        assert_eq!(to_deeplink_name("already-hyphenated"), "already-hyphenated");
    }

    #[test]
    fn deeplink_name_mixed_case() {
        assert_eq!(to_deeplink_name("CamelCaseScript"), "camelcasescript");
    }

    // =========================================================================
    // 23. Agent ScriptInfo with full flag set
    // =========================================================================

    #[test]
    fn agent_with_shortcut_alias_frecency() {
        let mut info = ScriptInfo::with_all(
            "My Agent",
            "/path/agent.md",
            false,
            "Run",
            Some("cmd+a".into()),
            Some("ma".into()),
        );
        info.is_agent = true;
        let info = info.with_frecency(true, Some("agent:/path".into()));

        let actions = get_script_context_actions(&info);
        let ids = action_ids(&actions);

        // Agent-specific actions
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");

        // Has update/remove for shortcut and alias
        assert!(ids.contains(&"update_shortcut"));
        assert!(ids.contains(&"remove_shortcut"));
        assert!(ids.contains(&"update_alias"));
        assert!(ids.contains(&"remove_alias"));

        // Has frecency reset
        assert!(ids.contains(&"reset_ranking"));

        // Has agent copy actions
        assert!(ids.contains(&"copy_path"));
        assert!(ids.contains(&"copy_content"));
        assert!(ids.contains(&"reveal_in_finder"));
    }

    // =========================================================================
    // 24. Global actions always empty
    // =========================================================================

    #[test]
    fn global_actions_is_empty() {
        assert!(get_global_actions().is_empty());
    }

    // =========================================================================
    // 25. Ordering determinism across repeated calls
    // =========================================================================

    #[test]
    fn script_actions_deterministic() {
        let info = ScriptInfo::new("Test", "/path/test.ts");
        let a1 = get_script_context_actions(&info);
        let a2 = get_script_context_actions(&info);
        let ids1 = action_ids(&a1);
        let ids2 = action_ids(&a2);
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn clipboard_actions_deterministic() {
        let entry = ClipboardEntryInfo {
            id: "det".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let a1 = get_clipboard_history_context_actions(&entry);
        let a2 = get_clipboard_history_context_actions(&entry);
        let ids1 = action_ids(&a1);
        let ids2 = action_ids(&a2);
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn ai_actions_deterministic() {
        let a1 = get_ai_command_bar_actions();
        let a2 = get_ai_command_bar_actions();
        let ids1 = action_ids(&a1);
        let ids2 = action_ids(&a2);
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn notes_actions_deterministic() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let a1 = get_notes_command_bar_actions(&info);
        let a2 = get_notes_command_bar_actions(&info);
        let ids1 = action_ids(&a1);
        let ids2 = action_ids(&a2);
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn path_actions_deterministic() {
        let info = PathInfo {
            path: "/test/dir".into(),
            name: "dir".into(),
            is_dir: true,
        };
        let a1 = get_path_context_actions(&info);
        let a2 = get_path_context_actions(&info);
        let ids1 = action_ids(&a1);
        let ids2 = action_ids(&a2);
        assert_eq!(ids1, ids2);
    }

    // =========================================================================
    // 26. has_action invariant across contexts
    // =========================================================================

    #[test]
    fn script_context_all_has_action_false() {
        let info = ScriptInfo::new("Test", "/path/test.ts");
        let actions = get_script_context_actions(&info);
        for action in &actions {
            assert!(
                !action.has_action,
                "Script action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn clipboard_context_all_has_action_false() {
        let entry = ClipboardEntryInfo {
            id: "ha".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        for action in &actions {
            assert!(
                !action.has_action,
                "Clipboard action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn path_context_all_has_action_false() {
        let info = PathInfo {
            path: "/test/file.txt".into(),
            name: "file.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        for action in &actions {
            assert!(
                !action.has_action,
                "Path action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn file_context_all_has_action_false() {
        let info = FileInfo {
            path: "/test/file.txt".into(),
            name: "file.txt".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        for action in &actions {
            assert!(
                !action.has_action,
                "File action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn ai_command_bar_all_has_action_false() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                !action.has_action,
                "AI action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn notes_command_bar_all_has_action_false() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for action in &actions {
            assert!(
                !action.has_action,
                "Notes action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn chat_context_builtin_actions_has_action_false() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        for action in &actions {
            assert!(
                !action.has_action,
                "Chat action '{}' should have has_action=false",
                action.id
            );
        }
    }

    // =========================================================================
    // 27. Scriptlet defined actions have has_action=true
    // =========================================================================

    #[test]
    fn scriptlet_defined_actions_all_have_has_action_true() {
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo test".into());
        scriptlet.actions = vec![
            ScriptletAction {
                name: "Action 1".into(),
                command: "action-1".into(),
                tool: "bash".into(),
                code: "echo 1".into(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
            ScriptletAction {
                name: "Action 2".into(),
                command: "action-2".into(),
                tool: "bash".into(),
                code: "echo 2".into(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
        ];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        for action in &actions {
            assert!(
                action.has_action,
                "Scriptlet defined action '{}' should have has_action=true",
                action.id
            );
        }
    }

