
#[test]
fn action_description_lower_is_cached() {
    let action = Action::new(
        "test",
        "title",
        Some("UPPERCASE DESC".to_string()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(action.description_lower, Some("uppercase desc".to_string()));
}

#[test]
fn action_description_lower_none_when_no_description() {
    let action = Action::new("test", "title", None, ActionCategory::ScriptContext);
    assert!(action.description_lower.is_none());
}

#[test]
fn action_shortcut_lower_is_cached_after_with_shortcut() {
    let action =
        Action::new("test", "title", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
    assert_eq!(action.shortcut_lower, Some("⌘⇧c".to_string()));
}

#[test]
fn action_shortcut_lower_none_by_default() {
    let action = Action::new("test", "title", None, ActionCategory::ScriptContext);
    assert!(action.shortcut_lower.is_none());
}

// =========================================================================
// 19. Action with_shortcut_opt
// =========================================================================

#[test]
fn action_with_shortcut_opt_some() {
    let action = Action::new("test", "title", None, ActionCategory::ScriptContext)
        .with_shortcut_opt(Some("⌘X".to_string()));
    assert_eq!(action.shortcut, Some("⌘X".to_string()));
    assert_eq!(action.shortcut_lower, Some("⌘x".to_string()));
}

#[test]
fn action_with_shortcut_opt_none() {
    let action =
        Action::new("test", "title", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
    assert!(action.shortcut.is_none());
    assert!(action.shortcut_lower.is_none());
}

// =========================================================================
// 20. Action builder chain
// =========================================================================

#[test]
fn action_builder_chain_all_methods() {
    let action = Action::new(
        "test",
        "Test",
        Some("desc".to_string()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘T")
    .with_icon(IconName::Star)
    .with_section("Section");

    assert_eq!(action.id, "test");
    assert_eq!(action.title, "Test");
    assert_eq!(action.description, Some("desc".to_string()));
    assert_eq!(action.shortcut, Some("⌘T".to_string()));
    assert_eq!(action.icon, Some(IconName::Star));
    assert_eq!(action.section, Some("Section".to_string()));
    assert!(!action.has_action);
    assert!(action.value.is_none());
}

#[test]
fn action_has_action_default_false() {
    let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
    assert!(!action.has_action);
}

// =========================================================================
// 21. ProtocolAction constructors and methods
// =========================================================================

#[test]
fn protocol_action_new_defaults() {
    let pa = ProtocolAction::new("Test".to_string());
    assert_eq!(pa.name, "Test");
    assert!(pa.description.is_none());
    assert!(pa.shortcut.is_none());
    assert!(pa.value.is_none());
    assert!(!pa.has_action);
    assert!(pa.visible.is_none());
    assert!(pa.close.is_none());
    assert!(pa.is_visible()); // None defaults to true
    assert!(pa.should_close()); // None defaults to true
}

#[test]
fn protocol_action_with_value() {
    let pa = ProtocolAction::with_value("Submit".to_string(), "submit-val".to_string());
    assert_eq!(pa.value, Some("submit-val".to_string()));
    assert!(!pa.has_action);
}

#[test]
fn protocol_action_visibility_combinations() {
    assert!(ProtocolAction {
        visible: None,
        ..ProtocolAction::new("a".into())
    }
    .is_visible());
    assert!(ProtocolAction {
        visible: Some(true),
        ..ProtocolAction::new("a".into())
    }
    .is_visible());
    assert!(!ProtocolAction {
        visible: Some(false),
        ..ProtocolAction::new("a".into())
    }
    .is_visible());
}

#[test]
fn protocol_action_close_combinations() {
    assert!(ProtocolAction {
        close: None,
        ..ProtocolAction::new("a".into())
    }
    .should_close());
    assert!(ProtocolAction {
        close: Some(true),
        ..ProtocolAction::new("a".into())
    }
    .should_close());
    assert!(!ProtocolAction {
        close: Some(false),
        ..ProtocolAction::new("a".into())
    }
    .should_close());
}

// =========================================================================
// 22. File context actions
// =========================================================================

#[test]
fn file_context_directory_primary_action() {
    let info = FileInfo {
        name: "Documents".to_string(),
        path: "/Users/test/Documents".to_string(),
        is_dir: true,
        file_type: FileType::Directory,
    };
    let actions = get_file_context_actions(&info);
    assert_eq!(actions[0].id, "open_directory");
    assert!(actions[0].title.contains("Documents"));
}

#[test]
fn file_context_file_primary_action() {
    let info = FileInfo {
        name: "readme.md".to_string(),
        path: "/Users/test/readme.md".to_string(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&info);
    assert_eq!(actions[0].id, "open_file");
    assert!(actions[0].title.contains("readme.md"));
}

#[test]
fn file_context_common_actions() {
    let info = FileInfo {
        name: "test.txt".to_string(),
        path: "/test.txt".to_string(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&info);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"reveal_in_finder"));
    assert!(ids.contains(&"copy_path"));
    assert!(ids.contains(&"copy_filename"));
}

// =========================================================================
// 23. Path context actions
// =========================================================================

#[test]
fn path_context_directory_has_open_directory() {
    let info = PathInfo {
        name: "src".to_string(),
        path: "/Users/test/src".to_string(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&info);
    assert_eq!(actions[0].id, "open_directory");
}

#[test]
fn path_context_file_has_select_file() {
    let info = PathInfo {
        name: "file.txt".to_string(),
        path: "/Users/test/file.txt".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    assert_eq!(actions[0].id, "select_file");
}

#[test]
fn path_context_trash_description_folder_vs_file() {
    let dir_info = PathInfo {
        name: "src".to_string(),
        path: "/src".to_string(),
        is_dir: true,
    };
    let file_info = PathInfo {
        name: "f.txt".to_string(),
        path: "/f.txt".to_string(),
        is_dir: false,
    };
    let dir_actions = get_path_context_actions(&dir_info);
    let dir_trash = find_action(&dir_actions, "move_to_trash").unwrap();
    let file_actions = get_path_context_actions(&file_info);
    let file_trash = find_action(&file_actions, "move_to_trash").unwrap();
    assert!(dir_trash.description.as_ref().unwrap().contains("folder"));
    assert!(file_trash.description.as_ref().unwrap().contains("file"));
}

#[test]
fn path_context_common_actions() {
    let info = PathInfo {
        name: "test".to_string(),
        path: "/test".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"copy_path"));
    assert!(ids.contains(&"open_in_finder"));
    assert!(ids.contains(&"open_in_editor"));
    assert!(ids.contains(&"open_in_terminal"));
    assert!(ids.contains(&"copy_filename"));
    assert!(ids.contains(&"move_to_trash"));
}

// =========================================================================
// 24. Scriptlet with custom actions
// =========================================================================

#[test]
fn scriptlet_custom_actions_ordering() {
    let script = ScriptInfo::scriptlet("Test Scriptlet", "/path/test.md", None, None);
    let scriptlet = Scriptlet::new(
        "Test Scriptlet".to_string(),
        "bash".to_string(),
        "echo hi".to_string(),
    );
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    // run_script should always be first
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn scriptlet_with_custom_action_has_has_action_true() {
    let script = ScriptInfo::scriptlet("Test Scriptlet", "/path/test.md", None, None);
    let mut scriptlet = Scriptlet::new(
        "Test Scriptlet".to_string(),
        "bash".to_string(),
        "echo hi".to_string(),
    );
    scriptlet.actions.push(ScriptletAction {
        name: "Copy to Clipboard".to_string(),
        command: "copy-to-clipboard".to_string(),
        tool: "bash".to_string(),
        code: "pbcopy".to_string(),
        inputs: vec![],
        shortcut: Some("cmd+c".to_string()),
        description: None,
    });
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let custom = actions
        .iter()
        .find(|a| a.id.starts_with("scriptlet_action:"))
        .unwrap();
    assert!(custom.has_action);
    assert!(custom.value.is_some());
}

// =========================================================================
// 25. ID uniqueness across contexts
// =========================================================================

#[test]
fn script_context_no_duplicate_ids() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let mut ids: Vec<&str> = action_ids(&actions);
    let total = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(total, ids.len(), "Duplicate IDs found in script context");
}

#[test]
fn clipboard_context_no_duplicate_ids() {
    let entry = make_text_entry(false, None);
    let actions = get_clipboard_history_context_actions(&entry);
    let mut ids: Vec<&str> = action_ids(&actions);
    let total = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(total, ids.len(), "Duplicate IDs found in clipboard context");
}

#[test]
fn ai_command_bar_no_duplicate_ids() {
    let actions = get_ai_command_bar_actions();
    let mut ids: Vec<&str> = action_ids(&actions);
    let total = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(total, ids.len(), "Duplicate IDs found in AI command bar");
}

#[test]
fn path_context_no_duplicate_ids() {
    let info = PathInfo {
        name: "test".to_string(),
        path: "/test".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    let mut ids: Vec<&str> = action_ids(&actions);
    let total = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(total, ids.len(), "Duplicate IDs found in path context");
}

// =========================================================================
// 26. Enum defaults
// =========================================================================

#[test]
fn enum_defaults() {
    assert_eq!(SearchPosition::default(), SearchPosition::Bottom);
    assert_eq!(SectionStyle::default(), SectionStyle::Separators);
    assert_eq!(AnchorPosition::default(), AnchorPosition::Bottom);
}

#[test]
fn actions_dialog_config_default() {
    let config = ActionsDialogConfig::default();
    assert_eq!(config.search_position, SearchPosition::Bottom);
    assert_eq!(config.section_style, SectionStyle::Separators);
    assert_eq!(config.anchor, AnchorPosition::Bottom);
    assert!(!config.show_icons);
    assert!(!config.show_footer);
}

// =========================================================================
// 27. Action categories
// =========================================================================

#[test]
fn all_script_context_actions_use_script_context_category() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    for action in &actions {
        assert_eq!(
            action.category,
            ActionCategory::ScriptContext,
            "Action '{}' has wrong category",
            action.id
        );
    }
}

#[test]
fn all_clipboard_actions_use_script_context_category() {
    let entry = make_text_entry(false, None);
    let actions = get_clipboard_history_context_actions(&entry);
    for action in &actions {
        assert_eq!(
            action.category,
            ActionCategory::ScriptContext,
            "Clipboard action '{}' has wrong category",
            action.id
        );
    }
}

// =========================================================================
// 28. Snake_case ID convention
// =========================================================================

#[test]
fn script_action_ids_are_snake_case() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    for action in &get_script_context_actions(&script) {
        assert!(
            !action.id.contains(' '),
            "Action ID '{}' contains space",
            action.id
        );
        assert!(
            action.id == action.id.to_lowercase()
                || action
                    .id
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c == '_'),
            "Action ID '{}' should be snake_case",
            action.id
        );
    }
}

#[test]
fn clipboard_action_ids_are_snake_case() {
    let entry = make_text_entry(false, None);
    for action in &get_clipboard_history_context_actions(&entry) {
        assert!(
            !action.id.contains(' '),
            "Clipboard action ID '{}' contains space",
            action.id
        );
    }
}
