    #[test]
    fn cat07_note_switcher_preview_and_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".to_string(),
            title: "Test".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "Hello world".to_string(),
            relative_time: "2m ago".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains("Hello world"));
        assert!(desc.contains("2m ago"));
        assert!(desc.contains("·"));
    }

    #[test]
    fn cat07_note_switcher_preview_no_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".to_string(),
            title: "Test".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "Hello world".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "Hello world");
    }

    #[test]
    fn cat07_note_switcher_no_preview_with_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".to_string(),
            title: "Test".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "".to_string(),
            relative_time: "5d ago".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "5d ago");
    }

    #[test]
    fn cat07_note_switcher_no_preview_no_time_char_count() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".to_string(),
            title: "Test".to_string(),
            char_count: 42,
            is_current: false,
            is_pinned: false,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "42 chars");
    }

    #[test]
    fn cat07_note_switcher_zero_chars_fallback() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".to_string(),
            title: "Test".to_string(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "0 chars");
    }

    #[test]
    fn cat07_note_switcher_one_char_singular() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".to_string(),
            title: "Test".to_string(),
            char_count: 1,
            is_current: false,
            is_pinned: false,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "1 char");
    }

    // =========================================================================
    // Category 08: Script context action ordering invariant — run_script first
    // Validates that run_script is always the very first action.
    // =========================================================================

    #[test]
    fn cat08_script_run_first_basic() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        assert_eq!(actions[0].id, "run_script");
    }

    #[test]
    fn cat08_script_run_first_with_shortcut() {
        let script = ScriptInfo::with_shortcut("test", "/path/test.ts", Some("cmd+t".to_string()));
        let actions = get_script_context_actions(&script);
        assert_eq!(actions[0].id, "run_script");
    }

    #[test]
    fn cat08_script_run_first_builtin() {
        let script = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&script);
        assert_eq!(actions[0].id, "run_script");
    }

    #[test]
    fn cat08_scriptlet_run_first() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert_eq!(actions[0].id, "run_script");
    }

    #[test]
    fn cat08_script_run_shortcut_is_enter() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
    }

    // =========================================================================
    // Category 09: Path context directory vs file primary action distinction
    // Verifies the different primary action IDs and titles for dirs vs files.
    // =========================================================================

    #[test]
    fn cat09_path_dir_primary_is_open_directory() {
        let info = PathInfo {
            path: "/users/test/Downloads".to_string(),
            is_dir: true,
            name: "Downloads".to_string(),
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].id, "open_directory");
    }

    #[test]
    fn cat09_path_file_primary_is_select_file() {
        let info = PathInfo {
            path: "/users/test/file.txt".to_string(),
            is_dir: false,
            name: "file.txt".to_string(),
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].id, "select_file");
    }

    #[test]
    fn cat09_path_dir_primary_shortcut_enter() {
        let info = PathInfo {
            path: "/users/test/Downloads".to_string(),
            is_dir: true,
            name: "Downloads".to_string(),
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
    }

    #[test]
    fn cat09_path_file_primary_shortcut_enter() {
        let info = PathInfo {
            path: "/users/test/file.txt".to_string(),
            is_dir: false,
            name: "file.txt".to_string(),
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
    }

    #[test]
    fn cat09_path_trash_always_last() {
        let info = PathInfo {
            path: "/users/test/Downloads".to_string(),
            is_dir: true,
            name: "Downloads".to_string(),
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions.last().unwrap().id, "move_to_trash");
    }

    // =========================================================================
    // Category 10: File context macOS-only action presence
    // Validates macOS-specific actions exist for file context on macOS.
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn cat10_file_macos_has_quick_look() {
        let info = FileInfo {
            path: "/users/test/photo.jpg".to_string(),
            is_dir: false,
            name: "photo.jpg".to_string(),
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "quick_look"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat10_file_macos_has_open_with() {
        let info = FileInfo {
            path: "/users/test/photo.jpg".to_string(),
            is_dir: false,
            name: "photo.jpg".to_string(),
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "open_with"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat10_file_macos_dir_no_quick_look() {
        let info = FileInfo {
            path: "/users/test/folder".to_string(),
            is_dir: true,
            name: "folder".to_string(),
            file_type: FileType::Directory,
        };
        let actions = get_file_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "quick_look"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat10_file_macos_has_show_info() {
        let info = FileInfo {
            path: "/users/test/photo.jpg".to_string(),
            is_dir: false,
            name: "photo.jpg".to_string(),
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "show_info"));
    }

    // =========================================================================
    // Category 11: AI command bar action section membership — exact IDs
    // Validates exact action IDs within each section of the AI command bar.
    // =========================================================================

    #[test]
    fn cat11_ai_response_section_ids() {
        let actions = get_ai_command_bar_actions();
        let response_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Response"))
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(
            response_ids,
            vec!["copy_response", "copy_chat", "copy_last_code"]
        );
    }

    #[test]
    fn cat11_ai_actions_section_ids() {
        let actions = get_ai_command_bar_actions();
        let action_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Actions"))
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(
            action_ids,
            vec!["submit", "new_chat", "delete_chat", "branch_from_last"]
        );
    }

    #[test]
    fn cat11_ai_attachments_section_ids() {
        let actions = get_ai_command_bar_actions();
        let attachment_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Attachments"))
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(attachment_ids, vec!["add_attachment", "paste_image"]);
    }

    #[test]
    fn cat11_ai_export_section_ids() {
        let actions = get_ai_command_bar_actions();
        let export_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Export"))
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(export_ids, vec!["export_markdown"]);
    }

    #[test]
    fn cat11_ai_help_and_settings_section_ids() {
        let actions = get_ai_command_bar_actions();
        let help_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Help"))
            .map(|a| a.id.as_str())
            .collect();
        let settings_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Settings"))
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(help_ids, vec!["toggle_shortcuts_help"]);
        assert_eq!(settings_ids, vec!["change_model"]);
    }

    // =========================================================================
    // Category 12: to_deeplink_name special character handling
    // Tests edge cases for special character replacement and collapsing.
    // =========================================================================

    #[test]
    fn cat12_deeplink_spaces_to_hyphens() {
        assert_eq!(to_deeplink_name("Hello World"), "hello-world");
    }

    #[test]
    fn cat12_deeplink_consecutive_specials_collapsed() {
        assert_eq!(to_deeplink_name("a--b__c  d"), "a-b-c-d");
    }

    #[test]
    fn cat12_deeplink_leading_trailing_stripped() {
        assert_eq!(to_deeplink_name("---hello---"), "hello");
    }

    #[test]
    fn cat12_deeplink_all_specials_returns_empty() {
        assert_eq!(to_deeplink_name("!@#$%^&*"), "");
    }

    #[test]
    fn cat12_deeplink_unicode_preserved() {
        let result = to_deeplink_name("日本語テスト");
        assert!(result.contains('日'));
        assert!(result.contains('語'));
    }

    #[test]
    fn cat12_deeplink_mixed_case_lowered() {
        assert_eq!(to_deeplink_name("MyScript"), "myscript");
    }

    // =========================================================================
    // Category 13: format_shortcut_hint modifier replacement
    // Tests that modifier keys are correctly replaced with symbols.
    // (format_shortcut_hint is private, so we test it indirectly via action shortcuts)
    // =========================================================================

    #[test]
    fn cat13_script_add_shortcut_uses_formatted_hint() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        let add_shortcut = actions.iter().find(|a| a.id == "add_shortcut").unwrap();
        // Expected: ⌘⇧K (cmd+shift+k formatted)
        assert_eq!(add_shortcut.shortcut.as_deref(), Some("⌘⇧K"));
    }

    #[test]
    fn cat13_script_edit_shortcut_formatted() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.shortcut.as_deref(), Some("⌘E"));
    }

    #[test]
    fn cat13_script_view_logs_shortcut_formatted() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        let logs = actions.iter().find(|a| a.id == "view_logs").unwrap();
        assert_eq!(logs.shortcut.as_deref(), Some("⌘L"));
    }

    #[test]
    fn cat13_clipboard_delete_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let del = actions.iter().find(|a| a.id == "clipboard_delete").unwrap();
        assert_eq!(del.shortcut.as_deref(), Some("⌃X"));
    }

    #[test]
    fn cat13_clipboard_delete_all_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let del_all = actions
            .iter()
            .find(|a| a.id == "clipboard_delete_all")
            .unwrap();
        assert_eq!(del_all.shortcut.as_deref(), Some("⌃⇧X"));
    }

    // =========================================================================
    // Category 14: score_action prefix vs contains vs fuzzy scoring
    // Tests the scoring function used for filtering actions.
    // =========================================================================

    #[test]
    fn cat14_score_prefix_match_100_plus() {
        let action = Action::new(
            "edit_script",
            "Edit Script",
            Some("Open in editor".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "edit");
        assert!(score >= 100, "Prefix match should be 100+, got {}", score);
    }

