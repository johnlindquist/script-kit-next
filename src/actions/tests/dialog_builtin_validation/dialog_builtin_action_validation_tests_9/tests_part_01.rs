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

