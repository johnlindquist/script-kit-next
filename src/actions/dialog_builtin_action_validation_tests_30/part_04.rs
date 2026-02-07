
#[test]
fn batch30_agent_has_reveal_in_finder() {
    let mut script = crate::actions::types::ScriptInfo::new("a", "/p");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
}

#[test]
fn batch30_agent_has_copy_path() {
    let mut script = crate::actions::types::ScriptInfo::new("a", "/p");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    assert!(actions.iter().any(|a| a.id == "copy_path"));
}

// ---------------------------------------------------------------------------
// 28. Action builder: cached lowercase fields
// ---------------------------------------------------------------------------
#[test]
fn batch30_action_title_lower_precomputed() {
    let action = Action::new("x", "Hello World", None, ActionCategory::ScriptContext);
    assert_eq!(action.title_lower, "hello world");
}

#[test]
fn batch30_action_description_lower_precomputed() {
    let action = Action::new(
        "x",
        "T",
        Some("Open in $EDITOR".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(action.description_lower.as_deref(), Some("open in $editor"));
}

#[test]
fn batch30_action_shortcut_lower_after_with_shortcut() {
    let action = Action::new("x", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
    assert_eq!(action.shortcut_lower.as_deref(), Some("⌘⇧c"));
}

#[test]
fn batch30_action_no_shortcut_lower_is_none() {
    let action = Action::new("x", "T", None, ActionCategory::ScriptContext);
    assert!(action.shortcut_lower.is_none());
}

// ---------------------------------------------------------------------------
// 29. Action builder: with_icon and with_section
// ---------------------------------------------------------------------------
#[test]
fn batch30_action_with_icon_sets_field() {
    let action =
        Action::new("x", "T", None, ActionCategory::ScriptContext).with_icon(IconName::Star);
    assert_eq!(action.icon, Some(IconName::Star));
}

#[test]
fn batch30_action_new_no_icon() {
    let action = Action::new("x", "T", None, ActionCategory::ScriptContext);
    assert!(action.icon.is_none());
}

#[test]
fn batch30_action_with_section_sets_field() {
    let action =
        Action::new("x", "T", None, ActionCategory::ScriptContext).with_section("MySection");
    assert_eq!(action.section.as_deref(), Some("MySection"));
}

#[test]
fn batch30_action_new_no_section() {
    let action = Action::new("x", "T", None, ActionCategory::ScriptContext);
    assert!(action.section.is_none());
}

// ---------------------------------------------------------------------------
// 30. Cross-context: all built-in actions have has_action=false
// ---------------------------------------------------------------------------
#[test]
fn batch30_cross_context_script_actions_has_action_false() {
    let script = crate::actions::types::ScriptInfo::new("s", "/p.ts");
    let actions = get_script_context_actions(&script);
    for a in &actions {
        assert!(
            !a.has_action,
            "Script action '{}' should have has_action=false",
            a.id
        );
    }
}

#[test]
fn batch30_cross_context_clipboard_actions_has_action_false() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    for a in &actions {
        assert!(
            !a.has_action,
            "Clipboard action '{}' should have has_action=false",
            a.id
        );
    }
}

#[test]
fn batch30_cross_context_file_actions_has_action_false() {
    let info = FileInfo {
        path: "/f.txt".into(),
        name: "f.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    for a in &actions {
        assert!(
            !a.has_action,
            "File action '{}' should have has_action=false",
            a.id
        );
    }
}

#[test]
fn batch30_cross_context_path_actions_has_action_false() {
    let info = PathInfo {
        path: "/f".into(),
        name: "f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    for a in &actions {
        assert!(
            !a.has_action,
            "Path action '{}' should have has_action=false",
            a.id
        );
    }
}

#[test]
fn batch30_cross_context_ai_bar_actions_has_action_false() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(
            !a.has_action,
            "AI bar action '{}' should have has_action=false",
            a.id
        );
    }
}

#[test]
fn batch30_cross_context_notes_actions_has_action_false() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    for a in &actions {
        assert!(
            !a.has_action,
            "Notes action '{}' should have has_action=false",
            a.id
        );
    }
}
