
#[test]
fn new_chat_model_ids_are_indexed() {
    let models = vec![
        NewChatModelInfo {
            model_id: "claude".into(),
            display_name: "Claude".into(),
            provider: "a".into(),
            provider_display_name: "Anthropic".into(),
        },
        NewChatModelInfo {
            model_id: "gpt".into(),
            display_name: "GPT".into(),
            provider: "o".into(),
            provider_display_name: "OpenAI".into(),
        },
    ];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].id, "model_0");
    assert_eq!(actions[1].id, "model_1");
}

// =========================================================================
// 25. All AI command bar actions have icon and section
// =========================================================================

#[test]
fn ai_command_bar_all_have_icon() {
    for a in &get_ai_command_bar_actions() {
        assert!(a.icon.is_some(), "AI action '{}' should have an icon", a.id);
    }
}

#[test]
fn ai_command_bar_all_have_section() {
    for a in &get_ai_command_bar_actions() {
        assert!(
            a.section.is_some(),
            "AI action '{}' should have a section",
            a.id
        );
    }
}

// =========================================================================
// 26. Notes command bar conditional icons
// =========================================================================

#[test]
fn notes_command_bar_all_have_icons_when_full() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for a in &get_notes_command_bar_actions(&info) {
        assert!(
            a.icon.is_some(),
            "Notes action '{}' should have an icon",
            a.id
        );
    }
}

#[test]
fn notes_command_bar_all_have_sections_when_full() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for a in &get_notes_command_bar_actions(&info) {
        assert!(
            a.section.is_some(),
            "Notes action '{}' should have a section",
            a.id
        );
    }
}

// =========================================================================
// 27. Clipboard attach_to_ai action present
// =========================================================================

#[test]
fn clipboard_text_has_attach_to_ai() {
    let entry = ClipboardEntryInfo {
        id: "ai".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_attach_to_ai"));
}

#[test]
fn clipboard_image_has_attach_to_ai() {
    let entry = ClipboardEntryInfo {
        id: "ai2".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_attach_to_ai"));
}

// =========================================================================
// 28. Scriptlet context built-in action set
// =========================================================================

#[test]
fn scriptlet_context_has_expected_builtin_ids() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();

    let expected = [
        "run_script",
        "add_shortcut",
        "add_alias",
        "edit_scriptlet",
        "reveal_scriptlet_in_finder",
        "copy_scriptlet_path",
        "copy_content",
        "copy_deeplink",
    ];
    for id in &expected {
        assert!(ids.contains(id), "Scriptlet context should have '{}'", id);
    }
}

#[test]
fn scriptlet_context_action_order_run_before_builtin() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let ids = action_ids(&actions);

    let run_idx = ids.iter().position(|id| *id == "run_script").unwrap();
    let edit_idx = ids.iter().position(|id| *id == "edit_scriptlet").unwrap();
    let deeplink_idx = ids.iter().position(|id| *id == "copy_deeplink").unwrap();

    assert!(run_idx < edit_idx, "run should come before edit_scriptlet");
    assert!(
        edit_idx < deeplink_idx,
        "edit_scriptlet should come before copy_deeplink"
    );
}

// =========================================================================
// 29. Path context trash description varies by is_dir
// =========================================================================

#[test]
fn path_trash_description_says_file_for_file() {
    let path = PathInfo {
        path: "/tmp/file.txt".into(),
        name: "file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    let trash = find_action(&actions, "move_to_trash").unwrap();
    assert_eq!(trash.description.as_deref(), Some("Delete file"),);
}

#[test]
fn path_trash_description_says_folder_for_dir() {
    let path = PathInfo {
        path: "/tmp/mydir".into(),
        name: "mydir".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path);
    let trash = find_action(&actions, "move_to_trash").unwrap();
    assert_eq!(trash.description.as_deref(), Some("Delete folder"),);
}

// =========================================================================
// 30. Note switcher all notes have "Notes" section
// =========================================================================

#[test]
fn note_switcher_all_actions_have_notes_section() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "a".into(),
            title: "Note A".into(),
            char_count: 10,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "b".into(),
            title: "Note B".into(),
            char_count: 20,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    for a in &actions {
        let section = a.section.as_deref();
        assert!(
            section == Some("Pinned") || section == Some("Recent"),
            "Note switcher action '{}' should have 'Pinned' or 'Recent' section, got {:?}",
            a.id,
            section
        );
    }
}

#[test]
fn note_switcher_empty_state_has_notes_section() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions[0].section.as_deref(), Some("Notes"));
}

// =========================================================================
// 31. New chat action icons
// =========================================================================

#[test]
fn new_chat_last_used_has_bolt_icon() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "M1".into(),
        provider: "p".into(),
        provider_display_name: "P".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
}

#[test]
fn new_chat_models_have_settings_icon() {
    let models = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "M1".into(),
        provider: "p".into(),
        provider_display_name: "P".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].icon, Some(IconName::Settings));
}

#[test]
fn new_chat_preset_uses_custom_icon() {
    let presets = vec![NewChatPresetInfo {
        id: "code".into(),
        name: "Code".into(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].icon, Some(IconName::Code));
}

// =========================================================================
// 32. Clipboard save actions have correct shortcuts
// =========================================================================

#[test]
fn clipboard_save_snippet_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "ss".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let save = find_action(&actions, "clipboard_save_snippet").unwrap();
    assert_eq!(save.shortcut.as_deref(), Some("⇧⌘S"));
}

#[test]
fn clipboard_save_file_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "sf".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let save = find_action(&actions, "clipboard_save_file").unwrap();
    assert_eq!(save.shortcut.as_deref(), Some("⌥⇧⌘S"));
}

// =========================================================================
// 33. Script context deeplink description format
// =========================================================================

#[test]
fn script_deeplink_description_contains_url() {
    let script = ScriptInfo::new("My Cool Script", "/path/script.ts");
    let actions = get_script_context_actions(&script);
    let deeplink = find_action(&actions, "copy_deeplink").unwrap();
    assert!(
        deeplink
            .description
            .as_ref()
            .unwrap()
            .contains("scriptkit://run/my-cool-script"),
        "Deeplink description should contain the URL"
    );
}

#[test]
fn scriptlet_deeplink_description_contains_url() {
    let script = ScriptInfo::scriptlet("Open GitHub", "/path/url.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let deeplink = find_action(&actions, "copy_deeplink").unwrap();
    assert!(deeplink
        .description
        .as_ref()
        .unwrap()
        .contains("scriptkit://run/open-github"),);
}

// =========================================================================
// 34. All built-in actions have ActionCategory::ScriptContext
// =========================================================================

#[test]
fn script_context_all_actions_are_script_context_category() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    for a in &get_script_context_actions(&script) {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "Action '{}' should be ScriptContext",
            a.id
        );
    }
}

#[test]
fn clipboard_all_actions_are_script_context_category() {
    let entry = ClipboardEntryInfo {
        id: "c".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    for a in &get_clipboard_history_context_actions(&entry) {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "Action '{}' should be ScriptContext",
            a.id
        );
    }
}

#[test]
fn file_all_actions_are_script_context_category() {
    let file = FileInfo {
        path: "/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    for a in &get_file_context_actions(&file) {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "Action '{}' should be ScriptContext",
            a.id
        );
    }
}

#[test]
fn path_all_actions_are_script_context_category() {
    let path = PathInfo {
        path: "/tmp".into(),
        name: "tmp".into(),
        is_dir: true,
    };
    for a in &get_path_context_actions(&path) {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "Action '{}' should be ScriptContext",
            a.id
        );
    }
}

#[test]
fn ai_command_bar_all_actions_are_script_context_category() {
    for a in &get_ai_command_bar_actions() {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "Action '{}' should be ScriptContext",
            a.id
        );
    }
}

#[test]
fn notes_command_bar_all_actions_are_script_context_category() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for a in &get_notes_command_bar_actions(&info) {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "Action '{}' should be ScriptContext",
            a.id
        );
    }
}

// =========================================================================
// 35. Action count bounds
// =========================================================================

#[test]
fn script_context_has_at_least_5_actions() {
    // Any script should have at minimum: run, shortcut, alias, deeplink, + edit/view/reveal/copy
    let script = ScriptInfo::new("test", "/path/test.ts");
    let count = get_script_context_actions(&script).len();
    assert!(
        count >= 5,
        "Script context should have at least 5 actions, got {}",
        count
    );
}
