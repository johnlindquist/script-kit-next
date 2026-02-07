
#[test]
fn chat_copy_response_only_with_response() {
    let without = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions_without = get_chat_context_actions(&without);
    assert!(
        !actions_without.iter().any(|a| a.id == "copy_response"),
        "copy_response should be absent without response"
    );

    let with = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: true,
    };
    let actions_with = get_chat_context_actions(&with);
    assert!(
        actions_with.iter().any(|a| a.id == "copy_response"),
        "copy_response should be present with response"
    );
}

#[test]
fn chat_clear_conversation_only_with_messages() {
    let without = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions_without = get_chat_context_actions(&without);
    assert!(!actions_without.iter().any(|a| a.id == "clear_conversation"),);

    let with = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions_with = get_chat_context_actions(&with);
    assert!(actions_with.iter().any(|a| a.id == "clear_conversation"));
}

// =========================================================================
// 18. Notes conditional action counts across all 8 permutations
//     (has_selection × is_trash × auto_sizing)
// =========================================================================

#[test]
fn notes_8_permutations_action_counts() {
    let bools = [false, true];
    for &sel in &bools {
        for &trash in &bools {
            for &auto in &bools {
                let info = NotesInfo {
                    has_selection: sel,
                    is_trash_view: trash,
                    auto_sizing_enabled: auto,
                };
                let actions = get_notes_command_bar_actions(&info);

                // new_note and browse_notes always present
                assert!(
                    actions.iter().any(|a| a.id == "new_note"),
                    "new_note always present (sel={}, trash={}, auto={})",
                    sel,
                    trash,
                    auto
                );
                assert!(
                    actions.iter().any(|a| a.id == "browse_notes"),
                    "browse_notes always present (sel={}, trash={}, auto={})",
                    sel,
                    trash,
                    auto
                );

                // Conditional: duplicate, find, format, copy, export
                // only when has_selection && !is_trash_view
                let has_conditionals = sel && !trash;
                let conditional_ids = [
                    "duplicate_note",
                    "find_in_note",
                    "format",
                    "copy_note_as",
                    "copy_deeplink",
                    "create_quicklink",
                    "export",
                ];
                for id in &conditional_ids {
                    assert_eq!(
                        actions.iter().any(|a| a.id == *id),
                        has_conditionals,
                        "Action '{}' should {} when sel={}, trash={}, auto={}",
                        id,
                        if has_conditionals {
                            "be present"
                        } else {
                            "be absent"
                        },
                        sel,
                        trash,
                        auto
                    );
                }

                // enable_auto_sizing only when auto_sizing_enabled is false
                assert_eq!(
                    actions.iter().any(|a| a.id == "enable_auto_sizing"),
                    !auto,
                    "enable_auto_sizing should {} when auto={}",
                    if !auto { "be present" } else { "be absent" },
                    auto
                );
            }
        }
    }
}

// =========================================================================
// 19. CommandBarConfig notes_style specifics
// =========================================================================

#[test]
fn command_bar_notes_style_search_top_separators_icons() {
    let config = CommandBarConfig::notes_style();
    assert!(
        matches!(config.dialog_config.search_position, SearchPosition::Top),
        "notes_style should have search at top"
    );
    assert!(
        matches!(config.dialog_config.section_style, SectionStyle::Separators),
        "notes_style should use Separators"
    );
    assert!(
        config.dialog_config.show_icons,
        "notes_style should show icons"
    );
    assert!(
        config.dialog_config.show_footer,
        "notes_style should show footer"
    );
    assert!(config.close_on_escape);
    assert!(config.close_on_select);
    assert!(config.close_on_click_outside);
}

// =========================================================================
// 20. Grouped items build correctness
// =========================================================================

#[test]
fn grouped_items_headers_style_produces_section_headers() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);

    // Should contain at least one SectionHeader
    let header_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
        .count();
    assert!(
        header_count >= 7,
        "Headers style should produce at least 7 section headers, got {}",
        header_count
    );
}

#[test]
fn grouped_items_none_style_has_no_headers() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);

    let header_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(
        header_count, 0,
        "None style should produce no section headers"
    );
}

#[test]
fn grouped_items_separators_style_has_no_headers() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);

    let header_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(
        header_count, 0,
        "Separators style should produce no section headers"
    );
}

#[test]
fn grouped_items_empty_filtered_returns_empty() {
    let actions = get_ai_command_bar_actions();
    let grouped = build_grouped_items_static(&actions, &[], SectionStyle::Headers);
    assert!(grouped.is_empty());
}

// =========================================================================
// 21. Coerce action selection correctness
// =========================================================================

#[test]
fn coerce_selection_on_item_returns_same_index() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::Item(1),
        GroupedActionItem::Item(2),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(1));
}

#[test]
fn coerce_selection_on_header_skips_to_next_item() {
    let rows = vec![
        GroupedActionItem::SectionHeader("Section".into()),
        GroupedActionItem::Item(0),
        GroupedActionItem::Item(1),
    ];
    // Index 0 is a header, should coerce to index 1
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn coerce_selection_on_trailing_header_goes_up() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("Section".into()),
    ];
    // Index 1 is a header at the end, should coerce back to index 0
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn coerce_selection_empty_returns_none() {
    assert_eq!(coerce_action_selection(&[], 0), None);
}

#[test]
fn coerce_selection_all_headers_returns_none() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".into()),
        GroupedActionItem::SectionHeader("B".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

// =========================================================================
// 22. Action cached lowercase fields consistency
// =========================================================================

#[test]
fn action_title_lower_matches_title() {
    let action = Action::new(
        "test",
        "My Title With CAPS",
        Some("Description HERE".to_string()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘⇧C");

    assert_eq!(action.title_lower, "my title with caps");
    assert_eq!(
        action.description_lower,
        Some("description here".to_string())
    );
    assert_eq!(action.shortcut_lower, Some("⌘⇧c".to_string()));
}

#[test]
fn all_script_actions_have_consistent_lowercase_caches() {
    let script = ScriptInfo::new("Test Script", "/path/test.ts");
    for a in &get_script_context_actions(&script) {
        assert_eq!(
            a.title_lower,
            a.title.to_lowercase(),
            "title_lower mismatch for '{}'",
            a.id
        );
        if let Some(ref desc) = a.description {
            assert_eq!(
                a.description_lower.as_deref(),
                Some(desc.to_lowercase()).as_deref(),
                "description_lower mismatch for '{}'",
                a.id
            );
        }
        if let Some(ref sc) = a.shortcut {
            assert_eq!(
                a.shortcut_lower.as_deref(),
                Some(sc.to_lowercase()).as_deref(),
                "shortcut_lower mismatch for '{}'",
                a.id
            );
        }
    }
}

#[test]
fn all_clipboard_actions_have_consistent_lowercase_caches() {
    let entry = ClipboardEntryInfo {
        id: "lc".into(),
        content_type: ContentType::Image,
        pinned: true,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: Some("Arc".into()),
    };
    for a in &get_clipboard_history_context_actions(&entry) {
        assert_eq!(
            a.title_lower,
            a.title.to_lowercase(),
            "title_lower mismatch for '{}'",
            a.id
        );
    }
}

#[test]
fn all_ai_command_bar_actions_have_consistent_lowercase_caches() {
    for a in &get_ai_command_bar_actions() {
        assert_eq!(a.title_lower, a.title.to_lowercase());
        if let Some(ref desc) = a.description {
            assert_eq!(
                a.description_lower.as_deref(),
                Some(desc.to_lowercase()).as_deref()
            );
        }
    }
}

// =========================================================================
// 23. New chat action descriptions
// =========================================================================

#[test]
fn new_chat_last_used_has_provider_description() {
    let last_used = vec![NewChatModelInfo {
        model_id: "claude-3".into(),
        display_name: "Claude 3".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    let a = &actions[0];
    assert_eq!(a.description.as_deref(), Some("Anthropic"));
}

#[test]
fn new_chat_presets_have_no_description() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Settings,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    let a = &actions[0];
    assert!(
        a.description.is_none(),
        "Presets should have no description"
    );
}

#[test]
fn new_chat_models_have_provider_description() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt-4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    let a = &actions[0];
    assert_eq!(a.description.as_deref(), Some("OpenAI"));
}

// =========================================================================
// 24. New chat action ID format
// =========================================================================

#[test]
fn new_chat_last_used_ids_are_indexed() {
    let last_used = vec![
        NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "M1".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        },
        NewChatModelInfo {
            model_id: "m2".into(),
            display_name: "M2".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        },
    ];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].id, "last_used_0");
    assert_eq!(actions[1].id, "last_used_1");
}

#[test]
fn new_chat_preset_ids_use_preset_id() {
    let presets = vec![
        NewChatPresetInfo {
            id: "gen".into(),
            name: "General".into(),
            icon: IconName::Settings,
        },
        NewChatPresetInfo {
            id: "code".into(),
            name: "Code".into(),
            icon: IconName::Code,
        },
    ];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].id, "preset_gen");
    assert_eq!(actions[1].id, "preset_code");
}
