    #[test]
    fn cat07_model_description_uses_provider_display_name() {
        let models = vec![NewChatModelInfo {
            model_id: "gpt-4".to_string(),
            display_name: "GPT-4".to_string(),
            provider: "openai".to_string(),
            provider_display_name: "OpenAI".to_string(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        let m = actions.iter().find(|a| a.id == "model_0").unwrap();
        assert_eq!(m.description.as_ref().unwrap(), "OpenAI");
    }

    #[test]
    fn cat07_preset_has_no_description() {
        let presets = vec![NewChatPresetInfo {
            id: "general".to_string(),
            name: "General".to_string(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        let p = actions.iter().find(|a| a.id == "preset_general").unwrap();
        assert!(p.description.is_none());
    }

    #[test]
    fn cat07_all_sections_present_when_all_inputs_provided() {
        let last_used = vec![NewChatModelInfo {
            model_id: "c3".to_string(),
            display_name: "Claude 3".to_string(),
            provider: "anthropic".to_string(),
            provider_display_name: "Anthropic".to_string(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "code".to_string(),
            name: "Code".to_string(),
            icon: IconName::Code,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "gpt-4".to_string(),
            display_name: "GPT-4".to_string(),
            provider: "openai".to_string(),
            provider_display_name: "OpenAI".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        let sections: HashSet<_> = actions.iter().filter_map(|a| a.section.as_ref()).collect();
        assert!(sections.contains(&"Last Used Settings".to_string()));
        assert!(sections.contains(&"Presets".to_string()));
        assert!(sections.contains(&"Models".to_string()));
    }

    // =========================================================================
    // Category 08: Note switcher preview boundary — exactly 60 chars
    // =========================================================================

    #[test]
    fn cat08_preview_exactly_60_chars_no_ellipsis() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".to_string(),
            title: "Note".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "a".repeat(60),
            relative_time: String::new(),
        };
        let actions = get_note_switcher_actions(&[note]);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(!desc.contains('…'));
    }

    #[test]
    fn cat08_preview_61_chars_has_ellipsis() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".to_string(),
            title: "Note".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "a".repeat(61),
            relative_time: String::new(),
        };
        let actions = get_note_switcher_actions(&[note]);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains('…'));
    }

    #[test]
    fn cat08_preview_59_chars_no_ellipsis() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".to_string(),
            title: "Note".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "a".repeat(59),
            relative_time: String::new(),
        };
        let actions = get_note_switcher_actions(&[note]);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(!desc.contains('…'));
    }

    #[test]
    fn cat08_empty_preview_no_time_uses_char_count() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".to_string(),
            title: "Note".to_string(),
            char_count: 42,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        };
        let actions = get_note_switcher_actions(&[note]);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "42 chars");
    }

    // =========================================================================
    // Category 09: Notes command bar find_in_note details
    // =========================================================================

    #[test]
    fn cat09_find_in_note_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let find = actions.iter().find(|a| a.id == "find_in_note").unwrap();
        assert_eq!(find.shortcut.as_ref().unwrap(), "⌘F");
    }

    #[test]
    fn cat09_find_in_note_icon() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let find = actions.iter().find(|a| a.id == "find_in_note").unwrap();
        assert_eq!(find.icon, Some(IconName::MagnifyingGlass));
    }

    #[test]
    fn cat09_find_in_note_section() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let find = actions.iter().find(|a| a.id == "find_in_note").unwrap();
        assert_eq!(find.section.as_ref().unwrap(), "Edit");
    }

    #[test]
    fn cat09_find_in_note_absent_in_trash() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "find_in_note"));
    }

    #[test]
    fn cat09_find_in_note_absent_no_selection() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "find_in_note"));
    }

    // =========================================================================
    // Category 10: Notes command bar export details
    // =========================================================================

    #[test]
    fn cat10_export_present_with_selection_no_trash() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "export"));
    }

    #[test]
    fn cat10_export_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let export = actions.iter().find(|a| a.id == "export").unwrap();
        assert_eq!(export.shortcut.as_ref().unwrap(), "⇧⌘E");
    }

    #[test]
    fn cat10_export_section_is_export() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let export = actions.iter().find(|a| a.id == "export").unwrap();
        assert_eq!(export.section.as_ref().unwrap(), "Export");
    }

    #[test]
    fn cat10_export_icon() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let export = actions.iter().find(|a| a.id == "export").unwrap();
        assert_eq!(export.icon, Some(IconName::ArrowRight));
    }

    #[test]
    fn cat10_export_absent_in_trash() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "export"));
    }

    // =========================================================================
    // Category 11: Path context open_in_finder description
    // =========================================================================

    #[test]
    fn cat11_open_in_finder_description() {
        let path = PathInfo {
            path: "/Users/test/Documents".to_string(),
            name: "Documents".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path);
        let action = actions.iter().find(|a| a.id == "open_in_finder").unwrap();
        assert_eq!(action.description.as_ref().unwrap(), "Reveal in Finder");
    }

    #[test]
    fn cat11_open_in_finder_shortcut() {
        let path = PathInfo {
            path: "/Users/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        let action = actions.iter().find(|a| a.id == "open_in_finder").unwrap();
        assert_eq!(action.shortcut.as_ref().unwrap(), "⌘⇧F");
    }

    #[test]
    fn cat11_open_in_editor_description_mentions_editor() {
        let path = PathInfo {
            path: "/Users/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        let action = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
        assert!(action.description.as_ref().unwrap().contains("$EDITOR"));
    }

    #[test]
    fn cat11_open_in_terminal_shortcut() {
        let path = PathInfo {
            path: "/Users/test/Documents".to_string(),
            name: "Documents".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path);
        let action = actions.iter().find(|a| a.id == "open_in_terminal").unwrap();
        assert_eq!(action.shortcut.as_ref().unwrap(), "⌘T");
    }

    // =========================================================================
    // Category 12: File context exact descriptions
    // =========================================================================

    #[test]
    fn cat12_file_open_description() {
        let file = FileInfo {
            path: "/test/doc.pdf".to_string(),
            name: "doc.pdf".to_string(),
            file_type: crate::file_search::FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let open = actions.iter().find(|a| a.id == "open_file").unwrap();
        assert_eq!(
            open.description.as_ref().unwrap(),
            "Open with default application"
        );
    }

    #[test]
    fn cat12_dir_open_description() {
        let dir = FileInfo {
            path: "/test/folder".to_string(),
            name: "folder".to_string(),
            file_type: crate::file_search::FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&dir);
        let open = actions.iter().find(|a| a.id == "open_directory").unwrap();
        assert_eq!(open.description.as_ref().unwrap(), "Open this folder");
    }

    #[test]
    fn cat12_reveal_in_finder_description() {
        let file = FileInfo {
            path: "/test/doc.pdf".to_string(),
            name: "doc.pdf".to_string(),
            file_type: crate::file_search::FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
        assert_eq!(reveal.description.as_ref().unwrap(), "Reveal in Finder");
    }

    #[test]
    fn cat12_copy_path_description() {
        let file = FileInfo {
            path: "/test/doc.pdf".to_string(),
            name: "doc.pdf".to_string(),
            file_type: crate::file_search::FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
        assert_eq!(
            cp.description.as_ref().unwrap(),
            "Copy the full path to clipboard"
        );
    }

    #[test]
    fn cat12_copy_filename_description() {
        let file = FileInfo {
            path: "/test/doc.pdf".to_string(),
            name: "doc.pdf".to_string(),
            file_type: crate::file_search::FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
        assert_eq!(
            cf.description.as_ref().unwrap(),
            "Copy just the filename to clipboard"
        );
    }

    // =========================================================================
    // Category 13: format_shortcut_hint edge cases
    // =========================================================================

    #[test]
    fn cat13_control_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("control+c"), "⌃C");
    }

    #[test]
    fn cat13_super_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("super+c"), "⌘C");
    }

    #[test]
    fn cat13_esc_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("esc"), "⎋");
    }

    #[test]
    fn cat13_tab_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("tab"), "⇥");
    }

    #[test]
    fn cat13_backspace_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("backspace"), "⌫");
    }

    #[test]
    fn cat13_delete_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("delete"), "⌫");
    }

    #[test]
    fn cat13_space_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("space"), "␣");
    }

    #[test]
    fn cat13_arrowleft_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("arrowleft"), "←");
    }

    #[test]
    fn cat13_arrowright_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("arrowright"), "→");
    }

    // =========================================================================
    // Category 14: parse_shortcut_keycaps — all symbol types
    // =========================================================================

    #[test]
    fn cat14_space_symbol() {
        let caps = ActionsDialog::parse_shortcut_keycaps("␣");
        assert_eq!(caps, vec!["␣"]);
    }

    #[test]
    fn cat14_backspace_symbol() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌫");
        assert_eq!(caps, vec!["⌫"]);
    }

    #[test]
    fn cat14_tab_symbol() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⇥");
        assert_eq!(caps, vec!["⇥"]);
    }

    #[test]
    fn cat14_escape_symbol() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⎋");
        assert_eq!(caps, vec!["⎋"]);
    }

