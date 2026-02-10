    use super::super::builders::*;
    use super::super::command_bar::CommandBarConfig;
    use super::super::dialog::{
        build_grouped_items_static, coerce_action_selection, ActionsDialog, GroupedActionItem,
    };
    use super::super::types::*;
    use crate::clipboard_history::ContentType;
    use crate::designs::icon_variations::IconName;
    use crate::file_search::FileInfo;
    use crate::prompts::PathInfo;
    use crate::scriptlets::{Scriptlet, ScriptletAction};

    fn action_ids(actions: &[Action]) -> Vec<String> {
        actions.iter().map(|a| a.id.clone()).collect()
    }

    // ================================================================
    // Cat 01: Script context exact action count by ScriptInfo type
    // ================================================================

    #[test]
    fn cat01_script_new_no_shortcut_no_alias_count() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        // run, add_shortcut, add_alias, edit_script, view_logs,
        // reveal_in_finder, copy_path, copy_content, copy_deeplink = 9
        assert_eq!(actions.len(), 9);
    }

    #[test]
    fn cat01_script_with_shortcut_count() {
        let script = ScriptInfo::with_shortcut("test", "/path/test.ts", Some("cmd+t".into()));
        let actions = get_script_context_actions(&script);
        // run, update_shortcut, remove_shortcut, add_alias, edit_script,
        // view_logs, reveal_in_finder, copy_path, copy_content, copy_deeplink = 10
        assert_eq!(actions.len(), 10);
    }

    #[test]
    fn cat01_script_with_shortcut_and_alias_count() {
        let script = ScriptInfo::with_shortcut_and_alias(
            "test",
            "/path/test.ts",
            Some("cmd+t".into()),
            Some("ts".into()),
        );
        let actions = get_script_context_actions(&script);
        // run, update_shortcut, remove_shortcut, update_alias, remove_alias,
        // edit_script, view_logs, reveal_in_finder, copy_path, copy_content, copy_deeplink = 11
        assert_eq!(actions.len(), 11);
    }

    #[test]
    fn cat01_builtin_no_shortcut_count() {
        let builtin = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&builtin);
        // run, add_shortcut, add_alias, copy_deeplink = 4
        assert_eq!(actions.len(), 4);
    }

    #[test]
    fn cat01_scriptlet_no_shortcut_count() {
        let scriptlet = ScriptInfo::scriptlet("Open URL", "/path/url.md", None, None);
        let actions = get_script_context_actions(&scriptlet);
        // run, add_shortcut, add_alias, edit_scriptlet, reveal_scriptlet_in_finder,
        // copy_scriptlet_path, copy_content, copy_deeplink = 8
        assert_eq!(actions.len(), 8);
    }

    #[test]
    fn cat01_script_with_frecency_adds_reset_ranking() {
        let script = ScriptInfo::new("test", "/path/test.ts")
            .with_frecency(true, Some("/path/test.ts".into()));
        let actions = get_script_context_actions(&script);
        // base 9 + reset_ranking = 10
        assert_eq!(actions.len(), 10);
    }

    // ================================================================
    // Cat 02: Scriptlet context copy_content action details
    // ================================================================

    #[test]
    fn cat02_scriptlet_context_has_copy_content() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(actions.iter().any(|a| a.id == "copy_content"));
    }

    #[test]
    fn cat02_scriptlet_copy_content_shortcut() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
        assert_eq!(cc.shortcut.as_deref(), Some("⌘⌥C"));
    }

    #[test]
    fn cat02_scriptlet_copy_content_description() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
        assert!(cc.description.as_ref().unwrap().contains("file content"));
    }

    #[test]
    fn cat02_script_copy_content_same_shortcut() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
        assert_eq!(cc.shortcut.as_deref(), Some("⌘⌥C"));
    }

    // ================================================================
    // Cat 03: Path context total action count and primary action
    // ================================================================

    #[test]
    fn cat03_path_dir_total_count() {
        let info = PathInfo {
            path: "/Users/test/Documents".into(),
            name: "Documents".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        // open_directory, copy_path, open_in_finder, open_in_editor,
        // open_in_terminal, copy_filename, move_to_trash = 7
        assert_eq!(actions.len(), 7);
    }

    #[test]
    fn cat03_path_file_total_count() {
        let info = PathInfo {
            path: "/Users/test/file.txt".into(),
            name: "file.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        // select_file, copy_path, open_in_finder, open_in_editor,
        // open_in_terminal, copy_filename, move_to_trash = 7
        assert_eq!(actions.len(), 7);
    }

    #[test]
    fn cat03_path_dir_primary_is_open_directory() {
        let info = PathInfo {
            path: "/Users/test/Documents".into(),
            name: "Documents".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].id, "open_directory");
    }

    #[test]
    fn cat03_path_file_primary_is_select_file() {
        let info = PathInfo {
            path: "/Users/test/file.txt".into(),
            name: "file.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].id, "select_file");
    }

    #[test]
    fn cat03_path_dir_and_file_same_count() {
        let dir = PathInfo {
            path: "/a".into(),
            name: "a".into(),
            is_dir: true,
        };
        let file = PathInfo {
            path: "/b".into(),
            name: "b".into(),
            is_dir: false,
        };
        assert_eq!(
            get_path_context_actions(&dir).len(),
            get_path_context_actions(&file).len()
        );
    }

    // ================================================================
    // Cat 04: Clipboard paste action description content
    // ================================================================

    #[test]
    fn cat04_clipboard_paste_description_mentions_clipboard() {
        let entry = ClipboardEntryInfo {
            id: "e1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
        assert!(paste.description.as_ref().unwrap().contains("clipboard"));
    }

    #[test]
    fn cat04_clipboard_copy_description_mentions_clipboard() {
        let entry = ClipboardEntryInfo {
            id: "e1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let copy = actions.iter().find(|a| a.id == "clipboard_copy").unwrap();
        assert!(copy.description.as_ref().unwrap().contains("clipboard"));
    }

    #[test]
    fn cat04_clipboard_paste_keep_open_description() {
        let entry = ClipboardEntryInfo {
            id: "e1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let pko = actions
            .iter()
            .find(|a| a.id == "clipboard_paste_keep_open")
            .unwrap();
        assert!(pko.description.as_ref().unwrap().contains("keep"));
    }

    // ================================================================
    // Cat 05: AI command bar shortcut completeness
    // ================================================================

    #[test]
    fn cat05_branch_from_last_has_no_shortcut() {
        let actions = get_ai_command_bar_actions();
        let bfl = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
        assert!(bfl.shortcut.is_none());
    }

    #[test]
    fn cat05_change_model_has_no_shortcut() {
        let actions = get_ai_command_bar_actions();
        let cm = actions.iter().find(|a| a.id == "change_model").unwrap();
        assert!(cm.shortcut.is_none());
    }

    #[test]
    fn cat05_submit_has_shortcut_enter() {
        let actions = get_ai_command_bar_actions();
        let s = actions.iter().find(|a| a.id == "submit").unwrap();
        assert_eq!(s.shortcut.as_deref(), Some("↵"));
    }

    #[test]
    fn cat05_new_chat_has_shortcut_cmd_n() {
        let actions = get_ai_command_bar_actions();
        let nc = actions.iter().find(|a| a.id == "new_chat").unwrap();
        assert_eq!(nc.shortcut.as_deref(), Some("⌘N"));
    }

    #[test]
    fn cat05_actions_with_shortcuts_count() {
        let actions = get_ai_command_bar_actions();
        let with_shortcuts = actions.iter().filter(|a| a.shortcut.is_some()).count();
        // branch_from_last and change_model lack shortcuts => 12 - 2 = 10
        assert_eq!(with_shortcuts, 10);
    }

    // ================================================================
    // Cat 06: Notes command bar duplicate_note conditional visibility
    // ================================================================

    #[test]
    fn cat06_duplicate_present_with_selection_no_trash() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "duplicate_note"));
    }

    #[test]
    fn cat06_duplicate_absent_no_selection() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
    }

    #[test]
    fn cat06_duplicate_absent_in_trash() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
    }

    #[test]
    fn cat06_duplicate_absent_trash_and_no_selection() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
    }

    #[test]
    fn cat06_duplicate_shortcut_is_cmd_d() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let dup = actions.iter().find(|a| a.id == "duplicate_note").unwrap();
        assert_eq!(dup.shortcut.as_deref(), Some("⌘D"));
    }

    // ================================================================
    // Cat 07: Note switcher empty notes fallback placeholder
    // ================================================================

    #[test]
    fn cat07_empty_notes_returns_placeholder() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions.len(), 1);
    }

    #[test]
    fn cat07_placeholder_id_is_no_notes() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions[0].id, "no_notes");
    }

    #[test]
    fn cat07_placeholder_title() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions[0].title, "No notes yet");
    }

    #[test]
    fn cat07_placeholder_icon_is_plus() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions[0].icon, Some(IconName::Plus));
    }

    #[test]
    fn cat07_placeholder_section_is_notes() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions[0].section.as_deref(), Some("Notes"));
    }

    #[test]
    fn cat07_placeholder_description_mentions_cmd_n() {
        let actions = get_note_switcher_actions(&[]);
        assert!(actions[0].description.as_ref().unwrap().contains("⌘N"));
    }

    // ================================================================
    // Cat 08: Chat context model ID format pattern
    // ================================================================

    #[test]
    fn cat08_model_id_uses_select_model_prefix() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "claude-3".into(),
                display_name: "Claude 3".into(),
                provider: "Anthropic".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "select_model_claude-3"));
    }

    #[test]
    fn cat08_multiple_models_sequential_ids() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![
                ChatModelInfo {
                    id: "m1".into(),
                    display_name: "M1".into(),
                    provider: "P".into(),
                },
                ChatModelInfo {
                    id: "m2".into(),
                    display_name: "M2".into(),
                    provider: "P".into(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"select_model_m1".to_string()));
        assert!(ids.contains(&"select_model_m2".to_string()));
    }

    #[test]
    fn cat08_current_model_has_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("Claude 3".into()),
            available_models: vec![ChatModelInfo {
                id: "claude-3".into(),
                display_name: "Claude 3".into(),
                provider: "Anthropic".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = actions
            .iter()
            .find(|a| a.id == "select_model_claude-3")
            .unwrap();
        assert!(model_action.title.contains("✓"));
    }

