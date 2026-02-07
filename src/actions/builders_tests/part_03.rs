// 8. Scriptlet context actions with custom H3 actions
// ============================================================

#[test]
fn scriptlet_context_includes_shortcut_and_alias_dynamics() {
    // Scriptlet with shortcut + alias
    let info = ScriptInfo::scriptlet(
        "Quick Open",
        "/path/to/urls.md#quick-open",
        Some("cmd+o".into()),
        Some("qo".into()),
    );
    let actions = get_scriptlet_context_actions_with_custom(&info, None);
    let ids = action_ids(&actions);

    // Should have update/remove (not add) for both shortcut and alias
    assert!(ids.contains(&"update_shortcut"));
    assert!(ids.contains(&"remove_shortcut"));
    assert!(!ids.contains(&"add_shortcut"));

    assert!(ids.contains(&"update_alias"));
    assert!(ids.contains(&"remove_alias"));
    assert!(!ids.contains(&"add_alias"));

    // Should have scriptlet-specific actions
    assert!(ids.contains(&"edit_scriptlet"));
    assert!(ids.contains(&"reveal_scriptlet_in_finder"));
    assert!(ids.contains(&"copy_scriptlet_path"));
    assert!(ids.contains(&"copy_content"));
    assert!(ids.contains(&"copy_deeplink"));
}

#[test]
fn scriptlet_context_without_shortcut_shows_add() {
    let info = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&info, None);
    let ids = action_ids(&actions);

    assert!(ids.contains(&"add_shortcut"));
    assert!(ids.contains(&"add_alias"));
    assert!(!ids.contains(&"update_shortcut"));
    assert!(!ids.contains(&"remove_shortcut"));
}

// ============================================================
// 9. Action struct builder methods
// ============================================================

#[test]
fn action_with_shortcut_caches_lowercase() {
    let action =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧K");

    assert_eq!(action.shortcut, Some("⌘⇧K".into()));
    assert!(action.shortcut_lower.is_some());
}

#[test]
fn action_with_shortcut_opt_none_no_op() {
    let action =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut_opt(None);

    assert!(action.shortcut.is_none());
    assert!(action.shortcut_lower.is_none());
}

#[test]
fn action_caches_lowercase_title_and_description() {
    let action = Action::new(
        "test",
        "Copy Path",
        Some("Copy the FULL Path".into()),
        ActionCategory::ScriptContext,
    );

    assert_eq!(action.title_lower, "copy path");
    assert_eq!(
        action.description_lower.as_deref(),
        Some("copy the full path")
    );
}

#[test]
fn action_has_action_defaults_to_false() {
    let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
    assert!(
        !action.has_action,
        "Built-in actions should default has_action to false"
    );
}

// ============================================================
// 10. Global actions (currently empty, verify contract)
// ============================================================

#[test]
fn global_actions_are_empty() {
    let actions = get_global_actions();
    assert!(
        actions.is_empty(),
        "Global actions should be empty (Settings/Quit are in main menu)"
    );
}

// ============================================================
// 11. Note switcher actions (Cmd+P in Notes window)
// ============================================================

#[test]
fn note_switcher_empty_shows_no_notes_message() {
    let notes: Vec<NoteSwitcherNoteInfo> = vec![];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "no_notes");
    assert_eq!(actions[0].title, "No notes yet");
    assert!(actions[0].description.as_ref().unwrap().contains("⌘N"));
    assert_eq!(actions[0].section.as_deref(), Some("Notes"));
}

#[test]
fn note_switcher_single_current_note() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid-1".into(),
        title: "My Note".into(),
        char_count: 42,
        is_current: true,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "note_uuid-1");
    assert!(
        actions[0].title.starts_with("• "),
        "Current note should have bullet prefix"
    );
    assert!(actions[0].title.contains("My Note"));
    assert_eq!(actions[0].description.as_deref(), Some("42 chars"));
}

#[test]
fn note_switcher_pinned_note_icon() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "pinned-1".into(),
            title: "Pinned Note".into(),
            char_count: 100,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "current-1".into(),
            title: "Current Note".into(),
            char_count: 50,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "plain-1".into(),
            title: "Plain Note".into(),
            char_count: 10,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);

    assert_eq!(actions.len(), 3);

    // Pinned note gets StarFilled icon
    let pinned = find_action(&actions, "note_pinned-1").unwrap();
    assert_eq!(
        pinned.icon,
        Some(crate::designs::icon_variations::IconName::StarFilled)
    );
    assert!(
        !pinned.title.starts_with("• "),
        "Non-current should not have bullet"
    );

    // Current note gets Check icon
    let current = find_action(&actions, "note_current-1").unwrap();
    assert_eq!(
        current.icon,
        Some(crate::designs::icon_variations::IconName::Check)
    );
    assert!(current.title.starts_with("• "));

    // Plain note gets File icon
    let plain = find_action(&actions, "note_plain-1").unwrap();
    assert_eq!(
        plain.icon,
        Some(crate::designs::icon_variations::IconName::File)
    );
}

#[test]
fn note_switcher_singular_character_count() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "one-char".into(),
        title: "Tiny".into(),
        char_count: 1,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(
        actions[0].description.as_deref(),
        Some("1 char"),
        "Single char should use singular 'char'"
    );
}

#[test]
fn note_switcher_zero_characters() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "empty".into(),
        title: "Empty Note".into(),
        char_count: 0,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(
        actions[0].description.as_deref(),
        Some("0 chars"),
        "Zero chars should use plural 'chars'"
    );
}

#[test]
fn note_switcher_all_notes_have_section() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "a".into(),
            title: "A".into(),
            char_count: 5,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "b".into(),
            title: "B".into(),
            char_count: 10,
            is_current: true,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    for action in &actions {
        let section = action.section.as_deref();
        assert!(
            section == Some("Pinned") || section == Some("Recent"),
            "Note switcher action should be in 'Pinned' or 'Recent' section, got {:?}",
            section
        );
    }
}

