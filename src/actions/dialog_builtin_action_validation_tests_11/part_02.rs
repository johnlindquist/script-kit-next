
#[test]
fn cat09_both_have_reveal_copy_path_copy_filename() {
    for info in &[file_info_file(), file_info_dir()] {
        let actions = get_file_context_actions(info);
        assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
        assert!(actions.iter().any(|a| a.id == "copy_path"));
        assert!(actions.iter().any(|a| a.id == "copy_filename"));
    }
}

#[cfg(target_os = "macos")]
#[test]
fn cat09_quick_look_only_for_files() {
    let file_actions = get_file_context_actions(&file_info_file());
    let dir_actions = get_file_context_actions(&file_info_dir());
    assert!(file_actions.iter().any(|a| a.id == "quick_look"));
    assert!(!dir_actions.iter().any(|a| a.id == "quick_look"));
}

// ============================================================================
// 10. File context — title includes quoted filename
// ============================================================================

#[test]
fn cat10_file_title_includes_name() {
    let actions = get_file_context_actions(&file_info_file());
    let primary = &actions[0];
    assert!(
        primary.title.contains("doc.pdf"),
        "Title should contain filename: {}",
        primary.title
    );
    assert!(primary.title.contains('"'));
}

#[test]
fn cat10_dir_title_includes_name() {
    let actions = get_file_context_actions(&file_info_dir());
    let primary = &actions[0];
    assert!(primary.title.contains("docs"));
}

// ============================================================================
// 11. Path context — directory vs file primary action
// ============================================================================

fn path_dir() -> PathInfo {
    PathInfo {
        path: "/tmp/projects".into(),
        name: "projects".into(),
        is_dir: true,
    }
}

fn path_file() -> PathInfo {
    PathInfo {
        path: "/tmp/readme.md".into(),
        name: "readme.md".into(),
        is_dir: false,
    }
}

#[test]
fn cat11_dir_primary_is_open_directory() {
    let actions = get_path_context_actions(&path_dir());
    assert_eq!(actions[0].id, "open_directory");
}

#[test]
fn cat11_file_primary_is_select_file() {
    let actions = get_path_context_actions(&path_file());
    assert_eq!(actions[0].id, "select_file");
}

#[test]
fn cat11_trash_is_always_last() {
    for info in &[path_dir(), path_file()] {
        let actions = get_path_context_actions(info);
        assert_eq!(actions.last().unwrap().id, "move_to_trash");
    }
}

#[test]
fn cat11_trash_description_mentions_folder_or_file() {
    let dir_actions = get_path_context_actions(&path_dir());
    let file_actions = get_path_context_actions(&path_file());
    let dir_trash = dir_actions
        .iter()
        .find(|a| a.id == "move_to_trash")
        .unwrap();
    let file_trash = file_actions
        .iter()
        .find(|a| a.id == "move_to_trash")
        .unwrap();
    assert!(dir_trash.description.as_ref().unwrap().contains("folder"));
    assert!(file_trash.description.as_ref().unwrap().contains("file"));
}

#[test]
fn cat11_dir_and_file_have_same_action_count() {
    let d = get_path_context_actions(&path_dir());
    let f = get_path_context_actions(&path_file());
    assert_eq!(d.len(), f.len());
}

// ============================================================================
// 12. Path context — common actions present for both
// ============================================================================

#[test]
fn cat12_always_has_copy_path_and_open_in_editor() {
    for info in &[path_dir(), path_file()] {
        let actions = get_path_context_actions(info);
        assert!(actions.iter().any(|a| a.id == "copy_path"));
        assert!(actions.iter().any(|a| a.id == "open_in_editor"));
        assert!(actions.iter().any(|a| a.id == "open_in_terminal"));
        assert!(actions.iter().any(|a| a.id == "open_in_finder"));
        assert!(actions.iter().any(|a| a.id == "copy_filename"));
    }
}

// ============================================================================
// 13. AI command bar — exact action count and section distribution
// ============================================================================

#[test]
fn cat13_ai_command_bar_has_12_actions() {
    let actions = get_ai_command_bar_actions();
    assert_eq!(actions.len(), 12);
}

#[test]
fn cat13_ai_sections_present() {
    let actions = get_ai_command_bar_actions();
    let sections: HashSet<String> = actions.iter().filter_map(|a| a.section.clone()).collect();
    for expected in &[
        "Response",
        "Actions",
        "Attachments",
        "Export",
        "Help",
        "Settings",
    ] {
        assert!(
            sections.contains(*expected),
            "Missing section: {}",
            expected
        );
    }
}

#[test]
fn cat13_all_ai_actions_have_icons() {
    for action in &get_ai_command_bar_actions() {
        assert!(
            action.icon.is_some(),
            "AI action {} missing icon",
            action.id
        );
    }
}

#[test]
fn cat13_all_ai_actions_have_sections() {
    for action in &get_ai_command_bar_actions() {
        assert!(
            action.section.is_some(),
            "AI action {} missing section",
            action.id
        );
    }
}

#[test]
fn cat13_ai_action_ids_unique() {
    let actions = get_ai_command_bar_actions();
    let ids: HashSet<String> = action_ids(&actions).into_iter().collect();
    assert_eq!(ids.len(), actions.len(), "Duplicate IDs in AI command bar");
}

// ============================================================================
// 14. Notes command bar — conditional actions based on state
// ============================================================================

#[test]
fn cat14_full_feature_notes_actions_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note, duplicate_note, browse_notes, find_in_note, format, copy_note_as,
    // copy_deeplink, create_quicklink, export, enable_auto_sizing = 10
    assert_eq!(actions.len(), 10);
}

#[test]
fn cat14_trash_view_hides_editing_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
    assert!(!actions.iter().any(|a| a.id == "find_in_note"));
    assert!(!actions.iter().any(|a| a.id == "format"));
    assert!(!actions.iter().any(|a| a.id == "export"));
}

#[test]
fn cat14_no_selection_minimal() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    // Only new_note and browse_notes (no auto_sizing since enabled)
    assert_eq!(actions.len(), 2);
}

#[test]
fn cat14_auto_sizing_disabled_adds_enable_action() {
    let with = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let without = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let a_with = get_notes_command_bar_actions(&with);
    let a_without = get_notes_command_bar_actions(&without);
    assert!(a_with.iter().any(|a| a.id == "enable_auto_sizing"));
    assert!(!a_without.iter().any(|a| a.id == "enable_auto_sizing"));
    assert_eq!(a_with.len(), a_without.len() + 1);
}

// ============================================================================
// 15. Notes command bar — all actions have icons and sections
// ============================================================================

#[test]
fn cat15_all_notes_actions_have_icons() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for action in &get_notes_command_bar_actions(&info) {
        assert!(
            action.icon.is_some(),
            "Notes action {} missing icon",
            action.id
        );
    }
}

#[test]
fn cat15_all_notes_actions_have_sections() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for action in &get_notes_command_bar_actions(&info) {
        assert!(
            action.section.is_some(),
            "Notes action {} missing section",
            action.id
        );
    }
}

// ============================================================================
// 16. New chat actions — section structure
// ============================================================================

fn sample_model() -> NewChatModelInfo {
    NewChatModelInfo {
        model_id: "claude-3".into(),
        display_name: "Claude 3".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }
}

fn sample_preset() -> NewChatPresetInfo {
    NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: crate::designs::icon_variations::IconName::Star,
    }
}

#[test]
fn cat16_empty_inputs_produce_empty_actions() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

#[test]
fn cat16_sections_appear_in_order() {
    let actions = get_new_chat_actions(&[sample_model()], &[sample_preset()], &[sample_model()]);
    let sections: Vec<String> = actions.iter().filter_map(|a| a.section.clone()).collect();
    let last_used_pos = sections.iter().position(|s| s == "Last Used Settings");
    let presets_pos = sections.iter().position(|s| s == "Presets");
    let models_pos = sections.iter().position(|s| s == "Models");
    assert!(last_used_pos.unwrap() < presets_pos.unwrap());
    assert!(presets_pos.unwrap() < models_pos.unwrap());
}

#[test]
fn cat16_preset_has_no_description() {
    let actions = get_new_chat_actions(&[], &[sample_preset()], &[]);
    assert!(actions[0].description.is_none());
}

#[test]
fn cat16_model_description_is_provider() {
    let actions = get_new_chat_actions(&[], &[], &[sample_model()]);
    assert_eq!(actions[0].description, Some("Anthropic".to_string()));
}

#[test]
fn cat16_last_used_has_bolt_icon() {
    let actions = get_new_chat_actions(&[sample_model()], &[], &[]);
    assert_eq!(
        actions[0].icon,
        Some(crate::designs::icon_variations::IconName::BoltFilled)
    );
}

#[test]
fn cat16_models_have_settings_icon() {
    let actions = get_new_chat_actions(&[], &[], &[sample_model()]);
    assert_eq!(
        actions[0].icon,
        Some(crate::designs::icon_variations::IconName::Settings)
    );
}

// ============================================================================
// 17. Note switcher — icon hierarchy and section assignment
// ============================================================================

fn make_note(id: &str, pinned: bool, current: bool) -> NoteSwitcherNoteInfo {
    NoteSwitcherNoteInfo {
        id: id.into(),
        title: format!("Note {}", id),
        char_count: 42,
        is_current: current,
        is_pinned: pinned,
        preview: "some preview text".into(),
        relative_time: "2m ago".into(),
    }
}

#[test]
fn cat17_pinned_gets_star_icon() {
    let actions = get_note_switcher_actions(&[make_note("1", true, false)]);
    assert_eq!(
        actions[0].icon,
        Some(crate::designs::icon_variations::IconName::StarFilled)
    );
}

#[test]
fn cat17_current_gets_check_icon() {
    let actions = get_note_switcher_actions(&[make_note("1", false, true)]);
    assert_eq!(
        actions[0].icon,
        Some(crate::designs::icon_variations::IconName::Check)
    );
}

#[test]
fn cat17_regular_gets_file_icon() {
    let actions = get_note_switcher_actions(&[make_note("1", false, false)]);
    assert_eq!(
        actions[0].icon,
        Some(crate::designs::icon_variations::IconName::File)
    );
}

#[test]
fn cat17_pinned_overrides_current_for_icon() {
    // When both pinned and current, pinned icon wins
    let actions = get_note_switcher_actions(&[make_note("1", true, true)]);
    assert_eq!(
        actions[0].icon,
        Some(crate::designs::icon_variations::IconName::StarFilled)
    );
}

#[test]
fn cat17_pinned_note_in_pinned_section() {
    let actions = get_note_switcher_actions(&[make_note("1", true, false)]);
    assert_eq!(actions[0].section, Some("Pinned".to_string()));
}

#[test]
fn cat17_unpinned_note_in_recent_section() {
    let actions = get_note_switcher_actions(&[make_note("1", false, false)]);
    assert_eq!(actions[0].section, Some("Recent".to_string()));
}

#[test]
fn cat17_current_note_has_bullet_prefix() {
    let actions = get_note_switcher_actions(&[make_note("1", false, true)]);
    assert!(
        actions[0].title.starts_with("• "),
        "Current note should have bullet: {}",
        actions[0].title
    );
}

#[test]
fn cat17_non_current_no_bullet() {
    let actions = get_note_switcher_actions(&[make_note("1", false, false)]);
    assert!(!actions[0].title.starts_with("• "));
}

// ============================================================================
// 18. Note switcher — description rendering edge cases
// ============================================================================

#[test]
fn cat18_preview_with_time_uses_separator() {
    let note = NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "hello".into(),
        relative_time: "5m ago".into(),
    };
    let actions = get_note_switcher_actions(&[note]);
    assert!(actions[0].description.as_ref().unwrap().contains(" · "));
}
