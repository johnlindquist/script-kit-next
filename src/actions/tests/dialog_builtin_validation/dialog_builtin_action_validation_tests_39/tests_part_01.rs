    use crate::actions::builders::*;
    use crate::actions::dialog::{build_grouped_items_static, ActionsDialog};
    use crate::actions::types::{Action, ActionCategory, ScriptInfo, SectionStyle};
    use crate::clipboard_history::ContentType;
    use crate::designs::icon_variations::IconName;
    use crate::file_search::{FileInfo, FileType};
    use crate::prompts::PathInfo;
    use crate::scriptlets::Scriptlet;

    use super::super::dialog::{coerce_action_selection, GroupedActionItem};

    // =========================================================================
    // 1. ScriptInfo::with_shortcut: preserves fields and sets shortcut
    // =========================================================================

    #[test]
    fn script_info_with_shortcut_sets_name_and_path() {
        let info =
            ScriptInfo::with_shortcut("my-script", "/scripts/my-script.ts", Some("cmd+k".into()));
        assert_eq!(info.name, "my-script");
        assert_eq!(info.path, "/scripts/my-script.ts");
    }

    #[test]
    fn script_info_with_shortcut_is_script_true() {
        let info = ScriptInfo::with_shortcut("x", "/x", Some("cmd+x".into()));
        assert!(info.is_script);
        assert!(!info.is_scriptlet);
        assert!(!info.is_agent);
    }

    #[test]
    fn script_info_with_shortcut_none_leaves_shortcut_none() {
        let info = ScriptInfo::with_shortcut("x", "/x", None);
        assert!(info.shortcut.is_none());
    }

    #[test]
    fn script_info_with_shortcut_defaults_verb_run() {
        let info = ScriptInfo::with_shortcut("x", "/x", Some("cmd+a".into()));
        assert_eq!(info.action_verb, "Run");
        assert!(info.alias.is_none());
    }

    // =========================================================================
    // 2. ScriptInfo::with_shortcut_and_alias: both fields set
    // =========================================================================

    #[test]
    fn script_info_with_shortcut_and_alias_both_set() {
        let info =
            ScriptInfo::with_shortcut_and_alias("x", "/x", Some("cmd+t".into()), Some("ts".into()));
        assert_eq!(info.shortcut, Some("cmd+t".into()));
        assert_eq!(info.alias, Some("ts".into()));
    }

    #[test]
    fn script_info_with_shortcut_and_alias_both_none() {
        let info = ScriptInfo::with_shortcut_and_alias("x", "/x", None, None);
        assert!(info.shortcut.is_none());
        assert!(info.alias.is_none());
    }

    #[test]
    fn script_info_with_shortcut_and_alias_defaults_not_suggested() {
        let info = ScriptInfo::with_shortcut_and_alias("x", "/x", None, None);
        assert!(!info.is_suggested);
        assert!(info.frecency_path.is_none());
    }

    #[test]
    fn script_info_with_shortcut_and_alias_is_script_true() {
        let info = ScriptInfo::with_shortcut_and_alias("x", "/x", None, None);
        assert!(info.is_script);
        assert!(!info.is_scriptlet);
    }

    // =========================================================================
    // 3. ScriptInfo::scriptlet: field validation
    // =========================================================================

    #[test]
    fn script_info_scriptlet_is_scriptlet_true_is_script_false() {
        let info = ScriptInfo::scriptlet("Copy URL", "/bundles/url.md", None, None);
        assert!(info.is_scriptlet);
        assert!(!info.is_script);
        assert!(!info.is_agent);
    }

    #[test]
    fn script_info_scriptlet_path_preserved() {
        let info = ScriptInfo::scriptlet("Copy URL", "/bundles/url.md#copy-url", None, None);
        assert_eq!(info.path, "/bundles/url.md#copy-url");
    }

    #[test]
    fn script_info_scriptlet_shortcut_and_alias_set() {
        let info =
            ScriptInfo::scriptlet("Copy URL", "/p", Some("cmd+u".into()), Some("url".into()));
        assert_eq!(info.shortcut, Some("cmd+u".into()));
        assert_eq!(info.alias, Some("url".into()));
    }

    #[test]
    fn script_info_scriptlet_verb_defaults_to_run() {
        let info = ScriptInfo::scriptlet("Copy URL", "/p", None, None);
        assert_eq!(info.action_verb, "Run");
    }

    // =========================================================================
    // 4. format_shortcut_hint: arrow keys and special keys
    // =========================================================================

    #[test]
    fn format_shortcut_hint_up_arrow() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+up"), "⌘↑");
    }

    #[test]
    fn format_shortcut_hint_down_arrow() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+down"), "⌘↓");
    }

    #[test]
    fn format_shortcut_hint_left_arrow() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+left"), "⌘←");
    }

    #[test]
    fn format_shortcut_hint_right_arrow() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+right"), "⌘→");
    }

    // =========================================================================
    // 5. format_shortcut_hint: alias variants (return, esc, opt, arrowdown)
    // =========================================================================

    #[test]
    fn format_shortcut_hint_return_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+return"), "⌘↵");
    }

    #[test]
    fn format_shortcut_hint_esc_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("esc"), "⎋");
    }

    #[test]
    fn format_shortcut_hint_opt_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("opt+c"), "⌥C");
    }

    #[test]
    fn format_shortcut_hint_arrowdown_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("arrowdown"), "↓");
    }

    // =========================================================================
    // 6. parse_shortcut_keycaps: multi-char modifier combos
    // =========================================================================

    #[test]
    fn parse_keycaps_cmd_shift_letter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⇧C");
        assert_eq!(keycaps, vec!["⌘", "⇧", "C"]);
    }

    #[test]
    fn parse_keycaps_ctrl_option_enter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌃⌥↵");
        assert_eq!(keycaps, vec!["⌃", "⌥", "↵"]);
    }

    #[test]
    fn parse_keycaps_single_escape() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⎋");
        assert_eq!(keycaps, vec!["⎋"]);
    }

    #[test]
    fn parse_keycaps_lowercase_letter_uppercased() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘k");
        assert_eq!(keycaps, vec!["⌘", "K"]);
    }

    // =========================================================================
    // 7. builders::format_shortcut_hint vs dialog::format_shortcut_hint
    // =========================================================================

    #[test]
    fn dialog_format_handles_command_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("command+c"), "⌘C");
    }

    #[test]
    fn dialog_format_handles_super_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("super+c"), "⌘C");
    }

    #[test]
    fn dialog_format_handles_control_full_word() {
        assert_eq!(ActionsDialog::format_shortcut_hint("control+x"), "⌃X");
    }

    #[test]
    fn dialog_format_handles_option_full_word() {
        assert_eq!(ActionsDialog::format_shortcut_hint("option+v"), "⌥V");
    }

    // =========================================================================
    // 8. Clipboard: ordering of common actions (paste, copy, paste_keep_open)
    // =========================================================================

    #[test]
    fn clipboard_first_action_is_paste() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[0].id, "clipboard_paste");
    }

    #[test]
    fn clipboard_second_action_is_copy() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[1].id, "clipboard_copy");
    }

    #[test]
    fn clipboard_third_action_is_paste_keep_open() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[2].id, "clipboard_paste_keep_open");
    }

    #[test]
    fn clipboard_fourth_action_is_share() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[3].id, "clipboard_share");
    }

    // =========================================================================
    // 9. Clipboard: destructive action ordering (delete, delete_multiple, delete_all)
    // =========================================================================

    #[test]
    fn clipboard_last_three_are_destructive() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let len = actions.len();
        assert_eq!(actions[len - 3].id, "clipboard_delete");
        assert_eq!(actions[len - 2].id, "clipboard_delete_multiple");
        assert_eq!(actions[len - 1].id, "clipboard_delete_all");
    }

    #[test]
    fn clipboard_delete_all_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let last = actions.last().unwrap();
        assert_eq!(last.shortcut.as_deref(), Some("⌃⇧X"));
    }

    #[test]
    fn clipboard_delete_all_desc_mentions_pinned() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let last = actions.last().unwrap();
        assert!(last.description.as_ref().unwrap().contains("pinned"));
    }

    #[test]
    fn clipboard_image_destructive_still_last_three() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "".into(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let len = actions.len();
        assert_eq!(actions[len - 3].id, "clipboard_delete");
        assert_eq!(actions[len - 2].id, "clipboard_delete_multiple");
        assert_eq!(actions[len - 1].id, "clipboard_delete_all");
    }

    // =========================================================================
    // 10. Clipboard: image OCR position relative to pin/unpin
    // =========================================================================

    #[test]
    fn clipboard_image_unpinned_pin_comes_before_ocr() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "".into(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let pin_pos = actions
            .iter()
            .position(|a| a.id == "clipboard_pin")
            .unwrap();
        let ocr_pos = actions
            .iter()
            .position(|a| a.id == "clipboard_ocr")
            .unwrap();
        assert!(
            pin_pos < ocr_pos,
            "pin should come before ocr in ordering: pin={} ocr={}",
            pin_pos,
            ocr_pos
        );
    }

    #[test]
    fn clipboard_image_pinned_unpin_comes_before_ocr() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Image,
            pinned: true,
            preview: "".into(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let unpin_pos = actions
            .iter()
            .position(|a| a.id == "clipboard_unpin")
            .unwrap();
        let ocr_pos = actions
            .iter()
            .position(|a| a.id == "clipboard_ocr")
            .unwrap();
        assert!(
            unpin_pos < ocr_pos,
            "unpin should come before ocr in ordering"
        );
    }

    #[test]
    fn clipboard_image_ocr_comes_before_save_snippet() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "".into(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ocr_pos = actions
            .iter()
            .position(|a| a.id == "clipboard_ocr")
            .unwrap();
        let snippet_pos = actions
            .iter()
            .position(|a| a.id == "clipboard_save_snippet")
            .unwrap();
        assert!(ocr_pos < snippet_pos);
    }

    #[test]
    fn clipboard_text_has_no_ocr() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hi".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(!actions.iter().any(|a| a.id == "clipboard_ocr"));
    }

    // =========================================================================
    // 11. File context: total action count file vs dir on macOS
    // =========================================================================

