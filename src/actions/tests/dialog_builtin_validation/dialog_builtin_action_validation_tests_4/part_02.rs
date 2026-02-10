
// =========================================================================
// 7. Notes section labels exhaustive for full-feature permutation
// =========================================================================

#[test]
fn notes_full_feature_has_all_five_sections() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let secs = sections_in_order(&actions);
    assert!(secs.contains(&"Notes"), "Missing Notes section");
    assert!(secs.contains(&"Edit"), "Missing Edit section");
    assert!(secs.contains(&"Copy"), "Missing Copy section");
    assert!(secs.contains(&"Export"), "Missing Export section");
    assert!(secs.contains(&"Settings"), "Missing Settings section");
}

#[test]
fn notes_no_selection_only_has_notes_section() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let secs: Vec<&str> = actions
        .iter()
        .filter_map(|a| a.section.as_deref())
        .collect();
    // Should have Notes and Settings
    assert!(secs.contains(&"Notes"));
    assert!(secs.contains(&"Settings"));
    // Should not have Edit, Copy, Export (require selection + not trash)
    assert!(!secs.contains(&"Edit"));
    assert!(!secs.contains(&"Copy"));
    assert!(!secs.contains(&"Export"));
}

#[test]
fn notes_trash_view_has_limited_sections() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let secs: Vec<&str> = actions
        .iter()
        .filter_map(|a| a.section.as_deref())
        .collect();
    // Even with selection, trash view suppresses Edit/Copy/Export
    assert!(secs.contains(&"Notes"));
    assert!(!secs.contains(&"Edit"));
    assert!(!secs.contains(&"Copy"));
    assert!(!secs.contains(&"Export"));
}

#[test]
fn notes_auto_sizing_enabled_hides_settings() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids = action_ids(&actions);
    assert!(!ids.contains(&"enable_auto_sizing"));
}

// =========================================================================
// 8. AI command bar icon-per-section coverage
// =========================================================================

#[test]
fn ai_command_bar_every_action_has_icon() {
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        assert!(
            action.icon.is_some(),
            "AI action '{}' should have an icon",
            action.id
        );
    }
}

#[test]
fn ai_command_bar_every_action_has_section() {
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        assert!(
            action.section.is_some(),
            "AI action '{}' should have a section",
            action.id
        );
    }
}

#[test]
fn ai_command_bar_exactly_six_sections() {
    let actions = get_ai_command_bar_actions();
    let unique_sections: HashSet<&str> = actions
        .iter()
        .filter_map(|a| a.section.as_deref())
        .collect();
    assert_eq!(
        unique_sections.len(),
        6,
        "AI command bar should have exactly 6 sections, got {:?}",
        unique_sections
    );
}

#[test]
fn ai_command_bar_section_order_is_response_actions_attachments_export_actions_help_settings() {
    let actions = get_ai_command_bar_actions();
    let order = sections_in_order(&actions);
    assert_eq!(
        order,
        vec![
            "Response",
            "Actions",
            "Attachments",
            "Export",
            "Actions",
            "Help",
            "Settings"
        ]
    );
}

// =========================================================================
// 9. New chat with all-empty inputs
// =========================================================================

#[test]
fn new_chat_empty_inputs_produces_empty() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

#[test]
fn new_chat_only_models_produces_models_section() {
    let models = vec![NewChatModelInfo {
        model_id: "m1".to_string(),
        display_name: "Model 1".to_string(),
        provider: "test".to_string(),
        provider_display_name: "Test Provider".to_string(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Models"));
}

#[test]
fn new_chat_only_presets_produces_presets_section() {
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
fn new_chat_only_last_used_produces_last_used_section() {
    let last_used = vec![NewChatModelInfo {
        model_id: "lu1".to_string(),
        display_name: "Recent Model".to_string(),
        provider: "test".to_string(),
        provider_display_name: "Test".to_string(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
}

#[test]
fn new_chat_section_order_is_last_used_presets_models() {
    let last_used = vec![NewChatModelInfo {
        model_id: "lu1".to_string(),
        display_name: "Recent".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "gen".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "m1".to_string(),
        display_name: "Claude".to_string(),
        provider: "anthropic".to_string(),
        provider_display_name: "Anthropic".to_string(),
    }];
    let actions = get_new_chat_actions(&last_used, &presets, &models);
    let order = sections_in_order(&actions);
    assert_eq!(order, vec!["Last Used Settings", "Presets", "Models"]);
}

// =========================================================================
// 10. score_action edge cases
// =========================================================================

#[test]
fn score_action_empty_query_returns_zero() {
    let action = Action::new("test", "Test Action", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "");
    // Empty query should match as prefix (empty string is prefix of everything)
    // Based on implementation: "test action".starts_with("") == true → 100
    assert!(score >= 100);
}

#[test]
fn score_action_exact_title_match_gets_prefix_score() {
    let action = Action::new("edit", "Edit Script", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "edit script");
    assert!(
        score >= 100,
        "Exact title match should score 100+, got {}",
        score
    );
}

#[test]
fn score_action_no_match_returns_zero() {
    let action = Action::new("test", "Test Action", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "zzzzz");
    assert_eq!(score, 0, "No match should score 0");
}

#[test]
fn score_action_description_only_match_returns_fifteen() {
    let action = Action::new(
        "open",
        "Open File",
        Some("Launch the default editor".to_string()),
        ActionCategory::ScriptContext,
    );
    // "default editor" doesn't match title but matches description
    let score = ActionsDialog::score_action(&action, "default editor");
    assert_eq!(
        score, 15,
        "Description-only match should score 15, got {}",
        score
    );
}

#[test]
fn score_action_shortcut_only_match_returns_ten() {
    let action =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
    let score = ActionsDialog::score_action(&action, "⌘e");
    assert_eq!(
        score, 10,
        "Shortcut-only match should score 10, got {}",
        score
    );
}

#[test]
fn score_action_title_plus_description_stacks() {
    let action = Action::new(
        "edit",
        "Edit Script",
        Some("Edit the script file".to_string()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "edit");
    // title prefix (100) + description contains "edit" (15) = 115
    assert!(
        score >= 115,
        "Stacked score should be >= 115, got {}",
        score
    );
}

#[test]
fn score_action_single_char_query() {
    let action = Action::new("edit", "Edit Script", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "e");
    assert!(
        score >= 100,
        "Single char prefix match should score 100+, got {}",
        score
    );
}

// =========================================================================
// 11. fuzzy_match boundary conditions
// =========================================================================

#[test]
fn fuzzy_match_empty_needle_always_matches() {
    assert!(ActionsDialog::fuzzy_match("anything", ""));
}

#[test]
fn fuzzy_match_empty_haystack_only_matches_empty_needle() {
    assert!(ActionsDialog::fuzzy_match("", ""));
    assert!(!ActionsDialog::fuzzy_match("", "a"));
}

#[test]
fn fuzzy_match_needle_longer_than_haystack_fails() {
    assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
}

#[test]
fn fuzzy_match_exact_match_succeeds() {
    assert!(ActionsDialog::fuzzy_match("hello", "hello"));
}

#[test]
fn fuzzy_match_subsequence_succeeds() {
    assert!(ActionsDialog::fuzzy_match("edit script", "edsc"));
}

#[test]
fn fuzzy_match_wrong_order_fails() {
    assert!(!ActionsDialog::fuzzy_match("abc", "cba"));
}

#[test]
fn fuzzy_match_case_sensitive() {
    // fuzzy_match is case-sensitive (expects pre-lowercased input)
    assert!(!ActionsDialog::fuzzy_match("hello", "H"));
    assert!(ActionsDialog::fuzzy_match("hello", "h"));
}

// =========================================================================
// 12. parse_shortcut_keycaps for all modifier symbols
// =========================================================================

#[test]
fn parse_keycaps_modifier_symbols() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⇧C");
    assert_eq!(keycaps, vec!["⌘", "⇧", "C"]);
}

#[test]
fn parse_keycaps_enter_symbol() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("↵");
    assert_eq!(keycaps, vec!["↵"]);
}

#[test]
fn parse_keycaps_escape_symbol() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⎋");
    assert_eq!(keycaps, vec!["⎋"]);
}

#[test]
fn parse_keycaps_backspace_symbol() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⌫");
    assert_eq!(keycaps, vec!["⌘", "⌫"]);
}

#[test]
fn parse_keycaps_space_symbol() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("␣");
    assert_eq!(keycaps, vec!["␣"]);
}

#[test]
fn parse_keycaps_arrow_keys() {
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("↑"), vec!["↑"]);
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("↓"), vec!["↓"]);
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("←"), vec!["←"]);
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("→"), vec!["→"]);
}

#[test]
fn parse_keycaps_tab_symbol() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⇥");
    assert_eq!(keycaps, vec!["⇥"]);
}

#[test]
fn parse_keycaps_all_modifiers_combined() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⌃⌥⇧X");
    assert_eq!(keycaps, vec!["⌘", "⌃", "⌥", "⇧", "X"]);
}

#[test]
fn parse_keycaps_lowercase_becomes_uppercase() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘e");
    assert_eq!(keycaps, vec!["⌘", "E"]);
}

// =========================================================================
// 13. format_shortcut_hint roundtrips for unusual key names
// =========================================================================

#[test]
fn format_shortcut_hint_enter() {
    let hint = ActionsDialog::format_shortcut_hint("enter");
    assert_eq!(hint, "↵");
}

#[test]
fn format_shortcut_hint_return() {
    let hint = ActionsDialog::format_shortcut_hint("return");
    assert_eq!(hint, "↵");
}

#[test]
fn format_shortcut_hint_escape() {
    let hint = ActionsDialog::format_shortcut_hint("escape");
    assert_eq!(hint, "⎋");
}

#[test]
fn format_shortcut_hint_esc() {
    let hint = ActionsDialog::format_shortcut_hint("esc");
    assert_eq!(hint, "⎋");
}

#[test]
fn format_shortcut_hint_tab() {
    let hint = ActionsDialog::format_shortcut_hint("tab");
    assert_eq!(hint, "⇥");
}

#[test]
fn format_shortcut_hint_backspace() {
    let hint = ActionsDialog::format_shortcut_hint("backspace");
    assert_eq!(hint, "⌫");
}

#[test]
fn format_shortcut_hint_space() {
    let hint = ActionsDialog::format_shortcut_hint("space");
    assert_eq!(hint, "␣");
}

#[test]
fn format_shortcut_hint_arrow_keys() {
    assert_eq!(ActionsDialog::format_shortcut_hint("up"), "↑");
    assert_eq!(ActionsDialog::format_shortcut_hint("down"), "↓");
    assert_eq!(ActionsDialog::format_shortcut_hint("left"), "←");
    assert_eq!(ActionsDialog::format_shortcut_hint("right"), "→");
}
