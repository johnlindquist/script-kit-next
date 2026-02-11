//! Batch 13 — Builtin action validation tests
//!
//! Focus areas:
//! - format_shortcut_hint edge cases (non-modifier intermediate parts, aliased modifiers)
//! - ScriptInfo mutually-exclusive flags (agent vs script vs scriptlet vs builtin)
//! - Scriptlet context custom action value/has_action propagation
//! - Clipboard save_snippet/save_file universality (text and image)
//! - Path context copy_filename has no shortcut
//! - Note switcher description ellipsis boundary (exactly 60 chars)
//! - Chat context multi-model ordering and checkmark logic
//! - AI command bar actions without shortcuts
//! - CommandBarConfig close flag defaults
//! - Cross-builder shortcut/alias action symmetry
//! - Scriptlet context action verb propagation
//! - Agent context exact action IDs
//! - Deeplink URL in description for scriptlet context
//! - Notes command bar create_quicklink and export actions
//! - Action::new lowercase caching correctness

#[cfg(test)]
mod tests {
    // --- merged from tests_part_01.rs ---
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
    // 1. format_shortcut_hint aliased modifier keywords
    // =========================================================================

    #[test]
    fn cat01_format_shortcut_hint_meta_maps_to_cmd() {
        let result = ActionsDialog::format_shortcut_hint("meta+c");
        assert_eq!(result, "⌘C");
    }

    #[test]
    fn cat01_format_shortcut_hint_super_maps_to_cmd() {
        let result = ActionsDialog::format_shortcut_hint("super+x");
        assert_eq!(result, "⌘X");
    }

    #[test]
    fn cat01_format_shortcut_hint_command_maps_to_cmd() {
        let result = ActionsDialog::format_shortcut_hint("command+a");
        assert_eq!(result, "⌘A");
    }

    #[test]
    fn cat01_format_shortcut_hint_option_maps_to_alt() {
        let result = ActionsDialog::format_shortcut_hint("option+z");
        assert_eq!(result, "⌥Z");
    }

    #[test]
    fn cat01_format_shortcut_hint_control_maps_to_ctrl() {
        let result = ActionsDialog::format_shortcut_hint("control+b");
        assert_eq!(result, "⌃B");
    }

    #[test]
    fn cat01_format_shortcut_hint_return_maps_to_enter() {
        let result = ActionsDialog::format_shortcut_hint("cmd+return");
        assert_eq!(result, "⌘↵");
    }

    #[test]
    fn cat01_format_shortcut_hint_esc_maps_to_escape() {
        let result = ActionsDialog::format_shortcut_hint("esc");
        assert_eq!(result, "⎋");
    }

    #[test]
    fn cat01_format_shortcut_hint_arrowup_maps_to_up() {
        let result = ActionsDialog::format_shortcut_hint("arrowup");
        assert_eq!(result, "↑");
    }

    #[test]
    fn cat01_format_shortcut_hint_arrowdown_maps_to_down() {
        let result = ActionsDialog::format_shortcut_hint("arrowdown");
        assert_eq!(result, "↓");
    }

    #[test]
    fn cat01_format_shortcut_hint_arrowleft_maps_to_left() {
        let result = ActionsDialog::format_shortcut_hint("arrowleft");
        assert_eq!(result, "←");
    }

    #[test]
    fn cat01_format_shortcut_hint_arrowright_maps_to_right() {
        let result = ActionsDialog::format_shortcut_hint("arrowright");
        assert_eq!(result, "→");
    }

    #[test]
    fn cat01_format_shortcut_hint_tab_maps_to_tab_symbol() {
        let result = ActionsDialog::format_shortcut_hint("tab");
        assert_eq!(result, "⇥");
    }

    #[test]
    fn cat01_format_shortcut_hint_backspace_maps_to_delete_symbol() {
        let result = ActionsDialog::format_shortcut_hint("backspace");
        assert_eq!(result, "⌫");
    }

    #[test]
    fn cat01_format_shortcut_hint_delete_maps_to_delete_symbol() {
        let result = ActionsDialog::format_shortcut_hint("delete");
        assert_eq!(result, "⌫");
    }

    #[test]
    fn cat01_format_shortcut_hint_space_maps_to_space_symbol() {
        let result = ActionsDialog::format_shortcut_hint("space");
        assert_eq!(result, "␣");
    }

    #[test]
    fn cat01_format_shortcut_hint_single_letter() {
        let result = ActionsDialog::format_shortcut_hint("a");
        assert_eq!(result, "A");
    }

    #[test]
    fn cat01_format_shortcut_hint_case_insensitive() {
        let result = ActionsDialog::format_shortcut_hint("CMD+SHIFT+C");
        assert_eq!(result, "⌘⇧C");
    }

    // =========================================================================
    // 2. ScriptInfo mutually-exclusive flags (agent vs script vs scriptlet)
    // =========================================================================

    #[test]
    fn cat02_script_info_new_has_is_script_true() {
        let s = ScriptInfo::new("t", "/p");
        assert!(s.is_script);
        assert!(!s.is_scriptlet);
        assert!(!s.is_agent);
    }

    #[test]
    fn cat02_scriptlet_has_is_scriptlet_true() {
        let s = ScriptInfo::scriptlet("t", "/p", None, None);
        assert!(!s.is_script);
        assert!(s.is_scriptlet);
        assert!(!s.is_agent);
    }

    #[test]
    fn cat02_builtin_has_all_false() {
        let s = ScriptInfo::builtin("B");
        assert!(!s.is_script);
        assert!(!s.is_scriptlet);
        assert!(!s.is_agent);
    }

    #[test]
    fn cat02_with_action_verb_not_script_not_scriptlet() {
        let s = ScriptInfo::with_action_verb("Safari", "/app", false, "Launch");
        assert!(!s.is_script);
        assert!(!s.is_scriptlet);
        assert!(!s.is_agent);
    }

    #[test]
    fn cat02_agent_flag_via_new_plus_mutation() {
        // ScriptInfo::new sets is_script=true; for agents we construct differently
        // The agent flag is set by the caller, not by the constructor
        let mut s = ScriptInfo::new("Agent", "/agent");
        s.is_agent = true;
        s.is_script = false;
        assert!(!s.is_script);
        assert!(!s.is_scriptlet);
        assert!(s.is_agent);
    }

    // =========================================================================
    // 3. Scriptlet context custom action value/has_action propagation
    // =========================================================================

    #[test]
    fn cat03_scriptlet_defined_action_has_action_true() {
        let mut scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Copy".into(),
            command: "copy-cmd".into(),
            tool: "bash".into(),
            code: "pbcopy".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions.len(), 1);
        assert!(actions[0].has_action);
        assert_eq!(actions[0].value, Some("copy-cmd".into()));
    }

    #[test]
    fn cat03_scriptlet_defined_action_id_prefix() {
        let mut scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "My Action".into(),
            command: "my-action".into(),
            tool: "bash".into(),
            code: "echo".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert!(actions[0].id.starts_with("scriptlet_action:"));
    }

    #[test]
    fn cat03_scriptlet_defined_action_shortcut_formatted() {
        let mut scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "A".into(),
            command: "a".into(),
            tool: "bash".into(),
            code: "".into(),
            inputs: vec![],
            shortcut: Some("cmd+shift+p".into()),
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        // format_shortcut_hint converts "cmd+shift+p" to "⌘⇧P"
        assert_eq!(actions[0].shortcut, Some("⌘⇧P".into()));
    }

    #[test]
    fn cat03_scriptlet_defined_action_description_propagated() {
        let mut scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "A".into(),
            command: "a".into(),
            tool: "bash".into(),
            code: "".into(),
            inputs: vec![],
            shortcut: None,
            description: Some("Custom desc".into()),
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions[0].description, Some("Custom desc".into()));
    }

    #[test]
    fn cat03_scriptlet_defined_action_no_shortcut_is_none() {
        let mut scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "A".into(),
            command: "a".into(),
            tool: "bash".into(),
            code: "".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert!(actions[0].shortcut.is_none());
    }

    #[test]
    fn cat03_empty_scriptlet_no_custom_actions() {
        let scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert!(actions.is_empty());
    }

    // =========================================================================
    // 4. Clipboard save_snippet/save_file present for both text and image
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
            preview: "img".into(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        }
    }

    #[test]
    fn cat04_text_has_save_snippet() {
        let actions = get_clipboard_history_context_actions(&make_text_entry());
        assert!(actions.iter().any(|a| a.id == "clipboard_save_snippet"));
    }

    #[test]
    fn cat04_text_has_save_file() {
        let actions = get_clipboard_history_context_actions(&make_text_entry());
        assert!(actions.iter().any(|a| a.id == "clipboard_save_file"));
    }

    #[test]
    fn cat04_image_has_save_snippet() {
        let actions = get_clipboard_history_context_actions(&make_image_entry());
        assert!(actions.iter().any(|a| a.id == "clipboard_save_snippet"));
    }

    #[test]
    fn cat04_image_has_save_file() {
        let actions = get_clipboard_history_context_actions(&make_image_entry());
        assert!(actions.iter().any(|a| a.id == "clipboard_save_file"));
    }

    #[test]
    fn cat04_save_snippet_shortcut() {
        let actions = get_clipboard_history_context_actions(&make_text_entry());
        let a = actions
            .iter()
            .find(|a| a.id == "clipboard_save_snippet")
            .unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⇧⌘S"));
    }

    #[test]
    fn cat04_save_file_shortcut() {
        let actions = get_clipboard_history_context_actions(&make_text_entry());
        let a = actions
            .iter()
            .find(|a| a.id == "clipboard_save_file")
            .unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌥⇧⌘S"));
    }

    // =========================================================================
    // 5. Path context copy_filename has no shortcut
    // =========================================================================

    #[test]
    fn cat05_path_copy_filename_no_shortcut() {
        let info = PathInfo {
            path: "/tmp/foo.txt".into(),
            name: "foo.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let a = actions.iter().find(|a| a.id == "copy_filename").unwrap();
        assert!(
            a.shortcut.is_none(),
            "Path copy_filename should have no shortcut"
        );
    }

    #[test]
    fn cat05_path_copy_filename_description() {
        let info = PathInfo {
            path: "/tmp/foo.txt".into(),
            name: "foo.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let a = actions.iter().find(|a| a.id == "copy_filename").unwrap();
        assert!(a.description.as_ref().unwrap().contains("filename"));
    }

    #[test]
    fn cat05_file_copy_filename_has_shortcut() {
        // In contrast, file context copy_filename DOES have a shortcut (⌘C)
        let info = FileInfo {
            path: "/tmp/foo.txt".into(),
            name: "foo.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        let a = actions.iter().find(|a| a.id == "copy_filename").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘C"));
    }

    // =========================================================================
    // 6. Note switcher description ellipsis boundary (exactly 60 chars)
    // =========================================================================

    fn make_note(preview: &str, time: &str, chars: usize) -> NoteSwitcherNoteInfo {
        NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "Note".into(),
            char_count: chars,
            is_current: false,
            is_pinned: false,
            preview: preview.into(),
            relative_time: time.into(),
        }
    }

    #[test]
    fn cat06_preview_exactly_60_chars_no_ellipsis() {
        let preview = "a".repeat(60);
        let actions = get_note_switcher_actions(&[make_note(&preview, "1m", 60)]);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(
            !desc.contains('…'),
            "60 chars should NOT have ellipsis: {}",
            desc
        );
    }

    #[test]
    fn cat06_preview_61_chars_has_ellipsis() {
        let preview = "a".repeat(61);
        let actions = get_note_switcher_actions(&[make_note(&preview, "1m", 61)]);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(
            desc.contains('…'),
            "61 chars should have ellipsis: {}",
            desc
        );
    }

    #[test]
    fn cat06_empty_preview_empty_time_uses_char_count() {
        let actions = get_note_switcher_actions(&[make_note("", "", 42)]);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains("42"), "Should show char count: {}", desc);
        assert!(desc.contains("chars"), "Should say 'chars': {}", desc);
    }

    #[test]
    fn cat06_empty_preview_with_time_uses_time() {
        let actions = get_note_switcher_actions(&[make_note("", "5m ago", 10)]);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "5m ago");
    }

    #[test]
    fn cat06_preview_with_time_has_separator() {
        let actions = get_note_switcher_actions(&[make_note("Hello world", "2h ago", 11)]);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains(" · "), "Should have separator: {}", desc);
        assert!(desc.contains("Hello world"));
        assert!(desc.contains("2h ago"));
    }


    // --- merged from tests_part_02.rs ---
    #[test]
    fn cat06_one_char_singular() {
        let actions = get_note_switcher_actions(&[make_note("", "", 1)]);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "1 char");
    }

    #[test]
    fn cat06_zero_chars_plural() {
        let actions = get_note_switcher_actions(&[make_note("", "", 0)]);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "0 chars");
    }

    // =========================================================================
    // 7. Chat context multi-model ordering and checkmark logic
    // =========================================================================

    fn make_chat_info(
        current: Option<&str>,
        models: &[(&str, &str, &str)],
        has_response: bool,
        has_messages: bool,
    ) -> ChatPromptInfo {
        ChatPromptInfo {
            current_model: current.map(|s| s.to_string()),
            available_models: models
                .iter()
                .map(|(id, name, provider)| ChatModelInfo {
                    id: id.to_string(),
                    display_name: name.to_string(),
                    provider: provider.to_string(),
                })
                .collect(),
            has_response,
            has_messages,
        }
    }

    #[test]
    fn cat07_model_actions_ordered_by_input() {
        let info = make_chat_info(
            None,
            &[("m1", "Model A", "P1"), ("m2", "Model B", "P2")],
            false,
            false,
        );
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions[0].id, "select_model_m1");
        assert_eq!(actions[1].id, "select_model_m2");
    }

    #[test]
    fn cat07_current_model_gets_checkmark() {
        let info = make_chat_info(
            Some("Model A"),
            &[("m1", "Model A", "P1"), ("m2", "Model B", "P2")],
            false,
            false,
        );
        let actions = get_chat_context_actions(&info);
        let m1 = actions.iter().find(|a| a.id == "select_model_m1").unwrap();
        assert!(m1.title.contains('✓'), "Current model should have ✓");
    }

    #[test]
    fn cat07_non_current_model_no_checkmark() {
        let info = make_chat_info(
            Some("Model A"),
            &[("m1", "Model A", "P1"), ("m2", "Model B", "P2")],
            false,
            false,
        );
        let actions = get_chat_context_actions(&info);
        let m2 = actions.iter().find(|a| a.id == "select_model_m2").unwrap();
        assert!(
            !m2.title.contains('✓'),
            "Non-current model should NOT have ✓"
        );
    }

    #[test]
    fn cat07_model_description_is_via_provider() {
        let info = make_chat_info(None, &[("m1", "Claude", "Anthropic")], false, false);
        let actions = get_chat_context_actions(&info);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "via Anthropic");
    }

    #[test]
    fn cat07_no_models_still_has_continue_in_chat() {
        let info = make_chat_info(None, &[], false, false);
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "continue_in_chat"));
    }

    #[test]
    fn cat07_has_response_adds_copy_response() {
        let info = make_chat_info(None, &[], true, false);
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "copy_response"));
    }

    #[test]
    fn cat07_no_response_no_copy_response() {
        let info = make_chat_info(None, &[], false, false);
        let actions = get_chat_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "copy_response"));
    }

    #[test]
    fn cat07_has_messages_adds_clear_conversation() {
        let info = make_chat_info(None, &[], false, true);
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "clear_conversation"));
    }

    #[test]
    fn cat07_no_messages_no_clear_conversation() {
        let info = make_chat_info(None, &[], false, false);
        let actions = get_chat_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "clear_conversation"));
    }

    #[test]
    fn cat07_continue_in_chat_shortcut() {
        let info = make_chat_info(None, &[], false, false);
        let actions = get_chat_context_actions(&info);
        let a = actions.iter().find(|a| a.id == "continue_in_chat").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘↵"));
    }

    // =========================================================================
    // 8. AI command bar actions without shortcuts
    // =========================================================================

    #[test]
    fn cat08_branch_from_last_no_shortcut() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
        assert!(
            a.shortcut.is_none(),
            "branch_from_last should have no shortcut"
        );
    }

    #[test]
    fn cat08_change_model_no_shortcut() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "change_model").unwrap();
        assert!(a.shortcut.is_none(), "change_model should have no shortcut");
    }

    #[test]
    fn cat08_toggle_shortcuts_help_has_shortcut() {
        let actions = get_ai_command_bar_actions();
        let a = actions
            .iter()
            .find(|a| a.id == "toggle_shortcuts_help")
            .unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘/"));
    }

    #[test]
    fn cat08_export_markdown_shortcut() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "export_markdown").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⇧⌘E"));
    }

    #[test]
    fn cat08_ai_all_have_icons() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                action.icon.is_some(),
                "AI action {} should have icon",
                action.id
            );
        }
    }

    #[test]
    fn cat08_ai_all_have_sections() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                action.section.is_some(),
                "AI action {} should have section",
                action.id
            );
        }
    }

    // =========================================================================
    // 9. CommandBarConfig close flag defaults
    // =========================================================================

    #[test]
    fn cat09_default_close_on_select_true() {
        let config = super::super::command_bar::CommandBarConfig::default();
        assert!(config.close_on_select);
    }

    #[test]
    fn cat09_default_close_on_click_outside_true() {
        let config = super::super::command_bar::CommandBarConfig::default();
        assert!(config.close_on_click_outside);
    }

    #[test]
    fn cat09_default_close_on_escape_true() {
        let config = super::super::command_bar::CommandBarConfig::default();
        assert!(config.close_on_escape);
    }

    #[test]
    fn cat09_ai_style_close_defaults_preserved() {
        let config = super::super::command_bar::CommandBarConfig::ai_style();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    #[test]
    fn cat09_main_menu_style_close_defaults_preserved() {
        let config = super::super::command_bar::CommandBarConfig::main_menu_style();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    #[test]
    fn cat09_no_search_style_close_defaults_preserved() {
        let config = super::super::command_bar::CommandBarConfig::no_search();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    #[test]
    fn cat09_notes_style_close_defaults_preserved() {
        let config = super::super::command_bar::CommandBarConfig::notes_style();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    // =========================================================================
    // 10. Cross-builder shortcut/alias action symmetry
    // =========================================================================

    #[test]
    fn cat10_script_no_shortcut_no_alias_has_add_both() {
        let s = ScriptInfo::new("t", "/p");
        let ids = action_ids(&get_script_context_actions(&s));
        assert!(ids.contains(&"add_shortcut".into()));
        assert!(ids.contains(&"add_alias".into()));
        assert!(!ids.contains(&"update_shortcut".into()));
        assert!(!ids.contains(&"update_alias".into()));
    }

    #[test]
    fn cat10_script_has_shortcut_has_alias_has_update_remove_both() {
        let s =
            ScriptInfo::with_shortcut_and_alias("t", "/p", Some("cmd+t".into()), Some("ts".into()));
        let ids = action_ids(&get_script_context_actions(&s));
        assert!(ids.contains(&"update_shortcut".into()));
        assert!(ids.contains(&"remove_shortcut".into()));
        assert!(ids.contains(&"update_alias".into()));
        assert!(ids.contains(&"remove_alias".into()));
        assert!(!ids.contains(&"add_shortcut".into()));
        assert!(!ids.contains(&"add_alias".into()));
    }

    #[test]
    fn cat10_scriptlet_context_same_shortcut_alias_logic() {
        let s = ScriptInfo::scriptlet("t", "/p", Some("cmd+k".into()), Some("tk".into()));
        let actions = get_scriptlet_context_actions_with_custom(&s, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"update_shortcut".into()));
        assert!(ids.contains(&"remove_shortcut".into()));
        assert!(ids.contains(&"update_alias".into()));
        assert!(ids.contains(&"remove_alias".into()));
    }

    #[test]
    fn cat10_scriptlet_no_shortcut_no_alias_has_add() {
        let s = ScriptInfo::scriptlet("t", "/p", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&s, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"add_shortcut".into()));
        assert!(ids.contains(&"add_alias".into()));
    }

    #[test]
    fn cat10_shortcut_and_alias_action_shortcut_values() {
        let s = ScriptInfo::new("t", "/p");
        let actions = get_script_context_actions(&s);
        let add_sc = actions.iter().find(|a| a.id == "add_shortcut").unwrap();
        assert_eq!(add_sc.shortcut.as_deref(), Some("⌘⇧K"));
        let add_al = actions.iter().find(|a| a.id == "add_alias").unwrap();
        assert_eq!(add_al.shortcut.as_deref(), Some("⌘⇧A"));
    }

    // =========================================================================
    // 11. Scriptlet context action verb propagation
    // =========================================================================

    #[test]
    fn cat11_scriptlet_run_title_uses_action_verb() {
        let s = ScriptInfo::scriptlet("My Script", "/p", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&s, None);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert!(
            run.title.starts_with("Run "),
            "Title should start with 'Run ': {}",
            run.title
        );
        assert!(
            run.title.contains("My Script"),
            "Title should contain name: {}",
            run.title
        );
    }

    #[test]
    fn cat11_scriptlet_run_description_uses_verb() {
        let s = ScriptInfo::scriptlet("T", "/p", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&s, None);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        let desc = run.description.as_ref().unwrap();
        assert!(
            desc.contains("Run"),
            "Description should contain verb: {}",
            desc
        );
    }

    #[test]
    fn cat11_script_context_custom_verb_propagates() {
        let s = ScriptInfo::with_action_verb("Safari", "/app", false, "Launch");
        let actions = get_script_context_actions(&s);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert!(
            run.title.starts_with("Launch "),
            "Title should start with 'Launch ': {}",
            run.title
        );
    }

    #[test]
    fn cat11_script_context_switch_to_verb() {
        let s = ScriptInfo::with_action_verb("Window", "win:1", false, "Switch to");
        let actions = get_script_context_actions(&s);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert!(run.title.starts_with("Switch to "), "Title: {}", run.title);
    }

    // =========================================================================
    // 12. Agent context exact action IDs
    // =========================================================================

    #[test]
    fn cat12_agent_has_edit_script_not_edit_agent_id() {
        // Agent uses "edit_script" as ID but "Edit Agent" as title
        let mut s = ScriptInfo::new("My Agent", "/agent");
        s.is_agent = true;
        s.is_script = false;
        let actions = get_script_context_actions(&s);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"edit_script".into()));
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    #[test]
    fn cat12_agent_has_reveal_in_finder() {
        let mut s = ScriptInfo::new("My Agent", "/agent");
        s.is_agent = true;
        s.is_script = false;
        let actions = get_script_context_actions(&s);
        assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
    }

    #[test]
    fn cat12_agent_has_copy_path() {
        let mut s = ScriptInfo::new("My Agent", "/agent");
        s.is_agent = true;
        s.is_script = false;
        let actions = get_script_context_actions(&s);
        assert!(actions.iter().any(|a| a.id == "copy_path"));
    }

    #[test]
    fn cat12_agent_has_copy_content() {
        let mut s = ScriptInfo::new("My Agent", "/agent");
        s.is_agent = true;
        s.is_script = false;
        let actions = get_script_context_actions(&s);
        assert!(actions.iter().any(|a| a.id == "copy_content"));
    }

    #[test]
    fn cat12_agent_no_view_logs() {
        let mut s = ScriptInfo::new("My Agent", "/agent");
        s.is_agent = true;
        s.is_script = false;
        let actions = get_script_context_actions(&s);
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    #[test]
    fn cat12_agent_descriptions_mention_agent() {
        let mut s = ScriptInfo::new("My Agent", "/agent");
        s.is_agent = true;
        s.is_script = false;
        let actions = get_script_context_actions(&s);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert!(edit.description.as_ref().unwrap().contains("agent"));
        let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
        assert!(reveal.description.as_ref().unwrap().contains("agent"));
    }

    // =========================================================================
    // 13. Deeplink URL in description for scriptlet context
    // =========================================================================

    #[test]
    fn cat13_scriptlet_deeplink_description_contains_url() {
        let s = ScriptInfo::scriptlet("My Script", "/p", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&s, None);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        let desc = dl.description.as_ref().unwrap();
        assert!(desc.contains("scriptkit://run/my-script"), "Desc: {}", desc);
    }

    #[test]
    fn cat13_script_deeplink_description_format() {
        let s = ScriptInfo::new("Hello World", "/p");
        let actions = get_script_context_actions(&s);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        let desc = dl.description.as_ref().unwrap();
        assert!(
            desc.contains("scriptkit://run/hello-world"),
            "Desc: {}",
            desc
        );
    }


    // --- merged from tests_part_03.rs ---
    #[test]
    fn cat13_deeplink_name_special_chars_collapsed() {
        assert_eq!(to_deeplink_name("a!!b"), "a-b");
    }

    #[test]
    fn cat13_deeplink_name_leading_trailing_stripped() {
        assert_eq!(to_deeplink_name("  hello  "), "hello");
    }

    #[test]
    fn cat13_deeplink_name_unicode_preserved() {
        let result = to_deeplink_name("café");
        assert!(
            result.contains("caf"),
            "Should contain ascii part: {}",
            result
        );
        assert!(result.contains("é"), "Should preserve unicode: {}", result);
    }

    // =========================================================================
    // 14. Notes command bar create_quicklink and export actions
    // =========================================================================

    #[test]
    fn cat14_full_feature_has_create_quicklink() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "create_quicklink"));
    }

    #[test]
    fn cat14_create_quicklink_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = actions.iter().find(|a| a.id == "create_quicklink").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⇧⌘L"));
    }

    #[test]
    fn cat14_create_quicklink_icon_is_star() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = actions.iter().find(|a| a.id == "create_quicklink").unwrap();
        assert_eq!(a.icon, Some(IconName::Star));
    }

    #[test]
    fn cat14_export_action_present() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "export"));
    }

    #[test]
    fn cat14_export_section_is_export() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = actions.iter().find(|a| a.id == "export").unwrap();
        assert_eq!(a.section.as_deref(), Some("Export"));
    }

    #[test]
    fn cat14_trash_view_no_quicklink_no_export() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "create_quicklink"));
        assert!(!actions.iter().any(|a| a.id == "export"));
    }

    #[test]
    fn cat14_no_selection_no_quicklink_no_export() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "create_quicklink"));
        assert!(!actions.iter().any(|a| a.id == "export"));
    }

    // =========================================================================
    // 15. Action::new lowercase caching correctness
    // =========================================================================

    #[test]
    fn cat15_title_lower_is_lowercase() {
        let a = Action::new("id", "Hello World", None, ActionCategory::ScriptContext);
        assert_eq!(a.title_lower, "hello world");
    }

    #[test]
    fn cat15_description_lower_is_lowercase() {
        let a = Action::new(
            "id",
            "T",
            Some("Foo BAR".into()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(a.description_lower, Some("foo bar".into()));
    }

    #[test]
    fn cat15_description_lower_none_when_no_description() {
        let a = Action::new("id", "T", None, ActionCategory::ScriptContext);
        assert!(a.description_lower.is_none());
    }

    #[test]
    fn cat15_shortcut_lower_none_initially() {
        let a = Action::new("id", "T", None, ActionCategory::ScriptContext);
        assert!(a.shortcut_lower.is_none());
    }

    #[test]
    fn cat15_shortcut_lower_set_after_with_shortcut() {
        let a = Action::new("id", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
        assert_eq!(a.shortcut_lower, Some("⌘e".into()));
    }

    #[test]
    fn cat15_with_shortcut_opt_none_does_not_set() {
        let a = Action::new("id", "T", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
        assert!(a.shortcut_lower.is_none());
        assert!(a.shortcut.is_none());
    }

    #[test]
    fn cat15_with_shortcut_opt_some_sets() {
        let a = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘X".into()));
        assert_eq!(a.shortcut, Some("⌘X".into()));
        assert_eq!(a.shortcut_lower, Some("⌘x".into()));
    }

    // =========================================================================
    // 16. parse_shortcut_keycaps for special symbols
    // =========================================================================

    #[test]
    fn cat16_parse_cmd_c() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘C");
        assert_eq!(caps, vec!["⌘", "C"]);
    }

    #[test]
    fn cat16_parse_all_modifiers() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘⌃⌥⇧X");
        assert_eq!(caps, vec!["⌘", "⌃", "⌥", "⇧", "X"]);
    }

    #[test]
    fn cat16_parse_enter() {
        let caps = ActionsDialog::parse_shortcut_keycaps("↵");
        assert_eq!(caps, vec!["↵"]);
    }

    #[test]
    fn cat16_parse_escape() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⎋");
        assert_eq!(caps, vec!["⎋"]);
    }

    #[test]
    fn cat16_parse_arrows() {
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("↑"), vec!["↑"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("↓"), vec!["↓"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("←"), vec!["←"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("→"), vec!["→"]);
    }

    #[test]
    fn cat16_parse_space() {
        let caps = ActionsDialog::parse_shortcut_keycaps("␣");
        assert_eq!(caps, vec!["␣"]);
    }

    #[test]
    fn cat16_parse_tab() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⇥");
        assert_eq!(caps, vec!["⇥"]);
    }

    #[test]
    fn cat16_parse_backspace() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌫");
        assert_eq!(caps, vec!["⌫"]);
    }

    #[test]
    fn cat16_parse_lowercase_uppercased() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘a");
        assert_eq!(caps, vec!["⌘", "A"]);
    }

    #[test]
    fn cat16_parse_empty() {
        let caps = ActionsDialog::parse_shortcut_keycaps("");
        assert!(caps.is_empty());
    }

    // =========================================================================
    // 17. score_action boundary thresholds
    // =========================================================================

    #[test]
    fn cat17_prefix_match_100() {
        let a = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
        assert_eq!(ActionsDialog::score_action(&a, "edit"), 100);
    }

    #[test]
    fn cat17_contains_match_50() {
        let a = Action::new("id", "Copy Edit Path", None, ActionCategory::ScriptContext);
        assert_eq!(ActionsDialog::score_action(&a, "edit"), 50);
    }

    #[test]
    fn cat17_fuzzy_match_25() {
        let a = Action::new(
            "id",
            "Reveal in Finder",
            None,
            ActionCategory::ScriptContext,
        );
        // "rvf" is a subsequence of "reveal in finder" (r-e-v-e-a-l-_-i-n-_-f)
        assert_eq!(ActionsDialog::score_action(&a, "rvf"), 25);
    }

    #[test]
    fn cat17_description_bonus_15() {
        let a = Action::new(
            "id",
            "Open",
            Some("Edit file in editor".into()),
            ActionCategory::ScriptContext,
        );
        // "editor" not in title but in description
        assert_eq!(ActionsDialog::score_action(&a, "editor"), 15);
    }

    #[test]
    fn cat17_shortcut_bonus_10() {
        let a = Action::new("id", "Open", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
        // "⌘e" is in shortcut_lower
        assert_eq!(ActionsDialog::score_action(&a, "⌘e"), 10);
    }

    #[test]
    fn cat17_no_match_0() {
        let a = Action::new("id", "Open", None, ActionCategory::ScriptContext);
        assert_eq!(ActionsDialog::score_action(&a, "xyz"), 0);
    }

    #[test]
    fn cat17_prefix_plus_description_115() {
        let a = Action::new(
            "id",
            "Edit Script",
            Some("Edit the script file".into()),
            ActionCategory::ScriptContext,
        );
        // "edit" is prefix (100) + description contains "edit" (15)
        assert_eq!(ActionsDialog::score_action(&a, "edit"), 115);
    }

    // =========================================================================
    // 18. fuzzy_match edge cases
    // =========================================================================

    #[test]
    fn cat18_empty_needle_true() {
        assert!(ActionsDialog::fuzzy_match("anything", ""));
    }

    #[test]
    fn cat18_empty_haystack_false() {
        assert!(!ActionsDialog::fuzzy_match("", "a"));
    }

    #[test]
    fn cat18_both_empty_true() {
        assert!(ActionsDialog::fuzzy_match("", ""));
    }

    #[test]
    fn cat18_exact_match() {
        assert!(ActionsDialog::fuzzy_match("abc", "abc"));
    }

    #[test]
    fn cat18_subsequence() {
        assert!(ActionsDialog::fuzzy_match("abcdef", "ace"));
    }

    #[test]
    fn cat18_no_match() {
        assert!(!ActionsDialog::fuzzy_match("abc", "xyz"));
    }

    #[test]
    fn cat18_needle_longer_than_haystack() {
        assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
    }

    // =========================================================================
    // 19. build_grouped_items_static
    // =========================================================================

    #[test]
    fn cat19_empty_filtered_empty_grouped() {
        let actions: Vec<Action> = vec![];
        let filtered: Vec<usize> = vec![];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(grouped.is_empty());
    }

    #[test]
    fn cat19_headers_inserts_section_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Should have: S1 header, item 0, S2 header, item 1
        assert_eq!(grouped.len(), 4);
        assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(_)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[2], GroupedActionItem::SectionHeader(_)));
        assert!(matches!(grouped[3], GroupedActionItem::Item(1)));
    }

    #[test]
    fn cat19_separators_no_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        // No section headers
        assert_eq!(grouped.len(), 2);
        assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(1)));
    }

    #[test]
    fn cat19_none_style_no_headers() {
        let actions =
            vec![Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1")];
        let filtered = vec![0];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        assert_eq!(grouped.len(), 1);
        assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
    }

    #[test]
    fn cat19_same_section_no_duplicate_header() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S1"),
        ];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // S1 header, item 0, item 1 (no second header)
        assert_eq!(grouped.len(), 3);
    }

    // =========================================================================
    // 20. coerce_action_selection edge cases
    // =========================================================================

    #[test]
    fn cat20_empty_returns_none() {
        assert_eq!(coerce_action_selection(&[], 0), None);
    }

    #[test]
    fn cat20_on_item_returns_same() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    }

    #[test]
    fn cat20_header_searches_down() {
        let rows = vec![
            GroupedActionItem::SectionHeader("S".into()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn cat20_trailing_header_searches_up() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("S".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(0));
    }

    #[test]
    fn cat20_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".into()),
            GroupedActionItem::SectionHeader("B".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn cat20_out_of_bounds_clamped() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 100), Some(0));
    }

    // =========================================================================
    // 21. New chat actions structure
    // =========================================================================


    // --- merged from tests_part_04.rs ---
    #[test]
    fn cat21_empty_inputs_empty_actions() {
        let actions = get_new_chat_actions(&[], &[], &[]);
        assert!(actions.is_empty());
    }

    #[test]
    fn cat21_last_used_section() {
        let lu = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&lu, &[], &[]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
        assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
    }

    #[test]
    fn cat21_preset_section() {
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].section.as_deref(), Some("Presets"));
        assert!(
            actions[0].description.is_none(),
            "Presets have no description"
        );
    }

    #[test]
    fn cat21_models_section() {
        let models = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "GPT-4".into(),
            provider: "openai".into(),
            provider_display_name: "OpenAI".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].section.as_deref(), Some("Models"));
        assert_eq!(actions[0].icon, Some(IconName::Settings));
        assert_eq!(actions[0].description, Some("OpenAI".into()));
    }

    #[test]
    fn cat21_section_ordering() {
        let lu = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "A".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "g".into(),
            name: "G".into(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m2".into(),
            display_name: "B".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        }];
        let actions = get_new_chat_actions(&lu, &presets, &models);
        assert_eq!(actions.len(), 3);
        assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
        assert_eq!(actions[1].section.as_deref(), Some("Presets"));
        assert_eq!(actions[2].section.as_deref(), Some("Models"));
    }

    // =========================================================================
    // 22. Notes command bar auto_sizing toggle
    // =========================================================================

    #[test]
    fn cat22_auto_sizing_disabled_shows_enable() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "enable_auto_sizing"));
    }

    #[test]
    fn cat22_auto_sizing_enabled_hides_enable() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "enable_auto_sizing"));
    }

    #[test]
    fn cat22_auto_sizing_in_settings_section() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = actions
            .iter()
            .find(|a| a.id == "enable_auto_sizing")
            .unwrap();
        assert_eq!(a.section.as_deref(), Some("Settings"));
    }

    #[test]
    fn cat22_auto_sizing_icon_is_settings() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = actions
            .iter()
            .find(|a| a.id == "enable_auto_sizing")
            .unwrap();
        assert_eq!(a.icon, Some(IconName::Settings));
    }

    // =========================================================================
    // 23. File context FileType variants
    // =========================================================================

    #[test]
    fn cat23_document_and_image_same_file_actions() {
        let doc = FileInfo {
            path: "/t".into(),
            name: "t".into(),
            file_type: crate::file_search::FileType::Document,
            is_dir: false,
        };
        let img = FileInfo {
            path: "/t".into(),
            name: "t".into(),
            file_type: crate::file_search::FileType::Image,
            is_dir: false,
        };
        let doc_ids = action_ids(&get_file_context_actions(&doc));
        let img_ids = action_ids(&get_file_context_actions(&img));
        assert_eq!(doc_ids, img_ids, "FileType should not affect action list");
    }

    #[test]
    fn cat23_directory_different_from_file() {
        let file = FileInfo {
            path: "/t".into(),
            name: "t".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let dir = FileInfo {
            path: "/t".into(),
            name: "t".into(),
            file_type: crate::file_search::FileType::Directory,
            is_dir: true,
        };
        let file_ids = action_ids(&get_file_context_actions(&file));
        let dir_ids = action_ids(&get_file_context_actions(&dir));
        assert_ne!(
            file_ids, dir_ids,
            "Dir and file should have different actions"
        );
    }

    // =========================================================================
    // 24. Clipboard destructive actions always last three
    // =========================================================================

    #[test]
    fn cat24_text_last_three_destructive() {
        let actions = get_clipboard_history_context_actions(&make_text_entry());
        let ids = action_ids(&actions);
        let n = ids.len();
        assert_eq!(ids[n - 3], "clipboard_delete");
        assert_eq!(ids[n - 2], "clipboard_delete_multiple");
        assert_eq!(ids[n - 1], "clipboard_delete_all");
    }

    #[test]
    fn cat24_image_last_three_destructive() {
        let actions = get_clipboard_history_context_actions(&make_image_entry());
        let ids = action_ids(&actions);
        let n = ids.len();
        assert_eq!(ids[n - 3], "clipboard_delete");
        assert_eq!(ids[n - 2], "clipboard_delete_multiple");
        assert_eq!(ids[n - 1], "clipboard_delete_all");
    }

    #[test]
    fn cat24_paste_always_first() {
        let actions = get_clipboard_history_context_actions(&make_text_entry());
        assert_eq!(actions[0].id, "clipboard_paste");
    }

    #[test]
    fn cat24_copy_always_second() {
        let actions = get_clipboard_history_context_actions(&make_text_entry());
        assert_eq!(actions[1].id, "clipboard_copy");
    }

    // =========================================================================
    // 25. Note switcher icon hierarchy
    // =========================================================================

    #[test]
    fn cat25_pinned_gets_star_filled() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "T".into(),
            char_count: 0,
            is_current: false,
            is_pinned: true,
            preview: "".into(),
            relative_time: "".into(),
        };
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    #[test]
    fn cat25_current_gets_check() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "T".into(),
            char_count: 0,
            is_current: true,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        };
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].icon, Some(IconName::Check));
    }

    #[test]
    fn cat25_regular_gets_file() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "T".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        };
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].icon, Some(IconName::File));
    }

    #[test]
    fn cat25_pinned_overrides_current() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "T".into(),
            char_count: 0,
            is_current: true,
            is_pinned: true,
            preview: "".into(),
            relative_time: "".into(),
        };
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    // =========================================================================
    // 26. Note switcher section assignment
    // =========================================================================

    #[test]
    fn cat26_pinned_in_pinned_section() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "T".into(),
            char_count: 0,
            is_current: false,
            is_pinned: true,
            preview: "".into(),
            relative_time: "".into(),
        };
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
    }

    #[test]
    fn cat26_unpinned_in_recent_section() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "T".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        };
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn cat26_current_note_bullet_prefix() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "My Note".into(),
            char_count: 0,
            is_current: true,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        };
        let actions = get_note_switcher_actions(&[note]);
        assert!(actions[0].title.starts_with("• "));
    }

    #[test]
    fn cat26_non_current_no_bullet() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "My Note".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        };
        let actions = get_note_switcher_actions(&[note]);
        assert!(!actions[0].title.starts_with("• "));
    }

    #[test]
    fn cat26_empty_notes_placeholder() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "no_notes");
        assert_eq!(actions[0].icon, Some(IconName::Plus));
    }

    // =========================================================================
    // 27. Action builder chaining preserves fields
    // =========================================================================

    #[test]
    fn cat27_with_icon_preserves_other_fields() {
        let a = Action::new(
            "id",
            "Title",
            Some("Desc".into()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Copy);
        assert_eq!(a.id, "id");
        assert_eq!(a.title, "Title");
        assert_eq!(a.description, Some("Desc".into()));
        assert_eq!(a.icon, Some(IconName::Copy));
    }

    #[test]
    fn cat27_with_section_preserves_other_fields() {
        let a =
            Action::new("id", "Title", None, ActionCategory::ScriptContext).with_section("MySec");
        assert_eq!(a.section, Some("MySec".into()));
        assert_eq!(a.id, "id");
    }

    #[test]
    fn cat27_chaining_all_builders() {
        let a = Action::new(
            "id",
            "Title",
            Some("D".into()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘E")
        .with_icon(IconName::Settings)
        .with_section("S");
        assert_eq!(a.shortcut, Some("⌘E".into()));
        assert_eq!(a.icon, Some(IconName::Settings));
        assert_eq!(a.section, Some("S".into()));
        assert_eq!(a.title, "Title");
    }

    // =========================================================================
    // 28. Cross-context ID uniqueness
    // =========================================================================

    #[test]
    fn cat28_script_ids_unique() {
        let s = ScriptInfo::new("t", "/p");
        let actions = get_script_context_actions(&s);
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len(), "Script action IDs must be unique");
    }

    #[test]
    fn cat28_clipboard_ids_unique() {
        let actions = get_clipboard_history_context_actions(&make_text_entry());
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(
            ids.len(),
            actions.len(),
            "Clipboard action IDs must be unique"
        );
    }

    #[test]
    fn cat28_ai_ids_unique() {
        let actions = get_ai_command_bar_actions();
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len(), "AI action IDs must be unique");
    }

    #[test]
    fn cat28_path_ids_unique() {
        let info = PathInfo {
            path: "/t".into(),
            name: "t".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len(), "Path action IDs must be unique");
    }

    #[test]
    fn cat28_file_ids_unique() {
        let info = FileInfo {
            path: "/t".into(),
            name: "t".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len(), "File action IDs must be unique");
    }


    // --- merged from tests_part_05.rs ---
    #[test]
    fn cat28_notes_ids_unique() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len(), "Notes action IDs must be unique");
    }

    // =========================================================================
    // 29. has_action=false for all built-in actions
    // =========================================================================

    #[test]
    fn cat29_script_all_has_action_false() {
        let s = ScriptInfo::new("t", "/p");
        for a in &get_script_context_actions(&s) {
            assert!(!a.has_action, "{} should be false", a.id);
        }
    }

    #[test]
    fn cat29_clipboard_all_has_action_false() {
        for a in &get_clipboard_history_context_actions(&make_text_entry()) {
            assert!(!a.has_action, "{} should be false", a.id);
        }
    }

    #[test]
    fn cat29_ai_all_has_action_false() {
        for a in &get_ai_command_bar_actions() {
            assert!(!a.has_action, "{} should be false", a.id);
        }
    }

    #[test]
    fn cat29_path_all_has_action_false() {
        let info = PathInfo {
            path: "/t".into(),
            name: "t".into(),
            is_dir: false,
        };
        for a in &get_path_context_actions(&info) {
            assert!(!a.has_action, "{} should be false", a.id);
        }
    }

    #[test]
    fn cat29_file_all_has_action_false() {
        let info = FileInfo {
            path: "/t".into(),
            name: "t".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        for a in &get_file_context_actions(&info) {
            assert!(!a.has_action, "{} should be false", a.id);
        }
    }

    #[test]
    fn cat29_notes_all_has_action_false() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for a in &get_notes_command_bar_actions(&info) {
            assert!(!a.has_action, "{} should be false", a.id);
        }
    }

    // =========================================================================
    // 30. Ordering determinism
    // =========================================================================

    #[test]
    fn cat30_script_ordering_deterministic() {
        let s = ScriptInfo::new("t", "/p");
        let a = action_ids(&get_script_context_actions(&s));
        let b = action_ids(&get_script_context_actions(&s));
        assert_eq!(a, b);
    }

    #[test]
    fn cat30_clipboard_ordering_deterministic() {
        let a = action_ids(&get_clipboard_history_context_actions(&make_text_entry()));
        let b = action_ids(&get_clipboard_history_context_actions(&make_text_entry()));
        assert_eq!(a, b);
    }

    #[test]
    fn cat30_ai_ordering_deterministic() {
        let a = action_ids(&get_ai_command_bar_actions());
        let b = action_ids(&get_ai_command_bar_actions());
        assert_eq!(a, b);
    }

    #[test]
    fn cat30_notes_ordering_deterministic() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let a = action_ids(&get_notes_command_bar_actions(&info));
        let b = action_ids(&get_notes_command_bar_actions(&info));
        assert_eq!(a, b);
    }

    #[test]
    fn cat30_path_ordering_deterministic() {
        let info = PathInfo {
            path: "/t".into(),
            name: "t".into(),
            is_dir: false,
        };
        let a = action_ids(&get_path_context_actions(&info));
        let b = action_ids(&get_path_context_actions(&info));
        assert_eq!(a, b);
    }

}
