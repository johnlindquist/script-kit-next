
// ─────────────────────────────────────────────
// 30. Cross-context: all action IDs are non-empty
// ─────────────────────────────────────────────

#[test]
fn cat27_30_script_action_ids_non_empty() {
    let script = ScriptInfo::new("test", "/p.ts");
    let actions = get_script_context_actions(&script);
    for a in &actions {
        assert!(!a.id.is_empty(), "action ID should not be empty");
    }
}

#[test]
fn cat27_30_clipboard_action_ids_non_empty() {
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
        assert!(!a.id.is_empty());
    }
}

#[test]
fn cat27_30_ai_action_ids_non_empty() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(!a.id.is_empty());
    }
}

#[test]
fn cat27_30_path_action_ids_non_empty() {
    let path = PathInfo {
        name: "f".into(),
        path: "/f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    for a in &actions {
        assert!(!a.id.is_empty());
    }
}

#[test]
fn cat27_30_file_action_ids_non_empty() {
    let file = FileInfo {
        name: "f.txt".into(),
        path: "/f.txt".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&file);
    for a in &actions {
        assert!(!a.id.is_empty());
    }
}

#[test]
fn cat27_30_notes_action_ids_non_empty() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    for a in &actions {
        assert!(!a.id.is_empty());
    }
}
