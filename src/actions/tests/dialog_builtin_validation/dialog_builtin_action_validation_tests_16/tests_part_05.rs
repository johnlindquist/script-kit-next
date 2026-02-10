    #[test]
    fn cat30_script_has_action_false() {
        let script = ScriptInfo::new("test", "/p/test.ts");
        for action in &get_script_context_actions(&script) {
            assert!(
                !action.has_action,
                "built-in action '{}' should be false",
                action.id
            );
        }
    }

    #[test]
    fn cat30_clipboard_has_action_false() {
        let entry = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        for action in &get_clipboard_history_context_actions(&entry) {
            assert!(
                !action.has_action,
                "clipboard action '{}' should be false",
                action.id
            );
        }
    }

    #[test]
    fn cat30_script_ids_unique() {
        let script = ScriptInfo::new("test", "/p/test.ts");
        let ids = action_ids(&get_script_context_actions(&script));
        let set: HashSet<&String> = ids.iter().collect();
        assert_eq!(ids.len(), set.len(), "script IDs must be unique");
    }

    #[test]
    fn cat30_ai_ids_unique() {
        let ids = action_ids(&get_ai_command_bar_actions());
        let set: HashSet<&String> = ids.iter().collect();
        assert_eq!(ids.len(), set.len(), "AI IDs must be unique");
    }

    #[test]
    fn cat30_path_ids_unique() {
        let info = PathInfo {
            name: "f".into(),
            path: "/f".into(),
            is_dir: false,
        };
        let ids = action_ids(&get_path_context_actions(&info));
        let set: HashSet<&String> = ids.iter().collect();
        assert_eq!(ids.len(), set.len(), "path IDs must be unique");
    }

    #[test]
    fn cat30_file_ids_unique() {
        let file = FileInfo {
            path: "/x.rs".into(),
            name: "x.rs".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let ids = action_ids(&get_file_context_actions(&file));
        let set: HashSet<&String> = ids.iter().collect();
        assert_eq!(ids.len(), set.len(), "file IDs must be unique");
    }

    #[test]
    fn cat30_notes_ids_unique() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let ids = action_ids(&get_notes_command_bar_actions(&info));
        let set: HashSet<&String> = ids.iter().collect();
        assert_eq!(ids.len(), set.len(), "notes IDs must be unique");
    }
