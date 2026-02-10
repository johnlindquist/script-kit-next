
#[test]
fn clipboard_paste_title_with_unicode_app_name() {
    let entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: Some("日本語App".to_string()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = find_action(&actions, "clipboard_paste").unwrap();
    assert_eq!(paste.title, "Paste to 日本語App");
}

#[test]
fn clipboard_paste_title_without_app_name() {
    let entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = find_action(&actions, "clipboard_paste").unwrap();
    assert_eq!(paste.title, "Paste to Active App");
}

// =========================================================================
// 19. Chat with no models, no messages, no response
// =========================================================================

#[test]
fn chat_no_models_no_messages_no_response() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"continue_in_chat"));
    assert!(!ids.contains(&"copy_response"));
    assert!(!ids.contains(&"clear_conversation"));
    assert_eq!(actions.len(), 1);
}

#[test]
fn chat_with_response_only_has_copy_response() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"continue_in_chat"));
    assert!(ids.contains(&"copy_response"));
    assert!(!ids.contains(&"clear_conversation"));
}

#[test]
fn chat_with_messages_only_has_clear_conversation() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"continue_in_chat"));
    assert!(!ids.contains(&"copy_response"));
    assert!(ids.contains(&"clear_conversation"));
}

#[test]
fn chat_with_all_flags_has_all_actions() {
    let info = ChatPromptInfo {
        current_model: Some("Claude 3.5".to_string()),
        available_models: vec![ChatModelInfo {
            id: "claude-3.5".to_string(),
            display_name: "Claude 3.5".to_string(),
            provider: "Anthropic".to_string(),
        }],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"continue_in_chat"));
    assert!(ids.contains(&"copy_response"));
    assert!(ids.contains(&"clear_conversation"));
    // Plus model selection action
    assert!(ids.iter().any(|id| id.starts_with("select_model_")));
}

#[test]
fn chat_model_checkmark_on_current() {
    let info = ChatPromptInfo {
        current_model: Some("Claude 3.5".to_string()),
        available_models: vec![
            ChatModelInfo {
                id: "claude-3.5".to_string(),
                display_name: "Claude 3.5".to_string(),
                provider: "Anthropic".to_string(),
            },
            ChatModelInfo {
                id: "gpt-4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let claude = find_action(&actions, "select_model_claude-3.5").unwrap();
    let gpt = find_action(&actions, "select_model_gpt-4").unwrap();
    assert!(
        claude.title.contains('✓'),
        "Current model should have checkmark"
    );
    assert!(
        !gpt.title.contains('✓'),
        "Non-current model should not have checkmark"
    );
}

// =========================================================================
// 20. Scriptlet custom actions ordering preservation
// =========================================================================

#[test]
fn scriptlet_custom_actions_preserve_declaration_order() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut scriptlet = Scriptlet::new(
        "Test".to_string(),
        "bash".to_string(),
        "echo main".to_string(),
    );
    scriptlet.actions = vec![
        ScriptletAction {
            name: "First Action".to_string(),
            command: "first".to_string(),
            tool: "bash".to_string(),
            code: "echo first".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
        ScriptletAction {
            name: "Second Action".to_string(),
            command: "second".to_string(),
            tool: "bash".to_string(),
            code: "echo second".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
        ScriptletAction {
            name: "Third Action".to_string(),
            command: "third".to_string(),
            tool: "bash".to_string(),
            code: "echo third".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
    ];

    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));

    let custom_ids: Vec<&str> = actions
        .iter()
        .filter(|a| a.id.starts_with("scriptlet_action:"))
        .map(|a| a.id.as_str())
        .collect();

    assert_eq!(
        custom_ids,
        vec![
            "scriptlet_action:first",
            "scriptlet_action:second",
            "scriptlet_action:third"
        ]
    );
}

#[test]
fn scriptlet_custom_actions_appear_after_run_before_builtins() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut scriptlet = Scriptlet::new(
        "Test".to_string(),
        "bash".to_string(),
        "echo main".to_string(),
    );
    scriptlet.actions = vec![ScriptletAction {
        name: "Custom".to_string(),
        command: "custom".to_string(),
        tool: "bash".to_string(),
        code: "echo custom".to_string(),
        inputs: vec![],
        shortcut: Some("cmd+1".to_string()),
        description: Some("A custom action".to_string()),
    }];

    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let ids = action_ids(&actions);

    let run_idx = ids.iter().position(|id| *id == "run_script").unwrap();
    let custom_idx = ids
        .iter()
        .position(|id| *id == "scriptlet_action:custom")
        .unwrap();
    let edit_idx = ids.iter().position(|id| *id == "edit_scriptlet").unwrap();

    assert!(run_idx < custom_idx, "run before custom");
    assert!(custom_idx < edit_idx, "custom before edit");
}

#[test]
fn scriptlet_custom_actions_have_has_action_true() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut scriptlet = Scriptlet::new(
        "Test".to_string(),
        "bash".to_string(),
        "echo main".to_string(),
    );
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
    let custom = find_action(&actions, "scriptlet_action:custom").unwrap();
    assert!(
        custom.has_action,
        "Custom scriptlet action should have has_action=true"
    );
    assert_eq!(custom.value, Some("custom".to_string()));
}

#[test]
fn scriptlet_custom_action_shortcut_formatted() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut scriptlet = Scriptlet::new(
        "Test".to_string(),
        "bash".to_string(),
        "echo main".to_string(),
    );
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".to_string(),
        command: "copy".to_string(),
        tool: "bash".to_string(),
        code: "echo copy".to_string(),
        inputs: vec![],
        shortcut: Some("cmd+shift+c".to_string()),
        description: None,
    }];

    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let custom = find_action(&actions, "scriptlet_action:copy").unwrap();
    assert_eq!(
        custom.shortcut,
        Some("⌘⇧C".to_string()),
        "Shortcut should be formatted with symbols"
    );
}

// =========================================================================
// 21. Action constructor lowercase caching
// =========================================================================

#[test]
fn action_title_lower_caches_correctly() {
    let action = Action::new("test", "Edit Script", None, ActionCategory::ScriptContext);
    assert_eq!(action.title_lower, "edit script");
}

#[test]
fn action_description_lower_caches_correctly() {
    let action = Action::new(
        "test",
        "Test",
        Some("Open In Editor".to_string()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(action.description_lower, Some("open in editor".to_string()));
}

#[test]
fn action_description_lower_none_when_no_description() {
    let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
    assert_eq!(action.description_lower, None);
}

#[test]
fn action_shortcut_lower_set_after_with_shortcut() {
    let action =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
    assert_eq!(action.shortcut_lower, Some("⌘e".to_string()));
}

#[test]
fn action_shortcut_lower_none_without_shortcut() {
    let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
    assert_eq!(action.shortcut_lower, None);
}

#[test]
fn action_title_lower_unicode() {
    let action = Action::new("test", "Café Résumé", None, ActionCategory::ScriptContext);
    assert_eq!(action.title_lower, "café résumé");
}

#[test]
fn action_with_shortcut_opt_some_sets_shortcut() {
    let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
        .with_shortcut_opt(Some("⌘X".to_string()));
    assert_eq!(action.shortcut, Some("⌘X".to_string()));
    assert_eq!(action.shortcut_lower, Some("⌘x".to_string()));
}

#[test]
fn action_with_shortcut_opt_none_leaves_shortcut_unset() {
    let action =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
    assert_eq!(action.shortcut, None);
    assert_eq!(action.shortcut_lower, None);
}

// =========================================================================
// 22. CommandBarConfig preset field values
// =========================================================================

#[test]
fn command_bar_config_ai_style_fields() {
    let config = CommandBarConfig::ai_style();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
    assert_eq!(config.dialog_config.section_style, SectionStyle::Headers);
    assert!(config.dialog_config.show_icons);
}

#[test]
fn command_bar_config_main_menu_style_fields() {
    let config = CommandBarConfig::main_menu_style();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Bottom);
    assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
    assert!(!config.dialog_config.show_icons);
}

#[test]
fn command_bar_config_notes_style_fields() {
    let config = CommandBarConfig::notes_style();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
    assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
    assert!(config.dialog_config.show_icons);
}

#[test]
fn command_bar_config_no_search_hides_search() {
    let config = CommandBarConfig::no_search();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Hidden);
}

// =========================================================================
// 23. Path context primary action varies by is_dir
// =========================================================================

#[test]
fn path_dir_primary_is_open_directory() {
    let path = PathInfo {
        name: "Documents".to_string(),
        path: "/home/user/Documents".to_string(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].id, "open_directory");
    assert!(actions[0].title.contains("Documents"));
}

#[test]
fn path_file_primary_is_select_file() {
    let path = PathInfo {
        name: "readme.md".to_string(),
        path: "/home/user/readme.md".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].id, "select_file");
    assert!(actions[0].title.contains("readme.md"));
}

#[test]
fn path_trash_description_differs_by_is_dir() {
    let dir_path = PathInfo {
        name: "src".to_string(),
        path: "/home/user/src".to_string(),
        is_dir: true,
    };
    let file_path = PathInfo {
        name: "file.txt".to_string(),
        path: "/home/user/file.txt".to_string(),
        is_dir: false,
    };
    let dir_actions = get_path_context_actions(&dir_path);
    let file_actions = get_path_context_actions(&file_path);
    let dir_trash = find_action(&dir_actions, "move_to_trash").unwrap();
    let file_trash = find_action(&file_actions, "move_to_trash").unwrap();
    assert_eq!(dir_trash.description.as_deref(), Some("Delete folder"));
    assert_eq!(file_trash.description.as_deref(), Some("Delete file"));
}

// =========================================================================
// 24. File context primary title includes name
// =========================================================================

#[test]
fn file_primary_title_includes_filename() {
    let info = FileInfo {
        path: "/tmp/report.pdf".to_string(),
        name: "report.pdf".to_string(),
        file_type: FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    assert!(
        actions[0].title.contains("report.pdf"),
        "Primary title should include filename: {}",
        actions[0].title
    );
}
