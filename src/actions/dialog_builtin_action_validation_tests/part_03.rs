
#[test]
fn note_switcher_pinned_current_same_note_gets_star_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "both".into(),
        title: "Both Pinned & Current".into(),
        char_count: 42,
        is_current: true,
        is_pinned: true,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    assert!(actions[0].title.starts_with("• "));
}

// =========================================================================
// 18. New chat actions — icons are correct per section
// =========================================================================

#[test]
fn new_chat_last_used_all_get_bolt_icon() {
    let last_used: Vec<NewChatModelInfo> = (0..3)
        .map(|i| NewChatModelInfo {
            model_id: format!("m{}", i),
            display_name: format!("M{}", i),
            provider: "p".into(),
            provider_display_name: "P".into(),
        })
        .collect();
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    for action in &actions {
        assert_eq!(
            action.icon,
            Some(IconName::BoltFilled),
            "Last used '{}' should have BoltFilled icon",
            action.id
        );
    }
}

#[test]
fn new_chat_models_all_get_settings_icon() {
    let models: Vec<NewChatModelInfo> = (0..3)
        .map(|i| NewChatModelInfo {
            model_id: format!("m{}", i),
            display_name: format!("M{}", i),
            provider: "p".into(),
            provider_display_name: "P".into(),
        })
        .collect();
    let actions = get_new_chat_actions(&[], &[], &models);
    for action in &actions {
        assert_eq!(
            action.icon,
            Some(IconName::Settings),
            "Model '{}' should have Settings icon",
            action.id
        );
    }
}

#[test]
fn new_chat_presets_preserve_custom_icons() {
    let presets = vec![
        NewChatPresetInfo {
            id: "code".into(),
            name: "Code".into(),
            icon: IconName::Code,
        },
        NewChatPresetInfo {
            id: "star".into(),
            name: "Star".into(),
            icon: IconName::Star,
        },
    ];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].icon, Some(IconName::Code));
    assert_eq!(actions[1].icon, Some(IconName::Star));
}

// =========================================================================
// 19. CommandBarConfig — field preservation across presets
// =========================================================================

#[test]
fn command_bar_all_presets_close_on_escape() {
    assert!(CommandBarConfig::default().close_on_escape);
    assert!(CommandBarConfig::ai_style().close_on_escape);
    assert!(CommandBarConfig::notes_style().close_on_escape);
    assert!(CommandBarConfig::main_menu_style().close_on_escape);
    assert!(CommandBarConfig::no_search().close_on_escape);
}

#[test]
fn command_bar_all_presets_close_on_click_outside() {
    assert!(CommandBarConfig::default().close_on_click_outside);
    assert!(CommandBarConfig::ai_style().close_on_click_outside);
    assert!(CommandBarConfig::notes_style().close_on_click_outside);
    assert!(CommandBarConfig::main_menu_style().close_on_click_outside);
    assert!(CommandBarConfig::no_search().close_on_click_outside);
}

#[test]
fn command_bar_all_presets_close_on_select() {
    assert!(CommandBarConfig::default().close_on_select);
    assert!(CommandBarConfig::ai_style().close_on_select);
    assert!(CommandBarConfig::notes_style().close_on_select);
    assert!(CommandBarConfig::main_menu_style().close_on_select);
    assert!(CommandBarConfig::no_search().close_on_select);
}

// =========================================================================
// 20. Fuzzy match with real action titles
// =========================================================================

#[test]
fn fuzzy_match_works_on_real_action_titles() {
    // Common user search patterns against actual action titles
    assert!(ActionsDialog::fuzzy_match("edit script", "es"));
    assert!(ActionsDialog::fuzzy_match("reveal in finder", "rif"));
    assert!(ActionsDialog::fuzzy_match("copy path", "cp"));
    assert!(ActionsDialog::fuzzy_match("copy deeplink", "cdl"));
    assert!(ActionsDialog::fuzzy_match("add keyboard shortcut", "aks"));
    assert!(ActionsDialog::fuzzy_match("reset ranking", "rr"));
    assert!(ActionsDialog::fuzzy_match("view logs", "vl"));
}

#[test]
fn fuzzy_match_fails_for_reversed_chars() {
    // "se" should not fuzzy match "edit script" (s comes after e)
    // Actually: e-d-i-t- -s-c-r-i-p-t → 's' at index 5, 'e' not found after 's'... wait
    // "se": s at 5, then e at... no e after index 5. So it fails.
    assert!(!ActionsDialog::fuzzy_match("edit script", "se"));
}

// =========================================================================
// 21. Action verb propagation in primary action
// =========================================================================

#[test]
fn action_verb_launch_propagates_to_run_action() {
    let script = ScriptInfo::with_action_verb("App Launcher", "builtin:launcher", false, "Launch");
    let actions = get_script_context_actions(&script);
    let run = find_action(&actions, "run_script").unwrap();
    assert!(
        run.title.starts_with("Launch"),
        "Primary action should use 'Launch' verb, got '{}'",
        run.title
    );
}

#[test]
fn action_verb_switch_to_propagates_to_run_action() {
    let script =
        ScriptInfo::with_action_verb("Window Switcher", "builtin:windows", false, "Switch to");
    let actions = get_script_context_actions(&script);
    let run = find_action(&actions, "run_script").unwrap();
    assert!(
        run.title.starts_with("Switch to"),
        "Primary action should use 'Switch to' verb, got '{}'",
        run.title
    );
}

#[test]
fn action_verb_open_propagates_to_run_action() {
    let script = ScriptInfo::with_action_verb("Notes", "builtin:notes", false, "Open");
    let actions = get_script_context_actions(&script);
    let run = find_action(&actions, "run_script").unwrap();
    assert!(
        run.title.starts_with("Open"),
        "Primary action should use 'Open' verb, got '{}'",
        run.title
    );
}

// =========================================================================
// 22. title_lower correctness across all contexts
// =========================================================================

#[test]
fn title_lower_matches_title_for_all_clipboard_actions() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: Some("Safari".into()),
    };
    for action in &get_clipboard_history_context_actions(&entry) {
        assert_eq!(
            action.title_lower,
            action.title.to_lowercase(),
            "title_lower mismatch for clipboard action '{}'",
            action.id
        );
    }
}

#[test]
fn title_lower_matches_title_for_all_file_actions() {
    let file = FileInfo {
        path: "/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    for action in &get_file_context_actions(&file) {
        assert_eq!(
            action.title_lower,
            action.title.to_lowercase(),
            "title_lower mismatch for file action '{}'",
            action.id
        );
    }
}

#[test]
fn title_lower_matches_title_for_all_path_actions() {
    let path = PathInfo::new("test", "/test", false);
    for action in &get_path_context_actions(&path) {
        assert_eq!(
            action.title_lower,
            action.title.to_lowercase(),
            "title_lower mismatch for path action '{}'",
            action.id
        );
    }
}

#[test]
fn title_lower_matches_title_for_all_chat_actions() {
    let info = ChatPromptInfo {
        current_model: Some("Model A".into()),
        available_models: vec![ChatModelInfo {
            id: "a".into(),
            display_name: "Model A".into(),
            provider: "PA".into(),
        }],
        has_messages: true,
        has_response: true,
    };
    for action in &get_chat_context_actions(&info) {
        assert_eq!(
            action.title_lower,
            action.title.to_lowercase(),
            "title_lower mismatch for chat action '{}'",
            action.id
        );
    }
}

// =========================================================================
// 23. Scriptlet with zero custom actions still has built-in actions
// =========================================================================

#[test]
fn scriptlet_zero_custom_actions_has_built_in_set() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo hi".into());
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    assert!(
        actions.len() >= 3,
        "Scriptlet with no custom actions should still have built-in actions"
    );
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn scriptlet_none_scriptlet_has_built_in_set() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(!actions.is_empty());
    assert_eq!(actions[0].id, "run_script");
}

// =========================================================================
// 24. ProtocolAction default behavior
// =========================================================================

#[test]
fn protocol_action_new_defaults_to_visible_and_closable() {
    let pa = ProtocolAction::new("Test".into());
    assert!(pa.is_visible());
    assert!(pa.should_close());
}

#[test]
fn protocol_action_explicit_false_overrides_defaults() {
    let pa = ProtocolAction {
        visible: Some(false),
        close: Some(false),
        ..ProtocolAction::new("Test".into())
    };
    assert!(!pa.is_visible());
    assert!(!pa.should_close());
}

#[test]
fn protocol_action_with_value_sets_value_and_keeps_has_action_false() {
    let pa = ProtocolAction::with_value("Submit".into(), "val".into());
    assert_eq!(pa.value, Some("val".into()));
    assert!(!pa.has_action);
}

// =========================================================================
// 25. Cross-context: minimum action counts
// =========================================================================

#[test]
fn script_context_has_at_least_seven_actions() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let count = get_script_context_actions(&script).len();
    assert!(
        count >= 7,
        "Script context should have at least 7 actions, got {}",
        count
    );
}

#[test]
fn file_context_has_at_least_four_actions() {
    let file = FileInfo {
        path: "/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let count = get_file_context_actions(&file).len();
    assert!(
        count >= 4,
        "File context should have at least 4 actions, got {}",
        count
    );
}

#[test]
fn clipboard_context_has_at_least_eight_actions() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let count = get_clipboard_history_context_actions(&entry).len();
    assert!(
        count >= 8,
        "Clipboard context should have at least 8 actions, got {}",
        count
    );
}

#[test]
fn path_context_has_at_least_six_actions() {
    let path = PathInfo::new("test", "/test", false);
    let count = get_path_context_actions(&path).len();
    assert!(
        count >= 6,
        "Path context should have at least 6 actions, got {}",
        count
    );
}

#[test]
fn chat_context_always_has_continue_in_chat() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"continue_in_chat"));
}

// =========================================================================
// 26. Scriptlet with many custom actions — all ordered after run
// =========================================================================

#[test]
fn scriptlet_ten_custom_actions_all_after_run() {
    let script = ScriptInfo::scriptlet("Big", "/path/big.md", None, None);
    let mut scriptlet = Scriptlet::new("Big".into(), "bash".into(), "echo main".into());
    for i in 0..10 {
        scriptlet.actions.push(ScriptletAction {
            name: format!("Action {}", i),
            command: format!("act-{}", i),
            tool: "bash".into(),
            code: format!("echo {}", i),
            inputs: vec![],
            shortcut: None,
            description: None,
        });
    }
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    assert_eq!(actions[0].id, "run_script");
    let custom_ids: Vec<&str> = actions
        .iter()
        .filter(|a| a.id.starts_with("scriptlet_action:"))
        .map(|a| a.id.as_str())
        .collect();
    assert_eq!(custom_ids.len(), 10);
    // All custom actions should come after run_script
    let run_pos = actions.iter().position(|a| a.id == "run_script").unwrap();
    for custom in &actions {
        if custom.id.starts_with("scriptlet_action:") {
            let pos = actions.iter().position(|a| a.id == custom.id).unwrap();
            assert!(
                pos > run_pos,
                "Custom action '{}' at {} should be after run_script at {}",
                custom.id,
                pos,
                run_pos
            );
        }
    }
}

// =========================================================================
// 27. Format shortcut hint — roundtrip patterns
// =========================================================================

#[test]
fn format_shortcut_hint_cmd_enter() {
    assert_eq!(ActionsDialog::format_shortcut_hint("cmd+enter"), "⌘↵");
}

#[test]
fn format_shortcut_hint_cmd_shift_delete() {
    let result = ActionsDialog::format_shortcut_hint("cmd+shift+delete");
    assert!(result.contains('⌘'));
    assert!(result.contains('⇧'));
    assert!(result.contains('⌫'));
}

#[test]
fn format_shortcut_hint_ctrl_alt_letter() {
    let result = ActionsDialog::format_shortcut_hint("ctrl+alt+z");
    assert!(result.contains('⌃'));
    assert!(result.contains('⌥'));
    assert!(result.contains('Z'));
}
