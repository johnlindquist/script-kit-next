
#[test]
fn scriptlet_custom_action_without_shortcut_is_none() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut s = Scriptlet::new("Test".into(), "bash".into(), "echo".into());
    s.actions = vec![ScriptletAction {
        name: "NoKey".into(),
        command: "nokey".into(),
        tool: "bash".into(),
        code: "echo".into(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&s));
    let custom = actions
        .iter()
        .find(|a| a.id == "scriptlet_action:nokey")
        .unwrap();
    assert!(custom.shortcut.is_none());
}

#[test]
fn scriptlet_custom_action_description_propagated() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut s = Scriptlet::new("Test".into(), "bash".into(), "echo".into());
    s.actions = vec![ScriptletAction {
        name: "Desc Action".into(),
        command: "desc-act".into(),
        tool: "bash".into(),
        code: "echo".into(),
        inputs: vec![],
        shortcut: None,
        description: Some("My description here".into()),
    }];
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&s));
    let custom = actions
        .iter()
        .find(|a| a.id == "scriptlet_action:desc-act")
        .unwrap();
    assert_eq!(custom.description.as_ref().unwrap(), "My description here");
}

#[test]
fn scriptlet_custom_action_title_is_name() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut s = Scriptlet::new("Test".into(), "bash".into(), "echo".into());
    s.actions = vec![ScriptletAction {
        name: "My Title".into(),
        command: "mt".into(),
        tool: "bash".into(),
        code: "echo".into(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&s));
    let custom = actions
        .iter()
        .find(|a| a.id == "scriptlet_action:mt")
        .unwrap();
    assert_eq!(custom.title, "My Title");
}

// =====================================================================
// 8. AI command bar: copy_chat and copy_last_code details
// =====================================================================

#[test]
fn ai_bar_copy_chat_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "copy_chat").unwrap();
    assert_eq!(a.shortcut.as_ref().unwrap(), "⌥⇧⌘C");
}

#[test]
fn ai_bar_copy_chat_icon_copy() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "copy_chat").unwrap();
    assert_eq!(a.icon, Some(IconName::Copy));
}

#[test]
fn ai_bar_copy_chat_section_response() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "copy_chat").unwrap();
    assert_eq!(a.section.as_ref().unwrap(), "Response");
}

#[test]
fn ai_bar_copy_last_code_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "copy_last_code").unwrap();
    assert_eq!(a.shortcut.as_ref().unwrap(), "⌥⌘C");
}

#[test]
fn ai_bar_copy_last_code_icon_code() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "copy_last_code").unwrap();
    assert_eq!(a.icon, Some(IconName::Code));
}

#[test]
fn ai_bar_copy_last_code_section_response() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "copy_last_code").unwrap();
    assert_eq!(a.section.as_ref().unwrap(), "Response");
}

// =====================================================================
// 9. AI command bar: all IDs are unique
// =====================================================================

#[test]
fn ai_bar_all_ids_unique() {
    let actions = get_ai_command_bar_actions();
    let mut ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let original_len = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), original_len);
}

#[test]
fn ai_bar_all_have_icon() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(
            a.icon.is_some(),
            "AI bar action {} should have an icon",
            a.id
        );
    }
}

#[test]
fn ai_bar_all_have_section() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(
            a.section.is_some(),
            "AI bar action {} should have a section",
            a.id
        );
    }
}

#[test]
fn ai_bar_count_is_12() {
    let actions = get_ai_command_bar_actions();
    assert_eq!(actions.len(), 12);
}

// =====================================================================
// 10. Chat context: select_model ID format
// =====================================================================

#[test]
fn chat_model_id_format() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "gpt-4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "select_model_gpt-4"));
}

#[test]
fn chat_model_current_check_by_display_name() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".into()),
        available_models: vec![ChatModelInfo {
            id: "gpt-4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model = actions
        .iter()
        .find(|a| a.id == "select_model_gpt-4")
        .unwrap();
    assert!(model.title.contains("✓"));
}

#[test]
fn chat_model_non_current_no_check() {
    let info = ChatPromptInfo {
        current_model: Some("Claude 3.5".into()),
        available_models: vec![ChatModelInfo {
            id: "gpt-4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model = actions
        .iter()
        .find(|a| a.id == "select_model_gpt-4")
        .unwrap();
    assert!(!model.title.contains("✓"));
}

#[test]
fn chat_model_desc_via_provider() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "claude".into(),
            display_name: "Claude".into(),
            provider: "Anthropic".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model = actions
        .iter()
        .find(|a| a.id == "select_model_claude")
        .unwrap();
    assert_eq!(model.description.as_ref().unwrap(), "via Anthropic");
}

// =====================================================================
// 11. Notes command bar: format action details
// =====================================================================

#[test]
fn notes_format_shortcut_shift_cmd_t() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let a = actions.iter().find(|a| a.id == "format").unwrap();
    assert_eq!(a.shortcut.as_ref().unwrap(), "⇧⌘T");
}

#[test]
fn notes_format_icon_code() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let a = actions.iter().find(|a| a.id == "format").unwrap();
    assert_eq!(a.icon, Some(IconName::Code));
}

#[test]
fn notes_format_section_edit() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let a = actions.iter().find(|a| a.id == "format").unwrap();
    assert_eq!(a.section.as_ref().unwrap(), "Edit");
}

#[test]
fn notes_format_absent_in_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "format"));
}

// =====================================================================
// 12. Notes command bar: trash view exact action set
// =====================================================================

#[test]
fn notes_trash_has_exactly_3_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert_eq!(actions.len(), 3);
}

#[test]
fn notes_trash_has_new_note() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "new_note"));
}

#[test]
fn notes_trash_has_browse_notes() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "browse_notes"));
}

#[test]
fn notes_trash_has_enable_auto_sizing_when_disabled() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "enable_auto_sizing"));
}

// =====================================================================
// 13. Note switcher: empty notes produces no_notes action
// =====================================================================

#[test]
fn note_switcher_empty_has_no_notes() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "no_notes");
}

#[test]
fn note_switcher_no_notes_title() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions[0].title, "No notes yet");
}

#[test]
fn note_switcher_no_notes_desc_mentions_cmd_n() {
    let actions = get_note_switcher_actions(&[]);
    assert!(actions[0].description.as_ref().unwrap().contains("⌘N"));
}

#[test]
fn note_switcher_no_notes_icon_plus() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions[0].icon, Some(IconName::Plus));
}

// =====================================================================
// 14. Note switcher: ID format is note_{uuid}
// =====================================================================

#[test]
fn note_switcher_id_format() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc-123-def".into(),
        title: "My Note".into(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].id, "note_abc-123-def");
}

#[test]
fn note_switcher_current_icon_check() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n1".into(),
        title: "Current".into(),
        char_count: 10,
        is_current: true,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::Check));
}

#[test]
fn note_switcher_regular_icon_file() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n2".into(),
        title: "Regular".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::File));
}

#[test]
fn note_switcher_pinned_trumps_current() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n3".into(),
        title: "Both".into(),
        char_count: 10,
        is_current: true,
        is_pinned: true,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

// =====================================================================
// 15. New chat: empty inputs produce expected results
// =====================================================================

#[test]
fn new_chat_all_empty() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

#[test]
fn new_chat_only_models() {
    let models = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "p".into(),
        provider_display_name: "Provider".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "model_0");
}
