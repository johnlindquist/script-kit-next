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
    use crate::actions::types::{
        Action, ActionCategory, ActionsDialogConfig, AnchorPosition, ScriptInfo, SearchPosition,
        SectionStyle,
    };
    use crate::actions::CommandBarConfig;
    use crate::clipboard_history::ContentType;
    use crate::designs::icon_variations::IconName;
    use crate::file_search::{FileInfo, FileType};
    use crate::prompts::PathInfo;
    use crate::scriptlets::{Scriptlet, ScriptletAction};
    use std::collections::HashSet;

    // ========================================
    // Helpers
    // ========================================

    fn action_ids(actions: &[Action]) -> Vec<&str> {
        actions.iter().map(|a| a.id.as_str()).collect()
    }

    fn find_action<'a>(actions: &'a [Action], id: &str) -> Option<&'a Action> {
        actions.iter().find(|a| a.id == id)
    }

    fn make_action(id: &str, title: &str, section: Option<&str>) -> Action {
        let mut a = Action::new(id, title, None, ActionCategory::ScriptContext);
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
        app: Option<&str>,
    ) -> ClipboardEntryInfo {
        ClipboardEntryInfo {
            id: "test-id".to_string(),
            content_type,
            pinned,
            preview: "test preview".to_string(),
            image_dimensions: if content_type == ContentType::Image {
                Some((800, 600))
            } else {
                None
            },
            frontmost_app_name: app.map(|s| s.to_string()),
        }
    }

    // ========================================
    // 1. Clipboard frontmost_app_name propagation (6 tests)
    // ========================================

    #[test]
    fn clipboard_paste_title_no_app() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = find_action(&actions, "clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to Active App");
    }

    #[test]
    fn clipboard_paste_title_with_safari() {
        let entry = make_clipboard_entry(ContentType::Text, false, Some("Safari"));
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = find_action(&actions, "clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to Safari");
    }

    #[test]
    fn clipboard_paste_title_with_unicode_app() {
        let entry = make_clipboard_entry(ContentType::Text, false, Some("日本語App"));
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = find_action(&actions, "clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to 日本語App");
    }

    #[test]
    fn clipboard_paste_title_with_empty_string_app() {
        let entry = make_clipboard_entry(ContentType::Text, false, Some(""));
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = find_action(&actions, "clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to ");
    }

    #[test]
    fn clipboard_app_name_only_affects_paste() {
        let entry_with = make_clipboard_entry(ContentType::Text, false, Some("VS Code"));
        let entry_without = make_clipboard_entry(ContentType::Text, false, None);
        let actions_with = get_clipboard_history_context_actions(&entry_with);
        let actions_without = get_clipboard_history_context_actions(&entry_without);
        // All non-paste actions should be identical
        for action_with in &actions_with {
            if action_with.id == "clipboard_paste" {
                continue;
            }
            let action_without = find_action(&actions_without, &action_with.id).unwrap();
            assert_eq!(action_with.title, action_without.title);
            assert_eq!(action_with.description, action_without.description);
            assert_eq!(action_with.shortcut, action_without.shortcut);
        }
    }

    #[test]
    fn clipboard_app_name_image_paste_title() {
        let entry = make_clipboard_entry(ContentType::Image, false, Some("Preview"));
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = find_action(&actions, "clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to Preview");
    }

    // ========================================
    // 2. Script action exact counts per flag combo (7 tests)
    // ========================================

    #[test]
    fn script_no_shortcut_no_alias_action_count() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        // run, add_shortcut, add_alias, edit, view_logs, reveal, copy_path, copy_content, copy_deeplink = 9
        assert_eq!(actions.len(), 9);
    }

    #[test]
    fn script_with_shortcut_action_count() {
        let script = ScriptInfo::with_shortcut("test", "/path/test.ts", Some("cmd+t".to_string()));
        let actions = get_script_context_actions(&script);
        // run, update_shortcut, remove_shortcut, add_alias, edit, view_logs, reveal, copy_path, copy_content, copy_deeplink = 10
        assert_eq!(actions.len(), 10);
    }

    #[test]
    fn script_with_shortcut_and_alias_action_count() {
        let script = ScriptInfo::with_shortcut_and_alias(
            "test",
            "/path/test.ts",
            Some("cmd+t".to_string()),
            Some("ts".to_string()),
        );
        let actions = get_script_context_actions(&script);
        // run, update_shortcut, remove_shortcut, update_alias, remove_alias, edit, view_logs, reveal, copy_path, copy_content, copy_deeplink = 11
        assert_eq!(actions.len(), 11);
    }

    #[test]
    fn builtin_no_shortcut_no_alias_action_count() {
        let builtin = ScriptInfo::builtin("Test Builtin");
        let actions = get_script_context_actions(&builtin);
        // run, add_shortcut, add_alias, copy_deeplink = 4
        assert_eq!(actions.len(), 4);
    }

    #[test]
    fn scriptlet_no_shortcut_no_alias_action_count() {
        let scriptlet = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_script_context_actions(&scriptlet);
        // run, add_shortcut, add_alias, edit_scriptlet, reveal_scriptlet, copy_scriptlet_path, copy_content, copy_deeplink = 8
        assert_eq!(actions.len(), 8);
    }

    #[test]
    fn agent_no_shortcut_no_alias_action_count() {
        let mut agent = ScriptInfo::new("Agent", "/path/agent.md");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        // run, add_shortcut, add_alias, edit(agent), reveal, copy_path, copy_content, copy_deeplink = 8
        assert_eq!(actions.len(), 8);
    }

    #[test]
    fn script_with_frecency_adds_one_action() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let base_count = get_script_context_actions(&script).len();
        let with_frecency = ScriptInfo::new("test", "/path/test.ts")
            .with_frecency(true, Some("/path/test.ts".to_string()));
        let frecency_count = get_script_context_actions(&with_frecency).len();
        assert_eq!(frecency_count, base_count + 1);
    }

    // ========================================
    // 3. Scriptlet ordering guarantees (5 tests)
    // ========================================

    #[test]
    fn scriptlet_context_run_is_first() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert_eq!(actions[0].id, "run_script");
    }

    #[test]
    fn scriptlet_context_custom_before_edit() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let mut scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo hi".to_string(),
        );
        scriptlet.actions = vec![ScriptletAction {
            name: "My Custom".to_string(),
            command: "my-custom".to_string(),
            tool: "bash".to_string(),
            code: "echo custom".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let custom_idx = actions
            .iter()
            .position(|a| a.id == "scriptlet_action:my-custom")
            .unwrap();
        let edit_idx = actions
            .iter()
            .position(|a| a.id == "edit_scriptlet")
            .unwrap();
        assert!(custom_idx < edit_idx);
    }

    #[test]
    fn scriptlet_context_edit_before_reveal() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let edit_idx = actions
            .iter()
            .position(|a| a.id == "edit_scriptlet")
            .unwrap();
        let reveal_idx = actions
            .iter()
            .position(|a| a.id == "reveal_scriptlet_in_finder")
            .unwrap();
        assert!(edit_idx < reveal_idx);
    }

    #[test]
    fn scriptlet_context_copy_content_before_deeplink() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let copy_idx = actions.iter().position(|a| a.id == "copy_content").unwrap();
        let deeplink_idx = actions
            .iter()
            .position(|a| a.id == "copy_deeplink")
            .unwrap();
        assert!(copy_idx < deeplink_idx);
    }

    #[test]
    fn scriptlet_context_deeplink_before_reset_ranking() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None)
            .with_frecency(true, Some("x".to_string()));
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let deeplink_idx = actions
            .iter()
            .position(|a| a.id == "copy_deeplink")
            .unwrap();
        let reset_idx = actions
            .iter()
            .position(|a| a.id == "reset_ranking")
            .unwrap();
        assert!(deeplink_idx < reset_idx);
    }

    // ========================================
    // 4. AI command bar exact shortcuts (9 tests)
    // ========================================

    #[test]
    fn ai_cmd_bar_copy_response_shortcut() {
        let actions = get_ai_command_bar_actions();
        let a = find_action(&actions, "copy_response").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⇧⌘C"));
    }

    #[test]
    fn ai_cmd_bar_copy_chat_shortcut() {
        let actions = get_ai_command_bar_actions();
        let a = find_action(&actions, "copy_chat").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌥⇧⌘C"));
    }

    #[test]
    fn ai_cmd_bar_copy_last_code_shortcut() {
        let actions = get_ai_command_bar_actions();
        let a = find_action(&actions, "copy_last_code").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌥⌘C"));
    }

    #[test]
    fn ai_cmd_bar_submit_shortcut() {
        let actions = get_ai_command_bar_actions();
        let a = find_action(&actions, "submit").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("↵"));
    }

    #[test]
    fn ai_cmd_bar_new_chat_shortcut() {
        let actions = get_ai_command_bar_actions();
        let a = find_action(&actions, "new_chat").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘N"));
    }

    #[test]
    fn ai_cmd_bar_delete_chat_shortcut() {
        let actions = get_ai_command_bar_actions();
        let a = find_action(&actions, "delete_chat").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘⌫"));
    }

    #[test]
    fn ai_cmd_bar_add_attachment_shortcut() {
        let actions = get_ai_command_bar_actions();
        let a = find_action(&actions, "add_attachment").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⇧⌘A"));
    }

    #[test]
    fn ai_cmd_bar_paste_image_shortcut() {
        let actions = get_ai_command_bar_actions();
        let a = find_action(&actions, "paste_image").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘V"));
    }

    #[test]
    fn ai_cmd_bar_change_model_no_shortcut() {
        let actions = get_ai_command_bar_actions();
        let a = find_action(&actions, "change_model").unwrap();
        assert!(a.shortcut.is_none());
    }

    // ========================================
    // 5. Notes command bar exact icons (8 tests)
    // ========================================

    #[test]
    fn notes_cmd_bar_new_note_icon_plus() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "new_note").unwrap();
        assert_eq!(a.icon, Some(IconName::Plus));
    }

    #[test]
    fn notes_cmd_bar_duplicate_note_icon_copy() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "duplicate_note").unwrap();
        assert_eq!(a.icon, Some(IconName::Copy));
    }

    #[test]
    fn notes_cmd_bar_browse_notes_icon_folder_open() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "browse_notes").unwrap();
        assert_eq!(a.icon, Some(IconName::FolderOpen));
    }

    #[test]
    fn notes_cmd_bar_find_in_note_icon_magnifying_glass() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "find_in_note").unwrap();
        assert_eq!(a.icon, Some(IconName::MagnifyingGlass));
    }

    #[test]
    fn notes_cmd_bar_format_icon_code() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "format").unwrap();
        assert_eq!(a.icon, Some(IconName::Code));
    }

    #[test]
    fn notes_cmd_bar_copy_deeplink_icon_arrow_right() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "copy_deeplink").unwrap();
        assert_eq!(a.icon, Some(IconName::ArrowRight));
    }

