
#[test]
fn deeplink_name_single_word() {
    assert_eq!(to_deeplink_name("script"), "script");
}

// =========================================================================
// 22. CommandBarConfig preset dialog_config specifics
// =========================================================================

#[test]
fn command_bar_ai_style_has_search_top_and_headers() {
    let config = CommandBarConfig::ai_style();
    assert!(matches!(
        config.dialog_config.search_position,
        SearchPosition::Top
    ));
    assert!(matches!(
        config.dialog_config.section_style,
        SectionStyle::Headers
    ));
    assert!(config.dialog_config.show_icons);
    assert!(config.dialog_config.show_footer);
}

#[test]
fn command_bar_main_menu_has_search_bottom_and_separators() {
    let config = CommandBarConfig::main_menu_style();
    assert!(matches!(
        config.dialog_config.search_position,
        SearchPosition::Bottom
    ));
    assert!(matches!(
        config.dialog_config.section_style,
        SectionStyle::Separators
    ));
    assert!(!config.dialog_config.show_icons);
    assert!(!config.dialog_config.show_footer);
}

#[test]
fn command_bar_notes_has_search_top_and_separators() {
    let config = CommandBarConfig::notes_style();
    assert!(matches!(
        config.dialog_config.search_position,
        SearchPosition::Top
    ));
    assert!(matches!(
        config.dialog_config.section_style,
        SectionStyle::Separators
    ));
    assert!(config.dialog_config.show_icons);
    assert!(config.dialog_config.show_footer);
}

#[test]
fn command_bar_no_search_has_hidden_search() {
    let config = CommandBarConfig::no_search();
    assert!(matches!(
        config.dialog_config.search_position,
        SearchPosition::Hidden
    ));
}

// =========================================================================
// 23. Grouped items with SectionStyle::None
// =========================================================================

#[test]
fn grouped_items_none_style_has_no_headers_or_separators() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
    for item in &grouped {
        assert!(
            matches!(item, GroupedActionItem::Item(_)),
            "SectionStyle::None should produce only Items, not headers/separators"
        );
    }
}

#[test]
fn grouped_items_none_style_count_matches_filtered() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
    assert_eq!(grouped.len(), filtered.len());
}

// =========================================================================
// 24. Coerce selection on notes grouped actions
// =========================================================================

#[test]
fn coerce_selection_on_notes_grouped_finds_valid_item() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    let result = coerce_action_selection(&grouped, 0);
    assert!(
        result.is_some(),
        "Should find valid item in notes grouped actions"
    );
    if let Some(idx) = result {
        assert!(matches!(grouped[idx], GroupedActionItem::Item(_)));
    }
}

// =========================================================================
// 25. title_lower correctness for AI and notes contexts
// =========================================================================

#[test]
fn title_lower_matches_title_for_all_ai_actions() {
    for action in &get_ai_command_bar_actions() {
        assert_eq!(
            action.title_lower,
            action.title.to_lowercase(),
            "title_lower mismatch for AI action '{}'",
            action.id
        );
    }
}

#[test]
fn title_lower_matches_title_for_all_notes_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for action in &get_notes_command_bar_actions(&info) {
        assert_eq!(
            action.title_lower,
            action.title.to_lowercase(),
            "title_lower mismatch for notes action '{}'",
            action.id
        );
    }
}

#[test]
fn title_lower_matches_title_for_note_switcher_actions() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "a".into(),
            title: "Capital Title".into(),
            char_count: 10,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "b".into(),
            title: "ALL CAPS NOTE".into(),
            char_count: 20,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];
    for action in &get_note_switcher_actions(&notes) {
        assert_eq!(
            action.title_lower,
            action.title.to_lowercase(),
            "title_lower mismatch for note switcher action '{}'",
            action.id
        );
    }
}

// =========================================================================
// 26. Scriptlet custom action with shortcut and description
// =========================================================================

#[test]
fn scriptlet_custom_action_shortcut_is_formatted() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo hi".into());
    scriptlet.actions.push(ScriptletAction {
        name: "Copy Output".into(),
        command: "copy-output".into(),
        tool: "bash".into(),
        code: "echo | pbcopy".into(),
        inputs: vec![],
        shortcut: Some("cmd+shift+c".into()),
        description: None,
    });
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let custom = actions
        .iter()
        .find(|a| a.id.starts_with("scriptlet_action:"))
        .unwrap();
    let sc = custom.shortcut.as_ref().unwrap();
    assert!(
        sc.contains('⌘') && sc.contains('⇧'),
        "Scriptlet shortcut should be formatted with symbols, got '{}'",
        sc
    );
}

#[test]
fn scriptlet_custom_action_description_propagated() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo hi".into());
    scriptlet.actions.push(ScriptletAction {
        name: "Explained".into(),
        command: "explained".into(),
        tool: "bash".into(),
        code: "echo".into(),
        inputs: vec![],
        shortcut: None,
        description: Some("A detailed description".into()),
    });
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let custom = actions
        .iter()
        .find(|a| a.id.starts_with("scriptlet_action:"))
        .unwrap();
    assert_eq!(
        custom.description.as_deref(),
        Some("A detailed description")
    );
}

// =========================================================================
// 27. All actions have ActionCategory::ScriptContext
// =========================================================================

#[test]
fn all_script_actions_are_script_context_category() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    for action in &get_script_context_actions(&script) {
        assert!(
            matches!(action.category, ActionCategory::ScriptContext),
            "Action '{}' should be ScriptContext category",
            action.id
        );
    }
}

#[test]
fn all_clipboard_actions_are_script_context_category() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    for action in &get_clipboard_history_context_actions(&entry) {
        assert!(
            matches!(action.category, ActionCategory::ScriptContext),
            "Clipboard action '{}' should be ScriptContext category",
            action.id
        );
    }
}

#[test]
fn all_file_actions_are_script_context_category() {
    let file = FileInfo {
        path: "/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    for action in &get_file_context_actions(&file) {
        assert!(
            matches!(action.category, ActionCategory::ScriptContext),
            "File action '{}' should be ScriptContext category",
            action.id
        );
    }
}

#[test]
fn all_ai_actions_are_script_context_category() {
    for action in &get_ai_command_bar_actions() {
        assert!(
            matches!(action.category, ActionCategory::ScriptContext),
            "AI action '{}' should be ScriptContext category",
            action.id
        );
    }
}

// =========================================================================
// 28. File context primary action title includes filename
// =========================================================================

#[test]
fn file_primary_action_title_includes_filename() {
    let file = FileInfo {
        path: "/docs/readme.md".into(),
        name: "readme.md".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    assert!(
        actions[0].title.contains("readme.md"),
        "Primary action title should include filename, got '{}'",
        actions[0].title
    );
}

#[test]
fn file_directory_primary_title_includes_dirname() {
    let dir = FileInfo {
        path: "/projects/my-app".into(),
        name: "my-app".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&dir);
    assert!(
        actions[0].title.contains("my-app"),
        "Directory primary action title should include dirname, got '{}'",
        actions[0].title
    );
}

// =========================================================================
// 29. Frecency reset ranking conditional
// =========================================================================

#[test]
fn frecency_not_suggested_lacks_reset_ranking() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions_tmp = get_script_context_actions(&script);
    let ids = action_ids(&actions_tmp);
    assert!(!ids.contains(&"reset_ranking"));
}

#[test]
fn frecency_suggested_has_reset_ranking() {
    let script = ScriptInfo::new("test", "/path/test.ts").with_frecency(true, Some("/path".into()));
    let actions_tmp = get_script_context_actions(&script);
    let ids = action_ids(&actions_tmp);
    assert!(ids.contains(&"reset_ranking"));
}

#[test]
fn frecency_suggested_reset_ranking_is_last() {
    let script = ScriptInfo::new("test", "/path/test.ts").with_frecency(true, Some("/path".into()));
    let actions = get_script_context_actions(&script);
    let last = actions.last().unwrap();
    assert_eq!(
        last.id, "reset_ranking",
        "reset_ranking should be the last action"
    );
}

// =========================================================================
// 30. All actions have descriptions (broad check)
// =========================================================================

#[test]
fn all_script_context_actions_have_descriptions() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    for action in &get_script_context_actions(&script) {
        assert!(
            action.description.is_some(),
            "Script action '{}' should have a description",
            action.id
        );
    }
}

#[test]
fn all_ai_command_bar_actions_have_descriptions() {
    for action in &get_ai_command_bar_actions() {
        assert!(
            action.description.is_some(),
            "AI action '{}' should have a description",
            action.id
        );
    }
}

#[test]
fn all_path_actions_have_descriptions() {
    let path = PathInfo::new("test", "/test", false);
    for action in &get_path_context_actions(&path) {
        assert!(
            action.description.is_some(),
            "Path action '{}' should have a description",
            action.id
        );
    }
}

#[test]
fn all_file_actions_have_descriptions() {
    let file = FileInfo {
        path: "/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    for action in &get_file_context_actions(&file) {
        assert!(
            action.description.is_some(),
            "File action '{}' should have a description",
            action.id
        );
    }
}

// =========================================================================
// 31. Clipboard has_action=false and no value for all entries
// =========================================================================

#[test]
fn clipboard_all_actions_have_no_value() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    for action in &get_clipboard_history_context_actions(&entry) {
        assert!(
            action.value.is_none(),
            "Clipboard action '{}' should have no value",
            action.id
        );
    }
}
