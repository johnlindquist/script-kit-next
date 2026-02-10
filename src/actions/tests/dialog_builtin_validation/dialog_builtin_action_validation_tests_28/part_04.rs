
#[test]
fn cat28_27_description_lower_none_when_no_desc() {
    let action = Action::new("a", "A", None, ActionCategory::ScriptContext);
    assert!(action.description_lower.is_none());
}

#[test]
fn cat28_27_shortcut_lower_after_with_shortcut() {
    let action = Action::new("a", "A", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧D");
    assert_eq!(action.shortcut_lower.as_deref(), Some("⌘⇧d"));
}

// =============================================================================
// Category 28: CommandBarConfig presets — dialog_config fields
// =============================================================================

#[test]
fn cat28_28_main_menu_search_bottom() {
    let cfg = CommandBarConfig::main_menu_style();
    assert_eq!(cfg.dialog_config.search_position, SearchPosition::Bottom);
}

#[test]
fn cat28_28_ai_style_search_top() {
    let cfg = CommandBarConfig::ai_style();
    assert_eq!(cfg.dialog_config.search_position, SearchPosition::Top);
}

#[test]
fn cat28_28_no_search_hidden() {
    let cfg = CommandBarConfig::no_search();
    assert_eq!(cfg.dialog_config.search_position, SearchPosition::Hidden);
}

#[test]
fn cat28_28_notes_style_search_top() {
    let cfg = CommandBarConfig::notes_style();
    assert_eq!(cfg.dialog_config.search_position, SearchPosition::Top);
}

// =============================================================================
// Category 29: Cross-context — action ID uniqueness within each context
// =============================================================================

#[test]
fn cat28_29_script_ids_unique() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len(), "Duplicate IDs found: {:?}", ids);
}

#[test]
fn cat28_29_clipboard_ids_unique() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len(), "Duplicate IDs found: {:?}", ids);
}

#[test]
fn cat28_29_ai_ids_unique() {
    let actions = get_ai_command_bar_actions();
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len(), "Duplicate IDs found: {:?}", ids);
}

#[test]
fn cat28_29_path_ids_unique() {
    let path_info = PathInfo {
        name: "test".into(),
        path: "/test".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len(), "Duplicate IDs found: {:?}", ids);
}

// =============================================================================
// Category 30: Cross-context — all contexts produce non-empty title and id
// =============================================================================

#[test]
fn cat28_30_script_actions_non_empty_titles() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    for a in &actions {
        assert!(!a.id.is_empty(), "Action ID must not be empty");
        assert!(!a.title.is_empty(), "Action title must not be empty");
    }
}

#[test]
fn cat28_30_clipboard_actions_non_empty_titles() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    for a in &actions {
        assert!(!a.id.is_empty(), "Action ID must not be empty");
        assert!(!a.title.is_empty(), "Action title must not be empty");
    }
}

#[test]
fn cat28_30_notes_actions_non_empty_titles() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    for a in &actions {
        assert!(!a.id.is_empty(), "Action ID must not be empty");
        assert!(!a.title.is_empty(), "Action title must not be empty");
    }
}

#[test]
fn cat28_30_file_actions_non_empty_titles() {
    let file_info = FileInfo {
        path: "/test/f.txt".into(),
        name: "f.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    for a in &actions {
        assert!(!a.id.is_empty(), "Action ID must not be empty");
        assert!(!a.title.is_empty(), "Action title must not be empty");
    }
}

#[test]
fn cat28_30_ai_actions_non_empty_titles() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(!a.id.is_empty(), "Action ID must not be empty");
        assert!(!a.title.is_empty(), "Action title must not be empty");
    }
}

#[test]
fn cat28_30_path_actions_non_empty_titles() {
    let path_info = PathInfo {
        name: "test".into(),
        path: "/test".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    for a in &actions {
        assert!(!a.id.is_empty(), "Action ID must not be empty");
        assert!(!a.title.is_empty(), "Action title must not be empty");
    }
}
