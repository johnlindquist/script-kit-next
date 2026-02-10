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
        for (i, action) in actions.iter().enumerate().take(5) {
            assert_eq!(
                action.section.as_deref(),
                Some("Pinned"),
                "Note {} should be in Pinned section",
                i
            );
        }

        // Remaining should be Recent
        for (i, action) in actions.iter().enumerate().take(50).skip(5) {
            assert_eq!(
                action.section.as_deref(),
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
