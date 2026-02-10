    #[test]
    fn cat22_pin_unpin_same_shortcut() {
        let pinned_entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let unpinned_entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
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
        assert_eq!(pin.shortcut.as_deref(), Some("⇧⌘P"));
    }

    #[test]
    fn cat22_pin_title_and_description() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let pin = actions.iter().find(|a| a.id == "clipboard_pin").unwrap();
        assert_eq!(pin.title, "Pin Entry");
        assert!(pin
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("pin"));
    }

    // =========================================================================
    // Category 23: Clipboard save actions — snippet and file shortcuts
    // Validates save snippet and save file action details.
    // =========================================================================

    #[test]
    fn cat23_save_snippet_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let snippet = actions
            .iter()
            .find(|a| a.id == "clipboard_save_snippet")
            .unwrap();
        assert_eq!(snippet.shortcut.as_deref(), Some("⇧⌘S"));
    }

    #[test]
    fn cat23_save_file_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let file = actions
            .iter()
            .find(|a| a.id == "clipboard_save_file")
            .unwrap();
        assert_eq!(file.shortcut.as_deref(), Some("⌥⇧⌘S"));
    }

    #[test]
    fn cat23_save_snippet_title() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let snippet = actions
            .iter()
            .find(|a| a.id == "clipboard_save_snippet")
            .unwrap();
        assert_eq!(snippet.title, "Save Text as Snippet");
    }

    #[test]
    fn cat23_save_file_title() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let file = actions
            .iter()
            .find(|a| a.id == "clipboard_save_file")
            .unwrap();
        assert_eq!(file.title, "Save as File...");
    }

    // =========================================================================
    // Category 24: Script context shortcut count — add vs update/remove
    // Validates exact action count difference between no-shortcut and with-shortcut.
    // =========================================================================

    #[test]
    fn cat24_no_shortcut_has_add_only() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        let shortcut_actions: Vec<&Action> = actions
            .iter()
            .filter(|a| {
                a.id == "add_shortcut" || a.id == "update_shortcut" || a.id == "remove_shortcut"
            })
            .collect();
        assert_eq!(shortcut_actions.len(), 1);
        assert_eq!(shortcut_actions[0].id, "add_shortcut");
    }

    #[test]
    fn cat24_with_shortcut_has_update_and_remove() {
        let script = ScriptInfo::with_shortcut("test", "/path/test.ts", Some("cmd+t".to_string()));
        let actions = get_script_context_actions(&script);
        let shortcut_actions: Vec<&Action> = actions
            .iter()
            .filter(|a| {
                a.id == "add_shortcut" || a.id == "update_shortcut" || a.id == "remove_shortcut"
            })
            .collect();
        assert_eq!(shortcut_actions.len(), 2);
        let ids: HashSet<&str> = shortcut_actions.iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains("update_shortcut"));
        assert!(ids.contains("remove_shortcut"));
    }

    #[test]
    fn cat24_with_shortcut_one_more_action() {
        let no_shortcut = ScriptInfo::new("test", "/path/test.ts");
        let with_shortcut =
            ScriptInfo::with_shortcut("test", "/path/test.ts", Some("cmd+t".to_string()));
        let count_no = get_script_context_actions(&no_shortcut).len();
        let count_with = get_script_context_actions(&with_shortcut).len();
        assert_eq!(count_with, count_no + 1); // update + remove = add + 1
    }

    #[test]
    fn cat24_same_pattern_for_alias() {
        let no_alias = ScriptInfo::new("test", "/path/test.ts");
        let with_alias = ScriptInfo::with_shortcut_and_alias(
            "test",
            "/path/test.ts",
            None,
            Some("ts".to_string()),
        );
        let count_no = get_script_context_actions(&no_alias).len();
        let count_with = get_script_context_actions(&with_alias).len();
        assert_eq!(count_with, count_no + 1);
    }

    // =========================================================================
    // Category 25: Note switcher icon assignment — pinned > current > default
    // Verifies the icon priority: StarFilled > Check > File.
    // =========================================================================

    #[test]
    fn cat25_pinned_gets_star_filled() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "Test".to_string(),
            char_count: 10,
            is_current: false,
            is_pinned: true,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    #[test]
    fn cat25_current_gets_check() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "Test".to_string(),
            char_count: 10,
            is_current: true,
            is_pinned: false,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::Check));
    }

    #[test]
    fn cat25_regular_gets_file() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "Test".to_string(),
            char_count: 10,
            is_current: false,
            is_pinned: false,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::File));
    }

    #[test]
    fn cat25_pinned_and_current_prefers_star() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "Test".to_string(),
            char_count: 10,
            is_current: true,
            is_pinned: true,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    #[test]
    fn cat25_note_switcher_empty_placeholder_icon() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions[0].icon, Some(IconName::Plus));
    }

    // =========================================================================
    // Category 26: Note switcher section assignment — Pinned vs Recent
    // Validates that pinned notes go to "Pinned" section and others to "Recent".
    // =========================================================================

    #[test]
    fn cat26_pinned_section() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "Pinned Note".to_string(),
            char_count: 10,
            is_current: false,
            is_pinned: true,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
    }

    #[test]
    fn cat26_unpinned_section() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "Recent Note".to_string(),
            char_count: 10,
            is_current: false,
            is_pinned: false,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn cat26_mixed_notes_correct_sections() {
        let notes = vec![
            NoteSwitcherNoteInfo {
                id: "a".to_string(),
                title: "Pinned".to_string(),
                char_count: 10,
                is_current: false,
                is_pinned: true,
                preview: "".to_string(),
                relative_time: "".to_string(),
            },
            NoteSwitcherNoteInfo {
                id: "b".to_string(),
                title: "Recent".to_string(),
                char_count: 20,
                is_current: false,
                is_pinned: false,
                preview: "".to_string(),
                relative_time: "".to_string(),
            },
        ];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
        assert_eq!(actions[1].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn cat26_empty_notes_placeholder_section() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions[0].section.as_deref(), Some("Notes"));
    }

    // =========================================================================
    // Category 27: Note switcher current indicator — bullet prefix
    // Validates the "• " prefix for current notes.
    // =========================================================================

    #[test]
    fn cat27_current_note_has_bullet() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "My Note".to_string(),
            char_count: 10,
            is_current: true,
            is_pinned: false,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(actions[0].title.starts_with("• "));
    }

    #[test]
    fn cat27_non_current_no_bullet() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "My Note".to_string(),
            char_count: 10,
            is_current: false,
            is_pinned: false,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(!actions[0].title.starts_with("• "));
        assert_eq!(actions[0].title, "My Note");
    }

    #[test]
    fn cat27_current_and_pinned_has_bullet() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "My Note".to_string(),
            char_count: 10,
            is_current: true,
            is_pinned: true,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(actions[0].title.starts_with("• "));
    }

    // =========================================================================
    // Category 28: Note switcher preview truncation boundary
    // Tests exact truncation at the 60-character boundary with ellipsis.
    // =========================================================================

    #[test]
    fn cat28_preview_exactly_60_no_ellipsis() {
        let preview = "a".repeat(60);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "Test".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview,
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(!desc.contains('…'));
        assert_eq!(desc.len(), 60);
    }

    #[test]
    fn cat28_preview_61_has_ellipsis() {
        let preview = "a".repeat(61);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "Test".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview,
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains('…'));
    }

    #[test]
    fn cat28_preview_59_no_ellipsis() {
        let preview = "b".repeat(59);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "Test".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview,
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(!desc.contains('…'));
    }

    // =========================================================================
    // Category 29: Cross-context has_action=false invariant
    // Validates that all built-in actions have has_action=false.
    // =========================================================================

