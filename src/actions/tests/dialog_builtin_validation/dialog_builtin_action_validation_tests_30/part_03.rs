
#[test]
fn batch30_new_chat_preset_desc_is_none() {
    let presets = vec![NewChatPresetInfo {
        id: "gen".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert!(actions[0].description.is_none());
}

#[test]
fn batch30_new_chat_last_used_desc_is_provider() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m".into(),
        display_name: "M".into(),
        provider: "p".into(),
        provider_display_name: "MyProvider".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].description.as_deref(), Some("MyProvider"));
}

#[test]
fn batch30_new_chat_model_desc_is_provider() {
    let models = vec![NewChatModelInfo {
        model_id: "m".into(),
        display_name: "M".into(),
        provider: "p".into(),
        provider_display_name: "ProvDisplay".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].description.as_deref(), Some("ProvDisplay"));
}

// ---------------------------------------------------------------------------
// 17. to_deeplink_name: various edge cases
// ---------------------------------------------------------------------------
#[test]
fn batch30_deeplink_name_unicode_preserved() {
    // alphanumeric unicode chars are preserved
    let result = to_deeplink_name("café");
    assert_eq!(result, "café");
}

#[test]
fn batch30_deeplink_name_all_special_chars() {
    let result = to_deeplink_name("!@#$%^");
    assert_eq!(result, "");
}

#[test]
fn batch30_deeplink_name_mixed_case_lowered() {
    let result = to_deeplink_name("MyScript");
    assert_eq!(result, "myscript");
}

#[test]
fn batch30_deeplink_name_numbers_preserved() {
    let result = to_deeplink_name("test123");
    assert_eq!(result, "test123");
}

// ---------------------------------------------------------------------------
// 18. Script context: action verb propagates to run_script title
// ---------------------------------------------------------------------------
#[test]
fn batch30_script_verb_run_default() {
    let script = crate::actions::types::ScriptInfo::new("foo", "/p/foo.ts");
    let actions = get_script_context_actions(&script);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert!(run.title.starts_with("Run "));
}

#[test]
fn batch30_script_verb_launch() {
    let script = crate::actions::types::ScriptInfo::with_action_verb(
        "Safari",
        "/Applications/Safari.app",
        false,
        "Launch",
    );
    let actions = get_script_context_actions(&script);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert!(run.title.starts_with("Launch "));
}

#[test]
fn batch30_script_verb_switch_to() {
    let script = crate::actions::types::ScriptInfo::with_action_verb(
        "Preview",
        "window:1",
        false,
        "Switch to",
    );
    let actions = get_script_context_actions(&script);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert!(run.title.starts_with("Switch to "));
}

#[test]
fn batch30_script_verb_desc_uses_verb() {
    let script = crate::actions::types::ScriptInfo::with_action_verb("X", "/p", false, "Open");
    let actions = get_script_context_actions(&script);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert_eq!(run.description.as_deref(), Some("Open this item"));
}

// ---------------------------------------------------------------------------
// 19. Script context: deeplink URL in copy_deeplink description
// ---------------------------------------------------------------------------
#[test]
fn batch30_deeplink_desc_contains_url() {
    let script = crate::actions::types::ScriptInfo::new("My Cool Script", "/p.ts");
    let actions = get_script_context_actions(&script);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert!(dl
        .description
        .as_ref()
        .unwrap()
        .contains("scriptkit://run/my-cool-script"));
}

#[test]
fn batch30_deeplink_shortcut_is_cmd_shift_d() {
    let script = crate::actions::types::ScriptInfo::new("X", "/p.ts");
    let actions = get_script_context_actions(&script);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert_eq!(dl.shortcut.as_deref(), Some("⌘⇧D"));
}

#[test]
fn batch30_deeplink_desc_for_builtin() {
    let script = crate::actions::types::ScriptInfo::builtin("Clipboard History");
    let actions = get_script_context_actions(&script);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert!(dl
        .description
        .as_ref()
        .unwrap()
        .contains("scriptkit://run/clipboard-history"));
}

// ---------------------------------------------------------------------------
// 20. CommandBarConfig: notes_style matches expected fields
// ---------------------------------------------------------------------------
#[test]
fn batch30_command_bar_notes_style_search_top() {
    let cfg = CommandBarConfig::notes_style();
    assert!(matches!(
        cfg.dialog_config.search_position,
        SearchPosition::Top
    ));
}

#[test]
fn batch30_command_bar_notes_style_section_separators() {
    let cfg = CommandBarConfig::notes_style();
    assert!(matches!(
        cfg.dialog_config.section_style,
        SectionStyle::Separators
    ));
}

#[test]
fn batch30_command_bar_notes_style_anchor_top() {
    let cfg = CommandBarConfig::notes_style();
    assert!(matches!(cfg.dialog_config.anchor, AnchorPosition::Top));
}

#[test]
fn batch30_command_bar_notes_style_show_icons_and_footer() {
    let cfg = CommandBarConfig::notes_style();
    assert!(cfg.dialog_config.show_icons);
    assert!(cfg.dialog_config.show_footer);
}

// ---------------------------------------------------------------------------
// 21. parse_shortcut_keycaps: modifier+letter combos
// ---------------------------------------------------------------------------
#[test]
fn batch30_parse_keycaps_cmd_c() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘C");
    assert_eq!(caps, vec!["⌘", "C"]);
}

#[test]
fn batch30_parse_keycaps_cmd_shift_a() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘⇧A");
    assert_eq!(caps, vec!["⌘", "⇧", "A"]);
}

#[test]
fn batch30_parse_keycaps_enter_alone() {
    let caps = ActionsDialog::parse_shortcut_keycaps("↵");
    assert_eq!(caps, vec!["↵"]);
}

#[test]
fn batch30_parse_keycaps_all_modifiers_plus_key() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌃⌥⇧⌘K");
    assert_eq!(caps, vec!["⌃", "⌥", "⇧", "⌘", "K"]);
}

// ---------------------------------------------------------------------------
// 22. score_action: various match scenarios
// ---------------------------------------------------------------------------
#[test]
fn batch30_score_prefix_match_gte_100() {
    let action = Action::new(
        "e",
        "Edit Script",
        Some("Open in $EDITOR".into()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "edit");
    assert!(score >= 100, "Prefix match should be ≥100, got {}", score);
}

#[test]
fn batch30_score_contains_match_50_to_99() {
    let action = Action::new("c", "Copy Edit Path", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "edit");
    assert!(
        (50..100).contains(&score),
        "Contains match should be 50..99, got {}",
        score
    );
}

#[test]
fn batch30_score_no_match_is_zero() {
    let action = Action::new("x", "Run Script", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "zzz");
    assert_eq!(score, 0);
}

#[test]
fn batch30_score_empty_search_is_prefix() {
    let action = Action::new("x", "Hello", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "");
    assert!(score >= 100, "Empty search is prefix match, got {}", score);
}

// ---------------------------------------------------------------------------
// 23. fuzzy_match: edge cases
// ---------------------------------------------------------------------------
#[test]
fn batch30_fuzzy_exact_match() {
    assert!(ActionsDialog::fuzzy_match("hello", "hello"));
}

#[test]
fn batch30_fuzzy_subsequence() {
    assert!(ActionsDialog::fuzzy_match("hello world", "hwo"));
}

#[test]
fn batch30_fuzzy_no_match() {
    assert!(!ActionsDialog::fuzzy_match("abc", "xyz"));
}

#[test]
fn batch30_fuzzy_needle_longer_than_haystack() {
    assert!(!ActionsDialog::fuzzy_match("ab", "abcdef"));
}

#[test]
fn batch30_fuzzy_empty_needle() {
    assert!(ActionsDialog::fuzzy_match("anything", ""));
}

// ---------------------------------------------------------------------------
// 24. build_grouped_items_static: Headers vs Separators behavior
// ---------------------------------------------------------------------------
#[test]
fn batch30_grouped_headers_adds_section_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered = vec![0, 1];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // Should be: Header("S1"), Item(0), Header("S2"), Item(1) = 4 items
    assert_eq!(grouped.len(), 4);
}

#[test]
fn batch30_grouped_separators_no_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered = vec![0, 1];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    // Should be: Item(0), Item(1) = 2 items (no headers)
    assert_eq!(grouped.len(), 2);
}

#[test]
fn batch30_grouped_empty_returns_empty() {
    let grouped = build_grouped_items_static(&[], &[], SectionStyle::Headers);
    assert!(grouped.is_empty());
}

#[test]
fn batch30_grouped_same_section_one_header() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S"),
    ];
    let filtered = vec![0, 1];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // Should be: Header("S"), Item(0), Item(1) = 3 items
    assert_eq!(grouped.len(), 3);
}

// ---------------------------------------------------------------------------
// 25. coerce_action_selection: header skipping
// ---------------------------------------------------------------------------
#[test]
fn batch30_coerce_on_item_stays() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
}

#[test]
fn batch30_coerce_on_header_jumps_down() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![
        GroupedActionItem::SectionHeader("H".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn batch30_coerce_trailing_header_jumps_up() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("H".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn batch30_coerce_all_headers_none() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![
        GroupedActionItem::SectionHeader("H1".into()),
        GroupedActionItem::SectionHeader("H2".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn batch30_coerce_empty_none() {
    assert_eq!(coerce_action_selection(&[], 0), None);
}

// ---------------------------------------------------------------------------
// 26. Clipboard: destructive actions ordering invariant
// ---------------------------------------------------------------------------
#[test]
fn batch30_clipboard_destructive_always_last_three() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let len = actions.len();
    assert!(len >= 3);
    assert_eq!(actions[len - 3].id, "clipboard_delete");
    assert_eq!(actions[len - 2].id, "clipboard_delete_multiple");
    assert_eq!(actions[len - 1].id, "clipboard_delete_all");
}

#[test]
fn batch30_clipboard_image_destructive_also_last_three() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "i".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let len = actions.len();
    assert!(len >= 3);
    assert_eq!(actions[len - 3].id, "clipboard_delete");
    assert_eq!(actions[len - 2].id, "clipboard_delete_multiple");
    assert_eq!(actions[len - 1].id, "clipboard_delete_all");
}

#[test]
fn batch30_clipboard_delete_all_desc_mentions_pinned() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let da = actions
        .iter()
        .find(|a| a.id == "clipboard_delete_all")
        .unwrap();
    assert!(da
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("pinned"));
}

// ---------------------------------------------------------------------------
// 27. Script context: agent has specific action set
// ---------------------------------------------------------------------------
#[test]
fn batch30_agent_has_edit_script_title_edit_agent() {
    let mut script = crate::actions::types::ScriptInfo::new("a", "/p");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.title, "Edit Agent");
}

#[test]
fn batch30_agent_has_no_view_logs() {
    let mut script = crate::actions::types::ScriptInfo::new("a", "/p");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}
