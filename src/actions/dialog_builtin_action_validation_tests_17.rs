//! Batch 17 – Dialog Builtin Action Validation Tests
//!
//! 30 categories, ~150 tests validating random built-in action behaviors.
//!
//! Categories:
//! 01. Script context exact action count by ScriptInfo type
//! 02. Scriptlet context copy_content action details
//! 03. Path context total action count and primary action
//! 04. Clipboard paste action description content
//! 05. AI command bar shortcut completeness (which actions lack shortcuts)
//! 06. Notes command bar duplicate_note conditional visibility
//! 07. Note switcher empty notes fallback placeholder
//! 08. Chat context model ID format pattern
//! 09. Scriptlet defined action ID prefix invariant
//! 10. Agent context reveal_in_finder and copy_path shortcuts
//! 11. File context exact description strings
//! 12. Path context exact description strings
//! 13. Clipboard text/image macOS action count difference
//! 14. Script run title format includes quotes
//! 15. to_deeplink_name with whitespace variations
//! 16. Action::new pre-computes lowercase fields
//! 17. ActionsDialog::format_shortcut_hint SDK-style shortcuts
//! 18. ActionsDialog::parse_shortcut_keycaps compound shortcuts
//! 19. ActionsDialog::score_action multi-field bonus stacking
//! 20. ActionsDialog::fuzzy_match character ordering requirement
//! 21. build_grouped_items_static with no-section actions
//! 22. coerce_action_selection alternating header-item pattern
//! 23. CommandBarConfig notes_style preset values
//! 24. Clipboard unpin action title and description text
//! 25. New chat actions mixed section sizes
//! 26. Notes command bar browse_notes action details
//! 27. Script context copy_deeplink description contains URL
//! 28. Cross-context all actions are ScriptContext category
//! 29. Action with_icon chaining preserves all fields
//! 30. Script context action stability across flag combinations

#[cfg(test)]
mod tests {
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

    #[test]
    fn cat08_non_current_model_no_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".into()),
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
        assert!(!model_action.title.contains("✓"));
    }

    // ================================================================
    // Cat 09: Scriptlet defined action ID prefix invariant
    // ================================================================

    #[test]
    fn cat09_scriptlet_action_id_has_prefix() {
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo hi".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Do Thing".into(),
            command: "do-thing".into(),
            tool: "bash".into(),
            code: "echo done".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert!(actions[0].id.starts_with("scriptlet_action:"));
    }

    #[test]
    fn cat09_scriptlet_action_id_contains_command() {
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo hi".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Do Thing".into(),
            command: "do-thing".into(),
            tool: "bash".into(),
            code: "echo done".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions[0].id, "scriptlet_action:do-thing");
    }

    #[test]
    fn cat09_all_scriptlet_actions_have_prefix() {
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo hi".into());
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
        let actions = get_scriptlet_defined_actions(&scriptlet);
        for action in &actions {
            assert!(
                action.id.starts_with("scriptlet_action:"),
                "ID: {}",
                action.id
            );
        }
    }

    #[test]
    fn cat09_scriptlet_actions_all_have_action_true() {
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo hi".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "A".into(),
            command: "a".into(),
            tool: "bash".into(),
            code: "echo a".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert!(actions[0].has_action);
    }

    // ================================================================
    // Cat 10: Agent context reveal_in_finder and copy_path shortcuts
    // ================================================================

    #[test]
    fn cat10_agent_has_reveal_in_finder() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
    }

    #[test]
    fn cat10_agent_reveal_shortcut() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
        assert_eq!(reveal.shortcut.as_deref(), Some("⌘⇧F"));
    }

    #[test]
    fn cat10_agent_copy_path_shortcut() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
        assert_eq!(cp.shortcut.as_deref(), Some("⌘⇧C"));
    }

    #[test]
    fn cat10_agent_edit_shortcut() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.shortcut.as_deref(), Some("⌘E"));
    }

    #[test]
    fn cat10_agent_has_copy_content() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "copy_content"));
    }

    // ================================================================
    // Cat 11: File context exact description strings
    // ================================================================

    #[test]
    fn cat11_file_open_description() {
        let fi = FileInfo {
            path: "/test/file.txt".into(),
            name: "file.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&fi);
        let open = actions.iter().find(|a| a.id == "open_file").unwrap();
        assert_eq!(
            open.description.as_deref(),
            Some("Open with default application")
        );
    }

    #[test]
    fn cat11_dir_open_description() {
        let fi = FileInfo {
            path: "/test/dir".into(),
            name: "dir".into(),
            file_type: crate::file_search::FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&fi);
        let open = actions.iter().find(|a| a.id == "open_directory").unwrap();
        assert_eq!(open.description.as_deref(), Some("Open this folder"));
    }

    #[test]
    fn cat11_reveal_description() {
        let fi = FileInfo {
            path: "/test/file.txt".into(),
            name: "file.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&fi);
        let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
        assert_eq!(reveal.description.as_deref(), Some("Reveal in Finder"));
    }

    #[test]
    fn cat11_copy_path_description() {
        let fi = FileInfo {
            path: "/test/file.txt".into(),
            name: "file.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&fi);
        let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
        assert!(cp.description.as_ref().unwrap().contains("path"));
    }

    #[test]
    fn cat11_copy_filename_description() {
        let fi = FileInfo {
            path: "/test/file.txt".into(),
            name: "file.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&fi);
        let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
        assert!(cf.description.as_ref().unwrap().contains("filename"));
    }

    // ================================================================
    // Cat 12: Path context exact description strings
    // ================================================================

    #[test]
    fn cat12_path_dir_primary_description() {
        let info = PathInfo {
            path: "/a/b".into(),
            name: "b".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let primary = actions.iter().find(|a| a.id == "open_directory").unwrap();
        assert!(primary.description.as_ref().unwrap().contains("directory"));
    }

    #[test]
    fn cat12_path_file_primary_description() {
        let info = PathInfo {
            path: "/a/b.txt".into(),
            name: "b.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let primary = actions.iter().find(|a| a.id == "select_file").unwrap();
        assert!(primary.description.as_ref().unwrap().contains("file"));
    }

    #[test]
    fn cat12_path_open_in_editor_description() {
        let info = PathInfo {
            path: "/a/b".into(),
            name: "b".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let editor = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
        assert!(editor.description.as_ref().unwrap().contains("$EDITOR"));
    }

    #[test]
    fn cat12_path_move_to_trash_dir_description() {
        let info = PathInfo {
            path: "/a/b".into(),
            name: "b".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
        assert!(trash.description.as_ref().unwrap().contains("folder"));
    }

    #[test]
    fn cat12_path_move_to_trash_file_description() {
        let info = PathInfo {
            path: "/a/b.txt".into(),
            name: "b.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
        assert!(trash.description.as_ref().unwrap().contains("file"));
    }

    // ================================================================
    // Cat 13: Clipboard text/image macOS action count difference
    // ================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn cat13_image_has_more_actions_than_text_macos() {
        let text_entry = ClipboardEntryInfo {
            id: "t".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let img_entry = ClipboardEntryInfo {
            id: "i".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "img".into(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let text_actions = get_clipboard_history_context_actions(&text_entry);
        let img_actions = get_clipboard_history_context_actions(&img_entry);
        assert!(img_actions.len() > text_actions.len());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat13_image_has_ocr_text_does_not_macos() {
        let text_entry = ClipboardEntryInfo {
            id: "t".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let img_entry = ClipboardEntryInfo {
            id: "i".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "img".into(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let text_ids = action_ids(&get_clipboard_history_context_actions(&text_entry));
        let img_ids = action_ids(&get_clipboard_history_context_actions(&img_entry));
        assert!(!text_ids.contains(&"clipboard_ocr".to_string()));
        assert!(img_ids.contains(&"clipboard_ocr".to_string()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat13_text_count_macos() {
        let entry = ClipboardEntryInfo {
            id: "t".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        // paste, copy, paste_keep_open, share, attach_to_ai, quick_look,
        // pin, save_snippet, save_file, delete, delete_multiple, delete_all = 12
        assert_eq!(actions.len(), 12);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat13_image_count_macos() {
        let entry = ClipboardEntryInfo {
            id: "i".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "img".into(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        // paste, copy, paste_keep_open, share, attach_to_ai, quick_look,
        // open_with, annotate_cleanshot, upload_cleanshot, pin, ocr,
        // save_snippet, save_file, delete, delete_multiple, delete_all = 16
        assert_eq!(actions.len(), 16);
    }

    // ================================================================
    // Cat 14: Script run title format includes quotes
    // ================================================================

    #[test]
    fn cat14_run_title_has_quotes_around_name() {
        let script = ScriptInfo::new("My Script", "/path/my-script.ts");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert_eq!(run.title, "Run \"My Script\"");
    }

    #[test]
    fn cat14_custom_verb_in_title() {
        let script =
            ScriptInfo::with_action_verb("Safari", "/Applications/Safari.app", false, "Launch");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert_eq!(run.title, "Launch \"Safari\"");
    }

    #[test]
    fn cat14_switch_to_verb_in_title() {
        let script = ScriptInfo::with_action_verb("My Window", "window:123", false, "Switch to");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert_eq!(run.title, "Switch to \"My Window\"");
    }

    #[test]
    fn cat14_run_shortcut_is_enter() {
        let script = ScriptInfo::new("Test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert_eq!(run.shortcut.as_deref(), Some("↵"));
    }

    // ================================================================
    // Cat 15: to_deeplink_name with whitespace variations
    // ================================================================

    #[test]
    fn cat15_single_space() {
        assert_eq!(to_deeplink_name("A B"), "a-b");
    }

    #[test]
    fn cat15_multiple_spaces() {
        assert_eq!(to_deeplink_name("A  B   C"), "a-b-c");
    }

    #[test]
    fn cat15_tabs_converted() {
        assert_eq!(to_deeplink_name("A\tB"), "a-b");
    }

    #[test]
    fn cat15_leading_trailing_spaces() {
        assert_eq!(to_deeplink_name("  hello  "), "hello");
    }

    #[test]
    fn cat15_mixed_separators() {
        assert_eq!(to_deeplink_name("foo_bar baz-qux"), "foo-bar-baz-qux");
    }

    // ================================================================
    // Cat 16: Action::new pre-computes lowercase fields
    // ================================================================

    #[test]
    fn cat16_title_lower_cached() {
        let action = Action::new("id", "Hello World", None, ActionCategory::ScriptContext);
        assert_eq!(action.title_lower, "hello world");
    }

    #[test]
    fn cat16_description_lower_cached() {
        let action = Action::new(
            "id",
            "T",
            Some("Foo Bar".into()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.description_lower, Some("foo bar".into()));
    }

    #[test]
    fn cat16_description_none_lower_none() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext);
        assert!(action.description_lower.is_none());
    }

    #[test]
    fn cat16_shortcut_lower_none_initially() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext);
        assert!(action.shortcut_lower.is_none());
    }

    #[test]
    fn cat16_shortcut_lower_set_after_with_shortcut() {
        let action =
            Action::new("id", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
        assert_eq!(action.shortcut_lower.as_deref(), Some("⌘e"));
    }

    // ================================================================
    // Cat 17: ActionsDialog::format_shortcut_hint SDK-style shortcuts
    // ================================================================

    #[test]
    fn cat17_cmd_c() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+c"), "⌘C");
    }

    #[test]
    fn cat17_command_c() {
        assert_eq!(ActionsDialog::format_shortcut_hint("command+c"), "⌘C");
    }

    #[test]
    fn cat17_meta_c() {
        assert_eq!(ActionsDialog::format_shortcut_hint("meta+c"), "⌘C");
    }

    #[test]
    fn cat17_ctrl_alt_delete() {
        assert_eq!(
            ActionsDialog::format_shortcut_hint("ctrl+alt+delete"),
            "⌃⌥⌫"
        );
    }

    #[test]
    fn cat17_shift_enter() {
        assert_eq!(ActionsDialog::format_shortcut_hint("shift+enter"), "⇧↵");
    }

    #[test]
    fn cat17_option_space() {
        assert_eq!(ActionsDialog::format_shortcut_hint("option+space"), "⌥␣");
    }

    #[test]
    fn cat17_arrowup() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+arrowup"), "⌘↑");
    }

    // ================================================================
    // Cat 18: ActionsDialog::parse_shortcut_keycaps compound shortcuts
    // ================================================================

    #[test]
    fn cat18_cmd_enter_two_keycaps() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘↵");
        assert_eq!(caps, vec!["⌘", "↵"]);
    }

    #[test]
    fn cat18_cmd_shift_c_three_keycaps() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘⇧C");
        assert_eq!(caps, vec!["⌘", "⇧", "C"]);
    }

    #[test]
    fn cat18_single_letter_uppercased() {
        let caps = ActionsDialog::parse_shortcut_keycaps("a");
        assert_eq!(caps, vec!["A"]);
    }

    #[test]
    fn cat18_arrows() {
        let caps = ActionsDialog::parse_shortcut_keycaps("↑↓←→");
        assert_eq!(caps, vec!["↑", "↓", "←", "→"]);
    }

    #[test]
    fn cat18_empty_string() {
        let caps = ActionsDialog::parse_shortcut_keycaps("");
        assert!(caps.is_empty());
    }

    // ================================================================
    // Cat 19: ActionsDialog::score_action multi-field bonus stacking
    // ================================================================

    #[test]
    fn cat19_prefix_match_100() {
        let action = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
        assert_eq!(ActionsDialog::score_action(&action, "edit"), 100);
    }

    #[test]
    fn cat19_prefix_plus_description_115() {
        let action = Action::new(
            "id",
            "Edit Script",
            Some("Edit this script".into()),
            ActionCategory::ScriptContext,
        );
        // prefix(100) + description(15) = 115
        assert_eq!(ActionsDialog::score_action(&action, "edit"), 115);
    }

    #[test]
    fn cat19_contains_match_50() {
        let action = Action::new("id", "Script Editor", None, ActionCategory::ScriptContext);
        // "edit" is contained but not prefix in "script editor"
        assert_eq!(ActionsDialog::score_action(&action, "edit"), 50);
    }

    #[test]
    fn cat19_no_match_0() {
        let action = Action::new("id", "Run Script", None, ActionCategory::ScriptContext);
        assert_eq!(ActionsDialog::score_action(&action, "xyz"), 0);
    }

    #[test]
    fn cat19_shortcut_bonus_10() {
        let action = Action::new("id", "No Match Title", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘E");
        // "⌘e" matches shortcut_lower "⌘e" => +10
        assert_eq!(ActionsDialog::score_action(&action, "⌘e"), 10);
    }

    // ================================================================
    // Cat 20: ActionsDialog::fuzzy_match character ordering requirement
    // ================================================================

    #[test]
    fn cat20_correct_order_matches() {
        assert!(ActionsDialog::fuzzy_match("hello world", "hlo"));
    }

    #[test]
    fn cat20_wrong_order_no_match() {
        assert!(!ActionsDialog::fuzzy_match("hello world", "olh"));
    }

    #[test]
    fn cat20_exact_match() {
        assert!(ActionsDialog::fuzzy_match("abc", "abc"));
    }

    #[test]
    fn cat20_empty_needle_matches() {
        assert!(ActionsDialog::fuzzy_match("anything", ""));
    }

    #[test]
    fn cat20_empty_haystack_no_match() {
        assert!(!ActionsDialog::fuzzy_match("", "a"));
    }

    #[test]
    fn cat20_needle_longer_no_match() {
        assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
    }

    // ================================================================
    // Cat 21: build_grouped_items_static with no-section actions
    // ================================================================

    #[test]
    fn cat21_no_section_no_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext),
            Action::new("b", "B", None, ActionCategory::ScriptContext),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // No section set => no headers, just 2 items
        assert_eq!(grouped.len(), 2);
        assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(1)));
    }

    #[test]
    fn cat21_with_section_adds_header() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S"),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // 1 header + 2 items = 3
        assert_eq!(grouped.len(), 3);
        assert!(matches!(&grouped[0], GroupedActionItem::SectionHeader(s) if s == "S"));
    }

    #[test]
    fn cat21_separators_style_no_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        // No headers in Separators mode
        assert_eq!(grouped.len(), 2);
    }

    #[test]
    fn cat21_empty_input_empty_output() {
        let actions: Vec<Action> = vec![];
        let filtered: Vec<usize> = vec![];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(grouped.is_empty());
    }

    // ================================================================
    // Cat 22: coerce_action_selection alternating header-item pattern
    // ================================================================

    #[test]
    fn cat22_alternating_header_item_select_item() {
        let rows = vec![
            GroupedActionItem::SectionHeader("H1".into()),
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("H2".into()),
            GroupedActionItem::Item(1),
        ];
        // Index 0 is header => coerce to 1 (next item)
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn cat22_alternating_last_header_coerces_up() {
        let rows = vec![
            GroupedActionItem::SectionHeader("H1".into()),
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("H2".into()),
        ];
        // Index 2 is header, no items below => search up, find item at 1
        assert_eq!(coerce_action_selection(&rows, 2), Some(1));
    }

    #[test]
    fn cat22_item_at_index_stays() {
        let rows = vec![
            GroupedActionItem::SectionHeader("H".into()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(1));
    }

    #[test]
    fn cat22_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("H1".into()),
            GroupedActionItem::SectionHeader("H2".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn cat22_empty_returns_none() {
        let rows: Vec<GroupedActionItem> = vec![];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    // ================================================================
    // Cat 23: CommandBarConfig notes_style preset values
    // ================================================================

    #[test]
    fn cat23_notes_style_search_top() {
        let config = CommandBarConfig::notes_style();
        assert!(matches!(
            config.dialog_config.search_position,
            SearchPosition::Top
        ));
    }

    #[test]
    fn cat23_notes_style_separators() {
        let config = CommandBarConfig::notes_style();
        assert!(matches!(
            config.dialog_config.section_style,
            SectionStyle::Separators
        ));
    }

    #[test]
    fn cat23_notes_style_icons_enabled() {
        let config = CommandBarConfig::notes_style();
        assert!(config.dialog_config.show_icons);
    }

    #[test]
    fn cat23_notes_style_footer_enabled() {
        let config = CommandBarConfig::notes_style();
        assert!(config.dialog_config.show_footer);
    }

    #[test]
    fn cat23_notes_style_close_defaults_true() {
        let config = CommandBarConfig::notes_style();
        assert!(config.close_on_select);
        assert!(config.close_on_escape);
        assert!(config.close_on_click_outside);
    }

    // ================================================================
    // Cat 24: Clipboard unpin action title and description text
    // ================================================================

    #[test]
    fn cat24_unpin_title() {
        let entry = ClipboardEntryInfo {
            id: "e1".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let unpin = actions.iter().find(|a| a.id == "clipboard_unpin").unwrap();
        assert_eq!(unpin.title, "Unpin Entry");
    }

    #[test]
    fn cat24_unpin_description() {
        let entry = ClipboardEntryInfo {
            id: "e1".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let unpin = actions.iter().find(|a| a.id == "clipboard_unpin").unwrap();
        assert!(unpin.description.as_ref().unwrap().contains("pin"));
    }

    #[test]
    fn cat24_pin_title() {
        let entry = ClipboardEntryInfo {
            id: "e1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let pin = actions.iter().find(|a| a.id == "clipboard_pin").unwrap();
        assert_eq!(pin.title, "Pin Entry");
    }

    #[test]
    fn cat24_pin_unpin_same_shortcut() {
        let pinned_entry = ClipboardEntryInfo {
            id: "e1".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let unpinned_entry = ClipboardEntryInfo {
            id: "e2".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
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

    // ================================================================
    // Cat 25: New chat actions mixed section sizes
    // ================================================================

    #[test]
    fn cat25_multiple_last_used_single_preset_single_model() {
        let last_used = vec![
            NewChatModelInfo {
                model_id: "m1".into(),
                display_name: "M1".into(),
                provider: "p1".into(),
                provider_display_name: "P1".into(),
            },
            NewChatModelInfo {
                model_id: "m2".into(),
                display_name: "M2".into(),
                provider: "p2".into(),
                provider_display_name: "P2".into(),
            },
        ];
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m3".into(),
            display_name: "M3".into(),
            provider: "p3".into(),
            provider_display_name: "P3".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        assert_eq!(actions.len(), 4); // 2 + 1 + 1
    }

    #[test]
    fn cat25_sections_are_correct() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "M1".into(),
            provider: "p1".into(),
            provider_display_name: "P1".into(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "gen".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m2".into(),
            display_name: "M2".into(),
            provider: "p2".into(),
            provider_display_name: "P2".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        let sections: Vec<&str> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        assert_eq!(sections, vec!["Last Used Settings", "Presets", "Models"]);
    }

    #[test]
    fn cat25_preset_has_no_description() {
        let presets = vec![NewChatPresetInfo {
            id: "gen".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert!(actions[0].description.is_none());
    }

    #[test]
    fn cat25_model_has_provider_description() {
        let models = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "M1".into(),
            provider: "p1".into(),
            provider_display_name: "Provider One".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].description.as_deref(), Some("Provider One"));
    }

    // ================================================================
    // Cat 26: Notes command bar browse_notes action details
    // ================================================================

    #[test]
    fn cat26_browse_notes_always_present() {
        let full = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let minimal = NotesInfo {
            has_selection: false,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let full_actions = get_notes_command_bar_actions(&full);
        let minimal_actions = get_notes_command_bar_actions(&minimal);
        assert!(full_actions.iter().any(|a| a.id == "browse_notes"));
        assert!(minimal_actions.iter().any(|a| a.id == "browse_notes"));
    }

    #[test]
    fn cat26_browse_notes_shortcut() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let bn = actions.iter().find(|a| a.id == "browse_notes").unwrap();
        assert_eq!(bn.shortcut.as_deref(), Some("⌘P"));
    }

    #[test]
    fn cat26_browse_notes_icon() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let bn = actions.iter().find(|a| a.id == "browse_notes").unwrap();
        assert_eq!(bn.icon, Some(IconName::FolderOpen));
    }

    #[test]
    fn cat26_browse_notes_section() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let bn = actions.iter().find(|a| a.id == "browse_notes").unwrap();
        assert_eq!(bn.section.as_deref(), Some("Notes"));
    }

    // ================================================================
    // Cat 27: Script context copy_deeplink description contains URL
    // ================================================================

    #[test]
    fn cat27_deeplink_description_contains_scriptkit_url() {
        let script = ScriptInfo::new("My Cool Script", "/path/script.ts");
        let actions = get_script_context_actions(&script);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert!(dl
            .description
            .as_ref()
            .unwrap()
            .contains("scriptkit://run/"));
    }

    #[test]
    fn cat27_deeplink_description_contains_deeplink_name() {
        let script = ScriptInfo::new("My Cool Script", "/path/script.ts");
        let actions = get_script_context_actions(&script);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert!(dl.description.as_ref().unwrap().contains("my-cool-script"));
    }

    #[test]
    fn cat27_deeplink_shortcut() {
        let script = ScriptInfo::new("Test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert_eq!(dl.shortcut.as_deref(), Some("⌘⇧D"));
    }

    #[test]
    fn cat27_scriptlet_deeplink_also_has_url() {
        let script = ScriptInfo::scriptlet("Open GitHub", "/path/url.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert!(dl
            .description
            .as_ref()
            .unwrap()
            .contains("scriptkit://run/open-github"));
    }

    // ================================================================
    // Cat 28: Cross-context all actions are ScriptContext category
    // ================================================================

    #[test]
    fn cat28_script_actions_all_script_context() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        for action in &get_script_context_actions(&script) {
            assert_eq!(
                action.category,
                ActionCategory::ScriptContext,
                "ID: {}",
                action.id
            );
        }
    }

    #[test]
    fn cat28_clipboard_actions_all_script_context() {
        let entry = ClipboardEntryInfo {
            id: "e".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "t".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        for action in &get_clipboard_history_context_actions(&entry) {
            assert_eq!(
                action.category,
                ActionCategory::ScriptContext,
                "ID: {}",
                action.id
            );
        }
    }

    #[test]
    fn cat28_ai_actions_all_script_context() {
        for action in &get_ai_command_bar_actions() {
            assert_eq!(
                action.category,
                ActionCategory::ScriptContext,
                "ID: {}",
                action.id
            );
        }
    }

    #[test]
    fn cat28_path_actions_all_script_context() {
        let info = PathInfo {
            path: "/a".into(),
            name: "a".into(),
            is_dir: true,
        };
        for action in &get_path_context_actions(&info) {
            assert_eq!(
                action.category,
                ActionCategory::ScriptContext,
                "ID: {}",
                action.id
            );
        }
    }

    #[test]
    fn cat28_file_actions_all_script_context() {
        let fi = FileInfo {
            path: "/test/f.txt".into(),
            name: "f.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        for action in &get_file_context_actions(&fi) {
            assert_eq!(
                action.category,
                ActionCategory::ScriptContext,
                "ID: {}",
                action.id
            );
        }
    }

    #[test]
    fn cat28_notes_actions_all_script_context() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for action in &get_notes_command_bar_actions(&info) {
            assert_eq!(
                action.category,
                ActionCategory::ScriptContext,
                "ID: {}",
                action.id
            );
        }
    }

    // ================================================================
    // Cat 29: Action with_icon chaining preserves all fields
    // ================================================================

    #[test]
    fn cat29_with_icon_preserves_shortcut() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘E")
            .with_icon(IconName::Star);
        assert_eq!(action.shortcut.as_deref(), Some("⌘E"));
        assert_eq!(action.icon, Some(IconName::Star));
    }

    #[test]
    fn cat29_with_section_preserves_icon() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_icon(IconName::Copy)
            .with_section("S");
        assert_eq!(action.icon, Some(IconName::Copy));
        assert_eq!(action.section.as_deref(), Some("S"));
    }

    #[test]
    fn cat29_full_chain_all_fields() {
        let action = Action::new(
            "id",
            "Title",
            Some("Desc".into()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘X")
        .with_icon(IconName::Trash)
        .with_section("Section");
        assert_eq!(action.id, "id");
        assert_eq!(action.title, "Title");
        assert_eq!(action.description.as_deref(), Some("Desc"));
        assert_eq!(action.shortcut.as_deref(), Some("⌘X"));
        assert_eq!(action.icon, Some(IconName::Trash));
        assert_eq!(action.section.as_deref(), Some("Section"));
        assert!(!action.has_action);
    }

    #[test]
    fn cat29_with_shortcut_opt_none_preserves_existing() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘A")
            .with_shortcut_opt(None);
        // with_shortcut_opt(None) does NOT clear existing shortcut
        assert_eq!(action.shortcut.as_deref(), Some("⌘A"));
    }

    #[test]
    fn cat29_with_shortcut_opt_some_sets() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘B".into()));
        assert_eq!(action.shortcut.as_deref(), Some("⌘B"));
    }

    // ================================================================
    // Cat 30: Script context action stability across flag combinations
    // ================================================================

    #[test]
    fn cat30_script_deterministic() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let a1 = action_ids(&get_script_context_actions(&script));
        let a2 = action_ids(&get_script_context_actions(&script));
        assert_eq!(a1, a2);
    }

    #[test]
    fn cat30_builtin_deterministic() {
        let builtin = ScriptInfo::builtin("Test");
        let a1 = action_ids(&get_script_context_actions(&builtin));
        let a2 = action_ids(&get_script_context_actions(&builtin));
        assert_eq!(a1, a2);
    }

    #[test]
    fn cat30_scriptlet_deterministic() {
        let scriptlet = ScriptInfo::scriptlet("Test", "/path.md", None, None);
        let a1 = action_ids(&get_script_context_actions(&scriptlet));
        let a2 = action_ids(&get_script_context_actions(&scriptlet));
        assert_eq!(a1, a2);
    }

    #[test]
    fn cat30_agent_deterministic() {
        let mut agent = ScriptInfo::new("Agent", "/path/agent.md");
        agent.is_script = false;
        agent.is_agent = true;
        let a1 = action_ids(&get_script_context_actions(&agent));
        let a2 = action_ids(&get_script_context_actions(&agent));
        assert_eq!(a1, a2);
    }

    #[test]
    fn cat30_frecency_flag_adds_exactly_one_action() {
        let base = ScriptInfo::new("test", "/path/test.ts");
        let with_frecency =
            ScriptInfo::new("test", "/path/test.ts").with_frecency(true, Some("/path".into()));
        let base_count = get_script_context_actions(&base).len();
        let frecency_count = get_script_context_actions(&with_frecency).len();
        assert_eq!(frecency_count, base_count + 1);
    }
}
