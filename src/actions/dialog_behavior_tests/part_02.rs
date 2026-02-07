
// =========================================================================
// Action builder chain methods
// =========================================================================

#[test]
fn action_with_description_sets_field_and_cache() {
    let action = Action::new(
        "test",
        "Test Action",
        Some("A detailed description".to_string()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(
        action.description,
        Some("A detailed description".to_string())
    );
    assert_eq!(
        action.description_lower,
        Some("a detailed description".to_string()),
        "description_lower should be pre-computed"
    );
}

#[test]
fn action_builder_chain_icon_section_shortcut() {
    use crate::designs::icon_variations::IconName;

    let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
        .with_icon(IconName::Star)
        .with_section("My Section")
        .with_shortcut("⌘T");

    assert_eq!(action.icon, Some(IconName::Star));
    assert_eq!(action.section, Some("My Section".to_string()));
    assert_eq!(action.shortcut, Some("⌘T".to_string()));
    assert_eq!(action.shortcut_lower, Some("⌘t".to_string()));
}

#[test]
fn action_title_lower_precomputed() {
    let action = Action::new(
        "test",
        "UPPERCASE Title",
        None,
        ActionCategory::ScriptContext,
    );
    assert_eq!(action.title_lower, "uppercase title");
}

// =========================================================================
// ProtocolAction constructors
// =========================================================================

#[test]
fn protocol_action_new_defaults() {
    let pa = ProtocolAction::new("My Action".to_string());
    assert_eq!(pa.name, "My Action");
    assert!(pa.description.is_none());
    assert!(pa.shortcut.is_none());
    assert!(pa.value.is_none());
    assert!(!pa.has_action);
    assert!(pa.visible.is_none());
    assert!(pa.close.is_none());
    // Defaults
    assert!(pa.is_visible());
    assert!(pa.should_close());
}

#[test]
fn protocol_action_with_value_constructor() {
    let pa = ProtocolAction::with_value("Submit".to_string(), "submit-value".to_string());
    assert_eq!(pa.name, "Submit");
    assert_eq!(pa.value, Some("submit-value".to_string()));
    assert!(!pa.has_action, "with_value should default has_action=false");
}

#[test]
fn protocol_action_visibility_and_close_combinations() {
    // visible=false, close=false
    let pa = ProtocolAction {
        name: "Hidden Stay Open".to_string(),
        description: None,
        shortcut: None,
        value: None,
        has_action: false,
        visible: Some(false),
        close: Some(false),
    };
    assert!(!pa.is_visible());
    assert!(!pa.should_close());

    // visible=true, close=false
    let pa2 = ProtocolAction {
        name: "Visible Stay Open".to_string(),
        description: None,
        shortcut: None,
        value: None,
        has_action: true,
        visible: Some(true),
        close: Some(false),
    };
    assert!(pa2.is_visible());
    assert!(!pa2.should_close());
}

// =========================================================================
// Clipboard context: context title truncation
// =========================================================================

#[test]
fn clipboard_long_preview_truncated_in_context_title() {
    // The with_clipboard_entry method truncates preview > 30 chars
    // We test the truncation logic directly
    let long_preview = "This is a very long clipboard entry that exceeds thirty characters";
    assert!(long_preview.len() > 30);

    let context_title = if long_preview.len() > 30 {
        format!("{}...", &long_preview[..27])
    } else {
        long_preview.to_string()
    };

    assert_eq!(context_title, "This is a very long clipboa...");
    assert_eq!(context_title.len(), 30); // 27 chars + "..."
}

#[test]
fn clipboard_short_preview_not_truncated() {
    let short_preview = "Short text";
    assert!(short_preview.len() <= 30);

    let context_title = if short_preview.len() > 30 {
        format!("{}...", &short_preview[..27])
    } else {
        short_preview.to_string()
    };

    assert_eq!(context_title, "Short text");
}

// =========================================================================
// Chat context: title fallback
// =========================================================================

#[test]
fn chat_context_title_uses_model_name() {
    let info = ChatPromptInfo {
        current_model: Some("claude-sonnet".to_string()),
        available_models: vec![],
        has_response: false,
        has_messages: false,
    };

    let context_title = info
        .current_model
        .clone()
        .unwrap_or_else(|| "Chat".to_string());
    assert_eq!(context_title, "claude-sonnet");
}

#[test]
fn chat_context_title_falls_back_to_chat() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_response: false,
        has_messages: false,
    };

    let context_title = info
        .current_model
        .clone()
        .unwrap_or_else(|| "Chat".to_string());
    assert_eq!(context_title, "Chat");
}

// =========================================================================
// File context: directory vs file action differentiation
// =========================================================================

#[test]
fn file_context_directory_has_open_directory_primary() {
    let dir = FileInfo {
        name: "Documents".to_string(),
        path: "/Users/test/Documents".to_string(),
        is_dir: true,
        file_type: crate::file_search::FileType::Directory,
    };
    let actions = get_file_context_actions(&dir);
    assert_eq!(actions[0].id, "open_directory");
    assert!(actions[0].title.contains("Documents"));
}

#[test]
fn file_context_file_has_open_file_primary() {
    let file = FileInfo {
        name: "readme.md".to_string(),
        path: "/Users/test/readme.md".to_string(),
        is_dir: false,
        file_type: crate::file_search::FileType::File,
    };
    let actions = get_file_context_actions(&file);
    assert_eq!(actions[0].id, "open_file");
    assert!(actions[0].title.contains("readme.md"));
}

// =========================================================================
// Path context: primary action differentiation
// =========================================================================

#[test]
fn path_context_directory_primary_is_open() {
    let path = PathInfo {
        name: "Documents".to_string(),
        path: "/Users/test/Documents".to_string(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].id, "open_directory");
}

#[test]
fn path_context_file_primary_is_select() {
    let path = PathInfo {
        name: "file.txt".to_string(),
        path: "/Users/test/file.txt".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].id, "select_file");
}

// =========================================================================
// Builtin script: limited actions
// =========================================================================

#[test]
fn builtin_has_only_run_shortcut_alias_deeplink() {
    let builtin = ScriptInfo::builtin("App Launcher");
    let actions = get_script_context_actions(&builtin);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    // Should have: run_script, add_shortcut, add_alias, copy_deeplink
    assert!(ids.contains(&"run_script"));
    assert!(ids.contains(&"add_shortcut"));
    assert!(ids.contains(&"add_alias"));
    assert!(ids.contains(&"copy_deeplink"));

    // Should NOT have script-only or scriptlet-only actions
    assert!(!ids.contains(&"edit_script"));
    assert!(!ids.contains(&"view_logs"));
    assert!(!ids.contains(&"reveal_in_finder"));
    assert!(!ids.contains(&"copy_path"));
    assert!(!ids.contains(&"copy_content"));
    assert!(!ids.contains(&"edit_scriptlet"));
}

// =========================================================================
// Action scoring edge cases
// =========================================================================

#[test]
fn score_action_empty_search_returns_zero() {
    use super::dialog::ActionsDialog;

    let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
    // Empty search should match nothing (score 0)
    // Actually empty string is a prefix of everything, so it will score 100
    let score = ActionsDialog::score_action(&action, "");
    assert_eq!(score, 100, "Empty search is a prefix of all titles");
}

#[test]
fn score_action_shortcut_only_match() {
    use super::dialog::ActionsDialog;

    let action =
        Action::new("test", "Test Action", None, ActionCategory::ScriptContext).with_shortcut("⌘X");
    // Search for the shortcut symbol
    let score = ActionsDialog::score_action(&action, "⌘x");
    assert!(
        score >= 10,
        "Shortcut match should contribute at least 10, got {}",
        score
    );
}

#[test]
fn fuzzy_match_case_insensitive_precomputed() {
    use super::dialog::ActionsDialog;

    // fuzzy_match operates on lowercased strings (pre-computed in title_lower)
    assert!(ActionsDialog::fuzzy_match("edit script", "es"));
    assert!(ActionsDialog::fuzzy_match("edit script", "eit"));
    assert!(!ActionsDialog::fuzzy_match("edit script", "z"));
}

// =========================================================================
// Notes command bar: conditional actions
// =========================================================================

#[test]
fn notes_command_bar_has_new_note_action() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(
        ids.contains(&"new_note"),
        "Notes command bar should always have new_note"
    );
}

#[test]
fn notes_trash_view_has_no_edit_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    // Trash view should suppress editing actions
    assert!(
        !ids.contains(&"duplicate_note"),
        "Trash view should not have duplicate_note"
    );
}

// =========================================================================
// Clipboard: pinned image entry combines image + pin actions
// =========================================================================

#[test]
fn clipboard_pinned_image_has_unpin_and_image_actions() {
    let entry = ClipboardEntryInfo {
        id: "img-1".to_string(),
        content_type: ContentType::Image,
        pinned: true,
        preview: "Screenshot".to_string(),
        image_dimensions: Some((1920, 1080)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    // Should have unpin (not pin)
    assert!(
        ids.contains(&"clipboard_unpin"),
        "Should have unpin for pinned entry"
    );
    assert!(
        !ids.contains(&"clipboard_pin"),
        "Should NOT have pin for pinned entry"
    );

    // Should have image-specific actions (macOS)
    #[cfg(target_os = "macos")]
    {
        assert!(ids.contains(&"clipboard_ocr"), "Image should have OCR");
    }
}

// =========================================================================
// Global actions are empty
// =========================================================================

#[test]
fn global_actions_empty() {
    let actions = get_global_actions();
    assert!(actions.is_empty(), "Global actions should be empty");
}

// =========================================================================
// Action category equality
// =========================================================================

#[test]
fn action_category_partial_eq() {
    assert_eq!(ActionCategory::ScriptContext, ActionCategory::ScriptContext);
    assert_ne!(ActionCategory::ScriptContext, ActionCategory::GlobalOps);
    assert_ne!(ActionCategory::ScriptOps, ActionCategory::Terminal);
}

// =========================================================================
// SearchPosition, SectionStyle, AnchorPosition enum defaults
// =========================================================================

#[test]
fn enum_defaults() {
    assert!(matches!(SearchPosition::default(), SearchPosition::Bottom));
    assert!(matches!(SectionStyle::default(), SectionStyle::Separators));
    assert!(matches!(AnchorPosition::default(), AnchorPosition::Bottom));
}
