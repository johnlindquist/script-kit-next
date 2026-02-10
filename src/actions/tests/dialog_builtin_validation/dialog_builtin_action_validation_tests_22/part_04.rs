
#[test]
fn batch22_cross_clipboard_non_empty_ids_titles() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    for a in get_clipboard_history_context_actions(&entry) {
        assert!(!a.id.is_empty());
        assert!(!a.title.is_empty());
    }
}

#[test]
fn batch22_cross_ai_non_empty_ids_titles() {
    for a in get_ai_command_bar_actions() {
        assert!(!a.id.is_empty());
        assert!(!a.title.is_empty());
    }
}

#[test]
fn batch22_cross_notes_non_empty_ids_titles() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for a in get_notes_command_bar_actions(&info) {
        assert!(!a.id.is_empty());
        assert!(!a.title.is_empty());
    }
}

// ============================================================
// 28. Cross-context: all built-in action IDs are snake_case
// ============================================================

fn is_snake_case(s: &str) -> bool {
    !s.contains(' ') && !s.contains('-') && s == s.to_lowercase()
        || s.starts_with("select_model_") // model IDs may contain mixed case
        || s.starts_with("note_") // note IDs contain UUIDs
        || s.starts_with("last_used_")
        || s.starts_with("preset_")
        || s.starts_with("model_")
        || s.starts_with("scriptlet_action:")
}

#[test]
fn batch22_cross_script_ids_snake_case() {
    let s = ScriptInfo::new("s", "/p");
    for a in get_script_context_actions(&s) {
        assert!(is_snake_case(&a.id), "ID '{}' should be snake_case", a.id);
    }
}

#[test]
fn batch22_cross_ai_ids_snake_case() {
    for a in get_ai_command_bar_actions() {
        assert!(is_snake_case(&a.id), "ID '{}' should be snake_case", a.id);
    }
}

#[test]
fn batch22_cross_clipboard_ids_snake_case() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    for a in get_clipboard_history_context_actions(&entry) {
        assert!(is_snake_case(&a.id), "ID '{}' should be snake_case", a.id);
    }
}

#[test]
fn batch22_cross_path_ids_snake_case() {
    let pi = PathInfo {
        name: "f".into(),
        path: "/f".into(),
        is_dir: false,
    };
    for a in get_path_context_actions(&pi) {
        assert!(is_snake_case(&a.id), "ID '{}' should be snake_case", a.id);
    }
}

// ============================================================
// 29. format_shortcut_hint: function keys and special aliases
// ============================================================

#[test]
fn batch22_format_shortcut_control_alias() {
    let result = ActionsDialog::format_shortcut_hint("control+c");
    assert_eq!(result, "⌃C");
}

#[test]
fn batch22_format_shortcut_meta_alias() {
    let result = ActionsDialog::format_shortcut_hint("meta+v");
    assert_eq!(result, "⌘V");
}

#[test]
fn batch22_format_shortcut_super_alias() {
    let result = ActionsDialog::format_shortcut_hint("super+v");
    assert_eq!(result, "⌘V");
}

#[test]
fn batch22_format_shortcut_option_alias() {
    let result = ActionsDialog::format_shortcut_hint("option+space");
    assert_eq!(result, "⌥␣");
}

#[test]
fn batch22_format_shortcut_esc_alias() {
    let result = ActionsDialog::format_shortcut_hint("esc");
    assert_eq!(result, "⎋");
}

// ============================================================
// 30. ActionsDialogConfig default values
// ============================================================

#[test]
fn batch22_actions_dialog_config_default_search_bottom() {
    let cfg = ActionsDialogConfig::default();
    assert_eq!(cfg.search_position, SearchPosition::Bottom);
}

#[test]
fn batch22_actions_dialog_config_default_section_separators() {
    let cfg = ActionsDialogConfig::default();
    assert_eq!(cfg.section_style, SectionStyle::Separators);
}

#[test]
fn batch22_actions_dialog_config_default_anchor_bottom() {
    let cfg = ActionsDialogConfig::default();
    assert_eq!(cfg.anchor, AnchorPosition::Bottom);
}

#[test]
fn batch22_actions_dialog_config_default_no_icons() {
    let cfg = ActionsDialogConfig::default();
    assert!(!cfg.show_icons);
}

#[test]
fn batch22_actions_dialog_config_default_no_footer() {
    let cfg = ActionsDialogConfig::default();
    assert!(!cfg.show_footer);
}
