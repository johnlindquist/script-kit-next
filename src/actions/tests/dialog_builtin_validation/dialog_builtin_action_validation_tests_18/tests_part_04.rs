    #[test]
    fn cat24_save_file_title() {
        let entry = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let save = actions
            .iter()
            .find(|a| a.id == "clipboard_save_file")
            .unwrap();
        assert_eq!(save.title, "Save as File...");
    }

    // =========================================================================
    // Category 25: Clipboard delete actions shortcuts
    // =========================================================================

    #[test]
    fn cat25_delete_entry_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let del = actions.iter().find(|a| a.id == "clipboard_delete").unwrap();
        assert_eq!(del.shortcut.as_ref().unwrap(), "⌃X");
    }

    #[test]
    fn cat25_delete_multiple_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let del = actions
            .iter()
            .find(|a| a.id == "clipboard_delete_multiple")
            .unwrap();
        assert_eq!(del.shortcut.as_ref().unwrap(), "⇧⌘X");
    }

    #[test]
    fn cat25_delete_all_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let del = actions
            .iter()
            .find(|a| a.id == "clipboard_delete_all")
            .unwrap();
        assert_eq!(del.shortcut.as_ref().unwrap(), "⌃⇧X");
    }

    #[test]
    fn cat25_delete_all_description_mentions_pinned() {
        let entry = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let del = actions
            .iter()
            .find(|a| a.id == "clipboard_delete_all")
            .unwrap();
        assert!(del.description.as_ref().unwrap().contains("pinned"));
    }

    // =========================================================================
    // Category 26: to_deeplink_name edge cases
    // =========================================================================

    #[test]
    fn cat26_numbers_only() {
        assert_eq!(to_deeplink_name("12345"), "12345");
    }

    #[test]
    fn cat26_all_special_chars_becomes_empty() {
        assert_eq!(to_deeplink_name("!@#$%^&*()"), "");
    }

    #[test]
    fn cat26_mixed_case_lowered() {
        assert_eq!(to_deeplink_name("CamelCase"), "camelcase");
    }

    #[test]
    fn cat26_consecutive_specials_collapsed() {
        assert_eq!(to_deeplink_name("hello---world"), "hello-world");
    }

    #[test]
    fn cat26_underscores_become_hyphens() {
        assert_eq!(to_deeplink_name("hello_world"), "hello-world");
    }

    #[test]
    fn cat26_leading_trailing_specials_stripped() {
        assert_eq!(to_deeplink_name("--hello--"), "hello");
    }

    #[test]
    fn cat26_unicode_preserved() {
        // CJK characters are alphanumeric in Unicode
        assert_eq!(to_deeplink_name("日本語"), "日本語");
    }

    // =========================================================================
    // Category 27: AI command bar per-section action counts
    // =========================================================================

    #[test]
    fn cat27_response_section_has_3_actions() {
        let actions = get_ai_command_bar_actions();
        let response_count = actions
            .iter()
            .filter(|a| a.section.as_ref() == Some(&"Response".to_string()))
            .count();
        assert_eq!(response_count, 3);
    }

    #[test]
    fn cat27_actions_section_has_4_actions() {
        let actions = get_ai_command_bar_actions();
        let actions_count = actions
            .iter()
            .filter(|a| a.section.as_ref() == Some(&"Actions".to_string()))
            .count();
        assert_eq!(actions_count, 4);
    }

    #[test]
    fn cat27_attachments_section_has_2_actions() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_ref() == Some(&"Attachments".to_string()))
            .count();
        assert_eq!(count, 2);
    }

    #[test]
    fn cat27_export_section_has_1_action() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_ref() == Some(&"Export".to_string()))
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn cat27_help_section_has_1_action() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_ref() == Some(&"Help".to_string()))
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn cat27_settings_section_has_1_action() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_ref() == Some(&"Settings".to_string()))
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn cat27_total_ai_actions_is_12() {
        let actions = get_ai_command_bar_actions();
        assert_eq!(actions.len(), 12);
    }

    // =========================================================================
    // Category 28: Notes command bar all flag combinations
    // =========================================================================

    #[test]
    fn cat28_full_feature_count() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Notes: new_note, duplicate_note, browse_notes
        // Edit: find_in_note, format
        // Copy: copy_note_as, copy_deeplink, create_quicklink
        // Export: export
        // Settings: enable_auto_sizing
        assert_eq!(actions.len(), 10);
    }

    #[test]
    fn cat28_auto_sizing_enabled_hides_setting() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert_eq!(actions.len(), 9);
        assert!(!actions.iter().any(|a| a.id == "enable_auto_sizing"));
    }

    #[test]
    fn cat28_trash_view_minimal() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Only new_note, browse_notes, enable_auto_sizing
        assert_eq!(actions.len(), 3);
    }

    #[test]
    fn cat28_no_selection_minimal() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Only new_note, browse_notes, enable_auto_sizing
        assert_eq!(actions.len(), 3);
    }

    #[test]
    fn cat28_trash_no_selection_auto_sizing() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Only new_note, browse_notes (auto_sizing hidden)
        assert_eq!(actions.len(), 2);
    }

    // =========================================================================
    // Category 29: Path context move_to_trash description formatting
    // =========================================================================

    #[test]
    fn cat29_trash_dir_description_says_folder() {
        let path = PathInfo {
            path: "/Users/test/Documents".to_string(),
            name: "Documents".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path);
        let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
        assert!(trash.description.as_ref().unwrap().contains("folder"));
    }

    #[test]
    fn cat29_trash_file_description_says_file() {
        let path = PathInfo {
            path: "/Users/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
        assert!(trash.description.as_ref().unwrap().contains("file"));
    }

    #[test]
    fn cat29_trash_shortcut() {
        let path = PathInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
        assert_eq!(trash.shortcut.as_ref().unwrap(), "⌘⌫");
    }

    #[test]
    fn cat29_trash_always_last() {
        let path = PathInfo {
            path: "/test/Documents".to_string(),
            name: "Documents".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path);
        let last = actions.last().unwrap();
        assert_eq!(last.id, "move_to_trash");
    }

    // =========================================================================
    // Category 30: Cross-context all actions have non-empty descriptions
    // =========================================================================

    #[test]
    fn cat30_script_actions_all_have_descriptions() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        for action in &actions {
            assert!(
                action.description.is_some(),
                "Script action '{}' should have description",
                action.id
            );
        }
    }

    #[test]
    fn cat30_clipboard_text_actions_all_have_descriptions() {
        let entry = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        for action in &actions {
            assert!(
                action.description.is_some(),
                "Clipboard action '{}' should have description",
                action.id
            );
        }
    }

    #[test]
    fn cat30_ai_actions_all_have_descriptions() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                action.description.is_some(),
                "AI action '{}' should have description",
                action.id
            );
        }
    }

    #[test]
    fn cat30_path_actions_all_have_descriptions() {
        let path = PathInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        for action in &actions {
            assert!(
                action.description.is_some(),
                "Path action '{}' should have description",
                action.id
            );
        }
    }

    #[test]
    fn cat30_file_actions_all_have_descriptions() {
        let file = FileInfo {
            path: "/test/doc.pdf".to_string(),
            name: "doc.pdf".to_string(),
            file_type: crate::file_search::FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        for action in &actions {
            assert!(
                action.description.is_some(),
                "File action '{}' should have description",
                action.id
            );
        }
    }

    #[test]
    fn cat30_notes_actions_all_have_descriptions() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for action in &actions {
            assert!(
                action.description.is_some(),
                "Notes action '{}' should have description",
                action.id
            );
        }
    }
