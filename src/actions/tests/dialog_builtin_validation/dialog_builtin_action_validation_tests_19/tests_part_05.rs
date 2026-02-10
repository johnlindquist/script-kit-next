    #[test]
    fn cat29_script_actions_has_action_false() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        for action in &get_script_context_actions(&script) {
            assert!(
                !action.has_action,
                "Script action {} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat29_clipboard_actions_has_action_false() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        for action in &get_clipboard_history_context_actions(&entry) {
            assert!(
                !action.has_action,
                "Clipboard action {} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat29_ai_command_bar_has_action_false() {
        for action in &get_ai_command_bar_actions() {
            assert!(
                !action.has_action,
                "AI action {} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat29_path_actions_has_action_false() {
        let info = PathInfo {
            path: "/test".to_string(),
            is_dir: true,
            name: "test".to_string(),
        };
        for action in &get_path_context_actions(&info) {
            assert!(
                !action.has_action,
                "Path action {} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat29_notes_actions_has_action_false() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for action in &get_notes_command_bar_actions(&info) {
            assert!(
                !action.has_action,
                "Notes action {} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat29_file_actions_has_action_false() {
        let info = FileInfo {
            path: "/test/file.txt".to_string(),
            is_dir: false,
            name: "file.txt".to_string(),
            file_type: FileType::File,
        };
        for action in &get_file_context_actions(&info) {
            assert!(
                !action.has_action,
                "File action {} should have has_action=false",
                action.id
            );
        }
    }

    // =========================================================================
    // Category 30: Cross-context ID uniqueness invariant
    // Validates that all action IDs within a single context are unique.
    // =========================================================================

    #[test]
    fn cat30_script_ids_unique() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len(), "Script action IDs not unique");
    }

    #[test]
    fn cat30_clipboard_text_ids_unique() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len(), "Clipboard text IDs not unique");
    }

    #[test]
    fn cat30_clipboard_image_ids_unique() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "".to_string(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len(), "Clipboard image IDs not unique");
    }

    #[test]
    fn cat30_ai_command_bar_ids_unique() {
        let actions = get_ai_command_bar_actions();
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len(), "AI command bar IDs not unique");
    }

    #[test]
    fn cat30_notes_ids_unique() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len(), "Notes action IDs not unique");
    }

    #[test]
    fn cat30_path_ids_unique() {
        let info = PathInfo {
            path: "/test".to_string(),
            is_dir: true,
            name: "test".to_string(),
        };
        let actions = get_path_context_actions(&info);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len(), "Path action IDs not unique");
    }
