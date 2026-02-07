
#[test]
fn notes_command_bar_section_order_no_selection() {
    // Without selection, only Notes and Settings sections should appear
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let sections = sections_in_order(&actions);
    assert_eq!(
        sections,
        vec!["Notes", "Settings"],
        "Notes without selection should only have Notes and Settings"
    );
}

#[test]
fn notes_command_bar_section_order_trash_view() {
    // In trash view, even with selection, only Notes appears (plus Settings if not auto-sizing)
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let sections = sections_in_order(&actions);
    assert_eq!(
        sections,
        vec!["Notes", "Settings"],
        "Notes in trash view should only have Notes and Settings"
    );
}

#[test]
fn notes_command_bar_auto_sizing_enabled_hides_settings() {
    // With auto-sizing already enabled, Settings section should be absent
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let sections = sections_in_order(&actions);
    assert_eq!(
        sections,
        vec!["Notes"],
        "With auto-sizing on and no selection, only Notes section"
    );
}

// =========================================================================
// 7. New chat section ordering: Last Used Settings > Presets > Models
// =========================================================================

#[test]
fn new_chat_section_order_all_populated() {
    let last_used = vec![NewChatModelInfo {
        model_id: "claude-3".into(),
        display_name: "Claude 3".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Settings,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "gpt-4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &presets, &models);
    let sections = sections_in_order(&actions);
    assert_eq!(sections, vec!["Last Used Settings", "Presets", "Models"],);
}

#[test]
fn new_chat_section_order_no_last_used() {
    let presets = vec![NewChatPresetInfo {
        id: "code".into(),
        name: "Code".into(),
        icon: IconName::Code,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "gpt-4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&[], &presets, &models);
    let sections = sections_in_order(&actions);
    assert_eq!(sections, vec!["Presets", "Models"]);
}

#[test]
fn new_chat_all_empty_returns_no_actions() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

// =========================================================================
// 8. Scriptlet with multiple custom H3 actions
// =========================================================================

#[test]
fn scriptlet_custom_actions_maintain_order() {
    let script = ScriptInfo::scriptlet("Multi Action", "/path/multi.md", None, None);
    let mut scriptlet = Scriptlet::new(
        "Multi Action".to_string(),
        "bash".to_string(),
        "echo main".to_string(),
    );
    scriptlet.actions = vec![
        ScriptletAction {
            name: "Alpha".to_string(),
            command: "alpha-cmd".to_string(),
            tool: "bash".to_string(),
            code: "echo alpha".to_string(),
            inputs: vec![],
            shortcut: Some("cmd+1".to_string()),
            description: Some("First action".to_string()),
        },
        ScriptletAction {
            name: "Beta".to_string(),
            command: "beta-cmd".to_string(),
            tool: "bash".to_string(),
            code: "echo beta".to_string(),
            inputs: vec![],
            shortcut: Some("cmd+2".to_string()),
            description: Some("Second action".to_string()),
        },
        ScriptletAction {
            name: "Gamma".to_string(),
            command: "gamma-cmd".to_string(),
            tool: "bash".to_string(),
            code: "echo gamma".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
    ];

    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let ids = action_ids(&actions);

    // run_script must be first
    assert_eq!(ids[0], "run_script");

    // Custom actions follow run in declaration order
    let alpha_idx = ids
        .iter()
        .position(|id| *id == "scriptlet_action:alpha-cmd")
        .unwrap();
    let beta_idx = ids
        .iter()
        .position(|id| *id == "scriptlet_action:beta-cmd")
        .unwrap();
    let gamma_idx = ids
        .iter()
        .position(|id| *id == "scriptlet_action:gamma-cmd")
        .unwrap();

    assert_eq!(alpha_idx, 1);
    assert_eq!(beta_idx, 2);
    assert_eq!(gamma_idx, 3);

    // Custom actions all have has_action=true
    for id in &[
        "scriptlet_action:alpha-cmd",
        "scriptlet_action:beta-cmd",
        "scriptlet_action:gamma-cmd",
    ] {
        let a = find_action(&actions, id).unwrap();
        assert!(
            a.has_action,
            "Custom action '{}' should have has_action=true",
            id
        );
        assert!(
            a.value.is_some(),
            "Custom action '{}' should have a value",
            id
        );
    }
}

#[test]
fn scriptlet_custom_action_id_format() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut scriptlet = Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Do Something".to_string(),
        command: "do-something".to_string(),
        tool: "bash".to_string(),
        code: "echo do".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];

    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let custom = find_action(&actions, "scriptlet_action:do-something").unwrap();
    assert!(custom.id.starts_with("scriptlet_action:"));
    assert_eq!(custom.title, "Do Something");
}

// =========================================================================
// 9. Action title formatting with varied action_verbs
// =========================================================================

#[test]
fn action_verb_appears_in_primary_title() {
    let verbs = ["Run", "Launch", "Switch to", "Open", "Execute"];
    for verb in &verbs {
        let script = ScriptInfo::with_action_verb("MyItem", "/path/item", false, *verb);
        let actions = get_script_context_actions(&script);
        let primary = &actions[0];
        assert!(
            primary.title.starts_with(verb),
            "Primary action title '{}' should start with verb '{}'",
            primary.title,
            verb
        );
        assert!(
            primary.title.contains("MyItem"),
            "Primary action title '{}' should contain the item name",
            primary.title
        );
    }
}

#[test]
fn scriptlet_primary_uses_action_verb() {
    let script = ScriptInfo::scriptlet("Open URL", "/path/url.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let primary = &actions[0];
    assert!(
        primary.title.starts_with("Run"),
        "Scriptlet primary should use 'Run' verb"
    );
    assert!(primary.title.contains("Open URL"));
}

// =========================================================================
// 10. Path context shortcut assignments
// =========================================================================

#[test]
fn path_file_has_enter_on_primary() {
    let path = PathInfo {
        path: "/usr/file.txt".into(),
        name: "file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].id, "select_file");
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn path_dir_has_enter_on_primary() {
    let path = PathInfo {
        path: "/usr/local".into(),
        name: "local".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].id, "open_directory");
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn path_context_has_trash_shortcut() {
    let path = PathInfo {
        path: "/tmp/file.txt".into(),
        name: "file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    let trash = find_action(&actions, "move_to_trash").unwrap();
    assert_eq!(trash.shortcut.as_deref(), Some("⌘⌫"));
}

#[test]
fn path_context_has_all_expected_actions() {
    let path = PathInfo {
        path: "/tmp/file.txt".into(),
        name: "file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();

    let expected = [
        "select_file",
        "copy_path",
        "open_in_finder",
        "open_in_editor",
        "open_in_terminal",
        "copy_filename",
        "move_to_trash",
    ];
    for id in &expected {
        assert!(
            ids.contains(id),
            "Path file context should have action '{}'",
            id
        );
    }
}

#[test]
fn path_dir_context_has_open_directory_not_select_file() {
    let path = PathInfo {
        path: "/usr/local".into(),
        name: "local".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path);
    let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();
    assert!(ids.contains("open_directory"));
    assert!(!ids.contains("select_file"));
}

// =========================================================================
// 11. Clipboard ordering invariant: paste first, deletes last
// =========================================================================

#[test]
fn clipboard_paste_always_first_text() {
    let entry = ClipboardEntryInfo {
        id: "t1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[0].id, "clipboard_paste");
}

#[test]
fn clipboard_paste_always_first_image() {
    let entry = ClipboardEntryInfo {
        id: "i1".into(),
        content_type: ContentType::Image,
        pinned: true,
        preview: "img".into(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: Some("Figma".into()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[0].id, "clipboard_paste");
}

#[test]
fn clipboard_delete_actions_always_last_three() {
    let entry = ClipboardEntryInfo {
        id: "d1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let len = actions.len();
    assert!(len >= 3);

    let last_three_ids: Vec<&str> = actions[len - 3..].iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        last_three_ids,
        vec![
            "clipboard_delete",
            "clipboard_delete_multiple",
            "clipboard_delete_all"
        ],
        "Last 3 clipboard actions should be the destructive ones in order"
    );
}

#[test]
fn clipboard_delete_actions_always_last_three_image() {
    let entry = ClipboardEntryInfo {
        id: "di".into(),
        content_type: ContentType::Image,
        pinned: true,
        preview: "img".into(),
        image_dimensions: Some((1920, 1080)),
        frontmost_app_name: Some("Preview".into()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let len = actions.len();

    let last_three_ids: Vec<&str> = actions[len - 3..].iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        last_three_ids,
        vec![
            "clipboard_delete",
            "clipboard_delete_multiple",
            "clipboard_delete_all"
        ],
    );
}

// =========================================================================
// 12. Mixed flag combinations on ScriptInfo
// =========================================================================

#[test]
fn script_with_both_shortcut_and_alias_has_update_remove_for_both() {
    let script = ScriptInfo::with_shortcut_and_alias(
        "full",
        "/path/full.ts",
        Some("cmd+f".into()),
        Some("fl".into()),
    );
    let actions = get_script_context_actions(&script);
    let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();

    assert!(ids.contains("update_shortcut"));
    assert!(ids.contains("remove_shortcut"));
    assert!(!ids.contains("add_shortcut"));
    assert!(ids.contains("update_alias"));
    assert!(ids.contains("remove_alias"));
    assert!(!ids.contains("add_alias"));
}

#[test]
fn builtin_with_frecency_has_reset_ranking_and_no_edit() {
    let builtin = ScriptInfo::builtin("Clipboard History")
        .with_frecency(true, Some("builtin:clipboard".into()));
    let actions = get_script_context_actions(&builtin);
    let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();

    assert!(ids.contains("reset_ranking"));
    assert!(ids.contains("run_script"));
    assert!(ids.contains("copy_deeplink"));
    assert!(!ids.contains("edit_script"));
    assert!(!ids.contains("view_logs"));
}
