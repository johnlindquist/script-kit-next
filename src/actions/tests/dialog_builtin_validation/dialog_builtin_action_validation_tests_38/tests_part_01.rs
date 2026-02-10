    use crate::actions::builders::*;
    use crate::actions::command_bar::CommandBarConfig;
    use crate::actions::dialog::ActionsDialog;
    use crate::actions::types::{Action, ActionCategory, AnchorPosition, ScriptInfo, SectionStyle};
    use crate::actions::window::{count_section_headers, WindowPosition};
    use crate::clipboard_history::ContentType;
    use crate::designs::icon_variations::IconName;
    use crate::file_search::{FileInfo, FileType};
    use crate::prompts::PathInfo;
    use crate::protocol::ProtocolAction;

    // =========================================================================
    // 1. ScriptInfo::with_action_verb: custom verb in primary action title
    // =========================================================================

    #[test]
    fn script_info_with_action_verb_sets_verb() {
        let info = ScriptInfo::with_action_verb("App Launcher", "/bin/launch", false, "Launch");
        assert_eq!(info.action_verb, "Launch");
    }

    #[test]
    fn script_info_with_action_verb_sets_is_script() {
        let info = ScriptInfo::with_action_verb("App Launcher", "/bin/launch", false, "Launch");
        assert!(!info.is_script);
    }

    #[test]
    fn script_info_with_action_verb_run_script_title_uses_verb() {
        let info = ScriptInfo::with_action_verb("Spotify", "/bin/spotify", false, "Switch to");
        let actions = get_script_context_actions(&info);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert!(run.title.starts_with("Switch to"));
    }

    #[test]
    fn script_info_with_action_verb_desc_uses_verb() {
        let info = ScriptInfo::with_action_verb("Calc", "/bin/calc", true, "Open");
        let actions = get_script_context_actions(&info);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert!(run.description.as_ref().unwrap().contains("Open"));
    }

    // =========================================================================
    // 2. ScriptInfo::with_all: preserves all fields
    // =========================================================================

    #[test]
    fn script_info_with_all_preserves_name_and_path() {
        let info = ScriptInfo::with_all(
            "Test",
            "/test.ts",
            true,
            "Run",
            Some("cmd+t".into()),
            Some("tt".into()),
        );
        assert_eq!(info.name, "Test");
        assert_eq!(info.path, "/test.ts");
    }

    #[test]
    fn script_info_with_all_preserves_shortcut_and_alias() {
        let info = ScriptInfo::with_all(
            "Test",
            "/test.ts",
            true,
            "Run",
            Some("cmd+t".into()),
            Some("tt".into()),
        );
        assert_eq!(info.shortcut, Some("cmd+t".to_string()));
        assert_eq!(info.alias, Some("tt".to_string()));
    }

    #[test]
    fn script_info_with_all_defaults_agent_false() {
        let info = ScriptInfo::with_all("Test", "/test.ts", true, "Run", None, None);
        assert!(!info.is_agent);
    }

    #[test]
    fn script_info_with_all_defaults_suggested_false() {
        let info = ScriptInfo::with_all("Test", "/test.ts", true, "Run", None, None);
        assert!(!info.is_suggested);
        assert!(info.frecency_path.is_none());
    }

    // =========================================================================
    // 3. ScriptInfo::with_is_script: explicit is_script flag
    // =========================================================================

    #[test]
    fn script_info_with_is_script_true_has_edit_actions() {
        let info = ScriptInfo::with_is_script("my-script", "/path.ts", true);
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "edit_script"));
        assert!(actions.iter().any(|a| a.id == "view_logs"));
    }

    #[test]
    fn script_info_with_is_script_false_no_edit_actions() {
        let info = ScriptInfo::with_is_script("built-in", "", false);
        let actions = get_script_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "edit_script"));
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    #[test]
    fn script_info_with_is_script_defaults_verb_to_run() {
        let info = ScriptInfo::with_is_script("test", "/p", true);
        assert_eq!(info.action_verb, "Run");
    }

    #[test]
    fn script_info_with_is_script_defaults_scriptlet_false() {
        let info = ScriptInfo::with_is_script("test", "/p", true);
        assert!(!info.is_scriptlet);
    }

    // =========================================================================
    // 4. Clipboard: save_snippet and save_file details
    // =========================================================================

    #[test]
    fn clipboard_save_snippet_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let save = actions
            .iter()
            .find(|a| a.id == "clipboard_save_snippet")
            .unwrap();
        assert_eq!(save.shortcut.as_deref(), Some("⇧⌘S"));
    }

    #[test]
    fn clipboard_save_snippet_title() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let save = actions
            .iter()
            .find(|a| a.id == "clipboard_save_snippet")
            .unwrap();
        assert_eq!(save.title, "Save Text as Snippet");
    }

    #[test]
    fn clipboard_save_file_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let save = actions
            .iter()
            .find(|a| a.id == "clipboard_save_file")
            .unwrap();
        assert_eq!(save.shortcut.as_deref(), Some("⌥⇧⌘S"));
    }

    #[test]
    fn clipboard_save_file_desc_mentions_file() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let save = actions
            .iter()
            .find(|a| a.id == "clipboard_save_file")
            .unwrap();
        assert!(save.description.as_ref().unwrap().contains("file"));
    }

    // =========================================================================
    // 5. Clipboard: frontmost_app_name dynamic paste title
    // =========================================================================

    #[test]
    fn clipboard_paste_title_with_app_name() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: Some("Safari".into()),
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to Safari");
    }

    #[test]
    fn clipboard_paste_title_without_app_name() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to Active App");
    }

    #[test]
    fn clipboard_paste_title_with_special_chars_in_app_name() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: Some("Visual Studio Code".into()),
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to Visual Studio Code");
    }

    #[test]
    fn clipboard_paste_shortcut_always_return() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: Some("Terminal".into()),
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
        assert_eq!(paste.shortcut.as_deref(), Some("↵"));
    }

    // =========================================================================
    // 6. Notes: duplicate_note and find_in_note details
    // =========================================================================

    #[test]
    fn notes_duplicate_note_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let dup = actions.iter().find(|a| a.id == "duplicate_note").unwrap();
        assert_eq!(dup.shortcut.as_deref(), Some("⌘D"));
    }

    #[test]
    fn notes_duplicate_note_icon_is_copy() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let dup = actions.iter().find(|a| a.id == "duplicate_note").unwrap();
        assert_eq!(dup.icon, Some(IconName::Copy));
    }

    #[test]
    fn notes_duplicate_note_absent_without_selection() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
    }

    #[test]
    fn notes_duplicate_note_absent_in_trash() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
    }

    // =========================================================================
    // 7. Notes: find_in_note details
    // =========================================================================

    #[test]
    fn notes_find_in_note_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let find = actions.iter().find(|a| a.id == "find_in_note").unwrap();
        assert_eq!(find.shortcut.as_deref(), Some("⌘F"));
    }

    #[test]
    fn notes_find_in_note_icon() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let find = actions.iter().find(|a| a.id == "find_in_note").unwrap();
        assert_eq!(find.icon, Some(IconName::MagnifyingGlass));
    }

    #[test]
    fn notes_find_in_note_section_is_edit() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let find = actions.iter().find(|a| a.id == "find_in_note").unwrap();
        assert_eq!(find.section.as_deref(), Some("Edit"));
    }

    #[test]
    fn notes_find_in_note_absent_without_selection() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "find_in_note"));
    }

    // =========================================================================
    // 8. Notes: copy_note_as details
    // =========================================================================

    #[test]
    fn notes_copy_note_as_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let copy = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
        assert_eq!(copy.shortcut.as_deref(), Some("⇧⌘C"));
    }

    #[test]
    fn notes_copy_note_as_icon_is_copy() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let copy = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
        assert_eq!(copy.icon, Some(IconName::Copy));
    }

    #[test]
    fn notes_copy_note_as_section_is_copy() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let copy = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
        assert_eq!(copy.section.as_deref(), Some("Copy"));
    }

    #[test]
    fn notes_copy_note_as_absent_in_trash() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "copy_note_as"));
    }

    // =========================================================================
    // 9. AI command bar: export_markdown details
    // =========================================================================

    #[test]
    fn ai_bar_export_markdown_shortcut() {
        let actions = get_ai_command_bar_actions();
        let exp = actions.iter().find(|a| a.id == "export_markdown").unwrap();
        assert_eq!(exp.shortcut.as_deref(), Some("⇧⌘E"));
    }

    #[test]
    fn ai_bar_export_markdown_icon() {
        let actions = get_ai_command_bar_actions();
        let exp = actions.iter().find(|a| a.id == "export_markdown").unwrap();
        assert_eq!(exp.icon, Some(IconName::FileCode));
    }

    #[test]
    fn ai_bar_export_markdown_section() {
        let actions = get_ai_command_bar_actions();
        let exp = actions.iter().find(|a| a.id == "export_markdown").unwrap();
        assert_eq!(exp.section.as_deref(), Some("Export"));
    }

    #[test]
    fn ai_bar_export_markdown_desc_mentions_markdown() {
        let actions = get_ai_command_bar_actions();
        let exp = actions.iter().find(|a| a.id == "export_markdown").unwrap();
        assert!(exp.description.as_ref().unwrap().contains("Markdown"));
    }

    // =========================================================================
    // 10. AI command bar: submit action details
    // =========================================================================

