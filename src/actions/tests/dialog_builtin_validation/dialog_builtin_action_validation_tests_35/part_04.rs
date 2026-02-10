
// =====================================================================
// 28. Clipboard: share action details
// =====================================================================

#[test]
fn clipboard_share_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "sh-1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let share = actions.iter().find(|a| a.id == "clipboard_share").unwrap();
    assert_eq!(share.shortcut.as_ref().unwrap(), "⇧⌘E");
}

#[test]
fn clipboard_share_title() {
    let entry = ClipboardEntryInfo {
        id: "sh-2".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let share = actions.iter().find(|a| a.id == "clipboard_share").unwrap();
    assert_eq!(share.title, "Share...");
}

#[test]
fn clipboard_share_desc_mentions_share() {
    let entry = ClipboardEntryInfo {
        id: "sh-3".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let share = actions.iter().find(|a| a.id == "clipboard_share").unwrap();
    assert!(share
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("share"));
}

#[test]
fn clipboard_share_present_for_image() {
    let entry = ClipboardEntryInfo {
        id: "sh-4".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_share"));
}

// =====================================================================
// 29. Action builder: cached lowercase consistency
// =====================================================================

#[test]
fn action_title_lower_matches_title() {
    let action = Action::new("id", "Hello World", None, ActionCategory::ScriptContext);
    assert_eq!(action.title_lower, "hello world");
}

#[test]
fn action_description_lower_matches_desc() {
    let action = Action::new(
        "id",
        "T",
        Some("My Description".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(action.description_lower.as_ref().unwrap(), "my description");
}

#[test]
fn action_shortcut_lower_after_with_shortcut() {
    let action = Action::new("id", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
    assert_eq!(action.shortcut_lower.as_ref().unwrap(), "⌘⇧c");
}

#[test]
fn action_no_shortcut_lower_is_none() {
    let action = Action::new("id", "T", None, ActionCategory::ScriptContext);
    assert!(action.shortcut_lower.is_none());
}

// =====================================================================
// 30. Cross-context: all built-in actions use snake_case IDs
// =====================================================================

#[test]
fn script_actions_ids_snake_case() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    for a in get_script_context_actions(&script) {
        assert!(
            !a.id.contains(' '),
            "Action ID '{}' should not contain spaces",
            a.id
        );
        assert!(
            !a.id.contains('-') || a.id.starts_with("scriptlet_action:"),
            "Action ID '{}' should be snake_case (no hyphens)",
            a.id
        );
    }
}

#[test]
fn file_actions_ids_snake_case() {
    let f = FileInfo {
        path: "/t.txt".into(),
        name: "t.txt".into(),
        file_type: crate::file_search::FileType::File,
        is_dir: false,
    };
    for a in get_file_context_actions(&f) {
        assert!(
            !a.id.contains(' '),
            "Action ID '{}' should not contain spaces",
            a.id
        );
    }
}

#[test]
fn path_actions_ids_snake_case() {
    let p = PathInfo {
        path: "/t".into(),
        name: "t".into(),
        is_dir: false,
    };
    for a in get_path_context_actions(&p) {
        assert!(
            !a.id.contains(' '),
            "Action ID '{}' should not contain spaces",
            a.id
        );
    }
}

#[test]
fn clipboard_actions_ids_snake_case() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    for a in get_clipboard_history_context_actions(&entry) {
        assert!(
            !a.id.contains(' '),
            "Action ID '{}' should not contain spaces",
            a.id
        );
    }
}
