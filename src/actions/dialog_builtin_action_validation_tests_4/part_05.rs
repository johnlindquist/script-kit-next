
#[test]
fn file_dir_primary_title_includes_dirname() {
    let info = FileInfo {
        path: "/tmp/build".to_string(),
        name: "build".to_string(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&info);
    assert!(
        actions[0].title.contains("build"),
        "Primary title should include dirname: {}",
        actions[0].title
    );
}

// =========================================================================
// 25. Deeplink description format in script context
// =========================================================================

#[test]
fn deeplink_description_contains_url_with_formatted_name() {
    let script = ScriptInfo::new("My Cool Script", "/path/to/script.ts");
    let actions = get_script_context_actions(&script);
    let dl = find_action(&actions, "copy_deeplink").unwrap();
    assert!(
        dl.description
            .as_ref()
            .unwrap()
            .contains("scriptkit://run/my-cool-script"),
        "Deeplink description should contain formatted URL: {:?}",
        dl.description
    );
}

#[test]
fn deeplink_description_for_builtin() {
    let builtin = ScriptInfo::builtin("Clipboard History");
    let actions = get_script_context_actions(&builtin);
    let dl = find_action(&actions, "copy_deeplink").unwrap();
    assert!(dl
        .description
        .as_ref()
        .unwrap()
        .contains("scriptkit://run/clipboard-history"));
}

// =========================================================================
// 26. All built-in actions have has_action=false
// =========================================================================

#[test]
fn script_context_all_actions_have_has_action_false() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    for action in &actions {
        assert!(
            !action.has_action,
            "Built-in action '{}' should have has_action=false",
            action.id
        );
    }
}

#[test]
fn clipboard_context_all_actions_have_has_action_false() {
    let entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    for action in &actions {
        assert!(
            !action.has_action,
            "Clipboard action '{}' should have has_action=false",
            action.id
        );
    }
}

#[test]
fn file_context_all_actions_have_has_action_false() {
    let info = FileInfo {
        path: "/tmp/test.txt".to_string(),
        name: "test.txt".to_string(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    for action in &actions {
        assert!(
            !action.has_action,
            "File action '{}' should have has_action=false",
            action.id
        );
    }
}

#[test]
fn path_context_all_actions_have_has_action_false() {
    let path = PathInfo {
        name: "test.txt".to_string(),
        path: "/tmp/test.txt".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    for action in &actions {
        assert!(
            !action.has_action,
            "Path action '{}' should have has_action=false",
            action.id
        );
    }
}

#[test]
fn ai_command_bar_all_actions_have_has_action_false() {
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        assert!(
            !action.has_action,
            "AI action '{}' should have has_action=false",
            action.id
        );
    }
}

// =========================================================================
// 27. All actions have non-empty title and ID
// =========================================================================

#[test]
fn script_context_all_actions_have_nonempty_title_and_id() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    for action in &actions {
        assert!(!action.id.is_empty(), "Action ID should not be empty");
        assert!(!action.title.is_empty(), "Action title should not be empty");
    }
}

#[test]
fn clipboard_context_all_actions_have_nonempty_title_and_id() {
    let entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    for action in &actions {
        assert!(!action.id.is_empty());
        assert!(!action.title.is_empty());
    }
}

#[test]
fn ai_command_bar_all_actions_have_nonempty_title_and_id() {
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        assert!(!action.id.is_empty());
        assert!(!action.title.is_empty());
    }
}

// =========================================================================
// 28. Action ID uniqueness within contexts
// =========================================================================

#[test]
fn script_context_ids_are_unique() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        ids.len(),
        actions.len(),
        "Duplicate action IDs found in script context"
    );
}

#[test]
fn clipboard_text_context_ids_are_unique() {
    let entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        ids.len(),
        actions.len(),
        "Duplicate action IDs found in clipboard text context"
    );
}

#[test]
fn clipboard_image_context_ids_are_unique() {
    let entry = ClipboardEntryInfo {
        id: "i1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "Image".to_string(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        ids.len(),
        actions.len(),
        "Duplicate action IDs found in clipboard image context"
    );
}

#[test]
fn path_context_ids_are_unique() {
    let path = PathInfo {
        name: "test.txt".to_string(),
        path: "/tmp/test.txt".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        ids.len(),
        actions.len(),
        "Duplicate action IDs found in path context"
    );
}

#[test]
fn ai_command_bar_ids_are_unique() {
    let actions = get_ai_command_bar_actions();
    let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        ids.len(),
        actions.len(),
        "Duplicate action IDs found in AI command bar"
    );
}

// =========================================================================
// 29. Clipboard destructive actions always last three
// =========================================================================

#[test]
fn clipboard_destructive_actions_are_last_three() {
    let entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let len = actions.len();
    assert!(len >= 3);
    assert_eq!(actions[len - 3].id, "clipboard_delete");
    assert_eq!(actions[len - 2].id, "clipboard_delete_multiple");
    assert_eq!(actions[len - 1].id, "clipboard_delete_all");
}

#[test]
fn clipboard_image_destructive_actions_are_last_three() {
    let entry = ClipboardEntryInfo {
        id: "i1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "Image".to_string(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let len = actions.len();
    assert!(len >= 3);
    assert_eq!(actions[len - 3].id, "clipboard_delete");
    assert_eq!(actions[len - 2].id, "clipboard_delete_multiple");
    assert_eq!(actions[len - 1].id, "clipboard_delete_all");
}

// =========================================================================
// 30. Clipboard paste is always first, copy is always second
// =========================================================================

#[test]
fn clipboard_paste_is_first_copy_is_second() {
    let entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[0].id, "clipboard_paste");
    assert_eq!(actions[1].id, "clipboard_copy");
}

// =========================================================================
// 31. All actions have ScriptContext category
// =========================================================================

#[test]
fn all_contexts_produce_script_context_category() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    for action in &get_script_context_actions(&script) {
        assert_eq!(action.category, ActionCategory::ScriptContext);
    }

    let entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    for action in &get_clipboard_history_context_actions(&entry) {
        assert_eq!(action.category, ActionCategory::ScriptContext);
    }

    let path = PathInfo {
        name: "test".to_string(),
        path: "/tmp/test".to_string(),
        is_dir: false,
    };
    for action in &get_path_context_actions(&path) {
        assert_eq!(action.category, ActionCategory::ScriptContext);
    }

    let file = FileInfo {
        path: "/tmp/test.txt".to_string(),
        name: "test.txt".to_string(),
        file_type: FileType::File,
        is_dir: false,
    };
    for action in &get_file_context_actions(&file) {
        assert_eq!(action.category, ActionCategory::ScriptContext);
    }

    for action in &get_ai_command_bar_actions() {
        assert_eq!(action.category, ActionCategory::ScriptContext);
    }
}

// =========================================================================
// 32. Primary action always first across contexts
// =========================================================================

#[test]
fn primary_action_first_in_script_context() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].id, "run_script");
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn primary_action_first_in_file_context() {
    let file = FileInfo {
        path: "/tmp/test.txt".to_string(),
        name: "test.txt".to_string(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    assert_eq!(actions[0].id, "open_file");
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn primary_action_first_in_path_context() {
    let path = PathInfo {
        name: "readme.md".to_string(),
        path: "/tmp/readme.md".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].id, "select_file");
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn primary_action_first_in_clipboard_context() {
    let entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[0].id, "clipboard_paste");
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

// =========================================================================
// 33. Ordering determinism
// =========================================================================

#[test]
fn script_context_ordering_deterministic_across_calls() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions1 = get_script_context_actions(&script);
    let actions2 = get_script_context_actions(&script);
    let ids1 = action_ids(&actions1);
    let ids2 = action_ids(&actions2);
    assert_eq!(ids1, ids2, "Action ordering should be deterministic");
}

#[test]
fn clipboard_context_ordering_deterministic_across_calls() {
    let entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions1 = get_clipboard_history_context_actions(&entry);
    let actions2 = get_clipboard_history_context_actions(&entry);
    let ids1 = action_ids(&actions1);
    let ids2 = action_ids(&actions2);
    assert_eq!(ids1, ids2);
}

#[test]
fn ai_command_bar_ordering_deterministic_across_calls() {
    let actions1 = get_ai_command_bar_actions();
    let actions2 = get_ai_command_bar_actions();
    let ids1 = action_ids(&actions1);
    let ids2 = action_ids(&actions2);
    assert_eq!(ids1, ids2);
}
