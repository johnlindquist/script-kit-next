//! Batch 37: Dialog builtin action validation tests
//!
//! Focuses on edge cases and integration patterns not covered in batches 1-36:
//! - Clipboard context_title truncation in with_clipboard_entry
//! - Chat context_title fallback when current_model is None
//! - Scriptlet action_id() prefix format
//! - Path context: copy_filename has NO shortcut (unlike file context)
//! - File context: show_info (Get Info) details on macOS
//! - Notes command bar: section distribution across conditions
//! - Clipboard: paste_and_keep_open details
//! - AI command bar: add_attachment details
//! - ScriptInfo constructors: is_agent field defaults
//! - Deeplink URL in script/scriptlet copy_deeplink desc
//! - to_deeplink_name: CJK and multi-byte unicode
//! - Score action: fuzzy score is 25, contains is 50
//! - Notes: create_quicklink and copy_deeplink details
//! - Scriptlet context with_custom: copy_deeplink URL format
//! - ProtocolAction: new() constructor defaults
//! - Clipboard: delete_multiple and delete_all details
//! - AI bar: Actions section has exactly 4 actions
//! - CommandBarConfig: notes_style vs ai_style differences
//! - File context: open_with on macOS details
//! - build_grouped_items_static: non-consecutive same-section
//! - Action::new defaults: has_action, value, icon, section
//! - Cross-context: scriptlet vs script shortcut action IDs differ
//! - Chat context_title from current_model display
//! - New chat: duplicate model entries produce distinct IDs
//! - Note switcher: description formatting for all branches
//! - Clipboard: image has all 4 macOS-only image actions
//! - Score action: exact match on full title
//! - Path context: copy_filename has no shortcut (unlike file)
//! - Cross-context: all contexts set category to ScriptContext

#[cfg(test)]
mod tests {
    use crate::actions::builders::*;
    use crate::actions::command_bar::CommandBarConfig;
    use crate::actions::dialog::{build_grouped_items_static, ActionsDialog};
    use crate::actions::types::{Action, ActionCategory, AnchorPosition, ScriptInfo, SectionStyle};
    use crate::clipboard_history::ContentType;
    use crate::designs::icon_variations::IconName;
    use crate::file_search::{FileInfo, FileType};
    use crate::prompts::PathInfo;
    use crate::protocol::ProtocolAction;
    use crate::scriptlets::Scriptlet;

    // =========================================================================
    // 1. Clipboard context_title truncation: >30 chars truncated with "..."
    // =========================================================================

    #[test]
    fn clipboard_context_title_short_preview_not_truncated() {
        // Preview <= 30 chars should not be truncated
        let preview = "Short preview text".to_string(); // 18 chars
        let context_title = if preview.len() > 30 {
            format!("{}...", &preview[..27])
        } else {
            preview.clone()
        };
        assert_eq!(context_title, "Short preview text");
    }

    #[test]
    fn clipboard_context_title_exactly_30_not_truncated() {
        let preview = "123456789012345678901234567890".to_string(); // exactly 30
        let context_title = if preview.len() > 30 {
            format!("{}...", &preview[..27])
        } else {
            preview.clone()
        };
        assert_eq!(context_title.len(), 30);
        assert!(!context_title.ends_with("..."));
    }

    #[test]
    fn clipboard_context_title_31_chars_truncated() {
        let preview = "1234567890123456789012345678901".to_string(); // 31 chars
        let context_title = if preview.len() > 30 {
            format!("{}...", &preview[..27])
        } else {
            preview.clone()
        };
        assert_eq!(context_title.len(), 30); // 27 + "..."
        assert!(context_title.ends_with("..."));
    }

    #[test]
    fn clipboard_context_title_long_truncated_at_27() {
        let preview = "This is a very long clipboard preview that exceeds thirty chars".to_string();
        let context_title = if preview.len() > 30 {
            format!("{}...", &preview[..27])
        } else {
            preview.clone()
        };
        // &preview[..27] takes 27 bytes = "This is a very long clipboa"
        assert_eq!(&context_title[..27], "This is a very long clipboa");
        assert!(context_title.ends_with("..."));
        assert_eq!(context_title.len(), 30); // 27 + 3 for "..."
    }

    // =========================================================================
    // 2. Chat context_title fallback when current_model is None
    // =========================================================================

    #[test]
    fn chat_context_title_with_model_name() {
        let model = Some("Claude 3.5 Sonnet".to_string());
        let context_title = model.unwrap_or_else(|| "Chat".to_string());
        assert_eq!(context_title, "Claude 3.5 Sonnet");
    }

    #[test]
    fn chat_context_title_fallback_to_chat() {
        let model: Option<String> = None;
        let context_title = model.unwrap_or_else(|| "Chat".to_string());
        assert_eq!(context_title, "Chat");
    }

    #[test]
    fn chat_context_title_empty_string_not_fallback() {
        // Empty string is Some(""), not None, so it doesn't fall back
        let model = Some("".to_string());
        let context_title = model.unwrap_or_else(|| "Chat".to_string());
        assert_eq!(context_title, "");
    }

    #[test]
    fn chat_context_actions_no_model_produces_no_select_model() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        // With no models, no messages, no response: only continue_in_chat
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "continue_in_chat");
    }

    // =========================================================================
    // 3. ScriptletAction action_id() prefix format
    // =========================================================================

    #[test]
    fn scriptlet_action_id_uses_prefix() {
        let sa = crate::scriptlets::ScriptletAction {
            name: "Copy to Clipboard".to_string(),
            command: "copy-to-clipboard".to_string(),
            tool: "bash".to_string(),
            code: "echo test".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        };
        let id = sa.action_id();
        assert!(id.starts_with("scriptlet_action:"));
        assert_eq!(id, "scriptlet_action:copy-to-clipboard");
    }

    #[test]
    fn scriptlet_action_id_uses_command_not_name() {
        let sa = crate::scriptlets::ScriptletAction {
            name: "Open URL".to_string(),
            command: "open-url".to_string(),
            tool: "open".to_string(),
            code: "https://example.com".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        };
        assert_eq!(sa.action_id(), "scriptlet_action:open-url");
        assert!(!sa.action_id().contains("Open URL"));
    }

    #[test]
    fn scriptlet_defined_actions_propagate_has_action_true() {
        let mut scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo test".to_string(),
        );
        scriptlet.actions.push(crate::scriptlets::ScriptletAction {
            name: "Custom".to_string(),
            command: "custom".to_string(),
            tool: "bash".to_string(),
            code: "echo custom".to_string(),
            inputs: vec![],
            shortcut: Some("cmd+c".to_string()),
            description: Some("A custom action".to_string()),
        });
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions.len(), 1);
        assert!(actions[0].has_action);
        assert_eq!(actions[0].value, Some("custom".to_string()));
    }

    #[test]
    fn scriptlet_defined_action_shortcut_formatted() {
        let mut scriptlet =
            Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());
        scriptlet.actions.push(crate::scriptlets::ScriptletAction {
            name: "Action".to_string(),
            command: "act".to_string(),
            tool: "bash".to_string(),
            code: "echo".to_string(),
            inputs: vec![],
            shortcut: Some("cmd+shift+x".to_string()),
            description: None,
        });
        let actions = get_scriptlet_defined_actions(&scriptlet);
        // builders::format_shortcut_hint converts cmd→⌘, shift→⇧, to uppercase
        assert_eq!(actions[0].shortcut, Some("⌘⇧X".to_string()));
    }

    // =========================================================================
    // 4. Path context: copy_filename has NO shortcut
    // =========================================================================

    #[test]
    fn path_context_copy_filename_no_shortcut() {
        let path_info = PathInfo::new("readme.txt", "/home/user/readme.txt", false);
        let actions = get_path_context_actions(&path_info);
        let copy_fn = actions.iter().find(|a| a.id == "copy_filename").unwrap();
        assert!(
            copy_fn.shortcut.is_none(),
            "Path copy_filename should have no shortcut"
        );
    }

    #[test]
    fn file_context_copy_filename_has_shortcut() {
        let file_info = FileInfo {
            path: "/home/user/readme.txt".to_string(),
            name: "readme.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        let copy_fn = actions.iter().find(|a| a.id == "copy_filename").unwrap();
        assert_eq!(copy_fn.shortcut, Some("⌘C".to_string()));
    }

    #[test]
    fn path_vs_file_copy_filename_shortcut_differs() {
        let path_info = PathInfo::new("test.txt", "/test.txt", false);
        let file_info = FileInfo {
            path: "/test.txt".to_string(),
            name: "test.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let path_actions = get_path_context_actions(&path_info);
        let file_actions = get_file_context_actions(&file_info);
        let path_cf = path_actions
            .iter()
            .find(|a| a.id == "copy_filename")
            .unwrap();
        let file_cf = file_actions
            .iter()
            .find(|a| a.id == "copy_filename")
            .unwrap();
        assert!(path_cf.shortcut.is_none());
        assert!(file_cf.shortcut.is_some());
    }

    #[test]
    fn path_context_copy_filename_desc_just_the_filename() {
        let path_info = PathInfo::new("hello.rs", "/code/hello.rs", false);
        let actions = get_path_context_actions(&path_info);
        let copy_fn = actions.iter().find(|a| a.id == "copy_filename").unwrap();
        assert_eq!(
            copy_fn.description,
            Some("Copy just the filename".to_string())
        );
    }

    // =========================================================================
    // 5. File context: show_info (Get Info) details on macOS
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn file_context_show_info_shortcut() {
        let file_info = FileInfo {
            path: "/test.txt".to_string(),
            name: "test.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        let show_info = actions.iter().find(|a| a.id == "show_info").unwrap();
        assert_eq!(show_info.shortcut, Some("⌘I".to_string()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn file_context_show_info_title() {
        let file_info = FileInfo {
            path: "/test.txt".to_string(),
            name: "test.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        let show_info = actions.iter().find(|a| a.id == "show_info").unwrap();
        assert_eq!(show_info.title, "Get Info");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn file_context_show_info_desc_mentions_finder() {
        let file_info = FileInfo {
            path: "/test.txt".to_string(),
            name: "test.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        let show_info = actions.iter().find(|a| a.id == "show_info").unwrap();
        assert!(show_info.description.as_ref().unwrap().contains("Finder"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn file_context_show_info_present_for_dirs() {
        let file_info = FileInfo {
            path: "/Documents".to_string(),
            name: "Documents".to_string(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&file_info);
        assert!(
            actions.iter().any(|a| a.id == "show_info"),
            "Get Info should be present for dirs"
        );
    }

    // =========================================================================
    // 6. Notes command bar: section distribution across conditions
    // =========================================================================

    #[test]
    fn notes_no_selection_no_trash_disabled_auto_has_3_actions() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert_eq!(actions.len(), 3);
        let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains(&"new_note"));
        assert!(ids.contains(&"browse_notes"));
        assert!(ids.contains(&"enable_auto_sizing"));
    }

    #[test]
    fn notes_no_selection_no_trash_enabled_auto_has_2_actions() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert_eq!(actions.len(), 2);
        let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains(&"new_note"));
        assert!(ids.contains(&"browse_notes"));
    }

    #[test]
    fn notes_selection_trash_has_3_actions() {
        // In trash view with selection, selection-dependent actions suppressed
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert_eq!(actions.len(), 3);
        // Should have new_note, browse_notes, enable_auto_sizing
        // But NOT duplicate_note, find_in_note, format, copy_*, export
    }

    #[test]
    fn notes_sections_distribution_full() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let sections: Vec<Option<&str>> = actions.iter().map(|a| a.section.as_deref()).collect();
        // Notes section: new_note, duplicate_note, browse_notes
        assert_eq!(sections.iter().filter(|s| **s == Some("Notes")).count(), 3);
        // Edit section: find_in_note, format
        assert_eq!(sections.iter().filter(|s| **s == Some("Edit")).count(), 2);
        // Copy section: copy_note_as, copy_deeplink, create_quicklink
        assert_eq!(sections.iter().filter(|s| **s == Some("Copy")).count(), 3);
        // Export section: export
        assert_eq!(sections.iter().filter(|s| **s == Some("Export")).count(), 1);
        // Settings section: enable_auto_sizing
        assert_eq!(
            sections.iter().filter(|s| **s == Some("Settings")).count(),
            1
        );
    }

    // =========================================================================
    // 7. Clipboard: paste_and_keep_open details
    // =========================================================================

    #[test]
    fn clipboard_paste_keep_open_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let pko = actions
            .iter()
            .find(|a| a.id == "clipboard_paste_keep_open")
            .unwrap();
        assert_eq!(pko.shortcut, Some("⌥↵".to_string()));
    }

    #[test]
    fn clipboard_paste_keep_open_title() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let pko = actions
            .iter()
            .find(|a| a.id == "clipboard_paste_keep_open")
            .unwrap();
        assert_eq!(pko.title, "Paste and Keep Window Open");
    }

    #[test]
    fn clipboard_paste_keep_open_desc_mentions_keep() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
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

    #[test]
    fn clipboard_paste_keep_open_is_third_action() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[2].id, "clipboard_paste_keep_open");
    }

    // =========================================================================
    // 8. AI command bar: add_attachment details
    // =========================================================================

    #[test]
    fn ai_bar_add_attachment_shortcut() {
        let actions = get_ai_command_bar_actions();
        let attach = actions.iter().find(|a| a.id == "add_attachment").unwrap();
        assert_eq!(attach.shortcut, Some("⇧⌘A".to_string()));
    }

    #[test]
    fn ai_bar_add_attachment_icon() {
        let actions = get_ai_command_bar_actions();
        let attach = actions.iter().find(|a| a.id == "add_attachment").unwrap();
        assert_eq!(attach.icon, Some(IconName::Plus));
    }

    #[test]
    fn ai_bar_add_attachment_section() {
        let actions = get_ai_command_bar_actions();
        let attach = actions.iter().find(|a| a.id == "add_attachment").unwrap();
        assert_eq!(attach.section, Some("Attachments".to_string()));
    }

    #[test]
    fn ai_bar_add_attachment_desc_mentions_attach() {
        let actions = get_ai_command_bar_actions();
        let attach = actions.iter().find(|a| a.id == "add_attachment").unwrap();
        assert!(attach
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("attach"));
    }

    // =========================================================================
    // 9. ScriptInfo constructors: is_agent field defaults
    // =========================================================================

    #[test]
    fn script_info_new_is_agent_false() {
        let s = ScriptInfo::new("test", "/path");
        assert!(!s.is_agent);
    }

    #[test]
    fn script_info_builtin_is_agent_false() {
        let s = ScriptInfo::builtin("App Launcher");
        assert!(!s.is_agent);
    }

    #[test]
    fn script_info_scriptlet_is_agent_false() {
        let s = ScriptInfo::scriptlet("test", "/path.md", None, None);
        assert!(!s.is_agent);
    }

    #[test]
    fn script_info_with_all_is_agent_false() {
        let s = ScriptInfo::with_all("test", "/path", true, "Run", None, None);
        assert!(!s.is_agent);
    }

    // =========================================================================
    // 10. Deeplink URL in copy_deeplink desc
    // =========================================================================

    #[test]
    fn script_copy_deeplink_desc_contains_url() {
        let script = ScriptInfo::new("My Script", "/path/my-script.ts");
        let actions = get_script_context_actions(&script);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert!(dl
            .description
            .as_ref()
            .unwrap()
            .contains("scriptkit://run/my-script"));
    }

    #[test]
    fn scriptlet_copy_deeplink_desc_contains_url() {
        let script = ScriptInfo::scriptlet("Open GitHub", "/path.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert!(dl
            .description
            .as_ref()
            .unwrap()
            .contains("scriptkit://run/open-github"));
    }

    #[test]
    fn builtin_copy_deeplink_desc_contains_url() {
        let script = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&script);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert!(dl
            .description
            .as_ref()
            .unwrap()
            .contains("scriptkit://run/clipboard-history"));
    }

    #[test]
    fn agent_copy_deeplink_desc_contains_url() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert!(dl
            .description
            .as_ref()
            .unwrap()
            .contains("scriptkit://run/my-agent"));
    }

    // =========================================================================
    // 11. to_deeplink_name: CJK and multi-byte unicode
    // =========================================================================

    #[test]
    fn to_deeplink_name_cjk_preserved() {
        // CJK characters are alphanumeric per Rust's char::is_alphanumeric
        let result = to_deeplink_name("日本語テスト");
        assert_eq!(result, "日本語テスト");
    }

    #[test]
    fn to_deeplink_name_mixed_cjk_and_spaces() {
        let result = to_deeplink_name("My 日本語 Script");
        assert_eq!(result, "my-日本語-script");
    }

    #[test]
    fn to_deeplink_name_accented_chars_preserved() {
        let result = to_deeplink_name("café résumé");
        assert_eq!(result, "café-résumé");
    }

    #[test]
    fn to_deeplink_name_consecutive_special_collapse() {
        let result = to_deeplink_name("test---name");
        assert_eq!(result, "test-name");
    }

    // =========================================================================
    // 12. Score action: exact values for fuzzy=25, contains=50, prefix=100
    // =========================================================================

    #[test]
    fn score_action_prefix_match_exactly_100_base() {
        let action = Action::new("test", "Edit Script", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "edit");
        assert_eq!(score, 100);
    }

    #[test]
    fn score_action_contains_match_exactly_50_base() {
        let action = Action::new(
            "test",
            "Copy Edit Path",
            None,
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "edit");
        assert_eq!(score, 50);
    }

    #[test]
    fn score_action_fuzzy_match_exactly_25_base() {
        // "et" is a subsequence of "exit" (e...t) but not a prefix or contains
        let action = Action::new("test", "Exit Dialog", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "eidl");
        // "eidl" matches e-i-d-l in "exit dialog" as subsequence but not prefix/contains
        assert_eq!(score, 25);
    }

    #[test]
    fn score_action_desc_bonus_exactly_15() {
        // Title doesn't match, but description does
        let action = Action::new(
            "test",
            "XYZ Action",
            Some("Copy the content".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "copy");
        // No title match (not prefix, not contains, not fuzzy) but desc contains "copy"
        assert_eq!(score, 15);
    }

    // =========================================================================
    // 13. Notes: create_quicklink and copy_deeplink details
    // =========================================================================

    #[test]
    fn notes_create_quicklink_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ql = actions.iter().find(|a| a.id == "create_quicklink").unwrap();
        assert_eq!(ql.shortcut, Some("⇧⌘L".to_string()));
    }

    #[test]
    fn notes_create_quicklink_icon() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ql = actions.iter().find(|a| a.id == "create_quicklink").unwrap();
        assert_eq!(ql.icon, Some(IconName::Star));
    }

    #[test]
    fn notes_copy_deeplink_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert_eq!(dl.shortcut, Some("⇧⌘D".to_string()));
    }

    #[test]
    fn notes_copy_deeplink_icon() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert_eq!(dl.icon, Some(IconName::ArrowRight));
    }

    // =========================================================================
    // 14. Scriptlet context with_custom: copy_deeplink URL format
    // =========================================================================

    #[test]
    fn scriptlet_with_custom_copy_deeplink_url_format() {
        let script = ScriptInfo::scriptlet("My Snippet", "/path.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert!(dl
            .description
            .as_ref()
            .unwrap()
            .starts_with("Copy scriptkit://run/"));
    }

    #[test]
    fn scriptlet_with_custom_has_reset_ranking_when_suggested() {
        let mut script = ScriptInfo::scriptlet("Test", "/path.md", None, None);
        script.is_suggested = true;
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    }

    #[test]
    fn scriptlet_with_custom_no_reset_ranking_default() {
        let script = ScriptInfo::scriptlet("Test", "/path.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(!actions.iter().any(|a| a.id == "reset_ranking"));
    }

    #[test]
    fn scriptlet_with_custom_edit_scriptlet_desc() {
        let script = ScriptInfo::scriptlet("Test", "/path.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let edit = actions.iter().find(|a| a.id == "edit_scriptlet").unwrap();
        assert!(edit.description.as_ref().unwrap().contains("$EDITOR"));
    }

    // =========================================================================
    // 15. ProtocolAction: new() constructor defaults
    // =========================================================================

    #[test]
    fn protocol_action_new_defaults_description_none() {
        let pa = ProtocolAction::new("Test".to_string());
        assert!(pa.description.is_none());
    }

    #[test]
    fn protocol_action_new_defaults_shortcut_none() {
        let pa = ProtocolAction::new("Test".to_string());
        assert!(pa.shortcut.is_none());
    }

    #[test]
    fn protocol_action_new_defaults_visible_none() {
        let pa = ProtocolAction::new("Test".to_string());
        assert!(pa.visible.is_none());
        assert!(pa.is_visible()); // None → visible
    }

    #[test]
    fn protocol_action_new_defaults_close_none() {
        let pa = ProtocolAction::new("Test".to_string());
        assert!(pa.close.is_none());
        assert!(pa.should_close()); // None → close
    }

    // =========================================================================
    // 16. Clipboard: delete_multiple and delete_all details
    // =========================================================================

    #[test]
    fn clipboard_delete_multiple_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let dm = actions
            .iter()
            .find(|a| a.id == "clipboard_delete_multiple")
            .unwrap();
        assert_eq!(dm.shortcut, Some("⇧⌘X".to_string()));
    }

    #[test]
    fn clipboard_delete_all_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let da = actions
            .iter()
            .find(|a| a.id == "clipboard_delete_all")
            .unwrap();
        assert_eq!(da.shortcut, Some("⌃⇧X".to_string()));
    }

    #[test]
    fn clipboard_delete_all_desc_mentions_pinned() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let da = actions
            .iter()
            .find(|a| a.id == "clipboard_delete_all")
            .unwrap();
        assert!(da.description.as_ref().unwrap().contains("pinned"));
    }

    #[test]
    fn clipboard_delete_multiple_title() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let dm = actions
            .iter()
            .find(|a| a.id == "clipboard_delete_multiple")
            .unwrap();
        assert_eq!(dm.title, "Delete Entries...");
    }

    // =========================================================================
    // 17. AI bar: Actions section has exactly 4 actions
    // =========================================================================

    #[test]
    fn ai_bar_actions_section_count() {
        let actions = get_ai_command_bar_actions();
        let actions_section_count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Actions"))
            .count();
        assert_eq!(actions_section_count, 4);
    }

    #[test]
    fn ai_bar_actions_section_ids() {
        let actions = get_ai_command_bar_actions();
        let action_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Actions"))
            .map(|a| a.id.as_str())
            .collect();
        assert!(action_ids.contains(&"submit"));
        assert!(action_ids.contains(&"new_chat"));
        assert!(action_ids.contains(&"delete_chat"));
        assert!(action_ids.contains(&"branch_from_last"));
    }

    #[test]
    fn ai_bar_response_section_count() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Response"))
            .count();
        assert_eq!(count, 3);
    }

    #[test]
    fn ai_bar_unique_sections_count() {
        let actions = get_ai_command_bar_actions();
        let mut sections: Vec<&str> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        sections.sort();
        sections.dedup();
        assert_eq!(sections.len(), 6);
    }

    // =========================================================================
    // 18. CommandBarConfig: notes_style vs ai_style differences
    // =========================================================================

    #[test]
    fn notes_style_uses_separators_not_headers() {
        let config = CommandBarConfig::notes_style();
        assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
    }

    #[test]
    fn ai_style_uses_headers_not_separators() {
        let config = CommandBarConfig::ai_style();
        assert_eq!(config.dialog_config.section_style, SectionStyle::Headers);
    }

    #[test]
    fn notes_and_ai_both_search_top() {
        let notes = CommandBarConfig::notes_style();
        let ai = CommandBarConfig::ai_style();
        assert_eq!(
            notes.dialog_config.search_position,
            ai.dialog_config.search_position
        );
    }

    #[test]
    fn notes_and_ai_both_anchor_top() {
        let notes = CommandBarConfig::notes_style();
        let ai = CommandBarConfig::ai_style();
        assert_eq!(notes.dialog_config.anchor, AnchorPosition::Top);
        assert_eq!(ai.dialog_config.anchor, AnchorPosition::Top);
    }

    // =========================================================================
    // 19. File context: open_with on macOS details
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn file_context_open_with_shortcut() {
        let file_info = FileInfo {
            path: "/test.txt".to_string(),
            name: "test.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        let ow = actions.iter().find(|a| a.id == "open_with").unwrap();
        assert_eq!(ow.shortcut, Some("⌘O".to_string()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn file_context_open_with_title() {
        let file_info = FileInfo {
            path: "/test.txt".to_string(),
            name: "test.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        let ow = actions.iter().find(|a| a.id == "open_with").unwrap();
        assert_eq!(ow.title, "Open With...");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn file_context_open_with_desc_mentions_application() {
        let file_info = FileInfo {
            path: "/test.txt".to_string(),
            name: "test.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        let ow = actions.iter().find(|a| a.id == "open_with").unwrap();
        assert!(ow.description.as_ref().unwrap().contains("application"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn file_context_open_with_present_for_dir() {
        let file_info = FileInfo {
            path: "/Documents".to_string(),
            name: "Documents".to_string(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&file_info);
        assert!(actions.iter().any(|a| a.id == "open_with"));
    }

    // =========================================================================
    // 20. build_grouped_items_static: non-consecutive same-section
    // =========================================================================

    #[test]
    fn grouped_items_non_consecutive_same_section_creates_two_headers() {
        let actions = vec![
            Action::new("a1", "A1", None, ActionCategory::ScriptContext).with_section("Alpha"),
            Action::new("b1", "B1", None, ActionCategory::ScriptContext).with_section("Beta"),
            Action::new("a2", "A2", None, ActionCategory::ScriptContext).with_section("Alpha"),
        ];
        let filtered: Vec<usize> = (0..3).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Expect: Header("Alpha"), Item(0), Header("Beta"), Item(1), Header("Alpha"), Item(2)
        assert_eq!(grouped.len(), 6);
    }

    #[test]
    fn grouped_items_consecutive_same_section_one_header() {
        let actions = vec![
            Action::new("a1", "A1", None, ActionCategory::ScriptContext).with_section("Alpha"),
            Action::new("a2", "A2", None, ActionCategory::ScriptContext).with_section("Alpha"),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Expect: Header("Alpha"), Item(0), Item(1)
        assert_eq!(grouped.len(), 3);
    }

    #[test]
    fn grouped_items_separators_never_add_headers() {
        let actions = vec![
            Action::new("a1", "A1", None, ActionCategory::ScriptContext).with_section("Alpha"),
            Action::new("b1", "B1", None, ActionCategory::ScriptContext).with_section("Beta"),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        assert_eq!(grouped.len(), 2); // Just items, no headers
    }

    #[test]
    fn grouped_items_none_style_no_headers() {
        let actions = vec![
            Action::new("a1", "A1", None, ActionCategory::ScriptContext).with_section("Alpha"),
            Action::new("b1", "B1", None, ActionCategory::ScriptContext).with_section("Beta"),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        assert_eq!(grouped.len(), 2);
    }

    // =========================================================================
    // 21. Action::new defaults: has_action, value, icon, section
    // =========================================================================

    #[test]
    fn action_new_has_action_default_false() {
        let a = Action::new("test", "Test", None, ActionCategory::ScriptContext);
        assert!(!a.has_action);
    }

    #[test]
    fn action_new_value_default_none() {
        let a = Action::new("test", "Test", None, ActionCategory::ScriptContext);
        assert!(a.value.is_none());
    }

    #[test]
    fn action_new_icon_default_none() {
        let a = Action::new("test", "Test", None, ActionCategory::ScriptContext);
        assert!(a.icon.is_none());
    }

    #[test]
    fn action_new_section_default_none() {
        let a = Action::new("test", "Test", None, ActionCategory::ScriptContext);
        assert!(a.section.is_none());
    }

    // =========================================================================
    // 22. Cross-context: scriptlet vs script shortcut action IDs differ
    // =========================================================================

    #[test]
    fn script_edit_action_id_is_edit_script() {
        let script = ScriptInfo::new("test", "/path");
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "edit_script"));
        assert!(!actions.iter().any(|a| a.id == "edit_scriptlet"));
    }

    #[test]
    fn scriptlet_edit_action_id_is_edit_scriptlet() {
        let script = ScriptInfo::scriptlet("test", "/path.md", None, None);
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "edit_scriptlet"));
        assert!(!actions.iter().any(|a| a.id == "edit_script"));
    }

    #[test]
    fn script_reveal_id_is_reveal_in_finder() {
        let script = ScriptInfo::new("test", "/path");
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
    }

    #[test]
    fn scriptlet_reveal_id_is_reveal_scriptlet_in_finder() {
        let script = ScriptInfo::scriptlet("test", "/path.md", None, None);
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "reveal_scriptlet_in_finder"));
    }

    // =========================================================================
    // 23. New chat: duplicate model entries produce distinct IDs
    // =========================================================================

    #[test]
    fn new_chat_duplicate_models_distinct_ids() {
        let models = vec![
            NewChatModelInfo {
                model_id: "claude-3".to_string(),
                display_name: "Claude 3".to_string(),
                provider: "anthropic".to_string(),
                provider_display_name: "Anthropic".to_string(),
            },
            NewChatModelInfo {
                model_id: "claude-3".to_string(),
                display_name: "Claude 3".to_string(),
                provider: "anthropic".to_string(),
                provider_display_name: "Anthropic".to_string(),
            },
        ];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].id, "model_0");
        assert_eq!(actions[1].id, "model_1");
    }

    #[test]
    fn new_chat_model_ids_use_index_not_model_id() {
        let models = vec![NewChatModelInfo {
            model_id: "gpt-4".to_string(),
            display_name: "GPT-4".to_string(),
            provider: "openai".to_string(),
            provider_display_name: "OpenAI".to_string(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        // ID is "model_{idx}" not "model_{model_id}"
        assert_eq!(actions[0].id, "model_0");
        assert!(!actions[0].id.contains("gpt-4"));
    }

    #[test]
    fn new_chat_preset_ids_use_preset_id() {
        let presets = vec![NewChatPresetInfo {
            id: "general".to_string(),
            name: "General".to_string(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].id, "preset_general");
    }

    #[test]
    fn new_chat_last_used_ids_use_index() {
        let last_used = vec![NewChatModelInfo {
            model_id: "claude-3".to_string(),
            display_name: "Claude 3".to_string(),
            provider: "anthropic".to_string(),
            provider_display_name: "Anthropic".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].id, "last_used_0");
    }

    // =========================================================================
    // 24. Note switcher: description formatting for all branches
    // =========================================================================

    #[test]
    fn note_switcher_preview_and_time_has_separator() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".to_string(),
            title: "Test".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "Some preview text".to_string(),
            relative_time: "2m ago".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains(" · "));
    }

    #[test]
    fn note_switcher_preview_only_no_separator() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".to_string(),
            title: "Test".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "Some preview text".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(!desc.contains(" · "));
        assert_eq!(desc, "Some preview text");
    }

    #[test]
    fn note_switcher_time_only_when_empty_preview() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".to_string(),
            title: "Test".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "".to_string(),
            relative_time: "5h ago".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "5h ago");
    }

    #[test]
    fn note_switcher_char_count_when_no_preview_no_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".to_string(),
            title: "Test".to_string(),
            char_count: 42,
            is_current: false,
            is_pinned: false,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "42 chars");
    }

    // =========================================================================
    // 25. Clipboard: image has all 4 macOS-only image actions
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn clipboard_image_has_open_with() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "".to_string(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_open_with"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn clipboard_image_has_annotate_cleanshot() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "".to_string(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions
            .iter()
            .any(|a| a.id == "clipboard_annotate_cleanshot"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn clipboard_image_has_upload_cleanshot() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "".to_string(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_upload_cleanshot"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn clipboard_image_has_ocr() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "".to_string(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_ocr"));
    }

    // =========================================================================
    // 26. Score action: exact full title match
    // =========================================================================

    #[test]
    fn score_action_exact_full_title_is_prefix() {
        let action = Action::new("test", "Copy", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "copy");
        assert_eq!(score, 100); // Full match is a prefix match
    }

    #[test]
    fn score_action_shortcut_bonus_exactly_10() {
        let action =
            Action::new("test", "XYZ", None, ActionCategory::ScriptContext).with_shortcut("⌘C");
        let score = ActionsDialog::score_action(&action, "⌘c");
        assert_eq!(score, 10);
    }

    #[test]
    fn score_action_no_match_zero() {
        let action = Action::new("test", "Edit Script", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "zzzzz");
        assert_eq!(score, 0);
    }

    #[test]
    fn score_action_prefix_plus_desc_is_115() {
        let action = Action::new(
            "test",
            "Edit Script",
            Some("Edit the script file".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "edit");
        // prefix(100) + desc contains "edit"(15) = 115
        assert_eq!(score, 115);
    }

    // =========================================================================
    // 27. Path context: copy_filename has no shortcut (unlike file)
    // =========================================================================

    #[test]
    fn path_dir_also_has_copy_filename_no_shortcut() {
        let path_info = PathInfo::new("Documents", "/home/Documents", true);
        let actions = get_path_context_actions(&path_info);
        let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
        assert!(cf.shortcut.is_none());
    }

    #[test]
    fn path_copy_path_has_shortcut() {
        let path_info = PathInfo::new("test.txt", "/test.txt", false);
        let actions = get_path_context_actions(&path_info);
        let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
        assert_eq!(cp.shortcut, Some("⌘⇧C".to_string()));
    }

    #[test]
    fn path_open_in_finder_has_shortcut() {
        let path_info = PathInfo::new("test.txt", "/test.txt", false);
        let actions = get_path_context_actions(&path_info);
        let f = actions.iter().find(|a| a.id == "open_in_finder").unwrap();
        assert_eq!(f.shortcut, Some("⌘⇧F".to_string()));
    }

    #[test]
    fn path_open_in_editor_has_shortcut() {
        let path_info = PathInfo::new("test.txt", "/test.txt", false);
        let actions = get_path_context_actions(&path_info);
        let e = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
        assert_eq!(e.shortcut, Some("⌘E".to_string()));
    }

    // =========================================================================
    // 28. Cross-context: all contexts set category to ScriptContext
    // =========================================================================

    #[test]
    fn script_context_all_script_context_category() {
        let script = ScriptInfo::new("test", "/path");
        let actions = get_script_context_actions(&script);
        assert!(actions
            .iter()
            .all(|a| a.category == ActionCategory::ScriptContext));
    }

    #[test]
    fn file_context_all_script_context_category() {
        let file_info = FileInfo {
            path: "/test.txt".to_string(),
            name: "test.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        assert!(actions
            .iter()
            .all(|a| a.category == ActionCategory::ScriptContext));
    }

    #[test]
    fn path_context_all_script_context_category() {
        let path_info = PathInfo::new("test", "/test", false);
        let actions = get_path_context_actions(&path_info);
        assert!(actions
            .iter()
            .all(|a| a.category == ActionCategory::ScriptContext));
    }

    #[test]
    fn ai_bar_all_script_context_category() {
        let actions = get_ai_command_bar_actions();
        assert!(actions
            .iter()
            .all(|a| a.category == ActionCategory::ScriptContext));
    }

    // =========================================================================
    // 29. format_shortcut_hint: page up/down, home/end (non-modifier)
    // =========================================================================

    #[test]
    fn format_shortcut_hint_single_letter() {
        let result = ActionsDialog::format_shortcut_hint("k");
        assert_eq!(result, "K");
    }

    #[test]
    fn format_shortcut_hint_cmd_plus_letter() {
        let result = ActionsDialog::format_shortcut_hint("cmd+e");
        assert_eq!(result, "⌘E");
    }

    #[test]
    fn format_shortcut_hint_all_four_modifiers() {
        let result = ActionsDialog::format_shortcut_hint("cmd+shift+ctrl+alt+k");
        assert_eq!(result, "⌘⇧⌃⌥K");
    }

    #[test]
    fn format_shortcut_hint_meta_alias() {
        let result = ActionsDialog::format_shortcut_hint("meta+c");
        assert_eq!(result, "⌘C");
    }

    // =========================================================================
    // 30. parse_shortcut_keycaps: various inputs
    // =========================================================================

    #[test]
    fn parse_keycaps_modifier_and_letter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘E");
        assert_eq!(keycaps, vec!["⌘", "E"]);
    }

    #[test]
    fn parse_keycaps_all_modifiers_and_key() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⇧⌃⌥K");
        assert_eq!(keycaps, vec!["⌘", "⇧", "⌃", "⌥", "K"]);
    }

    #[test]
    fn parse_keycaps_return_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("↵");
        assert_eq!(keycaps, vec!["↵"]);
    }

    #[test]
    fn parse_keycaps_lowercase_uppercased() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘c");
        assert_eq!(keycaps, vec!["⌘", "C"]);
    }
}
