//! Batch 9: Dialog builtin action validation tests
//!
//! Focuses on areas not exhaustively covered in batches 1-8:
//!
//! 1. AI command bar expanded actions (branch_from_last, export_markdown, toggle_shortcuts_help)
//! 2. File context macOS-specific actions (quick_look, open_with, show_info)
//! 3. Clipboard macOS-specific actions (quick_look, open_with, annotate/upload)
//! 4. CommandBarConfig struct field validation
//! 5. format_shortcut_hint alias coverage (meta, super, opt, esc, return, arrowdown/left/right)
//! 6. Cross-context shortcut symbol consistency (all shortcuts use symbol chars)
//! 7. Action verb formatting with special characters in names
//! 8. Notes command bar conditional section groups (section strings)
//! 9. ScriptInfo mixed agent+scriptlet flag precedence
//! 10. Clipboard save_snippet/save_file always present for both text and image
//! 11. AI command bar section completeness (12 actions across 6 sections)
//! 12. Path context open_in_finder/editor/terminal descriptions
//! 13. Note switcher empty notes placeholder action
//! 14. New chat action icon/section consistency
//! 15. Score_action with multi-word queries
//! 16. build_grouped_items_static with Headers style multi-section
//! 17. coerce_action_selection with single-item rows
//! 18. parse_shortcut_keycaps with multi-char sequences
//! 19. Deeplink name with Unicode characters preservation
//! 20. File context open title includes quoted name
//! 21. Clipboard share/attach_to_ai always present
//! 22. Notes command bar icon name validation
//! 23. Chat context continue_in_chat always present
//! 24. Scriptlet context copy_content always present
//! 25. Agent actions: has copy_content, edit title says "Edit Agent"
//! 26. Cross-context action count stability
//! 27. Action with_shortcut_opt preserves existing fields
//! 28. ActionsDialogConfig default values
//! 29. SectionStyle and SearchPosition enum values
//! 30. Clipboard action IDs all prefixed with "clipboard_"

#[cfg(test)]
mod tests {
    use crate::actions::builders::{
        get_ai_command_bar_actions, get_chat_context_actions,
        get_clipboard_history_context_actions, get_file_context_actions, get_new_chat_actions,
        get_note_switcher_actions, get_notes_command_bar_actions, get_path_context_actions,
        get_script_context_actions, get_scriptlet_context_actions_with_custom, to_deeplink_name,
        ChatPromptInfo, ClipboardEntryInfo, NewChatModelInfo, NewChatPresetInfo,
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
    #[allow(unused_imports)]
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
        app: Option<&str>,
    ) -> ClipboardEntryInfo {
        ClipboardEntryInfo {
            id: "entry-1".to_string(),
            content_type,
            pinned,
            preview: "test preview".to_string(),
            image_dimensions: if content_type == ContentType::Image {
                Some((640, 480))
            } else {
                None
            },
            frontmost_app_name: app.map(|s| s.to_string()),
        }
    }

    // ============================================================
    // 1. AI command bar expanded actions (12 actions, 6 sections)
    // ============================================================

    #[test]
    fn ai_command_bar_has_exactly_12_actions() {
        let actions = get_ai_command_bar_actions();
        assert_eq!(actions.len(), 12, "AI command bar should have 12 actions");
    }

    #[test]
    fn ai_command_bar_has_branch_from_last() {
        let actions = get_ai_command_bar_actions();
        let branch = find_action(&actions, "branch_from_last");
        assert!(branch.is_some(), "Should have branch_from_last action");
        let branch = branch.unwrap();
        assert_eq!(branch.section, Some("Actions".to_string()));
        assert!(
            branch.description.as_ref().unwrap().contains("branch"),
            "Description should mention branch"
        );
    }

    #[test]
    fn ai_command_bar_has_export_markdown() {
        let actions = get_ai_command_bar_actions();
        let export = find_action(&actions, "export_markdown");
        assert!(export.is_some(), "Should have export_markdown action");
        let export = export.unwrap();
        assert_eq!(export.section, Some("Export".to_string()));
        assert!(export.icon.is_some());
    }

    #[test]
    fn ai_command_bar_has_toggle_shortcuts_help() {
        let actions = get_ai_command_bar_actions();
        let help = find_action(&actions, "toggle_shortcuts_help");
        assert!(help.is_some(), "Should have toggle_shortcuts_help action");
        let help = help.unwrap();
        assert_eq!(help.section, Some("Help".to_string()));
    }

    #[test]
    fn ai_command_bar_has_six_sections() {
        let actions = get_ai_command_bar_actions();
        let sections: HashSet<_> = actions.iter().filter_map(|a| a.section.as_ref()).collect();
        assert_eq!(sections.len(), 6, "Should have 6 distinct sections");
        assert!(sections.contains(&"Response".to_string()));
        assert!(sections.contains(&"Actions".to_string()));
        assert!(sections.contains(&"Attachments".to_string()));
        assert!(sections.contains(&"Export".to_string()));
        assert!(sections.contains(&"Help".to_string()));
        assert!(sections.contains(&"Settings".to_string()));
    }

    #[test]
    fn ai_command_bar_response_section_has_3_items() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Response"))
            .count();
        assert_eq!(count, 3, "Response section should have 3 items");
    }

    #[test]
    fn ai_command_bar_actions_section_has_4_items() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Actions"))
            .count();
        assert_eq!(
            count, 4,
            "Actions section should have 4 items (submit, new_chat, delete_chat, branch_from_last)"
        );
    }

    #[test]
    fn ai_command_bar_all_actions_have_icons() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                action.icon.is_some(),
                "AI action '{}' should have an icon",
                action.id
            );
        }
    }

    #[test]
    fn ai_command_bar_all_actions_have_sections() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                action.section.is_some(),
                "AI action '{}' should have a section",
                action.id
            );
        }
    }

    #[test]
    fn ai_command_bar_id_uniqueness() {
        let actions = get_ai_command_bar_actions();
        let ids: HashSet<_> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(
            ids.len(),
            actions.len(),
            "All AI action IDs should be unique"
        );
    }

    // ============================================================
    // 2. File context macOS-specific actions
    // ============================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn file_context_file_has_quick_look() {
        let file_info = FileInfo {
            path: "/test/photo.jpg".to_string(),
            name: "photo.jpg".to_string(),
            file_type: FileType::Image,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        let ql = find_action(&actions, "quick_look");
        assert!(ql.is_some(), "File should have quick_look on macOS");
        assert_eq!(ql.unwrap().shortcut.as_deref(), Some("⌘Y"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn file_context_dir_no_quick_look() {
        let file_info = FileInfo {
            path: "/test/folder".to_string(),
            name: "folder".to_string(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&file_info);
        assert!(
            find_action(&actions, "quick_look").is_none(),
            "Directory should NOT have quick_look"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn file_context_has_open_with() {
        let file_info = FileInfo {
            path: "/test/doc.txt".to_string(),
            name: "doc.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        let ow = find_action(&actions, "open_with");
        assert!(ow.is_some(), "File should have open_with on macOS");
        assert_eq!(ow.unwrap().shortcut.as_deref(), Some("⌘O"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn file_context_has_show_info() {
        let file_info = FileInfo {
            path: "/test/doc.txt".to_string(),
            name: "doc.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        let si = find_action(&actions, "show_info");
        assert!(si.is_some(), "File should have show_info on macOS");
        assert_eq!(si.unwrap().shortcut.as_deref(), Some("⌘I"));
        assert!(
            si.unwrap()
                .description
                .as_ref()
                .unwrap()
                .contains("information"),
            "show_info description should mention information"
        );
    }

    // ============================================================
    // 3. Clipboard macOS-specific actions
    // ============================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn clipboard_image_has_open_with_on_macos() {
        let entry = make_clipboard_entry(ContentType::Image, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(
            find_action(&actions, "clipboard_open_with").is_some(),
            "Image entry should have open_with on macOS"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn clipboard_image_has_annotate_cleanshot_on_macos() {
        let entry = make_clipboard_entry(ContentType::Image, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let annotate = find_action(&actions, "clipboard_annotate_cleanshot");
        assert!(
            annotate.is_some(),
            "Image entry should have annotate_cleanshot on macOS"
        );
        assert_eq!(annotate.unwrap().shortcut.as_deref(), Some("⇧⌘A"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn clipboard_image_has_upload_cleanshot_on_macos() {
        let entry = make_clipboard_entry(ContentType::Image, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let upload = find_action(&actions, "clipboard_upload_cleanshot");
        assert!(
            upload.is_some(),
            "Image entry should have upload_cleanshot on macOS"
        );
        assert_eq!(upload.unwrap().shortcut.as_deref(), Some("⇧⌘U"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn clipboard_text_no_open_with_on_macos() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(
            find_action(&actions, "clipboard_open_with").is_none(),
            "Text entry should NOT have open_with"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn clipboard_has_quick_look_on_macos() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let ql = find_action(&actions, "clipboard_quick_look");
        assert!(ql.is_some(), "Clipboard should have quick_look on macOS");
        assert_eq!(ql.unwrap().shortcut.as_deref(), Some("␣"));
    }

    // ============================================================
    // 4. CommandBarConfig struct field validation
    // ============================================================

    #[test]
    fn command_bar_config_default_close_flags_all_true() {
        let config = CommandBarConfig::default();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    #[test]
    fn command_bar_config_ai_style_search_top() {
        let config = CommandBarConfig::ai_style();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
        assert_eq!(config.dialog_config.section_style, SectionStyle::Headers);
        assert!(config.dialog_config.show_icons);
        assert!(config.dialog_config.show_footer);
    }

    #[test]
    fn command_bar_config_main_menu_search_bottom() {
        let config = CommandBarConfig::main_menu_style();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Bottom);
        assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
        assert!(!config.dialog_config.show_icons);
        assert!(!config.dialog_config.show_footer);
    }

    #[test]
    fn command_bar_config_no_search_hidden() {
        let config = CommandBarConfig::no_search();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Hidden);
    }

    #[test]
    fn command_bar_config_notes_style_icons_and_footer() {
        let config = CommandBarConfig::notes_style();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
        assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
        assert!(config.dialog_config.show_icons);
        assert!(config.dialog_config.show_footer);
    }

    // ============================================================
    // 5. format_shortcut_hint alias coverage
    // ============================================================

    #[test]
    fn format_shortcut_hint_meta_maps_to_cmd_symbol() {
        assert_eq!(ActionsDialog::format_shortcut_hint("meta+c"), "⌘C");
    }

    #[test]
    fn format_shortcut_hint_super_maps_to_cmd_symbol() {
        assert_eq!(ActionsDialog::format_shortcut_hint("super+v"), "⌘V");
    }

    #[test]
    fn format_shortcut_hint_opt_maps_to_alt_symbol() {
        assert_eq!(ActionsDialog::format_shortcut_hint("opt+x"), "⌥X");
    }

    #[test]
    fn format_shortcut_hint_option_maps_to_alt_symbol() {
        assert_eq!(ActionsDialog::format_shortcut_hint("option+z"), "⌥Z");
    }

    #[test]
    fn format_shortcut_hint_control_maps_to_ctrl_symbol() {
        assert_eq!(ActionsDialog::format_shortcut_hint("control+a"), "⌃A");
    }

    #[test]
    fn format_shortcut_hint_return_maps_to_enter_symbol() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+return"), "⌘↵");
    }

    #[test]
    fn format_shortcut_hint_esc_maps_to_escape_symbol() {
        assert_eq!(ActionsDialog::format_shortcut_hint("esc"), "⎋");
    }

    #[test]
    fn format_shortcut_hint_tab_maps_to_symbol() {
        assert_eq!(ActionsDialog::format_shortcut_hint("tab"), "⇥");
    }

    #[test]
    fn format_shortcut_hint_arrowdown_maps_to_down() {
        assert_eq!(ActionsDialog::format_shortcut_hint("arrowdown"), "↓");
    }

    #[test]
    fn format_shortcut_hint_arrowleft_maps_to_left() {
        assert_eq!(ActionsDialog::format_shortcut_hint("arrowleft"), "←");
    }

    #[test]
    fn format_shortcut_hint_arrowright_maps_to_right() {
        assert_eq!(ActionsDialog::format_shortcut_hint("arrowright"), "→");
    }

    #[test]
    fn format_shortcut_hint_arrowup_maps_to_up() {
        assert_eq!(ActionsDialog::format_shortcut_hint("arrowup"), "↑");
    }

    #[test]
    fn format_shortcut_hint_command_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("command+k"), "⌘K");
    }

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

    #[test]
    fn new_chat_last_used_has_bolt_icon() {
        let last_used = vec![NewChatModelInfo {
            model_id: "claude-3".to_string(),
            display_name: "Claude 3".to_string(),
            provider: "anthropic".to_string(),
            provider_display_name: "Anthropic".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        let action = &actions[0];
        assert_eq!(action.icon, Some(IconName::BoltFilled));
        assert_eq!(action.section.as_deref(), Some("Last Used Settings"));
    }

    #[test]
    fn new_chat_presets_use_custom_icon() {
        let presets = vec![NewChatPresetInfo {
            id: "general".to_string(),
            name: "General".to_string(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        let action = &actions[0];
        assert_eq!(action.icon, Some(IconName::Star));
        assert_eq!(action.section.as_deref(), Some("Presets"));
        assert!(
            action.description.is_none(),
            "Presets should have no description"
        );
    }

    #[test]
    fn new_chat_models_use_settings_icon() {
        let models = vec![NewChatModelInfo {
            model_id: "gpt-4".to_string(),
            display_name: "GPT-4".to_string(),
            provider: "openai".to_string(),
            provider_display_name: "OpenAI".to_string(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        let action = &actions[0];
        assert_eq!(action.icon, Some(IconName::Settings));
        assert_eq!(action.section.as_deref(), Some("Models"));
        assert_eq!(
            action.description.as_deref(),
            Some("OpenAI"),
            "Model should show provider display name"
        );
    }

    #[test]
    fn new_chat_empty_inputs_empty_output() {
        let actions = get_new_chat_actions(&[], &[], &[]);
        assert!(actions.is_empty());
    }

    #[test]
    fn new_chat_all_sections_ordered() {
        let last_used = vec![NewChatModelInfo {
            model_id: "lu".to_string(),
            display_name: "LU".to_string(),
            provider: "p".to_string(),
            provider_display_name: "P".to_string(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "pr".to_string(),
            name: "PR".to_string(),
            icon: IconName::File,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m".to_string(),
            display_name: "M".to_string(),
            provider: "p".to_string(),
            provider_display_name: "P".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        assert_eq!(actions.len(), 3);
        assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
        assert_eq!(actions[1].section.as_deref(), Some("Presets"));
        assert_eq!(actions[2].section.as_deref(), Some("Models"));
    }

    // ============================================================
    // 15. Score_action with multi-word queries
    // ============================================================

    #[test]
    fn score_action_multi_word_prefix() {
        let action = Action::new(
            "edit_script",
            "Edit Script",
            Some("Open in editor".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "edit s");
        assert!(
            score >= 100,
            "Multi-word prefix should score >= 100, got {}",
            score
        );
    }

    #[test]
    fn score_action_multi_word_contains() {
        let action = Action::new(
            "copy_path",
            "Copy Path",
            Some("Copy the full path".to_string()),
            ActionCategory::ScriptContext,
        );
        // "path" is not a prefix of "Copy Path" but it is contained
        let score = ActionsDialog::score_action(&action, "path");
        assert!(
            score >= 50,
            "'path' should match contains on 'copy path', got {}",
            score
        );
    }

    #[test]
    fn score_action_description_only_match() {
        let action = Action::new(
            "reveal",
            "Reveal in Finder",
            Some("Show the file in your filesystem browser".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "filesystem");
        assert_eq!(
            score, 15,
            "Description-only match should score exactly 15, got {}",
            score
        );
    }

    #[test]
    fn score_action_no_match_returns_zero() {
        let action = Action::new(
            "test",
            "Test Action",
            Some("A test".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "zzzzz");
        assert_eq!(score, 0, "No match should return 0");
    }

    #[test]
    fn score_action_shortcut_match_bonus() {
        let action = Action::new("test", "Test Action", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘E");
        let score = ActionsDialog::score_action(&action, "⌘e");
        assert!(
            score >= 10,
            "Shortcut match should add >=10 bonus, got {}",
            score
        );
    }

    // ============================================================
    // 16. build_grouped_items_static with Headers style
    // ============================================================

    #[test]
    fn build_grouped_items_headers_inserts_section_headers() {
        let actions = vec![
            make_action("a1", "Action 1", Some("Group A")),
            make_action("a2", "Action 2", Some("Group A")),
            make_action("b1", "Action 3", Some("Group B")),
        ];
        let filtered: Vec<usize> = (0..3).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Should be: Header("Group A"), Item(0), Item(1), Header("Group B"), Item(2)
        assert_eq!(grouped.len(), 5);
        assert!(matches!(&grouped[0], GroupedActionItem::SectionHeader(s) if s == "Group A"));
        assert!(matches!(&grouped[1], GroupedActionItem::Item(0)));
        assert!(matches!(&grouped[2], GroupedActionItem::Item(1)));
        assert!(matches!(&grouped[3], GroupedActionItem::SectionHeader(s) if s == "Group B"));
        assert!(matches!(&grouped[4], GroupedActionItem::Item(2)));
    }

    #[test]
    fn build_grouped_items_separators_no_headers() {
        let actions = vec![
            make_action("a1", "Action 1", Some("Group A")),
            make_action("b1", "Action 2", Some("Group B")),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        // Separators style should NOT insert section headers
        assert_eq!(grouped.len(), 2);
        assert!(matches!(&grouped[0], GroupedActionItem::Item(0)));
        assert!(matches!(&grouped[1], GroupedActionItem::Item(1)));
    }

    #[test]
    fn build_grouped_items_none_no_headers() {
        let actions = vec![
            make_action("a1", "Action 1", Some("Group A")),
            make_action("b1", "Action 2", Some("Group B")),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        assert_eq!(grouped.len(), 2);
    }

    #[test]
    fn build_grouped_items_empty_returns_empty() {
        let actions: Vec<Action> = vec![];
        let filtered: Vec<usize> = vec![];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(grouped.is_empty());
    }

    #[test]
    fn build_grouped_items_same_section_no_duplicate_header() {
        let actions = vec![
            make_action("a1", "A1", Some("Same")),
            make_action("a2", "A2", Some("Same")),
            make_action("a3", "A3", Some("Same")),
        ];
        let filtered: Vec<usize> = (0..3).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 1, "Same section should produce only 1 header");
    }

    // ============================================================
    // 17. coerce_action_selection edge cases
    // ============================================================

    #[test]
    fn coerce_action_selection_single_item() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    }

    #[test]
    fn coerce_action_selection_single_header() {
        let rows = vec![GroupedActionItem::SectionHeader("Test".to_string())];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn coerce_action_selection_header_then_item() {
        let rows = vec![
            GroupedActionItem::SectionHeader("S".to_string()),
            GroupedActionItem::Item(0),
        ];
        // Landing on header should move to item
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn coerce_action_selection_item_then_header() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("S".to_string()),
        ];
        // Landing on header should search up to item
        assert_eq!(coerce_action_selection(&rows, 1), Some(0));
    }

    #[test]
    fn coerce_action_selection_out_of_bounds_clamped() {
        let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
        assert_eq!(
            coerce_action_selection(&rows, 999),
            Some(1),
            "Out of bounds should clamp to last"
        );
    }

    // ============================================================
    // 18. parse_shortcut_keycaps sequences
    // ============================================================

    #[test]
    fn parse_keycaps_modifier_and_letter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘E");
        assert_eq!(keycaps, vec!["⌘", "E"]);
    }

    #[test]
    fn parse_keycaps_two_modifiers_and_letter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⇧C");
        assert_eq!(keycaps, vec!["⌘", "⇧", "C"]);
    }

    #[test]
    fn parse_keycaps_enter_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("↵");
        assert_eq!(keycaps, vec!["↵"]);
    }

    #[test]
    fn parse_keycaps_cmd_enter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘↵");
        assert_eq!(keycaps, vec!["⌘", "↵"]);
    }

    #[test]
    fn parse_keycaps_escape() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⎋");
        assert_eq!(keycaps, vec!["⎋"]);
    }

    #[test]
    fn parse_keycaps_space() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("␣");
        assert_eq!(keycaps, vec!["␣"]);
    }

    #[test]
    fn parse_keycaps_arrows() {
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("↑"), vec!["↑"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("↓"), vec!["↓"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("←"), vec!["←"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("→"), vec!["→"]);
    }

    #[test]
    fn parse_keycaps_all_four_modifiers() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⌃⌥⇧X");
        assert_eq!(keycaps, vec!["⌘", "⌃", "⌥", "⇧", "X"]);
    }

    #[test]
    fn parse_keycaps_lowercase_uppercased() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘a");
        assert_eq!(keycaps, vec!["⌘", "A"]);
    }

    // ============================================================
    // 19. Deeplink name edge cases
    // ============================================================

    #[test]
    fn deeplink_name_preserves_unicode_alphanumeric() {
        assert_eq!(to_deeplink_name("café"), "café");
    }

    #[test]
    fn deeplink_name_numbers_preserved() {
        assert_eq!(to_deeplink_name("script123"), "script123");
    }

    #[test]
    fn deeplink_name_all_special_returns_empty() {
        assert_eq!(to_deeplink_name("@#$%^&"), "");
    }

    #[test]
    fn deeplink_name_mixed_case_lowered() {
        assert_eq!(to_deeplink_name("Hello World"), "hello-world");
    }

    #[test]
    fn deeplink_name_consecutive_specials_collapsed() {
        assert_eq!(to_deeplink_name("a---b"), "a-b");
    }

    // ============================================================
    // 20. File context open title includes quoted name
    // ============================================================

    #[test]
    fn file_context_open_title_includes_filename() {
        let file_info = FileInfo {
            path: "/test/report.pdf".to_string(),
            name: "report.pdf".to_string(),
            file_type: FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        let open = find_action(&actions, "open_file").unwrap();
        assert!(
            open.title.contains("report.pdf"),
            "Open title should include filename: {}",
            open.title
        );
        assert!(
            open.title.contains('"'),
            "Open title should quote the filename"
        );
    }

    #[test]
    fn file_context_dir_title_includes_dirname() {
        let file_info = FileInfo {
            path: "/test/Documents".to_string(),
            name: "Documents".to_string(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&file_info);
        let open = find_action(&actions, "open_directory").unwrap();
        assert!(
            open.title.contains("Documents"),
            "Open title should include dirname: {}",
            open.title
        );
    }

    // ============================================================
    // 21. Chat context continue_in_chat always present
    // ============================================================

    #[test]
    fn chat_context_continue_in_chat_always_present() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(
            find_action(&actions, "continue_in_chat").is_some(),
            "continue_in_chat should always be present"
        );
    }

    #[test]
    fn chat_context_continue_in_chat_shortcut() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let action = find_action(&actions, "continue_in_chat").unwrap();
        assert_eq!(action.shortcut.as_deref(), Some("⌘↵"));
    }

    #[test]
    fn chat_context_copy_response_only_when_has_response() {
        let no_response = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let with_response = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: true,
        };
        let actions_no = get_chat_context_actions(&no_response);
        let actions_yes = get_chat_context_actions(&with_response);
        assert!(find_action(&actions_no, "copy_response").is_none());
        assert!(find_action(&actions_yes, "copy_response").is_some());
    }

    #[test]
    fn chat_context_clear_conversation_only_when_has_messages() {
        let no_messages = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: true,
        };
        let with_messages = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: true,
        };
        let actions_no = get_chat_context_actions(&no_messages);
        let actions_yes = get_chat_context_actions(&with_messages);
        assert!(find_action(&actions_no, "clear_conversation").is_none());
        assert!(find_action(&actions_yes, "clear_conversation").is_some());
    }

    // ============================================================
    // 22. Scriptlet context copy_content always present
    // ============================================================

    #[test]
    fn scriptlet_context_has_copy_content() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(
            find_action(&actions, "copy_content").is_some(),
            "Scriptlet context should always have copy_content"
        );
    }

    #[test]
    fn scriptlet_context_copy_content_shortcut() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let action = find_action(&actions, "copy_content").unwrap();
        assert_eq!(action.shortcut.as_deref(), Some("⌘⌥C"));
    }

    #[test]
    fn scriptlet_context_has_copy_deeplink() {
        let script = ScriptInfo::scriptlet("My Scriptlet", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let deeplink = find_action(&actions, "copy_deeplink").unwrap();
        assert!(
            deeplink
                .description
                .as_ref()
                .unwrap()
                .contains("my-scriptlet"),
            "Deeplink description should contain deeplink name"
        );
    }

    // ============================================================
    // 23. Notes command bar icon name validation
    // ============================================================

    #[test]
    fn notes_command_bar_all_actions_have_icons() {
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
    fn notes_command_bar_new_note_has_plus_icon() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let new_note = find_action(&actions, "new_note").unwrap();
        assert_eq!(new_note.icon, Some(IconName::Plus));
    }

    #[test]
    fn notes_command_bar_browse_notes_has_folder_icon() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let browse = find_action(&actions, "browse_notes").unwrap();
        assert_eq!(browse.icon, Some(IconName::FolderOpen));
    }

    // ============================================================
    // 24. Cross-context action count stability
    // ============================================================

    #[test]
    fn script_context_action_count_deterministic() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let count1 = get_script_context_actions(&script).len();
        let count2 = get_script_context_actions(&script).len();
        assert_eq!(count1, count2, "Same input should produce same count");
    }

    #[test]
    fn clipboard_context_action_count_deterministic() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let count1 = get_clipboard_history_context_actions(&entry).len();
        let count2 = get_clipboard_history_context_actions(&entry).len();
        assert_eq!(count1, count2);
    }

    #[test]
    fn path_context_dir_and_file_same_count() {
        let dir = PathInfo {
            name: "test".to_string(),
            path: "/test".to_string(),
            is_dir: true,
        };
        let file = PathInfo {
            name: "test.txt".to_string(),
            path: "/test.txt".to_string(),
            is_dir: false,
        };
        let dir_count = get_path_context_actions(&dir).len();
        let file_count = get_path_context_actions(&file).len();
        assert_eq!(
            dir_count, file_count,
            "Dir and file path contexts should have same action count"
        );
    }

    // ============================================================
    // 25. Action builder chaining
    // ============================================================

    #[test]
    fn action_with_shortcut_opt_none_preserves_fields() {
        let action = Action::new(
            "test",
            "Test",
            Some("Desc".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Star)
        .with_section("MySection")
        .with_shortcut_opt(None);
        assert_eq!(action.icon, Some(IconName::Star));
        assert_eq!(action.section.as_deref(), Some("MySection"));
        assert!(action.shortcut.is_none());
    }

    #[test]
    fn action_with_shortcut_opt_some_sets_shortcut() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘Z".to_string()));
        assert_eq!(action.shortcut.as_deref(), Some("⌘Z"));
        assert_eq!(action.shortcut_lower.as_deref(), Some("⌘z"));
    }

    #[test]
    fn action_with_icon_and_section_order_independent() {
        let a1 = Action::new("t", "T", None, ActionCategory::ScriptContext)
            .with_icon(IconName::Code)
            .with_section("S");
        let a2 = Action::new("t", "T", None, ActionCategory::ScriptContext)
            .with_section("S")
            .with_icon(IconName::Code);
        assert_eq!(a1.icon, a2.icon);
        assert_eq!(a1.section, a2.section);
    }

    // ============================================================
    // 26. ActionsDialogConfig and enum defaults
    // ============================================================

    #[test]
    fn actions_dialog_config_default_values() {
        let config = ActionsDialogConfig::default();
        assert_eq!(config.search_position, SearchPosition::Bottom);
        assert_eq!(config.section_style, SectionStyle::Separators);
        assert_eq!(config.anchor, AnchorPosition::Bottom);
        assert!(!config.show_icons);
        assert!(!config.show_footer);
    }

    #[test]
    fn search_position_hidden_not_eq_top_or_bottom() {
        assert_ne!(SearchPosition::Hidden, SearchPosition::Top);
        assert_ne!(SearchPosition::Hidden, SearchPosition::Bottom);
    }

    #[test]
    fn section_style_headers_not_eq_separators() {
        assert_ne!(SectionStyle::Headers, SectionStyle::Separators);
        assert_ne!(SectionStyle::Headers, SectionStyle::None);
    }

    // ============================================================
    // 27. Clipboard action IDs all prefixed
    // ============================================================

    #[test]
    fn all_clipboard_action_ids_prefixed() {
        let entry = make_clipboard_entry(ContentType::Image, true, Some("Safari"));
        let actions = get_clipboard_history_context_actions(&entry);
        for action in &actions {
            assert!(
                action.id.starts_with("clipboard_"),
                "Clipboard action ID '{}' should start with 'clipboard_'",
                action.id
            );
        }
    }

    // ============================================================
    // 28. Fuzzy match edge cases
    // ============================================================

    #[test]
    fn fuzzy_match_empty_needle_matches() {
        assert!(ActionsDialog::fuzzy_match("anything", ""));
    }

    #[test]
    fn fuzzy_match_empty_haystack_empty_needle() {
        assert!(ActionsDialog::fuzzy_match("", ""));
    }

    #[test]
    fn fuzzy_match_empty_haystack_nonempty_needle() {
        assert!(!ActionsDialog::fuzzy_match("", "x"));
    }

    #[test]
    fn fuzzy_match_subsequence() {
        assert!(ActionsDialog::fuzzy_match("edit script", "esi"));
    }

    #[test]
    fn fuzzy_match_no_subsequence() {
        assert!(!ActionsDialog::fuzzy_match("abc", "abd"));
    }

    #[test]
    fn fuzzy_match_exact() {
        assert!(ActionsDialog::fuzzy_match("test", "test"));
    }

    #[test]
    fn fuzzy_match_needle_longer_than_haystack() {
        assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
    }

    // ============================================================
    // 29. has_action=false invariant for all builtin contexts
    // ============================================================

    #[test]
    fn script_context_all_has_action_false() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        for action in &get_script_context_actions(&script) {
            assert!(
                !action.has_action,
                "Script builtin '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn clipboard_context_all_has_action_false() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        for action in &get_clipboard_history_context_actions(&entry) {
            assert!(
                !action.has_action,
                "Clipboard builtin '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn path_context_all_has_action_false() {
        let path = PathInfo {
            name: "test".to_string(),
            path: "/test".to_string(),
            is_dir: false,
        };
        for action in &get_path_context_actions(&path) {
            assert!(
                !action.has_action,
                "Path builtin '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn file_context_all_has_action_false() {
        let file = FileInfo {
            path: "/test/f.txt".to_string(),
            name: "f.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        for action in &get_file_context_actions(&file) {
            assert!(
                !action.has_action,
                "File builtin '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn ai_command_bar_all_has_action_false() {
        for action in &get_ai_command_bar_actions() {
            assert!(
                !action.has_action,
                "AI builtin '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn notes_command_bar_all_has_action_false() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for action in &get_notes_command_bar_actions(&info) {
            assert!(
                !action.has_action,
                "Notes builtin '{}' should have has_action=false",
                action.id
            );
        }
    }

    // ============================================================
    // 30. ID uniqueness across contexts
    // ============================================================

    #[test]
    fn script_context_ids_unique() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        let ids: HashSet<_> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn clipboard_context_ids_unique() {
        let entry = make_clipboard_entry(ContentType::Image, true, Some("App"));
        let actions = get_clipboard_history_context_actions(&entry);
        let ids: HashSet<_> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn path_context_ids_unique() {
        let path = PathInfo {
            name: "dir".to_string(),
            path: "/dir".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path);
        let ids: HashSet<_> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn ai_command_bar_ids_no_overlap_with_notes() {
        let ai_actions = get_ai_command_bar_actions();
        let notes_info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let notes_actions = get_notes_command_bar_actions(&notes_info);
        let ai_ids: HashSet<_> = ai_actions.iter().map(|a| a.id.as_str()).collect();
        let notes_ids: HashSet<_> = notes_actions.iter().map(|a| a.id.as_str()).collect();
        let overlap: Vec<_> = ai_ids.intersection(&notes_ids).collect();
        // copy_deeplink appears in both — that's expected, it's the same action concept
        // But most should be unique
        assert!(
            overlap.len() <= 1,
            "AI and Notes should have minimal ID overlap, found: {:?}",
            overlap
        );
    }

    // ============================================================
    // Additional: title_lower/description_lower caching
    // ============================================================

    #[test]
    fn action_title_lower_computed_on_creation() {
        let action = Action::new("test", "Edit Script", None, ActionCategory::ScriptContext);
        assert_eq!(action.title_lower, "edit script");
    }

    #[test]
    fn action_description_lower_computed_on_creation() {
        let action = Action::new(
            "test",
            "Test",
            Some("Open in $EDITOR".to_string()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(
            action.description_lower,
            Some("open in $editor".to_string())
        );
    }

    #[test]
    fn action_no_description_lower_is_none() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
        assert!(action.description_lower.is_none());
    }

    #[test]
    fn action_shortcut_lower_set_by_with_shortcut() {
        let action =
            Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
        assert_eq!(action.shortcut_lower, Some("⌘⇧c".to_string()));
    }

    #[test]
    fn action_shortcut_lower_none_without_shortcut() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
        assert!(action.shortcut_lower.is_none());
    }

    // ============================================================
    // Additional: non-empty title/ID for all contexts
    // ============================================================

    #[test]
    fn all_script_actions_have_nonempty_title_and_id() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        for action in &get_script_context_actions(&script) {
            assert!(!action.id.is_empty(), "Action ID should not be empty");
            assert!(!action.title.is_empty(), "Action title should not be empty");
        }
    }

    #[test]
    fn all_clipboard_actions_have_nonempty_title_and_id() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        for action in &get_clipboard_history_context_actions(&entry) {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    #[test]
    fn all_ai_actions_have_nonempty_title_and_id() {
        for action in &get_ai_command_bar_actions() {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    // ============================================================
    // Additional: ordering determinism
    // ============================================================

    #[test]
    fn script_actions_ordering_deterministic() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions1 = get_script_context_actions(&script);
        let actions2 = get_script_context_actions(&script);
        let ids1 = action_ids(&actions1);
        let ids2 = action_ids(&actions2);
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn clipboard_actions_ordering_deterministic() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions1 = get_clipboard_history_context_actions(&entry);
        let actions2 = get_clipboard_history_context_actions(&entry);
        let ids1 = action_ids(&actions1);
        let ids2 = action_ids(&actions2);
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn ai_actions_ordering_deterministic() {
        let actions1 = get_ai_command_bar_actions();
        let actions2 = get_ai_command_bar_actions();
        let ids1 = action_ids(&actions1);
        let ids2 = action_ids(&actions2);
        assert_eq!(ids1, ids2);
    }

    // ============================================================
    // Additional: Clipboard destructive ordering
    // ============================================================

    #[test]
    fn clipboard_destructive_always_last_three() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let len = actions.len();
        assert!(len >= 3);
        assert_eq!(actions[len - 3].id, "clipboard_delete");
        assert_eq!(actions[len - 2].id, "clipboard_delete_multiple");
        assert_eq!(actions[len - 1].id, "clipboard_delete_all");
    }

    #[test]
    fn clipboard_paste_always_first() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[0].id, "clipboard_paste");
    }

    #[test]
    fn clipboard_copy_always_second() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[1].id, "clipboard_copy");
    }

    // ============================================================
    // Additional: ActionCategory enum
    // ============================================================

    #[test]
    fn all_script_context_actions_are_script_context_category() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        for action in &get_script_context_actions(&script) {
            assert_eq!(
                action.category,
                ActionCategory::ScriptContext,
                "Action '{}' should be ScriptContext",
                action.id
            );
        }
    }

    #[test]
    fn all_clipboard_actions_are_script_context_category() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        for action in &get_clipboard_history_context_actions(&entry) {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn all_ai_actions_are_script_context_category() {
        for action in &get_ai_command_bar_actions() {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn all_path_actions_are_script_context_category() {
        let path = PathInfo {
            name: "test".to_string(),
            path: "/test".to_string(),
            is_dir: false,
        };
        for action in &get_path_context_actions(&path) {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }
}
