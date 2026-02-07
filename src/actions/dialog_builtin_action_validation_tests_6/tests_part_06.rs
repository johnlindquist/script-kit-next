    #[test]
    fn scriptlet_defined_actions_have_values() {
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo test".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Custom".into(),
            command: "custom-cmd".into(),
            tool: "bash".into(),
            code: "echo custom".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions[0].value, Some("custom-cmd".into()));
    }

    #[test]
    fn scriptlet_defined_action_id_format() {
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo test".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "My Custom".into(),
            command: "my-custom".into(),
            tool: "bash".into(),
            code: "echo custom".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions[0].id, "scriptlet_action:my-custom");
    }

    // =========================================================================
    // 28. Action ID uniqueness within contexts
    // =========================================================================

    #[test]
    fn script_context_ids_unique() {
        let info = ScriptInfo::new("Test", "/path/test.ts");
        let actions = get_script_context_actions(&info);
        let ids = action_ids(&actions);
        let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "Script IDs should be unique");
    }

    #[test]
    fn clipboard_text_ids_unique() {
        let entry = ClipboardEntryInfo {
            id: "uniq".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "Clipboard IDs should be unique");
    }

    #[test]
    fn path_context_ids_unique() {
        let info = PathInfo {
            path: "/test".into(),
            name: "test".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let ids = action_ids(&actions);
        let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "Path IDs should be unique");
    }

    #[test]
    fn file_context_ids_unique() {
        let info = FileInfo {
            path: "/test/file.txt".into(),
            name: "file.txt".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        let ids = action_ids(&actions);
        let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "File IDs should be unique");
    }

    #[test]
    fn ai_command_bar_ids_unique() {
        let actions = get_ai_command_bar_actions();
        let ids = action_ids(&actions);
        let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "AI IDs should be unique");
    }

    #[test]
    fn notes_command_bar_ids_unique() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "Notes IDs should be unique");
    }

    // =========================================================================
    // 29. All actions have non-empty title and ID
    // =========================================================================

    #[test]
    fn all_script_actions_nonempty_title_and_id() {
        let info = ScriptInfo::new("Test", "/path/test.ts");
        let actions = get_script_context_actions(&info);
        for action in &actions {
            assert!(!action.id.is_empty(), "Action ID should not be empty");
            assert!(!action.title.is_empty(), "Action title should not be empty");
        }
    }

    #[test]
    fn all_clipboard_actions_nonempty_title_and_id() {
        let entry = ClipboardEntryInfo {
            id: "ne".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        for action in &actions {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    #[test]
    fn all_ai_actions_nonempty_title_and_id() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    // =========================================================================
    // 30. ActionCategory on all built-in actions
    // =========================================================================

    #[test]
    fn all_script_actions_are_script_context_category() {
        let info = ScriptInfo::new("Test", "/path/test.ts");
        let actions = get_script_context_actions(&info);
        for action in &actions {
            assert_eq!(
                action.category,
                ActionCategory::ScriptContext,
                "Action '{}' should be ScriptContext",
                action.id
            );
        }
    }

    #[test]
    fn all_clipboard_actions_are_script_context_category() {
        let entry = ClipboardEntryInfo {
            id: "cat".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        for action in &actions {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn all_path_actions_are_script_context_category() {
        let info = PathInfo {
            path: "/test".into(),
            name: "test".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        for action in &actions {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn all_file_actions_are_script_context_category() {
        let info = FileInfo {
            path: "/test/file.txt".into(),
            name: "file.txt".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        for action in &actions {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }
