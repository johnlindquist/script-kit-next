//! Batch 43: Dialog builtin action validation tests
//!
//! Focuses on:
//! - ScriptInfo::scriptlet constructor field validation
//! - ScriptInfo::builtin constructor sets empty path
//! - ScriptInfo::with_frecency chaining preserves original fields
//! - ScriptInfo::with_all sets is_agent false by default
//! - Action::new defaults: has_action false, value None, icon None, section None
//! - Action::with_section sets section correctly
//! - Action chaining: with_shortcut + with_icon + with_section
//! - builders format_shortcut_hint: specific .replace chain ordering
//! - to_deeplink_name: underscore and dot normalization
//! - to_deeplink_name: consecutive separator collapse
//! - Clipboard: text entry IDs are all clipboard_ prefixed
//! - Clipboard: image vs text action ID differences
//! - Clipboard: delete_multiple shortcut ⇧⌘X
//! - Clipboard: delete_all shortcut ⌃⇧X
//! - File context: show_info action on macOS
//! - File context: open_with shortcut ⌘O
//! - Path context: select_file desc says "Submit"
//! - Path context: open_directory desc says "Navigate"
//! - Script context: edit_script shortcut ⌘E
//! - Script context: view_logs shortcut ⌘L
//! - Script context: reveal_in_finder shortcut ⌘⇧F
//! - Script context: copy_path shortcut ⌘⇧C
//! - Scriptlet context with_custom: edit_scriptlet desc mentions $EDITOR
//! - AI bar: add_attachment shortcut ⇧⌘A
//! - AI bar: export section has exactly 1 action
//! - Notes: create_quicklink shortcut ⇧⌘L and icon Star
//! - Notes: copy_deeplink shortcut ⇧⌘D and icon ArrowRight
//! - Chat context: model ID format uses model.id not index
//! - coerce_action_selection: ix beyond rows.len() clamped
//! - build_grouped_items_static: Separators tracks category changes

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
    // 1. ScriptInfo::scriptlet constructor: is_script false, is_scriptlet true
    // =========================================================================

    #[test]
    fn scriptlet_constructor_is_script_is_false() {
        let s = ScriptInfo::scriptlet("Test", "/path.md", None, None);
        assert!(!s.is_script);
    }

    #[test]
    fn scriptlet_constructor_is_scriptlet_is_true() {
        let s = ScriptInfo::scriptlet("Test", "/path.md", None, None);
        assert!(s.is_scriptlet);
    }

    #[test]
    fn scriptlet_constructor_is_agent_is_false() {
        let s = ScriptInfo::scriptlet("Test", "/path.md", None, None);
        assert!(!s.is_agent);
    }

    #[test]
    fn scriptlet_constructor_action_verb_defaults_run() {
        let s = ScriptInfo::scriptlet("Test", "/path.md", None, None);
        assert_eq!(s.action_verb, "Run");
    }

    // =========================================================================
    // 2. ScriptInfo::builtin constructor: path is empty string
    // =========================================================================

    #[test]
    fn builtin_constructor_path_is_empty() {
        let b = ScriptInfo::builtin("App Launcher");
        assert!(b.path.is_empty());
    }

    #[test]
    fn builtin_constructor_is_script_false() {
        let b = ScriptInfo::builtin("App Launcher");
        assert!(!b.is_script);
    }

    #[test]
    fn builtin_constructor_is_scriptlet_false() {
        let b = ScriptInfo::builtin("App Launcher");
        assert!(!b.is_scriptlet);
    }

    #[test]
    fn builtin_constructor_is_agent_false() {
        let b = ScriptInfo::builtin("App Launcher");
        assert!(!b.is_agent);
    }

    // =========================================================================
    // 3. ScriptInfo::with_frecency chaining preserves all original fields
    // =========================================================================

    #[test]
    fn with_frecency_preserves_name() {
        let s = ScriptInfo::new("my-script", "/p.ts").with_frecency(true, Some("/p".into()));
        assert_eq!(s.name, "my-script");
    }

    #[test]
    fn with_frecency_preserves_path() {
        let s = ScriptInfo::new("my-script", "/p.ts").with_frecency(true, Some("/p".into()));
        assert_eq!(s.path, "/p.ts");
    }

    #[test]
    fn with_frecency_preserves_is_script() {
        let s = ScriptInfo::new("my-script", "/p.ts").with_frecency(true, None);
        assert!(s.is_script);
    }

    #[test]
    fn with_frecency_sets_is_suggested_and_path() {
        let s = ScriptInfo::new("x", "/x").with_frecency(true, Some("/fp".into()));
        assert!(s.is_suggested);
        assert_eq!(s.frecency_path, Some("/fp".to_string()));
    }

    // =========================================================================
    // 4. ScriptInfo::with_all sets is_agent false and is_scriptlet false
    // =========================================================================

    #[test]
    fn with_all_is_agent_defaults_false() {
        let s = ScriptInfo::with_all("A", "/a", true, "Run", None, None);
        assert!(!s.is_agent);
    }

    #[test]
    fn with_all_is_scriptlet_defaults_false() {
        let s = ScriptInfo::with_all("A", "/a", true, "Run", None, None);
        assert!(!s.is_scriptlet);
    }

    #[test]
    fn with_all_is_suggested_defaults_false() {
        let s = ScriptInfo::with_all("A", "/a", true, "Run", None, None);
        assert!(!s.is_suggested);
    }

    #[test]
    fn with_all_frecency_path_defaults_none() {
        let s = ScriptInfo::with_all("A", "/a", true, "Run", None, None);
        assert!(s.frecency_path.is_none());
    }

    // =========================================================================
    // 5. Action::new defaults: has_action false, value None, icon None, section None
    // =========================================================================

    #[test]
    fn action_new_has_action_false() {
        let a = Action::new("id", "title", None, ActionCategory::ScriptContext);
        assert!(!a.has_action);
    }

    #[test]
    fn action_new_value_none() {
        let a = Action::new("id", "title", None, ActionCategory::ScriptContext);
        assert!(a.value.is_none());
    }

    #[test]
    fn action_new_icon_none() {
        let a = Action::new("id", "title", None, ActionCategory::ScriptContext);
        assert!(a.icon.is_none());
    }

    #[test]
    fn action_new_section_none() {
        let a = Action::new("id", "title", None, ActionCategory::ScriptContext);
        assert!(a.section.is_none());
    }

    // =========================================================================
    // 6. Action::with_section sets section correctly
    // =========================================================================

    #[test]
    fn with_section_sets_value() {
        let a = Action::new("id", "title", None, ActionCategory::ScriptContext)
            .with_section("Response");
        assert_eq!(a.section, Some("Response".to_string()));
    }

    #[test]
    fn with_section_overwrites_previous() {
        let a = Action::new("id", "title", None, ActionCategory::ScriptContext)
            .with_section("A")
            .with_section("B");
        assert_eq!(a.section, Some("B".to_string()));
    }

    #[test]
    fn with_section_preserves_id() {
        let a =
            Action::new("test_id", "title", None, ActionCategory::ScriptContext).with_section("S");
        assert_eq!(a.id, "test_id");
    }

    #[test]
    fn with_section_preserves_shortcut() {
        let a = Action::new("id", "title", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘C")
            .with_section("S");
        assert_eq!(a.shortcut, Some("⌘C".to_string()));
    }

    // =========================================================================
    // 7. Action chaining: with_shortcut + with_icon + with_section
    // =========================================================================

    #[test]
    fn chained_all_three_shortcut_set() {
        let a = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘X")
            .with_icon(IconName::Copy)
            .with_section("Sec");
        assert_eq!(a.shortcut, Some("⌘X".to_string()));
    }

    #[test]
    fn chained_all_three_icon_set() {
        let a = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘X")
            .with_icon(IconName::Copy)
            .with_section("Sec");
        assert_eq!(a.icon, Some(IconName::Copy));
    }

    #[test]
    fn chained_all_three_section_set() {
        let a = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘X")
            .with_icon(IconName::Copy)
            .with_section("Sec");
        assert_eq!(a.section, Some("Sec".to_string()));
    }

    #[test]
    fn chained_all_three_title_lower_preserved() {
        let a = Action::new("id", "My Title", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘X")
            .with_icon(IconName::Copy)
            .with_section("Sec");
        assert_eq!(a.title_lower, "my title");
    }

    // =========================================================================
    // 8. builders format_shortcut_hint: .replace chain ordering
    // =========================================================================

    #[test]
    fn builders_format_cmd_c() {
        let actions = get_ai_command_bar_actions();
        // copy_response uses "⇧⌘C" directly (already formatted)
        let copy = actions.iter().find(|a| a.id == "copy_response").unwrap();
        assert_eq!(copy.shortcut, Some("⇧⌘C".to_string()));
    }

    #[test]
    fn builders_format_submit_enter() {
        let actions = get_ai_command_bar_actions();
        let submit = actions.iter().find(|a| a.id == "submit").unwrap();
        assert_eq!(submit.shortcut, Some("↵".to_string()));
    }

    #[test]
    fn builders_format_delete_chat_shortcut() {
        let actions = get_ai_command_bar_actions();
        let del = actions.iter().find(|a| a.id == "delete_chat").unwrap();
        assert_eq!(del.shortcut, Some("⌘⌫".to_string()));
    }

    #[test]
    fn builders_format_new_chat_shortcut() {
        let actions = get_ai_command_bar_actions();
        let nc = actions.iter().find(|a| a.id == "new_chat").unwrap();
        assert_eq!(nc.shortcut, Some("⌘N".to_string()));
    }

    // =========================================================================
    // 9. to_deeplink_name: underscore and dot normalization
    // =========================================================================

    #[test]
    fn deeplink_underscore_becomes_hyphen() {
        assert_eq!(to_deeplink_name("my_script"), "my-script");
    }

    #[test]
    fn deeplink_dot_becomes_hyphen() {
        assert_eq!(to_deeplink_name("my.script"), "my-script");
    }

    #[test]
    fn deeplink_mixed_separators_normalized() {
        assert_eq!(to_deeplink_name("my_script-name.ts"), "my-script-name-ts");
    }

    #[test]
    fn deeplink_preserves_numbers() {
        assert_eq!(to_deeplink_name("test123"), "test123");
    }

    // =========================================================================
    // 10. to_deeplink_name: consecutive separator collapse
    // =========================================================================

    #[test]
    fn deeplink_consecutive_spaces_collapse() {
        assert_eq!(to_deeplink_name("my   script"), "my-script");
    }

    #[test]
    fn deeplink_consecutive_mixed_collapse() {
        assert_eq!(to_deeplink_name("my__..--script"), "my-script");
    }

    #[test]
    fn deeplink_leading_trailing_trimmed() {
        assert_eq!(to_deeplink_name(" my script "), "my-script");
    }

    #[test]
    fn deeplink_single_word_lowercased() {
        assert_eq!(to_deeplink_name("Hello"), "hello");
    }

    // =========================================================================
    // 11. Clipboard: text entry IDs are all clipboard_ prefixed
    // =========================================================================

    #[test]
    fn clipboard_text_all_ids_have_clipboard_prefix() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        for a in &actions {
            assert!(
                a.id.starts_with("clipboard_"),
                "ID '{}' should start with 'clipboard_'",
                a.id
            );
        }
    }

    #[test]
    fn clipboard_text_unpinned_has_pin_id() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_pin"));
    }

    #[test]
    fn clipboard_text_pinned_has_unpin_id() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_unpin"));
    }

    #[test]
    fn clipboard_text_has_no_ocr() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(!actions.iter().any(|a| a.id == "clipboard_ocr"));
    }

    // =========================================================================
    // 12. Clipboard: image vs text action ID differences
    // =========================================================================

    #[test]
    fn clipboard_image_has_ocr() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_ocr"));
    }

    #[test]
    fn clipboard_image_has_open_with() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_open_with"));
    }

    #[test]
    fn clipboard_text_has_no_open_with() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(!actions.iter().any(|a| a.id == "clipboard_open_with"));
    }

    #[test]
    fn clipboard_image_has_annotate_cleanshot() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions
            .iter()
            .any(|a| a.id == "clipboard_annotate_cleanshot"));
    }

    // =========================================================================
    // 13. Clipboard: delete_multiple shortcut is ⇧⌘X
    // =========================================================================

    #[test]
    fn clipboard_delete_multiple_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
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
    fn clipboard_delete_multiple_title() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
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

    #[test]
    fn clipboard_delete_multiple_desc_mentions_filter() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let dm = actions
            .iter()
            .find(|a| a.id == "clipboard_delete_multiple")
            .unwrap();
        assert!(dm
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("filter"));
    }

    #[test]
    fn clipboard_delete_multiple_present_for_image() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_delete_multiple"));
    }

    // =========================================================================
    // 14. Clipboard: delete_all shortcut is ⌃⇧X
    // =========================================================================

    #[test]
    fn clipboard_delete_all_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
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
    fn clipboard_delete_all_title() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let da = actions
            .iter()
            .find(|a| a.id == "clipboard_delete_all")
            .unwrap();
        assert_eq!(da.title, "Delete All Entries");
    }

    #[test]
    fn clipboard_delete_all_desc_mentions_pinned() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let da = actions
            .iter()
            .find(|a| a.id == "clipboard_delete_all")
            .unwrap();
        assert!(da
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("pinned"));
    }

    #[test]
    fn clipboard_delete_all_is_last_action() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions.last().unwrap().id, "clipboard_delete_all");
    }

    // =========================================================================
    // 15. File context: show_info action on macOS
    // =========================================================================

    #[test]
    fn file_context_has_show_info_for_file() {
        let file = FileInfo {
            name: "test.txt".into(),
            path: "/tmp/test.txt".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        assert!(actions.iter().any(|a| a.id == "show_info"));
    }

    #[test]
    fn file_context_show_info_shortcut() {
        let file = FileInfo {
            name: "test.txt".into(),
            path: "/tmp/test.txt".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let si = actions.iter().find(|a| a.id == "show_info").unwrap();
        assert_eq!(si.shortcut, Some("⌘I".to_string()));
    }

    #[test]
    fn file_context_show_info_title() {
        let file = FileInfo {
            name: "test.txt".into(),
            path: "/tmp/test.txt".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let si = actions.iter().find(|a| a.id == "show_info").unwrap();
        assert_eq!(si.title, "Get Info");
    }

    #[test]
    fn file_context_show_info_present_for_dir() {
        let dir = FileInfo {
            name: "docs".into(),
            path: "/tmp/docs".into(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&dir);
        assert!(actions.iter().any(|a| a.id == "show_info"));
    }

    // =========================================================================
    // 16. File context: open_with shortcut ⌘O
    // =========================================================================

    #[test]
    fn file_context_open_with_shortcut() {
        let file = FileInfo {
            name: "test.txt".into(),
            path: "/tmp/test.txt".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let ow = actions.iter().find(|a| a.id == "open_with").unwrap();
        assert_eq!(ow.shortcut, Some("⌘O".to_string()));
    }

    #[test]
    fn file_context_open_with_title() {
        let file = FileInfo {
            name: "test.txt".into(),
            path: "/tmp/test.txt".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let ow = actions.iter().find(|a| a.id == "open_with").unwrap();
        assert_eq!(ow.title, "Open With...");
    }

    #[test]
    fn file_context_open_with_desc_mentions_application() {
        let file = FileInfo {
            name: "test.txt".into(),
            path: "/tmp/test.txt".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let ow = actions.iter().find(|a| a.id == "open_with").unwrap();
        assert!(ow
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("application"));
    }

    #[test]
    fn file_context_open_with_present_for_dir() {
        let dir = FileInfo {
            name: "docs".into(),
            path: "/tmp/docs".into(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&dir);
        assert!(actions.iter().any(|a| a.id == "open_with"));
    }

    // =========================================================================
    // 17. Path context: select_file desc says "Submit"
    // =========================================================================

    #[test]
    fn path_context_select_file_desc_contains_submit() {
        let p = PathInfo {
            name: "test.txt".into(),
            path: "/tmp/test.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&p);
        let sf = actions.iter().find(|a| a.id == "select_file").unwrap();
        assert!(sf.description.as_ref().unwrap().contains("Submit"));
    }

    #[test]
    fn path_context_select_file_title_quotes_name() {
        let p = PathInfo {
            name: "readme.md".into(),
            path: "/readme.md".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&p);
        let sf = actions.iter().find(|a| a.id == "select_file").unwrap();
        assert!(sf.title.contains("\"readme.md\""));
    }

    #[test]
    fn path_context_file_has_no_open_directory() {
        let p = PathInfo {
            name: "test.txt".into(),
            path: "/tmp/test.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&p);
        assert!(!actions.iter().any(|a| a.id == "open_directory"));
    }

    #[test]
    fn path_context_file_primary_is_first() {
        let p = PathInfo {
            name: "test.txt".into(),
            path: "/tmp/test.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&p);
        assert_eq!(actions[0].id, "select_file");
    }

    // =========================================================================
    // 18. Path context: open_directory desc says "Navigate"
    // =========================================================================

    #[test]
    fn path_context_open_directory_desc_contains_navigate() {
        let p = PathInfo {
            name: "docs".into(),
            path: "/tmp/docs".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&p);
        let od = actions.iter().find(|a| a.id == "open_directory").unwrap();
        assert!(od.description.as_ref().unwrap().contains("Navigate"));
    }

    #[test]
    fn path_context_open_directory_title_quotes_name() {
        let p = PathInfo {
            name: "src".into(),
            path: "/src".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&p);
        let od = actions.iter().find(|a| a.id == "open_directory").unwrap();
        assert!(od.title.contains("\"src\""));
    }

    #[test]
    fn path_context_dir_has_no_select_file() {
        let p = PathInfo {
            name: "docs".into(),
            path: "/tmp/docs".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&p);
        assert!(!actions.iter().any(|a| a.id == "select_file"));
    }

    #[test]
    fn path_context_dir_primary_is_first() {
        let p = PathInfo {
            name: "docs".into(),
            path: "/tmp/docs".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&p);
        assert_eq!(actions[0].id, "open_directory");
    }

    // =========================================================================
    // 19. Script context: edit_script shortcut ⌘E
    // =========================================================================

    #[test]
    fn script_edit_shortcut_cmd_e() {
        let s = ScriptInfo::new("test", "/p/test.ts");
        let actions = get_script_context_actions(&s);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.shortcut, Some("⌘E".to_string()));
    }

    #[test]
    fn script_edit_title_is_edit_script() {
        let s = ScriptInfo::new("test", "/p/test.ts");
        let actions = get_script_context_actions(&s);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Script");
    }

    #[test]
    fn script_edit_desc_mentions_editor() {
        let s = ScriptInfo::new("test", "/p/test.ts");
        let actions = get_script_context_actions(&s);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert!(edit.description.as_ref().unwrap().contains("$EDITOR"));
    }

    #[test]
    fn script_edit_not_present_for_builtin() {
        let b = ScriptInfo::builtin("Test");
        let actions = get_script_context_actions(&b);
        assert!(!actions.iter().any(|a| a.id == "edit_script"));
    }

    // =========================================================================
    // 20. Script context: view_logs shortcut ⌘L
    // =========================================================================

    #[test]
    fn script_view_logs_shortcut_cmd_l() {
        let s = ScriptInfo::new("test", "/p/test.ts");
        let actions = get_script_context_actions(&s);
        let vl = actions.iter().find(|a| a.id == "view_logs").unwrap();
        assert_eq!(vl.shortcut, Some("⌘L".to_string()));
    }

    #[test]
    fn script_view_logs_title() {
        let s = ScriptInfo::new("test", "/p/test.ts");
        let actions = get_script_context_actions(&s);
        let vl = actions.iter().find(|a| a.id == "view_logs").unwrap();
        assert_eq!(vl.title, "View Logs");
    }

    #[test]
    fn script_view_logs_desc_mentions_logs() {
        let s = ScriptInfo::new("test", "/p/test.ts");
        let actions = get_script_context_actions(&s);
        let vl = actions.iter().find(|a| a.id == "view_logs").unwrap();
        assert!(vl
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("log"));
    }

    #[test]
    fn script_view_logs_not_present_for_scriptlet() {
        let s = ScriptInfo::scriptlet("Test", "/p.md", None, None);
        let actions = get_script_context_actions(&s);
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    // =========================================================================
    // 21. Script context: reveal_in_finder shortcut ⌘⇧F
    // =========================================================================

    #[test]
    fn script_reveal_shortcut_cmd_shift_f() {
        let s = ScriptInfo::new("test", "/p/test.ts");
        let actions = get_script_context_actions(&s);
        let r = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
        assert_eq!(r.shortcut, Some("⌘⇧F".to_string()));
    }

    #[test]
    fn script_reveal_title() {
        let s = ScriptInfo::new("test", "/p/test.ts");
        let actions = get_script_context_actions(&s);
        let r = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
        assert_eq!(r.title, "Reveal in Finder");
    }

    #[test]
    fn script_reveal_desc_mentions_finder() {
        let s = ScriptInfo::new("test", "/p/test.ts");
        let actions = get_script_context_actions(&s);
        let r = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
        assert!(r
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("finder"));
    }

    #[test]
    fn script_reveal_not_present_for_builtin() {
        let b = ScriptInfo::builtin("Test");
        let actions = get_script_context_actions(&b);
        assert!(!actions.iter().any(|a| a.id == "reveal_in_finder"));
    }

    // =========================================================================
    // 22. Script context: copy_path shortcut ⌘⇧C
    // =========================================================================

    #[test]
    fn script_copy_path_shortcut_cmd_shift_c() {
        let s = ScriptInfo::new("test", "/p/test.ts");
        let actions = get_script_context_actions(&s);
        let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
        assert_eq!(cp.shortcut, Some("⌘⇧C".to_string()));
    }

    #[test]
    fn script_copy_path_title() {
        let s = ScriptInfo::new("test", "/p/test.ts");
        let actions = get_script_context_actions(&s);
        let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
        assert_eq!(cp.title, "Copy Path");
    }

    #[test]
    fn script_copy_path_desc_mentions_clipboard() {
        let s = ScriptInfo::new("test", "/p/test.ts");
        let actions = get_script_context_actions(&s);
        let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
        assert!(cp
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("clipboard"));
    }

    #[test]
    fn script_copy_path_not_present_for_builtin() {
        let b = ScriptInfo::builtin("Test");
        let actions = get_script_context_actions(&b);
        assert!(!actions.iter().any(|a| a.id == "copy_path"));
    }

    // =========================================================================
    // 23. Scriptlet context with_custom: edit_scriptlet desc mentions $EDITOR
    // =========================================================================

    #[test]
    fn scriptlet_with_custom_edit_desc_mentions_editor() {
        let info = ScriptInfo::scriptlet("Test", "/p.md", None, None);
        let scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo hi".to_string(),
        );
        let actions = get_scriptlet_context_actions_with_custom(&info, Some(&scriptlet));
        let edit = actions.iter().find(|a| a.id == "edit_scriptlet").unwrap();
        assert!(edit.description.as_ref().unwrap().contains("$EDITOR"));
    }

    #[test]
    fn scriptlet_with_custom_edit_shortcut_cmd_e() {
        let info = ScriptInfo::scriptlet("Test", "/p.md", None, None);
        let scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo hi".to_string(),
        );
        let actions = get_scriptlet_context_actions_with_custom(&info, Some(&scriptlet));
        let edit = actions.iter().find(|a| a.id == "edit_scriptlet").unwrap();
        assert_eq!(edit.shortcut, Some("⌘E".to_string()));
    }

    #[test]
    fn scriptlet_with_custom_reveal_shortcut_cmd_shift_f() {
        let info = ScriptInfo::scriptlet("Test", "/p.md", None, None);
        let scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo hi".to_string(),
        );
        let actions = get_scriptlet_context_actions_with_custom(&info, Some(&scriptlet));
        let reveal = actions
            .iter()
            .find(|a| a.id == "reveal_scriptlet_in_finder")
            .unwrap();
        assert_eq!(reveal.shortcut, Some("⌘⇧F".to_string()));
    }

    #[test]
    fn scriptlet_with_custom_copy_scriptlet_path_shortcut() {
        let info = ScriptInfo::scriptlet("Test", "/p.md", None, None);
        let scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo hi".to_string(),
        );
        let actions = get_scriptlet_context_actions_with_custom(&info, Some(&scriptlet));
        let cp = actions
            .iter()
            .find(|a| a.id == "copy_scriptlet_path")
            .unwrap();
        assert_eq!(cp.shortcut, Some("⌘⇧C".to_string()));
    }

    // =========================================================================
    // 24. AI bar: add_attachment shortcut ⇧⌘A
    // =========================================================================

    #[test]
    fn ai_bar_add_attachment_shortcut() {
        let actions = get_ai_command_bar_actions();
        let aa = actions.iter().find(|a| a.id == "add_attachment").unwrap();
        assert_eq!(aa.shortcut, Some("⇧⌘A".to_string()));
    }

    #[test]
    fn ai_bar_add_attachment_icon_plus() {
        let actions = get_ai_command_bar_actions();
        let aa = actions.iter().find(|a| a.id == "add_attachment").unwrap();
        assert_eq!(aa.icon, Some(IconName::Plus));
    }

    #[test]
    fn ai_bar_add_attachment_section_attachments() {
        let actions = get_ai_command_bar_actions();
        let aa = actions.iter().find(|a| a.id == "add_attachment").unwrap();
        assert_eq!(aa.section, Some("Attachments".to_string()));
    }

    #[test]
    fn ai_bar_add_attachment_desc_mentions_attach() {
        let actions = get_ai_command_bar_actions();
        let aa = actions.iter().find(|a| a.id == "add_attachment").unwrap();
        assert!(aa
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("attach"));
    }

    // =========================================================================
    // 25. AI bar: export section has exactly 1 action
    // =========================================================================

    #[test]
    fn ai_bar_export_section_count() {
        let actions = get_ai_command_bar_actions();
        let export_count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Export"))
            .count();
        assert_eq!(export_count, 1);
    }

    #[test]
    fn ai_bar_export_action_is_export_markdown() {
        let actions = get_ai_command_bar_actions();
        let export: Vec<_> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Export"))
            .collect();
        assert_eq!(export[0].id, "export_markdown");
    }

    #[test]
    fn ai_bar_help_section_count() {
        let actions = get_ai_command_bar_actions();
        let help_count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Help"))
            .count();
        assert_eq!(help_count, 1);
    }

    #[test]
    fn ai_bar_settings_section_count() {
        let actions = get_ai_command_bar_actions();
        let settings_count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Settings"))
            .count();
        assert_eq!(settings_count, 1);
    }

    // =========================================================================
    // 26. Notes: create_quicklink shortcut ⇧⌘L and icon Star
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
    fn notes_create_quicklink_icon_star() {
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
    fn notes_create_quicklink_section_copy() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ql = actions.iter().find(|a| a.id == "create_quicklink").unwrap();
        assert_eq!(ql.section, Some("Copy".to_string()));
    }

    #[test]
    fn notes_create_quicklink_absent_without_selection() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "create_quicklink"));
    }

    // =========================================================================
    // 27. Notes: copy_deeplink shortcut ⇧⌘D and icon ArrowRight
    // =========================================================================

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
    fn notes_copy_deeplink_icon_arrow_right() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert_eq!(dl.icon, Some(IconName::ArrowRight));
    }

    #[test]
    fn notes_copy_deeplink_section_copy() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert_eq!(dl.section, Some("Copy".to_string()));
    }

    #[test]
    fn notes_copy_deeplink_absent_in_trash() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "copy_deeplink"));
    }

    // =========================================================================
    // 28. Chat context: model ID format uses model.id not index
    // =========================================================================

    #[test]
    fn chat_model_id_uses_model_id() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "claude-3-opus".into(),
                display_name: "Claude 3 Opus".into(),
                provider: "Anthropic".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "select_model_claude-3-opus"));
    }

    #[test]
    fn chat_model_id_differs_from_new_chat_format() {
        // Chat uses select_model_{model.id}, new chat uses model_{index}
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "gpt-4".into(),
                display_name: "GPT-4".into(),
                provider: "OpenAI".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "select_model_gpt-4"));
        assert!(!actions.iter().any(|a| a.id == "model_0"));
    }

    #[test]
    fn chat_model_desc_uses_via_provider() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "x".into(),
                display_name: "X".into(),
                provider: "TestProvider".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = actions.iter().find(|a| a.id == "select_model_x").unwrap();
        assert_eq!(
            model_action.description,
            Some("via TestProvider".to_string())
        );
    }

    #[test]
    fn chat_multiple_models_unique_ids() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![
                ChatModelInfo {
                    id: "a".into(),
                    display_name: "A".into(),
                    provider: "P".into(),
                },
                ChatModelInfo {
                    id: "b".into(),
                    display_name: "B".into(),
                    provider: "P".into(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "select_model_a"));
        assert!(actions.iter().any(|a| a.id == "select_model_b"));
    }

    // =========================================================================
    // 29. coerce_action_selection: ix beyond rows.len() clamped
    // =========================================================================

    #[test]
    fn coerce_ix_beyond_len_clamped_to_last_item() {
        let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
        // ix=100 should be clamped to last valid index (1)
        let result = coerce_action_selection(&rows, 100);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn coerce_ix_beyond_with_trailing_header() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("S".into()),
        ];
        // ix=100 clamped to 1 (header), should search up to find Item(0)
        let result = coerce_action_selection(&rows, 100);
        assert_eq!(result, Some(0));
    }

    #[test]
    fn coerce_ix_exact_len_minus_one_on_item() {
        let rows = vec![
            GroupedActionItem::SectionHeader("H".into()),
            GroupedActionItem::Item(0),
        ];
        let result = coerce_action_selection(&rows, 1);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn coerce_single_item_ix_zero() {
        let rows = vec![GroupedActionItem::Item(0)];
        let result = coerce_action_selection(&rows, 0);
        assert_eq!(result, Some(0));
    }

    // =========================================================================
    // 30. build_grouped_items_static: Separators does NOT add section headers
    // =========================================================================

    #[test]
    fn separators_style_no_section_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        // Should have only Item entries, no SectionHeader
        for item in &grouped {
            assert!(
                matches!(item, GroupedActionItem::Item(_)),
                "Separators style should not add headers"
            );
        }
    }

    #[test]
    fn separators_style_item_count_matches_filtered() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
            Action::new("c", "C", None, ActionCategory::ScriptContext).with_section("S3"),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        assert_eq!(grouped.len(), 3);
    }

    #[test]
    fn headers_style_adds_section_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 2);
    }

    #[test]
    fn none_style_no_section_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        for item in &grouped {
            assert!(
                matches!(item, GroupedActionItem::Item(_)),
                "None style should not add headers"
            );
        }
    }
}
