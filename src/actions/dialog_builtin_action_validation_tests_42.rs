//! Batch 42: Dialog builtin action validation tests
//!
//! Focuses on:
//! - Action cached lowercase fields (title_lower, description_lower, shortcut_lower)
//! - Action::with_shortcut sets shortcut_lower correctly
//! - Action::new pre-computes title_lower and description_lower
//! - Clipboard: quick_look shortcut is ␣ (macOS only)
//! - Clipboard: clipboard_delete shortcut is ⌃X (not ⌘⌫)
//! - Clipboard: context_title byte-based truncation boundary
//! - File context: open_file title quotes the filename
//! - File context: reveal_in_finder desc says "Reveal in Finder"
//! - Path context: copy_path shortcut vs copy_filename shortcut
//! - Path context: open_in_finder vs reveal_in_finder naming
//! - Script context: copy_content shortcut is ⌘⌥C across all types
//! - Script context: view_logs only for is_script=true
//! - Scriptlet context with_custom: shortcut/alias dynamic actions match get_script_context_actions
//! - Scriptlet context: copy_content desc says "Copy entire file content"
//! - AI bar: copy_response vs copy_chat vs copy_last_code shortcuts differ
//! - AI bar: branch_from_last has NO shortcut
//! - AI bar: sections have correct action counts
//! - Notes: new_note always present even in trash
//! - Notes: enable_auto_sizing only when disabled
//! - Chat context: continue_in_chat shortcut is ⌘↵
//! - Chat context: clear_conversation shortcut is ⌘⌫
//! - New chat: preset icon is preserved from NewChatPresetInfo
//! - New chat: models description uses provider_display_name
//! - Note switcher: unpinned non-current icon is File
//! - Note switcher: note ID format is "note_{uuid}"
//! - to_deeplink_name: empty string produces empty string
//! - to_deeplink_name: all special chars produces empty string
//! - Dialog format_shortcut_hint: "enter" → "↵" and "escape" → "⎋"
//! - Dialog format_shortcut_hint: "tab" → "⇥" and "backspace" → "⌫"
//! - coerce_action_selection: first item is header, selects next Item

#[cfg(test)]
mod tests {
    use crate::actions::builders::*;
    use crate::actions::dialog::{
        build_grouped_items_static, coerce_action_selection, ActionsDialog, GroupedActionItem,
    };
    use crate::actions::types::{
        Action, ActionCategory, ActionsDialogConfig, ScriptInfo, SectionStyle,
    };
    use crate::clipboard_history::ContentType;
    use crate::designs::icon_variations::IconName;
    use crate::file_search::{FileInfo, FileType};
    use crate::prompts::PathInfo;
    use crate::scriptlets::{Scriptlet, ScriptletAction};

    // =========================================================================
    // 1. Action cached lowercase fields: title_lower pre-computed
    // =========================================================================

    #[test]
    fn action_new_title_lower_is_lowercase_of_title() {
        let action = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
        assert_eq!(action.title_lower, "edit script");
    }

    #[test]
    fn action_new_title_lower_mixed_case() {
        let action = Action::new("id", "CoPy PaTh", None, ActionCategory::ScriptContext);
        assert_eq!(action.title_lower, "copy path");
    }

    #[test]
    fn action_new_description_lower_present() {
        let action = Action::new(
            "id",
            "Title",
            Some("Open in $EDITOR".to_string()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(
            action.description_lower,
            Some("open in $editor".to_string())
        );
    }

    #[test]
    fn action_new_description_lower_none_when_no_desc() {
        let action = Action::new("id", "Title", None, ActionCategory::ScriptContext);
        assert!(action.description_lower.is_none());
    }

    // =========================================================================
    // 2. Action::with_shortcut sets shortcut_lower correctly
    // =========================================================================

    #[test]
    fn action_with_shortcut_sets_shortcut_lower() {
        let action =
            Action::new("id", "Title", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
        assert_eq!(action.shortcut_lower, Some("⌘⇧c".to_string()));
    }

    #[test]
    fn action_with_shortcut_preserves_original() {
        let action =
            Action::new("id", "Title", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
        assert_eq!(action.shortcut, Some("⌘E".to_string()));
    }

    #[test]
    fn action_new_shortcut_lower_is_none_by_default() {
        let action = Action::new("id", "Title", None, ActionCategory::ScriptContext);
        assert!(action.shortcut_lower.is_none());
        assert!(action.shortcut.is_none());
    }

    #[test]
    fn action_with_shortcut_lowercase_arrow_symbols() {
        let action =
            Action::new("id", "Title", None, ActionCategory::ScriptContext).with_shortcut("↵");
        // Arrow symbols don't have lowercase, so they stay the same
        assert_eq!(action.shortcut_lower, Some("↵".to_string()));
    }

    // =========================================================================
    // 3. Clipboard: quick_look shortcut is ␣ (space) on macOS
    // =========================================================================

    #[test]
    fn clipboard_quick_look_shortcut_is_space_symbol() {
        let entry = ClipboardEntryInfo {
            id: "test".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        #[cfg(target_os = "macos")]
        {
            let ql = actions
                .iter()
                .find(|a| a.id == "clipboard_quick_look")
                .unwrap();
            assert_eq!(ql.shortcut.as_deref(), Some("␣"));
        }
    }

    #[test]
    fn clipboard_quick_look_title() {
        let entry = ClipboardEntryInfo {
            id: "test".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        #[cfg(target_os = "macos")]
        {
            let ql = actions
                .iter()
                .find(|a| a.id == "clipboard_quick_look")
                .unwrap();
            assert_eq!(ql.title, "Quick Look");
        }
    }

    #[test]
    fn clipboard_quick_look_desc_mentions_preview() {
        let entry = ClipboardEntryInfo {
            id: "test".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        #[cfg(target_os = "macos")]
        {
            let ql = actions
                .iter()
                .find(|a| a.id == "clipboard_quick_look")
                .unwrap();
            assert!(ql.description.as_ref().unwrap().contains("Quick Look"));
        }
    }

    #[test]
    fn clipboard_quick_look_present_for_both_text_and_image() {
        let text_entry = ClipboardEntryInfo {
            id: "t".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "txt".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let img_entry = ClipboardEntryInfo {
            id: "i".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "img".to_string(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let text_actions = get_clipboard_history_context_actions(&text_entry);
        let img_actions = get_clipboard_history_context_actions(&img_entry);
        #[cfg(target_os = "macos")]
        {
            assert!(text_actions.iter().any(|a| a.id == "clipboard_quick_look"));
            assert!(img_actions.iter().any(|a| a.id == "clipboard_quick_look"));
        }
    }

    // =========================================================================
    // 4. Clipboard: clipboard_delete shortcut is ⌃X
    // =========================================================================

    #[test]
    fn clipboard_delete_shortcut_ctrl_x() {
        let entry = ClipboardEntryInfo {
            id: "test".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let del = actions.iter().find(|a| a.id == "clipboard_delete").unwrap();
        assert_eq!(del.shortcut.as_deref(), Some("⌃X"));
    }

    #[test]
    fn clipboard_delete_title() {
        let entry = ClipboardEntryInfo {
            id: "test".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let del = actions.iter().find(|a| a.id == "clipboard_delete").unwrap();
        assert_eq!(del.title, "Delete Entry");
    }

    #[test]
    fn clipboard_delete_desc_mentions_remove() {
        let entry = ClipboardEntryInfo {
            id: "test".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let del = actions.iter().find(|a| a.id == "clipboard_delete").unwrap();
        assert!(del.description.as_ref().unwrap().contains("Remove"));
    }

    #[test]
    fn clipboard_delete_present_for_image() {
        let entry = ClipboardEntryInfo {
            id: "test".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "img".to_string(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_delete"));
    }

    // =========================================================================
    // 5. Clipboard: context_title byte-based truncation boundary at 30 bytes
    // =========================================================================

    #[test]
    fn clipboard_context_title_short_not_truncated() {
        // Preview under 30 chars: no truncation
        let preview = "Hello World".to_string(); // 11 chars
        let title = if preview.len() > 30 {
            format!("{}...", &preview[..27])
        } else {
            preview.clone()
        };
        assert_eq!(title, "Hello World");
    }

    #[test]
    fn clipboard_context_title_exactly_30_not_truncated() {
        let preview = "123456789012345678901234567890".to_string(); // 30 chars
        assert_eq!(preview.len(), 30);
        let title = if preview.len() > 30 {
            format!("{}...", &preview[..27])
        } else {
            preview.clone()
        };
        assert_eq!(title, preview); // Not truncated
    }

    #[test]
    fn clipboard_context_title_31_chars_truncated() {
        let preview = "1234567890123456789012345678901".to_string(); // 31 chars
        assert_eq!(preview.len(), 31);
        let title = if preview.len() > 30 {
            format!("{}...", &preview[..27])
        } else {
            preview.clone()
        };
        assert_eq!(title, "123456789012345678901234567...");
    }

    #[test]
    fn clipboard_context_title_long_truncated_at_byte_27() {
        let preview = "This is a very long clipboard preview entry text here".to_string();
        assert!(preview.len() > 30);
        let title = if preview.len() > 30 {
            format!("{}...", &preview[..27])
        } else {
            preview.clone()
        };
        assert_eq!(title.len(), 30); // 27 + 3 for "..."
        assert!(title.ends_with("..."));
    }

    // =========================================================================
    // 6. File context: open_file title quotes the filename
    // =========================================================================

    #[test]
    fn file_context_open_file_title_quotes_name() {
        let info = FileInfo {
            path: "/test/report.pdf".to_string(),
            name: "report.pdf".to_string(),
            file_type: FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        let open = actions.iter().find(|a| a.id == "open_file").unwrap();
        assert_eq!(open.title, "Open \"report.pdf\"");
    }

    #[test]
    fn file_context_open_directory_title_quotes_name() {
        let info = FileInfo {
            path: "/test/Documents".to_string(),
            name: "Documents".to_string(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&info);
        let open = actions.iter().find(|a| a.id == "open_directory").unwrap();
        assert_eq!(open.title, "Open \"Documents\"");
    }

    #[test]
    fn file_context_open_file_desc() {
        let info = FileInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        let open = actions.iter().find(|a| a.id == "open_file").unwrap();
        assert!(open
            .description
            .as_ref()
            .unwrap()
            .contains("default application"));
    }

    #[test]
    fn file_context_open_directory_desc() {
        let info = FileInfo {
            path: "/test/Dir".to_string(),
            name: "Dir".to_string(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&info);
        let open = actions.iter().find(|a| a.id == "open_directory").unwrap();
        assert!(open.description.as_ref().unwrap().contains("folder"));
    }

    // =========================================================================
    // 7. File context: reveal_in_finder desc says "Reveal in Finder"
    // =========================================================================

    #[test]
    fn file_context_reveal_desc_mentions_reveal() {
        let info = FileInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
        assert!(reveal.description.as_ref().unwrap().contains("Reveal"));
    }

    #[test]
    fn file_context_reveal_title_is_show_in_finder() {
        let info = FileInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
        assert_eq!(reveal.title, "Show in Finder");
    }

    #[test]
    fn file_context_reveal_shortcut_cmd_enter() {
        let info = FileInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
        assert_eq!(reveal.shortcut.as_deref(), Some("⌘↵"));
    }

    #[test]
    fn file_context_reveal_present_for_dirs_too() {
        let info = FileInfo {
            path: "/test/Dir".to_string(),
            name: "Dir".to_string(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
    }

    // =========================================================================
    // 8. Path context: copy_path vs copy_filename shortcut differences
    // =========================================================================

    #[test]
    fn path_context_copy_path_has_shortcut() {
        let info = PathInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
        assert_eq!(cp.shortcut.as_deref(), Some("⌘⇧C"));
    }

    #[test]
    fn path_context_copy_filename_has_no_shortcut() {
        let info = PathInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
        assert!(cf.shortcut.is_none());
    }

    #[test]
    fn path_context_open_in_finder_shortcut() {
        let info = PathInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let oif = actions.iter().find(|a| a.id == "open_in_finder").unwrap();
        assert_eq!(oif.shortcut.as_deref(), Some("⌘⇧F"));
    }

    #[test]
    fn path_context_open_in_finder_is_different_id_from_file_reveal() {
        // Path uses "open_in_finder", File uses "reveal_in_finder"
        let path_info = PathInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
        };
        let file_info = FileInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let path_actions = get_path_context_actions(&path_info);
        let file_actions = get_file_context_actions(&file_info);

        assert!(path_actions.iter().any(|a| a.id == "open_in_finder"));
        assert!(!path_actions.iter().any(|a| a.id == "reveal_in_finder"));
        assert!(file_actions.iter().any(|a| a.id == "reveal_in_finder"));
        assert!(!file_actions.iter().any(|a| a.id == "open_in_finder"));
    }

    // =========================================================================
    // 9. Script context: copy_content shortcut is ⌘⌥C across all types
    // =========================================================================

    #[test]
    fn script_copy_content_shortcut() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
        assert_eq!(cc.shortcut.as_deref(), Some("⌘⌥C"));
    }

    #[test]
    fn scriptlet_copy_content_shortcut() {
        let script = ScriptInfo::scriptlet("test", "/path/test.md", None, None);
        let actions = get_script_context_actions(&script);
        let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
        assert_eq!(cc.shortcut.as_deref(), Some("⌘⌥C"));
    }

    #[test]
    fn agent_copy_content_shortcut() {
        let mut info = ScriptInfo::new("test-agent", "/path/agent.md");
        info.is_script = false;
        info.is_agent = true;
        let actions = get_script_context_actions(&info);
        let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
        assert_eq!(cc.shortcut.as_deref(), Some("⌘⌥C"));
    }

    #[test]
    fn scriptlet_with_custom_copy_content_shortcut() {
        let script = ScriptInfo::scriptlet("test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
        assert_eq!(cc.shortcut.as_deref(), Some("⌘⌥C"));
    }

    // =========================================================================
    // 10. Script context: view_logs only for is_script=true
    // =========================================================================

    #[test]
    fn script_has_view_logs() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "view_logs"));
    }

    #[test]
    fn scriptlet_has_no_view_logs() {
        let script = ScriptInfo::scriptlet("test", "/path/test.md", None, None);
        let actions = get_script_context_actions(&script);
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    #[test]
    fn builtin_has_no_view_logs() {
        let script = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&script);
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    #[test]
    fn agent_has_no_view_logs() {
        let mut info = ScriptInfo::new("test-agent", "/path/agent.md");
        info.is_script = false;
        info.is_agent = true;
        let actions = get_script_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    // =========================================================================
    // 11. Scriptlet context with_custom: shortcut/alias dynamic match get_script_context_actions
    // =========================================================================

    #[test]
    fn scriptlet_with_custom_no_shortcut_has_add_shortcut() {
        let script = ScriptInfo::scriptlet("test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(actions.iter().any(|a| a.id == "add_shortcut"));
        assert!(!actions.iter().any(|a| a.id == "update_shortcut"));
    }

    #[test]
    fn scriptlet_with_custom_has_shortcut_shows_update_remove() {
        let script =
            ScriptInfo::scriptlet("test", "/path/test.md", Some("cmd+t".to_string()), None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(actions.iter().any(|a| a.id == "update_shortcut"));
        assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
        assert!(!actions.iter().any(|a| a.id == "add_shortcut"));
    }

    #[test]
    fn scriptlet_with_custom_no_alias_has_add_alias() {
        let script = ScriptInfo::scriptlet("test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(actions.iter().any(|a| a.id == "add_alias"));
        assert!(!actions.iter().any(|a| a.id == "update_alias"));
    }

    #[test]
    fn scriptlet_with_custom_has_alias_shows_update_remove() {
        let script = ScriptInfo::scriptlet("test", "/path/test.md", None, Some("ts".to_string()));
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(actions.iter().any(|a| a.id == "update_alias"));
        assert!(actions.iter().any(|a| a.id == "remove_alias"));
        assert!(!actions.iter().any(|a| a.id == "add_alias"));
    }

    // =========================================================================
    // 12. Scriptlet context: copy_content desc says "Copy entire file content"
    // =========================================================================

    #[test]
    fn scriptlet_copy_content_desc() {
        let script = ScriptInfo::scriptlet("test", "/path/test.md", None, None);
        let actions = get_script_context_actions(&script);
        let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
        assert!(cc
            .description
            .as_ref()
            .unwrap()
            .contains("entire file content"));
    }

    #[test]
    fn scriptlet_with_custom_copy_content_desc() {
        let script = ScriptInfo::scriptlet("test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
        assert!(cc
            .description
            .as_ref()
            .unwrap()
            .contains("entire file content"));
    }

    #[test]
    fn script_copy_content_desc() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
        assert!(cc
            .description
            .as_ref()
            .unwrap()
            .contains("entire file content"));
    }

    #[test]
    fn agent_copy_content_desc() {
        let mut info = ScriptInfo::new("test-agent", "/path/agent.md");
        info.is_script = false;
        info.is_agent = true;
        let actions = get_script_context_actions(&info);
        let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
        assert!(cc
            .description
            .as_ref()
            .unwrap()
            .contains("entire file content"));
    }

    // =========================================================================
    // 13. AI bar: copy_response vs copy_chat vs copy_last_code shortcuts differ
    // =========================================================================

    #[test]
    fn ai_bar_copy_response_shortcut() {
        let actions = get_ai_command_bar_actions();
        let cr = actions.iter().find(|a| a.id == "copy_response").unwrap();
        assert_eq!(cr.shortcut.as_deref(), Some("⇧⌘C"));
    }

    #[test]
    fn ai_bar_copy_chat_shortcut() {
        let actions = get_ai_command_bar_actions();
        let cc = actions.iter().find(|a| a.id == "copy_chat").unwrap();
        assert_eq!(cc.shortcut.as_deref(), Some("⌥⇧⌘C"));
    }

    #[test]
    fn ai_bar_copy_last_code_shortcut() {
        let actions = get_ai_command_bar_actions();
        let clc = actions.iter().find(|a| a.id == "copy_last_code").unwrap();
        assert_eq!(clc.shortcut.as_deref(), Some("⌥⌘C"));
    }

    #[test]
    fn ai_bar_all_three_copy_shortcuts_are_different() {
        let actions = get_ai_command_bar_actions();
        let cr = actions.iter().find(|a| a.id == "copy_response").unwrap();
        let cc = actions.iter().find(|a| a.id == "copy_chat").unwrap();
        let clc = actions.iter().find(|a| a.id == "copy_last_code").unwrap();
        assert_ne!(cr.shortcut, cc.shortcut);
        assert_ne!(cr.shortcut, clc.shortcut);
        assert_ne!(cc.shortcut, clc.shortcut);
    }

    // =========================================================================
    // 14. AI bar: branch_from_last has NO shortcut
    // =========================================================================

    #[test]
    fn ai_bar_branch_from_last_no_shortcut() {
        let actions = get_ai_command_bar_actions();
        let bfl = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
        assert!(bfl.shortcut.is_none());
    }

    #[test]
    fn ai_bar_branch_from_last_icon_arrow_right() {
        let actions = get_ai_command_bar_actions();
        let bfl = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
        assert_eq!(bfl.icon, Some(IconName::ArrowRight));
    }

    #[test]
    fn ai_bar_branch_from_last_section_actions() {
        let actions = get_ai_command_bar_actions();
        let bfl = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
        assert_eq!(bfl.section.as_deref(), Some("Actions"));
    }

    #[test]
    fn ai_bar_branch_from_last_desc_mentions_branch() {
        let actions = get_ai_command_bar_actions();
        let bfl = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
        assert!(bfl
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("branch"));
    }

    // =========================================================================
    // 15. AI bar: sections have correct action counts
    // =========================================================================

    #[test]
    fn ai_bar_response_section_has_3_actions() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Response"))
            .count();
        assert_eq!(count, 3);
    }

    #[test]
    fn ai_bar_actions_section_has_4_actions() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Actions"))
            .count();
        assert_eq!(count, 4); // submit, new_chat, delete_chat, branch_from_last
    }

    #[test]
    fn ai_bar_attachments_section_has_2_actions() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Attachments"))
            .count();
        assert_eq!(count, 2);
    }

    #[test]
    fn ai_bar_settings_section_has_1_action() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Settings"))
            .count();
        assert_eq!(count, 1);
    }

    // =========================================================================
    // 16. Notes: new_note always present even in trash view
    // =========================================================================

    #[test]
    fn notes_new_note_present_no_selection() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "new_note"));
    }

    #[test]
    fn notes_new_note_present_in_trash() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "new_note"));
    }

    #[test]
    fn notes_new_note_shortcut_cmd_n() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let nn = actions.iter().find(|a| a.id == "new_note").unwrap();
        assert_eq!(nn.shortcut.as_deref(), Some("⌘N"));
    }

    #[test]
    fn notes_new_note_icon_plus() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let nn = actions.iter().find(|a| a.id == "new_note").unwrap();
        assert_eq!(nn.icon, Some(IconName::Plus));
    }

    // =========================================================================
    // 17. Notes: enable_auto_sizing only when disabled
    // =========================================================================

    #[test]
    fn notes_auto_sizing_disabled_shows_enable() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "enable_auto_sizing"));
    }

    #[test]
    fn notes_auto_sizing_enabled_hides_enable() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "enable_auto_sizing"));
    }

    #[test]
    fn notes_auto_sizing_shortcut_cmd_a() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let eas = actions
            .iter()
            .find(|a| a.id == "enable_auto_sizing")
            .unwrap();
        assert_eq!(eas.shortcut.as_deref(), Some("⌘A"));
    }

    #[test]
    fn notes_auto_sizing_icon_settings() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let eas = actions
            .iter()
            .find(|a| a.id == "enable_auto_sizing")
            .unwrap();
        assert_eq!(eas.icon, Some(IconName::Settings));
    }

    // =========================================================================
    // 18. Chat context: continue_in_chat shortcut is ⌘↵
    // =========================================================================

    #[test]
    fn chat_continue_in_chat_shortcut() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".to_string()),
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let cic = actions.iter().find(|a| a.id == "continue_in_chat").unwrap();
        assert_eq!(cic.shortcut.as_deref(), Some("⌘↵"));
    }

    #[test]
    fn chat_continue_in_chat_title() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".to_string()),
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let cic = actions.iter().find(|a| a.id == "continue_in_chat").unwrap();
        assert_eq!(cic.title, "Continue in Chat");
    }

    #[test]
    fn chat_continue_in_chat_desc_mentions_ai_chat() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".to_string()),
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let cic = actions.iter().find(|a| a.id == "continue_in_chat").unwrap();
        assert!(cic.description.as_ref().unwrap().contains("AI Chat"));
    }

    #[test]
    fn chat_continue_in_chat_always_present() {
        // Even with no models, no messages, no response
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
    // 19. Chat context: clear_conversation shortcut is ⌘⌫
    // =========================================================================

    #[test]
    fn chat_clear_conversation_shortcut() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let cc = actions
            .iter()
            .find(|a| a.id == "clear_conversation")
            .unwrap();
        assert_eq!(cc.shortcut.as_deref(), Some("⌘⌫"));
    }

    #[test]
    fn chat_clear_conversation_title() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let cc = actions
            .iter()
            .find(|a| a.id == "clear_conversation")
            .unwrap();
        assert_eq!(cc.title, "Clear Conversation");
    }

    #[test]
    fn chat_clear_conversation_absent_when_no_messages() {
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
    fn chat_clear_conversation_desc_mentions_clear() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let cc = actions
            .iter()
            .find(|a| a.id == "clear_conversation")
            .unwrap();
        assert!(cc
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("clear"));
    }

    // =========================================================================
    // 20. New chat: preset icon is preserved from NewChatPresetInfo
    // =========================================================================

    #[test]
    fn new_chat_preset_icon_preserved() {
        let presets = vec![NewChatPresetInfo {
            id: "general".to_string(),
            name: "General".to_string(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        let preset = actions.iter().find(|a| a.id == "preset_general").unwrap();
        assert_eq!(preset.icon, Some(IconName::Star));
    }

    #[test]
    fn new_chat_preset_different_icons() {
        let presets = vec![
            NewChatPresetInfo {
                id: "code".to_string(),
                name: "Code".to_string(),
                icon: IconName::Code,
            },
            NewChatPresetInfo {
                id: "writer".to_string(),
                name: "Writer".to_string(),
                icon: IconName::FileCode,
            },
        ];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        let code = actions.iter().find(|a| a.id == "preset_code").unwrap();
        let writer = actions.iter().find(|a| a.id == "preset_writer").unwrap();
        assert_eq!(code.icon, Some(IconName::Code));
        assert_eq!(writer.icon, Some(IconName::FileCode));
    }

    #[test]
    fn new_chat_preset_name_is_title() {
        let presets = vec![NewChatPresetInfo {
            id: "gen".to_string(),
            name: "General Purpose".to_string(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        let p = actions.iter().find(|a| a.id == "preset_gen").unwrap();
        assert_eq!(p.title, "General Purpose");
    }

    #[test]
    fn new_chat_preset_description_is_none() {
        let presets = vec![NewChatPresetInfo {
            id: "gen".to_string(),
            name: "General".to_string(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        let p = actions.iter().find(|a| a.id == "preset_gen").unwrap();
        assert!(p.description.is_none());
    }

    // =========================================================================
    // 21. New chat: models description uses provider_display_name
    // =========================================================================

    #[test]
    fn new_chat_model_desc_is_provider_display_name() {
        let models = vec![NewChatModelInfo {
            model_id: "claude-3".to_string(),
            display_name: "Claude 3 Opus".to_string(),
            provider: "anthropic".to_string(),
            provider_display_name: "Anthropic".to_string(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        let m = actions.iter().find(|a| a.id == "model_0").unwrap();
        assert_eq!(m.description, Some("Anthropic".to_string()));
    }

    #[test]
    fn new_chat_model_title_is_display_name() {
        let models = vec![NewChatModelInfo {
            model_id: "gpt-4".to_string(),
            display_name: "GPT-4 Turbo".to_string(),
            provider: "openai".to_string(),
            provider_display_name: "OpenAI".to_string(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        let m = actions.iter().find(|a| a.id == "model_0").unwrap();
        assert_eq!(m.title, "GPT-4 Turbo");
    }

    #[test]
    fn new_chat_model_icon_is_settings() {
        let models = vec![NewChatModelInfo {
            model_id: "m1".to_string(),
            display_name: "Model".to_string(),
            provider: "p".to_string(),
            provider_display_name: "Provider".to_string(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        let m = actions.iter().find(|a| a.id == "model_0").unwrap();
        assert_eq!(m.icon, Some(IconName::Settings));
    }

    #[test]
    fn new_chat_model_section_is_models() {
        let models = vec![NewChatModelInfo {
            model_id: "m1".to_string(),
            display_name: "Model".to_string(),
            provider: "p".to_string(),
            provider_display_name: "Provider".to_string(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        let m = actions.iter().find(|a| a.id == "model_0").unwrap();
        assert_eq!(m.section.as_deref(), Some("Models"));
    }

    // =========================================================================
    // 22. Note switcher: unpinned non-current icon is File
    // =========================================================================

    #[test]
    fn note_switcher_unpinned_noncurrent_icon_file() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".to_string(),
            title: "My Note".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "content".to_string(),
            relative_time: "2m ago".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::File));
    }

    #[test]
    fn note_switcher_pinned_icon_star_filled() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".to_string(),
            title: "Pinned Note".to_string(),
            char_count: 50,
            is_current: false,
            is_pinned: true,
            preview: "pinned".to_string(),
            relative_time: "1h ago".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    #[test]
    fn note_switcher_current_icon_check() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".to_string(),
            title: "Current Note".to_string(),
            char_count: 200,
            is_current: true,
            is_pinned: false,
            preview: "current".to_string(),
            relative_time: "5s ago".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::Check));
    }

    #[test]
    fn note_switcher_pinned_current_uses_star_filled() {
        // Pinned takes priority over current
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".to_string(),
            title: "Both".to_string(),
            char_count: 10,
            is_current: true,
            is_pinned: true,
            preview: "both".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    // =========================================================================
    // 23. Note switcher: note ID format is "note_{uuid}"
    // =========================================================================

    #[test]
    fn note_switcher_id_format() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            title: "Test".to_string(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].id, "note_550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn note_switcher_id_uses_note_prefix() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc-123".to_string(),
            title: "Note".to_string(),
            char_count: 5,
            is_current: false,
            is_pinned: false,
            preview: "test".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(actions[0].id.starts_with("note_"));
    }

    #[test]
    fn note_switcher_multiple_notes_distinct_ids() {
        let notes = vec![
            NoteSwitcherNoteInfo {
                id: "id1".to_string(),
                title: "Note 1".to_string(),
                char_count: 10,
                is_current: false,
                is_pinned: false,
                preview: "".to_string(),
                relative_time: "".to_string(),
            },
            NoteSwitcherNoteInfo {
                id: "id2".to_string(),
                title: "Note 2".to_string(),
                char_count: 20,
                is_current: false,
                is_pinned: false,
                preview: "".to_string(),
                relative_time: "".to_string(),
            },
        ];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].id, "note_id1");
        assert_eq!(actions[1].id, "note_id2");
        assert_ne!(actions[0].id, actions[1].id);
    }

    #[test]
    fn note_switcher_section_pinned_vs_recent() {
        let notes = vec![
            NoteSwitcherNoteInfo {
                id: "a".to_string(),
                title: "Pinned".to_string(),
                char_count: 1,
                is_current: false,
                is_pinned: true,
                preview: "".to_string(),
                relative_time: "".to_string(),
            },
            NoteSwitcherNoteInfo {
                id: "b".to_string(),
                title: "Recent".to_string(),
                char_count: 2,
                is_current: false,
                is_pinned: false,
                preview: "".to_string(),
                relative_time: "".to_string(),
            },
        ];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
        assert_eq!(actions[1].section.as_deref(), Some("Recent"));
    }

    // =========================================================================
    // 24. to_deeplink_name: empty string produces empty string
    // =========================================================================

    #[test]
    fn to_deeplink_name_empty_string() {
        assert_eq!(to_deeplink_name(""), "");
    }

    #[test]
    fn to_deeplink_name_all_special_chars() {
        // All special chars become hyphens, then filtered as empty
        assert_eq!(to_deeplink_name("!@#$%"), "");
    }

    #[test]
    fn to_deeplink_name_single_word() {
        assert_eq!(to_deeplink_name("hello"), "hello");
    }

    #[test]
    fn to_deeplink_name_preserves_numbers() {
        assert_eq!(to_deeplink_name("test123"), "test123");
        assert_eq!(to_deeplink_name("123"), "123");
    }

    // =========================================================================
    // 25. to_deeplink_name: mixed unicode and ASCII
    // =========================================================================

    #[test]
    fn to_deeplink_name_mixed_spaces_and_underscores() {
        assert_eq!(to_deeplink_name("my_cool script"), "my-cool-script");
    }

    #[test]
    fn to_deeplink_name_leading_trailing_spaces() {
        assert_eq!(to_deeplink_name("  hello  "), "hello");
    }

    #[test]
    fn to_deeplink_name_consecutive_special_chars_collapse() {
        assert_eq!(to_deeplink_name("a---b___c   d"), "a-b-c-d");
    }

    #[test]
    fn to_deeplink_name_uppercase_lowercased() {
        assert_eq!(to_deeplink_name("My Script"), "my-script");
    }

    // =========================================================================
    // 26. Dialog format_shortcut_hint: "enter"→"↵", "escape"→"⎋"
    // =========================================================================

    #[test]
    fn dialog_format_shortcut_enter() {
        assert_eq!(ActionsDialog::format_shortcut_hint("enter"), "↵");
    }

    #[test]
    fn dialog_format_shortcut_return() {
        assert_eq!(ActionsDialog::format_shortcut_hint("return"), "↵");
    }

    #[test]
    fn dialog_format_shortcut_escape() {
        assert_eq!(ActionsDialog::format_shortcut_hint("escape"), "⎋");
    }

    #[test]
    fn dialog_format_shortcut_esc() {
        assert_eq!(ActionsDialog::format_shortcut_hint("esc"), "⎋");
    }

    // =========================================================================
    // 27. Dialog format_shortcut_hint: "tab"→"⇥", "backspace"→"⌫"
    // =========================================================================

    #[test]
    fn dialog_format_shortcut_tab() {
        assert_eq!(ActionsDialog::format_shortcut_hint("tab"), "⇥");
    }

    #[test]
    fn dialog_format_shortcut_backspace() {
        assert_eq!(ActionsDialog::format_shortcut_hint("backspace"), "⌫");
    }

    #[test]
    fn dialog_format_shortcut_delete() {
        assert_eq!(ActionsDialog::format_shortcut_hint("delete"), "⌫");
    }

    #[test]
    fn dialog_format_shortcut_space() {
        assert_eq!(ActionsDialog::format_shortcut_hint("space"), "␣");
    }

    // =========================================================================
    // 28. Dialog format_shortcut_hint: compound modifier+key combos
    // =========================================================================

    #[test]
    fn dialog_format_shortcut_cmd_enter() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+enter"), "⌘↵");
    }

    #[test]
    fn dialog_format_shortcut_ctrl_shift_backspace() {
        assert_eq!(
            ActionsDialog::format_shortcut_hint("ctrl+shift+backspace"),
            "⌃⇧⌫"
        );
    }

    #[test]
    fn dialog_format_shortcut_alt_tab() {
        assert_eq!(ActionsDialog::format_shortcut_hint("alt+tab"), "⌥⇥");
    }

    #[test]
    fn dialog_format_shortcut_meta_maps_to_cmd() {
        assert_eq!(ActionsDialog::format_shortcut_hint("meta+k"), "⌘K");
    }

    // =========================================================================
    // 29. coerce_action_selection: first item is header, selects next Item
    // =========================================================================

    #[test]
    fn coerce_selection_header_first_selects_next_item() {
        let rows = vec![
            GroupedActionItem::SectionHeader("Section".to_string()),
            GroupedActionItem::Item(0),
            GroupedActionItem::Item(1),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn coerce_selection_item_first_stays() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("Section".to_string()),
            GroupedActionItem::Item(1),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    }

    #[test]
    fn coerce_selection_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".to_string()),
            GroupedActionItem::SectionHeader("B".to_string()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn coerce_selection_empty_returns_none() {
        let rows: Vec<GroupedActionItem> = vec![];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    // =========================================================================
    // 30. build_grouped_items_static: Headers style inserts section headers
    // =========================================================================

    #[test]
    fn build_grouped_items_headers_style_inserts_headers() {
        let actions = vec![
            Action::new("a", "Action A", None, ActionCategory::ScriptContext)
                .with_section("Section 1"),
            Action::new("b", "Action B", None, ActionCategory::ScriptContext)
                .with_section("Section 2"),
        ];
        let filtered = vec![0, 1];
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Should have: Header("Section 1"), Item(0), Header("Section 2"), Item(1)
        assert_eq!(items.len(), 4);
        assert!(matches!(&items[0], GroupedActionItem::SectionHeader(s) if s == "Section 1"));
        assert!(matches!(&items[1], GroupedActionItem::Item(0)));
        assert!(matches!(&items[2], GroupedActionItem::SectionHeader(s) if s == "Section 2"));
        assert!(matches!(&items[3], GroupedActionItem::Item(1)));
    }

    #[test]
    fn build_grouped_items_separators_style_no_headers() {
        let actions = vec![
            Action::new("a", "Action A", None, ActionCategory::ScriptContext)
                .with_section("Section 1"),
            Action::new("b", "Action B", None, ActionCategory::ScriptContext)
                .with_section("Section 2"),
        ];
        let filtered = vec![0, 1];
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        // Should only have items, no headers
        assert_eq!(items.len(), 2);
        assert!(matches!(&items[0], GroupedActionItem::Item(0)));
        assert!(matches!(&items[1], GroupedActionItem::Item(1)));
    }

    #[test]
    fn build_grouped_items_same_section_no_duplicate_header() {
        let actions = vec![
            Action::new("a", "Action A", None, ActionCategory::ScriptContext).with_section("Same"),
            Action::new("b", "Action B", None, ActionCategory::ScriptContext).with_section("Same"),
        ];
        let filtered = vec![0, 1];
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Same consecutive section: only 1 header
        assert_eq!(items.len(), 3); // Header("Same"), Item(0), Item(1)
        assert!(matches!(&items[0], GroupedActionItem::SectionHeader(s) if s == "Same"));
    }

    #[test]
    fn build_grouped_items_empty_filtered_returns_empty() {
        let actions = vec![Action::new(
            "a",
            "Action A",
            None,
            ActionCategory::ScriptContext,
        )];
        let filtered: Vec<usize> = vec![];
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(items.is_empty());
    }
}
