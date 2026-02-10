    #[test]
    fn clipboard_attach_to_ai_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hi".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let attach = actions
            .iter()
            .find(|a| a.id == "clipboard_attach_to_ai")
            .unwrap();
        assert_eq!(attach.shortcut.as_deref(), Some("⌃⌘A"));
    }

    #[test]
    fn clipboard_attach_to_ai_title() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hi".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let attach = actions
            .iter()
            .find(|a| a.id == "clipboard_attach_to_ai")
            .unwrap();
        assert_eq!(attach.title, "Attach to AI Chat");
    }

    #[test]
    fn clipboard_attach_to_ai_desc_mentions_ai() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hi".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let attach = actions
            .iter()
            .find(|a| a.id == "clipboard_attach_to_ai")
            .unwrap();
        assert!(attach
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("ai"));
    }

    #[test]
    fn clipboard_attach_to_ai_present_for_image_too() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_attach_to_ai"));
    }

    // =========================================================================
    // 12. Clipboard: image open_with is macOS only
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn clipboard_image_has_open_with() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_open_with"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn clipboard_text_has_no_open_with() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(!actions.iter().any(|a| a.id == "clipboard_open_with"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn clipboard_image_open_with_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let open_with = actions
            .iter()
            .find(|a| a.id == "clipboard_open_with")
            .unwrap();
        assert_eq!(open_with.shortcut.as_deref(), Some("⌘O"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn clipboard_image_annotate_cleanshot_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let annotate = actions
            .iter()
            .find(|a| a.id == "clipboard_annotate_cleanshot")
            .unwrap();
        assert_eq!(annotate.shortcut.as_deref(), Some("⇧⌘A"));
    }

    // =========================================================================
    // 13. File context: primary action ID differs file vs dir
    // =========================================================================

    #[test]
    fn file_context_file_primary_id_is_open_file() {
        let info = FileInfo {
            name: "readme.md".to_string(),
            path: "/docs/readme.md".to_string(),
            is_dir: false,
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&info);
        assert_eq!(actions[0].id, "open_file");
    }

    #[test]
    fn file_context_dir_primary_id_is_open_directory() {
        let info = FileInfo {
            name: "src".to_string(),
            path: "/project/src".to_string(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        let actions = get_file_context_actions(&info);
        assert_eq!(actions[0].id, "open_directory");
    }

    #[test]
    fn file_context_primary_shortcut_is_enter() {
        let info = FileInfo {
            name: "test.txt".to_string(),
            path: "/test.txt".to_string(),
            is_dir: false,
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&info);
        assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
    }

    #[test]
    fn file_context_dir_primary_desc_mentions_folder() {
        let info = FileInfo {
            name: "lib".to_string(),
            path: "/lib".to_string(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        let actions = get_file_context_actions(&info);
        assert!(actions[0]
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("folder"));
    }

    // =========================================================================
    // 14. File context: all IDs unique within context
    // =========================================================================

    #[test]
    fn file_context_file_all_ids_unique() {
        let info = FileInfo {
            name: "test.rs".to_string(),
            path: "/test.rs".to_string(),
            is_dir: false,
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&info);
        let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        let mut unique_ids = ids.clone();
        unique_ids.sort();
        unique_ids.dedup();
        assert_eq!(ids.len(), unique_ids.len());
    }

    #[test]
    fn file_context_dir_all_ids_unique() {
        let info = FileInfo {
            name: "docs".to_string(),
            path: "/docs".to_string(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        let actions = get_file_context_actions(&info);
        let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        let mut unique_ids = ids.clone();
        unique_ids.sort();
        unique_ids.dedup();
        assert_eq!(ids.len(), unique_ids.len());
    }

    #[test]
    fn file_context_file_has_copy_path_and_copy_filename() {
        let info = FileInfo {
            name: "foo.txt".to_string(),
            path: "/foo.txt".to_string(),
            is_dir: false,
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "copy_path"));
        assert!(actions.iter().any(|a| a.id == "copy_filename"));
    }

    #[test]
    fn file_context_reveal_in_finder_always_present() {
        let file_info = FileInfo {
            name: "a.txt".to_string(),
            path: "/a.txt".to_string(),
            is_dir: false,
            file_type: FileType::File,
        };
        let dir_info = FileInfo {
            name: "b".to_string(),
            path: "/b".to_string(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        assert!(get_file_context_actions(&file_info)
            .iter()
            .any(|a| a.id == "reveal_in_finder"));
        assert!(get_file_context_actions(&dir_info)
            .iter()
            .any(|a| a.id == "reveal_in_finder"));
    }

    // =========================================================================
    // 15. Path context: open_in_terminal shortcut and desc
    // =========================================================================

    #[test]
    fn path_context_open_in_terminal_shortcut() {
        let info = PathInfo {
            name: "src".to_string(),
            path: "/project/src".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let term = actions.iter().find(|a| a.id == "open_in_terminal").unwrap();
        assert_eq!(term.shortcut.as_deref(), Some("⌘T"));
    }

    #[test]
    fn path_context_open_in_terminal_desc_mentions_terminal() {
        let info = PathInfo {
            name: "src".to_string(),
            path: "/project/src".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let term = actions.iter().find(|a| a.id == "open_in_terminal").unwrap();
        assert!(term
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("terminal"));
    }

    #[test]
    fn path_context_open_in_terminal_present_for_files() {
        let info = PathInfo {
            name: "script.sh".to_string(),
            path: "/project/script.sh".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "open_in_terminal"));
    }

    #[test]
    fn path_context_open_in_terminal_title() {
        let info = PathInfo {
            name: "foo".to_string(),
            path: "/foo".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let term = actions.iter().find(|a| a.id == "open_in_terminal").unwrap();
        assert_eq!(term.title, "Open in Terminal");
    }

    // =========================================================================
    // 16. Path context: move_to_trash desc differs file vs dir
    // =========================================================================

    #[test]
    fn path_context_trash_desc_file() {
        let info = PathInfo {
            name: "test.txt".to_string(),
            path: "/test.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
        assert!(trash.description.as_ref().unwrap().contains("file"));
    }

    #[test]
    fn path_context_trash_desc_dir() {
        let info = PathInfo {
            name: "src".to_string(),
            path: "/src".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
        assert!(trash.description.as_ref().unwrap().contains("folder"));
    }

    #[test]
    fn path_context_trash_shortcut() {
        let info = PathInfo {
            name: "x".to_string(),
            path: "/x".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
        assert_eq!(trash.shortcut.as_deref(), Some("⌘⌫"));
    }

    #[test]
    fn path_context_trash_is_last_action() {
        let info = PathInfo {
            name: "y".to_string(),
            path: "/y".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions.last().unwrap().id, "move_to_trash");
    }

    // =========================================================================
    // 17. Script context: with shortcut yields update_shortcut + remove_shortcut
    // =========================================================================

    #[test]
    fn script_with_shortcut_has_update_shortcut() {
        let info = ScriptInfo::with_shortcut("my-script", "/s.ts", Some("cmd+k".into()));
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "update_shortcut"));
    }

    #[test]
    fn script_with_shortcut_has_remove_shortcut() {
        let info = ScriptInfo::with_shortcut("my-script", "/s.ts", Some("cmd+k".into()));
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
    }

    #[test]
    fn script_with_shortcut_has_no_add_shortcut() {
        let info = ScriptInfo::with_shortcut("my-script", "/s.ts", Some("cmd+k".into()));
        let actions = get_script_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "add_shortcut"));
    }

    #[test]
    fn script_without_shortcut_has_add_shortcut() {
        let info = ScriptInfo::new("my-script", "/s.ts");
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "add_shortcut"));
        assert!(!actions.iter().any(|a| a.id == "update_shortcut"));
        assert!(!actions.iter().any(|a| a.id == "remove_shortcut"));
    }

    // =========================================================================
    // 18. Script context: with alias yields update_alias + remove_alias
    // =========================================================================

    #[test]
    fn script_with_alias_has_update_alias() {
        let info =
            ScriptInfo::with_shortcut_and_alias("my-script", "/s.ts", None, Some("ms".to_string()));
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "update_alias"));
    }

    #[test]
    fn script_with_alias_has_remove_alias() {
        let info =
            ScriptInfo::with_shortcut_and_alias("my-script", "/s.ts", None, Some("ms".to_string()));
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "remove_alias"));
    }

    #[test]
    fn script_with_alias_has_no_add_alias() {
        let info =
            ScriptInfo::with_shortcut_and_alias("my-script", "/s.ts", None, Some("ms".to_string()));
        let actions = get_script_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "add_alias"));
    }

