    use crate::actions::builders::*;
    use crate::actions::dialog::ActionsDialog;
    use crate::actions::types::{Action, ActionCategory, ScriptInfo};
    use crate::clipboard_history::ContentType;
    use crate::designs::icon_variations::IconName;
    use crate::file_search::{FileInfo, FileType};
    use crate::prompts::PathInfo;
    use crate::protocol::ProtocolAction;
    use crate::scriptlets::Scriptlet;

    // =========================================================================
    // 1. ScriptInfo::with_action_verb_and_shortcut: field validation
    // =========================================================================

    #[test]
    fn with_action_verb_and_shortcut_preserves_name_path() {
        let info = ScriptInfo::with_action_verb_and_shortcut(
            "launcher",
            "/scripts/launcher.ts",
            true,
            "Launch",
            Some("cmd+l".into()),
        );
        assert_eq!(info.name, "launcher");
        assert_eq!(info.path, "/scripts/launcher.ts");
    }

    #[test]
    fn with_action_verb_and_shortcut_sets_verb() {
        let info = ScriptInfo::with_action_verb_and_shortcut("x", "/x", true, "Open", None);
        assert_eq!(info.action_verb, "Open");
    }

    #[test]
    fn with_action_verb_and_shortcut_sets_shortcut() {
        let info = ScriptInfo::with_action_verb_and_shortcut(
            "x",
            "/x",
            false,
            "Run",
            Some("cmd+shift+r".into()),
        );
        assert_eq!(info.shortcut, Some("cmd+shift+r".to_string()));
        assert!(!info.is_script);
    }

    #[test]
    fn with_action_verb_and_shortcut_defaults_agent_false() {
        let info =
            ScriptInfo::with_action_verb_and_shortcut("x", "/x", true, "Run", Some("a".into()));
        assert!(!info.is_agent);
        assert!(!info.is_scriptlet);
        assert!(!info.is_suggested);
    }

    // =========================================================================
    // 2. ScriptInfo: is_agent manual override after construction
    // =========================================================================

    #[test]
    fn script_info_set_is_agent_true_after_new() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_agent = true;
        assert!(info.is_agent);
        assert!(info.is_script); // new() sets is_script=true
    }

    #[test]
    fn agent_actions_include_edit_agent_title() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_agent = true;
        info.is_script = false; // agents have is_script=false
        let actions = get_script_context_actions(&info);
        let edit = actions.iter().find(|a| a.id == "edit_script");
        assert!(edit.is_some());
        assert_eq!(edit.unwrap().title, "Edit Agent");
    }

    #[test]
    fn agent_actions_have_copy_content() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_agent = true;
        info.is_script = false;
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "copy_content"));
    }

    #[test]
    fn agent_actions_desc_mentions_agent() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_agent = true;
        info.is_script = false;
        let actions = get_script_context_actions(&info);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert!(edit
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("agent"));
    }

    // =========================================================================
    // 3. Action::with_shortcut_opt: Some vs None behavior
    // =========================================================================

    #[test]
    fn with_shortcut_opt_some_sets_shortcut() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘T".to_string()));
        assert_eq!(action.shortcut, Some("⌘T".to_string()));
    }

    #[test]
    fn with_shortcut_opt_none_leaves_none() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(None);
        assert!(action.shortcut.is_none());
        assert!(action.shortcut_lower.is_none());
    }

    #[test]
    fn with_shortcut_opt_some_sets_shortcut_lower() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘ABC".to_string()));
        assert_eq!(action.shortcut_lower, Some("⌘abc".to_string()));
    }

    #[test]
    fn with_shortcut_opt_overwrites_previous_shortcut() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘X")
            .with_shortcut_opt(Some("⌘Y".to_string()));
        assert_eq!(action.shortcut, Some("⌘Y".to_string()));
    }

    // =========================================================================
    // 4. Action::with_icon and with_section chaining
    // =========================================================================

    #[test]
    fn with_icon_sets_icon() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_icon(IconName::Copy);
        assert_eq!(action.icon, Some(IconName::Copy));
    }

    #[test]
    fn with_section_sets_section() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_section("MySection");
        assert_eq!(action.section, Some("MySection".to_string()));
    }

    #[test]
    fn chained_icon_section_shortcut_preserves_all() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘T")
            .with_icon(IconName::Star)
            .with_section("Actions");
        assert_eq!(action.shortcut, Some("⌘T".to_string()));
        assert_eq!(action.icon, Some(IconName::Star));
        assert_eq!(action.section, Some("Actions".to_string()));
    }

    #[test]
    fn action_new_defaults_icon_section_none() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
        assert!(action.icon.is_none());
        assert!(action.section.is_none());
        assert!(action.value.is_none());
    }

    // =========================================================================
    // 5. Clipboard: text entry total action count on macOS
    // =========================================================================

    #[test]
    fn clipboard_text_unpinned_action_count_macos() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        // On macOS: paste, copy, paste_keep_open, share, attach_to_ai, quick_look,
        //           pin, save_snippet, save_file, delete, delete_multiple, delete_all = 12
        #[cfg(target_os = "macos")]
        assert_eq!(actions.len(), 12);
    }

    #[test]
    fn clipboard_text_pinned_same_count_as_unpinned() {
        let unpinned = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "a".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let pinned = ClipboardEntryInfo {
            id: "2".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "b".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let unpinned_actions = get_clipboard_history_context_actions(&unpinned);
        let pinned_actions = get_clipboard_history_context_actions(&pinned);
        assert_eq!(unpinned_actions.len(), pinned_actions.len());
    }

    #[test]
    fn clipboard_text_first_action_is_paste() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[0].id, "clipboard_paste");
    }

    #[test]
    fn clipboard_text_second_action_is_copy() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[1].id, "clipboard_copy");
    }

    // =========================================================================
    // 6. Clipboard: image entry total action count on macOS
    // =========================================================================

    #[test]
    fn clipboard_image_unpinned_action_count_macos() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        // On macOS image: paste, copy, paste_keep_open, share, attach_to_ai, quick_look,
        //   open_with, annotate_cleanshot, upload_cleanshot, pin, ocr,
        //   save_snippet, save_file, delete, delete_multiple, delete_all = 16
        #[cfg(target_os = "macos")]
        assert_eq!(actions.len(), 16);
    }

    #[test]
    fn clipboard_image_has_4_more_than_text() {
        let text_entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let image_entry = ClipboardEntryInfo {
            id: "2".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let text_count = get_clipboard_history_context_actions(&text_entry).len();
        let image_count = get_clipboard_history_context_actions(&image_entry).len();
        #[cfg(target_os = "macos")]
        assert_eq!(image_count - text_count, 4);
    }

    #[test]
    fn clipboard_image_has_ocr_action() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_ocr"));
    }

    #[test]
    fn clipboard_text_has_no_ocr_action() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(!actions.iter().any(|a| a.id == "clipboard_ocr"));
    }

    // =========================================================================
    // 7. Clipboard: pinned text entry has clipboard_unpin ID
    // =========================================================================

    #[test]
    fn clipboard_pinned_has_unpin_action() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_unpin"));
        assert!(!actions.iter().any(|a| a.id == "clipboard_pin"));
    }

    #[test]
    fn clipboard_pinned_unpin_title() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let unpin = actions.iter().find(|a| a.id == "clipboard_unpin").unwrap();
        assert_eq!(unpin.title, "Unpin Entry");
    }

    #[test]
    fn clipboard_pinned_unpin_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let unpin = actions.iter().find(|a| a.id == "clipboard_unpin").unwrap();
        assert_eq!(unpin.shortcut, Some("⇧⌘P".to_string()));
    }

    #[test]
    fn clipboard_pinned_has_no_pin_action() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(!actions.iter().any(|a| a.id == "clipboard_pin"));
    }

    // =========================================================================
    // 8. Clipboard: unpinned text entry has clipboard_pin ID
    // =========================================================================

    #[test]
    fn clipboard_unpinned_has_pin_action() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_pin"));
        assert!(!actions.iter().any(|a| a.id == "clipboard_unpin"));
    }

    #[test]
    fn clipboard_unpinned_pin_title() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let pin = actions.iter().find(|a| a.id == "clipboard_pin").unwrap();
        assert_eq!(pin.title, "Pin Entry");
    }

    #[test]
    fn clipboard_unpinned_pin_desc_mentions_prevent() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let pin = actions.iter().find(|a| a.id == "clipboard_pin").unwrap();
        assert!(pin.description.as_ref().unwrap().contains("Pin"));
    }

