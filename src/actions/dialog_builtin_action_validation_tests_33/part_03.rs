
#[test]
fn notes_cmd_bar_trash_view_has_3_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note + browse_notes + enable_auto_sizing = 3 (trash blocks selection-dependent)
    assert_eq!(actions.len(), 3);
}

#[test]
fn notes_cmd_bar_full_mode_has_10_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert_eq!(actions.len(), 10);
}

#[test]
fn notes_cmd_bar_auto_sizing_enabled_has_9_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert_eq!(actions.len(), 9);
}

// =====================================================================
// 24. Path context: exact action count for file vs dir
// =====================================================================

#[test]
fn path_context_file_has_7_actions() {
    let path_info = PathInfo {
        path: "/tmp/test.txt".to_string(),
        name: "test.txt".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    assert_eq!(actions.len(), 7);
}

#[test]
fn path_context_dir_has_7_actions() {
    let path_info = PathInfo {
        path: "/tmp/mydir".to_string(),
        name: "mydir".to_string(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    assert_eq!(actions.len(), 7);
}

#[test]
fn path_context_file_first_is_select_file() {
    let path_info = PathInfo {
        path: "/tmp/test.txt".to_string(),
        name: "test.txt".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    assert_eq!(actions[0].id, "select_file");
}

#[test]
fn path_context_dir_first_is_open_directory() {
    let path_info = PathInfo {
        path: "/tmp/mydir".to_string(),
        name: "mydir".to_string(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    assert_eq!(actions[0].id, "open_directory");
}

// =====================================================================
// 25. File context: macOS action count for file vs dir
// =====================================================================

#[cfg(target_os = "macos")]
#[test]
fn file_context_file_macos_has_7_actions() {
    let file_info = FileInfo {
        path: "/tmp/test.txt".to_string(),
        name: "test.txt".to_string(),
        file_type: crate::file_search::FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    // open_file + reveal + quick_look + open_with + show_info + copy_path + copy_filename = 7
    assert_eq!(actions.len(), 7);
}

#[cfg(target_os = "macos")]
#[test]
fn file_context_dir_macos_has_6_actions() {
    let file_info = FileInfo {
        path: "/tmp/mydir".to_string(),
        name: "mydir".to_string(),
        file_type: crate::file_search::FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&file_info);
    // open_directory + reveal + open_with + show_info + copy_path + copy_filename = 6 (no quick_look)
    assert_eq!(actions.len(), 6);
}

#[test]
fn file_context_file_title_quoted() {
    let file_info = FileInfo {
        path: "/tmp/doc.pdf".to_string(),
        name: "doc.pdf".to_string(),
        file_type: crate::file_search::FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    assert_eq!(actions[0].title, "Open \"doc.pdf\"");
}

#[test]
fn file_context_dir_title_quoted() {
    let file_info = FileInfo {
        path: "/tmp/docs".to_string(),
        name: "docs".to_string(),
        file_type: crate::file_search::FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&file_info);
    assert_eq!(actions[0].title, "Open \"docs\"");
}

// =====================================================================
// 26. Scriptlet context with H3 custom: ordering invariant
// =====================================================================

#[test]
fn scriptlet_with_custom_run_before_custom_actions() {
    use crate::scriptlets::{Scriptlet, ScriptletAction};

    let script = ScriptInfo::scriptlet("Test", "/test.md", None, None);
    let mut scriptlet = Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Custom".to_string(),
        command: "custom".to_string(),
        tool: "bash".to_string(),
        code: "echo custom".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];

    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let run_idx = actions.iter().position(|a| a.id == "run_script").unwrap();
    let custom_idx = actions
        .iter()
        .position(|a| a.id == "scriptlet_action:custom")
        .unwrap();
    assert!(run_idx < custom_idx);
}

#[test]
fn scriptlet_with_custom_builtins_after_custom() {
    use crate::scriptlets::{Scriptlet, ScriptletAction};

    let script = ScriptInfo::scriptlet("Test", "/test.md", None, None);
    let mut scriptlet = Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Custom".to_string(),
        command: "custom".to_string(),
        tool: "bash".to_string(),
        code: "echo custom".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];

    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let custom_idx = actions
        .iter()
        .position(|a| a.id == "scriptlet_action:custom")
        .unwrap();
    let edit_idx = actions
        .iter()
        .position(|a| a.id == "edit_scriptlet")
        .unwrap();
    assert!(custom_idx < edit_idx);
}

#[test]
fn scriptlet_custom_action_has_action_true() {
    use crate::scriptlets::{Scriptlet, ScriptletAction};

    let script = ScriptInfo::scriptlet("Test", "/test.md", None, None);
    let mut scriptlet = Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Custom".to_string(),
        command: "custom".to_string(),
        tool: "bash".to_string(),
        code: "echo custom".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];

    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let custom = actions
        .iter()
        .find(|a| a.id == "scriptlet_action:custom")
        .unwrap();
    assert!(custom.has_action);
}

// =====================================================================
// 27. New chat: section ordering and ID format
// =====================================================================

#[test]
fn new_chat_last_used_section_name() {
    let last_used = vec![NewChatModelInfo {
        model_id: "claude-3".to_string(),
        display_name: "Claude 3".to_string(),
        provider: "anthropic".to_string(),
        provider_display_name: "Anthropic".to_string(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
}

#[test]
fn new_chat_model_id_format() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt-4".to_string(),
        display_name: "GPT-4".to_string(),
        provider: "openai".to_string(),
        provider_display_name: "OpenAI".to_string(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].id, "model_0");
}

#[test]
fn new_chat_preset_id_format() {
    let presets = vec![NewChatPresetInfo {
        id: "general".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].id, "preset_general");
}

#[test]
fn new_chat_preset_description_is_none() {
    let presets = vec![NewChatPresetInfo {
        id: "code".to_string(),
        name: "Code".to_string(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert!(actions[0].description.is_none());
}

// =====================================================================
// 28. Action builder: with_shortcut_opt(None) vs with_shortcut_opt(Some)
// =====================================================================

#[test]
fn action_with_shortcut_opt_none_leaves_shortcut_none() {
    let action =
        Action::new("id", "Title", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
    assert!(action.shortcut.is_none());
    assert!(action.shortcut_lower.is_none());
}

#[test]
fn action_with_shortcut_opt_some_sets_shortcut() {
    let action = Action::new("id", "Title", None, ActionCategory::ScriptContext)
        .with_shortcut_opt(Some("⌘K".to_string()));
    assert_eq!(action.shortcut.as_deref(), Some("⌘K"));
    assert_eq!(action.shortcut_lower.as_deref(), Some("⌘k"));
}

#[test]
fn action_with_icon_sets_icon() {
    let action =
        Action::new("id", "Title", None, ActionCategory::ScriptContext).with_icon(IconName::Copy);
    assert_eq!(action.icon, Some(IconName::Copy));
}

#[test]
fn action_with_section_sets_section() {
    let action =
        Action::new("id", "Title", None, ActionCategory::ScriptContext).with_section("Response");
    assert_eq!(action.section.as_deref(), Some("Response"));
}

// =====================================================================
// 29. Cross-context: all built-in actions have has_action=false
// =====================================================================

#[test]
fn all_script_actions_have_has_action_false() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    for action in get_script_context_actions(&script) {
        assert!(
            !action.has_action,
            "Action {} should have has_action=false",
            action.id
        );
    }
}

#[test]
fn all_clipboard_actions_have_has_action_false() {
    let entry = ClipboardEntryInfo {
        id: "t".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    for action in get_clipboard_history_context_actions(&entry) {
        assert!(
            !action.has_action,
            "Action {} should have has_action=false",
            action.id
        );
    }
}

#[test]
fn all_ai_bar_actions_have_has_action_false() {
    for action in get_ai_command_bar_actions() {
        assert!(
            !action.has_action,
            "Action {} should have has_action=false",
            action.id
        );
    }
}

#[test]
fn all_notes_actions_have_has_action_false() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for action in get_notes_command_bar_actions(&info) {
        assert!(
            !action.has_action,
            "Action {} should have has_action=false",
            action.id
        );
    }
}

#[test]
fn all_path_actions_have_has_action_false() {
    let path_info = PathInfo {
        path: "/tmp/test.txt".to_string(),
        name: "test.txt".to_string(),
        is_dir: false,
    };
    for action in get_path_context_actions(&path_info) {
        assert!(
            !action.has_action,
            "Action {} should have has_action=false",
            action.id
        );
    }
}

#[test]
fn all_file_actions_have_has_action_false() {
    let file_info = FileInfo {
        path: "/tmp/test.txt".to_string(),
        name: "test.txt".to_string(),
        file_type: crate::file_search::FileType::File,
        is_dir: false,
    };
    for action in get_file_context_actions(&file_info) {
        assert!(
            !action.has_action,
            "Action {} should have has_action=false",
            action.id
        );
    }
}

// =====================================================================
// 30. Cross-context: all actions have non-empty title and id
// =====================================================================

#[test]
fn all_ai_bar_actions_have_nonempty_title_and_id() {
    for action in get_ai_command_bar_actions() {
        assert!(!action.id.is_empty(), "Action should have non-empty id");
        assert!(
            !action.title.is_empty(),
            "Action {} should have non-empty title",
            action.id
        );
    }
}

#[test]
fn all_notes_actions_have_nonempty_title_and_id() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for action in get_notes_command_bar_actions(&info) {
        assert!(!action.id.is_empty());
        assert!(
            !action.title.is_empty(),
            "Action {} should have non-empty title",
            action.id
        );
    }
}
