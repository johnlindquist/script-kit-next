    #[test]
    fn chat_model_descriptions_show_provider() {
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
        let claude = actions
            .iter()
            .find(|a| a.id == "select_model_claude")
            .unwrap();
        assert_eq!(claude.description.as_deref(), Some("via Anthropic"));
    }

    #[test]
    fn chat_copy_response_only_when_has_response() {
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
        let with_actions = get_chat_context_actions(&with);
        assert!(!without_actions.iter().any(|a| a.id == "copy_response"));
        assert!(with_actions.iter().any(|a| a.id == "copy_response"));
    }

    #[test]
    fn chat_clear_conversation_only_when_has_messages() {
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
        let with_actions = get_chat_context_actions(&with);
        assert!(!without_actions.iter().any(|a| a.id == "clear_conversation"));
        assert!(with_actions.iter().any(|a| a.id == "clear_conversation"));
    }

    // =========================================================================
    // 14. Path context specifics
    // =========================================================================

    #[test]
    fn path_dir_primary_is_open_directory() {
        let path_info = PathInfo {
            path: "/tmp/mydir".into(),
            name: "mydir".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path_info);
        assert_eq!(actions[0].id, "open_directory");
        assert!(actions[0].title.contains("mydir"));
    }

    #[test]
    fn path_file_primary_is_select_file() {
        let path_info = PathInfo {
            path: "/tmp/myfile.txt".into(),
            name: "myfile.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        assert_eq!(actions[0].id, "select_file");
        assert!(actions[0].title.contains("myfile.txt"));
    }

    #[test]
    fn path_all_have_descriptions() {
        let path_info = PathInfo {
            path: "/tmp/test".into(),
            name: "test".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        for action in &actions {
            assert!(
                action.description.is_some(),
                "Path action '{}' should have a description",
                action.id
            );
        }
    }

    #[test]
    fn path_has_expected_actions() {
        let path_info = PathInfo {
            path: "/tmp/test".into(),
            name: "test".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"copy_path"));
        assert!(ids.contains(&"open_in_finder"));
        assert!(ids.contains(&"open_in_editor"));
        assert!(ids.contains(&"open_in_terminal"));
        assert!(ids.contains(&"copy_filename"));
        assert!(ids.contains(&"move_to_trash"));
    }

    #[test]
    fn path_dir_trash_says_folder() {
        let path_info = PathInfo {
            path: "/tmp/mydir".into(),
            name: "mydir".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path_info);
        let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
        assert!(
            trash.description.as_ref().unwrap().contains("folder"),
            "Dir trash should say 'folder'"
        );
    }

    #[test]
    fn path_file_trash_says_file() {
        let path_info = PathInfo {
            path: "/tmp/myfile.txt".into(),
            name: "myfile.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
        assert!(
            trash.description.as_ref().unwrap().contains("file"),
            "File trash should say 'file'"
        );
    }

    // =========================================================================
    // 15. File context specifics
    // =========================================================================

    #[test]
    fn file_open_title_includes_name() {
        let file_info = FileInfo {
            path: "/Users/test/readme.md".into(),
            name: "readme.md".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        assert!(actions[0].title.contains("readme.md"));
    }

    #[test]
    fn file_dir_open_title_includes_dirname() {
        let file_info = FileInfo {
            path: "/Users/test/Documents".into(),
            name: "Documents".into(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&file_info);
        assert!(actions[0].title.contains("Documents"));
    }

    #[test]
    fn file_all_have_descriptions() {
        let file_info = FileInfo {
            path: "/test/file.rs".into(),
            name: "file.rs".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        for action in &actions {
            assert!(
                action.description.is_some(),
                "File action '{}' should have a description",
                action.id
            );
        }
    }

    #[test]
    fn file_all_have_shortcuts() {
        let file_info = FileInfo {
            path: "/test/file.rs".into(),
            name: "file.rs".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        for action in &actions {
            assert!(
                action.shortcut.is_some(),
                "File action '{}' should have a shortcut",
                action.id
            );
        }
    }

    // =========================================================================
    // 16. Notes command bar conditional logic
    // =========================================================================

    #[test]
    fn notes_no_selection_no_trash_no_auto() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"new_note"));
        assert!(ids.contains(&"browse_notes"));
        assert!(ids.contains(&"enable_auto_sizing"));
        assert!(!ids.contains(&"duplicate_note"));
        assert!(!ids.contains(&"find_in_note"));
        assert!(!ids.contains(&"format"));
        assert!(!ids.contains(&"export"));
    }

    #[test]
    fn notes_with_selection_not_trash_auto_disabled() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"new_note"));
        assert!(ids.contains(&"duplicate_note"));
        assert!(ids.contains(&"browse_notes"));
        assert!(ids.contains(&"find_in_note"));
        assert!(ids.contains(&"format"));
        assert!(ids.contains(&"copy_note_as"));
        assert!(ids.contains(&"copy_deeplink"));
        assert!(ids.contains(&"create_quicklink"));
        assert!(ids.contains(&"export"));
        assert!(ids.contains(&"enable_auto_sizing"));
    }

    #[test]
    fn notes_with_selection_in_trash_hides_edit_copy_export() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"new_note"));
        assert!(ids.contains(&"browse_notes"));
        assert!(!ids.contains(&"duplicate_note"));
        assert!(!ids.contains(&"find_in_note"));
        assert!(!ids.contains(&"format"));
        assert!(!ids.contains(&"copy_note_as"));
        assert!(!ids.contains(&"export"));
    }

    #[test]
    fn notes_auto_sizing_enabled_hides_enable_action() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        assert!(!ids.contains(&"enable_auto_sizing"));
    }

    #[test]
    fn notes_full_feature_action_count() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        // new_note, duplicate, browse, find, format, copy_note_as,
        // copy_deeplink, create_quicklink, export, enable_auto_sizing = 10
        assert_eq!(actions.len(), 10);
    }

    #[test]
    fn notes_minimal_action_count() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        // new_note + browse_notes = 2
        assert_eq!(actions.len(), 2);
    }

    // =========================================================================
    // 17. to_deeplink_name comprehensive
    // =========================================================================

    #[test]
    fn deeplink_name_basic_spaces() {
        assert_eq!(to_deeplink_name("My Script"), "my-script");
    }

    #[test]
    fn deeplink_name_underscores_to_hyphens() {
        assert_eq!(to_deeplink_name("hello_world"), "hello-world");
    }

    #[test]
    fn deeplink_name_special_chars_stripped() {
        assert_eq!(to_deeplink_name("test!@#$%^&*()"), "test");
    }

    #[test]
    fn deeplink_name_multiple_spaces_collapsed() {
        assert_eq!(to_deeplink_name("a   b   c"), "a-b-c");
    }

    #[test]
    fn deeplink_name_leading_trailing_stripped() {
        assert_eq!(to_deeplink_name("  hello  "), "hello");
    }

    #[test]
    fn deeplink_name_numbers_preserved() {
        assert_eq!(to_deeplink_name("Test 123"), "test-123");
    }

    #[test]
    fn deeplink_name_all_special_returns_empty() {
        assert_eq!(to_deeplink_name("!@#$%"), "");
    }

    #[test]
    fn deeplink_name_single_word() {
        assert_eq!(to_deeplink_name("hello"), "hello");
    }

    #[test]
    fn deeplink_name_already_hyphenated() {
        assert_eq!(to_deeplink_name("my-script"), "my-script");
    }

    // =========================================================================
    // 18. format_shortcut_hint specifics
    // =========================================================================

    #[test]
    fn format_shortcut_cmd_enter() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+enter"), "⌘↵");
    }

    #[test]
    fn format_shortcut_ctrl_shift_escape() {
        assert_eq!(
            ActionsDialog::format_shortcut_hint("ctrl+shift+escape"),
            "⌃⇧⎋"
        );
    }

    #[test]
    fn format_shortcut_alt_backspace() {
        assert_eq!(ActionsDialog::format_shortcut_hint("alt+backspace"), "⌥⌫");
    }

    #[test]
    fn format_shortcut_command_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("command+c"), "⌘C");
    }

    #[test]
    fn format_shortcut_meta_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("meta+k"), "⌘K");
    }

    #[test]
    fn format_shortcut_option_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("option+tab"), "⌥⇥");
    }

    #[test]
    fn format_shortcut_control_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("control+space"), "⌃␣");
    }

    #[test]
    fn format_shortcut_arrows() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+up"), "⌘↑");
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+down"), "⌘↓");
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+left"), "⌘←");
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+right"), "⌘→");
    }

    #[test]
    fn format_shortcut_arrowup_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+arrowup"), "⌘↑");
    }

    // =========================================================================
    // 19. parse_shortcut_keycaps specifics
    // =========================================================================

    #[test]
    fn parse_keycaps_modifier_plus_letter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘E");
        assert_eq!(keycaps, vec!["⌘", "E"]);
    }

    #[test]
    fn parse_keycaps_two_modifiers_plus_letter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⇧C");
        assert_eq!(keycaps, vec!["⌘", "⇧", "C"]);
    }

    #[test]
    fn parse_keycaps_enter_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("↵");
        assert_eq!(keycaps, vec!["↵"]);
    }

    #[test]
    fn parse_keycaps_modifier_plus_enter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘↵");
        assert_eq!(keycaps, vec!["⌘", "↵"]);
    }

