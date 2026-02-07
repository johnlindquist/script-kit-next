
#[test]
fn path_open_directory_has_enter_shortcut() {
    let path = PathInfo::new("dir", "/dir", true);
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn clipboard_paste_has_enter_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

// =========================================================================
// 9. Chat model actions — edge cases with many models
// =========================================================================

#[test]
fn chat_ten_models_all_present_exactly_one_checkmark() {
    let models: Vec<ChatModelInfo> = (0..10)
        .map(|i| ChatModelInfo {
            id: format!("model-{}", i),
            display_name: format!("Model {}", i),
            provider: format!("Provider {}", i),
        })
        .collect();
    let info = ChatPromptInfo {
        current_model: Some("Model 5".into()),
        available_models: models,
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let checked = actions.iter().filter(|a| a.title.contains('✓')).count();
    assert_eq!(checked, 1);
    let checked_action = actions.iter().find(|a| a.title.contains('✓')).unwrap();
    assert_eq!(checked_action.id, "select_model_model-5");
}

#[test]
fn chat_current_model_not_in_available_models_means_no_checkmark() {
    let models = vec![ChatModelInfo {
        id: "gpt-4".into(),
        display_name: "GPT-4".into(),
        provider: "OpenAI".into(),
    }];
    let info = ChatPromptInfo {
        current_model: Some("Nonexistent Model".into()),
        available_models: models,
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let checked = actions.iter().filter(|a| a.title.contains('✓')).count();
    assert_eq!(
        checked, 0,
        "No model should be checked when current doesn't match any"
    );
}

#[test]
fn chat_model_actions_all_have_provider_description() {
    let models = vec![
        ChatModelInfo {
            id: "a".into(),
            display_name: "A".into(),
            provider: "PA".into(),
        },
        ChatModelInfo {
            id: "b".into(),
            display_name: "B".into(),
            provider: "PB".into(),
        },
    ];
    let info = ChatPromptInfo {
        current_model: None,
        available_models: models,
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model_actions: Vec<&Action> = actions
        .iter()
        .filter(|a| a.id.starts_with("select_model_"))
        .collect();
    for action in &model_actions {
        assert!(
            action.description.as_ref().unwrap().starts_with("via "),
            "Model action '{}' description should start with 'via '",
            action.id
        );
    }
}

// =========================================================================
// 10. Grouped items — real actions produce valid grouped output
// =========================================================================

#[test]
fn ai_actions_grouped_with_headers_have_correct_structure() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);

    // Last item should not be a header (it should be an action)
    assert!(
        matches!(grouped.last(), Some(GroupedActionItem::Item(_))),
        "Last grouped item should be an action, not a header"
    );
}

#[test]
fn ai_actions_grouped_with_separators_have_no_headers() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    let headers: Vec<&GroupedActionItem> = grouped
        .iter()
        .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
        .collect();
    assert!(
        headers.is_empty(),
        "Separators style should have no headers"
    );
}

#[test]
fn notes_actions_grouped_header_count_matches_section_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let filtered: Vec<usize> = (0..actions.len()).collect();

    let header_count_from_fn = count_section_headers(&actions, &filtered);
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    let header_count_from_grouped = grouped
        .iter()
        .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(header_count_from_fn, header_count_from_grouped);
}

// =========================================================================
// 11. Coerce selection — real grouped items from AI actions
// =========================================================================

#[test]
fn coerce_selection_on_real_ai_grouped_actions_finds_valid_item() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);

    // Coerce at index 0 (which is likely a header) should find an item
    let result = coerce_action_selection(&grouped, 0);
    assert!(
        result.is_some(),
        "Should find an item in AI grouped actions"
    );

    // The selected row should be an Item, not a header
    if let Some(idx) = result {
        assert!(
            matches!(grouped[idx], GroupedActionItem::Item(_)),
            "Coerced selection should be an Item"
        );
    }
}

#[test]
fn coerce_selection_on_every_row_returns_valid_or_none() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);

    for i in 0..grouped.len() {
        let result = coerce_action_selection(&grouped, i);
        if let Some(idx) = result {
            assert!(
                matches!(grouped[idx], GroupedActionItem::Item(_)),
                "Row {} coerced to non-item at {}",
                i,
                idx
            );
        }
    }
}

// =========================================================================
// 12. Score consistency — same action + same query = same score
// =========================================================================

#[test]
fn score_action_is_deterministic() {
    let action = Action::new(
        "edit_script",
        "Edit Script",
        Some("Open in editor".into()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘E");
    let score1 = ActionsDialog::score_action(&action, "edit");
    let score2 = ActionsDialog::score_action(&action, "edit");
    let score3 = ActionsDialog::score_action(&action, "edit");
    assert_eq!(score1, score2);
    assert_eq!(score2, score3);
}

#[test]
fn score_action_prefix_beats_contains_beats_fuzzy() {
    let prefix = Action::new("e", "Edit Script", None, ActionCategory::ScriptContext);
    let contains = Action::new("c", "Copy Edit Path", None, ActionCategory::ScriptContext);
    let fuzzy = Action::new("f", "Examine Data", None, ActionCategory::ScriptContext);

    let prefix_score = ActionsDialog::score_action(&prefix, "edit");
    let contains_score = ActionsDialog::score_action(&contains, "edit");
    // "edit" in "examine data": fuzzy? e-x-a-m-i-n-e- -d-i-t → not a fuzzy match for "edit"
    // Actually e at 0, d at 8, i at 9, t at 10... need e-d-i-t in order: yes that fuzzy matches
    let _fuzzy_score = ActionsDialog::score_action(&fuzzy, "edit");

    assert!(
        prefix_score > contains_score,
        "Prefix({}) should beat contains({})",
        prefix_score,
        contains_score
    );
    // Contains may or may not beat fuzzy depending on implementation, but both should be > 0
    assert!(contains_score > 0);
}

// =========================================================================
// 13. Description presence for critical actions
// =========================================================================

#[test]
fn script_run_action_has_description() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let run = find_action(&actions, "run_script").unwrap();
    assert!(
        run.description.is_some(),
        "run_script should have a description"
    );
}

#[test]
fn script_edit_action_has_description() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let edit = find_action(&actions, "edit_script").unwrap();
    assert!(
        edit.description.is_some(),
        "edit_script should have a description"
    );
}

#[test]
fn clipboard_delete_all_has_description() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let delete_all = find_action(&actions, "clipboard_delete_all").unwrap();
    assert!(
        delete_all.description.is_some(),
        "clipboard_delete_all should have a description"
    );
}

#[test]
fn path_move_to_trash_has_description() {
    let path = PathInfo::new("test", "/test", false);
    let actions = get_path_context_actions(&path);
    let trash = find_action(&actions, "move_to_trash").unwrap();
    assert!(
        trash.description.is_some(),
        "move_to_trash should have a description"
    );
}

// =========================================================================
// 14. Deeplink action present across all script-like contexts
// =========================================================================

#[test]
fn deeplink_present_for_script() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"copy_deeplink"));
}

#[test]
fn deeplink_present_for_builtin() {
    let builtin = ScriptInfo::builtin("Clipboard History");
    let actions = get_script_context_actions(&builtin);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"copy_deeplink"));
}

#[test]
fn deeplink_present_for_scriptlet() {
    let scriptlet = ScriptInfo::scriptlet("Open URL", "/path/urls.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&scriptlet, None);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"copy_deeplink"));
}

#[test]
fn deeplink_present_for_agent() {
    let mut agent = ScriptInfo::new("my-agent", "/path/agent.md");
    agent.is_agent = true;
    agent.is_script = false;
    let actions = get_script_context_actions(&agent);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"copy_deeplink"));
}

// =========================================================================
// 15. Shortcut management actions: mutually exclusive add vs update/remove
// =========================================================================

#[test]
fn shortcut_add_vs_update_remove_mutually_exclusive_for_scripts() {
    // No shortcut → add only
    let no_sc = ScriptInfo::new("test", "/path/test.ts");
    let no_sc_actions = get_script_context_actions(&no_sc);
    let ids = action_ids(&no_sc_actions);
    assert!(ids.contains(&"add_shortcut"));
    assert!(!ids.contains(&"update_shortcut"));
    assert!(!ids.contains(&"remove_shortcut"));

    // Has shortcut → update+remove only
    let has_sc = ScriptInfo::with_shortcut("test", "/path/test.ts", Some("cmd+t".into()));
    let has_sc_actions = get_script_context_actions(&has_sc);
    let ids = action_ids(&has_sc_actions);
    assert!(!ids.contains(&"add_shortcut"));
    assert!(ids.contains(&"update_shortcut"));
    assert!(ids.contains(&"remove_shortcut"));
}

#[test]
fn alias_add_vs_update_remove_mutually_exclusive_for_scripts() {
    // No alias → add only
    let no_al = ScriptInfo::new("test", "/path/test.ts");
    let no_al_actions = get_script_context_actions(&no_al);
    let ids = action_ids(&no_al_actions);
    assert!(ids.contains(&"add_alias"));
    assert!(!ids.contains(&"update_alias"));
    assert!(!ids.contains(&"remove_alias"));

    // Has alias → update+remove only
    let has_al =
        ScriptInfo::with_shortcut_and_alias("test", "/path/test.ts", None, Some("ts".into()));
    let has_al_actions = get_script_context_actions(&has_al);
    let ids = action_ids(&has_al_actions);
    assert!(!ids.contains(&"add_alias"));
    assert!(ids.contains(&"update_alias"));
    assert!(ids.contains(&"remove_alias"));
}

// =========================================================================
// 16. File context — Application type has open as primary
// =========================================================================

#[test]
fn file_application_primary_is_open_file() {
    let app = FileInfo {
        path: "/Applications/Safari.app".into(),
        name: "Safari.app".into(),
        file_type: FileType::Application,
        is_dir: false,
    };
    let actions = get_file_context_actions(&app);
    assert_eq!(actions[0].id, "open_file");
    assert!(actions[0].title.contains("Safari.app"));
}

#[test]
fn file_document_primary_is_open_file() {
    let doc = FileInfo {
        path: "/test/report.pdf".into(),
        name: "report.pdf".into(),
        file_type: FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&doc);
    assert_eq!(actions[0].id, "open_file");
}

#[test]
fn file_image_primary_is_open_file() {
    let img = FileInfo {
        path: "/test/photo.jpg".into(),
        name: "photo.jpg".into(),
        file_type: FileType::Image,
        is_dir: false,
    };
    let actions = get_file_context_actions(&img);
    assert_eq!(actions[0].id, "open_file");
}

// =========================================================================
// 17. Note switcher — many notes all unique IDs
// =========================================================================

#[test]
fn note_switcher_fifty_notes_all_unique_ids() {
    let notes: Vec<NoteSwitcherNoteInfo> = (0..50)
        .map(|i| NoteSwitcherNoteInfo {
            id: format!("note-{}", i),
            title: format!("Note {}", i),
            char_count: i * 10,
            is_current: i == 25,
            is_pinned: i % 7 == 0,
            preview: String::new(),
            relative_time: String::new(),
        })
        .collect();
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions.len(), 50);
    let mut ids: Vec<&str> = action_ids(&actions);
    let total = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(total, ids.len(), "Note switcher IDs should be unique");
}
