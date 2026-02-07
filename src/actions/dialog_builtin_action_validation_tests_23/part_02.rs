
// ============================================================
// 9. AI command bar: action shortcut presence/absence matrix
// ============================================================

#[test]
fn batch23_ai_copy_response_has_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "copy_response").unwrap();
    assert_eq!(a.shortcut.as_ref().unwrap(), "⇧⌘C");
}

#[test]
fn batch23_ai_copy_chat_has_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "copy_chat").unwrap();
    assert_eq!(a.shortcut.as_ref().unwrap(), "⌥⇧⌘C");
}

#[test]
fn batch23_ai_copy_last_code_has_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "copy_last_code").unwrap();
    assert_eq!(a.shortcut.as_ref().unwrap(), "⌥⌘C");
}

#[test]
fn batch23_ai_branch_from_last_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
    assert!(a.shortcut.is_none());
}

#[test]
fn batch23_ai_change_model_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "change_model").unwrap();
    assert!(a.shortcut.is_none());
}

// ============================================================
// 10. AI command bar: description content validation
// ============================================================

#[test]
fn batch23_ai_submit_description() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "submit").unwrap();
    assert!(a
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("send"));
}

#[test]
fn batch23_ai_new_chat_description() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "new_chat").unwrap();
    assert!(a
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("new"));
}

#[test]
fn batch23_ai_delete_chat_description() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "delete_chat").unwrap();
    assert!(a
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("delete"));
}

#[test]
fn batch23_ai_export_markdown_description() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "export_markdown").unwrap();
    assert!(a
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("markdown"));
}

#[test]
fn batch23_ai_paste_image_description() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert!(a
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("clipboard"));
}

// ============================================================
// 11. Chat context: model IDs use select_model_ prefix
// ============================================================

#[test]
fn batch23_chat_model_id_prefix() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "gpt-4".to_string(),
            display_name: "GPT-4".to_string(),
            provider: "OpenAI".to_string(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions[0].id.starts_with("select_model_"));
    assert_eq!(actions[0].id, "select_model_gpt-4");
}

#[test]
fn batch23_chat_multiple_models_sequential_ids() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![
            ChatModelInfo {
                id: "claude-3".to_string(),
                display_name: "Claude 3".to_string(),
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
    assert_eq!(actions[0].id, "select_model_claude-3");
    assert_eq!(actions[1].id, "select_model_gpt-4");
}

#[test]
fn batch23_chat_model_descriptions_via_provider() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "opus".to_string(),
            display_name: "Claude Opus".to_string(),
            provider: "Anthropic".to_string(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions[0].description.as_ref().unwrap(), "via Anthropic");
}

#[test]
fn batch23_chat_current_model_checkmark() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".to_string()),
        available_models: vec![
            ChatModelInfo {
                id: "gpt-4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            },
            ChatModelInfo {
                id: "claude-3".to_string(),
                display_name: "Claude 3".to_string(),
                provider: "Anthropic".to_string(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions[0].title.contains("✓"));
    assert!(!actions[1].title.contains("✓"));
}

// ============================================================
// 12. Chat context: continue_in_chat is always present
// ============================================================

#[test]
fn batch23_chat_continue_in_chat_always_present() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "continue_in_chat"));
}

#[test]
fn batch23_chat_continue_in_chat_shortcut() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let c = actions.iter().find(|a| a.id == "continue_in_chat").unwrap();
    assert_eq!(c.shortcut.as_ref().unwrap(), "⌘↵");
}

#[test]
fn batch23_chat_continue_after_models() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "m1".to_string(),
            display_name: "M1".to_string(),
            provider: "P".to_string(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model_idx = actions
        .iter()
        .position(|a| a.id.starts_with("select_model_"))
        .unwrap();
    let continue_idx = actions
        .iter()
        .position(|a| a.id == "continue_in_chat")
        .unwrap();
    assert!(continue_idx > model_idx);
}

// ============================================================
// 13. Notes command bar: section icon assignments
// ============================================================

#[test]
fn batch23_notes_new_note_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let note = actions.iter().find(|a| a.id == "new_note").unwrap();
    assert_eq!(note.icon, Some(IconName::Plus));
}

#[test]
fn batch23_notes_browse_notes_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let browse = actions.iter().find(|a| a.id == "browse_notes").unwrap();
    assert_eq!(browse.icon, Some(IconName::FolderOpen));
}

#[test]
fn batch23_notes_find_in_note_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let find = actions.iter().find(|a| a.id == "find_in_note").unwrap();
    assert_eq!(find.icon, Some(IconName::MagnifyingGlass));
}

#[test]
fn batch23_notes_format_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let fmt = actions.iter().find(|a| a.id == "format").unwrap();
    assert_eq!(fmt.icon, Some(IconName::Code));
}

#[test]
fn batch23_notes_enable_auto_sizing_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let auto = actions
        .iter()
        .find(|a| a.id == "enable_auto_sizing")
        .unwrap();
    assert_eq!(auto.icon, Some(IconName::Settings));
}

// ============================================================
// 14. Notes command bar: shortcut assignments
// ============================================================

#[test]
fn batch23_notes_new_note_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let note = actions.iter().find(|a| a.id == "new_note").unwrap();
    assert_eq!(note.shortcut.as_ref().unwrap(), "⌘N");
}

#[test]
fn batch23_notes_browse_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let browse = actions.iter().find(|a| a.id == "browse_notes").unwrap();
    assert_eq!(browse.shortcut.as_ref().unwrap(), "⌘P");
}

#[test]
fn batch23_notes_duplicate_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let dup = actions.iter().find(|a| a.id == "duplicate_note").unwrap();
    assert_eq!(dup.shortcut.as_ref().unwrap(), "⌘D");
}

#[test]
fn batch23_notes_format_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let fmt = actions.iter().find(|a| a.id == "format").unwrap();
    assert_eq!(fmt.shortcut.as_ref().unwrap(), "⇧⌘T");
}

// ============================================================
// 15. New chat actions: empty inputs produce empty output
// ============================================================

#[test]
fn batch23_new_chat_all_empty() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

#[test]
fn batch23_new_chat_only_last_used() {
    let last_used = vec![NewChatModelInfo {
        model_id: "claude".to_string(),
        display_name: "Claude".to_string(),
        provider: "anthropic".to_string(),
        provider_display_name: "Anthropic".to_string(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_ref().unwrap(), "Last Used Settings");
}

#[test]
fn batch23_new_chat_only_presets() {
    let presets = vec![NewChatPresetInfo {
        id: "general".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_ref().unwrap(), "Presets");
}

#[test]
fn batch23_new_chat_only_models() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt4".to_string(),
        display_name: "GPT-4".to_string(),
        provider: "openai".to_string(),
        provider_display_name: "OpenAI".to_string(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_ref().unwrap(), "Models");
}

#[test]
fn batch23_new_chat_mixed_sections_count() {
    let last = vec![NewChatModelInfo {
        model_id: "c".to_string(),
        display_name: "C".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "g".to_string(),
        name: "G".to_string(),
        icon: IconName::Star,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "m".to_string(),
        display_name: "M".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let actions = get_new_chat_actions(&last, &presets, &models);
    assert_eq!(actions.len(), 3);
}

// ============================================================
// 16. New chat actions: icon assignments
// ============================================================

#[test]
fn batch23_new_chat_last_used_icon() {
    let last = vec![NewChatModelInfo {
        model_id: "c".to_string(),
        display_name: "C".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let actions = get_new_chat_actions(&last, &[], &[]);
    assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
}
