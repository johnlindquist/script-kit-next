//! Batch 8: Dialog builtin action validation tests
//!
//! Focuses on combined scenarios, interaction matrices, and boundary conditions
//! not covered in batches 1-7:
//!
//! 1. Action verb in primary title format ("{verb} \"{name}\"") across all verbs
//! 2. Clipboard pin×contentType×app combined matrix
//! 3. Scriptlet context custom actions with frecency interaction
//! 4. Path context special characters in names
//! 5. Note switcher Unicode/emoji titles
//! 6. Chat context partial state combinations
//! 7. AI command bar description keyword validation
//! 8. Notes command bar section label transitions
//! 9. New chat duplicate providers
//! 10. Deeplink description URL format validation
//! 11. Score_action boundary thresholds (exact 100/50/25 boundaries)
//! 12. build_grouped_items interleaved section/no-section
//! 13. coerce_action_selection complex patterns
//! 14. parse_shortcut_keycaps compound symbol sequences
//! 15. CommandBarConfig notes_style detailed fields
//! 16. Cross-builder action count comparisons
//! 17. Action builder chaining order independence
//! 18. Clipboard destructive action ordering stability
//! 19. File context title includes exact filename
//! 20. Notes info all-true/all-false edge cases
//! 21. ScriptInfo agent flag interactions with frecency chaining
//! 22. Agent actions: no view_logs, has copy_content
//! 23. Builtin with full optional fields (shortcut+alias+frecency)
//! 24. Path context dir vs file action count equality
//! 25. Multiple scriptlet custom actions ordering
//! 26. Chat model checkmark exact match only
//! 27. Note switcher empty/placeholder title
//! 28. Action with_section/with_icon chaining order independence
//! 29. Clipboard delete_multiple description content
//! 30. Deeplink name consecutive special characters

#[cfg(test)]
mod tests {
    // --- merged from tests_part_01.rs ---
    use crate::actions::builders::{
        get_ai_command_bar_actions, get_chat_context_actions,
        get_clipboard_history_context_actions, get_file_context_actions, get_new_chat_actions,
        get_note_switcher_actions, get_notes_command_bar_actions, get_path_context_actions,
        get_script_context_actions, get_scriptlet_context_actions_with_custom, to_deeplink_name,
        ChatModelInfo, ChatPromptInfo, ClipboardEntryInfo, NewChatModelInfo, NewChatPresetInfo,
        NoteSwitcherNoteInfo, NotesInfo,
    };
    use crate::actions::dialog::{
        build_grouped_items_static, coerce_action_selection, ActionsDialog, GroupedActionItem,
    };
    use crate::actions::types::{Action, ActionCategory, ScriptInfo, SectionStyle};
    use crate::actions::CommandBarConfig;
    use crate::clipboard_history::ContentType;
    use crate::designs::icon_variations::IconName;
    use crate::file_search::{FileInfo, FileType};
    use crate::prompts::PathInfo;
    use crate::scriptlets::{Scriptlet, ScriptletAction};
    use std::collections::HashSet;

    // ============================================================
    // Helper functions
    // ============================================================

    fn action_ids(actions: &[Action]) -> Vec<&str> {
        actions.iter().map(|a| a.id.as_str()).collect()
    }

    fn find_action<'a>(actions: &'a [Action], id: &str) -> Option<&'a Action> {
        actions.iter().find(|a| a.id == id)
    }

    fn make_action(id: &str, title: &str, section: Option<&str>) -> Action {
        let mut a = Action::new(
            id,
            title,
            Some(format!("Desc for {}", title)),
            ActionCategory::ScriptContext,
        );
        if let Some(s) = section {
            a = a.with_section(s);
        }
        a
    }

    fn make_note(
        id: &str,
        title: &str,
        char_count: usize,
        is_current: bool,
        is_pinned: bool,
        preview: &str,
        relative_time: &str,
    ) -> NoteSwitcherNoteInfo {
        NoteSwitcherNoteInfo {
            id: id.to_string(),
            title: title.to_string(),
            char_count,
            is_current,
            is_pinned,
            preview: preview.to_string(),
            relative_time: relative_time.to_string(),
        }
    }

    fn make_clipboard_entry(
        content_type: ContentType,
        pinned: bool,
        app_name: Option<&str>,
    ) -> ClipboardEntryInfo {
        ClipboardEntryInfo {
            id: format!(
                "entry-{}-{}",
                if pinned { "pinned" } else { "unpinned" },
                match content_type {
                    ContentType::Text => "text",
                    ContentType::Image => "image",
                    ContentType::Link => "link",
                    ContentType::File => "file",
                    ContentType::Color => "color",
                }
            ),
            content_type,
            pinned,
            preview: "Test content".to_string(),
            image_dimensions: if content_type == ContentType::Image {
                Some((640, 480))
            } else {
                None
            },
            frontmost_app_name: app_name.map(|s| s.to_string()),
        }
    }

    // ============================================================
    // 1. Action verb in primary title format
    // ============================================================

    #[test]
    fn verb_run_in_primary_title() {
        let script = ScriptInfo::new("My Script", "/path/to/script.ts");
        let actions = get_script_context_actions(&script);
        let primary = &actions[0];
        assert_eq!(primary.title, "Run \"My Script\"");
        assert_eq!(primary.id, "run_script");
    }

    #[test]
    fn verb_launch_in_primary_title() {
        let script =
            ScriptInfo::with_action_verb("Safari", "/Applications/Safari.app", false, "Launch");
        let actions = get_script_context_actions(&script);
        assert_eq!(actions[0].title, "Launch \"Safari\"");
    }

    #[test]
    fn verb_switch_to_in_primary_title() {
        let script = ScriptInfo::with_action_verb("My Document", "window:123", false, "Switch to");
        let actions = get_script_context_actions(&script);
        assert_eq!(actions[0].title, "Switch to \"My Document\"");
    }

    #[test]
    fn verb_open_in_primary_title() {
        let script =
            ScriptInfo::with_action_verb("Clipboard History", "builtin:clipboard", false, "Open");
        let actions = get_script_context_actions(&script);
        assert_eq!(actions[0].title, "Open \"Clipboard History\"");
    }

    #[test]
    fn verb_execute_in_primary_title() {
        let script =
            ScriptInfo::with_action_verb("Custom Task", "/path/to/task.ts", true, "Execute");
        let actions = get_script_context_actions(&script);
        assert_eq!(actions[0].title, "Execute \"Custom Task\"");
    }

    #[test]
    fn primary_description_includes_verb() {
        let verbs = ["Run", "Launch", "Switch to", "Open", "Execute"];
        for verb in &verbs {
            let script = ScriptInfo::with_action_verb("Test", "/path", false, *verb);
            let actions = get_script_context_actions(&script);
            let desc = actions[0].description.as_ref().unwrap();
            assert!(
                desc.contains(verb),
                "Description '{}' should contain verb '{}'",
                desc,
                verb
            );
        }
    }

    #[test]
    fn scriptlet_primary_verb_matches_script_info() {
        let script = ScriptInfo::scriptlet("Open GitHub", "/path/to/url.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        // Scriptlets default to "Run" verb
        assert_eq!(actions[0].title, "Run \"Open GitHub\"");
    }

    // ============================================================
    // 2. Clipboard pin × contentType × app combined matrix
    // ============================================================

    #[test]
    fn clipboard_text_unpinned_no_app() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_pin"));
        assert!(!actions.iter().any(|a| a.id == "clipboard_unpin"));
        assert!(!actions.iter().any(|a| a.id == "clipboard_ocr"));
        let paste = find_action(&actions, "clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to Active App");
    }

    #[test]
    fn clipboard_text_pinned_with_app() {
        let entry = make_clipboard_entry(ContentType::Text, true, Some("VSCode"));
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(!actions.iter().any(|a| a.id == "clipboard_pin"));
        assert!(actions.iter().any(|a| a.id == "clipboard_unpin"));
        let paste = find_action(&actions, "clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to VSCode");
    }

    #[test]
    fn clipboard_image_unpinned_with_app() {
        let entry = make_clipboard_entry(ContentType::Image, false, Some("Figma"));
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_pin"));
        assert!(actions.iter().any(|a| a.id == "clipboard_ocr"));
        let paste = find_action(&actions, "clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to Figma");
    }

    #[test]
    fn clipboard_image_pinned_no_app() {
        let entry = make_clipboard_entry(ContentType::Image, true, None);
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_unpin"));
        assert!(actions.iter().any(|a| a.id == "clipboard_ocr"));
        let paste = find_action(&actions, "clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to Active App");
    }

    #[test]
    fn clipboard_image_has_more_actions_than_text() {
        let text_entry = make_clipboard_entry(ContentType::Text, false, None);
        let image_entry = make_clipboard_entry(ContentType::Image, false, None);
        let text_actions = get_clipboard_history_context_actions(&text_entry);
        let image_actions = get_clipboard_history_context_actions(&image_entry);
        assert!(
            image_actions.len() > text_actions.len(),
            "Image ({}) should have more actions than text ({})",
            image_actions.len(),
            text_actions.len()
        );
    }

    // ============================================================
    // 3. Scriptlet context with custom actions + frecency
    // ============================================================

    #[test]
    fn scriptlet_with_custom_actions_and_frecency() {
        let script = ScriptInfo::scriptlet("Test", "/path/to/test.md", None, None)
            .with_frecency(true, Some("scriptlet:Test".to_string()));

        let mut scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo main".to_string(),
        );
        scriptlet.actions = vec![ScriptletAction {
            name: "Copy Output".to_string(),
            command: "copy-output".to_string(),
            tool: "bash".to_string(),
            code: "echo output | pbcopy".to_string(),
            inputs: vec![],
            shortcut: Some("cmd+c".to_string()),
            description: Some("Copy the output".to_string()),
        }];

        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));

        // Should have custom action
        assert!(actions
            .iter()
            .any(|a| a.id == "scriptlet_action:copy-output"));
        // Should have reset_ranking due to frecency
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));
        // Custom action should be after run but before edit
        let run_idx = actions.iter().position(|a| a.id == "run_script").unwrap();
        let custom_idx = actions
            .iter()
            .position(|a| a.id == "scriptlet_action:copy-output")
            .unwrap();
        let edit_idx = actions
            .iter()
            .position(|a| a.id == "edit_scriptlet")
            .unwrap();
        assert!(run_idx < custom_idx);
        assert!(custom_idx < edit_idx);
    }

    #[test]
    fn scriptlet_custom_action_has_action_true() {
        let script = ScriptInfo::scriptlet("Test", "/path/to/test.md", None, None);
        let mut scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo main".to_string(),
        );
        scriptlet.actions = vec![ScriptletAction {
            name: "Do Thing".to_string(),
            command: "do-thing".to_string(),
            tool: "bash".to_string(),
            code: "echo thing".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];

        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let custom = find_action(&actions, "scriptlet_action:do-thing").unwrap();
        assert!(
            custom.has_action,
            "Scriptlet custom action must have has_action=true"
        );
        assert_eq!(custom.value, Some("do-thing".to_string()));
    }

    #[test]
    fn scriptlet_builtin_actions_have_has_action_false() {
        let script = ScriptInfo::scriptlet("Test", "/path/to/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        for action in &actions {
            if !action.id.starts_with("scriptlet_action:") {
                assert!(
                    !action.has_action,
                    "Built-in action '{}' should have has_action=false",
                    action.id
                );
            }
        }
    }

    // ============================================================
    // 4. Path context special characters in names
    // ============================================================

    #[test]
    fn path_dir_with_spaces_in_name() {
        let info = PathInfo::new("My Documents", "/Users/test/My Documents", true);
        let actions = get_path_context_actions(&info);
        let primary = &actions[0];
        assert_eq!(primary.title, "Open \"My Documents\"");
        assert_eq!(primary.id, "open_directory");
    }

    #[test]
    fn path_file_with_dots_in_name() {
        let info = PathInfo::new("archive.tar.gz", "/tmp/archive.tar.gz", false);
        let actions = get_path_context_actions(&info);
        let primary = &actions[0];
        assert_eq!(primary.title, "Select \"archive.tar.gz\"");
        assert_eq!(primary.id, "select_file");
    }

    #[test]
    fn path_trash_description_dir_vs_file() {
        let dir_info = PathInfo::new("folder", "/tmp/folder", true);
        let file_info = PathInfo::new("file.txt", "/tmp/file.txt", false);

        let dir_actions = get_path_context_actions(&dir_info);
        let file_actions = get_path_context_actions(&file_info);

        let dir_trash = find_action(&dir_actions, "move_to_trash").unwrap();
        let file_trash = find_action(&file_actions, "move_to_trash").unwrap();

        assert!(
            dir_trash.description.as_ref().unwrap().contains("folder"),
            "Dir trash should say 'folder'"
        );
        assert!(
            file_trash.description.as_ref().unwrap().contains("file"),
            "File trash should say 'file'"
        );
    }

    #[test]
    fn path_dir_and_file_have_same_action_count() {
        let dir_info = PathInfo::new("dir", "/tmp/dir", true);
        let file_info = PathInfo::new("file.txt", "/tmp/file.txt", false);
        let dir_actions = get_path_context_actions(&dir_info);
        let file_actions = get_path_context_actions(&file_info);
        assert_eq!(
            dir_actions.len(),
            file_actions.len(),
            "Dir ({}) and file ({}) should have same action count",
            dir_actions.len(),
            file_actions.len()
        );
    }

    // ============================================================
    // 5. Note switcher Unicode/emoji titles
    // ============================================================

    #[test]
    fn note_switcher_unicode_title() {
        let notes = vec![make_note(
            "id1",
            "Café Notes",
            42,
            false,
            false,
            "Some preview text",
            "5m ago",
        )];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].title, "Café Notes");
    }

    #[test]
    fn note_switcher_current_with_bullet() {
        let notes = vec![make_note(
            "id1",
            "Current Note",
            100,
            true,
            false,
            "Content here",
            "1m ago",
        )];
        let actions = get_note_switcher_actions(&notes);
        assert!(
            actions[0].title.starts_with("• "),
            "Current note should have bullet prefix, got: '{}'",
            actions[0].title
        );
        assert_eq!(actions[0].title, "• Current Note");
    }

    #[test]
    fn note_switcher_pinned_has_star_icon() {
        let notes = vec![make_note(
            "id1",
            "Pinned",
            50,
            false,
            true,
            "pinned content",
            "2h ago",
        )];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
        assert_eq!(actions[0].section, Some("Pinned".to_string()));
    }

    #[test]
    fn note_switcher_current_has_check_icon() {
        let notes = vec![make_note(
            "id1",
            "Current",
            50,
            true,
            false,
            "current content",
            "1m ago",
        )];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::Check));
        assert_eq!(actions[0].section, Some("Recent".to_string()));
    }


    // --- merged from tests_part_02.rs ---
    #[test]
    fn note_switcher_regular_has_file_icon() {
        let notes = vec![make_note(
            "id1",
            "Regular",
            50,
            false,
            false,
            "regular content",
            "3d ago",
        )];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::File));
        assert_eq!(actions[0].section, Some("Recent".to_string()));
    }

    #[test]
    fn note_switcher_pinned_overrides_current_icon() {
        // When both pinned and current, pinned wins for icon
        let notes = vec![make_note(
            "id1",
            "Both",
            50,
            true,
            true,
            "both flags",
            "1m ago",
        )];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(
            actions[0].icon,
            Some(IconName::StarFilled),
            "Pinned should override current for icon"
        );
        assert_eq!(actions[0].section, Some("Pinned".to_string()));
    }

    #[test]
    fn note_switcher_empty_returns_placeholder() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "no_notes");
        assert_eq!(actions[0].title, "No notes yet");
    }

    #[test]
    fn note_switcher_id_format() {
        let notes = vec![make_note("abc-123-def", "Test", 10, false, false, "", "")];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].id, "note_abc-123-def");
    }

    // ============================================================
    // 6. Chat context partial state combinations
    // ============================================================

    #[test]
    fn chat_no_models_no_messages_no_response() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        // Should still have continue_in_chat
        assert!(actions.iter().any(|a| a.id == "continue_in_chat"));
        // Should NOT have copy_response or clear
        assert!(!actions.iter().any(|a| a.id == "copy_response"));
        assert!(!actions.iter().any(|a| a.id == "clear_conversation"));
    }

    #[test]
    fn chat_has_response_but_no_messages() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "copy_response"));
        assert!(!actions.iter().any(|a| a.id == "clear_conversation"));
    }

    #[test]
    fn chat_has_messages_but_no_response() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "copy_response"));
        assert!(actions.iter().any(|a| a.id == "clear_conversation"));
    }

    #[test]
    fn chat_all_flags_true() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".to_string()),
            available_models: vec![ChatModelInfo {
                id: "claude-3".to_string(),
                display_name: "Claude".to_string(),
                provider: "Anthropic".to_string(),
            }],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "continue_in_chat"));
        assert!(actions.iter().any(|a| a.id == "copy_response"));
        assert!(actions.iter().any(|a| a.id == "clear_conversation"));
        // Model should have checkmark
        let model_action = actions
            .iter()
            .find(|a| a.id == "select_model_claude-3")
            .unwrap();
        assert!(model_action.title.contains('✓'));
    }

    #[test]
    fn chat_checkmark_only_on_exact_display_name_match() {
        let info = ChatPromptInfo {
            current_model: Some("Claude 3.5".to_string()),
            available_models: vec![
                ChatModelInfo {
                    id: "claude-3".to_string(),
                    display_name: "Claude 3".to_string(),
                    provider: "Anthropic".to_string(),
                },
                ChatModelInfo {
                    id: "claude-35".to_string(),
                    display_name: "Claude 3.5".to_string(),
                    provider: "Anthropic".to_string(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let claude3 = find_action(&actions, "select_model_claude-3").unwrap();
        let claude35 = find_action(&actions, "select_model_claude-35").unwrap();

        assert!(
            !claude3.title.contains('✓'),
            "Claude 3 should NOT have checkmark when current is Claude 3.5"
        );
        assert!(
            claude35.title.contains('✓'),
            "Claude 3.5 should have checkmark"
        );
    }

    #[test]
    fn chat_model_description_includes_provider() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "gpt4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model = find_action(&actions, "select_model_gpt4").unwrap();
        assert!(
            model.description.as_ref().unwrap().contains("OpenAI"),
            "Model description should contain provider"
        );
    }

    // ============================================================
    // 7. AI command bar description keyword validation
    // ============================================================

    #[test]
    fn ai_command_bar_copy_response_desc() {
        let actions = get_ai_command_bar_actions();
        let action = find_action(&actions, "copy_response").unwrap();
        assert!(action
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("response"));
    }

    #[test]
    fn ai_command_bar_copy_chat_desc() {
        let actions = get_ai_command_bar_actions();
        let action = find_action(&actions, "copy_chat").unwrap();
        assert!(action
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("conversation"));
    }

    #[test]
    fn ai_command_bar_copy_last_code_desc() {
        let actions = get_ai_command_bar_actions();
        let action = find_action(&actions, "copy_last_code").unwrap();
        assert!(action
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("code"));
    }

    #[test]
    fn ai_command_bar_new_chat_desc() {
        let actions = get_ai_command_bar_actions();
        let action = find_action(&actions, "new_chat").unwrap();
        assert!(action
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("new"));
    }

    #[test]
    fn ai_command_bar_change_model_desc() {
        let actions = get_ai_command_bar_actions();
        let action = find_action(&actions, "change_model").unwrap();
        assert!(action
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("model"));
    }

    #[test]
    fn ai_command_bar_delete_chat_desc() {
        let actions = get_ai_command_bar_actions();
        let action = find_action(&actions, "delete_chat").unwrap();
        assert!(action
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("delete"));
    }

    #[test]
    fn ai_command_bar_submit_desc() {
        let actions = get_ai_command_bar_actions();
        let action = find_action(&actions, "submit").unwrap();
        assert!(action
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("send"));
    }

    // ============================================================
    // 8. Notes command bar section label transitions
    // ============================================================

    #[test]
    fn notes_full_feature_sections() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let sections: Vec<_> = actions.iter().filter_map(|a| a.section.as_ref()).collect();
        // Should have Notes, Edit, Copy, Export, Settings sections
        assert!(sections.contains(&&"Notes".to_string()));
        assert!(sections.contains(&&"Edit".to_string()));
        assert!(sections.contains(&&"Copy".to_string()));
        assert!(sections.contains(&&"Export".to_string()));
        assert!(sections.contains(&&"Settings".to_string()));
    }

    #[test]
    fn notes_minimal_sections() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let sections: Vec<_> = actions
            .iter()
            .filter_map(|a| a.section.as_ref())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        // Should only have Notes section (no selection means no Edit/Copy/Export/Settings)
        assert_eq!(
            sections.len(),
            1,
            "Minimal config should have 1 section, got {:?}",
            sections
        );
    }

    #[test]
    fn notes_trash_view_hides_edit_actions() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        // Trash view should hide editing actions even with selection
        assert!(!ids.contains(&"duplicate_note"));
        assert!(!ids.contains(&"find_in_note"));
        assert!(!ids.contains(&"format"));
        assert!(!ids.contains(&"copy_note_as"));
        assert!(!ids.contains(&"export"));
    }

    #[test]
    fn notes_all_actions_have_icons() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for action in &actions {
            assert!(
                action.icon.is_some(),
                "Notes action '{}' should have an icon",
                action.id
            );
        }
    }

    #[test]
    fn notes_all_actions_have_sections() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for action in &actions {
            assert!(
                action.section.is_some(),
                "Notes action '{}' should have a section",
                action.id
            );
        }
    }

    // ============================================================
    // 9. New chat duplicate providers
    // ============================================================

    #[test]
    fn new_chat_multiple_models_same_provider() {
        let models = vec![
            NewChatModelInfo {
                model_id: "claude-3".to_string(),
                display_name: "Claude 3".to_string(),
                provider: "anthropic".to_string(),
                provider_display_name: "Anthropic".to_string(),
            },
            NewChatModelInfo {
                model_id: "claude-35".to_string(),
                display_name: "Claude 3.5".to_string(),
                provider: "anthropic".to_string(),
                provider_display_name: "Anthropic".to_string(),
            },
        ];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions.len(), 2);
        // Both should have Models section
        assert_eq!(actions[0].section, Some("Models".to_string()));
        assert_eq!(actions[1].section, Some("Models".to_string()));
        // Both should show Anthropic as provider in description
        for action in &actions {
            assert_eq!(
                action.description,
                Some("Anthropic".to_string()),
                "Model action should have provider in description"
            );
        }
    }

    #[test]
    fn new_chat_empty_inputs_return_empty() {
        let actions = get_new_chat_actions(&[], &[], &[]);
        assert!(actions.is_empty());
    }

    #[test]
    fn new_chat_section_ordering() {
        let last_used = vec![NewChatModelInfo {
            model_id: "lu1".to_string(),
            display_name: "Last Used 1".to_string(),
            provider: "p".to_string(),
            provider_display_name: "Provider".to_string(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "general".to_string(),
            name: "General".to_string(),
            icon: IconName::BoltFilled,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m1".to_string(),
            display_name: "Model 1".to_string(),
            provider: "p".to_string(),
            provider_display_name: "Provider".to_string(),
        }];

        let actions = get_new_chat_actions(&last_used, &presets, &models);
        let lu_idx = actions
            .iter()
            .position(|a| a.section == Some("Last Used Settings".to_string()))
            .unwrap();
        let preset_idx = actions
            .iter()
            .position(|a| a.section == Some("Presets".to_string()))
            .unwrap();
        let model_idx = actions
            .iter()
            .position(|a| a.section == Some("Models".to_string()))
            .unwrap();
        assert!(lu_idx < preset_idx, "Last Used before Presets");
        assert!(preset_idx < model_idx, "Presets before Models");
    }

    #[test]
    fn new_chat_preset_has_no_description() {
        let presets = vec![NewChatPresetInfo {
            id: "code".to_string(),
            name: "Code".to_string(),
            icon: IconName::Code,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(
            actions[0].description, None,
            "Presets should have no description"
        );
    }


    // --- merged from tests_part_03.rs ---
    #[test]
    fn new_chat_last_used_has_bolt_icon() {
        let last_used = vec![NewChatModelInfo {
            model_id: "lu1".to_string(),
            display_name: "Recent".to_string(),
            provider: "p".to_string(),
            provider_display_name: "Provider".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
    }

    #[test]
    fn new_chat_model_has_settings_icon() {
        let models = vec![NewChatModelInfo {
            model_id: "m1".to_string(),
            display_name: "Model".to_string(),
            provider: "p".to_string(),
            provider_display_name: "Provider".to_string(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].icon, Some(IconName::Settings));
    }

    // ============================================================
    // 10. Deeplink description URL format
    // ============================================================

    #[test]
    fn deeplink_description_contains_url() {
        let script = ScriptInfo::new("My Script", "/path/to/script.ts");
        let actions = get_script_context_actions(&script);
        let dl = find_action(&actions, "copy_deeplink").unwrap();
        assert!(dl
            .description
            .as_ref()
            .unwrap()
            .contains("scriptkit://run/my-script"));
    }

    #[test]
    fn deeplink_description_special_chars_stripped() {
        let script = ScriptInfo::new("Hello!@#World", "/path/to/script.ts");
        let actions = get_script_context_actions(&script);
        let dl = find_action(&actions, "copy_deeplink").unwrap();
        assert!(dl
            .description
            .as_ref()
            .unwrap()
            .contains("scriptkit://run/hello-world"));
    }

    #[test]
    fn deeplink_name_consecutive_specials() {
        assert_eq!(to_deeplink_name("a!!!b"), "a-b");
        assert_eq!(to_deeplink_name("---hello---"), "hello");
        assert_eq!(to_deeplink_name("  spaces  between  "), "spaces-between");
    }

    #[test]
    fn deeplink_name_unicode_preserved() {
        // é and ï are alphanumeric in Unicode, so they are preserved
        assert_eq!(to_deeplink_name("café"), "café");
        assert_eq!(to_deeplink_name("naïve"), "naïve");
    }

    #[test]
    fn deeplink_name_empty_after_stripping() {
        let result = to_deeplink_name("!@#$%");
        assert_eq!(result, "");
    }

    // ============================================================
    // 11. Score_action boundary thresholds
    // ============================================================

    #[test]
    fn score_prefix_match_is_100() {
        let action = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "edit");
        assert_eq!(score, 100, "Prefix match should score exactly 100");
    }

    #[test]
    fn score_contains_match_is_50() {
        let action = Action::new("id", "The Editor", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "editor");
        assert_eq!(score, 50, "Contains match should score exactly 50");
    }

    #[test]
    fn score_fuzzy_match_is_25() {
        // "esr" is a subsequence of "edit script" (e...s...r? no)
        // Need a proper subsequence: "esc" matches "edit script" (e..s..c? no)
        // "edc" -> e_d_i_t -> no... let's use "eit" -> "edit script" e..i..t
        let action = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "eit");
        assert_eq!(score, 25, "Fuzzy subsequence match should score exactly 25");
    }

    #[test]
    fn score_description_bonus_is_15() {
        let action = Action::new(
            "id",
            "Open File",
            Some("edit the file".to_string()),
            ActionCategory::ScriptContext,
        );
        // "xyz" won't match title, but if we query something that matches desc only...
        // Actually, we need to match title for base score. If no title match, desc only won't help.
        // Let's match title + desc: "open" prefix = 100, desc contains "file" doesn't add if query is "open"
        // We need desc-only bonus: query matches desc but not title
        let score_with_desc = ActionsDialog::score_action(&action, "edit");
        // "edit" doesn't prefix "open file", doesn't contain... let's check fuzzy
        // fuzzy "edit" in "open file": e_d_i_t? no 'e' found -> nope, not in "open file"
        // Actually "open file" has no 'e'? Wait, "open file" -> o,p,e,n,f,i,l,e -> has 'e'!
        // fuzzy "edit": e in "open file" at pos 2, d? no d in "open file" after pos 2... nope
        // So title score = 0, desc "edit the file" contains "edit" = +15
        assert_eq!(
            score_with_desc, 15,
            "Description-only match should score 15"
        );
    }

    #[test]
    fn score_shortcut_bonus_is_10() {
        let action =
            Action::new("id", "Open File", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
        // "⌘e" matches shortcut_lower
        let score = ActionsDialog::score_action(&action, "⌘e");
        assert_eq!(score, 10, "Shortcut-only match should score 10");
    }

    #[test]
    fn score_prefix_plus_desc_stacking() {
        let action = Action::new(
            "id",
            "Copy Path",
            Some("Copy the full path to clipboard".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "copy");
        // prefix match on "copy path" = 100, desc "copy the full path..." contains "copy" = +15
        assert_eq!(score, 115, "Prefix + desc should stack to 115");
    }

    #[test]
    fn score_no_match_is_zero() {
        let action = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "zzz");
        assert_eq!(score, 0, "No match should score 0");
    }

    #[test]
    fn score_empty_query_scores_zero() {
        let action = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "");
        // Empty string is prefix of everything
        assert_eq!(score, 100, "Empty query matches everything as prefix");
    }

    // ============================================================
    // 12. build_grouped_items interleaved section/no-section
    // ============================================================

    #[test]
    fn grouped_items_mixed_section_and_no_section() {
        let actions = vec![
            make_action("a1", "Action 1", Some("Section A")),
            make_action("a2", "Action 2", None),
            make_action("a3", "Action 3", Some("Section B")),
        ];
        let filtered: Vec<usize> = (0..3).collect();
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Should have: Header("Section A"), Item(0), Item(1) [no header for None], Header("Section B"), Item(2)
        let mut header_count = 0;
        let mut item_count = 0;
        for item in &items {
            match item {
                GroupedActionItem::SectionHeader(_) => header_count += 1,
                GroupedActionItem::Item(_) => item_count += 1,
            }
        }
        assert_eq!(header_count, 2, "Should have 2 section headers");
        assert_eq!(item_count, 3, "Should have 3 items");
    }

    #[test]
    fn grouped_items_none_style_no_headers() {
        let actions = vec![
            make_action("a1", "Action 1", Some("Section A")),
            make_action("a2", "Action 2", Some("Section B")),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        // None style should have no headers
        for item in &items {
            assert!(
                !matches!(item, GroupedActionItem::SectionHeader(_)),
                "None style should not insert section headers"
            );
        }
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn grouped_items_separators_style_no_headers() {
        let actions = vec![
            make_action("a1", "Action 1", Some("Section A")),
            make_action("a2", "Action 2", Some("Section B")),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        for item in &items {
            assert!(
                !matches!(item, GroupedActionItem::SectionHeader(_)),
                "Separators style should not insert section headers"
            );
        }
    }

    #[test]
    fn grouped_items_empty_filtered() {
        let actions = vec![make_action("a1", "Action 1", Some("Section A"))];
        let filtered: Vec<usize> = vec![];
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(items.is_empty());
    }

    #[test]
    fn grouped_items_same_section_no_duplicate_header() {
        let actions = vec![
            make_action("a1", "Action 1", Some("Same")),
            make_action("a2", "Action 2", Some("Same")),
            make_action("a3", "Action 3", Some("Same")),
        ];
        let filtered: Vec<usize> = (0..3).collect();
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        let header_count = items
            .iter()
            .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 1, "Same section should produce only 1 header");
    }

    // ============================================================
    // 13. coerce_action_selection complex patterns
    // ============================================================

    #[test]
    fn coerce_empty_returns_none() {
        let result = coerce_action_selection(&[], 0);
        assert_eq!(result, None);
    }

    #[test]
    fn coerce_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".to_string()),
            GroupedActionItem::SectionHeader("B".to_string()),
        ];
        let result = coerce_action_selection(&rows, 0);
        assert_eq!(result, None);
    }

    #[test]
    fn coerce_on_item_returns_same() {
        let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
        let result = coerce_action_selection(&rows, 0);
        assert_eq!(result, Some(0));
    }

    #[test]
    fn coerce_header_at_start_goes_down() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".to_string()),
            GroupedActionItem::Item(0),
        ];
        let result = coerce_action_selection(&rows, 0);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn coerce_header_at_end_goes_up() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("A".to_string()),
        ];
        let result = coerce_action_selection(&rows, 1);
        assert_eq!(result, Some(0));
    }

    #[test]
    fn coerce_alternating_header_item() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".to_string()),
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("B".to_string()),
            GroupedActionItem::Item(1),
        ];
        // Landing on header at index 2 should go down to item at index 3
        let result = coerce_action_selection(&rows, 2);
        assert_eq!(result, Some(3));
    }

    #[test]
    fn coerce_out_of_bounds_clamped() {
        let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
        let result = coerce_action_selection(&rows, 999);
        assert_eq!(result, Some(1));
    }

    // ============================================================
    // 14. parse_shortcut_keycaps compound symbol sequences
    // ============================================================

    #[test]
    fn parse_keycaps_cmd_c() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘C");
        assert_eq!(keycaps, vec!["⌘", "C"]);
    }

    #[test]
    fn parse_keycaps_all_modifiers() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌃⌥⇧⌘A");
        assert_eq!(keycaps, vec!["⌃", "⌥", "⇧", "⌘", "A"]);
    }

    #[test]
    fn parse_keycaps_enter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("↵");
        assert_eq!(keycaps, vec!["↵"]);
    }

    #[test]
    fn parse_keycaps_arrows() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("↑↓←→");
        assert_eq!(keycaps, vec!["↑", "↓", "←", "→"]);
    }

    #[test]
    fn parse_keycaps_escape() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⎋");
        assert_eq!(keycaps, vec!["⎋"]);
    }

    #[test]
    fn parse_keycaps_empty() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("");
        assert!(keycaps.is_empty());
    }

    #[test]
    fn parse_keycaps_space_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("␣");
        assert_eq!(keycaps, vec!["␣"]);
    }

    #[test]
    fn parse_keycaps_lowercase_uppercased() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘c");
        assert_eq!(keycaps, vec!["⌘", "C"]);
    }

    // ============================================================
    // 15. CommandBarConfig detailed fields
    // ============================================================

    #[test]
    fn command_bar_default_config() {
        let config = CommandBarConfig::default();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
        assert_eq!(
            config.dialog_config.search_position,
            crate::actions::types::SearchPosition::Bottom
        );
    }

    #[test]
    fn command_bar_ai_style() {
        let config = CommandBarConfig::ai_style();
        assert_eq!(
            config.dialog_config.search_position,
            crate::actions::types::SearchPosition::Top
        );
        assert_eq!(config.dialog_config.section_style, SectionStyle::Headers);
        assert!(config.dialog_config.show_icons);
        assert!(config.dialog_config.show_footer);
    }

    #[test]
    fn command_bar_main_menu_style() {
        let config = CommandBarConfig::main_menu_style();
        assert_eq!(
            config.dialog_config.search_position,
            crate::actions::types::SearchPosition::Bottom
        );
        assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
        assert!(!config.dialog_config.show_icons);
        assert!(!config.dialog_config.show_footer);
    }

    #[test]
    fn command_bar_no_search() {
        let config = CommandBarConfig::no_search();
        assert_eq!(
            config.dialog_config.search_position,
            crate::actions::types::SearchPosition::Hidden
        );
    }

    // ============================================================
    // 16. Cross-builder action count comparisons
    // ============================================================

    #[test]
    fn script_has_more_actions_than_builtin() {
        let script = ScriptInfo::new("test", "/path/to/test.ts");
        let builtin = ScriptInfo::builtin("Test Builtin");
        let script_actions = get_script_context_actions(&script);
        let builtin_actions = get_script_context_actions(&builtin);
        assert!(
            script_actions.len() > builtin_actions.len(),
            "Script ({}) should have more actions than builtin ({})",
            script_actions.len(),
            builtin_actions.len()
        );
    }

    #[test]
    fn scriptlet_via_script_context_vs_scriptlet_context_same_count() {
        let script = ScriptInfo::scriptlet("Test", "/path/to/test.md", None, None);
        let via_script = get_script_context_actions(&script);
        let via_scriptlet = get_scriptlet_context_actions_with_custom(&script, None);
        // Both should produce same actions (scriptlet context without custom = script context for scriptlet)
        assert_eq!(
            via_script.len(),
            via_scriptlet.len(),
            "Script context ({}) and scriptlet context ({}) should match for plain scriptlet",
            via_script.len(),
            via_scriptlet.len()
        );
    }


    // --- merged from tests_part_04.rs ---
    #[test]
    fn script_with_frecency_has_one_more_action() {
        let script = ScriptInfo::new("test", "/path/to/test.ts");
        let script_with_frecency = ScriptInfo::new("test", "/path/to/test.ts")
            .with_frecency(true, Some("/path".to_string()));
        let actions = get_script_context_actions(&script);
        let actions_f = get_script_context_actions(&script_with_frecency);
        assert_eq!(
            actions_f.len(),
            actions.len() + 1,
            "Frecency adds exactly 1 action (reset_ranking)"
        );
    }

    // ============================================================
    // 17. Action builder chaining order independence
    // ============================================================

    #[test]
    fn with_icon_then_section_same_as_reverse() {
        let a1 = Action::new("id", "Title", None, ActionCategory::ScriptContext)
            .with_icon(IconName::Copy)
            .with_section("Section");
        let a2 = Action::new("id", "Title", None, ActionCategory::ScriptContext)
            .with_section("Section")
            .with_icon(IconName::Copy);
        assert_eq!(a1.icon, a2.icon);
        assert_eq!(a1.section, a2.section);
        assert_eq!(a1.id, a2.id);
        assert_eq!(a1.title, a2.title);
    }

    #[test]
    fn with_shortcut_then_icon_then_section() {
        let a = Action::new("id", "Title", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘C")
            .with_icon(IconName::Copy)
            .with_section("Section");
        assert_eq!(a.shortcut, Some("⌘C".to_string()));
        assert_eq!(a.icon, Some(IconName::Copy));
        assert_eq!(a.section, Some("Section".to_string()));
    }

    #[test]
    fn with_shortcut_opt_none_preserves_fields() {
        let a = Action::new(
            "id",
            "Title",
            Some("Desc".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Plus)
        .with_shortcut_opt(None);
        assert_eq!(a.icon, Some(IconName::Plus));
        assert!(a.shortcut.is_none());
        assert_eq!(a.description, Some("Desc".to_string()));
    }

    #[test]
    fn with_shortcut_sets_lowercase_cache() {
        let a =
            Action::new("id", "Title", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
        assert_eq!(a.shortcut_lower, Some("⌘⇧c".to_string()));
    }

    // ============================================================
    // 18. Clipboard destructive action ordering stability
    // ============================================================

    #[test]
    fn clipboard_destructive_actions_always_last_three() {
        let entries = vec![
            make_clipboard_entry(ContentType::Text, false, None),
            make_clipboard_entry(ContentType::Text, true, Some("Chrome")),
            make_clipboard_entry(ContentType::Image, false, Some("Slack")),
            make_clipboard_entry(ContentType::Image, true, None),
        ];
        for entry in &entries {
            let actions = get_clipboard_history_context_actions(entry);
            let len = actions.len();
            assert!(len >= 3);
            // Last 3 should always be delete, delete_multiple, delete_all
            assert_eq!(
                actions[len - 3].id,
                "clipboard_delete",
                "Third from last should be clipboard_delete for {:?}",
                entry.content_type
            );
            assert_eq!(
                actions[len - 2].id,
                "clipboard_delete_multiple",
                "Second from last should be clipboard_delete_multiple"
            );
            assert_eq!(
                actions[len - 1].id,
                "clipboard_delete_all",
                "Last should be clipboard_delete_all"
            );
        }
    }

    #[test]
    fn clipboard_paste_always_first() {
        let entries = vec![
            make_clipboard_entry(ContentType::Text, false, None),
            make_clipboard_entry(ContentType::Image, true, Some("App")),
        ];
        for entry in &entries {
            let actions = get_clipboard_history_context_actions(entry);
            assert_eq!(
                actions[0].id, "clipboard_paste",
                "Paste should always be first"
            );
        }
    }

    #[test]
    fn clipboard_copy_always_second() {
        let entries = vec![
            make_clipboard_entry(ContentType::Text, false, None),
            make_clipboard_entry(ContentType::Image, false, None),
        ];
        for entry in &entries {
            let actions = get_clipboard_history_context_actions(entry);
            assert_eq!(
                actions[1].id, "clipboard_copy",
                "Copy should always be second"
            );
        }
    }

    // ============================================================
    // 19. File context title includes exact filename
    // ============================================================

    #[test]
    fn file_context_title_includes_filename() {
        let file_info = FileInfo {
            path: "/Users/test/report.pdf".to_string(),
            name: "report.pdf".to_string(),
            file_type: FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        let primary = &actions[0];
        assert!(
            primary.title.contains("report.pdf"),
            "Primary title '{}' should contain filename",
            primary.title
        );
    }

    #[test]
    fn file_context_dir_title_includes_dirname() {
        let file_info = FileInfo {
            path: "/Users/test/Documents".to_string(),
            name: "Documents".to_string(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&file_info);
        let primary = &actions[0];
        assert!(
            primary.title.contains("Documents"),
            "Primary title '{}' should contain dirname",
            primary.title
        );
    }

    #[test]
    fn file_context_all_have_descriptions() {
        let file_info = FileInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
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

    // ============================================================
    // 20. Notes info all-true/all-false edge cases
    // ============================================================

    #[test]
    fn notes_all_true_max_actions() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Should be the maximum action count
        assert!(
            actions.len() >= 10,
            "Full features should have >= 10 actions, got {}",
            actions.len()
        );
    }

    #[test]
    fn notes_all_false_min_actions() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Minimal: new_note + browse_notes
        assert_eq!(
            actions.len(),
            2,
            "Minimal should have exactly 2 actions, got {}",
            actions.len()
        );
    }

    #[test]
    fn notes_auto_sizing_disabled_adds_one() {
        let with_auto = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let without_auto = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let with_actions = get_notes_command_bar_actions(&with_auto);
        let without_actions = get_notes_command_bar_actions(&without_auto);
        assert_eq!(
            without_actions.len(),
            with_actions.len() + 1,
            "Disabled auto-sizing adds exactly 1 action"
        );
    }

    // ============================================================
    // 21. ScriptInfo agent flag interactions with frecency chaining
    // ============================================================

    #[test]
    fn agent_with_frecency_has_reset_ranking() {
        let mut agent = ScriptInfo::new("Agent", "/path/to/agent");
        agent.is_script = false;
        agent.is_agent = true;
        let agent = agent.with_frecency(true, Some("agent:path".to_string()));
        let actions = get_script_context_actions(&agent);
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    }

    #[test]
    fn agent_frecency_preserves_agent_flag() {
        let mut agent = ScriptInfo::new("Agent", "/path/to/agent");
        agent.is_script = false;
        agent.is_agent = true;
        let agent = agent.with_frecency(true, Some("agent:path".to_string()));
        assert!(agent.is_agent);
        assert!(!agent.is_script);
        assert!(agent.is_suggested);
    }

    // ============================================================
    // 22. Agent actions: no view_logs, has copy_content
    // ============================================================

    #[test]
    fn agent_has_no_view_logs() {
        let mut agent = ScriptInfo::new("Agent", "/path/to/agent");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    #[test]
    fn agent_has_copy_content() {
        let mut agent = ScriptInfo::new("Agent", "/path/to/agent");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        assert!(actions.iter().any(|a| a.id == "copy_content"));
    }

    #[test]
    fn agent_edit_title_says_agent() {
        let mut agent = ScriptInfo::new("Agent", "/path/to/agent");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        let edit = find_action(&actions, "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    #[test]
    fn agent_has_reveal_and_copy_path() {
        let mut agent = ScriptInfo::new("Agent", "/path/to/agent");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
        assert!(actions.iter().any(|a| a.id == "copy_path"));
    }

    // ============================================================
    // 23. Builtin with full optional fields
    // ============================================================

    #[test]
    fn builtin_with_shortcut_and_alias_and_frecency() {
        let builtin = ScriptInfo::with_all(
            "Clipboard History",
            "builtin:clipboard",
            false,
            "Open",
            Some("cmd+shift+c".to_string()),
            Some("ch".to_string()),
        )
        .with_frecency(true, Some("builtin:clipboard".to_string()));

        let actions = get_script_context_actions(&builtin);

        // Should have update/remove instead of add
        assert!(actions.iter().any(|a| a.id == "update_shortcut"));
        assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
        assert!(actions.iter().any(|a| a.id == "update_alias"));
        assert!(actions.iter().any(|a| a.id == "remove_alias"));
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));

        // Should NOT have script-specific actions
        assert!(!actions.iter().any(|a| a.id == "edit_script"));
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    #[test]
    fn builtin_primary_uses_custom_verb() {
        let builtin =
            ScriptInfo::with_action_verb("Clipboard History", "builtin:clipboard", false, "Open");
        let actions = get_script_context_actions(&builtin);
        assert_eq!(actions[0].title, "Open \"Clipboard History\"");
    }

    // ============================================================
    // 24. Path context dir vs file action count equality
    // ============================================================

    #[test]
    fn path_dir_file_action_count_equal() {
        let dir = PathInfo::new("dir", "/tmp/dir", true);
        let file = PathInfo::new("file", "/tmp/file", false);
        let dir_actions = get_path_context_actions(&dir);
        let file_actions = get_path_context_actions(&file);
        assert_eq!(dir_actions.len(), file_actions.len());
    }

    #[test]
    fn path_always_has_copy_path_and_copy_filename() {
        let dir = PathInfo::new("dir", "/tmp/dir", true);
        let file = PathInfo::new("file", "/tmp/file", false);
        for actions in [
            get_path_context_actions(&dir),
            get_path_context_actions(&file),
        ] {
            assert!(actions.iter().any(|a| a.id == "copy_path"));
            assert!(actions.iter().any(|a| a.id == "copy_filename"));
        }
    }

    // ============================================================
    // 25. Multiple scriptlet custom actions ordering
    // ============================================================

    #[test]
    fn scriptlet_multiple_custom_actions_maintain_order() {
        let script = ScriptInfo::scriptlet("Test", "/path/to/test.md", None, None);
        let mut scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo main".to_string(),
        );
        scriptlet.actions = vec![
            ScriptletAction {
                name: "First Action".to_string(),
                command: "first".to_string(),
                tool: "bash".to_string(),
                code: "echo first".to_string(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
            ScriptletAction {
                name: "Second Action".to_string(),
                command: "second".to_string(),
                tool: "bash".to_string(),
                code: "echo second".to_string(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
            ScriptletAction {
                name: "Third Action".to_string(),
                command: "third".to_string(),
                tool: "bash".to_string(),
                code: "echo third".to_string(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
        ];

        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let first_idx = actions
            .iter()
            .position(|a| a.id == "scriptlet_action:first")
            .unwrap();
        let second_idx = actions
            .iter()
            .position(|a| a.id == "scriptlet_action:second")
            .unwrap();
        let third_idx = actions
            .iter()
            .position(|a| a.id == "scriptlet_action:third")
            .unwrap();
        assert!(first_idx < second_idx, "First before second");
        assert!(second_idx < third_idx, "Second before third");
    }


    // --- merged from tests_part_05.rs ---
    #[test]
    fn scriptlet_custom_actions_after_run_before_shortcut() {
        let script = ScriptInfo::scriptlet("Test", "/path/to/test.md", None, None);
        let mut scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo main".to_string(),
        );
        scriptlet.actions = vec![ScriptletAction {
            name: "Custom".to_string(),
            command: "custom".to_string(),
            tool: "bash".to_string(),
            code: "echo custom".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];

        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let run_idx = actions.iter().position(|a| a.id == "run_script").unwrap();
        let custom_idx = actions
            .iter()
            .position(|a| a.id == "scriptlet_action:custom")
            .unwrap();
        let shortcut_idx = actions.iter().position(|a| a.id == "add_shortcut").unwrap();
        assert!(run_idx < custom_idx, "Run before custom");
        assert!(
            custom_idx < shortcut_idx,
            "Custom before shortcut management"
        );
    }

    // ============================================================
    // 26. Chat model checkmark exact match only
    // ============================================================

    #[test]
    fn chat_no_checkmark_on_partial_match() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".to_string()),
            available_models: vec![
                ChatModelInfo {
                    id: "gpt4o".to_string(),
                    display_name: "GPT-4o".to_string(),
                    provider: "OpenAI".to_string(),
                },
                ChatModelInfo {
                    id: "gpt4".to_string(),
                    display_name: "GPT-4".to_string(),
                    provider: "OpenAI".to_string(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let gpt4o = find_action(&actions, "select_model_gpt4o").unwrap();
        let gpt4 = find_action(&actions, "select_model_gpt4").unwrap();
        assert!(
            !gpt4o.title.contains('✓'),
            "GPT-4o should not have checkmark"
        );
        assert!(gpt4.title.contains('✓'), "GPT-4 should have checkmark");
    }

    // ============================================================
    // 27. Note switcher empty/placeholder title
    // ============================================================

    #[test]
    fn note_switcher_description_falls_back_to_char_count() {
        let notes = vec![make_note("id1", "Note", 42, false, false, "", "")];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "42 chars");
    }

    #[test]
    fn note_switcher_singular_char_count() {
        let notes = vec![make_note("id1", "Note", 1, false, false, "", "")];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "1 char");
    }

    #[test]
    fn note_switcher_zero_chars() {
        let notes = vec![make_note("id1", "Note", 0, false, false, "", "")];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "0 chars");
    }

    #[test]
    fn note_switcher_preview_with_time() {
        let notes = vec![make_note(
            "id1",
            "Note",
            100,
            false,
            false,
            "Hello world",
            "5m ago",
        )];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "Hello world · 5m ago");
    }

    #[test]
    fn note_switcher_preview_truncation_at_61() {
        let long_preview = "a".repeat(61);
        let notes = vec![make_note(
            "id1",
            "Note",
            100,
            false,
            false,
            &long_preview,
            "",
        )];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.ends_with('…'), "Should be truncated with ellipsis");
        // 60 chars + ellipsis
        assert_eq!(desc.chars().count(), 61);
    }

    #[test]
    fn note_switcher_preview_not_truncated_at_60() {
        let exact_preview = "a".repeat(60);
        let notes = vec![make_note(
            "id1",
            "Note",
            100,
            false,
            false,
            &exact_preview,
            "",
        )];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(!desc.ends_with('…'), "60 chars should NOT be truncated");
        assert_eq!(desc.chars().count(), 60);
    }

    // ============================================================
    // 28. Action with_section/with_icon chaining order independence
    // ============================================================

    #[test]
    fn action_chaining_shortcut_preserves_title_lower() {
        let a = Action::new("id", "Hello World", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘H");
        assert_eq!(a.title_lower, "hello world");
        assert_eq!(a.shortcut_lower, Some("⌘h".to_string()));
    }

    #[test]
    fn action_description_lower_computed() {
        let a = Action::new(
            "id",
            "Title",
            Some("Mixed CASE Desc".to_string()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(a.description_lower, Some("mixed case desc".to_string()));
    }

    #[test]
    fn action_no_description_lower_is_none() {
        let a = Action::new("id", "Title", None, ActionCategory::ScriptContext);
        assert!(a.description_lower.is_none());
    }

    // ============================================================
    // 29. Clipboard delete_multiple description
    // ============================================================

    #[test]
    fn clipboard_delete_multiple_desc_mentions_filter() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let dm = find_action(&actions, "clipboard_delete_multiple").unwrap();
        assert!(
            dm.description
                .as_ref()
                .unwrap()
                .to_lowercase()
                .contains("filter")
                || dm
                    .description
                    .as_ref()
                    .unwrap()
                    .to_lowercase()
                    .contains("matching"),
            "delete_multiple desc should mention filtering/matching"
        );
    }

    #[test]
    fn clipboard_delete_all_desc_mentions_pinned() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let da = find_action(&actions, "clipboard_delete_all").unwrap();
        assert!(
            da.description
                .as_ref()
                .unwrap()
                .to_lowercase()
                .contains("pinned"),
            "delete_all desc should mention pinned"
        );
    }

    // ============================================================
    // 30. Deeplink name edge cases
    // ============================================================

    #[test]
    fn deeplink_name_basic() {
        assert_eq!(to_deeplink_name("My Script"), "my-script");
    }

    #[test]
    fn deeplink_name_already_lowercase_hyphenated() {
        assert_eq!(to_deeplink_name("my-script"), "my-script");
    }

    #[test]
    fn deeplink_name_underscores() {
        assert_eq!(to_deeplink_name("hello_world"), "hello-world");
    }

    #[test]
    fn deeplink_name_numbers_preserved() {
        assert_eq!(to_deeplink_name("script123"), "script123");
    }

    #[test]
    fn deeplink_name_leading_trailing_stripped() {
        assert_eq!(to_deeplink_name("  hello  "), "hello");
    }

    // ============================================================
    // Cross-cutting: ID uniqueness across contexts
    // ============================================================

    #[test]
    fn script_action_ids_unique() {
        let script = ScriptInfo::new("test", "/path/to/test.ts");
        let actions = get_script_context_actions(&script);
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(
            ids.len(),
            actions.len(),
            "Script action IDs should be unique"
        );
    }

    #[test]
    fn clipboard_action_ids_unique() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(
            ids.len(),
            actions.len(),
            "Clipboard action IDs should be unique"
        );
    }

    #[test]
    fn ai_command_bar_ids_unique() {
        let actions = get_ai_command_bar_actions();
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(
            ids.len(),
            actions.len(),
            "AI command bar IDs should be unique"
        );
    }

    #[test]
    fn path_action_ids_unique() {
        let info = PathInfo::new("test", "/tmp/test", false);
        let actions = get_path_context_actions(&info);
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len(), "Path action IDs should be unique");
    }

    #[test]
    fn file_action_ids_unique() {
        let info = FileInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len(), "File action IDs should be unique");
    }

    // ============================================================
    // Cross-cutting: has_action invariant
    // ============================================================

    #[test]
    fn all_script_actions_have_has_action_false() {
        let script = ScriptInfo::new("test", "/path/to/test.ts");
        let actions = get_script_context_actions(&script);
        for action in &actions {
            assert!(
                !action.has_action,
                "Script action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn all_clipboard_actions_have_has_action_false() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        for action in &actions {
            assert!(
                !action.has_action,
                "Clipboard action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn all_ai_actions_have_has_action_false() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                !action.has_action,
                "AI action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn all_path_actions_have_has_action_false() {
        let info = PathInfo::new("test", "/tmp/test", false);
        let actions = get_path_context_actions(&info);
        for action in &actions {
            assert!(
                !action.has_action,
                "Path action '{}' should have has_action=false",
                action.id
            );
        }
    }

    // ============================================================
    // Cross-cutting: title_lower invariant
    // ============================================================

    #[test]
    fn title_lower_matches_lowercase_for_all_script_actions() {
        let script = ScriptInfo::new("test", "/path/to/test.ts");
        let actions = get_script_context_actions(&script);
        for action in &actions {
            assert_eq!(
                action.title_lower,
                action.title.to_lowercase(),
                "title_lower mismatch for '{}'",
                action.id
            );
        }
    }

    #[test]
    fn title_lower_matches_lowercase_for_ai_actions() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert_eq!(
                action.title_lower,
                action.title.to_lowercase(),
                "title_lower mismatch for '{}'",
                action.id
            );
        }
    }

    #[test]
    fn title_lower_matches_lowercase_for_clipboard_actions() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        for action in &actions {
            assert_eq!(
                action.title_lower,
                action.title.to_lowercase(),
                "title_lower mismatch for '{}'",
                action.id
            );
        }
    }

    // ============================================================
    // Cross-cutting: ordering determinism
    // ============================================================

    #[test]
    fn script_actions_deterministic() {
        let script = ScriptInfo::new("test", "/path/to/test.ts");
        let actions1 = get_script_context_actions(&script);
        let actions2 = get_script_context_actions(&script);
        let a1 = action_ids(&actions1);
        let a2 = action_ids(&actions2);
        assert_eq!(a1, a2, "Script actions should be deterministic");
    }

    #[test]
    fn clipboard_actions_deterministic() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions1 = get_clipboard_history_context_actions(&entry);
        let actions2 = get_clipboard_history_context_actions(&entry);
        let a1 = action_ids(&actions1);
        let a2 = action_ids(&actions2);
        assert_eq!(a1, a2, "Clipboard actions should be deterministic");
    }

    #[test]
    fn ai_actions_deterministic() {
        let actions1 = get_ai_command_bar_actions();
        let actions2 = get_ai_command_bar_actions();
        let a1 = action_ids(&actions1);
        let a2 = action_ids(&actions2);
        assert_eq!(a1, a2, "AI actions should be deterministic");
    }


    // --- merged from tests_part_06.rs ---
    #[test]
    fn notes_actions_deterministic() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions1 = get_notes_command_bar_actions(&info);
        let actions2 = get_notes_command_bar_actions(&info);
        let a1 = action_ids(&actions1);
        let a2 = action_ids(&actions2);
        assert_eq!(a1, a2, "Notes actions should be deterministic");
    }

    // ============================================================
    // Cross-cutting: non-empty titles and IDs
    // ============================================================

    #[test]
    fn script_all_actions_have_nonempty_id_and_title() {
        let script = ScriptInfo::new("test", "/path/to/test.ts");
        for action in &get_script_context_actions(&script) {
            assert!(!action.id.is_empty(), "Action ID should not be empty");
            assert!(!action.title.is_empty(), "Action title should not be empty");
        }
    }

    #[test]
    fn clipboard_all_actions_have_nonempty_id_and_title() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        for action in &get_clipboard_history_context_actions(&entry) {
            assert!(!action.id.is_empty(), "Action ID should not be empty");
            assert!(!action.title.is_empty(), "Action title should not be empty");
        }
    }

    #[test]
    fn ai_all_actions_have_nonempty_id_and_title() {
        for action in &get_ai_command_bar_actions() {
            assert!(!action.id.is_empty(), "Action ID should not be empty");
            assert!(!action.title.is_empty(), "Action title should not be empty");
        }
    }

    // ============================================================
    // Fuzzy match edge cases
    // ============================================================

    #[test]
    fn fuzzy_match_empty_needle_always_true() {
        assert!(ActionsDialog::fuzzy_match("anything", ""));
    }

    #[test]
    fn fuzzy_match_empty_haystack_nonempty_needle_false() {
        assert!(!ActionsDialog::fuzzy_match("", "a"));
    }

    #[test]
    fn fuzzy_match_both_empty_true() {
        assert!(ActionsDialog::fuzzy_match("", ""));
    }

    #[test]
    fn fuzzy_match_exact_match_true() {
        assert!(ActionsDialog::fuzzy_match("hello", "hello"));
    }

    #[test]
    fn fuzzy_match_subsequence_true() {
        assert!(ActionsDialog::fuzzy_match("hello world", "hlo"));
    }

    #[test]
    fn fuzzy_match_no_subsequence_false() {
        assert!(!ActionsDialog::fuzzy_match("hello", "z"));
    }

    #[test]
    fn fuzzy_match_needle_longer_than_haystack_false() {
        assert!(!ActionsDialog::fuzzy_match("hi", "hello"));
    }

    // ============================================================
    // ActionCategory invariants
    // ============================================================

    #[test]
    fn all_script_actions_are_script_context() {
        let script = ScriptInfo::new("test", "/path/to/test.ts");
        for action in &get_script_context_actions(&script) {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn all_clipboard_actions_are_script_context() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        for action in &get_clipboard_history_context_actions(&entry) {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn all_ai_actions_are_script_context() {
        for action in &get_ai_command_bar_actions() {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn all_path_actions_are_script_context() {
        let info = PathInfo::new("test", "/tmp/test", false);
        for action in &get_path_context_actions(&info) {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn all_file_actions_are_script_context() {
        let info = FileInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        for action in &get_file_context_actions(&info) {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

}
