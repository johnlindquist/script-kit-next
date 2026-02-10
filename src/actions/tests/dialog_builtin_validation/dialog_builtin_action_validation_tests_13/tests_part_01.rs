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

