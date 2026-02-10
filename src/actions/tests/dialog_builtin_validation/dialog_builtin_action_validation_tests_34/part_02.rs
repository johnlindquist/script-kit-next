
// =====================================================================
// 9. Notes command bar: export details
// =====================================================================

#[test]
fn notes_export_shortcut_shift_cmd_e() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let exp = actions.iter().find(|a| a.id == "export").unwrap();
    assert_eq!(exp.shortcut.as_deref(), Some("⇧⌘E"));
}

#[test]
fn notes_export_icon_arrow_right() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let exp = actions.iter().find(|a| a.id == "export").unwrap();
    assert_eq!(exp.icon, Some(IconName::ArrowRight));
}

#[test]
fn notes_export_section_export() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let exp = actions.iter().find(|a| a.id == "export").unwrap();
    assert_eq!(exp.section.as_deref(), Some("Export"));
}

#[test]
fn notes_export_absent_without_selection() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "export"));
}

// =====================================================================
// 10. Chat context: all 4 flag combinations (has_messages x has_response)
// =====================================================================

#[test]
fn chat_no_messages_no_response_has_only_models_and_continue() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "m1".into(),
            display_name: "Model1".into(),
            provider: "P1".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    // 1 model + continue_in_chat = 2
    assert_eq!(actions.len(), 2);
    assert!(!actions.iter().any(|a| a.id == "copy_response"));
    assert!(!actions.iter().any(|a| a.id == "clear_conversation"));
}

#[test]
fn chat_has_messages_no_response_has_clear_no_copy() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "m1".into(),
            display_name: "Model1".into(),
            provider: "P1".into(),
        }],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "clear_conversation"));
    assert!(!actions.iter().any(|a| a.id == "copy_response"));
}

#[test]
fn chat_no_messages_has_response_has_copy_no_clear() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "m1".into(),
            display_name: "Model1".into(),
            provider: "P1".into(),
        }],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "copy_response"));
    assert!(!actions.iter().any(|a| a.id == "clear_conversation"));
}

#[test]
fn chat_has_both_flags_has_copy_and_clear() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "m1".into(),
            display_name: "Model1".into(),
            provider: "P1".into(),
        }],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "copy_response"));
    assert!(actions.iter().any(|a| a.id == "clear_conversation"));
    // 1 model + continue + copy + clear = 4
    assert_eq!(actions.len(), 4);
}

// =====================================================================
// 11. Chat context: continue_in_chat always present regardless of flags
// =====================================================================

#[test]
fn chat_continue_in_chat_always_present() {
    for (has_messages, has_response) in [(false, false), (true, false), (false, true), (true, true)]
    {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages,
            has_response,
        };
        let actions = get_chat_context_actions(&info);
        assert!(
            actions.iter().any(|a| a.id == "continue_in_chat"),
            "continue_in_chat missing for has_messages={has_messages}, has_response={has_response}"
        );
    }
}

#[test]
fn chat_continue_in_chat_shortcut_cmd_enter() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let cont = actions.iter().find(|a| a.id == "continue_in_chat").unwrap();
    assert_eq!(cont.shortcut.as_deref(), Some("⌘↵"));
}

#[test]
fn chat_continue_in_chat_desc_mentions_ai_chat() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let cont = actions.iter().find(|a| a.id == "continue_in_chat").unwrap();
    assert!(cont.description.as_ref().unwrap().contains("AI Chat"));
}

#[test]
fn chat_clear_conversation_shortcut_cmd_delete() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let clear = actions
        .iter()
        .find(|a| a.id == "clear_conversation")
        .unwrap();
    assert_eq!(clear.shortcut.as_deref(), Some("⌘⌫"));
}

// =====================================================================
// 12. Chat context: copy_response shortcut is ⌘C
// =====================================================================

#[test]
fn chat_copy_response_shortcut_cmd_c() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let cr = actions.iter().find(|a| a.id == "copy_response").unwrap();
    assert_eq!(cr.shortcut.as_deref(), Some("⌘C"));
}

#[test]
fn chat_copy_response_title() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let cr = actions.iter().find(|a| a.id == "copy_response").unwrap();
    assert_eq!(cr.title, "Copy Last Response");
}

#[test]
fn chat_copy_response_desc_mentions_assistant() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let cr = actions.iter().find(|a| a.id == "copy_response").unwrap();
    assert!(cr.description.as_ref().unwrap().contains("assistant"));
}

// =====================================================================
// 13. Script context: scriptlet with shortcut gets update/remove not add
// =====================================================================

#[test]
fn scriptlet_with_shortcut_has_update_shortcut() {
    let script = ScriptInfo::scriptlet(
        "My Scriptlet",
        "/path/bundle.md",
        Some("cmd+s".into()),
        None,
    );
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "update_shortcut"));
    assert!(!actions.iter().any(|a| a.id == "add_shortcut"));
}

#[test]
fn scriptlet_with_shortcut_has_remove_shortcut() {
    let script = ScriptInfo::scriptlet(
        "My Scriptlet",
        "/path/bundle.md",
        Some("cmd+s".into()),
        None,
    );
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
}

#[test]
fn scriptlet_without_shortcut_has_add_shortcut() {
    let script = ScriptInfo::scriptlet("My Scriptlet", "/path/bundle.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "add_shortcut"));
    assert!(!actions.iter().any(|a| a.id == "update_shortcut"));
}

#[test]
fn scriptlet_with_alias_has_update_alias() {
    let script = ScriptInfo::scriptlet("My Scriptlet", "/path/bundle.md", None, Some("ms".into()));
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "update_alias"));
    assert!(actions.iter().any(|a| a.id == "remove_alias"));
    assert!(!actions.iter().any(|a| a.id == "add_alias"));
}

// =====================================================================
// 14. Scriptlet context: copy_content desc mentions entire file
// =====================================================================

#[test]
fn scriptlet_copy_content_desc_mentions_entire_file() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert!(cc.description.as_ref().unwrap().contains("entire file"));
}

#[test]
fn scriptlet_copy_content_shortcut_opt_cmd_c() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(cc.shortcut.as_deref(), Some("⌘⌥C"));
}

#[test]
fn scriptlet_edit_scriptlet_shortcut_cmd_e() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let edit = actions.iter().find(|a| a.id == "edit_scriptlet").unwrap();
    assert_eq!(edit.shortcut.as_deref(), Some("⌘E"));
}

#[test]
fn scriptlet_edit_scriptlet_desc_mentions_editor() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let edit = actions.iter().find(|a| a.id == "edit_scriptlet").unwrap();
    assert!(edit.description.as_ref().unwrap().contains("$EDITOR"));
}

// =====================================================================
// 15. builders::format_shortcut_hint vs ActionsDialog::format_shortcut_hint
// =====================================================================

#[test]
fn builders_format_basic_cmd_c() {
    // builders::format_shortcut_hint is private, but we test via to_deeplink_name
    // and scriptlet-defined actions. The ActionsDialog version handles more aliases.
    let result = ActionsDialog::format_shortcut_hint("cmd+c");
    assert_eq!(result, "⌘C");
}

#[test]
fn dialog_format_handles_control_alias() {
    let result = ActionsDialog::format_shortcut_hint("control+x");
    assert_eq!(result, "⌃X");
}

#[test]
fn dialog_format_handles_option_alias() {
    let result = ActionsDialog::format_shortcut_hint("option+v");
    assert_eq!(result, "⌥V");
}

#[test]
fn dialog_format_handles_backspace_key() {
    let result = ActionsDialog::format_shortcut_hint("cmd+backspace");
    assert_eq!(result, "⌘⌫");
}

// =====================================================================
// 16. to_deeplink_name: various transformations
// =====================================================================

#[test]
fn deeplink_underscores_become_hyphens() {
    assert_eq!(to_deeplink_name("hello_world"), "hello-world");
}

#[test]
fn deeplink_multiple_underscores_collapse() {
    assert_eq!(to_deeplink_name("a___b"), "a-b");
}

#[test]
fn deeplink_mixed_punctuation() {
    assert_eq!(to_deeplink_name("Hello, World!"), "hello-world");
}

#[test]
fn deeplink_empty_string() {
    assert_eq!(to_deeplink_name(""), "");
}

// =====================================================================
// 17. Constants: UI dimensions validation
// =====================================================================

#[test]
fn constant_popup_width_320() {
    use crate::actions::constants::POPUP_WIDTH;
    assert_eq!(POPUP_WIDTH, 320.0);
}

#[test]
fn constant_popup_max_height_400() {
    use crate::actions::constants::POPUP_MAX_HEIGHT;
    assert_eq!(POPUP_MAX_HEIGHT, 400.0);
}

#[test]
fn constant_action_item_height_36() {
    use crate::actions::constants::ACTION_ITEM_HEIGHT;
    assert_eq!(ACTION_ITEM_HEIGHT, 36.0);
}

#[test]
fn constant_search_input_height_44() {
    use crate::actions::constants::SEARCH_INPUT_HEIGHT;
    assert_eq!(SEARCH_INPUT_HEIGHT, 44.0);
}

// =====================================================================
// 18. CommandBarConfig notes_style preset
// =====================================================================

#[test]
fn notes_style_search_position_top() {
    let config = CommandBarConfig::notes_style();
    assert!(matches!(
        config.dialog_config.search_position,
        SearchPosition::Top
    ));
}

#[test]
fn notes_style_section_style_separators() {
    let config = CommandBarConfig::notes_style();
    assert!(matches!(
        config.dialog_config.section_style,
        SectionStyle::Separators
    ));
}

#[test]
fn notes_style_show_icons_true() {
    let config = CommandBarConfig::notes_style();
    assert!(config.dialog_config.show_icons);
}

#[test]
fn notes_style_show_footer_true() {
    let config = CommandBarConfig::notes_style();
    assert!(config.dialog_config.show_footer);
}

// =====================================================================
// 19. Global actions: always empty
// =====================================================================

#[test]
fn global_actions_empty() {
    use crate::actions::builders::get_global_actions;
    let actions = get_global_actions();
    assert!(actions.is_empty());
}

#[test]
fn global_actions_returns_vec() {
    use crate::actions::builders::get_global_actions;
    let actions = get_global_actions();
    assert_eq!(actions.len(), 0);
}
