
#[test]
fn scriptlet_defined_id_uses_action_id() {
    let mut scriptlet = Scriptlet::new(
        "test".to_string(),
        "bash".to_string(),
        "echo hi".to_string(),
    );
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".to_string(),
        command: "pbcopy".to_string(),
        tool: "bash".to_string(),
        code: "echo hi | pbcopy".to_string(),
        description: None,
        shortcut: None,
        inputs: vec![],
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert!(actions[0].id.contains("pbcopy") || actions[0].id.starts_with("scriptlet_action:"));
}

// =========== 20. AI bar: all 12 have descriptions ===========

#[test]
fn ai_bar_all_have_descriptions() {
    let actions = get_ai_command_bar_actions();
    assert!(actions.iter().all(|a| a.description.is_some()));
}

#[test]
fn ai_bar_all_descriptions_non_empty() {
    let actions = get_ai_command_bar_actions();
    assert!(actions
        .iter()
        .all(|a| !a.description.as_ref().unwrap().is_empty()));
}

#[test]
fn ai_bar_count_is_12() {
    let actions = get_ai_command_bar_actions();
    assert_eq!(actions.len(), 12);
}

#[test]
fn ai_bar_all_have_icons() {
    let actions = get_ai_command_bar_actions();
    assert!(actions.iter().all(|a| a.icon.is_some()));
}

// =========== 21. AI bar: Response section has 3 actions ===========

#[test]
fn ai_bar_response_section_count() {
    let actions = get_ai_command_bar_actions();
    let response_count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .count();
    assert_eq!(response_count, 3);
}

#[test]
fn ai_bar_response_has_copy_response() {
    let actions = get_ai_command_bar_actions();
    let response: Vec<_> = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .collect();
    assert!(response.iter().any(|a| a.id == "copy_response"));
}

#[test]
fn ai_bar_response_has_copy_chat() {
    let actions = get_ai_command_bar_actions();
    let response: Vec<_> = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .collect();
    assert!(response.iter().any(|a| a.id == "copy_chat"));
}

#[test]
fn ai_bar_response_has_copy_last_code() {
    let actions = get_ai_command_bar_actions();
    let response: Vec<_> = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .collect();
    assert!(response.iter().any(|a| a.id == "copy_last_code"));
}

// =========== 22. Notes: untested boolean combos ===========

#[test]
fn notes_no_selection_trash_no_auto_count() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert_eq!(actions.len(), 3);
}

#[test]
fn notes_no_selection_trash_auto_count() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert_eq!(actions.len(), 2);
}

#[test]
fn notes_selection_trash_auto_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert_eq!(actions.len(), 2);
}

#[test]
fn notes_no_selection_no_trash_auto_count() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert_eq!(actions.len(), 2);
}

// =========== 23. Notes: trash+selection suppresses selection-dependent ===========

#[test]
fn notes_trash_selection_no_duplicate() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
}

#[test]
fn notes_trash_selection_no_find() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "find_in_note"));
}

#[test]
fn notes_trash_selection_no_format() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "format"));
}

#[test]
fn notes_trash_selection_no_export() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "export"));
}

// =========== 24. Chat: 2 models + response + messages = 5 actions ===========

#[test]
fn chat_2_models_response_messages_count() {
    let info = ChatPromptInfo {
        current_model: Some("Claude".into()),
        available_models: vec![
            ChatModelInfo {
                id: "claude".into(),
                display_name: "Claude".into(),
                provider: "Anthropic".into(),
            },
            ChatModelInfo {
                id: "gpt4".into(),
                display_name: "GPT-4".into(),
                provider: "OpenAI".into(),
            },
        ],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions.len(), 5);
}

#[test]
fn chat_2_models_response_messages_has_continue() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![
            ChatModelInfo {
                id: "a".into(),
                display_name: "A".into(),
                provider: "P".into(),
            },
            ChatModelInfo {
                id: "b".into(),
                display_name: "B".into(),
                provider: "Q".into(),
            },
        ],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "continue_in_chat"));
}

#[test]
fn chat_2_models_response_messages_has_copy_response() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![
            ChatModelInfo {
                id: "a".into(),
                display_name: "A".into(),
                provider: "P".into(),
            },
            ChatModelInfo {
                id: "b".into(),
                display_name: "B".into(),
                provider: "Q".into(),
            },
        ],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "copy_response"));
}

#[test]
fn chat_2_models_response_messages_has_clear() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![
            ChatModelInfo {
                id: "a".into(),
                display_name: "A".into(),
                provider: "P".into(),
            },
            ChatModelInfo {
                id: "b".into(),
                display_name: "B".into(),
                provider: "Q".into(),
            },
        ],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "clear_conversation"));
}

// =========== 25. Chat: models before continue_in_chat in ordering ===========

#[test]
fn chat_model_at_index_0() {
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
    assert!(actions[0].id.starts_with("select_model_"));
}

#[test]
fn chat_continue_after_models() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![
            ChatModelInfo {
                id: "a".into(),
                display_name: "A".into(),
                provider: "P".into(),
            },
            ChatModelInfo {
                id: "b".into(),
                display_name: "B".into(),
                provider: "Q".into(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions[2].id, "continue_in_chat");
}

#[test]
fn chat_models_preserve_insertion_order() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![
            ChatModelInfo {
                id: "first".into(),
                display_name: "First".into(),
                provider: "P".into(),
            },
            ChatModelInfo {
                id: "second".into(),
                display_name: "Second".into(),
                provider: "Q".into(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions[0].id, "select_model_first");
    assert_eq!(actions[1].id, "select_model_second");
}

#[test]
fn chat_single_model_continue_at_index_1() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "only".into(),
            display_name: "Only".into(),
            provider: "P".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions[1].id, "continue_in_chat");
}

// =========== 26. New chat: section assignment per type ===========

#[test]
fn new_chat_last_used_section() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "p".into(),
        provider_display_name: "Provider".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
}

#[test]
fn new_chat_preset_section() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].section.as_deref(), Some("Presets"));
}

#[test]
fn new_chat_model_section() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].section.as_deref(), Some("Models"));
}

#[test]
fn new_chat_all_three_sections_present() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "M1".into(),
        provider: "p".into(),
        provider_display_name: "P".into(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "gen".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &presets, &models);
    let sections: Vec<_> = actions
        .iter()
        .filter_map(|a| a.section.as_deref())
        .collect();
    assert!(sections.contains(&"Last Used Settings"));
    assert!(sections.contains(&"Presets"));
    assert!(sections.contains(&"Models"));
}

// =========== 27. Clipboard: text has no image-specific actions ===========

#[test]
fn clipboard_text_no_ocr() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_ocr"));
}
