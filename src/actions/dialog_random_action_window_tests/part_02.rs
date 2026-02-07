
#[test]
fn chat_many_models_only_one_checkmark() {
    let models: Vec<ChatModelInfo> = (0..5)
        .map(|i| ChatModelInfo {
            id: format!("m{}", i),
            display_name: format!("Model {}", i),
            provider: format!("P{}", i),
        })
        .collect();
    let info = ChatPromptInfo {
        current_model: Some("Model 2".into()),
        available_models: models,
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let checkmark_count = actions.iter().filter(|a| a.title.contains('✓')).count();
    assert_eq!(checkmark_count, 1, "Only one model should have checkmark");
    let checked = actions.iter().find(|a| a.title.contains('✓')).unwrap();
    assert_eq!(checked.id, "select_model_m2");
}

#[test]
fn chat_response_without_messages_still_gives_copy_response() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"copy_response"));
    assert!(!ids.contains(&"clear_conversation"));
}

#[test]
fn chat_messages_without_response_gives_clear_but_no_copy() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let ids = action_ids(&actions);
    assert!(!ids.contains(&"copy_response"));
    assert!(ids.contains(&"clear_conversation"));
}

// =========================================================================
// 7. Notes command bar — trash view disables most actions
// =========================================================================

#[test]
fn notes_trash_view_disables_edit_copy_export() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    // Trash view should hide duplicate, find, format, copy, export
    assert!(!ids.contains(&"duplicate_note"));
    assert!(!ids.contains(&"find_in_note"));
    assert!(!ids.contains(&"format"));
    assert!(!ids.contains(&"copy_note_as"));
    assert!(!ids.contains(&"export"));
    // But new_note and browse_notes are always present
    assert!(ids.contains(&"new_note"));
    assert!(ids.contains(&"browse_notes"));
}

#[test]
fn notes_no_selection_disables_edit_copy_export() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(!ids.contains(&"duplicate_note"));
    assert!(!ids.contains(&"find_in_note"));
    assert!(!ids.contains(&"copy_note_as"));
}

#[test]
fn notes_auto_sizing_toggled_all_permutations() {
    for &sel in &[false, true] {
        for &trash in &[false, true] {
            // auto=false => show enable_auto_sizing
            let off_ids: Vec<String> = get_notes_command_bar_actions(&NotesInfo {
                has_selection: sel,
                is_trash_view: trash,
                auto_sizing_enabled: false,
            })
            .iter()
            .map(|a| a.id.clone())
            .collect();
            assert!(
                off_ids.contains(&"enable_auto_sizing".to_string()),
                "Missing enable_auto_sizing for sel={}, trash={}, auto=false",
                sel,
                trash
            );

            // auto=true => no enable_auto_sizing
            let on_ids: Vec<String> = get_notes_command_bar_actions(&NotesInfo {
                has_selection: sel,
                is_trash_view: trash,
                auto_sizing_enabled: true,
            })
            .iter()
            .map(|a| a.id.clone())
            .collect();
            assert!(
                !on_ids.contains(&"enable_auto_sizing".to_string()),
                "Unexpected enable_auto_sizing for sel={}, trash={}, auto=true",
                sel,
                trash
            );
        }
    }
}

// =========================================================================
// 8. File context — all FileType variants have primary action
// =========================================================================

#[test]
fn file_context_all_types_have_primary_with_enter_shortcut() {
    let types = [
        (FileType::File, false),
        (FileType::Directory, true),
        (FileType::Document, false),
        (FileType::Image, false),
        (FileType::Application, false),
    ];
    for (ft, is_dir) in types {
        let info = FileInfo {
            path: format!("/test/{:?}", ft),
            name: format!("{:?}", ft),
            file_type: ft,
            is_dir,
        };
        let actions = get_file_context_actions(&info);
        assert!(
            !actions.is_empty(),
            "FileType {:?} should produce actions",
            ft
        );
        assert_eq!(
            actions[0].shortcut.as_deref(),
            Some("↵"),
            "Primary action for {:?} should have enter shortcut",
            ft
        );
    }
}

#[test]
fn file_context_directory_never_has_quick_look() {
    let info = FileInfo {
        path: "/test/dir".into(),
        name: "dir".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&info);
    let ids = action_ids(&actions);
    assert!(
        !ids.contains(&"quick_look"),
        "Directory should not have quick_look"
    );
}

// =========================================================================
// 9. Path context — directory vs file primary action
// =========================================================================

#[test]
fn path_context_dir_starts_with_open_directory() {
    let info = PathInfo::new("mydir", "/home/mydir", true);
    let actions = get_path_context_actions(&info);
    assert_eq!(actions[0].id, "open_directory");
    assert!(actions[0].title.contains("mydir"));
}

#[test]
fn path_context_file_starts_with_select_file() {
    let info = PathInfo::new("data.csv", "/home/data.csv", false);
    let actions = get_path_context_actions(&info);
    assert_eq!(actions[0].id, "select_file");
    assert!(actions[0].title.contains("data.csv"));
}

#[test]
fn path_context_trash_differs_for_file_and_dir() {
    let dir = get_path_context_actions(&PathInfo::new("d", "/d", true));
    let file = get_path_context_actions(&PathInfo::new("f", "/f", false));
    let dir_trash = find_action(&dir, "move_to_trash").unwrap();
    let file_trash = find_action(&file, "move_to_trash").unwrap();
    assert_ne!(dir_trash.description, file_trash.description);
    assert!(dir_trash.description.as_ref().unwrap().contains("folder"));
    assert!(file_trash.description.as_ref().unwrap().contains("file"));
}

// =========================================================================
// 10. New chat actions — varied inputs
// =========================================================================

#[test]
fn new_chat_only_last_used() {
    let last = vec![
        NewChatModelInfo {
            model_id: "a".into(),
            display_name: "A".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        },
        NewChatModelInfo {
            model_id: "b".into(),
            display_name: "B".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        },
    ];
    let actions = get_new_chat_actions(&last, &[], &[]);
    assert_eq!(actions.len(), 2);
    assert!(actions
        .iter()
        .all(|a| a.section.as_deref() == Some("Last Used Settings")));
    assert!(actions.iter().all(|a| a.icon == Some(IconName::BoltFilled)));
}

#[test]
fn new_chat_only_presets() {
    let presets = vec![NewChatPresetInfo {
        id: "code".into(),
        name: "Code".into(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Presets"));
    assert_eq!(actions[0].icon, Some(IconName::Code));
}

#[test]
fn new_chat_only_models() {
    let models = vec![NewChatModelInfo {
        model_id: "x".into(),
        display_name: "X".into(),
        provider: "xp".into(),
        provider_display_name: "XP".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Models"));
    assert_eq!(actions[0].icon, Some(IconName::Settings));
}

#[test]
fn new_chat_mixed_sections_order() {
    let last = vec![NewChatModelInfo {
        model_id: "l".into(),
        display_name: "L".into(),
        provider: "lp".into(),
        provider_display_name: "LP".into(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "p".into(),
        name: "P".into(),
        icon: IconName::Star,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "m".into(),
        display_name: "M".into(),
        provider: "mp".into(),
        provider_display_name: "MP".into(),
    }];
    let actions = get_new_chat_actions(&last, &presets, &models);
    assert_eq!(actions.len(), 3);
    // Verify order: Last Used Settings → Presets → Models
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
    assert_eq!(actions[1].section.as_deref(), Some("Presets"));
    assert_eq!(actions[2].section.as_deref(), Some("Models"));
}

// =========================================================================
// 11. Note switcher — edge cases
// =========================================================================

#[test]
fn note_switcher_many_notes_all_have_correct_ids() {
    let notes: Vec<NoteSwitcherNoteInfo> = (0..10)
        .map(|i| NoteSwitcherNoteInfo {
            id: format!("note-{}", i),
            title: format!("Note {}", i),
            char_count: i * 100,
            is_current: i == 3,
            is_pinned: i == 0 || i == 5,
            preview: String::new(),
            relative_time: String::new(),
        })
        .collect();
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions.len(), 10);
    for (i, action) in actions.iter().enumerate() {
        assert_eq!(action.id, format!("note_note-{}", i));
        assert!(
            action.section.as_deref() == Some("Recent")
                || action.section.as_deref() == Some("Pinned"),
            "Note switcher action '{}' should be in 'Recent' or 'Pinned' section, got {:?}",
            action.id,
            action.section
        );
    }
    // Current note (index 3) has bullet
    assert!(actions[3].title.starts_with("• "));
    // Non-current notes don't
    assert!(!actions[0].title.starts_with("• "));
    // Pinned notes get star icon
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    assert_eq!(actions[5].icon, Some(IconName::StarFilled));
    // Current non-pinned gets check
    assert_eq!(actions[3].icon, Some(IconName::Check));
    // Regular notes get file icon
    assert_eq!(actions[1].icon, Some(IconName::File));
}

#[test]
fn note_switcher_large_char_count() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "big".into(),
        title: "Big Note".into(),
        char_count: 1_000_000,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("1000000 chars"));
}

// =========================================================================
// 12. Action scoring — boundary conditions
// =========================================================================

#[test]
fn score_action_single_char_search() {
    let action = Action::new("run", "Run Script", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "r");
    assert!(
        score >= 100,
        "Single char 'r' should prefix match 'run script', got {}",
        score
    );
}

#[test]
fn score_action_exact_title_match() {
    let action = Action::new("test", "Run Script", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "run script");
    assert!(
        score >= 100,
        "Exact title match should score high, got {}",
        score
    );
}

#[test]
fn score_action_expects_lowercased_query() {
    // score_action expects the caller to pass a pre-lowercased search string
    // (matching ActionsDialog::handle_char which lowercases the query)
    let action = Action::new("test", "Edit Script", None, ActionCategory::ScriptContext);
    let lower = ActionsDialog::score_action(&action, "edit");
    assert!(
        lower >= 100,
        "Lowercased prefix should score high, got {}",
        lower
    );
    // Uppercase query won't match because score_action compares against title_lower
    let upper = ActionsDialog::score_action(&action, "EDIT");
    assert_eq!(
        upper, 0,
        "Non-lowercased query should not match title_lower"
    );
}

#[test]
fn score_action_partial_word_still_matches() {
    let action = Action::new(
        "test",
        "Reveal in Finder",
        None,
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "rev");
    assert!(
        score >= 100,
        "'rev' should prefix match 'reveal in finder', got {}",
        score
    );
}

// =========================================================================
// 13. Fuzzy match — more edge cases
// =========================================================================

#[test]
fn fuzzy_match_single_char_at_end() {
    assert!(ActionsDialog::fuzzy_match("hello world", "d"));
}

#[test]
fn fuzzy_match_single_char_not_present() {
    assert!(!ActionsDialog::fuzzy_match("hello world", "z"));
}

#[test]
fn fuzzy_match_full_string() {
    assert!(ActionsDialog::fuzzy_match("test", "test"));
}

#[test]
fn fuzzy_match_unicode_chars() {
    assert!(ActionsDialog::fuzzy_match("café résumé", "cr"));
}

#[test]
fn fuzzy_match_interleaved_chars() {
    assert!(ActionsDialog::fuzzy_match("abcdefghij", "acegi"));
}

// =========================================================================
// 14. format_shortcut_hint — more combinations
// =========================================================================

#[test]
fn format_shortcut_delete_key() {
    let result = ActionsDialog::format_shortcut_hint("cmd+delete");
    assert!(result.contains('⌘'));
    assert!(result.contains('⌫'));
}
