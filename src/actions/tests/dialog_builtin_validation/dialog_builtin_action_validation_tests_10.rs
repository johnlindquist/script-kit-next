// Batch 10: Builtin action validation tests
//
// 155 tests across 30 categories focusing on:
// - Clipboard frontmost_app_name propagation and isolation
// - Script action exact counts per flag combination
// - Scriptlet ordering guarantees with custom actions
// - AI command bar exact shortcut/icon values
// - Notes command bar exact icon/shortcut/section values
// - Path context exact shortcut values
// - File context exact description strings
// - FileType variants have no effect on file actions
// - Chat model checkmark logic and ID format
// - New chat provider_display_name propagation
// - Clipboard exact description strings
// - Script context with custom verbs
// - ActionsDialogConfig field defaults
// - ActionCategory PartialEq
// - Agent description content keywords
// - Cross-context frecency reset consistency

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


    // --- merged from tests_part_02.rs ---
    #[test]
    fn notes_cmd_bar_create_quicklink_icon_star() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "create_quicklink").unwrap();
        assert_eq!(a.icon, Some(IconName::Star));
    }

    #[test]
    fn notes_cmd_bar_enable_auto_sizing_icon_settings() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "enable_auto_sizing").unwrap();
        assert_eq!(a.icon, Some(IconName::Settings));
    }

    // ========================================
    // 6. Path context exact shortcut values (7 tests)
    // ========================================

    #[test]
    fn path_dir_primary_shortcut_enter() {
        let info = PathInfo {
            name: "docs".to_string(),
            path: "/home/docs".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let a = find_action(&actions, "open_directory").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("↵"));
    }

    #[test]
    fn path_file_primary_shortcut_enter() {
        let info = PathInfo {
            name: "readme.md".to_string(),
            path: "/home/readme.md".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let a = find_action(&actions, "select_file").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("↵"));
    }

    #[test]
    fn path_copy_path_shortcut() {
        let info = PathInfo {
            name: "f".to_string(),
            path: "/f".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let a = find_action(&actions, "copy_path").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘⇧C"));
    }

    #[test]
    fn path_open_in_finder_shortcut() {
        let info = PathInfo {
            name: "f".to_string(),
            path: "/f".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let a = find_action(&actions, "open_in_finder").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘⇧F"));
    }

    #[test]
    fn path_open_in_editor_shortcut() {
        let info = PathInfo {
            name: "f".to_string(),
            path: "/f".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let a = find_action(&actions, "open_in_editor").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘E"));
    }

    #[test]
    fn path_open_in_terminal_shortcut() {
        let info = PathInfo {
            name: "f".to_string(),
            path: "/f".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let a = find_action(&actions, "open_in_terminal").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘T"));
    }

    #[test]
    fn path_move_to_trash_shortcut() {
        let info = PathInfo {
            name: "f".to_string(),
            path: "/f".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let a = find_action(&actions, "move_to_trash").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘⌫"));
    }

    // ========================================
    // 7. File context exact description strings (6 tests)
    // ========================================

    #[test]
    fn file_open_file_description() {
        let fi = FileInfo {
            path: "/x/y.txt".to_string(),
            name: "y.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&fi);
        let a = find_action(&actions, "open_file").unwrap();
        assert_eq!(
            a.description.as_deref(),
            Some("Open with default application")
        );
    }

    #[test]
    fn file_open_directory_description() {
        let fi = FileInfo {
            path: "/x/dir".to_string(),
            name: "dir".to_string(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&fi);
        let a = find_action(&actions, "open_directory").unwrap();
        assert_eq!(a.description.as_deref(), Some("Open this folder"));
    }

    #[test]
    fn file_reveal_in_finder_description() {
        let fi = FileInfo {
            path: "/x/y.txt".to_string(),
            name: "y.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&fi);
        let a = find_action(&actions, "reveal_in_finder").unwrap();
        assert_eq!(a.description.as_deref(), Some("Reveal in Finder"));
    }

    #[test]
    fn file_copy_path_description() {
        let fi = FileInfo {
            path: "/x/y.txt".to_string(),
            name: "y.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&fi);
        let a = find_action(&actions, "copy_path").unwrap();
        assert_eq!(
            a.description.as_deref(),
            Some("Copy the full path to clipboard")
        );
    }

    #[test]
    fn file_copy_filename_description() {
        let fi = FileInfo {
            path: "/x/y.txt".to_string(),
            name: "y.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&fi);
        let a = find_action(&actions, "copy_filename").unwrap();
        assert_eq!(
            a.description.as_deref(),
            Some("Copy just the filename to clipboard")
        );
    }

    #[test]
    fn file_open_title_includes_name() {
        let fi = FileInfo {
            path: "/x/report.pdf".to_string(),
            name: "report.pdf".to_string(),
            file_type: FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&fi);
        let a = find_action(&actions, "open_file").unwrap();
        assert!(a.title.contains("report.pdf"));
    }

    // ========================================
    // 8. FileType variants have no effect on file actions (7 tests)
    // ========================================

    #[test]
    fn filetype_document_same_actions_as_file() {
        let a = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::Document,
            is_dir: false,
        });
        let b = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::File,
            is_dir: false,
        });
        assert_eq!(action_ids(&a), action_ids(&b));
    }

    #[test]
    fn filetype_image_same_actions() {
        let a = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::Image,
            is_dir: false,
        });
        let b = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::Other,
            is_dir: false,
        });
        assert_eq!(action_ids(&a), action_ids(&b));
    }

    #[test]
    fn filetype_audio_same_actions() {
        let a = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::Audio,
            is_dir: false,
        });
        let b = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::File,
            is_dir: false,
        });
        assert_eq!(action_ids(&a), action_ids(&b));
    }

    #[test]
    fn filetype_video_same_actions() {
        let a = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::Video,
            is_dir: false,
        });
        let b = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::File,
            is_dir: false,
        });
        assert_eq!(action_ids(&a), action_ids(&b));
    }

    #[test]
    fn filetype_application_same_actions() {
        let a = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::Application,
            is_dir: false,
        });
        let b = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::File,
            is_dir: false,
        });
        assert_eq!(action_ids(&a), action_ids(&b));
    }

    #[test]
    fn filetype_other_same_actions() {
        let a = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::Other,
            is_dir: false,
        });
        let b = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::Document,
            is_dir: false,
        });
        assert_eq!(action_ids(&a), action_ids(&b));
    }

    #[test]
    fn filetype_directory_different_from_file() {
        let a = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::Directory,
            is_dir: true,
        });
        let b = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::File,
            is_dir: false,
        });
        // is_dir changes actions
        assert_ne!(action_ids(&a), action_ids(&b));
    }

    // ========================================
    // 9. Chat model checkmark logic and ID format (6 tests)
    // ========================================

    #[test]
    fn chat_model_id_format_select_model_prefix() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "gpt-4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions[0].id.starts_with("select_model_"));
        assert_eq!(actions[0].id, "select_model_gpt-4");
    }

    #[test]
    fn chat_current_model_gets_checkmark_in_title() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".to_string()),
            available_models: vec![ChatModelInfo {
                id: "gpt-4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions[0].title.contains("✓"));
        assert_eq!(actions[0].title, "GPT-4 ✓");
    }

    #[test]
    fn chat_non_current_model_no_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".to_string()),
            available_models: vec![ChatModelInfo {
                id: "gpt-4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(!actions[0].title.contains("✓"));
        assert_eq!(actions[0].title, "GPT-4");
    }

    #[test]
    fn chat_model_description_shows_provider() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "claude-3".to_string(),
                display_name: "Claude 3".to_string(),
                provider: "Anthropic".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions[0].description.as_deref(), Some("via Anthropic"));
    }

    #[test]
    fn chat_no_models_only_continue_in_chat() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "continue_in_chat");
    }

    #[test]
    fn chat_checkmark_exact_match_only() {
        let info = ChatPromptInfo {
            current_model: Some("GPT".to_string()),
            available_models: vec![ChatModelInfo {
                id: "gpt-4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        // "GPT" != "GPT-4", so no checkmark
        assert!(!actions[0].title.contains("✓"));
    }

    // ========================================
    // 10. New chat provider_display_name propagation (5 tests)
    // ========================================

    #[test]
    fn new_chat_last_used_description_is_provider_display_name() {
        let actions = get_new_chat_actions(
            &[NewChatModelInfo {
                model_id: "m1".to_string(),
                display_name: "Model 1".to_string(),
                provider: "provider-id".to_string(),
                provider_display_name: "My Provider".to_string(),
            }],
            &[],
            &[],
        );
        assert_eq!(actions[0].description.as_deref(), Some("My Provider"));
    }


    // --- merged from tests_part_03.rs ---
    #[test]
    fn new_chat_models_section_description_is_provider_display_name() {
        let actions = get_new_chat_actions(
            &[],
            &[],
            &[NewChatModelInfo {
                model_id: "m1".to_string(),
                display_name: "M1".to_string(),
                provider: "pid".to_string(),
                provider_display_name: "Anthropic AI".to_string(),
            }],
        );
        assert_eq!(actions[0].description.as_deref(), Some("Anthropic AI"));
    }

    #[test]
    fn new_chat_presets_have_no_description() {
        let actions = get_new_chat_actions(
            &[],
            &[NewChatPresetInfo {
                id: "general".to_string(),
                name: "General".to_string(),
                icon: IconName::Settings,
            }],
            &[],
        );
        assert!(actions[0].description.is_none());
    }

    #[test]
    fn new_chat_preset_uses_its_icon() {
        let actions = get_new_chat_actions(
            &[],
            &[NewChatPresetInfo {
                id: "code".to_string(),
                name: "Code".to_string(),
                icon: IconName::Code,
            }],
            &[],
        );
        assert_eq!(actions[0].icon, Some(IconName::Code));
    }

    #[test]
    fn new_chat_mixed_sections_in_order() {
        let actions = get_new_chat_actions(
            &[NewChatModelInfo {
                model_id: "lu1".to_string(),
                display_name: "LU1".to_string(),
                provider: "p".to_string(),
                provider_display_name: "P".to_string(),
            }],
            &[NewChatPresetInfo {
                id: "gen".to_string(),
                name: "Gen".to_string(),
                icon: IconName::File,
            }],
            &[NewChatModelInfo {
                model_id: "m1".to_string(),
                display_name: "M1".to_string(),
                provider: "p".to_string(),
                provider_display_name: "P".to_string(),
            }],
        );
        assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
        assert_eq!(actions[1].section.as_deref(), Some("Presets"));
        assert_eq!(actions[2].section.as_deref(), Some("Models"));
    }

    // ========================================
    // 11. Clipboard exact description strings (8 tests)
    // ========================================

    #[test]
    fn clipboard_paste_description() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_paste").unwrap();
        assert_eq!(
            a.description.as_deref(),
            Some("Copy to clipboard and paste to focused app")
        );
    }

    #[test]
    fn clipboard_copy_description() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_copy").unwrap();
        assert_eq!(
            a.description.as_deref(),
            Some("Copy entry to clipboard without pasting")
        );
    }

    #[test]
    fn clipboard_paste_keep_open_description() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_paste_keep_open").unwrap();
        assert!(a.description.as_ref().unwrap().contains("keep"));
    }

    #[test]
    fn clipboard_pin_description() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_pin").unwrap();
        assert!(a.description.as_ref().unwrap().contains("Pin"));
    }

    #[test]
    fn clipboard_unpin_description() {
        let entry = make_clipboard_entry(ContentType::Text, true, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_unpin").unwrap();
        assert!(a.description.as_ref().unwrap().contains("pin"));
    }

    #[test]
    fn clipboard_delete_description() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_delete").unwrap();
        assert!(a.description.as_ref().unwrap().contains("Remove"));
    }

    #[test]
    fn clipboard_delete_multiple_description() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_delete_multiple").unwrap();
        assert!(a.description.as_ref().unwrap().contains("filter"));
    }

    #[test]
    fn clipboard_delete_all_description_mentions_pinned() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_delete_all").unwrap();
        assert!(a.description.as_ref().unwrap().contains("pinned"));
    }

    // ========================================
    // 12. Script context with custom verbs (5 tests)
    // ========================================

    #[test]
    fn custom_verb_launch_in_primary_title() {
        let script =
            ScriptInfo::with_action_verb("Safari", "/Applications/Safari.app", false, "Launch");
        let actions = get_script_context_actions(&script);
        let run = find_action(&actions, "run_script").unwrap();
        assert!(run.title.starts_with("Launch"));
        assert!(run.title.contains("Safari"));
    }

    #[test]
    fn custom_verb_switch_to_in_primary_title() {
        let script = ScriptInfo::with_action_verb("My Window", "window:123", false, "Switch to");
        let actions = get_script_context_actions(&script);
        let run = find_action(&actions, "run_script").unwrap();
        assert!(run.title.starts_with("Switch to"));
    }

    #[test]
    fn custom_verb_open_in_primary_title() {
        let script =
            ScriptInfo::with_action_verb("App Launcher", "builtin:launcher", false, "Open");
        let actions = get_script_context_actions(&script);
        let run = find_action(&actions, "run_script").unwrap();
        assert_eq!(run.title, "Open \"App Launcher\"");
    }

    #[test]
    fn custom_verb_execute_in_description() {
        let script = ScriptInfo::with_action_verb("Task", "/path/task.ts", true, "Execute");
        let actions = get_script_context_actions(&script);
        let run = find_action(&actions, "run_script").unwrap();
        assert!(run.description.as_ref().unwrap().contains("Execute"));
    }

    #[test]
    fn default_verb_is_run() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        assert_eq!(script.action_verb, "Run");
        let actions = get_script_context_actions(&script);
        let run = find_action(&actions, "run_script").unwrap();
        assert!(run.title.starts_with("Run"));
    }

    // ========================================
    // 13. ActionsDialogConfig defaults (5 tests)
    // ========================================

    #[test]
    fn actions_dialog_config_default_search_bottom() {
        let config = ActionsDialogConfig::default();
        assert_eq!(config.search_position, SearchPosition::Bottom);
    }

    #[test]
    fn actions_dialog_config_default_section_separators() {
        let config = ActionsDialogConfig::default();
        assert_eq!(config.section_style, SectionStyle::Separators);
    }

    #[test]
    fn actions_dialog_config_default_anchor_bottom() {
        let config = ActionsDialogConfig::default();
        assert_eq!(config.anchor, AnchorPosition::Bottom);
    }

    #[test]
    fn actions_dialog_config_default_no_icons() {
        let config = ActionsDialogConfig::default();
        assert!(!config.show_icons);
    }

    #[test]
    fn actions_dialog_config_default_no_footer() {
        let config = ActionsDialogConfig::default();
        assert!(!config.show_footer);
    }

    // ========================================
    // 14. ActionCategory PartialEq (4 tests)
    // ========================================

    #[test]
    fn action_category_eq_same() {
        assert_eq!(ActionCategory::ScriptContext, ActionCategory::ScriptContext);
        assert_eq!(ActionCategory::ScriptOps, ActionCategory::ScriptOps);
        assert_eq!(ActionCategory::GlobalOps, ActionCategory::GlobalOps);
        assert_eq!(ActionCategory::Terminal, ActionCategory::Terminal);
    }

    #[test]
    fn action_category_ne_different() {
        assert_ne!(ActionCategory::ScriptContext, ActionCategory::ScriptOps);
        assert_ne!(ActionCategory::ScriptContext, ActionCategory::GlobalOps);
        assert_ne!(ActionCategory::ScriptContext, ActionCategory::Terminal);
    }

    #[test]
    fn action_category_ne_script_ops_vs_global() {
        assert_ne!(ActionCategory::ScriptOps, ActionCategory::GlobalOps);
    }

    #[test]
    fn action_category_ne_terminal_vs_global() {
        assert_ne!(ActionCategory::Terminal, ActionCategory::GlobalOps);
    }

    // ========================================
    // 15. Agent description content keywords (5 tests)
    // ========================================

    #[test]
    fn agent_edit_description_mentions_agent_file() {
        let mut agent = ScriptInfo::new("MyAgent", "/path/agent.md");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        let a = find_action(&actions, "edit_script").unwrap();
        assert!(a.description.as_ref().unwrap().contains("agent"));
    }

    #[test]
    fn agent_reveal_description_mentions_agent() {
        let mut agent = ScriptInfo::new("MyAgent", "/path/agent.md");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        let a = find_action(&actions, "reveal_in_finder").unwrap();
        assert!(a.description.as_ref().unwrap().contains("agent"));
    }

    #[test]
    fn agent_copy_path_description_mentions_agent() {
        let mut agent = ScriptInfo::new("MyAgent", "/path/agent.md");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        let a = find_action(&actions, "copy_path").unwrap();
        assert!(a.description.as_ref().unwrap().contains("agent"));
    }

    #[test]
    fn agent_copy_content_description() {
        let mut agent = ScriptInfo::new("MyAgent", "/path/agent.md");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        let a = find_action(&actions, "copy_content").unwrap();
        assert!(a.description.as_ref().unwrap().contains("content"));
    }

    #[test]
    fn agent_edit_title_says_edit_agent() {
        let mut agent = ScriptInfo::new("MyAgent", "/path/agent.md");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        let a = find_action(&actions, "edit_script").unwrap();
        assert_eq!(a.title, "Edit Agent");
    }

    // ========================================
    // 16. Cross-context frecency reset consistency (3 tests)
    // ========================================

    #[test]
    fn frecency_reset_present_for_script() {
        let script = ScriptInfo::new("s", "/p").with_frecency(true, Some("/p".to_string()));
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    }

    #[test]
    fn frecency_reset_present_for_scriptlet() {
        let script = ScriptInfo::scriptlet("s", "/p.md", None, None)
            .with_frecency(true, Some("x".to_string()));
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    }

    #[test]
    fn frecency_reset_present_for_builtin() {
        let script = ScriptInfo::builtin("B").with_frecency(true, Some("b".to_string()));
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    }

    // ========================================
    // 17. Script context exact shortcut values (5 tests)
    // ========================================

    #[test]
    fn script_edit_shortcut_cmd_e() {
        let script = ScriptInfo::new("t", "/p/t.ts");
        let actions = get_script_context_actions(&script);
        let a = find_action(&actions, "edit_script").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘E"));
    }

    #[test]
    fn script_view_logs_shortcut_cmd_l() {
        let script = ScriptInfo::new("t", "/p/t.ts");
        let actions = get_script_context_actions(&script);
        let a = find_action(&actions, "view_logs").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘L"));
    }

    #[test]
    fn script_reveal_shortcut_cmd_shift_f() {
        let script = ScriptInfo::new("t", "/p/t.ts");
        let actions = get_script_context_actions(&script);
        let a = find_action(&actions, "reveal_in_finder").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘⇧F"));
    }

    #[test]
    fn script_copy_path_shortcut_cmd_shift_c() {
        let script = ScriptInfo::new("t", "/p/t.ts");
        let actions = get_script_context_actions(&script);
        let a = find_action(&actions, "copy_path").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘⇧C"));
    }

    #[test]
    fn script_copy_content_shortcut_cmd_opt_c() {
        let script = ScriptInfo::new("t", "/p/t.ts");
        let actions = get_script_context_actions(&script);
        let a = find_action(&actions, "copy_content").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘⌥C"));
    }

    // ========================================
    // 18. CommandBarConfig factory methods (5 tests)
    // ========================================

    #[test]
    fn command_bar_ai_style_search_top_headers_icons_footer() {
        let c = CommandBarConfig::ai_style();
        assert_eq!(c.dialog_config.search_position, SearchPosition::Top);
        assert_eq!(c.dialog_config.section_style, SectionStyle::Headers);
        assert!(c.dialog_config.show_icons);
        assert!(c.dialog_config.show_footer);
    }

    #[test]
    fn command_bar_main_menu_search_bottom_separators() {
        let c = CommandBarConfig::main_menu_style();
        assert_eq!(c.dialog_config.search_position, SearchPosition::Bottom);
        assert_eq!(c.dialog_config.section_style, SectionStyle::Separators);
        assert!(!c.dialog_config.show_icons);
        assert!(!c.dialog_config.show_footer);
    }

    #[test]
    fn command_bar_no_search_hidden() {
        let c = CommandBarConfig::no_search();
        assert_eq!(c.dialog_config.search_position, SearchPosition::Hidden);
    }

    #[test]
    fn command_bar_notes_style_search_top_separators_icons_footer() {
        let c = CommandBarConfig::notes_style();
        assert_eq!(c.dialog_config.search_position, SearchPosition::Top);
        assert_eq!(c.dialog_config.section_style, SectionStyle::Separators);
        assert!(c.dialog_config.show_icons);
        assert!(c.dialog_config.show_footer);
    }

    #[test]
    fn command_bar_default_close_flags_all_true() {
        let c = CommandBarConfig::default();
        assert!(c.close_on_select);
        assert!(c.close_on_click_outside);
        assert!(c.close_on_escape);
    }

    // ========================================
    // 19. Notes command bar exact shortcuts (6 tests)
    // ========================================

    #[test]
    fn notes_cmd_bar_new_note_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "new_note").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘N"));
    }


    // --- merged from tests_part_04.rs ---
    #[test]
    fn notes_cmd_bar_duplicate_note_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "duplicate_note").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘D"));
    }

    #[test]
    fn notes_cmd_bar_browse_notes_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "browse_notes").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘P"));
    }

    #[test]
    fn notes_cmd_bar_find_in_note_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "find_in_note").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘F"));
    }

    #[test]
    fn notes_cmd_bar_copy_note_as_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "copy_note_as").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⇧⌘C"));
    }

    #[test]
    fn notes_cmd_bar_export_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "export").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⇧⌘E"));
    }

    // ========================================
    // 20. Note switcher complex scenarios (5 tests)
    // ========================================

    #[test]
    fn note_switcher_10_notes_all_have_actions() {
        let notes: Vec<_> = (0..10)
            .map(|i| {
                make_note(
                    &format!("id-{}", i),
                    &format!("Note {}", i),
                    100,
                    false,
                    false,
                    "",
                    "",
                )
            })
            .collect();
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions.len(), 10);
    }

    #[test]
    fn note_switcher_pinned_section_label() {
        let note = make_note("1", "Pinned Note", 50, false, true, "", "");
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
    }

    #[test]
    fn note_switcher_recent_section_label() {
        let note = make_note("1", "Regular Note", 50, false, false, "", "");
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn note_switcher_mixed_pinned_and_recent_sections() {
        let notes = vec![
            make_note("1", "A", 10, false, true, "", ""),
            make_note("2", "B", 20, false, false, "", ""),
            make_note("3", "C", 30, false, true, "", ""),
        ];
        let actions = get_note_switcher_actions(&notes);
        let sections: Vec<_> = actions.iter().map(|a| a.section.as_deref()).collect();
        assert_eq!(
            sections,
            vec![Some("Pinned"), Some("Recent"), Some("Pinned")]
        );
    }

    #[test]
    fn note_switcher_current_note_has_bullet_prefix() {
        let note = make_note("1", "Current Note", 50, true, false, "", "");
        let actions = get_note_switcher_actions(&[note]);
        assert!(actions[0].title.starts_with("• "));
    }

    // ========================================
    // 21. build_grouped_items section header content (5 tests)
    // ========================================

    #[test]
    fn grouped_items_header_text_matches_section_name() {
        let actions = vec![
            make_action("a1", "Action 1", Some("Section A")),
            make_action("a2", "Action 2", Some("Section B")),
        ];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        match &grouped[0] {
            GroupedActionItem::SectionHeader(s) => assert_eq!(s, "Section A"),
            _ => panic!("Expected SectionHeader"),
        }
        match &grouped[2] {
            GroupedActionItem::SectionHeader(s) => assert_eq!(s, "Section B"),
            _ => panic!("Expected SectionHeader"),
        }
    }

    #[test]
    fn grouped_items_headers_count_matches_unique_sections() {
        let actions = vec![
            make_action("a1", "A1", Some("S1")),
            make_action("a2", "A2", Some("S1")),
            make_action("a3", "A3", Some("S2")),
            make_action("a4", "A4", Some("S3")),
        ];
        let filtered = vec![0, 1, 2, 3];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 3);
    }

    #[test]
    fn grouped_items_no_section_no_header() {
        let actions = vec![make_action("a1", "A1", None), make_action("a2", "A2", None)];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 0);
    }

    #[test]
    fn grouped_items_headers_precede_their_items() {
        let actions = vec![
            make_action("a1", "A1", Some("First")),
            make_action("a2", "A2", Some("Second")),
        ];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Expected: [Header("First"), Item(0), Header("Second"), Item(1)]
        assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(_)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[2], GroupedActionItem::SectionHeader(_)));
        assert!(matches!(grouped[3], GroupedActionItem::Item(1)));
    }

    #[test]
    fn grouped_items_separators_style_no_headers() {
        let actions = vec![
            make_action("a1", "A1", Some("S1")),
            make_action("a2", "A2", Some("S2")),
        ];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 0);
        assert_eq!(grouped.len(), 2); // Just Items
    }

    // ========================================
    // 22. coerce_action_selection edge cases (5 tests)
    // ========================================

    #[test]
    fn coerce_empty_returns_none() {
        assert_eq!(coerce_action_selection(&[], 0), None);
    }

    #[test]
    fn coerce_on_item_returns_same() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    }

    #[test]
    fn coerce_header_followed_by_item_goes_down() {
        let rows = vec![
            GroupedActionItem::SectionHeader("S".to_string()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn coerce_item_then_header_at_end() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("S".to_string()),
        ];
        // Index 1 is header, search down finds nothing, search up finds Item at 0
        assert_eq!(coerce_action_selection(&rows, 1), Some(0));
    }

    #[test]
    fn coerce_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".to_string()),
            GroupedActionItem::SectionHeader("B".to_string()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    // ========================================
    // 23. Score_action with cached lowercase (5 tests)
    // ========================================

    #[test]
    fn score_prefix_match_100() {
        let a = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
        assert!(ActionsDialog::score_action(&a, "edit") >= 100);
    }

    #[test]
    fn score_contains_match_50() {
        let a = Action::new("id", "Quick Edit", None, ActionCategory::ScriptContext);
        let s = ActionsDialog::score_action(&a, "edit");
        assert!((50..100).contains(&s));
    }

    #[test]
    fn score_fuzzy_match_25() {
        // "et" subsequence in "edit" => e...t
        let a = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
        let s = ActionsDialog::score_action(&a, "eit");
        // 'e','i','t' are subsequence of "edit script"
        assert!(s >= 25);
    }

    #[test]
    fn score_description_bonus_15() {
        let a = Action::new(
            "id",
            "Open File",
            Some("Launch editor".to_string()),
            ActionCategory::ScriptContext,
        );
        let s = ActionsDialog::score_action(&a, "editor");
        assert!(s >= 15);
    }

    #[test]
    fn score_no_match_zero() {
        let a = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
        assert_eq!(ActionsDialog::score_action(&a, "zzzzz"), 0);
    }

    // ========================================
    // 24. fuzzy_match edge cases (5 tests)
    // ========================================

    #[test]
    fn fuzzy_empty_needle_always_matches() {
        assert!(ActionsDialog::fuzzy_match("anything", ""));
    }

    #[test]
    fn fuzzy_empty_haystack_nonempty_needle_fails() {
        assert!(!ActionsDialog::fuzzy_match("", "a"));
    }

    #[test]
    fn fuzzy_exact_match() {
        assert!(ActionsDialog::fuzzy_match("hello", "hello"));
    }

    #[test]
    fn fuzzy_subsequence_match() {
        assert!(ActionsDialog::fuzzy_match("abcdef", "ace"));
    }

    #[test]
    fn fuzzy_no_match() {
        assert!(!ActionsDialog::fuzzy_match("abc", "z"));
    }

    // ========================================
    // 25. parse_shortcut_keycaps (6 tests)
    // ========================================

    #[test]
    fn keycaps_cmd_c() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘C");
        assert_eq!(caps, vec!["⌘", "C"]);
    }

    #[test]
    fn keycaps_cmd_shift_enter() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘⇧↵");
        assert_eq!(caps, vec!["⌘", "⇧", "↵"]);
    }

    #[test]
    fn keycaps_ctrl_x() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌃X");
        assert_eq!(caps, vec!["⌃", "X"]);
    }

    #[test]
    fn keycaps_space() {
        let caps = ActionsDialog::parse_shortcut_keycaps("␣");
        assert_eq!(caps, vec!["␣"]);
    }

    #[test]
    fn keycaps_arrows() {
        let caps = ActionsDialog::parse_shortcut_keycaps("↑↓←→");
        assert_eq!(caps, vec!["↑", "↓", "←", "→"]);
    }

    #[test]
    fn keycaps_lowercase_uppercased() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘c");
        assert_eq!(caps, vec!["⌘", "C"]);
    }

    // ========================================
    // 26. to_deeplink_name additional edge cases (4 tests)
    // ========================================

    #[test]
    fn deeplink_cjk_characters_preserved() {
        let result = to_deeplink_name("测试脚本");
        assert_eq!(result, "测试脚本");
    }

    #[test]
    fn deeplink_mixed_ascii_unicode() {
        let result = to_deeplink_name("My 脚本 Script");
        assert_eq!(result, "my-脚本-script");
    }

    #[test]
    fn deeplink_accented_preserved() {
        let result = to_deeplink_name("café résumé");
        assert_eq!(result, "café-résumé");
    }

    #[test]
    fn deeplink_emoji_stripped() {
        // Emoji are not alphanumeric, so they become hyphens
        let result = to_deeplink_name("Test 🚀 Script");
        // 🚀 becomes -, collapses with surrounding hyphens
        assert_eq!(result, "test-script");
    }

    // ========================================
    // 27. Clipboard exact shortcut values (6 tests)
    // ========================================

    #[test]
    fn clipboard_share_shortcut() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_share").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⇧⌘E"));
    }

    #[test]
    fn clipboard_attach_to_ai_shortcut() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_attach_to_ai").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌃⌘A"));
    }

    #[test]
    fn clipboard_pin_shortcut() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_pin").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⇧⌘P"));
    }

    #[test]
    fn clipboard_unpin_shortcut_same_as_pin() {
        let entry = make_clipboard_entry(ContentType::Text, true, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_unpin").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⇧⌘P"));
    }

    #[test]
    fn clipboard_save_snippet_shortcut() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_save_snippet").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⇧⌘S"));
    }

    #[test]
    fn clipboard_save_file_shortcut() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_save_file").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌥⇧⌘S"));
    }

    // ========================================
    // 28. Notes command bar section labels (4 tests)
    // ========================================

    // --- merged from tests_part_05.rs ---
    #[test]
    fn notes_cmd_bar_new_note_section_notes() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "new_note").unwrap();
        assert_eq!(a.section.as_deref(), Some("Notes"));
    }

    #[test]
    fn notes_cmd_bar_find_in_note_section_edit() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "find_in_note").unwrap();
        assert_eq!(a.section.as_deref(), Some("Edit"));
    }

    #[test]
    fn notes_cmd_bar_copy_note_as_section_copy() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "copy_note_as").unwrap();
        assert_eq!(a.section.as_deref(), Some("Copy"));
    }

    #[test]
    fn notes_cmd_bar_export_section_export() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "export").unwrap();
        assert_eq!(a.section.as_deref(), Some("Export"));
    }

    // ========================================
    // 29. ID uniqueness and non-empty invariants (6 tests)
    // ========================================

    #[test]
    fn notes_cmd_bar_ids_unique() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids: HashSet<_> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn chat_context_ids_unique() {
        let info = ChatPromptInfo {
            current_model: Some("M1".to_string()),
            available_models: vec![
                ChatModelInfo {
                    id: "m1".to_string(),
                    display_name: "M1".to_string(),
                    provider: "P1".to_string(),
                },
                ChatModelInfo {
                    id: "m2".to_string(),
                    display_name: "M2".to_string(),
                    provider: "P2".to_string(),
                },
            ],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        let ids: HashSet<_> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn new_chat_ids_unique() {
        let actions = get_new_chat_actions(
            &[NewChatModelInfo {
                model_id: "l1".to_string(),
                display_name: "L1".to_string(),
                provider: "p".to_string(),
                provider_display_name: "P".to_string(),
            }],
            &[NewChatPresetInfo {
                id: "gen".to_string(),
                name: "Gen".to_string(),
                icon: IconName::File,
            }],
            &[NewChatModelInfo {
                model_id: "m1".to_string(),
                display_name: "M1".to_string(),
                provider: "p".to_string(),
                provider_display_name: "P".to_string(),
            }],
        );
        let ids: HashSet<_> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn note_switcher_ids_unique() {
        let notes = vec![
            make_note("uuid-1", "Note 1", 10, false, false, "", ""),
            make_note("uuid-2", "Note 2", 20, true, false, "", ""),
            make_note("uuid-3", "Note 3", 30, false, true, "", ""),
        ];
        let actions = get_note_switcher_actions(&notes);
        let ids: HashSet<_> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn all_note_switcher_actions_nonempty_title() {
        let notes = vec![
            make_note("1", "A", 1, false, false, "", ""),
            make_note("2", "B", 2, true, true, "preview", "1m ago"),
        ];
        let actions = get_note_switcher_actions(&notes);
        for a in &actions {
            assert!(!a.title.is_empty(), "Action {} has empty title", a.id);
            assert!(!a.id.is_empty(), "Action has empty id");
        }
    }

    #[test]
    fn all_path_actions_nonempty_title_and_id() {
        let info = PathInfo {
            name: "test".to_string(),
            path: "/test".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        for a in &actions {
            assert!(!a.title.is_empty());
            assert!(!a.id.is_empty());
        }
    }

    // ========================================
    // 30. Ordering determinism (4 tests)
    // ========================================

    #[test]
    fn notes_cmd_bar_ordering_deterministic() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let a = get_notes_command_bar_actions(&info);
        let b = get_notes_command_bar_actions(&info);
        assert_eq!(action_ids(&a), action_ids(&b));
    }

    #[test]
    fn chat_context_ordering_deterministic() {
        let info = ChatPromptInfo {
            current_model: Some("X".to_string()),
            available_models: vec![
                ChatModelInfo {
                    id: "a".to_string(),
                    display_name: "A".to_string(),
                    provider: "P".to_string(),
                },
                ChatModelInfo {
                    id: "b".to_string(),
                    display_name: "B".to_string(),
                    provider: "P".to_string(),
                },
            ],
            has_messages: true,
            has_response: true,
        };
        let a = get_chat_context_actions(&info);
        let b = get_chat_context_actions(&info);
        assert_eq!(action_ids(&a), action_ids(&b));
    }

    #[test]
    fn new_chat_ordering_deterministic() {
        let last = vec![NewChatModelInfo {
            model_id: "l".to_string(),
            display_name: "L".to_string(),
            provider: "p".to_string(),
            provider_display_name: "P".to_string(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "g".to_string(),
            name: "G".to_string(),
            icon: IconName::File,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m".to_string(),
            display_name: "M".to_string(),
            provider: "p".to_string(),
            provider_display_name: "P".to_string(),
        }];
        let a = get_new_chat_actions(&last, &presets, &models);
        let b = get_new_chat_actions(&last, &presets, &models);
        assert_eq!(action_ids(&a), action_ids(&b));
    }

    #[test]
    fn path_context_ordering_deterministic() {
        let info = PathInfo {
            name: "f".to_string(),
            path: "/f".to_string(),
            is_dir: false,
        };
        let a = get_path_context_actions(&info);
        let b = get_path_context_actions(&info);
        assert_eq!(action_ids(&a), action_ids(&b));
    }

}
