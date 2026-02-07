    use crate::actions::builders::*;
    use crate::actions::dialog::{
        build_grouped_items_static, coerce_action_selection, ActionsDialog, GroupedActionItem,
    };
    use crate::actions::types::*;
    use crate::actions::CommandBarConfig;
    use crate::clipboard_history::ContentType;
    use crate::designs::icon_variations::IconName;
    use crate::file_search::{FileInfo, FileType};
    use crate::prompts::PathInfo;
    use crate::scriptlets::{Scriptlet, ScriptletAction};

    // =========================================================================
    // Helper: collect action IDs from a Vec<Action>
    // =========================================================================
    fn action_ids(actions: &[Action]) -> Vec<&str> {
        actions.iter().map(|a| a.id.as_str()).collect()
    }

    fn action_titles(actions: &[Action]) -> Vec<&str> {
        actions.iter().map(|a| a.title.as_str()).collect()
    }

    // =========================================================================
    // 1. Note switcher description rendering (preview + relative_time combos)
    // =========================================================================

    #[test]
    fn note_switcher_desc_preview_and_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "My Note".into(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "Hello world snippet".into(),
            relative_time: "5m ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(
            desc.contains("Hello world snippet"),
            "Description should contain preview"
        );
        assert!(desc.contains("5m ago"), "Description should contain time");
        assert!(desc.contains(" · "), "Preview and time joined with ' · '");
    }

    #[test]
    fn note_switcher_desc_preview_only_no_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n2".into(),
            title: "Note".into(),
            char_count: 50,
            is_current: false,
            is_pinned: false,
            preview: "Some preview text".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "Some preview text");
        assert!(!desc.contains(" · "), "No separator when no time");
    }

    #[test]
    fn note_switcher_desc_time_only_no_preview() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n3".into(),
            title: "Note".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "1h ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "1h ago");
    }

    #[test]
    fn note_switcher_desc_no_preview_no_time_zero_chars() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n4".into(),
            title: "Empty".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "0 chars");
    }

    #[test]
    fn note_switcher_desc_no_preview_no_time_one_char() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n5".into(),
            title: "Tiny".into(),
            char_count: 1,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "1 char");
    }

    #[test]
    fn note_switcher_desc_no_preview_no_time_many_chars() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n6".into(),
            title: "Long".into(),
            char_count: 42,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "42 chars");
    }

    #[test]
    fn note_switcher_preview_truncated_at_60_chars() {
        let long_preview = "A".repeat(80);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n7".into(),
            title: "Long Preview".into(),
            char_count: 80,
            is_current: false,
            is_pinned: false,
            preview: long_preview,
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        // Should be 60 A's followed by "…"
        assert!(
            desc.ends_with('…'),
            "Long preview should be truncated with ellipsis"
        );
        // Count: 60 A's + "…" = 61 chars
        assert_eq!(desc.chars().count(), 61);
    }

    #[test]
    fn note_switcher_preview_exactly_60_chars_not_truncated() {
        let exact_preview = "B".repeat(60);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n8".into(),
            title: "Exact".into(),
            char_count: 60,
            is_current: false,
            is_pinned: false,
            preview: exact_preview.clone(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, &exact_preview);
        assert!(!desc.ends_with('…'));
    }

    #[test]
    fn note_switcher_preview_61_chars_truncated() {
        let preview_61 = "C".repeat(61);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n9".into(),
            title: "Just Over".into(),
            char_count: 61,
            is_current: false,
            is_pinned: false,
            preview: preview_61,
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.ends_with('…'));
    }

    #[test]
    fn note_switcher_preview_truncated_with_time() {
        let long_preview = "D".repeat(80);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n10".into(),
            title: "Truncated + Time".into(),
            char_count: 80,
            is_current: false,
            is_pinned: false,
            preview: long_preview,
            relative_time: "2d ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains("…"));
        assert!(desc.contains("2d ago"));
        assert!(desc.contains(" · "));
    }

    // =========================================================================
    // 2. Clipboard action position invariants (beyond first/last)
    // =========================================================================

    #[test]
    fn clipboard_text_paste_keep_open_is_third() {
        let entry = ClipboardEntryInfo {
            id: "e1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[2].id, "clipboard_paste_keep_open");
    }

    #[test]
    fn clipboard_text_share_is_fourth() {
        let entry = ClipboardEntryInfo {
            id: "e2".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[3].id, "clipboard_share");
    }

    #[test]
    fn clipboard_text_attach_ai_is_fifth() {
        let entry = ClipboardEntryInfo {
            id: "e3".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[4].id, "clipboard_attach_to_ai");
    }

    #[test]
    fn clipboard_save_snippet_before_save_file() {
        let entry = ClipboardEntryInfo {
            id: "e4".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        let snippet_pos = ids
            .iter()
            .position(|&id| id == "clipboard_save_snippet")
            .unwrap();
        let file_pos = ids
            .iter()
            .position(|&id| id == "clipboard_save_file")
            .unwrap();
        assert!(
            snippet_pos < file_pos,
            "Save snippet should come before save file"
        );
    }

    #[test]
    fn clipboard_delete_order_is_single_then_multiple_then_all() {
        let entry = ClipboardEntryInfo {
            id: "e5".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        let del_pos = ids.iter().position(|&id| id == "clipboard_delete").unwrap();
        let del_multi_pos = ids
            .iter()
            .position(|&id| id == "clipboard_delete_multiple")
            .unwrap();
        let del_all_pos = ids
            .iter()
            .position(|&id| id == "clipboard_delete_all")
            .unwrap();
        assert!(del_pos < del_multi_pos);
        assert!(del_multi_pos < del_all_pos);
    }

    #[test]
    fn clipboard_text_unpinned_action_count() {
        let entry = ClipboardEntryInfo {
            id: "e6".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        // On macOS: paste, copy, paste_keep_open, share, attach_ai, quick_look,
        //           pin, save_snippet, save_file, delete, delete_multiple, delete_all = 12
        // Non-macOS: no quick_look = 11
        #[cfg(target_os = "macos")]
        assert_eq!(actions.len(), 12);
        #[cfg(not(target_os = "macos"))]
        assert_eq!(actions.len(), 11);
    }

    #[test]
    fn clipboard_image_pinned_action_count() {
        let entry = ClipboardEntryInfo {
            id: "e7".into(),
            content_type: ContentType::Image,
            pinned: true,
            preview: "img".into(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        // On macOS: paste, copy, paste_keep_open, share, attach_ai, quick_look,
        //           open_with, annotate_cleanshot, upload_cleanshot, unpin,
        //           ocr, save_snippet, save_file, delete, delete_multiple, delete_all = 16
        #[cfg(target_os = "macos")]
        assert_eq!(actions.len(), 16);
        // Non-macOS: no quick_look, open_with, annotate_cleanshot, upload_cleanshot = 12
        #[cfg(not(target_os = "macos"))]
        assert_eq!(actions.len(), 12);
    }

    // =========================================================================
    // 3. AI command bar section item counts
    // =========================================================================

    #[test]
    fn ai_command_bar_response_section_has_3_actions() {
        let actions = get_ai_command_bar_actions();
        let response_count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Response"))
            .count();
        assert_eq!(response_count, 3);
    }

    #[test]
    fn ai_command_bar_actions_section_has_4_actions() {
        let actions = get_ai_command_bar_actions();
        let actions_count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Actions"))
            .count();
        assert_eq!(actions_count, 4);
    }

    #[test]
    fn ai_command_bar_attachments_section_has_2_actions() {
        let actions = get_ai_command_bar_actions();
        let attach_count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Attachments"))
            .count();
        assert_eq!(attach_count, 2);
    }

    #[test]
    fn ai_command_bar_settings_section_has_1_action() {
        let actions = get_ai_command_bar_actions();
        let settings_count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Settings"))
            .count();
        assert_eq!(settings_count, 1);
    }

    #[test]
    fn ai_command_bar_total_is_12() {
        let actions = get_ai_command_bar_actions();
        assert_eq!(actions.len(), 12);
    }

    #[test]
    fn ai_command_bar_section_order_preserved() {
        let actions = get_ai_command_bar_actions();
        let sections: Vec<&str> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        // Should transition: Response, Actions, Attachments, Export, Actions, Help, Settings
        let mut seen_sections: Vec<&str> = Vec::new();
        for s in &sections {
            if seen_sections.last() != Some(s) {
                seen_sections.push(s);
            }
        }
        assert_eq!(
            seen_sections,
            vec![
                "Response",
                "Actions",
                "Attachments",
                "Export",
                "Actions",
                "Help",
                "Settings"
            ]
        );
    }

    #[test]
    fn ai_command_bar_all_have_icons() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                action.icon.is_some(),
                "AI action '{}' should have an icon",
                action.id
            );
        }
    }

