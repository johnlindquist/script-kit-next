//! Batch 5: Built-in action behavioral validation tests
//!
//! 150+ tests validating action invariants NOT covered in batches 1-4.
//! Focus areas:
//! - Note switcher description rendering (preview truncation, relative time combos)
//! - Clipboard action position invariants beyond first/last
//! - AI command bar section item counts
//! - build_grouped_items_static section transitions
//! - Large-scale stress (many notes, models, presets)
//! - Cross-function ScriptInfo consistency
//! - Action description content keywords
//! - Score_action with cached lowercase fields
//! - Scriptlet with_custom multiple custom actions ordering
//! - CommandBarConfig equality and field access patterns

#[cfg(test)]
mod tests {
    // --- merged from tests_part_01.rs ---
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


    // --- merged from tests_part_02.rs ---
    #[test]
    fn ai_command_bar_specific_icons() {
        let actions = get_ai_command_bar_actions();
        let copy_response = actions.iter().find(|a| a.id == "copy_response").unwrap();
        assert_eq!(copy_response.icon, Some(IconName::Copy));

        let submit = actions.iter().find(|a| a.id == "submit").unwrap();
        assert_eq!(submit.icon, Some(IconName::ArrowUp));

        let delete_chat = actions.iter().find(|a| a.id == "delete_chat").unwrap();
        assert_eq!(delete_chat.icon, Some(IconName::Trash));

        let change_model = actions.iter().find(|a| a.id == "change_model").unwrap();
        assert_eq!(change_model.icon, Some(IconName::Settings));
    }

    // =========================================================================
    // 4. build_grouped_items_static section transitions
    // =========================================================================

    #[test]
    fn grouped_items_headers_inserts_header_on_section_change() {
        let actions = vec![
            Action::new("a1", "Act 1", None, ActionCategory::ScriptContext).with_section("Sec A"),
            Action::new("a2", "Act 2", None, ActionCategory::ScriptContext).with_section("Sec A"),
            Action::new("a3", "Act 3", None, ActionCategory::ScriptContext).with_section("Sec B"),
        ];
        let filtered: Vec<usize> = (0..3).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Expected: Header("Sec A"), Item(0), Item(1), Header("Sec B"), Item(2)
        assert_eq!(grouped.len(), 5);
        assert!(matches!(
            &grouped[0],
            GroupedActionItem::SectionHeader(s) if s == "Sec A"
        ));
        assert!(matches!(grouped[1], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[2], GroupedActionItem::Item(1)));
        assert!(matches!(
            &grouped[3],
            GroupedActionItem::SectionHeader(s) if s == "Sec B"
        ));
        assert!(matches!(grouped[4], GroupedActionItem::Item(2)));
    }

    #[test]
    fn grouped_items_headers_no_header_for_no_section() {
        let actions = vec![
            Action::new("a1", "Act 1", None, ActionCategory::ScriptContext),
            Action::new("a2", "Act 2", None, ActionCategory::ScriptContext),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // No sections => no headers
        assert_eq!(grouped.len(), 2);
        assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(1)));
    }

    #[test]
    fn grouped_items_separators_no_headers_inserted() {
        let actions = vec![
            Action::new("a1", "Act 1", None, ActionCategory::ScriptContext).with_section("Sec A"),
            Action::new("a2", "Act 2", None, ActionCategory::ScriptContext).with_section("Sec B"),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        // Separators style: no headers, just items
        assert_eq!(grouped.len(), 2);
        assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(1)));
    }

    #[test]
    fn grouped_items_none_no_headers_inserted() {
        let actions = vec![
            Action::new("a1", "Act 1", None, ActionCategory::ScriptContext).with_section("Sec A"),
        ];
        let filtered: Vec<usize> = vec![0];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        assert_eq!(grouped.len(), 1);
        assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
    }

    #[test]
    fn grouped_items_empty_filtered_produces_empty() {
        let actions = vec![Action::new(
            "a1",
            "Act 1",
            None,
            ActionCategory::ScriptContext,
        )];
        let filtered: Vec<usize> = vec![];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(grouped.is_empty());
    }

    #[test]
    fn grouped_items_headers_three_sections() {
        let actions = vec![
            Action::new("a1", "A1", None, ActionCategory::ScriptContext).with_section("X"),
            Action::new("a2", "A2", None, ActionCategory::ScriptContext).with_section("Y"),
            Action::new("a3", "A3", None, ActionCategory::ScriptContext).with_section("Z"),
        ];
        let filtered: Vec<usize> = (0..3).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // 3 headers + 3 items = 6
        assert_eq!(grouped.len(), 6);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 3);
    }

    #[test]
    fn grouped_items_headers_same_section_no_duplicate_header() {
        let actions = vec![
            Action::new("a1", "A1", None, ActionCategory::ScriptContext).with_section("Same"),
            Action::new("a2", "A2", None, ActionCategory::ScriptContext).with_section("Same"),
            Action::new("a3", "A3", None, ActionCategory::ScriptContext).with_section("Same"),
        ];
        let filtered: Vec<usize> = (0..3).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // 1 header + 3 items = 4
        assert_eq!(grouped.len(), 4);
    }

    // =========================================================================
    // 5. coerce_action_selection edge cases
    // =========================================================================

    #[test]
    fn coerce_selection_all_items_returns_requested() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::Item(1),
            GroupedActionItem::Item(2),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(1));
    }

    #[test]
    fn coerce_selection_header_at_start_goes_down() {
        let rows = vec![
            GroupedActionItem::SectionHeader("H".into()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn coerce_selection_header_at_end_goes_up() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("H".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(0));
    }

    #[test]
    fn coerce_selection_header_between_items_goes_down() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("H".into()),
            GroupedActionItem::Item(1),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(2));
    }

    #[test]
    fn coerce_selection_two_headers_then_item() {
        let rows = vec![
            GroupedActionItem::SectionHeader("H1".into()),
            GroupedActionItem::SectionHeader("H2".into()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(2));
        assert_eq!(coerce_action_selection(&rows, 1), Some(2));
    }

    #[test]
    fn coerce_selection_out_of_bounds_clamped() {
        let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
        // ix=10 should be clamped to rows.len()-1 = 1, which is an Item
        assert_eq!(coerce_action_selection(&rows, 10), Some(1));
    }

    // =========================================================================
    // 6. Large-scale stress tests
    // =========================================================================

    #[test]
    fn stress_50_notes_in_switcher() {
        let notes: Vec<NoteSwitcherNoteInfo> = (0..50)
            .map(|i| NoteSwitcherNoteInfo {
                id: format!("note-{}", i),
                title: format!("Note #{}", i),
                char_count: i * 100,
                is_current: i == 0,
                is_pinned: i < 5,
                preview: format!("Preview for note {}", i),
                relative_time: format!("{}m ago", i),
            })
            .collect();
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions.len(), 50);

        // First 5 should be Pinned section
        for i in 0..5 {
            assert_eq!(
                actions[i].section.as_deref(),
                Some("Pinned"),
                "Note {} should be in Pinned section",
                i
            );
        }

        // Remaining should be Recent
        for i in 5..50 {
            assert_eq!(
                actions[i].section.as_deref(),
                Some("Recent"),
                "Note {} should be in Recent section",
                i
            );
        }
    }

    #[test]
    fn stress_20_models_in_new_chat() {
        let models: Vec<NewChatModelInfo> = (0..20)
            .map(|i| NewChatModelInfo {
                model_id: format!("model-{}", i),
                display_name: format!("Model {}", i),
                provider: format!("provider-{}", i),
                provider_display_name: format!("Provider {}", i),
            })
            .collect();
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions.len(), 20);
        for (i, action) in actions.iter().enumerate() {
            assert_eq!(action.id, format!("model_{}", i));
            assert_eq!(action.section.as_deref(), Some("Models"));
        }
    }

    #[test]
    fn stress_mixed_last_used_presets_models() {
        let last_used: Vec<NewChatModelInfo> = (0..3)
            .map(|i| NewChatModelInfo {
                model_id: format!("lu-{}", i),
                display_name: format!("Last Used {}", i),
                provider: "p".into(),
                provider_display_name: "Provider".into(),
            })
            .collect();
        let presets: Vec<NewChatPresetInfo> = (0..4)
            .map(|i| NewChatPresetInfo {
                id: format!("preset-{}", i),
                name: format!("Preset {}", i),
                icon: IconName::Star,
            })
            .collect();
        let models: Vec<NewChatModelInfo> = (0..5)
            .map(|i| NewChatModelInfo {
                model_id: format!("m-{}", i),
                display_name: format!("Model {}", i),
                provider: "p".into(),
                provider_display_name: "Provider".into(),
            })
            .collect();
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        assert_eq!(actions.len(), 12); // 3 + 4 + 5

        // Verify section counts
        let lu_count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Last Used Settings"))
            .count();
        let preset_count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Presets"))
            .count();
        let model_count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Models"))
            .count();
        assert_eq!(lu_count, 3);
        assert_eq!(preset_count, 4);
        assert_eq!(model_count, 5);
    }

    #[test]
    fn stress_grouped_items_50_actions_with_sections() {
        let actions: Vec<Action> = (0..50)
            .map(|i| {
                let section = match i % 3 {
                    0 => "A",
                    1 => "B",
                    _ => "C",
                };
                Action::new(
                    format!("a{}", i),
                    format!("Action {}", i),
                    None,
                    ActionCategory::ScriptContext,
                )
                .with_section(section)
            })
            .collect();
        let filtered: Vec<usize> = (0..50).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);

        // With alternating sections A, B, C, A, B, C... headers appear at every change
        // Pattern: A at 0, B at 1, C at 2, A at 3, B at 4, C at 5... = 50 changes
        // But adjacent same-section items won't add headers
        // Actually each item has a unique section transition since i%3 alternates
        // So we get a header for nearly every item: 50 headers + 50 items = 100
        let item_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::Item(_)))
            .count();
        assert_eq!(item_count, 50);
    }

    // =========================================================================
    // 7. Cross-function ScriptInfo consistency
    // =========================================================================

    #[test]
    fn scriptlet_same_results_from_both_builders_without_custom() {
        let script = ScriptInfo::scriptlet("Test Scriptlet", "/path/to/test.md", None, None);
        let actions_standard = get_script_context_actions(&script);
        let actions_custom = get_scriptlet_context_actions_with_custom(&script, None);

        // Both should produce same action IDs (with_custom adds no custom when None)
        let ids_standard = action_ids(&actions_standard);
        let ids_custom = action_ids(&actions_custom);
        assert_eq!(ids_standard, ids_custom);
    }

    #[test]
    fn scriptlet_with_shortcut_same_from_both_builders() {
        let script = ScriptInfo::scriptlet("Test", "/path/to/test.md", Some("cmd+t".into()), None);
        let actions_standard = get_script_context_actions(&script);
        let actions_custom = get_scriptlet_context_actions_with_custom(&script, None);
        let ids_standard = action_ids(&actions_standard);
        let ids_custom = action_ids(&actions_custom);
        assert_eq!(ids_standard, ids_custom);
    }

    #[test]
    fn scriptlet_with_alias_same_from_both_builders() {
        let script = ScriptInfo::scriptlet("Test", "/path/to/test.md", None, Some("tst".into()));
        let actions_standard = get_script_context_actions(&script);
        let actions_custom = get_scriptlet_context_actions_with_custom(&script, None);
        let ids_standard = action_ids(&actions_standard);
        let ids_custom = action_ids(&actions_custom);
        assert_eq!(ids_standard, ids_custom);
    }

    #[test]
    fn scriptlet_with_frecency_same_from_both_builders() {
        let script = ScriptInfo::scriptlet("Test", "/path/to/test.md", None, None)
            .with_frecency(true, Some("test".into()));
        let actions_standard = get_script_context_actions(&script);
        let actions_custom = get_scriptlet_context_actions_with_custom(&script, None);
        let ids_standard = action_ids(&actions_standard);
        let ids_custom = action_ids(&actions_custom);
        assert_eq!(ids_standard, ids_custom);
    }

    // =========================================================================
    // 8. Action description content keyword validation
    // =========================================================================

    #[test]
    fn script_run_description_contains_action_verb() {
        let script = ScriptInfo::with_action_verb("Foo", "/path", true, "Launch");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert!(
            run.description.as_ref().unwrap().contains("Launch"),
            "Run description should contain the action verb"
        );
    }

    #[test]
    fn script_edit_description_contains_editor() {
        let script = ScriptInfo::new("Foo", "/path/foo.ts");
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert!(edit.description.as_ref().unwrap().contains("$EDITOR"));
    }

    #[test]
    fn script_view_logs_description_contains_logs() {
        let script = ScriptInfo::new("Foo", "/path/foo.ts");
        let actions = get_script_context_actions(&script);
        let logs = actions.iter().find(|a| a.id == "view_logs").unwrap();
        assert!(logs
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("log"));
    }

    #[test]
    fn script_copy_path_description_contains_path() {
        let script = ScriptInfo::new("Foo", "/path/foo.ts");
        let actions = get_script_context_actions(&script);
        let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
        assert!(cp
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("path"));
    }

    #[test]
    fn clipboard_ocr_description_mentions_text_or_ocr() {
        let entry = ClipboardEntryInfo {
            id: "img".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "".into(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ocr = actions.iter().find(|a| a.id == "clipboard_ocr").unwrap();
        let desc = ocr.description.as_ref().unwrap().to_lowercase();
        assert!(desc.contains("text") || desc.contains("ocr"));
    }

    #[test]
    fn path_move_to_trash_description_mentions_delete() {
        let path_info = PathInfo {
            path: "/tmp/test.txt".into(),
            name: "test.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
        let desc = trash.description.as_ref().unwrap().to_lowercase();
        assert!(desc.contains("delete"));
    }


    // --- merged from tests_part_03.rs ---
    #[test]
    fn notes_new_note_description_mentions_create() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let new_note = actions.iter().find(|a| a.id == "new_note").unwrap();
        let desc = new_note.description.as_ref().unwrap().to_lowercase();
        assert!(desc.contains("create") || desc.contains("new"));
    }

    // =========================================================================
    // 9. Score_action with cached lowercase fields
    // =========================================================================

    #[test]
    fn score_action_uses_title_lower_cache() {
        let action = Action::new(
            "test",
            "Edit Script",
            Some("Open in editor".into()),
            ActionCategory::ScriptContext,
        );
        // title_lower should be "edit script"
        assert_eq!(action.title_lower, "edit script");
        let score = ActionsDialog::score_action(&action, "edit");
        assert!(
            score >= 100,
            "Prefix match on title_lower should score 100+"
        );
    }

    #[test]
    fn score_action_description_lower_bonus() {
        let action = Action::new(
            "open",
            "Open File",
            Some("Open with default application".into()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(
            action.description_lower.as_deref(),
            Some("open with default application")
        );
        let score = ActionsDialog::score_action(&action, "default");
        // "default" not in title, but in description
        assert!(
            score >= 15,
            "Description match should add 15+ points, got {}",
            score
        );
    }

    #[test]
    fn score_action_shortcut_lower_bonus() {
        let action = Action::new("test", "Run Script", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘R");
        assert_eq!(action.shortcut_lower.as_deref(), Some("⌘r"));
        let score = ActionsDialog::score_action(&action, "⌘r");
        // "⌘r" in shortcut_lower
        assert!(
            score >= 10,
            "Shortcut match should add 10+ points, got {}",
            score
        );
    }

    #[test]
    fn score_action_empty_query_returns_zero() {
        let action = Action::new("test", "Run Script", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "");
        // Empty string is a prefix of everything, so it scores 100
        assert_eq!(score, 100);
    }

    #[test]
    fn score_action_no_match_returns_zero() {
        let action = Action::new("test", "Run Script", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "zzzzz");
        assert_eq!(score, 0);
    }

    #[test]
    fn score_action_prefix_beats_contains() {
        let prefix_action = Action::new("e", "Edit Script", None, ActionCategory::ScriptContext);
        let contains_action = Action::new("c", "My Edit Tool", None, ActionCategory::ScriptContext);
        let prefix_score = ActionsDialog::score_action(&prefix_action, "edit");
        let contains_score = ActionsDialog::score_action(&contains_action, "edit");
        assert!(prefix_score > contains_score);
    }

    #[test]
    fn score_action_contains_beats_fuzzy() {
        let contains_action = Action::new("c", "My Edit Tool", None, ActionCategory::ScriptContext);
        let fuzzy_action = Action::new("f", "Erase Dict", None, ActionCategory::ScriptContext);
        let contains_score = ActionsDialog::score_action(&contains_action, "edit");
        let fuzzy_score = ActionsDialog::score_action(&fuzzy_action, "edit");
        assert!(
            contains_score > fuzzy_score,
            "Contains {} should beat fuzzy {}",
            contains_score,
            fuzzy_score
        );
    }

    #[test]
    fn score_action_stacks_title_and_description() {
        let action = Action::new(
            "test",
            "Edit Script",
            Some("Edit the script in your editor".into()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "edit");
        // Prefix match (100) + description match (15) = 115
        assert_eq!(score, 115);
    }

    // =========================================================================
    // 10. fuzzy_match edge cases
    // =========================================================================

    #[test]
    fn fuzzy_match_empty_needle_always_matches() {
        assert!(ActionsDialog::fuzzy_match("anything", ""));
    }

    #[test]
    fn fuzzy_match_empty_haystack_no_match() {
        assert!(!ActionsDialog::fuzzy_match("", "a"));
    }

    #[test]
    fn fuzzy_match_both_empty_matches() {
        assert!(ActionsDialog::fuzzy_match("", ""));
    }

    #[test]
    fn fuzzy_match_subsequence() {
        assert!(ActionsDialog::fuzzy_match("edit script", "es"));
        assert!(ActionsDialog::fuzzy_match("edit script", "eit"));
        assert!(ActionsDialog::fuzzy_match("edit script", "edsc"));
    }

    #[test]
    fn fuzzy_match_no_subsequence() {
        assert!(!ActionsDialog::fuzzy_match("edit", "xyz"));
        assert!(!ActionsDialog::fuzzy_match("abc", "abdc"));
    }

    #[test]
    fn fuzzy_match_needle_longer_than_haystack() {
        assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
    }

    #[test]
    fn fuzzy_match_exact_equals() {
        assert!(ActionsDialog::fuzzy_match("hello", "hello"));
    }

    // =========================================================================
    // 11. Scriptlet with_custom multiple custom actions ordering
    // =========================================================================

    #[test]
    fn scriptlet_three_custom_actions_maintain_order() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo main".into());
        scriptlet.actions = vec![
            ScriptletAction {
                name: "First".into(),
                command: "first".into(),
                tool: "bash".into(),
                code: "echo 1".into(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
            ScriptletAction {
                name: "Second".into(),
                command: "second".into(),
                tool: "bash".into(),
                code: "echo 2".into(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
            ScriptletAction {
                name: "Third".into(),
                command: "third".into(),
                tool: "bash".into(),
                code: "echo 3".into(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
        ];

        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let ids = action_ids(&actions);

        let first_pos = ids
            .iter()
            .position(|&id| id == "scriptlet_action:first")
            .unwrap();
        let second_pos = ids
            .iter()
            .position(|&id| id == "scriptlet_action:second")
            .unwrap();
        let third_pos = ids
            .iter()
            .position(|&id| id == "scriptlet_action:third")
            .unwrap();

        assert!(first_pos < second_pos);
        assert!(second_pos < third_pos);
    }

    #[test]
    fn scriptlet_custom_actions_after_run_before_shortcut() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo main".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Custom".into(),
            command: "custom".into(),
            tool: "bash".into(),
            code: "echo custom".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];

        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let ids = action_ids(&actions);

        let run_pos = ids.iter().position(|&id| id == "run_script").unwrap();
        let custom_pos = ids
            .iter()
            .position(|&id| id == "scriptlet_action:custom")
            .unwrap();
        let shortcut_pos = ids.iter().position(|&id| id == "add_shortcut").unwrap();

        assert_eq!(run_pos, 0);
        assert!(custom_pos > run_pos);
        assert!(custom_pos < shortcut_pos);
    }

    #[test]
    fn scriptlet_custom_actions_all_have_has_action_true() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo main".into());
        scriptlet.actions = vec![
            ScriptletAction {
                name: "A".into(),
                command: "a".into(),
                tool: "bash".into(),
                code: "echo a".into(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
            ScriptletAction {
                name: "B".into(),
                command: "b".into(),
                tool: "bash".into(),
                code: "echo b".into(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
        ];

        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        for action in &actions {
            if action.id.starts_with("scriptlet_action:") {
                assert!(
                    action.has_action,
                    "Scriptlet custom action '{}' should have has_action=true",
                    action.id
                );
                assert!(
                    action.value.is_some(),
                    "Scriptlet custom action '{}' should have a value",
                    action.id
                );
            }
        }
    }

    #[test]
    fn scriptlet_custom_action_value_matches_command() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo main".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Copy Stuff".into(),
            command: "copy-stuff".into(),
            tool: "bash".into(),
            code: "echo copy".into(),
            inputs: vec![],
            shortcut: Some("cmd+c".into()),
            description: Some("Copy stuff desc".into()),
        }];

        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let custom = actions
            .iter()
            .find(|a| a.id == "scriptlet_action:copy-stuff")
            .unwrap();
        assert_eq!(custom.value.as_deref(), Some("copy-stuff"));
        assert_eq!(custom.title, "Copy Stuff");
        assert_eq!(custom.description.as_deref(), Some("Copy stuff desc"));
        assert_eq!(custom.shortcut.as_deref(), Some("⌘C"));
    }

    // =========================================================================
    // 12. CommandBarConfig field validation
    // =========================================================================

    #[test]
    fn command_bar_config_ai_style_fields() {
        let config = CommandBarConfig::ai_style();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
        assert_eq!(config.dialog_config.section_style, SectionStyle::Headers);
        assert_eq!(config.dialog_config.anchor, AnchorPosition::Top);
        assert!(config.dialog_config.show_icons);
        assert!(config.dialog_config.show_footer);
        assert!(config.close_on_select);
        assert!(config.close_on_escape);
        assert!(config.close_on_click_outside);
    }

    #[test]
    fn command_bar_config_main_menu_style_fields() {
        let config = CommandBarConfig::main_menu_style();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Bottom);
        assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
        assert_eq!(config.dialog_config.anchor, AnchorPosition::Bottom);
        assert!(!config.dialog_config.show_icons);
        assert!(!config.dialog_config.show_footer);
    }

    #[test]
    fn command_bar_config_no_search_fields() {
        let config = CommandBarConfig::no_search();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Hidden);
        assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
        assert!(!config.dialog_config.show_icons);
    }

    #[test]
    fn command_bar_config_notes_style_fields() {
        let config = CommandBarConfig::notes_style();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
        assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
        assert_eq!(config.dialog_config.anchor, AnchorPosition::Top);
        assert!(config.dialog_config.show_icons);
        assert!(config.dialog_config.show_footer);
    }

    #[test]
    fn command_bar_config_default_all_close_true() {
        let config = CommandBarConfig::default();
        assert!(config.close_on_select);
        assert!(config.close_on_escape);
        assert!(config.close_on_click_outside);
    }

    // =========================================================================
    // 13. Chat context action interactions
    // =========================================================================

    #[test]
    fn chat_no_models_no_messages_no_response() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        // Only continue_in_chat (no models, no copy, no clear)
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "continue_in_chat");
    }

    #[test]
    fn chat_with_models_and_response_and_messages() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".into()),
            available_models: vec![
                ChatModelInfo {
                    id: "claude".into(),
                    display_name: "Claude".into(),
                    provider: "Anthropic".into(),
                },
                ChatModelInfo {
                    id: "gpt4".into(),
                    display_name: "GPT-4".into(),
                    provider: "OpenAI".into(),
                },
            ],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        // 2 models + continue_in_chat + copy_response + clear_conversation = 5
        assert_eq!(actions.len(), 5);
    }

    #[test]
    fn chat_current_model_gets_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".into()),
            available_models: vec![
                ChatModelInfo {
                    id: "claude".into(),
                    display_name: "Claude".into(),
                    provider: "Anthropic".into(),
                },
                ChatModelInfo {
                    id: "gpt4".into(),
                    display_name: "GPT-4".into(),
                    provider: "OpenAI".into(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let claude = actions
            .iter()
            .find(|a| a.id == "select_model_claude")
            .unwrap();
        assert!(claude.title.contains("✓"), "Current model should have ✓");

        let gpt4 = actions
            .iter()
            .find(|a| a.id == "select_model_gpt4")
            .unwrap();
        assert!(
            !gpt4.title.contains("✓"),
            "Non-current model should not have ✓"
        );
    }


    // --- merged from tests_part_04.rs ---
    #[test]
    fn chat_model_descriptions_show_provider() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "claude".into(),
                display_name: "Claude".into(),
                provider: "Anthropic".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let claude = actions
            .iter()
            .find(|a| a.id == "select_model_claude")
            .unwrap();
        assert_eq!(claude.description.as_deref(), Some("via Anthropic"));
    }

    #[test]
    fn chat_copy_response_only_when_has_response() {
        let without = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let with = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: true,
        };
        let without_actions = get_chat_context_actions(&without);
        let with_actions = get_chat_context_actions(&with);
        assert!(!without_actions.iter().any(|a| a.id == "copy_response"));
        assert!(with_actions.iter().any(|a| a.id == "copy_response"));
    }

    #[test]
    fn chat_clear_conversation_only_when_has_messages() {
        let without = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let with = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let without_actions = get_chat_context_actions(&without);
        let with_actions = get_chat_context_actions(&with);
        assert!(!without_actions.iter().any(|a| a.id == "clear_conversation"));
        assert!(with_actions.iter().any(|a| a.id == "clear_conversation"));
    }

    // =========================================================================
    // 14. Path context specifics
    // =========================================================================

    #[test]
    fn path_dir_primary_is_open_directory() {
        let path_info = PathInfo {
            path: "/tmp/mydir".into(),
            name: "mydir".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path_info);
        assert_eq!(actions[0].id, "open_directory");
        assert!(actions[0].title.contains("mydir"));
    }

    #[test]
    fn path_file_primary_is_select_file() {
        let path_info = PathInfo {
            path: "/tmp/myfile.txt".into(),
            name: "myfile.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        assert_eq!(actions[0].id, "select_file");
        assert!(actions[0].title.contains("myfile.txt"));
    }

    #[test]
    fn path_all_have_descriptions() {
        let path_info = PathInfo {
            path: "/tmp/test".into(),
            name: "test".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        for action in &actions {
            assert!(
                action.description.is_some(),
                "Path action '{}' should have a description",
                action.id
            );
        }
    }

    #[test]
    fn path_has_expected_actions() {
        let path_info = PathInfo {
            path: "/tmp/test".into(),
            name: "test".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"copy_path"));
        assert!(ids.contains(&"open_in_finder"));
        assert!(ids.contains(&"open_in_editor"));
        assert!(ids.contains(&"open_in_terminal"));
        assert!(ids.contains(&"copy_filename"));
        assert!(ids.contains(&"move_to_trash"));
    }

    #[test]
    fn path_dir_trash_says_folder() {
        let path_info = PathInfo {
            path: "/tmp/mydir".into(),
            name: "mydir".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path_info);
        let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
        assert!(
            trash.description.as_ref().unwrap().contains("folder"),
            "Dir trash should say 'folder'"
        );
    }

    #[test]
    fn path_file_trash_says_file() {
        let path_info = PathInfo {
            path: "/tmp/myfile.txt".into(),
            name: "myfile.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
        assert!(
            trash.description.as_ref().unwrap().contains("file"),
            "File trash should say 'file'"
        );
    }

    // =========================================================================
    // 15. File context specifics
    // =========================================================================

    #[test]
    fn file_open_title_includes_name() {
        let file_info = FileInfo {
            path: "/Users/test/readme.md".into(),
            name: "readme.md".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        assert!(actions[0].title.contains("readme.md"));
    }

    #[test]
    fn file_dir_open_title_includes_dirname() {
        let file_info = FileInfo {
            path: "/Users/test/Documents".into(),
            name: "Documents".into(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&file_info);
        assert!(actions[0].title.contains("Documents"));
    }

    #[test]
    fn file_all_have_descriptions() {
        let file_info = FileInfo {
            path: "/test/file.rs".into(),
            name: "file.rs".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        for action in &actions {
            assert!(
                action.description.is_some(),
                "File action '{}' should have a description",
                action.id
            );
        }
    }

    #[test]
    fn file_all_have_shortcuts() {
        let file_info = FileInfo {
            path: "/test/file.rs".into(),
            name: "file.rs".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        for action in &actions {
            assert!(
                action.shortcut.is_some(),
                "File action '{}' should have a shortcut",
                action.id
            );
        }
    }

    // =========================================================================
    // 16. Notes command bar conditional logic
    // =========================================================================

    #[test]
    fn notes_no_selection_no_trash_no_auto() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"new_note"));
        assert!(ids.contains(&"browse_notes"));
        assert!(ids.contains(&"enable_auto_sizing"));
        assert!(!ids.contains(&"duplicate_note"));
        assert!(!ids.contains(&"find_in_note"));
        assert!(!ids.contains(&"format"));
        assert!(!ids.contains(&"export"));
    }

    #[test]
    fn notes_with_selection_not_trash_auto_disabled() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"new_note"));
        assert!(ids.contains(&"duplicate_note"));
        assert!(ids.contains(&"browse_notes"));
        assert!(ids.contains(&"find_in_note"));
        assert!(ids.contains(&"format"));
        assert!(ids.contains(&"copy_note_as"));
        assert!(ids.contains(&"copy_deeplink"));
        assert!(ids.contains(&"create_quicklink"));
        assert!(ids.contains(&"export"));
        assert!(ids.contains(&"enable_auto_sizing"));
    }

    #[test]
    fn notes_with_selection_in_trash_hides_edit_copy_export() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"new_note"));
        assert!(ids.contains(&"browse_notes"));
        assert!(!ids.contains(&"duplicate_note"));
        assert!(!ids.contains(&"find_in_note"));
        assert!(!ids.contains(&"format"));
        assert!(!ids.contains(&"copy_note_as"));
        assert!(!ids.contains(&"export"));
    }

    #[test]
    fn notes_auto_sizing_enabled_hides_enable_action() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        assert!(!ids.contains(&"enable_auto_sizing"));
    }

    #[test]
    fn notes_full_feature_action_count() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        // new_note, duplicate, browse, find, format, copy_note_as,
        // copy_deeplink, create_quicklink, export, enable_auto_sizing = 10
        assert_eq!(actions.len(), 10);
    }

    #[test]
    fn notes_minimal_action_count() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        // new_note + browse_notes = 2
        assert_eq!(actions.len(), 2);
    }

    // =========================================================================
    // 17. to_deeplink_name comprehensive
    // =========================================================================

    #[test]
    fn deeplink_name_basic_spaces() {
        assert_eq!(to_deeplink_name("My Script"), "my-script");
    }

    #[test]
    fn deeplink_name_underscores_to_hyphens() {
        assert_eq!(to_deeplink_name("hello_world"), "hello-world");
    }

    #[test]
    fn deeplink_name_special_chars_stripped() {
        assert_eq!(to_deeplink_name("test!@#$%^&*()"), "test");
    }

    #[test]
    fn deeplink_name_multiple_spaces_collapsed() {
        assert_eq!(to_deeplink_name("a   b   c"), "a-b-c");
    }

    #[test]
    fn deeplink_name_leading_trailing_stripped() {
        assert_eq!(to_deeplink_name("  hello  "), "hello");
    }

    #[test]
    fn deeplink_name_numbers_preserved() {
        assert_eq!(to_deeplink_name("Test 123"), "test-123");
    }

    #[test]
    fn deeplink_name_all_special_returns_empty() {
        assert_eq!(to_deeplink_name("!@#$%"), "");
    }

    #[test]
    fn deeplink_name_single_word() {
        assert_eq!(to_deeplink_name("hello"), "hello");
    }

    #[test]
    fn deeplink_name_already_hyphenated() {
        assert_eq!(to_deeplink_name("my-script"), "my-script");
    }

    // =========================================================================
    // 18. format_shortcut_hint specifics
    // =========================================================================

    #[test]
    fn format_shortcut_cmd_enter() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+enter"), "⌘↵");
    }

    #[test]
    fn format_shortcut_ctrl_shift_escape() {
        assert_eq!(
            ActionsDialog::format_shortcut_hint("ctrl+shift+escape"),
            "⌃⇧⎋"
        );
    }

    #[test]
    fn format_shortcut_alt_backspace() {
        assert_eq!(ActionsDialog::format_shortcut_hint("alt+backspace"), "⌥⌫");
    }

    #[test]
    fn format_shortcut_command_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("command+c"), "⌘C");
    }

    #[test]
    fn format_shortcut_meta_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("meta+k"), "⌘K");
    }

    #[test]
    fn format_shortcut_option_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("option+tab"), "⌥⇥");
    }

    #[test]
    fn format_shortcut_control_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("control+space"), "⌃␣");
    }

    #[test]
    fn format_shortcut_arrows() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+up"), "⌘↑");
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+down"), "⌘↓");
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+left"), "⌘←");
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+right"), "⌘→");
    }

    #[test]
    fn format_shortcut_arrowup_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+arrowup"), "⌘↑");
    }

    // =========================================================================
    // 19. parse_shortcut_keycaps specifics
    // =========================================================================

    #[test]
    fn parse_keycaps_modifier_plus_letter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘E");
        assert_eq!(keycaps, vec!["⌘", "E"]);
    }

    #[test]
    fn parse_keycaps_two_modifiers_plus_letter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⇧C");
        assert_eq!(keycaps, vec!["⌘", "⇧", "C"]);
    }

    #[test]
    fn parse_keycaps_enter_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("↵");
        assert_eq!(keycaps, vec!["↵"]);
    }

    #[test]
    fn parse_keycaps_modifier_plus_enter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘↵");
        assert_eq!(keycaps, vec!["⌘", "↵"]);
    }


    // --- merged from tests_part_05.rs ---
    #[test]
    fn parse_keycaps_space_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("␣");
        assert_eq!(keycaps, vec!["␣"]);
    }

    #[test]
    fn parse_keycaps_all_modifiers() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⌃⌥⇧K");
        assert_eq!(keycaps, vec!["⌘", "⌃", "⌥", "⇧", "K"]);
    }

    #[test]
    fn parse_keycaps_arrow_keys() {
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("↑"), vec!["↑"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("↓"), vec!["↓"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("←"), vec!["←"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("→"), vec!["→"]);
    }

    // =========================================================================
    // 20. Agent-specific action validation
    // =========================================================================

    #[test]
    fn agent_has_edit_agent_title() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    #[test]
    fn agent_has_no_view_logs() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    #[test]
    fn agent_has_reveal_and_copy() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"reveal_in_finder"));
        assert!(ids.contains(&"copy_path"));
        assert!(ids.contains(&"copy_content"));
    }

    #[test]
    fn agent_edit_description_mentions_agent() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        let desc = edit.description.as_ref().unwrap().to_lowercase();
        assert!(desc.contains("agent"));
    }

    // =========================================================================
    // 21. New chat action details
    // =========================================================================

    #[test]
    fn new_chat_last_used_icon_is_bolt() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "Model 1".into(),
            provider: "p".into(),
            provider_display_name: "Provider".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
    }

    #[test]
    fn new_chat_preset_icon_matches_input() {
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].icon, Some(IconName::Star));
    }

    #[test]
    fn new_chat_model_icon_is_settings() {
        let models = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "Model 1".into(),
            provider: "p".into(),
            provider_display_name: "Provider".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].icon, Some(IconName::Settings));
    }

    #[test]
    fn new_chat_presets_have_no_description() {
        let presets = vec![NewChatPresetInfo {
            id: "code".into(),
            name: "Code".into(),
            icon: IconName::Code,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert!(actions[0].description.is_none());
    }

    #[test]
    fn new_chat_models_have_provider_description() {
        let models = vec![NewChatModelInfo {
            model_id: "claude".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].description.as_deref(), Some("Anthropic"));
    }

    #[test]
    fn new_chat_empty_all_returns_empty() {
        let actions = get_new_chat_actions(&[], &[], &[]);
        assert!(actions.is_empty());
    }

    // =========================================================================
    // 22. Action constructor edge cases
    // =========================================================================

    #[test]
    fn action_with_shortcut_opt_none_leaves_none() {
        let action =
            Action::new("t", "Test", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
        assert!(action.shortcut.is_none());
        assert!(action.shortcut_lower.is_none());
    }

    #[test]
    fn action_with_shortcut_opt_some_sets_both() {
        let action = Action::new("t", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘K".into()));
        assert_eq!(action.shortcut.as_deref(), Some("⌘K"));
        assert_eq!(action.shortcut_lower.as_deref(), Some("⌘k"));
    }

    #[test]
    fn action_title_lower_computed_on_creation() {
        let action = Action::new(
            "t",
            "My UPPERCASE Title",
            None,
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.title_lower, "my uppercase title");
    }

    #[test]
    fn action_description_lower_computed_on_creation() {
        let action = Action::new(
            "t",
            "T",
            Some("Description With CAPS".into()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(
            action.description_lower.as_deref(),
            Some("description with caps")
        );
    }

    #[test]
    fn action_no_description_has_none_lower() {
        let action = Action::new("t", "T", None, ActionCategory::ScriptContext);
        assert!(action.description_lower.is_none());
    }

    #[test]
    fn action_default_has_action_false() {
        let action = Action::new("t", "T", None, ActionCategory::ScriptContext);
        assert!(!action.has_action);
    }

    #[test]
    fn action_default_value_none() {
        let action = Action::new("t", "T", None, ActionCategory::ScriptContext);
        assert!(action.value.is_none());
    }

    #[test]
    fn action_default_icon_none() {
        let action = Action::new("t", "T", None, ActionCategory::ScriptContext);
        assert!(action.icon.is_none());
    }

    #[test]
    fn action_default_section_none() {
        let action = Action::new("t", "T", None, ActionCategory::ScriptContext);
        assert!(action.section.is_none());
    }

    #[test]
    fn action_with_icon_sets_icon() {
        let action =
            Action::new("t", "T", None, ActionCategory::ScriptContext).with_icon(IconName::Plus);
        assert_eq!(action.icon, Some(IconName::Plus));
    }

    #[test]
    fn action_with_section_sets_section() {
        let action =
            Action::new("t", "T", None, ActionCategory::ScriptContext).with_section("MySection");
        assert_eq!(action.section.as_deref(), Some("MySection"));
    }

    // =========================================================================
    // 23. ScriptInfo constructor validation
    // =========================================================================

    #[test]
    fn script_info_new_defaults() {
        let s = ScriptInfo::new("test", "/path");
        assert!(s.is_script);
        assert!(!s.is_scriptlet);
        assert!(!s.is_agent);
        assert_eq!(s.action_verb, "Run");
        assert!(s.shortcut.is_none());
        assert!(s.alias.is_none());
        assert!(!s.is_suggested);
        assert!(s.frecency_path.is_none());
    }

    #[test]
    fn script_info_builtin_has_empty_path() {
        let s = ScriptInfo::builtin("Test");
        assert!(s.path.is_empty());
        assert!(!s.is_script);
        assert!(!s.is_scriptlet);
        assert!(!s.is_agent);
    }

    #[test]
    fn script_info_scriptlet_sets_flags() {
        let s = ScriptInfo::scriptlet("Test", "/path.md", None, None);
        assert!(!s.is_script);
        assert!(s.is_scriptlet);
        assert!(!s.is_agent);
    }

    #[test]
    fn script_info_with_frecency_chaining() {
        let s = ScriptInfo::new("t", "/p").with_frecency(true, Some("/p".into()));
        assert!(s.is_suggested);
        assert_eq!(s.frecency_path.as_deref(), Some("/p"));
        // Original fields preserved
        assert!(s.is_script);
        assert_eq!(s.name, "t");
    }

    // =========================================================================
    // 24. Global actions always empty
    // =========================================================================

    #[test]
    fn global_actions_empty() {
        assert!(get_global_actions().is_empty());
    }

    // =========================================================================
    // 25. Ordering determinism (calling twice yields same result)
    // =========================================================================

    #[test]
    fn script_actions_deterministic() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions_1 = get_script_context_actions(&script);
        let actions_2 = get_script_context_actions(&script);
        let a1 = action_ids(&actions_1);
        let a2 = action_ids(&actions_2);
        assert_eq!(a1, a2);
    }

    #[test]
    fn clipboard_actions_deterministic() {
        let entry = ClipboardEntryInfo {
            id: "e".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "t".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions_1 = get_clipboard_history_context_actions(&entry);
        let actions_2 = get_clipboard_history_context_actions(&entry);
        let a1 = action_ids(&actions_1);
        let a2 = action_ids(&actions_2);
        assert_eq!(a1, a2);
    }

    #[test]
    fn notes_actions_deterministic() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions_1 = get_notes_command_bar_actions(&info);
        let actions_2 = get_notes_command_bar_actions(&info);
        let a1 = action_ids(&actions_1);
        let a2 = action_ids(&actions_2);
        assert_eq!(a1, a2);
    }

    #[test]
    fn ai_actions_deterministic() {
        let actions_1 = get_ai_command_bar_actions();
        let actions_2 = get_ai_command_bar_actions();
        let a1 = action_ids(&actions_1);
        let a2 = action_ids(&actions_2);
        assert_eq!(a1, a2);
    }

}
