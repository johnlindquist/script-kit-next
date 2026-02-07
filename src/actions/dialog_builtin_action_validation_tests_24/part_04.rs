
#[test]
fn batch24_cmdbar_no_search_hidden() {
    let config = CommandBarConfig::no_search();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Hidden);
}

#[test]
fn batch24_cmdbar_notes_style_separators() {
    let config = CommandBarConfig::notes_style();
    assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
    assert!(config.dialog_config.show_icons);
    assert!(config.dialog_config.show_footer);
}

// ============================================================
// 29. Action builder: defaults and chaining
// ============================================================

#[test]
fn batch24_action_default_has_action_false() {
    let a = Action::new("id", "Title", None, ActionCategory::ScriptContext);
    assert!(!a.has_action);
}

#[test]
fn batch24_action_default_value_none() {
    let a = Action::new("id", "Title", None, ActionCategory::ScriptContext);
    assert!(a.value.is_none());
}

#[test]
fn batch24_action_default_icon_none() {
    let a = Action::new("id", "Title", None, ActionCategory::ScriptContext);
    assert!(a.icon.is_none());
}

#[test]
fn batch24_action_default_section_none() {
    let a = Action::new("id", "Title", None, ActionCategory::ScriptContext);
    assert!(a.section.is_none());
}

#[test]
fn batch24_action_chain_preserves_all() {
    let a = Action::new(
        "id",
        "Title",
        Some("Desc".to_string()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘C")
    .with_icon(IconName::Copy)
    .with_section("Section");
    assert_eq!(a.shortcut.as_deref(), Some("⌘C"));
    assert_eq!(a.icon, Some(IconName::Copy));
    assert_eq!(a.section.as_deref(), Some("Section"));
    assert_eq!(a.description.as_deref(), Some("Desc"));
}

// ============================================================
// 30. Cross-context: all actions have ScriptContext category
// ============================================================

#[test]
fn batch24_cross_script_all_script_context() {
    let script = ScriptInfo::new("test", "/path");
    for a in get_script_context_actions(&script) {
        assert_eq!(a.category, ActionCategory::ScriptContext);
    }
}

#[test]
fn batch24_cross_clipboard_all_script_context() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    for a in get_clipboard_history_context_actions(&entry) {
        assert_eq!(a.category, ActionCategory::ScriptContext);
    }
}

#[test]
fn batch24_cross_ai_all_script_context() {
    for a in get_ai_command_bar_actions() {
        assert_eq!(a.category, ActionCategory::ScriptContext);
    }
}

#[test]
fn batch24_cross_notes_all_script_context() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for a in get_notes_command_bar_actions(&info) {
        assert_eq!(a.category, ActionCategory::ScriptContext);
    }
}

#[test]
fn batch24_cross_path_all_script_context() {
    let p = PathInfo::new("f", "/f", false);
    for a in get_path_context_actions(&p) {
        assert_eq!(a.category, ActionCategory::ScriptContext);
    }
}

#[test]
fn batch24_cross_file_all_script_context() {
    let f = FileInfo {
        path: "/f.txt".to_string(),
        name: "f.txt".to_string(),
        file_type: FileType::File,
        is_dir: false,
    };
    for a in get_file_context_actions(&f) {
        assert_eq!(a.category, ActionCategory::ScriptContext);
    }
}
