    use crate::actions::builders::{
        get_ai_command_bar_actions, get_chat_context_actions,
        get_clipboard_history_context_actions, get_file_context_actions, get_new_chat_actions,
        get_note_switcher_actions, get_notes_command_bar_actions, get_path_context_actions,
        get_script_context_actions, get_scriptlet_context_actions_with_custom, to_deeplink_name,
        ChatModelInfo, ChatPromptInfo, ClipboardEntryInfo, NewChatModelInfo, NewChatPresetInfo,
        NoteSwitcherNoteInfo, NotesInfo,
    };
    use crate::actions::dialog::{
        build_grouped_items_static, coerce_action_selection, ActionsDialog, GroupedActionItem,
    };
    use crate::actions::types::{Action, ActionCategory, ScriptInfo, SectionStyle};
    use crate::actions::CommandBarConfig;
    use crate::clipboard_history::ContentType;
    use crate::designs::icon_variations::IconName;
    use crate::file_search::{FileInfo, FileType};
    use crate::prompts::PathInfo;
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
        app_name: Option<&str>,
    ) -> ClipboardEntryInfo {
        ClipboardEntryInfo {
            id: format!(
                "entry-{}-{}",
                if pinned { "pinned" } else { "unpinned" },
                match content_type {
                    ContentType::Text => "text",
                    ContentType::Image => "image",
                }
            ),
            content_type,
            pinned,
            preview: "Test content".to_string(),
            image_dimensions: if content_type == ContentType::Image {
                Some((640, 480))
            } else {
                None
            },
            frontmost_app_name: app_name.map(|s| s.to_string()),
        }
    }

    // ============================================================
    // 1. Action verb in primary title format
    // ============================================================

    #[test]
    fn verb_run_in_primary_title() {
        let script = ScriptInfo::new("My Script", "/path/to/script.ts");
        let actions = get_script_context_actions(&script);
        let primary = &actions[0];
        assert_eq!(primary.title, "Run \"My Script\"");
        assert_eq!(primary.id, "run_script");
    }

    #[test]
    fn verb_launch_in_primary_title() {
        let script =
            ScriptInfo::with_action_verb("Safari", "/Applications/Safari.app", false, "Launch");
        let actions = get_script_context_actions(&script);
        assert_eq!(actions[0].title, "Launch \"Safari\"");
    }

    #[test]
    fn verb_switch_to_in_primary_title() {
        let script = ScriptInfo::with_action_verb("My Document", "window:123", false, "Switch to");
        let actions = get_script_context_actions(&script);
        assert_eq!(actions[0].title, "Switch to \"My Document\"");
    }

    #[test]
    fn verb_open_in_primary_title() {
        let script =
            ScriptInfo::with_action_verb("Clipboard History", "builtin:clipboard", false, "Open");
        let actions = get_script_context_actions(&script);
        assert_eq!(actions[0].title, "Open \"Clipboard History\"");
    }

    #[test]
    fn verb_execute_in_primary_title() {
        let script =
            ScriptInfo::with_action_verb("Custom Task", "/path/to/task.ts", true, "Execute");
        let actions = get_script_context_actions(&script);
        assert_eq!(actions[0].title, "Execute \"Custom Task\"");
    }

    #[test]
    fn primary_description_includes_verb() {
        let verbs = ["Run", "Launch", "Switch to", "Open", "Execute"];
        for verb in &verbs {
            let script = ScriptInfo::with_action_verb("Test", "/path", false, *verb);
            let actions = get_script_context_actions(&script);
            let desc = actions[0].description.as_ref().unwrap();
            assert!(
                desc.contains(verb),
                "Description '{}' should contain verb '{}'",
                desc,
                verb
            );
        }
    }

    #[test]
    fn scriptlet_primary_verb_matches_script_info() {
        let script = ScriptInfo::scriptlet("Open GitHub", "/path/to/url.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        // Scriptlets default to "Run" verb
        assert_eq!(actions[0].title, "Run \"Open GitHub\"");
    }

    // ============================================================
    // 2. Clipboard pin × contentType × app combined matrix
    // ============================================================

    #[test]
    fn clipboard_text_unpinned_no_app() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_pin"));
        assert!(!actions.iter().any(|a| a.id == "clipboard_unpin"));
        assert!(!actions.iter().any(|a| a.id == "clipboard_ocr"));
        let paste = find_action(&actions, "clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to Active App");
    }

    #[test]
    fn clipboard_text_pinned_with_app() {
        let entry = make_clipboard_entry(ContentType::Text, true, Some("VSCode"));
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(!actions.iter().any(|a| a.id == "clipboard_pin"));
        assert!(actions.iter().any(|a| a.id == "clipboard_unpin"));
        let paste = find_action(&actions, "clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to VSCode");
    }

    #[test]
    fn clipboard_image_unpinned_with_app() {
        let entry = make_clipboard_entry(ContentType::Image, false, Some("Figma"));
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_pin"));
        assert!(actions.iter().any(|a| a.id == "clipboard_ocr"));
        let paste = find_action(&actions, "clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to Figma");
    }

    #[test]
    fn clipboard_image_pinned_no_app() {
        let entry = make_clipboard_entry(ContentType::Image, true, None);
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_unpin"));
        assert!(actions.iter().any(|a| a.id == "clipboard_ocr"));
        let paste = find_action(&actions, "clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to Active App");
    }

    #[test]
    fn clipboard_image_has_more_actions_than_text() {
        let text_entry = make_clipboard_entry(ContentType::Text, false, None);
        let image_entry = make_clipboard_entry(ContentType::Image, false, None);
        let text_actions = get_clipboard_history_context_actions(&text_entry);
        let image_actions = get_clipboard_history_context_actions(&image_entry);
        assert!(
            image_actions.len() > text_actions.len(),
            "Image ({}) should have more actions than text ({})",
            image_actions.len(),
            text_actions.len()
        );
    }

    // ============================================================
    // 3. Scriptlet context with custom actions + frecency
    // ============================================================

    #[test]
    fn scriptlet_with_custom_actions_and_frecency() {
        let script = ScriptInfo::scriptlet("Test", "/path/to/test.md", None, None)
            .with_frecency(true, Some("scriptlet:Test".to_string()));

        let mut scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo main".to_string(),
        );
        scriptlet.actions = vec![ScriptletAction {
            name: "Copy Output".to_string(),
            command: "copy-output".to_string(),
            tool: "bash".to_string(),
            code: "echo output | pbcopy".to_string(),
            inputs: vec![],
            shortcut: Some("cmd+c".to_string()),
            description: Some("Copy the output".to_string()),
        }];

        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));

        // Should have custom action
        assert!(actions
            .iter()
            .any(|a| a.id == "scriptlet_action:copy-output"));
        // Should have reset_ranking due to frecency
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));
        // Custom action should be after run but before edit
        let run_idx = actions.iter().position(|a| a.id == "run_script").unwrap();
        let custom_idx = actions
            .iter()
            .position(|a| a.id == "scriptlet_action:copy-output")
            .unwrap();
        let edit_idx = actions
            .iter()
            .position(|a| a.id == "edit_scriptlet")
            .unwrap();
        assert!(run_idx < custom_idx);
        assert!(custom_idx < edit_idx);
    }

    #[test]
    fn scriptlet_custom_action_has_action_true() {
        let script = ScriptInfo::scriptlet("Test", "/path/to/test.md", None, None);
        let mut scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo main".to_string(),
        );
        scriptlet.actions = vec![ScriptletAction {
            name: "Do Thing".to_string(),
            command: "do-thing".to_string(),
            tool: "bash".to_string(),
            code: "echo thing".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];

        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let custom = find_action(&actions, "scriptlet_action:do-thing").unwrap();
        assert!(
            custom.has_action,
            "Scriptlet custom action must have has_action=true"
        );
        assert_eq!(custom.value, Some("do-thing".to_string()));
    }

    #[test]
    fn scriptlet_builtin_actions_have_has_action_false() {
        let script = ScriptInfo::scriptlet("Test", "/path/to/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        for action in &actions {
            if !action.id.starts_with("scriptlet_action:") {
                assert!(
                    !action.has_action,
                    "Built-in action '{}' should have has_action=false",
                    action.id
                );
            }
        }
    }

    // ============================================================
    // 4. Path context special characters in names
    // ============================================================

    #[test]
    fn path_dir_with_spaces_in_name() {
        let info = PathInfo::new("My Documents", "/Users/test/My Documents", true);
        let actions = get_path_context_actions(&info);
        let primary = &actions[0];
        assert_eq!(primary.title, "Open \"My Documents\"");
        assert_eq!(primary.id, "open_directory");
    }

    #[test]
    fn path_file_with_dots_in_name() {
        let info = PathInfo::new("archive.tar.gz", "/tmp/archive.tar.gz", false);
        let actions = get_path_context_actions(&info);
        let primary = &actions[0];
        assert_eq!(primary.title, "Select \"archive.tar.gz\"");
        assert_eq!(primary.id, "select_file");
    }

    #[test]
    fn path_trash_description_dir_vs_file() {
        let dir_info = PathInfo::new("folder", "/tmp/folder", true);
        let file_info = PathInfo::new("file.txt", "/tmp/file.txt", false);

        let dir_actions = get_path_context_actions(&dir_info);
        let file_actions = get_path_context_actions(&file_info);

        let dir_trash = find_action(&dir_actions, "move_to_trash").unwrap();
        let file_trash = find_action(&file_actions, "move_to_trash").unwrap();

        assert!(
            dir_trash.description.as_ref().unwrap().contains("folder"),
            "Dir trash should say 'folder'"
        );
        assert!(
            file_trash.description.as_ref().unwrap().contains("file"),
            "File trash should say 'file'"
        );
    }

    #[test]
    fn path_dir_and_file_have_same_action_count() {
        let dir_info = PathInfo::new("dir", "/tmp/dir", true);
        let file_info = PathInfo::new("file.txt", "/tmp/file.txt", false);
        let dir_actions = get_path_context_actions(&dir_info);
        let file_actions = get_path_context_actions(&file_info);
        assert_eq!(
            dir_actions.len(),
            file_actions.len(),
            "Dir ({}) and file ({}) should have same action count",
            dir_actions.len(),
            file_actions.len()
        );
    }

    // ============================================================
    // 5. Note switcher Unicode/emoji titles
    // ============================================================

    #[test]
    fn note_switcher_unicode_title() {
        let notes = vec![make_note(
            "id1",
            "Café Notes",
            42,
            false,
            false,
            "Some preview text",
            "5m ago",
        )];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].title, "Café Notes");
    }

    #[test]
    fn note_switcher_current_with_bullet() {
        let notes = vec![make_note(
            "id1",
            "Current Note",
            100,
            true,
            false,
            "Content here",
            "1m ago",
        )];
        let actions = get_note_switcher_actions(&notes);
        assert!(
            actions[0].title.starts_with("• "),
            "Current note should have bullet prefix, got: '{}'",
            actions[0].title
        );
        assert_eq!(actions[0].title, "• Current Note");
    }

    #[test]
    fn note_switcher_pinned_has_star_icon() {
        let notes = vec![make_note(
            "id1",
            "Pinned",
            50,
            false,
            true,
            "pinned content",
            "2h ago",
        )];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
        assert_eq!(actions[0].section, Some("Pinned".to_string()));
    }

    #[test]
    fn note_switcher_current_has_check_icon() {
        let notes = vec![make_note(
            "id1",
            "Current",
            50,
            true,
            false,
            "current content",
            "1m ago",
        )];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::Check));
        assert_eq!(actions[0].section, Some("Recent".to_string()));
    }

