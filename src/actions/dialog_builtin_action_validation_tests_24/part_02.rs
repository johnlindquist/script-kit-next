
#[test]
fn batch24_notes_trash_no_duplicate() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
}

#[test]
fn batch24_notes_trash_no_find() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "find_in_note"));
}

// ============================================================
// 10. Notes full mode with selection: maximum actions
// ============================================================

#[test]
fn batch24_notes_full_mode_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note, duplicate_note, browse_notes, find_in_note, format,
    // copy_note_as, copy_deeplink, create_quicklink, export, enable_auto_sizing = 10
    assert_eq!(actions.len(), 10);
}

#[test]
fn batch24_notes_full_auto_sizing_enabled_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    // Same minus enable_auto_sizing = 9
    assert_eq!(actions.len(), 9);
}

#[test]
fn batch24_notes_no_selection_count() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note, browse_notes, enable_auto_sizing = 3
    assert_eq!(actions.len(), 3);
}

// ============================================================
// 11. Notes icon assignments
// ============================================================

#[test]
fn batch24_notes_new_note_icon_plus() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let new_note = actions.iter().find(|a| a.id == "new_note").unwrap();
    assert_eq!(new_note.icon, Some(IconName::Plus));
}

#[test]
fn batch24_notes_browse_icon_folder_open() {
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
fn batch24_notes_find_icon_magnifying() {
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
fn batch24_notes_auto_sizing_icon_settings() {
    let info = NotesInfo {
        has_selection: false,
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
// 12. Note switcher: notes with empty preview fall back to char count
// ============================================================

#[test]
fn batch24_note_switcher_empty_preview_zero_chars() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".to_string(),
        title: "Empty".to_string(),
        char_count: 0,
        is_current: false,
        is_pinned: false,
        preview: "".to_string(),
        relative_time: "".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_ref().unwrap(), "0 chars");
}

#[test]
fn batch24_note_switcher_empty_preview_one_char() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".to_string(),
        title: "One".to_string(),
        char_count: 1,
        is_current: false,
        is_pinned: false,
        preview: "".to_string(),
        relative_time: "".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_ref().unwrap(), "1 char");
}

#[test]
fn batch24_note_switcher_empty_preview_with_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".to_string(),
        title: "T".to_string(),
        char_count: 50,
        is_current: false,
        is_pinned: false,
        preview: "".to_string(),
        relative_time: "5m ago".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_ref().unwrap(), "5m ago");
}

#[test]
fn batch24_note_switcher_preview_with_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".to_string(),
        title: "T".to_string(),
        char_count: 50,
        is_current: false,
        is_pinned: false,
        preview: "Some content".to_string(),
        relative_time: "2h ago".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(actions[0].description.as_ref().unwrap().contains(" · "));
}

// ============================================================
// 13. Note switcher: pinned + current icon priority
// ============================================================

#[test]
fn batch24_note_switcher_pinned_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".to_string(),
        title: "Pinned".to_string(),
        char_count: 10,
        is_current: false,
        is_pinned: true,
        preview: "".to_string(),
        relative_time: "".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn batch24_note_switcher_current_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".to_string(),
        title: "Current".to_string(),
        char_count: 10,
        is_current: true,
        is_pinned: false,
        preview: "".to_string(),
        relative_time: "".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::Check));
}

#[test]
fn batch24_note_switcher_pinned_trumps_current() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".to_string(),
        title: "Both".to_string(),
        char_count: 10,
        is_current: true,
        is_pinned: true,
        preview: "".to_string(),
        relative_time: "".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn batch24_note_switcher_regular_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".to_string(),
        title: "Regular".to_string(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "".to_string(),
        relative_time: "".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::File));
}

// ============================================================
// 14. AI command bar: all 12 actions present
// ============================================================

#[test]
fn batch24_ai_command_bar_total_12() {
    let actions = get_ai_command_bar_actions();
    assert_eq!(actions.len(), 12);
}

#[test]
fn batch24_ai_command_bar_all_have_icons() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(a.icon.is_some(), "Action {} missing icon", a.id);
    }
}

#[test]
fn batch24_ai_command_bar_all_have_sections() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(a.section.is_some(), "Action {} missing section", a.id);
    }
}

#[test]
fn batch24_ai_command_bar_response_section_count() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .count();
    assert_eq!(count, 3);
}

#[test]
fn batch24_ai_command_bar_actions_section_count() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Actions"))
        .count();
    assert_eq!(count, 4);
}

// ============================================================
// 15. AI command bar: specific shortcut and icon pairs
// ============================================================

#[test]
fn batch24_ai_export_markdown_shortcut_icon() {
    let actions = get_ai_command_bar_actions();
    let export = actions.iter().find(|a| a.id == "export_markdown").unwrap();
    assert_eq!(export.shortcut.as_ref().unwrap(), "⇧⌘E");
    assert_eq!(export.icon, Some(IconName::FileCode));
}

#[test]
fn batch24_ai_branch_from_last_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let branch = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
    assert!(branch.shortcut.is_none());
    assert_eq!(branch.icon, Some(IconName::ArrowRight));
}

#[test]
fn batch24_ai_change_model_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let model = actions.iter().find(|a| a.id == "change_model").unwrap();
    assert!(model.shortcut.is_none());
    assert_eq!(model.icon, Some(IconName::Settings));
}

#[test]
fn batch24_ai_toggle_shortcuts_help_shortcut() {
    let actions = get_ai_command_bar_actions();
    let help = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(help.shortcut.as_ref().unwrap(), "⌘/");
}

// ============================================================
// 16. New chat actions: empty inputs
// ============================================================

#[test]
fn batch24_new_chat_all_empty() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

#[test]
fn batch24_new_chat_only_last_used() {
    let last = vec![NewChatModelInfo {
        model_id: "m1".to_string(),
        display_name: "Model 1".to_string(),
        provider: "p".to_string(),
        provider_display_name: "Provider".to_string(),
    }];
    let actions = get_new_chat_actions(&last, &[], &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
}

#[test]
fn batch24_new_chat_only_presets() {
    let presets = vec![NewChatPresetInfo {
        id: "general".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Presets"));
}

#[test]
fn batch24_new_chat_only_models() {
    let models = vec![NewChatModelInfo {
        model_id: "m1".to_string(),
        display_name: "Model 1".to_string(),
        provider: "p".to_string(),
        provider_display_name: "Provider".to_string(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Models"));
}

#[test]
fn batch24_new_chat_mixed() {
    let last = vec![NewChatModelInfo {
        model_id: "m1".to_string(),
        display_name: "M1".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "g".to_string(),
        name: "G".to_string(),
        icon: IconName::Star,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "m2".to_string(),
        display_name: "M2".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let actions = get_new_chat_actions(&last, &presets, &models);
    assert_eq!(actions.len(), 3);
}

// ============================================================
// 17. New chat actions: icon assignments
// ============================================================

#[test]
fn batch24_new_chat_last_used_icon_bolt() {
    let last = vec![NewChatModelInfo {
        model_id: "m1".to_string(),
        display_name: "M1".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let actions = get_new_chat_actions(&last, &[], &[]);
    assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
}

#[test]
fn batch24_new_chat_model_icon_settings() {
    let models = vec![NewChatModelInfo {
        model_id: "m1".to_string(),
        display_name: "M1".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].icon, Some(IconName::Settings));
}

#[test]
fn batch24_new_chat_preset_icon_preserved() {
    let presets = vec![NewChatPresetInfo {
        id: "g".to_string(),
        name: "General".to_string(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].icon, Some(IconName::Code));
}
