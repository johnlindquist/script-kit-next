    #[test]
    fn clipboard_pin_and_unpin_same_shortcut() {
        let pinned_entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "a".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let unpinned_entry = ClipboardEntryInfo {
            id: "2".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "b".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let pinned_actions = get_clipboard_history_context_actions(&pinned_entry);
        let unpinned_actions = get_clipboard_history_context_actions(&unpinned_entry);
        let unpin = pinned_actions
            .iter()
            .find(|a| a.id == "clipboard_unpin")
            .unwrap();
        let pin = unpinned_actions
            .iter()
            .find(|a| a.id == "clipboard_pin")
            .unwrap();
        assert_eq!(unpin.shortcut, pin.shortcut);
    }

    // =========================================================================
    // 9. File context: dir has no quick_look but has open_with on macOS
    // =========================================================================

    #[test]
    fn file_dir_has_no_quick_look() {
        let file_info = FileInfo {
            name: "Documents".into(),
            path: "/Users/test/Documents".into(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        let actions = get_file_context_actions(&file_info);
        assert!(!actions.iter().any(|a| a.id == "quick_look"));
    }

    #[test]
    fn file_file_has_quick_look_on_macos() {
        let file_info = FileInfo {
            name: "readme.md".into(),
            path: "/Users/test/readme.md".into(),
            is_dir: false,
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&file_info);
        #[cfg(target_os = "macos")]
        assert!(actions.iter().any(|a| a.id == "quick_look"));
    }

    #[test]
    fn file_dir_has_open_with_on_macos() {
        let file_info = FileInfo {
            name: "Documents".into(),
            path: "/Users/test/Documents".into(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        let actions = get_file_context_actions(&file_info);
        #[cfg(target_os = "macos")]
        assert!(actions.iter().any(|a| a.id == "open_with"));
    }

    #[test]
    fn file_dir_has_show_info_on_macos() {
        let file_info = FileInfo {
            name: "Documents".into(),
            path: "/Users/test/Documents".into(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        let actions = get_file_context_actions(&file_info);
        #[cfg(target_os = "macos")]
        assert!(actions.iter().any(|a| a.id == "show_info"));
    }

    // =========================================================================
    // 10. File context: file primary title format uses quoted name
    // =========================================================================

    #[test]
    fn file_primary_title_quotes_filename() {
        let file_info = FileInfo {
            name: "report.pdf".into(),
            path: "/Users/test/report.pdf".into(),
            is_dir: false,
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&file_info);
        assert_eq!(actions[0].title, "Open \"report.pdf\"");
    }

    #[test]
    fn file_dir_primary_title_quotes_dirname() {
        let file_info = FileInfo {
            name: "src".into(),
            path: "/Users/test/src".into(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        let actions = get_file_context_actions(&file_info);
        assert_eq!(actions[0].title, "Open \"src\"");
    }

    #[test]
    fn file_primary_id_is_open_file_for_files() {
        let file_info = FileInfo {
            name: "test.txt".into(),
            path: "/test.txt".into(),
            is_dir: false,
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&file_info);
        assert_eq!(actions[0].id, "open_file");
    }

    #[test]
    fn file_primary_id_is_open_directory_for_dirs() {
        let file_info = FileInfo {
            name: "docs".into(),
            path: "/docs".into(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        let actions = get_file_context_actions(&file_info);
        assert_eq!(actions[0].id, "open_directory");
    }

    // =========================================================================
    // 11. Path context: all action IDs are snake_case
    // =========================================================================

    #[test]
    fn path_file_all_ids_snake_case() {
        let path_info = PathInfo {
            name: "test.txt".into(),
            path: "/test.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        for action in &actions {
            assert!(
                action.id.chars().all(|c| c.is_lowercase() || c == '_'),
                "Action ID '{}' is not snake_case",
                action.id
            );
        }
    }

    #[test]
    fn path_dir_all_ids_snake_case() {
        let path_info = PathInfo {
            name: "docs".into(),
            path: "/docs".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path_info);
        for action in &actions {
            assert!(
                action.id.chars().all(|c| c.is_lowercase() || c == '_'),
                "Action ID '{}' is not snake_case",
                action.id
            );
        }
    }

    #[test]
    fn path_file_has_seven_actions() {
        let path_info = PathInfo {
            name: "test.txt".into(),
            path: "/test.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        assert_eq!(actions.len(), 7);
    }

    #[test]
    fn path_dir_has_seven_actions() {
        let path_info = PathInfo {
            name: "docs".into(),
            path: "/docs".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path_info);
        assert_eq!(actions.len(), 7);
    }

    // =========================================================================
    // 12. Path context: open_in_editor desc mentions $EDITOR
    // =========================================================================

    #[test]
    fn path_open_in_editor_desc_mentions_editor() {
        let path_info = PathInfo {
            name: "test.txt".into(),
            path: "/test.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        let editor_action = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
        assert!(editor_action
            .description
            .as_ref()
            .unwrap()
            .contains("$EDITOR"));
    }

    #[test]
    fn path_open_in_editor_shortcut() {
        let path_info = PathInfo {
            name: "test.txt".into(),
            path: "/test.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        let editor_action = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
        assert_eq!(editor_action.shortcut, Some("⌘E".to_string()));
    }

    #[test]
    fn path_open_in_finder_shortcut() {
        let path_info = PathInfo {
            name: "test.txt".into(),
            path: "/test.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        let finder_action = actions.iter().find(|a| a.id == "open_in_finder").unwrap();
        assert_eq!(finder_action.shortcut, Some("⌘⇧F".to_string()));
    }

    #[test]
    fn path_move_to_trash_shortcut() {
        let path_info = PathInfo {
            name: "test.txt".into(),
            path: "/test.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        let trash_action = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
        assert_eq!(trash_action.shortcut, Some("⌘⌫".to_string()));
    }

    // =========================================================================
    // 13. Script context: scriptlet is_scriptlet true has edit_scriptlet
    // =========================================================================

    #[test]
    fn script_context_scriptlet_has_edit_scriptlet() {
        let info = ScriptInfo::scriptlet("My Snippet", "/path/to/snippets.md", None, None);
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "edit_scriptlet"));
    }

    #[test]
    fn script_context_scriptlet_no_edit_script() {
        let info = ScriptInfo::scriptlet("My Snippet", "/path/to/snippets.md", None, None);
        let actions = get_script_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "edit_script"));
    }

    #[test]
    fn script_context_scriptlet_has_reveal_scriptlet_in_finder() {
        let info = ScriptInfo::scriptlet("My Snippet", "/path/to/snippets.md", None, None);
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "reveal_scriptlet_in_finder"));
    }

    #[test]
    fn script_context_scriptlet_no_view_logs() {
        let info = ScriptInfo::scriptlet("My Snippet", "/path/to/snippets.md", None, None);
        let actions = get_script_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    // =========================================================================
    // 14. Script context: builtin has exactly 4 actions when no shortcut/alias
    // =========================================================================

    #[test]
    fn builtin_no_shortcut_no_alias_has_4_actions() {
        let info = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&info);
        // run_script, add_shortcut, add_alias, copy_deeplink = 4
        assert_eq!(actions.len(), 4);
    }

    #[test]
    fn builtin_action_ids() {
        let info = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&info);
        let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains(&"run_script"));
        assert!(ids.contains(&"add_shortcut"));
        assert!(ids.contains(&"add_alias"));
        assert!(ids.contains(&"copy_deeplink"));
    }

    #[test]
    fn builtin_no_edit_no_reveal_no_copy_path() {
        let info = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "edit_script"));
        assert!(!actions.iter().any(|a| a.id == "reveal_in_finder"));
        assert!(!actions.iter().any(|a| a.id == "copy_path"));
    }

    #[test]
    fn builtin_no_view_logs() {
        let info = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    // =========================================================================
    // 15. Script context: primary title uses action_verb
    // =========================================================================

    #[test]
    fn script_primary_title_uses_run_verb() {
        let info = ScriptInfo::new("my-script", "/path/to/my-script.ts");
        let actions = get_script_context_actions(&info);
        assert_eq!(actions[0].title, "Run \"my-script\"");
    }

    #[test]
    fn script_primary_title_uses_custom_verb() {
        let info = ScriptInfo::with_action_verb("launcher", "/path", true, "Launch");
        let actions = get_script_context_actions(&info);
        assert_eq!(actions[0].title, "Launch \"launcher\"");
    }

    #[test]
    fn script_primary_desc_uses_verb() {
        let info = ScriptInfo::with_action_verb("app", "/path", false, "Open");
        let actions = get_script_context_actions(&info);
        assert!(actions[0].description.as_ref().unwrap().contains("Open"));
    }

    #[test]
    fn script_primary_shortcut_is_enter() {
        let info = ScriptInfo::new("test", "/test");
        let actions = get_script_context_actions(&info);
        assert_eq!(actions[0].shortcut, Some("↵".to_string()));
    }

    // =========================================================================
    // 16. Scriptlet context: with_custom run_script is first action
    // =========================================================================

    #[test]
    fn scriptlet_with_custom_first_is_run_script() {
        let script = ScriptInfo::scriptlet("snippet", "/path/snippet.md", None, None);
        let scriptlet = Scriptlet::new(
            "snippet".to_string(),
            "bash".to_string(),
            "echo hello".to_string(),
        );
        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        assert_eq!(actions[0].id, "run_script");
    }

    #[test]
    fn scriptlet_with_custom_run_title_includes_name() {
        let script = ScriptInfo::scriptlet("My Snippet", "/path/snippet.md", None, None);
        let scriptlet = Scriptlet::new(
            "My Snippet".to_string(),
            "bash".to_string(),
            "echo hello".to_string(),
        );
        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        assert!(actions[0].title.contains("My Snippet"));
    }

    #[test]
    fn scriptlet_with_custom_run_shortcut_enter() {
        let script = ScriptInfo::scriptlet("snippet", "/path/snippet.md", None, None);
        let scriptlet = Scriptlet::new(
            "snippet".to_string(),
            "bash".to_string(),
            "echo hello".to_string(),
        );
        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        assert_eq!(actions[0].shortcut, Some("↵".to_string()));
    }

    #[test]
    fn scriptlet_with_custom_none_scriptlet_has_no_custom_actions() {
        let script = ScriptInfo::scriptlet("snippet", "/path/snippet.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        // Should not have any scriptlet_action: prefixed actions
        assert!(!actions
            .iter()
            .any(|a| a.id.starts_with("scriptlet_action:")));
    }

    // =========================================================================
    // 17. Scriptlet context: with_custom copy_deeplink URL uses to_deeplink_name
    // =========================================================================

    #[test]
    fn scriptlet_copy_deeplink_uses_deeplink_name() {
        let script = ScriptInfo::scriptlet("Open GitHub", "/path/snippet.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let deeplink = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert!(deeplink
            .description
            .as_ref()
            .unwrap()
            .contains("open-github"));
    }

    #[test]
    fn scriptlet_copy_deeplink_shortcut() {
        let script = ScriptInfo::scriptlet("Test", "/path", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let deeplink = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert_eq!(deeplink.shortcut, Some("⌘⇧D".to_string()));
    }

    #[test]
    fn scriptlet_copy_content_shortcut() {
        let script = ScriptInfo::scriptlet("Test", "/path", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
        assert_eq!(cc.shortcut, Some("⌘⌥C".to_string()));
    }

