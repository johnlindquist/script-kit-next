
// =============================================================================
// Category 18: Notes — browse_notes details
// =============================================================================

#[test]
fn cat28_18_notes_browse_shortcut() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let bn = actions.iter().find(|a| a.id == "browse_notes").unwrap();
    assert_eq!(bn.shortcut.as_deref(), Some("⌘P"));
}

#[test]
fn cat28_18_notes_browse_icon() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let bn = actions.iter().find(|a| a.id == "browse_notes").unwrap();
    assert_eq!(bn.icon, Some(IconName::FolderOpen));
}

#[test]
fn cat28_18_notes_browse_section() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let bn = actions.iter().find(|a| a.id == "browse_notes").unwrap();
    assert_eq!(bn.section.as_deref(), Some("Notes"));
}

#[test]
fn cat28_18_notes_browse_always_present() {
    // Present even in trash view
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "browse_notes"));
}

// =============================================================================
// Category 19: Notes full mode action count
// =============================================================================

#[test]
fn cat28_19_notes_full_mode_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note, duplicate, browse, find, format, copy_note_as, copy_deeplink, create_quicklink, export, auto_sizing
    assert_eq!(actions.len(), 10);
}

#[test]
fn cat28_19_notes_full_auto_sizing_enabled_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    // Same minus auto_sizing = 9
    assert_eq!(actions.len(), 9);
}

#[test]
fn cat28_19_notes_no_selection_count() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note, browse, auto_sizing = 3
    assert_eq!(actions.len(), 3);
}

#[test]
fn cat28_19_notes_trash_view_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note, browse, auto_sizing = 3 (trash hides duplicate, edit, copy sections)
    assert_eq!(actions.len(), 3);
}

// =============================================================================
// Category 20: Note switcher — pinned note icon and section
// =============================================================================

#[test]
fn cat28_20_note_switcher_pinned_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Pinned Note".into(),
        char_count: 100,
        is_current: false,
        is_pinned: true,
        preview: "Some content".into(),
        relative_time: "1d ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn cat28_20_note_switcher_pinned_section() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Pinned Note".into(),
        char_count: 100,
        is_current: false,
        is_pinned: true,
        preview: "Some content".into(),
        relative_time: "1d ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
}

#[test]
fn cat28_20_note_switcher_regular_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Regular Note".into(),
        char_count: 50,
        is_current: false,
        is_pinned: false,
        preview: "test".into(),
        relative_time: "2h ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::File));
}

#[test]
fn cat28_20_note_switcher_regular_section() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Regular Note".into(),
        char_count: 50,
        is_current: false,
        is_pinned: false,
        preview: "test".into(),
        relative_time: "2h ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Recent"));
}

// =============================================================================
// Category 21: Note switcher — current note icon and title prefix
// =============================================================================

#[test]
fn cat28_21_note_switcher_current_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "xyz".into(),
        title: "Current Note".into(),
        char_count: 200,
        is_current: true,
        is_pinned: false,
        preview: "body".into(),
        relative_time: "5m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::Check));
}

#[test]
fn cat28_21_note_switcher_current_bullet_prefix() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "xyz".into(),
        title: "Current Note".into(),
        char_count: 200,
        is_current: true,
        is_pinned: false,
        preview: "body".into(),
        relative_time: "5m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(actions[0].title.starts_with("• "));
}

#[test]
fn cat28_21_note_switcher_non_current_no_bullet() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "xyz".into(),
        title: "Other Note".into(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "test".into(),
        relative_time: "1h ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(!actions[0].title.starts_with("• "));
}

#[test]
fn cat28_21_pinned_trumps_current_for_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "xyz".into(),
        title: "Both".into(),
        char_count: 100,
        is_current: true,
        is_pinned: true,
        preview: "test".into(),
        relative_time: "1h ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

// =============================================================================
// Category 22: New chat — model description is provider_display_name
// =============================================================================

#[test]
fn cat28_22_new_chat_model_description() {
    let models = vec![NewChatModelInfo {
        model_id: "claude-3-opus".into(),
        display_name: "Claude 3 Opus".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    let model_action = actions.iter().find(|a| a.id == "model_0").unwrap();
    assert_eq!(model_action.description.as_deref(), Some("Anthropic"));
}

#[test]
fn cat28_22_new_chat_model_icon() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    let model_action = actions.iter().find(|a| a.id == "model_0").unwrap();
    assert_eq!(model_action.icon, Some(IconName::Settings));
}

#[test]
fn cat28_22_new_chat_model_section() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    let model_action = actions.iter().find(|a| a.id == "model_0").unwrap();
    assert_eq!(model_action.section.as_deref(), Some("Models"));
}

#[test]
fn cat28_22_new_chat_model_title_is_display_name() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    let model_action = actions.iter().find(|a| a.id == "model_0").unwrap();
    assert_eq!(model_action.title, "GPT-4");
}

// =============================================================================
// Category 23: New chat — preset description is None
// =============================================================================

#[test]
fn cat28_23_new_chat_preset_description_none() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    let preset = actions.iter().find(|a| a.id == "preset_general").unwrap();
    assert!(preset.description.is_none());
}

#[test]
fn cat28_23_new_chat_preset_icon_preserved() {
    let presets = vec![NewChatPresetInfo {
        id: "code".into(),
        name: "Code".into(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    let preset = actions.iter().find(|a| a.id == "preset_code").unwrap();
    assert_eq!(preset.icon, Some(IconName::Code));
}

#[test]
fn cat28_23_new_chat_preset_section() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    let preset = actions.iter().find(|a| a.id == "preset_general").unwrap();
    assert_eq!(preset.section.as_deref(), Some("Presets"));
}

#[test]
fn cat28_23_new_chat_preset_title() {
    let presets = vec![NewChatPresetInfo {
        id: "writer".into(),
        name: "Writer".into(),
        icon: IconName::File,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    let preset = actions.iter().find(|a| a.id == "preset_writer").unwrap();
    assert_eq!(preset.title, "Writer");
}

// =============================================================================
// Category 24: format_shortcut_hint (builders.rs version) — simple transforms
// =============================================================================

#[test]
fn cat28_24_builders_format_hint_cmd_c() {
    assert_eq!(super::builders::to_deeplink_name("cmd+c"), "cmd-c");
}

#[test]
fn cat28_24_to_deeplink_name_basic() {
    assert_eq!(to_deeplink_name("My Script"), "my-script");
}

#[test]
fn cat28_24_to_deeplink_name_underscores() {
    assert_eq!(to_deeplink_name("hello_world"), "hello-world");
}

#[test]
fn cat28_24_to_deeplink_name_empty() {
    assert_eq!(to_deeplink_name(""), "");
}

// =============================================================================
// Category 25: Action with_shortcut_opt: None vs Some
// =============================================================================

#[test]
fn cat28_25_with_shortcut_opt_some() {
    let action = Action::new("a", "A", None, ActionCategory::ScriptContext)
        .with_shortcut_opt(Some("⌘A".into()));
    assert_eq!(action.shortcut.as_deref(), Some("⌘A"));
    assert_eq!(action.shortcut_lower.as_deref(), Some("⌘a"));
}

#[test]
fn cat28_25_with_shortcut_opt_none() {
    let action = Action::new("a", "A", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
    assert!(action.shortcut.is_none());
    assert!(action.shortcut_lower.is_none());
}

#[test]
fn cat28_25_with_shortcut_sets_lower() {
    let action = Action::new("a", "A", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
    assert_eq!(action.shortcut_lower.as_deref(), Some("⌘⇧c"));
}

#[test]
fn cat28_25_action_new_no_shortcut_lower() {
    let action = Action::new("a", "A", None, ActionCategory::ScriptContext);
    assert!(action.shortcut_lower.is_none());
}

// =============================================================================
// Category 26: Action with_icon and with_section
// =============================================================================

#[test]
fn cat28_26_with_icon_sets_field() {
    let action =
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_icon(IconName::Copy);
    assert_eq!(action.icon, Some(IconName::Copy));
}

#[test]
fn cat28_26_action_new_no_icon() {
    let action = Action::new("a", "A", None, ActionCategory::ScriptContext);
    assert!(action.icon.is_none());
}

#[test]
fn cat28_26_with_section_sets_field() {
    let action =
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("My Section");
    assert_eq!(action.section.as_deref(), Some("My Section"));
}

#[test]
fn cat28_26_action_new_no_section() {
    let action = Action::new("a", "A", None, ActionCategory::ScriptContext);
    assert!(action.section.is_none());
}

// =============================================================================
// Category 27: Action cached lowercase fields
// =============================================================================

#[test]
fn cat28_27_title_lower_computed() {
    let action = Action::new("a", "Hello World", None, ActionCategory::ScriptContext);
    assert_eq!(action.title_lower, "hello world");
}

#[test]
fn cat28_27_description_lower_computed() {
    let action = Action::new(
        "a",
        "A",
        Some("Some Description".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(
        action.description_lower.as_deref(),
        Some("some description")
    );
}
