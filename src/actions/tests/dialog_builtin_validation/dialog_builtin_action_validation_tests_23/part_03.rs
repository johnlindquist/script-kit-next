
#[test]
fn batch23_new_chat_model_icon() {
    let models = vec![NewChatModelInfo {
        model_id: "m".to_string(),
        display_name: "M".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].icon, Some(IconName::Settings));
}

#[test]
fn batch23_new_chat_preset_icon_preserved() {
    let presets = vec![NewChatPresetInfo {
        id: "code".to_string(),
        name: "Code".to_string(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].icon, Some(IconName::Code));
}

#[test]
fn batch23_new_chat_preset_no_description() {
    let presets = vec![NewChatPresetInfo {
        id: "g".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert!(actions[0].description.is_none());
}

// ============================================================
// 17. Note switcher: empty list produces placeholder
// ============================================================

#[test]
fn batch23_note_switcher_empty_placeholder_id() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "no_notes");
}

#[test]
fn batch23_note_switcher_empty_placeholder_title() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions[0].title, "No notes yet");
}

#[test]
fn batch23_note_switcher_empty_placeholder_icon() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions[0].icon, Some(IconName::Plus));
}

#[test]
fn batch23_note_switcher_empty_placeholder_section() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions[0].section.as_ref().unwrap(), "Notes");
}

#[test]
fn batch23_note_switcher_empty_placeholder_description() {
    let actions = get_note_switcher_actions(&[]);
    assert!(actions[0].description.as_ref().unwrap().contains("⌘N"));
}

// ============================================================
// 18. Note switcher: multi-note section assignment
// ============================================================

#[test]
fn batch23_note_switcher_pinned_and_recent_sections() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "1".to_string(),
            title: "Pinned Note".to_string(),
            char_count: 50,
            is_current: false,
            is_pinned: true,
            preview: "pinned content".to_string(),
            relative_time: "1h ago".to_string(),
        },
        NoteSwitcherNoteInfo {
            id: "2".to_string(),
            title: "Recent Note".to_string(),
            char_count: 30,
            is_current: false,
            is_pinned: false,
            preview: "recent content".to_string(),
            relative_time: "5m ago".to_string(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_ref().unwrap(), "Pinned");
    assert_eq!(actions[1].section.as_ref().unwrap(), "Recent");
}

#[test]
fn batch23_note_switcher_all_pinned() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "1".to_string(),
            title: "A".to_string(),
            char_count: 10,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "2".to_string(),
            title: "B".to_string(),
            char_count: 20,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    assert!(actions
        .iter()
        .all(|a| a.section.as_ref().unwrap() == "Pinned"));
}

#[test]
fn batch23_note_switcher_all_recent() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".to_string(),
        title: "A".to_string(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_ref().unwrap(), "Recent");
}

// ============================================================
// 19. Note switcher: id format uses note_{uuid}
// ============================================================

#[test]
fn batch23_note_switcher_id_format() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc-123".to_string(),
        title: "Test".to_string(),
        char_count: 5,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].id, "note_abc-123");
}

#[test]
fn batch23_note_switcher_multiple_ids_unique() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "A".to_string(),
            char_count: 1,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "b".to_string(),
            title: "B".to_string(),
            char_count: 2,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    assert_ne!(actions[0].id, actions[1].id);
}

// ============================================================
// 20. Scriptlet defined actions: has_action and value
// ============================================================

#[test]
fn batch23_scriptlet_defined_has_action_true() {
    let mut scriptlet = Scriptlet::new("T".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Act".to_string(),
        command: "act-cmd".to_string(),
        tool: "bash".to_string(),
        code: "echo act".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert!(actions[0].has_action);
}

#[test]
fn batch23_scriptlet_defined_value_is_command() {
    let mut scriptlet = Scriptlet::new("T".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".to_string(),
        command: "copy-cmd".to_string(),
        tool: "bash".to_string(),
        code: "pbcopy".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert_eq!(actions[0].value.as_ref().unwrap(), "copy-cmd");
}

#[test]
fn batch23_scriptlet_defined_id_prefix() {
    let mut scriptlet = Scriptlet::new("T".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "My Action".to_string(),
        command: "my-action".to_string(),
        tool: "bash".to_string(),
        code: "echo".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert!(actions[0].id.starts_with("scriptlet_action:"));
    assert_eq!(actions[0].id, "scriptlet_action:my-action");
}

#[test]
fn batch23_scriptlet_defined_shortcut_formatted() {
    let mut scriptlet = Scriptlet::new("T".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Test".to_string(),
        command: "test".to_string(),
        tool: "bash".to_string(),
        code: "echo".to_string(),
        inputs: vec![],
        shortcut: Some("cmd+shift+x".to_string()),
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert_eq!(actions[0].shortcut.as_ref().unwrap(), "⌘⇧X");
}

// ============================================================
// 21. Scriptlet context with custom: ordering of custom vs built-in
// ============================================================

#[test]
fn batch23_scriptlet_custom_between_run_and_shortcut() {
    let script = ScriptInfo::scriptlet("Test", "/test.md", None, None);
    let mut scriptlet = Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Custom".to_string(),
        command: "custom".to_string(),
        tool: "bash".to_string(),
        code: "echo".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let run_idx = actions.iter().position(|a| a.id == "run_script").unwrap();
    let custom_idx = actions
        .iter()
        .position(|a| a.id == "scriptlet_action:custom")
        .unwrap();
    let shortcut_idx = actions.iter().position(|a| a.id == "add_shortcut").unwrap();
    assert_eq!(run_idx, 0);
    assert_eq!(custom_idx, 1);
    assert!(shortcut_idx > custom_idx);
}

#[test]
fn batch23_scriptlet_custom_multiple_preserve_order() {
    let script = ScriptInfo::scriptlet("Test", "/test.md", None, None);
    let mut scriptlet = Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![
        ScriptletAction {
            name: "First".to_string(),
            command: "first".to_string(),
            tool: "bash".to_string(),
            code: "echo 1".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
        ScriptletAction {
            name: "Second".to_string(),
            command: "second".to_string(),
            tool: "bash".to_string(),
            code: "echo 2".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
    ];
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let first_idx = actions
        .iter()
        .position(|a| a.id == "scriptlet_action:first")
        .unwrap();
    let second_idx = actions
        .iter()
        .position(|a| a.id == "scriptlet_action:second")
        .unwrap();
    assert!(first_idx < second_idx);
    assert_eq!(first_idx, 1); // right after run_script
    assert_eq!(second_idx, 2);
}

// ============================================================
// 22. to_deeplink_name: whitespace and mixed input
// ============================================================

#[test]
fn batch23_deeplink_tabs_and_newlines() {
    assert_eq!(to_deeplink_name("hello\tworld\ntest"), "hello-world-test");
}

#[test]
fn batch23_deeplink_multiple_spaces() {
    assert_eq!(to_deeplink_name("a   b"), "a-b");
}

#[test]
fn batch23_deeplink_leading_trailing_specials() {
    assert_eq!(to_deeplink_name("--hello--"), "hello");
}

#[test]
fn batch23_deeplink_mixed_alpha_numeric_special() {
    assert_eq!(to_deeplink_name("Script #1 (beta)"), "script-1-beta");
}

#[test]
fn batch23_deeplink_unicode_preserved() {
    let result = to_deeplink_name("日本語スクリプト");
    assert!(result.contains("日本語スクリプト"));
}

// ============================================================
// 23. format_shortcut_hint (ActionsDialog): modifier ordering
// ============================================================

#[test]
fn batch23_format_cmd_c() {
    assert_eq!(ActionsDialog::format_shortcut_hint("cmd+c"), "⌘C");
}

#[test]
fn batch23_format_ctrl_shift_delete() {
    assert_eq!(
        ActionsDialog::format_shortcut_hint("ctrl+shift+delete"),
        "⌃⇧⌫"
    );
}

#[test]
fn batch23_format_alt_enter() {
    assert_eq!(ActionsDialog::format_shortcut_hint("alt+enter"), "⌥↵");
}

#[test]
fn batch23_format_meta_is_cmd() {
    assert_eq!(ActionsDialog::format_shortcut_hint("meta+a"), "⌘A");
}

#[test]
fn batch23_format_super_is_cmd() {
    assert_eq!(ActionsDialog::format_shortcut_hint("super+k"), "⌘K");
}

// ============================================================
// 24. parse_shortcut_keycaps: multi-char and edge cases
// ============================================================

#[test]
fn batch23_parse_keycaps_cmd_enter() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘↵");
    assert_eq!(caps, vec!["⌘", "↵"]);
}

#[test]
fn batch23_parse_keycaps_single_letter() {
    let caps = ActionsDialog::parse_shortcut_keycaps("a");
    assert_eq!(caps, vec!["A"]);
}

#[test]
fn batch23_parse_keycaps_arrows() {
    let caps = ActionsDialog::parse_shortcut_keycaps("↑↓←→");
    assert_eq!(caps, vec!["↑", "↓", "←", "→"]);
}

#[test]
fn batch23_parse_keycaps_all_modifiers() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘⌃⌥⇧");
    assert_eq!(caps, vec!["⌘", "⌃", "⌥", "⇧"]);
}

#[test]
fn batch23_parse_keycaps_lowercase_uppercased() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘c");
    assert_eq!(caps, vec!["⌘", "C"]);
}

// ============================================================
// 25. score_action: fuzzy vs prefix vs contains
// ============================================================

#[test]
fn batch23_score_prefix_highest() {
    let action = Action::new("a", "Copy Path", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "copy");
    assert!(score >= 100);
}

#[test]
fn batch23_score_contains_medium() {
    let action = Action::new("a", "Full Copy Path", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "copy");
    assert!((50..100).contains(&score));
}
