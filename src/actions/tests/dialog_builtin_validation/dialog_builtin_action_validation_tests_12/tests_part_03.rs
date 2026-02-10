    #[test]
    fn cat12_scriptlet_shortcut_cmd_shift_becomes_symbols() {
        let mut scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Act".into(),
            command: "act".into(),
            tool: "bash".into(),
            code: "echo".into(),
            inputs: vec![],
            shortcut: Some("cmd+shift+c".into()),
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions[0].shortcut.as_deref(), Some("⌘⇧C"));
    }

    #[test]
    fn cat12_scriptlet_shortcut_ctrl_alt_becomes_symbols() {
        let mut scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Act".into(),
            command: "act".into(),
            tool: "bash".into(),
            code: "echo".into(),
            inputs: vec![],
            shortcut: Some("ctrl+alt+x".into()),
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions[0].shortcut.as_deref(), Some("⌃⌥X"));
    }

    #[test]
    fn cat12_scriptlet_shortcut_no_shortcut_is_none() {
        let mut scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Act".into(),
            command: "act".into(),
            tool: "bash".into(),
            code: "echo".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert!(actions[0].shortcut.is_none());
    }

    #[test]
    fn cat12_scriptlet_shortcut_single_key() {
        let mut scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Act".into(),
            command: "act".into(),
            tool: "bash".into(),
            code: "echo".into(),
            inputs: vec![],
            shortcut: Some("a".into()),
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions[0].shortcut.as_deref(), Some("A"));
    }

    // =========================================================================
    // 13. Action builder with_icon and with_section field isolation
    // =========================================================================

    #[test]
    fn cat13_with_icon_does_not_affect_section() {
        let action =
            Action::new("x", "X", None, ActionCategory::ScriptContext).with_icon(IconName::Copy);
        assert_eq!(action.icon, Some(IconName::Copy));
        assert!(action.section.is_none());
    }

    #[test]
    fn cat13_with_section_does_not_affect_icon() {
        let action =
            Action::new("x", "X", None, ActionCategory::ScriptContext).with_section("Test");
        assert!(action.icon.is_none());
        assert_eq!(action.section.as_deref(), Some("Test"));
    }

    #[test]
    fn cat13_chaining_icon_then_section() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_icon(IconName::Star)
            .with_section("Sec");
        assert_eq!(action.icon, Some(IconName::Star));
        assert_eq!(action.section.as_deref(), Some("Sec"));
    }

    #[test]
    fn cat13_chaining_section_then_icon() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_section("Sec")
            .with_icon(IconName::Star);
        assert_eq!(action.icon, Some(IconName::Star));
        assert_eq!(action.section.as_deref(), Some("Sec"));
    }

    #[test]
    fn cat13_with_shortcut_preserves_icon_section() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_icon(IconName::Plus)
            .with_section("S")
            .with_shortcut("⌘X");
        assert_eq!(action.icon, Some(IconName::Plus));
        assert_eq!(action.section.as_deref(), Some("S"));
        assert_eq!(action.shortcut.as_deref(), Some("⌘X"));
    }

    // =========================================================================
    // 14. ScriptInfo with_is_script constructor
    // =========================================================================

    #[test]
    fn cat14_with_is_script_true() {
        let s = ScriptInfo::with_is_script("test", "/path", true);
        assert!(s.is_script);
        assert!(!s.is_scriptlet);
        assert!(!s.is_agent);
        assert_eq!(s.action_verb, "Run");
    }

    #[test]
    fn cat14_with_is_script_false() {
        let s = ScriptInfo::with_is_script("builtin", "", false);
        assert!(!s.is_script);
        assert!(!s.is_scriptlet);
        assert!(!s.is_agent);
    }

    #[test]
    fn cat14_with_is_script_false_has_limited_actions() {
        let s = ScriptInfo::with_is_script("App", "", false);
        let actions = get_script_context_actions(&s);
        let ids = action_ids(&actions);
        // Non-script, non-scriptlet, non-agent = builtin-like
        assert!(!ids.contains(&"edit_script".to_string()));
        assert!(!ids.contains(&"view_logs".to_string()));
        assert!(ids.contains(&"run_script".to_string()));
        assert!(ids.contains(&"copy_deeplink".to_string()));
    }

    // =========================================================================
    // 15. Deeplink name with mixed Unicode scripts
    // =========================================================================

    #[test]
    fn cat15_deeplink_cjk_preserved() {
        let result = to_deeplink_name("日本語テスト");
        assert!(result.contains('日'));
        assert!(result.contains('語'));
    }

    #[test]
    fn cat15_deeplink_mixed_ascii_and_accents() {
        let result = to_deeplink_name("Café Script");
        assert!(result.contains("caf"));
        assert!(result.contains("é"));
    }

    #[test]
    fn cat15_deeplink_all_special_chars() {
        let result = to_deeplink_name("!@#$%^&*()");
        assert_eq!(result, "");
    }

    #[test]
    fn cat15_deeplink_leading_trailing_stripped() {
        let result = to_deeplink_name("  hello  ");
        assert_eq!(result, "hello");
    }

    #[test]
    fn cat15_deeplink_consecutive_specials_collapsed() {
        let result = to_deeplink_name("a---b___c");
        assert_eq!(result, "a-b-c");
    }

    // =========================================================================
    // 16. parse_shortcut_keycaps special symbol recognition
    // =========================================================================

    #[test]
    fn cat16_modifier_symbols_are_individual_keycaps() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⇧C");
        assert_eq!(keycaps, vec!["⌘", "⇧", "C"]);
    }

    #[test]
    fn cat16_enter_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("↵");
        assert_eq!(keycaps, vec!["↵"]);
    }

    #[test]
    fn cat16_escape_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⎋");
        assert_eq!(keycaps, vec!["⎋"]);
    }

    #[test]
    fn cat16_arrow_symbols() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("↑↓←→");
        assert_eq!(keycaps, vec!["↑", "↓", "←", "→"]);
    }

    #[test]
    fn cat16_backspace_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⌫");
        assert_eq!(keycaps, vec!["⌘", "⌫"]);
    }

    #[test]
    fn cat16_space_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("␣");
        assert_eq!(keycaps, vec!["␣"]);
    }

    #[test]
    fn cat16_tab_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⇥");
        assert_eq!(keycaps, vec!["⇥"]);
    }

    #[test]
    fn cat16_lowercase_uppercased() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘c");
        assert_eq!(keycaps, vec!["⌘", "C"]);
    }

    // =========================================================================
    // 17. Clipboard destructive action shortcut exact values
    // =========================================================================

    #[test]
    fn cat17_delete_shortcut() {
        let actions = get_clipboard_history_context_actions(&make_text_entry());
        let del = actions.iter().find(|a| a.id == "clipboard_delete").unwrap();
        assert_eq!(del.shortcut.as_deref(), Some("⌃X"));
    }

    #[test]
    fn cat17_delete_multiple_shortcut() {
        let actions = get_clipboard_history_context_actions(&make_text_entry());
        let del = actions
            .iter()
            .find(|a| a.id == "clipboard_delete_multiple")
            .unwrap();
        assert_eq!(del.shortcut.as_deref(), Some("⇧⌘X"));
    }

    #[test]
    fn cat17_delete_all_shortcut() {
        let actions = get_clipboard_history_context_actions(&make_text_entry());
        let del = actions
            .iter()
            .find(|a| a.id == "clipboard_delete_all")
            .unwrap();
        assert_eq!(del.shortcut.as_deref(), Some("⌃⇧X"));
    }

    #[test]
    fn cat17_destructive_actions_are_last_three() {
        let actions = get_clipboard_history_context_actions(&make_text_entry());
        let len = actions.len();
        assert_eq!(actions[len - 3].id, "clipboard_delete");
        assert_eq!(actions[len - 2].id, "clipboard_delete_multiple");
        assert_eq!(actions[len - 1].id, "clipboard_delete_all");
    }

    // =========================================================================
    // 18. File context macOS action count (file vs dir)
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn cat18_file_has_more_actions_than_dir_on_macos() {
        let file_info = FileInfo {
            path: "/tmp/a.txt".into(),
            name: "a.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let dir_info = FileInfo {
            path: "/tmp/docs".into(),
            name: "docs".into(),
            file_type: crate::file_search::FileType::Directory,
            is_dir: true,
        };
        let file_count = get_file_context_actions(&file_info).len();
        let dir_count = get_file_context_actions(&dir_info).len();
        // File has quick_look (macOS), dir doesn't
        assert!(
            file_count > dir_count,
            "file {} > dir {}",
            file_count,
            dir_count
        );
    }

    #[test]
    fn cat18_both_have_reveal_copy_path_copy_filename() {
        let file_info = FileInfo {
            path: "/tmp/a.txt".into(),
            name: "a.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let dir_info = FileInfo {
            path: "/tmp/docs".into(),
            name: "docs".into(),
            file_type: crate::file_search::FileType::Directory,
            is_dir: true,
        };
        for info in [&file_info, &dir_info] {
            let ids = action_ids(&get_file_context_actions(info));
            assert!(ids.contains(&"reveal_in_finder".to_string()));
            assert!(ids.contains(&"copy_path".to_string()));
            assert!(ids.contains(&"copy_filename".to_string()));
        }
    }

    // =========================================================================
    // 19. New chat action ID prefix patterns
    // =========================================================================

    #[test]
    fn cat19_last_used_ids_start_with_last_used() {
        let actions = get_new_chat_actions(
            &[NewChatModelInfo {
                model_id: "m1".into(),
                display_name: "M1".into(),
                provider: "P".into(),
                provider_display_name: "Provider".into(),
            }],
            &[],
            &[],
        );
        assert!(actions[0].id.starts_with("last_used_"));
    }

    #[test]
    fn cat19_preset_ids_start_with_preset() {
        let actions = get_new_chat_actions(
            &[],
            &[NewChatPresetInfo {
                id: "general".into(),
                name: "General".into(),
                icon: IconName::Star,
            }],
            &[],
        );
        assert!(actions[0].id.starts_with("preset_"));
    }

    #[test]
    fn cat19_model_ids_start_with_model() {
        let actions = get_new_chat_actions(
            &[],
            &[],
            &[NewChatModelInfo {
                model_id: "gpt4".into(),
                display_name: "GPT-4".into(),
                provider: "OpenAI".into(),
                provider_display_name: "OpenAI".into(),
            }],
        );
        assert!(actions[0].id.starts_with("model_"));
    }

    #[test]
    fn cat19_empty_inputs_empty_output() {
        let actions = get_new_chat_actions(&[], &[], &[]);
        assert!(actions.is_empty());
    }

    #[test]
    fn cat19_all_three_sections_present() {
        let actions = get_new_chat_actions(
            &[NewChatModelInfo {
                model_id: "m".into(),
                display_name: "M".into(),
                provider: "P".into(),
                provider_display_name: "PP".into(),
            }],
            &[NewChatPresetInfo {
                id: "p".into(),
                name: "P".into(),
                icon: IconName::Star,
            }],
            &[NewChatModelInfo {
                model_id: "x".into(),
                display_name: "X".into(),
                provider: "Q".into(),
                provider_display_name: "QQ".into(),
            }],
        );
        let sections: Vec<_> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        assert!(sections.contains(&"Last Used Settings"));
        assert!(sections.contains(&"Presets"));
        assert!(sections.contains(&"Models"));
    }

    // =========================================================================
    // 20. Notes command bar section-to-action mapping
    // =========================================================================

    #[test]
    fn cat20_full_feature_has_5_sections() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let sections: HashSet<_> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        assert_eq!(sections.len(), 5, "Expected 5 sections: {:?}", sections);
    }

    #[test]
    fn cat20_trash_view_fewer_sections() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let sections: HashSet<_> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        // Trash view hides Edit/Copy/Export, keeps only Notes
        assert!(
            sections.len() < 5,
            "Trash should have fewer sections: {:?}",
            sections
        );
    }

