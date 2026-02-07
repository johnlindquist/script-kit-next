
#[test]
fn batch21_path_trash_last_for_file() {
    let pi = PathInfo {
        path: "/f".into(),
        name: "f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    assert_eq!(actions.last().unwrap().id, "move_to_trash");
}

#[test]
fn batch21_path_trash_description_dir() {
    let pi = PathInfo {
        path: "/d".into(),
        name: "d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert!(trash
        .description
        .as_deref()
        .unwrap()
        .to_lowercase()
        .contains("folder"));
}

#[test]
fn batch21_path_trash_description_file() {
    let pi = PathInfo {
        path: "/f".into(),
        name: "f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert!(trash
        .description
        .as_deref()
        .unwrap()
        .to_lowercase()
        .contains("file"));
}

// ============================================================
// 27. Cross-context: all built-in IDs are snake_case
// ============================================================

fn assert_snake_case_ids(actions: &[Action], context: &str) {
    for a in actions {
        // Scriptlet-defined actions have "scriptlet_action:" prefix and are allowed colons
        if a.id.starts_with("scriptlet_action:") {
            continue;
        }
        assert!(
            !a.id.contains(' '),
            "{} action '{}' has spaces (not snake_case)",
            context,
            a.id
        );
        assert!(
            !a.id.contains('-'),
            "{} action '{}' has hyphens (not snake_case)",
            context,
            a.id
        );
    }
}

#[test]
fn batch21_snake_case_ids_script() {
    let s = ScriptInfo::new("s", "/p");
    assert_snake_case_ids(&get_script_context_actions(&s), "script");
}

#[test]
fn batch21_snake_case_ids_builtin() {
    let s = ScriptInfo::builtin("B");
    assert_snake_case_ids(&get_script_context_actions(&s), "builtin");
}

#[test]
fn batch21_snake_case_ids_scriptlet() {
    let s = ScriptInfo::scriptlet("S", "/p", None, None);
    assert_snake_case_ids(&get_script_context_actions(&s), "scriptlet");
}

#[test]
fn batch21_snake_case_ids_clipboard() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    assert_snake_case_ids(&get_clipboard_history_context_actions(&entry), "clipboard");
}

#[test]
fn batch21_snake_case_ids_ai() {
    assert_snake_case_ids(&get_ai_command_bar_actions(), "ai");
}

#[test]
fn batch21_snake_case_ids_notes() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    assert_snake_case_ids(&get_notes_command_bar_actions(&info), "notes");
}

// ============================================================
// 28. Cross-context: all actions have non-empty IDs and titles
// ============================================================

fn assert_nonempty_id_title(actions: &[Action], context: &str) {
    for a in actions {
        assert!(
            !a.id.is_empty(),
            "{}: action has empty ID (title={})",
            context,
            a.title
        );
        assert!(
            !a.title.is_empty(),
            "{}: action has empty title (id={})",
            context,
            a.id
        );
    }
}

#[test]
fn batch21_nonempty_id_title_script() {
    let s = ScriptInfo::new("s", "/p");
    assert_nonempty_id_title(&get_script_context_actions(&s), "script");
}

#[test]
fn batch21_nonempty_id_title_clipboard() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: true,
        preview: "".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    assert_nonempty_id_title(
        &get_clipboard_history_context_actions(&entry),
        "clipboard_image",
    );
}

#[test]
fn batch21_nonempty_id_title_path() {
    let pi = PathInfo {
        path: "/d".into(),
        name: "d".into(),
        is_dir: true,
    };
    assert_nonempty_id_title(&get_path_context_actions(&pi), "path");
}

#[test]
fn batch21_nonempty_id_title_file() {
    let fi = FileInfo {
        path: "/f".into(),
        name: "f".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    assert_nonempty_id_title(&get_file_context_actions(&fi), "file");
}

// ============================================================
// 29. Script context: deeplink description URL format
// ============================================================

#[test]
fn batch21_deeplink_url_format_script() {
    let s = ScriptInfo::new("My Script", "/p");
    let actions = get_script_context_actions(&s);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    let desc = dl.description.as_deref().unwrap();
    assert!(desc.contains("scriptkit://run/my-script"));
}

#[test]
fn batch21_deeplink_url_format_builtin() {
    let s = ScriptInfo::builtin("Clipboard History");
    let actions = get_script_context_actions(&s);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    let desc = dl.description.as_deref().unwrap();
    assert!(desc.contains("scriptkit://run/clipboard-history"));
}

#[test]
fn batch21_deeplink_url_format_scriptlet() {
    let s = ScriptInfo::scriptlet("Open GitHub", "/p", None, None);
    let actions = get_script_context_actions(&s);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    let desc = dl.description.as_deref().unwrap();
    assert!(desc.contains("scriptkit://run/open-github"));
}

// ============================================================
// 30. Cross-context: ID uniqueness within each context
// ============================================================

fn assert_unique_ids(actions: &[Action], context: &str) {
    let mut seen = std::collections::HashSet::new();
    for a in actions {
        assert!(
            seen.insert(&a.id),
            "{}: duplicate action ID '{}'",
            context,
            a.id
        );
    }
}

#[test]
fn batch21_unique_ids_script() {
    let s = ScriptInfo::new("s", "/p");
    assert_unique_ids(&get_script_context_actions(&s), "script");
}

#[test]
fn batch21_unique_ids_script_with_shortcut_and_alias() {
    let s = ScriptInfo::with_shortcut_and_alias("s", "/p", Some("⌘T".into()), Some("t".into()));
    assert_unique_ids(&get_script_context_actions(&s), "script_full");
}

#[test]
fn batch21_unique_ids_clipboard_text() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    assert_unique_ids(
        &get_clipboard_history_context_actions(&entry),
        "clipboard_text",
    );
}

#[test]
fn batch21_unique_ids_clipboard_image() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: true,
        preview: "".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    assert_unique_ids(
        &get_clipboard_history_context_actions(&entry),
        "clipboard_image",
    );
}

#[test]
fn batch21_unique_ids_ai() {
    assert_unique_ids(&get_ai_command_bar_actions(), "ai");
}

#[test]
fn batch21_unique_ids_notes() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    assert_unique_ids(&get_notes_command_bar_actions(&info), "notes");
}

#[test]
fn batch21_unique_ids_path_dir() {
    let pi = PathInfo {
        path: "/d".into(),
        name: "d".into(),
        is_dir: true,
    };
    assert_unique_ids(&get_path_context_actions(&pi), "path_dir");
}

#[test]
fn batch21_unique_ids_path_file() {
    let pi = PathInfo {
        path: "/f".into(),
        name: "f".into(),
        is_dir: false,
    };
    assert_unique_ids(&get_path_context_actions(&pi), "path_file");
}
