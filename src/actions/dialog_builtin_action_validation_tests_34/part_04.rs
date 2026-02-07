
// =====================================================================
// 28. Script context: copy_content shortcut ⌘⌥C for all types
// =====================================================================

#[test]
fn script_copy_content_shortcut_opt_cmd_c() {
    let script = ScriptInfo::new("Test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(cc.shortcut.as_deref(), Some("⌘⌥C"));
}

#[test]
fn agent_copy_content_shortcut_opt_cmd_c() {
    let mut script = ScriptInfo::new("Agent", "/path/agent.md");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(cc.shortcut.as_deref(), Some("⌘⌥C"));
}

#[test]
fn script_copy_content_desc_mentions_entire_file() {
    let script = ScriptInfo::new("Test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert!(cc.description.as_ref().unwrap().contains("entire file"));
}

#[test]
fn agent_copy_content_desc_mentions_entire_file() {
    let mut script = ScriptInfo::new("Agent", "/path/agent.md");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert!(cc.description.as_ref().unwrap().contains("entire file"));
}

// =====================================================================
// 29. score_action: title_lower and description_lower used for matching
// =====================================================================

#[test]
fn score_action_matches_case_insensitive() {
    let action = Action::new(
        "test",
        "Edit Script",
        Some("Open in editor".into()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "edit");
    assert!(score >= 100, "Prefix match should score >=100, got {score}");
}

#[test]
fn score_action_description_bonus_adds_points() {
    let action = Action::new(
        "test",
        "Open File",
        Some("Open in editor for editing".into()),
        ActionCategory::ScriptContext,
    );
    // "editor" is not in title but is in description
    let score = ActionsDialog::score_action(&action, "editor");
    assert!(
        score >= 15,
        "Description match should score >=15, got {score}"
    );
}

#[test]
fn score_action_no_match_returns_zero() {
    let action = Action::new(
        "test",
        "Edit Script",
        Some("Open in editor".into()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "zzzzz");
    assert_eq!(score, 0);
}

#[test]
fn score_action_shortcut_bonus() {
    let action =
        Action::new("test", "Something", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
    // "⌘e" matches shortcut_lower "⌘e"
    let score = ActionsDialog::score_action(&action, "⌘e");
    assert!(score >= 10, "Shortcut match should score >=10, got {score}");
}

// =====================================================================
// 30. Cross-context: all clipboard text actions have ScriptContext category
// =====================================================================

#[test]
fn all_clipboard_text_actions_have_script_context_category() {
    let entry = ClipboardEntryInfo {
        id: "cat-1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    for action in &actions {
        assert_eq!(
            action.category,
            ActionCategory::ScriptContext,
            "Action '{}' should have ScriptContext category",
            action.id
        );
    }
}

#[test]
fn all_path_actions_have_script_context_category() {
    let path_info = PathInfo {
        path: "/tmp/foo".into(),
        name: "foo".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    for action in &actions {
        assert_eq!(
            action.category,
            ActionCategory::ScriptContext,
            "Action '{}' should have ScriptContext category",
            action.id
        );
    }
}

#[test]
fn all_ai_bar_actions_have_script_context_category() {
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        assert_eq!(
            action.category,
            ActionCategory::ScriptContext,
            "Action '{}' should have ScriptContext category",
            action.id
        );
    }
}

#[test]
fn all_note_switcher_actions_have_script_context_category() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid".into(),
        title: "Note".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    for action in &actions {
        assert_eq!(
            action.category,
            ActionCategory::ScriptContext,
            "Action '{}' should have ScriptContext category",
            action.id
        );
    }
}
