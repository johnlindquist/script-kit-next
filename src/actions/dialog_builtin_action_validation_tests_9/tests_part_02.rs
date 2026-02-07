    #[test]
    fn format_shortcut_hint_triple_modifier() {
        assert_eq!(
            ActionsDialog::format_shortcut_hint("cmd+shift+alt+z"),
            "⌘⇧⌥Z"
        );
    }

    #[test]
    fn format_shortcut_hint_space_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("space"), "␣");
    }

    #[test]
    fn format_shortcut_hint_backspace_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+backspace"), "⌘⌫");
    }

    #[test]
    fn format_shortcut_hint_delete_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("delete"), "⌫");
    }

    // ============================================================
    // 6. Cross-context shortcut symbol consistency
    // ============================================================

    #[test]
    fn all_script_context_shortcuts_use_symbols() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        for action in &actions {
            if let Some(ref shortcut) = action.shortcut {
                assert!(
                    !shortcut.contains("cmd")
                        && !shortcut.contains("shift")
                        && !shortcut.contains("alt")
                        && !shortcut.contains("ctrl"),
                    "Shortcut '{}' for action '{}' should use symbols not text",
                    shortcut,
                    action.id
                );
            }
        }
    }

    #[test]
    fn all_clipboard_context_shortcuts_use_symbols() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        for action in &actions {
            if let Some(ref shortcut) = action.shortcut {
                assert!(
                    !shortcut.contains("cmd")
                        && !shortcut.contains("shift")
                        && !shortcut.contains("alt")
                        && !shortcut.contains("ctrl"),
                    "Shortcut '{}' for action '{}' should use symbols not text",
                    shortcut,
                    action.id
                );
            }
        }
    }

    #[test]
    fn all_path_context_shortcuts_use_symbols() {
        let path = PathInfo {
            name: "test".to_string(),
            path: "/test".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        for action in &actions {
            if let Some(ref shortcut) = action.shortcut {
                assert!(
                    !shortcut.contains("cmd")
                        && !shortcut.contains("shift")
                        && !shortcut.contains("alt")
                        && !shortcut.contains("ctrl"),
                    "Shortcut '{}' for action '{}' should use symbols not text",
                    shortcut,
                    action.id
                );
            }
        }
    }

    // ============================================================
    // 7. Action verb formatting with special characters in names
    // ============================================================

    #[test]
    fn action_verb_with_quotes_in_name() {
        let script = ScriptInfo::new("My \"Best\" Script", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        let run = find_action(&actions, "run_script").unwrap();
        assert!(
            run.title.contains("My \"Best\" Script"),
            "Title should preserve quotes in name: {}",
            run.title
        );
    }

    #[test]
    fn action_verb_with_unicode_in_name() {
        let script = ScriptInfo::with_action_verb("Café Finder", "/path/cafe.ts", false, "Launch");
        let actions = get_script_context_actions(&script);
        let run = find_action(&actions, "run_script").unwrap();
        assert_eq!(run.title, "Launch \"Café Finder\"");
    }

    #[test]
    fn action_verb_with_empty_name() {
        let script = ScriptInfo::new("", "/path/empty.ts");
        let actions = get_script_context_actions(&script);
        let run = find_action(&actions, "run_script").unwrap();
        assert_eq!(run.title, "Run \"\"");
    }

    #[test]
    fn action_verb_execute_formatting() {
        let script = ScriptInfo::with_action_verb("Task Runner", "/path/task.ts", false, "Execute");
        let actions = get_script_context_actions(&script);
        let run = find_action(&actions, "run_script").unwrap();
        assert!(run.title.starts_with("Execute "));
        assert!(
            run.description.as_ref().unwrap().contains("Execute"),
            "Description should include verb"
        );
    }

    // ============================================================
    // 8. Notes command bar conditional section groups
    // ============================================================

    #[test]
    fn notes_command_bar_full_feature_sections() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let sections: HashSet<_> = actions.iter().filter_map(|a| a.section.as_ref()).collect();
        assert!(sections.contains(&"Notes".to_string()));
        assert!(sections.contains(&"Edit".to_string()));
        assert!(sections.contains(&"Copy".to_string()));
        assert!(sections.contains(&"Export".to_string()));
        assert!(sections.contains(&"Settings".to_string()));
    }

    #[test]
    fn notes_command_bar_trash_view_no_edit_section() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let sections: HashSet<_> = actions.iter().filter_map(|a| a.section.as_ref()).collect();
        assert!(
            !sections.contains(&"Edit".to_string()),
            "Trash view should not have Edit section"
        );
        assert!(
            !sections.contains(&"Copy".to_string()),
            "Trash view should not have Copy section"
        );
        assert!(
            !sections.contains(&"Export".to_string()),
            "Trash view should not have Export section"
        );
    }

    #[test]
    fn notes_command_bar_no_selection_minimal() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Only Notes section should be present (new_note, browse_notes)
        assert_eq!(
            actions.len(),
            2,
            "No selection + auto_sizing should give 2 actions"
        );
        for action in &actions {
            assert_eq!(action.section.as_deref(), Some("Notes"));
        }
    }

    #[test]
    fn notes_command_bar_auto_sizing_disabled_adds_setting() {
        let info_disabled = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let info_enabled = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions_disabled = get_notes_command_bar_actions(&info_disabled);
        let actions_enabled = get_notes_command_bar_actions(&info_enabled);
        assert_eq!(
            actions_disabled.len(),
            actions_enabled.len() + 1,
            "Disabled auto_sizing should add one more action"
        );
        assert!(
            find_action(&actions_disabled, "enable_auto_sizing").is_some(),
            "Should have enable_auto_sizing when disabled"
        );
        assert!(
            find_action(&actions_enabled, "enable_auto_sizing").is_none(),
            "Should NOT have enable_auto_sizing when enabled"
        );
    }

    // ============================================================
    // 9. ScriptInfo mixed agent+scriptlet flag precedence
    // ============================================================

    #[test]
    fn script_info_agent_flag_set_after_new() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let ids = action_ids(&actions);
        assert!(
            ids.contains(&"edit_script"),
            "Agent should have edit_script (titled 'Edit Agent')"
        );
        let edit = find_action(&actions, "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    #[test]
    fn script_info_agent_no_view_logs() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let ids = action_ids(&actions);
        assert!(
            !ids.contains(&"view_logs"),
            "Agent should NOT have view_logs"
        );
    }

    #[test]
    fn script_info_agent_has_copy_content() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        assert!(
            find_action(&actions, "copy_content").is_some(),
            "Agent should have copy_content"
        );
    }

    #[test]
    fn script_info_agent_has_reveal_and_copy_path() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        assert!(find_action(&actions, "reveal_in_finder").is_some());
        assert!(find_action(&actions, "copy_path").is_some());
    }

    // ============================================================
    // 10. Clipboard save_snippet/save_file always present
    // ============================================================

    #[test]
    fn clipboard_text_has_save_snippet_and_file() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(find_action(&actions, "clipboard_save_snippet").is_some());
        assert!(find_action(&actions, "clipboard_save_file").is_some());
    }

    #[test]
    fn clipboard_image_has_save_snippet_and_file() {
        let entry = make_clipboard_entry(ContentType::Image, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(find_action(&actions, "clipboard_save_snippet").is_some());
        assert!(find_action(&actions, "clipboard_save_file").is_some());
    }

    #[test]
    fn clipboard_save_snippet_shortcut() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let snippet = find_action(&actions, "clipboard_save_snippet").unwrap();
        assert_eq!(snippet.shortcut.as_deref(), Some("⇧⌘S"));
    }

    #[test]
    fn clipboard_save_file_shortcut() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let save = find_action(&actions, "clipboard_save_file").unwrap();
        assert_eq!(save.shortcut.as_deref(), Some("⌥⇧⌘S"));
    }

    // ============================================================
    // 11. Clipboard share/attach_to_ai always present
    // ============================================================

    #[test]
    fn clipboard_text_has_share_and_attach() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(find_action(&actions, "clipboard_share").is_some());
        assert!(find_action(&actions, "clipboard_attach_to_ai").is_some());
    }

    #[test]
    fn clipboard_image_has_share_and_attach() {
        let entry = make_clipboard_entry(ContentType::Image, true, Some("Finder"));
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(find_action(&actions, "clipboard_share").is_some());
        assert!(find_action(&actions, "clipboard_attach_to_ai").is_some());
    }

    #[test]
    fn clipboard_share_shortcut_value() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let share = find_action(&actions, "clipboard_share").unwrap();
        assert_eq!(share.shortcut.as_deref(), Some("⇧⌘E"));
    }

    #[test]
    fn clipboard_attach_to_ai_shortcut_value() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let attach = find_action(&actions, "clipboard_attach_to_ai").unwrap();
        assert_eq!(attach.shortcut.as_deref(), Some("⌃⌘A"));
    }

    // ============================================================
    // 12. Path context descriptions
    // ============================================================

    #[test]
    fn path_context_open_in_finder_description() {
        let path = PathInfo {
            name: "test".to_string(),
            path: "/test".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        let finder = find_action(&actions, "open_in_finder").unwrap();
        assert!(
            finder.description.as_ref().unwrap().contains("Finder"),
            "open_in_finder description should mention Finder"
        );
    }

    #[test]
    fn path_context_open_in_editor_description() {
        let path = PathInfo {
            name: "test".to_string(),
            path: "/test".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        let editor = find_action(&actions, "open_in_editor").unwrap();
        assert!(
            editor.description.as_ref().unwrap().contains("$EDITOR"),
            "open_in_editor description should mention $EDITOR"
        );
    }

    #[test]
    fn path_context_open_in_terminal_description() {
        let path = PathInfo {
            name: "test".to_string(),
            path: "/test".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        let terminal = find_action(&actions, "open_in_terminal").unwrap();
        assert!(
            terminal.description.as_ref().unwrap().contains("terminal"),
            "open_in_terminal description should mention terminal"
        );
    }

    #[test]
    fn path_context_move_to_trash_file_says_file() {
        let path = PathInfo {
            name: "test.txt".to_string(),
            path: "/test.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        let trash = find_action(&actions, "move_to_trash").unwrap();
        assert!(
            trash.description.as_ref().unwrap().contains("file"),
            "Trash description for file should say 'file'"
        );
    }

    #[test]
    fn path_context_move_to_trash_dir_says_folder() {
        let path = PathInfo {
            name: "docs".to_string(),
            path: "/docs".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path);
        let trash = find_action(&actions, "move_to_trash").unwrap();
        assert!(
            trash.description.as_ref().unwrap().contains("folder"),
            "Trash description for dir should say 'folder'"
        );
    }

    // ============================================================
    // 13. Note switcher empty notes placeholder
    // ============================================================

    #[test]
    fn note_switcher_empty_shows_placeholder() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "no_notes");
        assert_eq!(actions[0].title, "No notes yet");
        assert!(
            actions[0].description.as_ref().unwrap().contains("⌘N"),
            "Placeholder should hint at ⌘N"
        );
        assert_eq!(actions[0].icon, Some(IconName::Plus));
        assert_eq!(actions[0].section.as_deref(), Some("Notes"));
    }

    // ============================================================
    // 14. New chat action icon/section consistency
    // ============================================================

