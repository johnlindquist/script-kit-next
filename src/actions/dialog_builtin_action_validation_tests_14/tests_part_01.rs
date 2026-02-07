    use super::super::builders::*;
    use super::super::command_bar::CommandBarConfig;
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
    // 1. Path context action ordering: primary first, trash last
    // =========================================================================

    #[test]
    fn cat01_path_dir_primary_is_first_action() {
        let info = PathInfo {
            path: "/Users/test/Documents".to_string(),
            name: "Documents".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].id, "open_directory");
    }

    #[test]
    fn cat01_path_file_primary_is_first_action() {
        let info = PathInfo {
            path: "/Users/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].id, "select_file");
    }

    #[test]
    fn cat01_path_trash_is_always_last() {
        for is_dir in [true, false] {
            let info = PathInfo {
                path: "/test/item".to_string(),
                name: "item".to_string(),
                is_dir,
            };
            let actions = get_path_context_actions(&info);
            assert_eq!(
                actions.last().unwrap().id,
                "move_to_trash",
                "move_to_trash should be last for is_dir={}",
                is_dir
            );
        }
    }

    #[test]
    fn cat01_path_dir_and_file_same_action_count() {
        let dir_info = PathInfo {
            path: "/test/dir".to_string(),
            name: "dir".to_string(),
            is_dir: true,
        };
        let file_info = PathInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
        };
        assert_eq!(
            get_path_context_actions(&dir_info).len(),
            get_path_context_actions(&file_info).len()
        );
    }

    #[test]
    fn cat01_path_common_actions_always_present() {
        let info = PathInfo {
            path: "/test/thing".to_string(),
            name: "thing".to_string(),
            is_dir: false,
        };
        let ids = action_ids(&get_path_context_actions(&info));
        for expected in &[
            "copy_path",
            "open_in_finder",
            "open_in_editor",
            "open_in_terminal",
            "copy_filename",
            "move_to_trash",
        ] {
            assert!(
                ids.contains(&expected.to_string()),
                "Missing expected action: {}",
                expected
            );
        }
    }

    #[test]
    fn cat01_path_dir_title_contains_name() {
        let info = PathInfo {
            path: "/test/MyFolder".to_string(),
            name: "MyFolder".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        assert!(actions[0].title.contains("MyFolder"));
    }

    // =========================================================================
    // 2. Clipboard OCR only on image, not text
    // =========================================================================

    #[test]
    fn cat02_clipboard_text_has_no_ocr() {
        let entry = ClipboardEntryInfo {
            id: "t1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let ids = action_ids(&get_clipboard_history_context_actions(&entry));
        assert!(!ids.contains(&"clipboard_ocr".to_string()));
    }

    #[test]
    fn cat02_clipboard_image_has_ocr() {
        let entry = ClipboardEntryInfo {
            id: "i1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "img".to_string(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let ids = action_ids(&get_clipboard_history_context_actions(&entry));
        assert!(ids.contains(&"clipboard_ocr".to_string()));
    }

    #[test]
    fn cat02_clipboard_image_has_more_actions_than_text() {
        let text_entry = ClipboardEntryInfo {
            id: "t".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let img_entry = ClipboardEntryInfo {
            id: "i".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "y".to_string(),
            image_dimensions: Some((10, 10)),
            frontmost_app_name: None,
        };
        let text_count = get_clipboard_history_context_actions(&text_entry).len();
        let img_count = get_clipboard_history_context_actions(&img_entry).len();
        assert!(
            img_count > text_count,
            "Image ({}) should have more actions than text ({})",
            img_count,
            text_count
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat02_clipboard_text_no_open_with_macos() {
        let entry = ClipboardEntryInfo {
            id: "t2".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let ids = action_ids(&get_clipboard_history_context_actions(&entry));
        assert!(!ids.contains(&"clipboard_open_with".to_string()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat02_clipboard_image_has_open_with_macos() {
        let entry = ClipboardEntryInfo {
            id: "i2".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "img".to_string(),
            image_dimensions: Some((50, 50)),
            frontmost_app_name: None,
        };
        let ids = action_ids(&get_clipboard_history_context_actions(&entry));
        assert!(ids.contains(&"clipboard_open_with".to_string()));
    }

    // =========================================================================
    // 3. ScriptInfo::with_all sets exactly the right fields
    // =========================================================================

    #[test]
    fn cat03_with_all_sets_name_and_path() {
        let s = ScriptInfo::with_all("Foo", "/bar", true, "Run", None, None);
        assert_eq!(s.name, "Foo");
        assert_eq!(s.path, "/bar");
    }

    #[test]
    fn cat03_with_all_sets_is_script() {
        let s_true = ScriptInfo::with_all("A", "/a", true, "Run", None, None);
        assert!(s_true.is_script);
        let s_false = ScriptInfo::with_all("B", "/b", false, "Run", None, None);
        assert!(!s_false.is_script);
    }

    #[test]
    fn cat03_with_all_sets_verb() {
        let s = ScriptInfo::with_all("X", "/x", false, "Execute", None, None);
        assert_eq!(s.action_verb, "Execute");
    }

    #[test]
    fn cat03_with_all_sets_shortcut_and_alias() {
        let s = ScriptInfo::with_all(
            "Y",
            "/y",
            false,
            "Open",
            Some("cmd+y".into()),
            Some("yy".into()),
        );
        assert_eq!(s.shortcut, Some("cmd+y".to_string()));
        assert_eq!(s.alias, Some("yy".to_string()));
    }

    #[test]
    fn cat03_with_all_defaults_no_agent_no_scriptlet() {
        let s = ScriptInfo::with_all("Z", "/z", true, "Run", None, None);
        assert!(!s.is_agent);
        assert!(!s.is_scriptlet);
        assert!(!s.is_suggested);
        assert!(s.frecency_path.is_none());
    }

    // =========================================================================
    // 4. AI command bar exact icon-to-section mapping
    // =========================================================================

    #[test]
    fn cat04_ai_response_section_has_3_actions() {
        let actions = get_ai_command_bar_actions();
        let response_actions: Vec<_> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Response"))
            .collect();
        assert_eq!(response_actions.len(), 3);
    }

    #[test]
    fn cat04_ai_actions_section_has_4_actions() {
        let actions = get_ai_command_bar_actions();
        let section_actions: Vec<_> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Actions"))
            .collect();
        assert_eq!(section_actions.len(), 4);
    }

    #[test]
    fn cat04_ai_attachments_section_has_2_actions() {
        let actions = get_ai_command_bar_actions();
        let section_actions: Vec<_> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Attachments"))
            .collect();
        assert_eq!(section_actions.len(), 2);
    }

    #[test]
    fn cat04_ai_export_section_has_1_action() {
        let actions = get_ai_command_bar_actions();
        let section_actions: Vec<_> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Export"))
            .collect();
        assert_eq!(section_actions.len(), 1);
    }

    #[test]
    fn cat04_ai_help_section_has_1_action() {
        let actions = get_ai_command_bar_actions();
        let section_actions: Vec<_> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Help"))
            .collect();
        assert_eq!(section_actions.len(), 1);
    }

    #[test]
    fn cat04_ai_settings_section_has_1_action() {
        let actions = get_ai_command_bar_actions();
        let section_actions: Vec<_> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Settings"))
            .collect();
        assert_eq!(section_actions.len(), 1);
    }

    #[test]
    fn cat04_ai_copy_response_icon_is_copy() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "copy_response").unwrap();
        assert_eq!(a.icon, Some(IconName::Copy));
    }

    #[test]
    fn cat04_ai_submit_icon_is_arrowup() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "submit").unwrap();
        assert_eq!(a.icon, Some(IconName::ArrowUp));
    }

    #[test]
    fn cat04_ai_new_chat_icon_is_plus() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "new_chat").unwrap();
        assert_eq!(a.icon, Some(IconName::Plus));
    }

    #[test]
    fn cat04_ai_delete_chat_icon_is_trash() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "delete_chat").unwrap();
        assert_eq!(a.icon, Some(IconName::Trash));
    }

    #[test]
    fn cat04_ai_change_model_icon_is_settings() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "change_model").unwrap();
        assert_eq!(a.icon, Some(IconName::Settings));
    }

    // =========================================================================
    // 5. Notes command bar section-action membership
    // =========================================================================

    #[test]
    fn cat05_notes_full_feature_notes_section_has_3() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Notes"))
            .count();
        assert_eq!(count, 3);
    }

    #[test]
    fn cat05_notes_full_feature_edit_section_has_2() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Edit"))
            .count();
        assert_eq!(count, 2);
    }

    #[test]
    fn cat05_notes_full_feature_copy_section_has_3() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Copy"))
            .count();
        assert_eq!(count, 3);
    }

    #[test]
    fn cat05_notes_full_feature_export_section_has_1() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Export"))
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn cat05_notes_trash_view_only_notes_section() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Only new_note and browse_notes in Notes section
        let sections: HashSet<_> = actions.iter().filter_map(|a| a.section.clone()).collect();
        assert_eq!(sections.len(), 1);
        assert!(sections.contains("Notes"));
    }

    #[test]
    fn cat05_notes_no_selection_minimal() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        // new_note, browse_notes, enable_auto_sizing
        assert_eq!(actions.len(), 3);
    }

    // =========================================================================
    // 6. Note switcher title rendering
    // =========================================================================

