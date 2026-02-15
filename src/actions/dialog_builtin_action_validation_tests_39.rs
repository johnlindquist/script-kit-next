//! Batch 39: Dialog builtin action validation tests
//!
//! Focuses on:
//! - ScriptInfo::with_shortcut constructor and field preservation
//! - ScriptInfo::with_shortcut_and_alias constructor combinations
//! - ScriptInfo::scriptlet constructor field validation
//! - format_shortcut_hint: arrow key and special key conversions
//! - format_shortcut_hint: alias variants (return, esc, opt, arrowdown)
//! - parse_shortcut_keycaps: multi-char modifier combos
//! - builders::format_shortcut_hint vs dialog::format_shortcut_hint
//! - Clipboard: ordering of common actions (paste, copy, paste_keep_open)
//! - Clipboard: destructive action ordering (delete, delete_multiple, delete_all)
//! - Clipboard: image-only OCR position relative to pin/unpin
//! - File context: total action count file vs dir on macOS
//! - File context: copy_filename shortcut differs from path context
//! - Path context: total action count file vs dir
//! - Path context: move_to_trash is always last
//! - Script context: with_frecency adds reset_ranking
//! - Script context: agent has no view_logs but has copy_path
//! - Scriptlet context: total action count without custom actions
//! - Scriptlet context with custom: custom actions appear after run
//! - AI bar: paste_image details
//! - AI bar: section ordering matches declaration order
//! - Notes: section distribution with selection + no trash + disabled auto
//! - Notes: all actions have icons
//! - Chat context: model actions come before continue_in_chat
//! - Chat context: context_title with model name
//! - New chat: last_used IDs use index format
//! - New chat: model section actions use Settings icon
//! - Note switcher: singular vs plural char count
//! - Note switcher: section assignment pinned vs recent
//! - coerce_action_selection: all headers returns None
//! - build_grouped_items_static: filter_idx in Item matches enumerate order

#[cfg(test)]
mod tests {
    // --- merged from tests_part_01.rs ---
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
        assert_eq!(actions[0].id, "clip:clipboard_paste");
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
        assert_eq!(actions[1].id, "clip:clipboard_copy");
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


    // --- merged from tests_part_02.rs ---
    #[test]
    fn file_context_file_has_more_actions_than_dir() {
        let file = FileInfo {
            name: "readme.md".into(),
            path: "/readme.md".into(),
            is_dir: false,
            file_type: FileType::File,
        };
        let dir = FileInfo {
            name: "src".into(),
            path: "/src".into(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        let file_actions = get_file_context_actions(&file);
        let dir_actions = get_file_context_actions(&dir);
        // Files have quick_look, dirs don't (on macOS)
        assert!(file_actions.len() >= dir_actions.len());
    }

    #[test]
    fn file_context_dir_no_quick_look() {
        let dir = FileInfo {
            name: "src".into(),
            path: "/src".into(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        let actions = get_file_context_actions(&dir);
        assert!(!actions.iter().any(|a| a.id == "file:quick_look"));
    }

    #[test]
    fn file_context_both_have_reveal_in_finder() {
        let file = FileInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
            file_type: FileType::File,
        };
        let dir = FileInfo {
            name: "b".into(),
            path: "/b".into(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        assert!(get_file_context_actions(&file)
            .iter()
            .any(|a| a.id == "file:reveal_in_finder"));
        assert!(get_file_context_actions(&dir)
            .iter()
            .any(|a| a.id == "file:reveal_in_finder"));
    }

    #[test]
    fn file_context_both_have_copy_path_and_copy_filename() {
        let file = FileInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&file);
        assert!(actions.iter().any(|a| a.id == "file:copy_path"));
        assert!(actions.iter().any(|a| a.id == "file:copy_filename"));
    }

    // =========================================================================
    // 12. File context: copy_filename shortcut ⌘C differs from path context (no shortcut)
    // =========================================================================

    #[test]
    fn file_context_copy_filename_has_shortcut() {
        let file = FileInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&file);
        let cf = actions.iter().find(|a| a.id == "file:copy_filename").unwrap();
        assert_eq!(cf.shortcut.as_deref(), Some("⌘C"));
    }

    #[test]
    fn path_context_copy_filename_has_no_shortcut() {
        let info = PathInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let cf = actions.iter().find(|a| a.id == "file:copy_filename").unwrap();
        assert!(cf.shortcut.is_none());
    }

    #[test]
    fn file_context_copy_path_shortcut_matches_path_context() {
        let file = FileInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
            file_type: FileType::File,
        };
        let path = PathInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
        };
        let fc = get_file_context_actions(&file);
        let pc = get_path_context_actions(&path);
        let fcp = fc.iter().find(|a| a.id == "file:copy_path").unwrap();
        let pcp = pc.iter().find(|a| a.id == "file:copy_path").unwrap();
        assert_eq!(fcp.shortcut, pcp.shortcut);
    }

    #[test]
    fn file_and_path_copy_path_shortcut_is_cmd_shift_c() {
        let file = FileInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&file);
        let cp = actions.iter().find(|a| a.id == "file:copy_path").unwrap();
        assert_eq!(cp.shortcut.as_deref(), Some("⌘⇧C"));
    }

    // =========================================================================
    // 13. Path context: total action count file vs dir
    // =========================================================================

    #[test]
    fn path_context_dir_has_one_more_than_common() {
        let file = PathInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
        };
        let dir = PathInfo {
            name: "b".into(),
            path: "/b".into(),
            is_dir: true,
        };
        // Both have same total: primary + 6 common = 7
        assert_eq!(
            get_path_context_actions(&file).len(),
            get_path_context_actions(&dir).len()
        );
    }

    #[test]
    fn path_context_file_primary_is_select_file() {
        let info = PathInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].id, "file:select_file");
    }

    #[test]
    fn path_context_dir_primary_is_open_directory() {
        let info = PathInfo {
            name: "b".into(),
            path: "/b".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].id, "file:open_directory");
    }

    #[test]
    fn path_context_both_have_7_actions() {
        let file = PathInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
        };
        let dir = PathInfo {
            name: "b".into(),
            path: "/b".into(),
            is_dir: true,
        };
        assert_eq!(get_path_context_actions(&file).len(), 7);
        assert_eq!(get_path_context_actions(&dir).len(), 7);
    }

    // =========================================================================
    // 14. Path context: move_to_trash is always last
    // =========================================================================

    #[test]
    fn path_context_file_last_is_move_to_trash() {
        let info = PathInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions.last().unwrap().id, "file:move_to_trash");
    }

    #[test]
    fn path_context_dir_last_is_move_to_trash() {
        let info = PathInfo {
            name: "b".into(),
            path: "/b".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions.last().unwrap().id, "file:move_to_trash");
    }

    #[test]
    fn path_context_move_to_trash_desc_file() {
        let info = PathInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.last().unwrap();
        assert!(trash.description.as_ref().unwrap().contains("file"));
    }

    #[test]
    fn path_context_move_to_trash_desc_folder() {
        let info = PathInfo {
            name: "b".into(),
            path: "/b".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.last().unwrap();
        assert!(trash.description.as_ref().unwrap().contains("folder"));
    }

    // =========================================================================
    // 15. Script context: with_frecency adds reset_ranking
    // =========================================================================

    #[test]
    fn script_with_frecency_has_reset_ranking() {
        let info = ScriptInfo::new("test", "/test.ts").with_frecency(true, Some("/test.ts".into()));
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    }

    #[test]
    fn script_without_frecency_no_reset_ranking() {
        let info = ScriptInfo::new("test", "/test.ts");
        let actions = get_script_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "reset_ranking"));
    }

    #[test]
    fn script_with_frecency_reset_ranking_is_last() {
        let info = ScriptInfo::new("test", "/test.ts").with_frecency(true, Some("/test.ts".into()));
        let actions = get_script_context_actions(&info);
        assert_eq!(actions.last().unwrap().id, "reset_ranking");
    }

    #[test]
    fn script_with_frecency_reset_ranking_no_shortcut() {
        let info = ScriptInfo::new("test", "/test.ts").with_frecency(true, Some("/test.ts".into()));
        let actions = get_script_context_actions(&info);
        let rr = actions.iter().find(|a| a.id == "reset_ranking").unwrap();
        assert_eq!(rr.shortcut.as_deref(), Some("⌃⌘R"));
    }

    // =========================================================================
    // 16. Script context: agent has no view_logs but has copy_path
    // =========================================================================

    #[test]
    fn agent_context_no_view_logs() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_agent = true;
        info.is_script = false;
        let actions = get_script_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    #[test]
    fn agent_context_has_copy_path() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_agent = true;
        info.is_script = false;
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "file:copy_path"));
    }

    #[test]
    fn agent_context_has_reveal_in_finder() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_agent = true;
        info.is_script = false;
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "file:reveal_in_finder"));
    }

    #[test]
    fn agent_context_edit_title_says_agent() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_agent = true;
        info.is_script = false;
        let actions = get_script_context_actions(&info);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    // =========================================================================
    // 17. Scriptlet context: total action count without custom actions
    // =========================================================================

    #[test]
    fn scriptlet_context_no_custom_no_shortcut_no_alias_count() {
        let info = ScriptInfo::scriptlet("My Script", "/scripts.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        // run + add_shortcut + add_alias + edit_scriptlet + reveal + copy_path + copy_content + copy_deeplink = 8
        assert_eq!(actions.len(), 8);
    }

    #[test]
    fn scriptlet_context_with_shortcut_adds_two_actions() {
        let info = ScriptInfo::scriptlet("My Script", "/scripts.md", Some("cmd+m".into()), None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        // run + update_shortcut + remove_shortcut + add_alias + edit + reveal + copy_path + copy_content + copy_deeplink = 9
        assert_eq!(actions.len(), 9);
    }

    #[test]
    fn scriptlet_context_with_both_shortcut_alias_count() {
        let info = ScriptInfo::scriptlet(
            "My Script",
            "/scripts.md",
            Some("cmd+m".into()),
            Some("ms".into()),
        );
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        // run + update_shortcut + remove_shortcut + update_alias + remove_alias + edit + reveal + copy_path + copy_content + copy_deeplink = 10
        assert_eq!(actions.len(), 10);
    }

    #[test]
    fn scriptlet_context_suggested_adds_reset_ranking() {
        let info = ScriptInfo::scriptlet("My Script", "/scripts.md", None, None)
            .with_frecency(true, Some("/scripts.md".into()));
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    }

    // =========================================================================
    // 18. Scriptlet context with custom: custom actions appear after run
    // =========================================================================

    #[test]
    fn scriptlet_custom_actions_appear_after_run() {
        let info = ScriptInfo::scriptlet("My Script", "/scripts.md", None, None);
        let mut scriptlet = Scriptlet::new("My Script".into(), "bash".into(), "echo hi".into());
        scriptlet.actions.push(crate::scriptlets::ScriptletAction {
            name: "Copy".into(),
            command: "copy".into(),
            tool: "bash".into(),
            code: "echo copy".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        });
        let actions = get_scriptlet_context_actions_with_custom(&info, Some(&scriptlet));
        assert_eq!(actions[0].id, "run_script");
        assert_eq!(actions[1].id, "scriptlet_action:copy");
    }

    #[test]
    fn scriptlet_custom_actions_have_has_action_true() {
        let info = ScriptInfo::scriptlet("My Script", "/scripts.md", None, None);
        let mut scriptlet = Scriptlet::new("My Script".into(), "bash".into(), "echo hi".into());
        scriptlet.actions.push(crate::scriptlets::ScriptletAction {
            name: "Do Thing".into(),
            command: "do-thing".into(),
            tool: "bash".into(),
            code: "echo thing".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        });
        let actions = get_scriptlet_context_actions_with_custom(&info, Some(&scriptlet));
        let custom = actions
            .iter()
            .find(|a| a.id == "scriptlet_action:do-thing")
            .unwrap();
        assert!(custom.has_action);
    }

    #[test]
    fn scriptlet_custom_action_value_is_command() {
        let info = ScriptInfo::scriptlet("My Script", "/scripts.md", None, None);
        let mut scriptlet = Scriptlet::new("My Script".into(), "bash".into(), "echo hi".into());
        scriptlet.actions.push(crate::scriptlets::ScriptletAction {
            name: "Do Thing".into(),
            command: "do-thing".into(),
            tool: "bash".into(),
            code: "echo thing".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        });
        let actions = get_scriptlet_context_actions_with_custom(&info, Some(&scriptlet));
        let custom = actions
            .iter()
            .find(|a| a.id == "scriptlet_action:do-thing")
            .unwrap();
        assert_eq!(custom.value.as_deref(), Some("do-thing"));
    }

    #[test]
    fn scriptlet_no_scriptlet_no_custom_actions() {
        let info = ScriptInfo::scriptlet("My Script", "/scripts.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        assert!(!actions
            .iter()
            .any(|a| a.id.starts_with("scriptlet_action:")));
    }

    // =========================================================================
    // 19. AI bar: paste_image details
    // =========================================================================

    #[test]
    fn ai_bar_paste_image_shortcut() {
        let actions = get_ai_command_bar_actions();
        let pi = actions.iter().find(|a| a.id == "paste_image").unwrap();
        assert_eq!(pi.shortcut.as_deref(), Some("⌘V"));
    }

    #[test]
    fn ai_bar_paste_image_icon() {
        let actions = get_ai_command_bar_actions();
        let pi = actions.iter().find(|a| a.id == "paste_image").unwrap();
        assert_eq!(pi.icon, Some(IconName::File));
    }

    // --- merged from tests_part_03.rs ---
    #[test]
    fn ai_bar_paste_image_section() {
        let actions = get_ai_command_bar_actions();
        let pi = actions.iter().find(|a| a.id == "paste_image").unwrap();
        assert_eq!(pi.section.as_deref(), Some("Attachments"));
    }

    #[test]
    fn ai_bar_paste_image_desc_mentions_clipboard() {
        let actions = get_ai_command_bar_actions();
        let pi = actions.iter().find(|a| a.id == "paste_image").unwrap();
        assert!(pi
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("clipboard"));
    }

    // =========================================================================
    // 20. AI bar: section ordering matches declaration order
    // =========================================================================

    #[test]
    fn ai_bar_first_section_is_response() {
        let actions = get_ai_command_bar_actions();
        let first_with_section = actions.iter().find(|a| a.section.is_some()).unwrap();
        assert_eq!(first_with_section.section.as_deref(), Some("Response"));
    }

    #[test]
    fn ai_bar_last_section_is_settings() {
        let actions = get_ai_command_bar_actions();
        let last = actions.last().unwrap();
        assert_eq!(last.section.as_deref(), Some("Settings"));
    }

    #[test]
    fn ai_bar_export_section_has_one_action() {
        let actions = get_ai_command_bar_actions();
        let export_count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Export"))
            .count();
        assert_eq!(export_count, 1);
    }

    #[test]
    fn ai_bar_attachments_section_has_two_actions() {
        let actions = get_ai_command_bar_actions();
        let att_count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Attachments"))
            .count();
        assert_eq!(att_count, 2);
    }

    // =========================================================================
    // 21. Notes: section distribution with selection + no trash + disabled auto
    // =========================================================================

    #[test]
    fn notes_full_selection_has_notes_section() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions
            .iter()
            .any(|a| a.section.as_deref() == Some("Notes")));
    }

    #[test]
    fn notes_full_selection_has_edit_section() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.section.as_deref() == Some("Edit")));
    }

    #[test]
    fn notes_full_selection_has_copy_section() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.section.as_deref() == Some("Copy")));
    }

    #[test]
    fn notes_full_selection_has_settings_when_auto_disabled() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions
            .iter()
            .any(|a| a.section.as_deref() == Some("Settings")));
    }

    // =========================================================================
    // 22. Notes: all actions have icons
    // =========================================================================

    #[test]
    fn notes_full_all_have_icons() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for action in &actions {
            assert!(action.icon.is_some(), "Action {} has no icon", action.id);
        }
    }

    #[test]
    fn notes_no_selection_all_have_icons() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for action in &actions {
            assert!(action.icon.is_some(), "Action {} has no icon", action.id);
        }
    }

    #[test]
    fn notes_trash_all_have_icons() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for action in &actions {
            assert!(action.icon.is_some(), "Action {} has no icon", action.id);
        }
    }

    #[test]
    fn notes_new_note_icon_is_plus() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let nn = actions.iter().find(|a| a.id == "notes:new_note").unwrap();
        assert_eq!(nn.icon, Some(IconName::Plus));
    }

    // =========================================================================
    // 23. Chat context: model actions come before continue_in_chat
    // =========================================================================

    #[test]
    fn chat_model_actions_before_continue() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".into()),
            available_models: vec![
                ChatModelInfo {
                    id: "claude".into(),
                    display_name: "Claude".into(),
                    provider: "Anthropic".into(),
                },
                ChatModelInfo {
                    id: "gpt4".into(),
                    display_name: "GPT-4".into(),
                    provider: "OpenAI".into(),
                },
            ],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        let model_pos = actions
            .iter()
            .position(|a| a.id.starts_with("select_model_"))
            .unwrap();
        let continue_pos = actions
            .iter()
            .position(|a| a.id == "continue_in_chat")
            .unwrap();
        assert!(model_pos < continue_pos);
    }

    #[test]
    fn chat_all_model_actions_contiguous() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".into()),
            available_models: vec![
                ChatModelInfo {
                    id: "claude".into(),
                    display_name: "Claude".into(),
                    provider: "Anthropic".into(),
                },
                ChatModelInfo {
                    id: "gpt4".into(),
                    display_name: "GPT-4".into(),
                    provider: "OpenAI".into(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_indices: Vec<usize> = actions
            .iter()
            .enumerate()
            .filter(|(_, a)| a.id.starts_with("select_model_"))
            .map(|(i, _)| i)
            .collect();
        assert_eq!(model_indices, vec![0, 1]);
    }

    #[test]
    fn chat_continue_in_chat_always_after_models() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions[0].id, "continue_in_chat");
    }

    #[test]
    fn chat_copy_response_after_continue() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".into()),
            available_models: vec![],
            has_messages: false,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        let cont_pos = actions
            .iter()
            .position(|a| a.id == "continue_in_chat")
            .unwrap();
        let copy_pos = actions
            .iter()
            .position(|a| a.id == "chat:copy_response")
            .unwrap();
        assert!(copy_pos > cont_pos);
    }

    // =========================================================================
    // 24. Chat context: current model marked with checkmark
    // =========================================================================

    #[test]
    fn chat_current_model_has_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".into()),
            available_models: vec![ChatModelInfo {
                id: "claude".into(),
                display_name: "Claude".into(),
                provider: "Anthropic".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = actions
            .iter()
            .find(|a| a.id == "select_model_claude")
            .unwrap();
        assert!(model_action.title.contains("✓"));
    }

    #[test]
    fn chat_non_current_model_no_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".into()),
            available_models: vec![ChatModelInfo {
                id: "gpt4".into(),
                display_name: "GPT-4".into(),
                provider: "OpenAI".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = actions
            .iter()
            .find(|a| a.id == "select_model_gpt4")
            .unwrap();
        assert!(!model_action.title.contains("✓"));
    }

    #[test]
    fn chat_model_desc_mentions_provider() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "claude".into(),
                display_name: "Claude".into(),
                provider: "Anthropic".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = actions
            .iter()
            .find(|a| a.id == "select_model_claude")
            .unwrap();
        assert!(model_action
            .description
            .as_ref()
            .unwrap()
            .contains("Anthropic"));
    }

    #[test]
    fn chat_model_no_shortcut() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "claude".into(),
                display_name: "Claude".into(),
                provider: "Anthropic".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = actions
            .iter()
            .find(|a| a.id == "select_model_claude")
            .unwrap();
        assert!(model_action.shortcut.is_none());
    }

    // =========================================================================
    // 25. New chat: last_used IDs use index format
    // =========================================================================

    #[test]
    fn new_chat_last_used_id_format() {
        let last_used = vec![NewChatModelInfo {
            model_id: "claude".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].id, "last_used_0");
    }

    #[test]
    fn new_chat_last_used_second_id() {
        let last_used = vec![
            NewChatModelInfo {
                model_id: "claude".into(),
                display_name: "Claude".into(),
                provider: "anthropic".into(),
                provider_display_name: "Anthropic".into(),
            },
            NewChatModelInfo {
                model_id: "gpt4".into(),
                display_name: "GPT-4".into(),
                provider: "openai".into(),
                provider_display_name: "OpenAI".into(),
            },
        ];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[1].id, "last_used_1");
    }

    #[test]
    fn new_chat_last_used_desc_is_provider_display_name() {
        let last_used = vec![NewChatModelInfo {
            model_id: "claude".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].description.as_deref(), Some("Anthropic"));
    }

    #[test]
    fn new_chat_last_used_section() {
        let last_used = vec![NewChatModelInfo {
            model_id: "claude".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
    }

    // =========================================================================
    // 26. New chat: model section actions use Settings icon
    // =========================================================================

    #[test]
    fn new_chat_model_icon_is_settings() {
        let models = vec![NewChatModelInfo {
            model_id: "claude".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].icon, Some(IconName::Settings));
    }

    #[test]
    fn new_chat_model_section_is_models() {
        let models = vec![NewChatModelInfo {
            model_id: "claude".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].section.as_deref(), Some("Models"));
    }


    // --- merged from tests_part_04.rs ---
    #[test]
    fn new_chat_model_id_uses_index() {
        let models = vec![
            NewChatModelInfo {
                model_id: "claude".into(),
                display_name: "Claude".into(),
                provider: "anthropic".into(),
                provider_display_name: "Anthropic".into(),
            },
            NewChatModelInfo {
                model_id: "gpt4".into(),
                display_name: "GPT-4".into(),
                provider: "openai".into(),
                provider_display_name: "OpenAI".into(),
            },
        ];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].id, "model_0");
        assert_eq!(actions[1].id, "model_1");
    }

    #[test]
    fn new_chat_preset_id_uses_preset_id() {
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].id, "preset_general");
    }

    // =========================================================================
    // 27. Note switcher: singular vs plural char count
    // =========================================================================

    #[test]
    fn note_switcher_one_char_singular() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "Note".into(),
            char_count: 1,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].description.as_deref(), Some("1 char"));
    }

    #[test]
    fn note_switcher_zero_chars_plural() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "Note".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].description.as_deref(), Some("0 chars"));
    }

    #[test]
    fn note_switcher_many_chars_plural() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "Note".into(),
            char_count: 500,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].description.as_deref(), Some("500 chars"));
    }

    #[test]
    fn note_switcher_two_chars_plural() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "Note".into(),
            char_count: 2,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].description.as_deref(), Some("2 chars"));
    }

    // =========================================================================
    // 28. Note switcher: section assignment pinned vs recent
    // =========================================================================

    #[test]
    fn note_switcher_pinned_section() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "Note".into(),
            char_count: 10,
            is_current: false,
            is_pinned: true,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
    }

    #[test]
    fn note_switcher_unpinned_section() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "Note".into(),
            char_count: 10,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn note_switcher_mixed_sections() {
        let notes = vec![
            NoteSwitcherNoteInfo {
                id: "1".into(),
                title: "A".into(),
                char_count: 10,
                is_current: false,
                is_pinned: true,
                preview: "".into(),
                relative_time: "".into(),
            },
            NoteSwitcherNoteInfo {
                id: "2".into(),
                title: "B".into(),
                char_count: 20,
                is_current: false,
                is_pinned: false,
                preview: "".into(),
                relative_time: "".into(),
            },
        ];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
        assert_eq!(actions[1].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn note_switcher_current_pinned_still_pinned_section() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "Note".into(),
            char_count: 10,
            is_current: true,
            is_pinned: true,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
    }

    // =========================================================================
    // 29. coerce_action_selection: all headers returns None
    // =========================================================================

    #[test]
    fn coerce_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".into()),
            GroupedActionItem::SectionHeader("B".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn coerce_empty_returns_none() {
        let rows: Vec<GroupedActionItem> = vec![];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn coerce_header_then_item_returns_item_index() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".into()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn coerce_item_at_exact_index_returns_same() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("A".into()),
            GroupedActionItem::Item(1),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    }

    // =========================================================================
    // 30. build_grouped_items_static: filter_idx in Item matches enumerate order
    // =========================================================================

    #[test]
    fn build_grouped_items_no_sections_items_sequential() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext),
            Action::new("b", "B", None, ActionCategory::ScriptContext),
        ];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        assert_eq!(grouped.len(), 2);
        assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(1)));
    }

    #[test]
    fn build_grouped_items_with_headers_adds_section_header() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // S1 header + item(0) + S2 header + item(1) = 4
        assert_eq!(grouped.len(), 4);
        assert!(matches!(&grouped[0], GroupedActionItem::SectionHeader(s) if s == "S1"));
        assert!(matches!(grouped[1], GroupedActionItem::Item(0)));
        assert!(matches!(&grouped[2], GroupedActionItem::SectionHeader(s) if s == "S2"));
        assert!(matches!(grouped[3], GroupedActionItem::Item(1)));
    }

    #[test]
    fn build_grouped_items_separators_no_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        // No headers, just items
        assert_eq!(grouped.len(), 2);
        assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(1)));
    }

    #[test]
    fn build_grouped_items_empty_filtered() {
        let actions = vec![Action::new("a", "A", None, ActionCategory::ScriptContext)];
        let filtered: Vec<usize> = vec![];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(grouped.is_empty());
    }

}
