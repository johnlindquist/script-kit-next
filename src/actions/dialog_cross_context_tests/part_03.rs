
#[test]
fn chat_context_all_four_conditional_combos() {
    for (has_response, has_messages) in [(false, false), (true, false), (false, true), (true, true)]
    {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages,
            has_response,
        };
        let actions = get_chat_context_actions(&info);
        let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

        // continue_in_chat always present
        assert!(ids.contains(&"continue_in_chat"));

        // copy_response only when has_response
        assert_eq!(
            ids.contains(&"copy_response"),
            has_response,
            "copy_response presence should match has_response={}",
            has_response
        );

        // clear_conversation only when has_messages
        assert_eq!(
            ids.contains(&"clear_conversation"),
            has_messages,
            "clear_conversation presence should match has_messages={}",
            has_messages
        );
    }
}

// ============================================================================
// New chat actions
// ============================================================================

#[test]
fn new_chat_empty_inputs_returns_empty() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

#[test]
fn new_chat_section_names_are_correct() {
    let last = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "P".into(),
        provider_display_name: "Provider".into(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "p1".into(),
        name: "General".into(),
        icon: crate::designs::icon_variations::IconName::BoltFilled,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "m2".into(),
        display_name: "Model 2".into(),
        provider: "P".into(),
        provider_display_name: "Provider 2".into(),
    }];

    let actions = get_new_chat_actions(&last, &presets, &models);
    assert_eq!(actions.len(), 3);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
    assert_eq!(actions[1].section.as_deref(), Some("Presets"));
    assert_eq!(actions[2].section.as_deref(), Some("Models"));
}

#[test]
fn new_chat_preset_uses_custom_icon() {
    let presets = vec![NewChatPresetInfo {
        id: "code".into(),
        name: "Code".into(),
        icon: crate::designs::icon_variations::IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(
        actions[0].icon,
        Some(crate::designs::icon_variations::IconName::Code)
    );
}

// ============================================================================
// Scriptlet custom actions ordering and fields
// ============================================================================

#[test]
fn scriptlet_custom_actions_have_correct_value_field() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo main".into());
    scriptlet.actions = vec![ScriptletAction {
        name: "My Action".into(),
        command: "my-action-cmd".into(),
        tool: "bash".into(),
        code: "echo custom".into(),
        inputs: vec![],
        shortcut: None,
        description: Some("Does something".into()),
    }];

    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let custom = actions
        .iter()
        .find(|a| a.id == "scriptlet_action:my-action-cmd")
        .unwrap();

    assert!(custom.has_action);
    assert_eq!(custom.value.as_deref(), Some("my-action-cmd"));
    assert_eq!(custom.title, "My Action");
    assert_eq!(custom.description.as_deref(), Some("Does something"));
}

#[test]
fn scriptlet_multiple_custom_actions_preserve_order() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo main".into());
    scriptlet.actions = vec![
        ScriptletAction {
            name: "Alpha".into(),
            command: "alpha".into(),
            tool: "bash".into(),
            code: "echo a".into(),
            inputs: vec![],
            shortcut: Some("cmd+1".into()),
            description: None,
        },
        ScriptletAction {
            name: "Beta".into(),
            command: "beta".into(),
            tool: "bash".into(),
            code: "echo b".into(),
            inputs: vec![],
            shortcut: Some("cmd+2".into()),
            description: None,
        },
        ScriptletAction {
            name: "Gamma".into(),
            command: "gamma".into(),
            tool: "bash".into(),
            code: "echo g".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
    ];

    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));

    // run_script at 0, then custom actions in order, then built-in
    assert_eq!(actions[0].id, "run_script");
    assert_eq!(actions[1].id, "scriptlet_action:alpha");
    assert_eq!(actions[2].id, "scriptlet_action:beta");
    assert_eq!(actions[3].id, "scriptlet_action:gamma");

    // Alpha has formatted shortcut
    assert_eq!(actions[1].shortcut.as_deref(), Some("⌘1"));
    // Beta has formatted shortcut
    assert_eq!(actions[2].shortcut.as_deref(), Some("⌘2"));
    // Gamma has no shortcut
    assert!(actions[3].shortcut.is_none());
}

// ============================================================================
// CommandBarConfig field interactions
// ============================================================================

#[test]
fn command_bar_config_notes_style_specific_fields() {
    let config = CommandBarConfig::notes_style();
    // Notes style uses Top search, Separators, Top anchor, icons+footer
    assert!(matches!(
        config.dialog_config.search_position,
        SearchPosition::Top
    ));
    assert!(matches!(
        config.dialog_config.section_style,
        SectionStyle::Separators
    ));
    assert!(matches!(config.dialog_config.anchor, AnchorPosition::Top));
    assert!(config.dialog_config.show_icons);
    assert!(config.dialog_config.show_footer);
    // Close behaviors default to true
    assert!(config.close_on_select);
    assert!(config.close_on_escape);
    assert!(config.close_on_click_outside);
}

#[test]
fn command_bar_config_all_presets_have_close_defaults() {
    for config in [
        CommandBarConfig::default(),
        CommandBarConfig::main_menu_style(),
        CommandBarConfig::ai_style(),
        CommandBarConfig::no_search(),
        CommandBarConfig::notes_style(),
    ] {
        assert!(
            config.close_on_select,
            "close_on_select should default to true"
        );
        assert!(
            config.close_on_escape,
            "close_on_escape should default to true"
        );
        assert!(
            config.close_on_click_outside,
            "close_on_click_outside should default to true"
        );
    }
}

// ============================================================================
// Grouped items and coercion edge cases
// ============================================================================

#[test]
fn grouped_items_with_headers_counts_match_count_section_headers() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();

    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    let header_count_from_grouped = grouped
        .iter()
        .filter(|item| matches!(item, GroupedActionItem::SectionHeader(_)))
        .count();
    let header_count_from_window = count_section_headers(&actions, &filtered);

    assert_eq!(header_count_from_grouped, header_count_from_window);
}

#[test]
fn coerce_navigation_skips_all_headers_to_reach_items() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".into()),
        GroupedActionItem::SectionHeader("B".into()),
        GroupedActionItem::SectionHeader("C".into()),
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("D".into()),
        GroupedActionItem::Item(1),
    ];

    // Starting at header 0 should find Item at index 3
    assert_eq!(coerce_action_selection(&rows, 0), Some(3));
    // Starting at header 1 should find Item at index 3
    assert_eq!(coerce_action_selection(&rows, 1), Some(3));
    // Starting at header 4 should find Item at index 5
    assert_eq!(coerce_action_selection(&rows, 4), Some(5));
    // Item at 3 stays at 3
    assert_eq!(coerce_action_selection(&rows, 3), Some(3));
}

#[test]
fn coerce_last_item_is_header_searches_backwards() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::Item(1),
        GroupedActionItem::SectionHeader("End".into()),
    ];

    // At header index 2, should search backwards to item at index 1
    assert_eq!(coerce_action_selection(&rows, 2), Some(1));
}

// ============================================================================
// WindowPosition default and variants
// ============================================================================

#[test]
fn window_position_default_is_bottom_right() {
    let pos = WindowPosition::default();
    assert_eq!(pos, WindowPosition::BottomRight);
}

#[test]
fn window_position_all_variants_distinct() {
    assert_ne!(WindowPosition::BottomRight, WindowPosition::TopRight);
    assert_ne!(WindowPosition::TopRight, WindowPosition::TopCenter);
    assert_ne!(WindowPosition::BottomRight, WindowPosition::TopCenter);
}

// ============================================================================
// ProtocolAction constructor variants
// ============================================================================

#[test]
fn protocol_action_with_handler_has_action_true() {
    let action = ProtocolAction::with_handler("My Handler".into());
    assert!(action.has_action);
    assert!(action.value.is_none());
    assert!(action.description.is_none());
    assert!(action.is_visible());
    assert!(action.should_close());
}

#[test]
fn protocol_action_with_value_has_action_false() {
    let action = ProtocolAction::with_value("Submit".into(), "submit-val".into());
    assert!(!action.has_action);
    assert_eq!(action.value.as_deref(), Some("submit-val"));
    assert!(action.is_visible());
    assert!(action.should_close());
}

#[test]
fn protocol_action_hidden_not_visible() {
    let action = ProtocolAction {
        name: "Hidden".into(),
        description: None,
        shortcut: None,
        value: None,
        has_action: false,
        visible: Some(false),
        close: None,
    };
    assert!(!action.is_visible());
}

#[test]
fn protocol_action_close_false_stays_open() {
    let action = ProtocolAction {
        name: "Stay Open".into(),
        description: None,
        shortcut: None,
        value: None,
        has_action: true,
        visible: None,
        close: Some(false),
    };
    assert!(!action.should_close());
    assert!(action.is_visible()); // visible defaults to true
}

// ============================================================================
// Action struct caching behavior
// ============================================================================

#[test]
fn action_title_lower_cache_matches_lowercase() {
    let action = Action::new(
        "test",
        "Edit Script",
        Some("Open in $EDITOR".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(action.title_lower, "edit script");
    assert_eq!(action.description_lower.as_deref(), Some("open in $editor"));
}

#[test]
fn action_shortcut_lower_cache_set_by_with_shortcut() {
    let action =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
    assert_eq!(action.shortcut_lower.as_deref(), Some("⌘⇧c"));
}

#[test]
fn action_shortcut_lower_cache_not_set_without_shortcut() {
    let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
    assert!(action.shortcut_lower.is_none());
}

#[test]
fn action_with_icon_and_section_chain() {
    let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
        .with_icon(crate::designs::icon_variations::IconName::Plus)
        .with_section("MySection")
        .with_shortcut("⌘T");

    assert_eq!(
        action.icon,
        Some(crate::designs::icon_variations::IconName::Plus)
    );
    assert_eq!(action.section.as_deref(), Some("MySection"));
    assert_eq!(action.shortcut.as_deref(), Some("⌘T"));
}

// ============================================================================
// Exact action count for script type permutations
// ============================================================================

#[test]
fn script_action_count_without_shortcut_or_alias() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    // run + add_shortcut + add_alias + edit + view_logs + reveal + copy_path + copy_content + copy_deeplink = 9
    assert_eq!(actions.len(), 9);
}

#[test]
fn script_action_count_with_shortcut_and_alias() {
    let script = ScriptInfo::with_shortcut_and_alias(
        "test",
        "/path/test.ts",
        Some("cmd+t".into()),
        Some("ts".into()),
    );
    let actions = get_script_context_actions(&script);
    // run + update_shortcut + remove_shortcut + update_alias + remove_alias
    // + edit + view_logs + reveal + copy_path + copy_content + copy_deeplink = 11
    assert_eq!(actions.len(), 11);
}

#[test]
fn builtin_action_count() {
    let builtin = ScriptInfo::builtin("Test Builtin");
    let actions = get_script_context_actions(&builtin);
    // run + add_shortcut + add_alias + copy_deeplink = 4
    assert_eq!(actions.len(), 4);
}

#[test]
fn scriptlet_action_count_without_shortcut_or_alias() {
    let scriptlet = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_script_context_actions(&scriptlet);
    // run + add_shortcut + add_alias + edit_scriptlet + reveal_scriptlet + copy_scriptlet_path + copy_content + copy_deeplink = 8
    assert_eq!(actions.len(), 8);
}

#[test]
fn path_context_file_action_count() {
    let info = PathInfo::new("file.txt", "/test/file.txt", false);
    let actions = get_path_context_actions(&info);
    // select_file + copy_path + open_in_finder + open_in_editor + open_in_terminal + copy_filename + move_to_trash = 7
    assert_eq!(actions.len(), 7);
}

#[test]
fn path_context_dir_action_count() {
    let info = PathInfo::new("dir", "/test/dir", true);
    let actions = get_path_context_actions(&info);
    // open_directory + copy_path + open_in_finder + open_in_editor + open_in_terminal + copy_filename + move_to_trash = 7
    assert_eq!(actions.len(), 7);
}

// ============================================================================
// Deeplink name edge cases
// ============================================================================

#[test]
fn deeplink_name_with_unicode() {
    // Unicode accented chars are alphanumeric per Rust's is_alphanumeric()
    assert_eq!(to_deeplink_name("café"), "café");
}
