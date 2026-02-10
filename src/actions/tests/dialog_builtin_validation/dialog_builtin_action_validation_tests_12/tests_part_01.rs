    use super::super::builders::*;
    use super::super::dialog::{
        build_grouped_items_static, coerce_action_selection, ActionsDialog, GroupedActionItem,
    };
    use super::super::types::*;
    use crate::clipboard_history::ContentType;
    use crate::designs::icon_variations::IconName;
    use crate::file_search::FileInfo;
    use crate::prompts::PathInfo;
    use crate::scriptlets::{Scriptlet, ScriptletAction};
    use std::collections::HashSet;

    // =========================================================================
    // Helper
    // =========================================================================
    fn action_ids(actions: &[Action]) -> Vec<String> {
        actions.iter().map(|a| a.id.clone()).collect()
    }

    // =========================================================================
    // 1. CommandBarConfig preset field validation — all four presets
    // =========================================================================

    #[test]
    fn cat01_command_bar_default_dialog_config() {
        let cfg = ActionsDialogConfig::default();
        assert_eq!(cfg.search_position, SearchPosition::Bottom);
        assert_eq!(cfg.section_style, SectionStyle::Separators);
        assert_eq!(cfg.anchor, AnchorPosition::Bottom);
        assert!(!cfg.show_icons);
        assert!(!cfg.show_footer);
    }

    #[test]
    fn cat01_command_bar_ai_style_dialog_config() {
        let cfg = super::super::command_bar::CommandBarConfig::ai_style();
        assert_eq!(cfg.dialog_config.search_position, SearchPosition::Top);
        assert_eq!(cfg.dialog_config.section_style, SectionStyle::Headers);
        assert_eq!(cfg.dialog_config.anchor, AnchorPosition::Top);
        assert!(cfg.dialog_config.show_icons);
        assert!(cfg.dialog_config.show_footer);
        assert!(cfg.close_on_select);
        assert!(cfg.close_on_click_outside);
        assert!(cfg.close_on_escape);
    }

    #[test]
    fn cat01_command_bar_main_menu_style_dialog_config() {
        let cfg = super::super::command_bar::CommandBarConfig::main_menu_style();
        assert_eq!(cfg.dialog_config.search_position, SearchPosition::Bottom);
        assert_eq!(cfg.dialog_config.section_style, SectionStyle::Separators);
        assert_eq!(cfg.dialog_config.anchor, AnchorPosition::Bottom);
        assert!(!cfg.dialog_config.show_icons);
        assert!(!cfg.dialog_config.show_footer);
    }

    #[test]
    fn cat01_command_bar_no_search_dialog_config() {
        let cfg = super::super::command_bar::CommandBarConfig::no_search();
        assert_eq!(cfg.dialog_config.search_position, SearchPosition::Hidden);
        assert_eq!(cfg.dialog_config.section_style, SectionStyle::Separators);
        assert_eq!(cfg.dialog_config.anchor, AnchorPosition::Bottom);
        assert!(!cfg.dialog_config.show_icons);
        assert!(!cfg.dialog_config.show_footer);
    }

    #[test]
    fn cat01_command_bar_notes_style_dialog_config() {
        let cfg = super::super::command_bar::CommandBarConfig::notes_style();
        assert_eq!(cfg.dialog_config.search_position, SearchPosition::Top);
        assert_eq!(cfg.dialog_config.section_style, SectionStyle::Separators);
        assert_eq!(cfg.dialog_config.anchor, AnchorPosition::Top);
        assert!(cfg.dialog_config.show_icons);
        assert!(cfg.dialog_config.show_footer);
    }

    #[test]
    fn cat01_command_bar_all_presets_close_defaults_true() {
        for cfg in [
            super::super::command_bar::CommandBarConfig::default(),
            super::super::command_bar::CommandBarConfig::ai_style(),
            super::super::command_bar::CommandBarConfig::main_menu_style(),
            super::super::command_bar::CommandBarConfig::no_search(),
            super::super::command_bar::CommandBarConfig::notes_style(),
        ] {
            assert!(cfg.close_on_select, "close_on_select should default true");
            assert!(
                cfg.close_on_click_outside,
                "close_on_click_outside should default true"
            );
            assert!(cfg.close_on_escape, "close_on_escape should default true");
        }
    }

    // =========================================================================
    // 2. Clipboard text vs image — exact ID difference
    // =========================================================================

    fn make_text_entry() -> ClipboardEntryInfo {
        ClipboardEntryInfo {
            id: "t1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        }
    }

    fn make_image_entry() -> ClipboardEntryInfo {
        ClipboardEntryInfo {
            id: "i1".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "Image".into(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        }
    }

    #[test]
    fn cat02_image_has_ocr_text_does_not() {
        let text_ids = action_ids(&get_clipboard_history_context_actions(&make_text_entry()));
        let image_ids = action_ids(&get_clipboard_history_context_actions(&make_image_entry()));
        assert!(!text_ids.contains(&"clipboard_ocr".to_string()));
        assert!(image_ids.contains(&"clipboard_ocr".to_string()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat02_image_has_open_with_and_cleanshot_text_does_not() {
        let text_ids = action_ids(&get_clipboard_history_context_actions(&make_text_entry()));
        let image_ids = action_ids(&get_clipboard_history_context_actions(&make_image_entry()));
        for id in [
            "clipboard_open_with",
            "clipboard_annotate_cleanshot",
            "clipboard_upload_cleanshot",
        ] {
            assert!(
                !text_ids.contains(&id.to_string()),
                "text should not have {}",
                id
            );
            assert!(
                image_ids.contains(&id.to_string()),
                "image should have {}",
                id
            );
        }
    }

    #[test]
    fn cat02_image_more_actions_than_text() {
        let text_count = get_clipboard_history_context_actions(&make_text_entry()).len();
        let image_count = get_clipboard_history_context_actions(&make_image_entry()).len();
        assert!(
            image_count > text_count,
            "image {} > text {}",
            image_count,
            text_count
        );
    }

    #[test]
    fn cat02_shared_actions_between_text_and_image() {
        let text_ids: HashSet<String> =
            action_ids(&get_clipboard_history_context_actions(&make_text_entry()))
                .into_iter()
                .collect();
        let image_ids: HashSet<String> =
            action_ids(&get_clipboard_history_context_actions(&make_image_entry()))
                .into_iter()
                .collect();
        let shared: Vec<&str> = vec![
            "clipboard_paste",
            "clipboard_copy",
            "clipboard_paste_keep_open",
            "clipboard_share",
            "clipboard_attach_to_ai",
            "clipboard_pin",
            "clipboard_save_snippet",
            "clipboard_save_file",
            "clipboard_delete",
            "clipboard_delete_multiple",
            "clipboard_delete_all",
        ];
        for id in shared {
            assert!(text_ids.contains(id), "text missing {}", id);
            assert!(image_ids.contains(id), "image missing {}", id);
        }
    }

    // =========================================================================
    // 3. Scriptlet context shortcut/alias dynamic switching
    // =========================================================================

    #[test]
    fn cat03_scriptlet_with_custom_no_shortcut_no_alias() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"add_shortcut".to_string()));
        assert!(ids.contains(&"add_alias".to_string()));
        assert!(!ids.contains(&"update_shortcut".to_string()));
        assert!(!ids.contains(&"update_alias".to_string()));
    }

    #[test]
    fn cat03_scriptlet_with_custom_has_shortcut_has_alias() {
        let script = ScriptInfo::scriptlet(
            "Test",
            "/path/test.md",
            Some("cmd+t".into()),
            Some("ts".into()),
        );
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"update_shortcut".to_string()));
        assert!(ids.contains(&"remove_shortcut".to_string()));
        assert!(ids.contains(&"update_alias".to_string()));
        assert!(ids.contains(&"remove_alias".to_string()));
        assert!(!ids.contains(&"add_shortcut".to_string()));
        assert!(!ids.contains(&"add_alias".to_string()));
    }

    #[test]
    fn cat03_scriptlet_with_custom_has_shortcut_no_alias() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", Some("cmd+t".into()), None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"update_shortcut".to_string()));
        assert!(ids.contains(&"add_alias".to_string()));
    }

    #[test]
    fn cat03_scriptlet_with_custom_no_shortcut_has_alias() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, Some("ts".into()));
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"add_shortcut".to_string()));
        assert!(ids.contains(&"update_alias".to_string()));
    }

    #[test]
    fn cat03_scriptlet_context_always_has_edit_reveal_copy() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"edit_scriptlet".to_string()));
        assert!(ids.contains(&"reveal_scriptlet_in_finder".to_string()));
        assert!(ids.contains(&"copy_scriptlet_path".to_string()));
        assert!(ids.contains(&"copy_content".to_string()));
        assert!(ids.contains(&"copy_deeplink".to_string()));
    }

    // =========================================================================
    // 4. Agent context description keyword content
    // =========================================================================

    #[test]
    fn cat04_agent_edit_title_says_agent() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    #[test]
    fn cat04_agent_edit_description_mentions_agent() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert!(edit.description.as_ref().unwrap().contains("agent"));
    }

    #[test]
    fn cat04_agent_reveal_description_mentions_agent() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
        assert!(reveal.description.as_ref().unwrap().contains("agent"));
    }

    #[test]
    fn cat04_agent_copy_path_description_mentions_agent() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
        assert!(cp.description.as_ref().unwrap().contains("agent"));
    }

    #[test]
    fn cat04_agent_no_view_logs() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    // =========================================================================
    // 5. Note switcher multi-note section assignment
    // =========================================================================

    fn make_note(id: &str, title: &str, pinned: bool, current: bool) -> NoteSwitcherNoteInfo {
        NoteSwitcherNoteInfo {
            id: id.into(),
            title: title.into(),
            char_count: 100,
            is_current: current,
            is_pinned: pinned,
            preview: "Some preview text".into(),
            relative_time: "2m ago".into(),
        }
    }

    #[test]
    fn cat05_pinned_notes_in_pinned_section() {
        let notes = vec![make_note("1", "Pinned Note", true, false)];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
    }

    #[test]
    fn cat05_unpinned_notes_in_recent_section() {
        let notes = vec![make_note("2", "Recent Note", false, false)];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn cat05_mixed_notes_correct_sections() {
        let notes = vec![
            make_note("1", "Pinned", true, false),
            make_note("2", "Recent", false, false),
            make_note("3", "Also Pinned", true, true),
        ];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
        assert_eq!(actions[1].section.as_deref(), Some("Recent"));
        assert_eq!(actions[2].section.as_deref(), Some("Pinned"));
    }

    #[test]
    fn cat05_current_note_gets_bullet_prefix() {
        let notes = vec![make_note("1", "My Note", false, true)];
        let actions = get_note_switcher_actions(&notes);
        assert!(actions[0].title.starts_with("• "));
    }

    #[test]
    fn cat05_non_current_note_no_bullet() {
        let notes = vec![make_note("1", "My Note", false, false)];
        let actions = get_note_switcher_actions(&notes);
        assert!(!actions[0].title.starts_with("• "));
    }

    #[test]
    fn cat05_note_ids_follow_pattern() {
        let notes = vec![
            make_note("abc-123", "Note A", false, false),
            make_note("def-456", "Note B", true, false),
        ];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].id, "note_abc-123");
        assert_eq!(actions[1].id, "note_def-456");
    }

    // =========================================================================
    // 6. AI command bar section-to-action-count mapping
    // =========================================================================

    #[test]
    fn cat06_ai_response_section_has_3_actions() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Response"))
            .count();
        assert_eq!(count, 3, "Response section should have 3 actions");
    }

    #[test]
    fn cat06_ai_actions_section_has_4_actions() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Actions"))
            .count();
        assert_eq!(count, 4, "Actions section should have 4 actions (submit, new_chat, delete_chat, branch_from_last)");
    }

    #[test]
    fn cat06_ai_attachments_section_has_2_actions() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Attachments"))
            .count();
        assert_eq!(count, 2, "Attachments section should have 2 actions");
    }

    #[test]
    fn cat06_ai_export_section_has_1_action() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Export"))
            .count();
        assert_eq!(count, 1, "Export section should have 1 action");
    }

    #[test]
    fn cat06_ai_help_section_has_1_action() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Help"))
            .count();
        assert_eq!(count, 1, "Help section should have 1 action");
    }

    #[test]
    fn cat06_ai_settings_section_has_1_action() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Settings"))
            .count();
        assert_eq!(count, 1, "Settings section should have 1 action");
    }

    #[test]
    fn cat06_ai_total_12_actions() {
        assert_eq!(get_ai_command_bar_actions().len(), 12);
    }

