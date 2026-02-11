// Batch 40: Dialog builtin action validation tests
//
// Focuses on:
// - ScriptInfo::with_action_verb_and_shortcut: field validation
// - ScriptInfo: is_agent manual override after construction
// - Action::with_shortcut_opt: Some vs None behavior
// - Action::with_icon and with_section chaining
// - Clipboard: text entry total action count on macOS
// - Clipboard: image entry total action count on macOS
// - Clipboard: pinned text entry has clipboard_unpin ID
// - Clipboard: unpinned text entry has clipboard_pin ID
// - File context: dir has no quick_look but has open_with on macOS
// - File context: file primary title format uses quoted name
// - Path context: all action IDs are snake_case
// - Path context: open_in_editor desc mentions $EDITOR
// - Script context: scriptlet is_scriptlet true has edit_scriptlet
// - Script context: builtin has exactly 4 actions when no shortcut/alias
// - Script context: primary title uses action_verb
// - Scriptlet context: with_custom run_script is first action
// - Scriptlet context: with_custom copy_deeplink URL uses to_deeplink_name
// - AI bar: toggle_shortcuts_help details
// - AI bar: change_model has no shortcut
// - AI bar: unique IDs across all 12 actions
// - Notes: export action requires selection and not trash
// - Notes: browse_notes always present
// - Chat context: copy_response only when has_response
// - Chat context: clear_conversation only when has_messages
// - New chat: empty lists produce zero actions
// - New chat: preset IDs use preset_{id} format
// - Note switcher: empty notes produces no_notes action
// - Note switcher: preview with relative_time has separator
// - ProtocolAction: with_value sets value and has_action false
// - format_shortcut_hint: dialog vs builders produce different results

#[cfg(test)]
mod tests {
    // --- merged from tests_part_01.rs ---
    use crate::actions::builders::*;
    use crate::actions::dialog::ActionsDialog;
    use crate::actions::types::{Action, ActionCategory, ScriptInfo};
    use crate::clipboard_history::ContentType;
    use crate::designs::icon_variations::IconName;
    use crate::file_search::{FileInfo, FileType};
    use crate::prompts::PathInfo;
    use crate::protocol::ProtocolAction;
    use crate::scriptlets::Scriptlet;

    // =========================================================================
    // 1. ScriptInfo::with_action_verb_and_shortcut: field validation
    // =========================================================================

    #[test]
    fn with_action_verb_and_shortcut_preserves_name_path() {
        let info = ScriptInfo::with_action_verb_and_shortcut(
            "launcher",
            "/scripts/launcher.ts",
            true,
            "Launch",
            Some("cmd+l".into()),
        );
        assert_eq!(info.name, "launcher");
        assert_eq!(info.path, "/scripts/launcher.ts");
    }

    #[test]
    fn with_action_verb_and_shortcut_sets_verb() {
        let info = ScriptInfo::with_action_verb_and_shortcut("x", "/x", true, "Open", None);
        assert_eq!(info.action_verb, "Open");
    }

    #[test]
    fn with_action_verb_and_shortcut_sets_shortcut() {
        let info = ScriptInfo::with_action_verb_and_shortcut(
            "x",
            "/x",
            false,
            "Run",
            Some("cmd+shift+r".into()),
        );
        assert_eq!(info.shortcut, Some("cmd+shift+r".to_string()));
        assert!(!info.is_script);
    }

    #[test]
    fn with_action_verb_and_shortcut_defaults_agent_false() {
        let info =
            ScriptInfo::with_action_verb_and_shortcut("x", "/x", true, "Run", Some("a".into()));
        assert!(!info.is_agent);
        assert!(!info.is_scriptlet);
        assert!(!info.is_suggested);
    }

    // =========================================================================
    // 2. ScriptInfo: is_agent manual override after construction
    // =========================================================================

    #[test]
    fn script_info_set_is_agent_true_after_new() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_agent = true;
        assert!(info.is_agent);
        assert!(info.is_script); // new() sets is_script=true
    }

    #[test]
    fn agent_actions_include_edit_agent_title() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_agent = true;
        info.is_script = false; // agents have is_script=false
        let actions = get_script_context_actions(&info);
        let edit = actions.iter().find(|a| a.id == "edit_script");
        assert!(edit.is_some());
        assert_eq!(edit.unwrap().title, "Edit Agent");
    }

    #[test]
    fn agent_actions_have_copy_content() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_agent = true;
        info.is_script = false;
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "copy_content"));
    }

    #[test]
    fn agent_actions_desc_mentions_agent() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_agent = true;
        info.is_script = false;
        let actions = get_script_context_actions(&info);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert!(edit
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("agent"));
    }

    // =========================================================================
    // 3. Action::with_shortcut_opt: Some vs None behavior
    // =========================================================================

    #[test]
    fn with_shortcut_opt_some_sets_shortcut() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘T".to_string()));
        assert_eq!(action.shortcut, Some("⌘T".to_string()));
    }

    #[test]
    fn with_shortcut_opt_none_leaves_none() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(None);
        assert!(action.shortcut.is_none());
        assert!(action.shortcut_lower.is_none());
    }

    #[test]
    fn with_shortcut_opt_some_sets_shortcut_lower() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘ABC".to_string()));
        assert_eq!(action.shortcut_lower, Some("⌘abc".to_string()));
    }

    #[test]
    fn with_shortcut_opt_overwrites_previous_shortcut() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘X")
            .with_shortcut_opt(Some("⌘Y".to_string()));
        assert_eq!(action.shortcut, Some("⌘Y".to_string()));
    }

    // =========================================================================
    // 4. Action::with_icon and with_section chaining
    // =========================================================================

    #[test]
    fn with_icon_sets_icon() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_icon(IconName::Copy);
        assert_eq!(action.icon, Some(IconName::Copy));
    }

    #[test]
    fn with_section_sets_section() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_section("MySection");
        assert_eq!(action.section, Some("MySection".to_string()));
    }

    #[test]
    fn chained_icon_section_shortcut_preserves_all() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘T")
            .with_icon(IconName::Star)
            .with_section("Actions");
        assert_eq!(action.shortcut, Some("⌘T".to_string()));
        assert_eq!(action.icon, Some(IconName::Star));
        assert_eq!(action.section, Some("Actions".to_string()));
    }

    #[test]
    fn action_new_defaults_icon_section_none() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
        assert!(action.icon.is_none());
        assert!(action.section.is_none());
        assert!(action.value.is_none());
    }

    // =========================================================================
    // 5. Clipboard: text entry total action count on macOS
    // =========================================================================

    #[test]
    fn clipboard_text_unpinned_action_count_macos() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        // On macOS: paste, copy, paste_keep_open, share, attach_to_ai, quick_look,
        //           pin, save_snippet, save_file, delete, delete_multiple, delete_all = 12
        #[cfg(target_os = "macos")]
        assert_eq!(actions.len(), 12);
    }

    #[test]
    fn clipboard_text_pinned_same_count_as_unpinned() {
        let unpinned = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "a".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let pinned = ClipboardEntryInfo {
            id: "2".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "b".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let unpinned_actions = get_clipboard_history_context_actions(&unpinned);
        let pinned_actions = get_clipboard_history_context_actions(&pinned);
        assert_eq!(unpinned_actions.len(), pinned_actions.len());
    }

    #[test]
    fn clipboard_text_first_action_is_paste() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[0].id, "clipboard_paste");
    }

    #[test]
    fn clipboard_text_second_action_is_copy() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[1].id, "clipboard_copy");
    }

    // =========================================================================
    // 6. Clipboard: image entry total action count on macOS
    // =========================================================================

    #[test]
    fn clipboard_image_unpinned_action_count_macos() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        // On macOS image: paste, copy, paste_keep_open, share, attach_to_ai, quick_look,
        //   open_with, annotate_cleanshot, upload_cleanshot, pin, ocr,
        //   save_snippet, save_file, delete, delete_multiple, delete_all = 16
        #[cfg(target_os = "macos")]
        assert_eq!(actions.len(), 16);
    }

    #[test]
    fn clipboard_image_has_4_more_than_text() {
        let text_entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let image_entry = ClipboardEntryInfo {
            id: "2".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let text_count = get_clipboard_history_context_actions(&text_entry).len();
        let image_count = get_clipboard_history_context_actions(&image_entry).len();
        #[cfg(target_os = "macos")]
        assert_eq!(image_count - text_count, 4);
    }

    #[test]
    fn clipboard_image_has_ocr_action() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_ocr"));
    }

    #[test]
    fn clipboard_text_has_no_ocr_action() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(!actions.iter().any(|a| a.id == "clipboard_ocr"));
    }

    // =========================================================================
    // 7. Clipboard: pinned text entry has clipboard_unpin ID
    // =========================================================================

    #[test]
    fn clipboard_pinned_has_unpin_action() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_unpin"));
        assert!(!actions.iter().any(|a| a.id == "clipboard_pin"));
    }

    #[test]
    fn clipboard_pinned_unpin_title() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let unpin = actions.iter().find(|a| a.id == "clipboard_unpin").unwrap();
        assert_eq!(unpin.title, "Unpin Entry");
    }

    #[test]
    fn clipboard_pinned_unpin_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let unpin = actions.iter().find(|a| a.id == "clipboard_unpin").unwrap();
        assert_eq!(unpin.shortcut, Some("⇧⌘P".to_string()));
    }

    #[test]
    fn clipboard_pinned_has_no_pin_action() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(!actions.iter().any(|a| a.id == "clipboard_pin"));
    }

    // =========================================================================
    // 8. Clipboard: unpinned text entry has clipboard_pin ID
    // =========================================================================

    #[test]
    fn clipboard_unpinned_has_pin_action() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_pin"));
        assert!(!actions.iter().any(|a| a.id == "clipboard_unpin"));
    }

    #[test]
    fn clipboard_unpinned_pin_title() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let pin = actions.iter().find(|a| a.id == "clipboard_pin").unwrap();
        assert_eq!(pin.title, "Pin Entry");
    }

    #[test]
    fn clipboard_unpinned_pin_desc_mentions_prevent() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let pin = actions.iter().find(|a| a.id == "clipboard_pin").unwrap();
        assert!(pin.description.as_ref().unwrap().contains("Pin"));
    }


    // --- merged from tests_part_02.rs ---
    #[test]
    fn clipboard_pin_and_unpin_same_shortcut() {
        let pinned_entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "a".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let unpinned_entry = ClipboardEntryInfo {
            id: "2".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "b".into(),
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
    }

    // =========================================================================
    // 9. File context: dir has no quick_look but has open_with on macOS
    // =========================================================================

    #[test]
    fn file_dir_has_no_quick_look() {
        let file_info = FileInfo {
            name: "Documents".into(),
            path: "/Users/test/Documents".into(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        let actions = get_file_context_actions(&file_info);
        assert!(!actions.iter().any(|a| a.id == "quick_look"));
    }

    #[test]
    fn file_file_has_quick_look_on_macos() {
        let file_info = FileInfo {
            name: "readme.md".into(),
            path: "/Users/test/readme.md".into(),
            is_dir: false,
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&file_info);
        #[cfg(target_os = "macos")]
        assert!(actions.iter().any(|a| a.id == "quick_look"));
    }

    #[test]
    fn file_dir_has_open_with_on_macos() {
        let file_info = FileInfo {
            name: "Documents".into(),
            path: "/Users/test/Documents".into(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        let actions = get_file_context_actions(&file_info);
        #[cfg(target_os = "macos")]
        assert!(actions.iter().any(|a| a.id == "open_with"));
    }

    #[test]
    fn file_dir_has_show_info_on_macos() {
        let file_info = FileInfo {
            name: "Documents".into(),
            path: "/Users/test/Documents".into(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        let actions = get_file_context_actions(&file_info);
        #[cfg(target_os = "macos")]
        assert!(actions.iter().any(|a| a.id == "show_info"));
    }

    // =========================================================================
    // 10. File context: file primary title format uses quoted name
    // =========================================================================

    #[test]
    fn file_primary_title_quotes_filename() {
        let file_info = FileInfo {
            name: "report.pdf".into(),
            path: "/Users/test/report.pdf".into(),
            is_dir: false,
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&file_info);
        assert_eq!(actions[0].title, "Open \"report.pdf\"");
    }

    #[test]
    fn file_dir_primary_title_quotes_dirname() {
        let file_info = FileInfo {
            name: "src".into(),
            path: "/Users/test/src".into(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        let actions = get_file_context_actions(&file_info);
        assert_eq!(actions[0].title, "Open \"src\"");
    }

    #[test]
    fn file_primary_id_is_open_file_for_files() {
        let file_info = FileInfo {
            name: "test.txt".into(),
            path: "/test.txt".into(),
            is_dir: false,
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&file_info);
        assert_eq!(actions[0].id, "open_file");
    }

    #[test]
    fn file_primary_id_is_open_directory_for_dirs() {
        let file_info = FileInfo {
            name: "docs".into(),
            path: "/docs".into(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        let actions = get_file_context_actions(&file_info);
        assert_eq!(actions[0].id, "open_directory");
    }

    // =========================================================================
    // 11. Path context: all action IDs are snake_case
    // =========================================================================

    #[test]
    fn path_file_all_ids_snake_case() {
        let path_info = PathInfo {
            name: "test.txt".into(),
            path: "/test.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        for action in &actions {
            assert!(
                action.id.chars().all(|c| c.is_lowercase() || c == '_'),
                "Action ID '{}' is not snake_case",
                action.id
            );
        }
    }

    #[test]
    fn path_dir_all_ids_snake_case() {
        let path_info = PathInfo {
            name: "docs".into(),
            path: "/docs".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path_info);
        for action in &actions {
            assert!(
                action.id.chars().all(|c| c.is_lowercase() || c == '_'),
                "Action ID '{}' is not snake_case",
                action.id
            );
        }
    }

    #[test]
    fn path_file_has_seven_actions() {
        let path_info = PathInfo {
            name: "test.txt".into(),
            path: "/test.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        assert_eq!(actions.len(), 7);
    }

    #[test]
    fn path_dir_has_seven_actions() {
        let path_info = PathInfo {
            name: "docs".into(),
            path: "/docs".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path_info);
        assert_eq!(actions.len(), 7);
    }

    // =========================================================================
    // 12. Path context: open_in_editor desc mentions $EDITOR
    // =========================================================================

    #[test]
    fn path_open_in_editor_desc_mentions_editor() {
        let path_info = PathInfo {
            name: "test.txt".into(),
            path: "/test.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        let editor_action = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
        assert!(editor_action
            .description
            .as_ref()
            .unwrap()
            .contains("$EDITOR"));
    }

    #[test]
    fn path_open_in_editor_shortcut() {
        let path_info = PathInfo {
            name: "test.txt".into(),
            path: "/test.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        let editor_action = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
        assert_eq!(editor_action.shortcut, Some("⌘E".to_string()));
    }

    #[test]
    fn path_open_in_finder_shortcut() {
        let path_info = PathInfo {
            name: "test.txt".into(),
            path: "/test.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        let finder_action = actions.iter().find(|a| a.id == "open_in_finder").unwrap();
        assert_eq!(finder_action.shortcut, Some("⌘⇧F".to_string()));
    }

    #[test]
    fn path_move_to_trash_shortcut() {
        let path_info = PathInfo {
            name: "test.txt".into(),
            path: "/test.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        let trash_action = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
        assert_eq!(trash_action.shortcut, Some("⌘⌫".to_string()));
    }

    // =========================================================================
    // 13. Script context: scriptlet is_scriptlet true has edit_scriptlet
    // =========================================================================

    #[test]
    fn script_context_scriptlet_has_edit_scriptlet() {
        let info = ScriptInfo::scriptlet("My Snippet", "/path/to/snippets.md", None, None);
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "edit_scriptlet"));
    }

    #[test]
    fn script_context_scriptlet_no_edit_script() {
        let info = ScriptInfo::scriptlet("My Snippet", "/path/to/snippets.md", None, None);
        let actions = get_script_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "edit_script"));
    }

    #[test]
    fn script_context_scriptlet_has_reveal_scriptlet_in_finder() {
        let info = ScriptInfo::scriptlet("My Snippet", "/path/to/snippets.md", None, None);
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "reveal_scriptlet_in_finder"));
    }

    #[test]
    fn script_context_scriptlet_no_view_logs() {
        let info = ScriptInfo::scriptlet("My Snippet", "/path/to/snippets.md", None, None);
        let actions = get_script_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    // =========================================================================
    // 14. Script context: builtin has exactly 4 actions when no shortcut/alias
    // =========================================================================

    #[test]
    fn builtin_no_shortcut_no_alias_has_4_actions() {
        let info = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&info);
        // run_script, add_shortcut, add_alias, copy_deeplink = 4
        assert_eq!(actions.len(), 4);
    }

    #[test]
    fn builtin_action_ids() {
        let info = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&info);
        let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains(&"run_script"));
        assert!(ids.contains(&"add_shortcut"));
        assert!(ids.contains(&"add_alias"));
        assert!(ids.contains(&"copy_deeplink"));
    }

    #[test]
    fn builtin_no_edit_no_reveal_no_copy_path() {
        let info = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "edit_script"));
        assert!(!actions.iter().any(|a| a.id == "reveal_in_finder"));
        assert!(!actions.iter().any(|a| a.id == "copy_path"));
    }

    #[test]
    fn builtin_no_view_logs() {
        let info = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    // =========================================================================
    // 15. Script context: primary title uses action_verb
    // =========================================================================

    #[test]
    fn script_primary_title_uses_run_verb() {
        let info = ScriptInfo::new("my-script", "/path/to/my-script.ts");
        let actions = get_script_context_actions(&info);
        assert_eq!(actions[0].title, "Run \"my-script\"");
    }

    #[test]
    fn script_primary_title_uses_custom_verb() {
        let info = ScriptInfo::with_action_verb("launcher", "/path", true, "Launch");
        let actions = get_script_context_actions(&info);
        assert_eq!(actions[0].title, "Launch \"launcher\"");
    }

    #[test]
    fn script_primary_desc_uses_verb() {
        let info = ScriptInfo::with_action_verb("app", "/path", false, "Open");
        let actions = get_script_context_actions(&info);
        assert!(actions[0].description.as_ref().unwrap().contains("Open"));
    }

    #[test]
    fn script_primary_shortcut_is_enter() {
        let info = ScriptInfo::new("test", "/test");
        let actions = get_script_context_actions(&info);
        assert_eq!(actions[0].shortcut, Some("↵".to_string()));
    }

    // =========================================================================
    // 16. Scriptlet context: with_custom run_script is first action
    // =========================================================================

    #[test]
    fn scriptlet_with_custom_first_is_run_script() {
        let script = ScriptInfo::scriptlet("snippet", "/path/snippet.md", None, None);
        let scriptlet = Scriptlet::new(
            "snippet".to_string(),
            "bash".to_string(),
            "echo hello".to_string(),
        );
        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        assert_eq!(actions[0].id, "run_script");
    }

    #[test]
    fn scriptlet_with_custom_run_title_includes_name() {
        let script = ScriptInfo::scriptlet("My Snippet", "/path/snippet.md", None, None);
        let scriptlet = Scriptlet::new(
            "My Snippet".to_string(),
            "bash".to_string(),
            "echo hello".to_string(),
        );
        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        assert!(actions[0].title.contains("My Snippet"));
    }

    #[test]
    fn scriptlet_with_custom_run_shortcut_enter() {
        let script = ScriptInfo::scriptlet("snippet", "/path/snippet.md", None, None);
        let scriptlet = Scriptlet::new(
            "snippet".to_string(),
            "bash".to_string(),
            "echo hello".to_string(),
        );
        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        assert_eq!(actions[0].shortcut, Some("↵".to_string()));
    }

    #[test]
    fn scriptlet_with_custom_none_scriptlet_has_no_custom_actions() {
        let script = ScriptInfo::scriptlet("snippet", "/path/snippet.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        // Should not have any scriptlet_action: prefixed actions
        assert!(!actions
            .iter()
            .any(|a| a.id.starts_with("scriptlet_action:")));
    }

    // =========================================================================
    // 17. Scriptlet context: with_custom copy_deeplink URL uses to_deeplink_name
    // =========================================================================

    #[test]
    fn scriptlet_copy_deeplink_uses_deeplink_name() {
        let script = ScriptInfo::scriptlet("Open GitHub", "/path/snippet.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let deeplink = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert!(deeplink
            .description
            .as_ref()
            .unwrap()
            .contains("open-github"));
    }

    #[test]
    fn scriptlet_copy_deeplink_shortcut() {
        let script = ScriptInfo::scriptlet("Test", "/path", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let deeplink = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert_eq!(deeplink.shortcut, Some("⌘⇧D".to_string()));
    }

    #[test]
    fn scriptlet_copy_content_shortcut() {
        let script = ScriptInfo::scriptlet("Test", "/path", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
        assert_eq!(cc.shortcut, Some("⌘⌥C".to_string()));
    }


    // --- merged from tests_part_03.rs ---
    #[test]
    fn scriptlet_edit_scriptlet_shortcut() {
        let script = ScriptInfo::scriptlet("Test", "/path", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let edit = actions.iter().find(|a| a.id == "edit_scriptlet").unwrap();
        assert_eq!(edit.shortcut, Some("⌘E".to_string()));
    }

    // =========================================================================
    // 18. AI bar: toggle_shortcuts_help details
    // =========================================================================

    #[test]
    fn ai_bar_toggle_shortcuts_help_shortcut() {
        let actions = get_ai_command_bar_actions();
        let help = actions
            .iter()
            .find(|a| a.id == "toggle_shortcuts_help")
            .unwrap();
        assert_eq!(help.shortcut, Some("⌘/".to_string()));
    }

    #[test]
    fn ai_bar_toggle_shortcuts_help_icon() {
        let actions = get_ai_command_bar_actions();
        let help = actions
            .iter()
            .find(|a| a.id == "toggle_shortcuts_help")
            .unwrap();
        assert_eq!(help.icon, Some(IconName::Star));
    }

    #[test]
    fn ai_bar_toggle_shortcuts_help_section() {
        let actions = get_ai_command_bar_actions();
        let help = actions
            .iter()
            .find(|a| a.id == "toggle_shortcuts_help")
            .unwrap();
        assert_eq!(help.section, Some("Help".to_string()));
    }

    #[test]
    fn ai_bar_toggle_shortcuts_help_title() {
        let actions = get_ai_command_bar_actions();
        let help = actions
            .iter()
            .find(|a| a.id == "toggle_shortcuts_help")
            .unwrap();
        assert_eq!(help.title, "Keyboard Shortcuts");
    }

    // =========================================================================
    // 19. AI bar: change_model has no shortcut
    // =========================================================================

    #[test]
    fn ai_bar_change_model_no_shortcut() {
        let actions = get_ai_command_bar_actions();
        let model = actions.iter().find(|a| a.id == "change_model").unwrap();
        assert!(model.shortcut.is_none());
    }

    #[test]
    fn ai_bar_change_model_icon_settings() {
        let actions = get_ai_command_bar_actions();
        let model = actions.iter().find(|a| a.id == "change_model").unwrap();
        assert_eq!(model.icon, Some(IconName::Settings));
    }

    #[test]
    fn ai_bar_change_model_section_settings() {
        let actions = get_ai_command_bar_actions();
        let model = actions.iter().find(|a| a.id == "change_model").unwrap();
        assert_eq!(model.section, Some("Settings".to_string()));
    }

    #[test]
    fn ai_bar_branch_from_last_no_shortcut() {
        let actions = get_ai_command_bar_actions();
        let branch = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
        assert!(branch.shortcut.is_none());
    }

    // =========================================================================
    // 20. AI bar: unique IDs across all 12 actions
    // =========================================================================

    #[test]
    fn ai_bar_has_12_actions() {
        let actions = get_ai_command_bar_actions();
        assert_eq!(actions.len(), 12);
    }

    #[test]
    fn ai_bar_all_ids_unique() {
        let actions = get_ai_command_bar_actions();
        let mut ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        let original_len = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), original_len);
    }

    #[test]
    fn ai_bar_all_have_icons() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                action.icon.is_some(),
                "AI bar action '{}' has no icon",
                action.id
            );
        }
    }

    #[test]
    fn ai_bar_all_have_sections() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                action.section.is_some(),
                "AI bar action '{}' has no section",
                action.id
            );
        }
    }

    // =========================================================================
    // 21. Notes: export action requires selection and not trash
    // =========================================================================

    #[test]
    fn notes_export_present_with_selection_no_trash() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "export"));
    }

    #[test]
    fn notes_export_absent_without_selection() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "export"));
    }

    #[test]
    fn notes_export_absent_in_trash() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "export"));
    }

    #[test]
    fn notes_export_shortcut_and_section() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let export = actions.iter().find(|a| a.id == "export").unwrap();
        assert_eq!(export.shortcut, Some("⇧⌘E".to_string()));
        assert_eq!(export.section, Some("Export".to_string()));
    }

    // =========================================================================
    // 22. Notes: browse_notes always present
    // =========================================================================

    #[test]
    fn notes_browse_notes_present_no_selection() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "browse_notes"));
    }

    #[test]
    fn notes_browse_notes_present_with_selection() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "browse_notes"));
    }

    #[test]
    fn notes_browse_notes_present_in_trash() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "browse_notes"));
    }

    #[test]
    fn notes_browse_notes_shortcut_and_icon() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let browse = actions.iter().find(|a| a.id == "browse_notes").unwrap();
        assert_eq!(browse.shortcut, Some("⌘P".to_string()));
        assert_eq!(browse.icon, Some(IconName::FolderOpen));
    }

    // =========================================================================
    // 23. Chat context: copy_response only when has_response
    // =========================================================================

    #[test]
    fn chat_copy_response_present_when_has_response() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".into()),
            available_models: vec![],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "copy_response"));
    }

    #[test]
    fn chat_copy_response_absent_when_no_response() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".into()),
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "copy_response"));
    }

    #[test]
    fn chat_copy_response_shortcut() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        let copy = actions.iter().find(|a| a.id == "copy_response").unwrap();
        assert_eq!(copy.shortcut, Some("⌘C".to_string()));
    }

    #[test]
    fn chat_copy_response_title() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        let copy = actions.iter().find(|a| a.id == "copy_response").unwrap();
        assert_eq!(copy.title, "Copy Last Response");
    }

    // =========================================================================
    // 24. Chat context: clear_conversation only when has_messages
    // =========================================================================

    #[test]
    fn chat_clear_present_when_has_messages() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "clear_conversation"));
    }

    #[test]
    fn chat_clear_absent_when_no_messages() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "clear_conversation"));
    }

    #[test]
    fn chat_clear_shortcut() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let clear = actions
            .iter()
            .find(|a| a.id == "clear_conversation")
            .unwrap();
        assert_eq!(clear.shortcut, Some("⌘⌫".to_string()));
    }

    #[test]
    fn chat_continue_in_chat_always_present() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "continue_in_chat"));
    }

    // =========================================================================
    // 25. New chat: empty lists produce zero actions
    // =========================================================================

    #[test]
    fn new_chat_empty_inputs_zero_actions() {
        let actions = get_new_chat_actions(&[], &[], &[]);
        assert_eq!(actions.len(), 0);
    }

    #[test]
    fn new_chat_only_last_used_produces_correct_count() {
        let last_used = vec![NewChatModelInfo {
            model_id: "claude".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions.len(), 1);
    }

    #[test]
    fn new_chat_only_models_produces_correct_count() {
        let models = vec![
            NewChatModelInfo {
                model_id: "gpt4".into(),
                display_name: "GPT-4".into(),
                provider: "openai".into(),
                provider_display_name: "OpenAI".into(),
            },
            NewChatModelInfo {
                model_id: "claude".into(),
                display_name: "Claude".into(),
                provider: "anthropic".into(),
                provider_display_name: "Anthropic".into(),
            },
        ];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions.len(), 2);
    }

    #[test]
    fn new_chat_all_three_sections_total() {
        let last_used = vec![NewChatModelInfo {
            model_id: "claude".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "openai".into(),
            provider_display_name: "OpenAI".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        assert_eq!(actions.len(), 3);
    }

    // =========================================================================
    // 26. New chat: preset IDs use preset_{id} format
    // =========================================================================

    #[test]
    fn new_chat_preset_id_format() {
        let presets = vec![NewChatPresetInfo {
            id: "code-review".into(),
            name: "Code Review".into(),
            icon: IconName::Code,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].id, "preset_code-review");
    }

    #[test]
    fn new_chat_preset_section_is_presets() {
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].section, Some("Presets".to_string()));
    }

    #[test]
    fn new_chat_preset_icon_preserved() {
        let presets = vec![NewChatPresetInfo {
            id: "code".into(),
            name: "Code".into(),
            icon: IconName::Code,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].icon, Some(IconName::Code));
    }


    // --- merged from tests_part_04.rs ---
    #[test]
    fn new_chat_preset_description_is_none() {
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert!(actions[0].description.is_none());
    }

    // =========================================================================
    // 27. Note switcher: empty notes produces no_notes action
    // =========================================================================

    #[test]
    fn note_switcher_empty_produces_no_notes() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "no_notes");
    }

    #[test]
    fn note_switcher_empty_title() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions[0].title, "No notes yet");
    }

    #[test]
    fn note_switcher_empty_icon_plus() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions[0].icon, Some(IconName::Plus));
    }

    #[test]
    fn note_switcher_empty_desc_mentions_cmd_n() {
        let actions = get_note_switcher_actions(&[]);
        assert!(actions[0].description.as_ref().unwrap().contains("⌘N"));
    }

    // =========================================================================
    // 28. Note switcher: preview with relative_time has separator
    // =========================================================================

    #[test]
    fn note_switcher_preview_and_time_has_dot_separator() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc-123".into(),
            title: "Test Note".into(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "Hello world".into(),
            relative_time: "2m ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(actions[0].description.as_ref().unwrap().contains(" · "));
    }

    #[test]
    fn note_switcher_preview_only_no_separator() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc-123".into(),
            title: "Test Note".into(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "Hello world".into(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(!actions[0].description.as_ref().unwrap().contains(" · "));
    }

    #[test]
    fn note_switcher_time_only_when_empty_preview() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc-123".into(),
            title: "Test Note".into(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: "5d ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].description.as_ref().unwrap(), "5d ago");
    }

    #[test]
    fn note_switcher_no_preview_no_time_shows_char_count() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc-123".into(),
            title: "Test Note".into(),
            char_count: 42,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].description.as_ref().unwrap(), "42 chars");
    }

    // =========================================================================
    // 29. ProtocolAction: with_value sets value and has_action false
    // =========================================================================

    #[test]
    fn protocol_action_with_value_sets_name() {
        let pa = ProtocolAction::with_value("Copy".into(), "copy-cmd".into());
        assert_eq!(pa.name, "Copy");
    }

    #[test]
    fn protocol_action_with_value_sets_value() {
        let pa = ProtocolAction::with_value("Copy".into(), "copy-cmd".into());
        assert_eq!(pa.value, Some("copy-cmd".to_string()));
    }

    #[test]
    fn protocol_action_with_value_has_action_false() {
        let pa = ProtocolAction::with_value("Copy".into(), "copy-cmd".into());
        assert!(!pa.has_action);
    }

    #[test]
    fn protocol_action_with_value_defaults_visible_close_none() {
        let pa = ProtocolAction::with_value("X".into(), "y".into());
        assert!(pa.visible.is_none());
        assert!(pa.close.is_none());
        // But defaults still work:
        assert!(pa.is_visible());
        assert!(pa.should_close());
    }

    // =========================================================================
    // 30. format_shortcut_hint: dialog vs builders produce different results
    // =========================================================================

    #[test]
    fn dialog_format_shortcut_hint_handles_meta() {
        let result = ActionsDialog::format_shortcut_hint("meta+c");
        assert_eq!(result, "⌘C");
    }

    #[test]
    fn dialog_format_shortcut_hint_handles_super() {
        let result = ActionsDialog::format_shortcut_hint("super+x");
        assert_eq!(result, "⌘X");
    }

    #[test]
    fn dialog_format_shortcut_hint_handles_option() {
        let result = ActionsDialog::format_shortcut_hint("option+v");
        assert_eq!(result, "⌥V");
    }

    #[test]
    fn dialog_format_shortcut_hint_handles_compound() {
        let result = ActionsDialog::format_shortcut_hint("ctrl+shift+alt+k");
        assert_eq!(result, "⌃⇧⌥K");
    }

}
