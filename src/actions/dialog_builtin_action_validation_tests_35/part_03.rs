
#[test]
fn new_chat_only_presets() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "preset_general");
}

#[test]
fn new_chat_only_last_used() {
    let last_used = vec![NewChatModelInfo {
        model_id: "lu1".into(),
        display_name: "Last Used 1".into(),
        provider: "p".into(),
        provider_display_name: "Provider".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "last_used_0");
}

// =====================================================================
// 16. to_deeplink_name: additional transformations
// =====================================================================

#[test]
fn deeplink_name_preserves_numbers() {
    assert_eq!(to_deeplink_name("Script 123"), "script-123");
}

#[test]
fn deeplink_name_emoji_to_hyphens() {
    // Emojis are non-alphanumeric so they become hyphens (then collapse)
    assert_eq!(to_deeplink_name("Cool Script"), "cool-script");
}

#[test]
fn deeplink_name_already_lowercase() {
    assert_eq!(to_deeplink_name("already-lowercase"), "already-lowercase");
}

#[test]
fn deeplink_name_single_char() {
    assert_eq!(to_deeplink_name("A"), "a");
}

// =====================================================================
// 17. Constants: secondary dimension values
// =====================================================================

#[test]
fn constant_section_header_height() {
    assert_eq!(SECTION_HEADER_HEIGHT, 22.0);
}

#[test]
fn constant_header_height() {
    assert_eq!(HEADER_HEIGHT, 24.0);
}

#[test]
fn constant_action_row_inset() {
    assert_eq!(ACTION_ROW_INSET, 6.0);
}

#[test]
fn constant_selection_radius() {
    assert_eq!(SELECTION_RADIUS, 8.0);
}

// =====================================================================
// 18. Constants: keycap and accent bar
// =====================================================================

#[test]
fn constant_keycap_min_width() {
    assert_eq!(KEYCAP_MIN_WIDTH, 22.0);
}

#[test]
fn constant_keycap_height() {
    assert_eq!(KEYCAP_HEIGHT, 22.0);
}

#[test]
fn constant_accent_bar_width() {
    assert_eq!(ACCENT_BAR_WIDTH, 3.0);
}

#[test]
fn constant_search_input_height() {
    assert_eq!(SEARCH_INPUT_HEIGHT, 44.0);
}

// =====================================================================
// 19. parse_shortcut_keycaps: modifier and special key parsing
// =====================================================================

#[test]
fn parse_keycaps_cmd_enter() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘↵");
    assert_eq!(caps, vec!["⌘", "↵"]);
}

#[test]
fn parse_keycaps_all_modifiers_and_key() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘⇧⌃⌥K");
    assert_eq!(caps, vec!["⌘", "⇧", "⌃", "⌥", "K"]);
}

#[test]
fn parse_keycaps_single_letter() {
    let caps = ActionsDialog::parse_shortcut_keycaps("A");
    assert_eq!(caps, vec!["A"]);
}

#[test]
fn parse_keycaps_lowercase_uppercased() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘c");
    assert_eq!(caps, vec!["⌘", "C"]);
}

// =====================================================================
// 20. format_shortcut_hint: additional conversions
// =====================================================================

#[test]
fn format_shortcut_hint_cmd_backspace() {
    assert_eq!(ActionsDialog::format_shortcut_hint("cmd+backspace"), "⌘⌫");
}

#[test]
fn format_shortcut_hint_ctrl_tab() {
    assert_eq!(ActionsDialog::format_shortcut_hint("ctrl+tab"), "⌃⇥");
}

#[test]
fn format_shortcut_hint_option_space() {
    assert_eq!(ActionsDialog::format_shortcut_hint("option+space"), "⌥␣");
}

#[test]
fn format_shortcut_hint_single_escape() {
    assert_eq!(ActionsDialog::format_shortcut_hint("escape"), "⎋");
}

// =====================================================================
// 21. build_grouped_items_static: None section handling
// =====================================================================

#[test]
fn grouped_items_none_section_no_header() {
    let actions = vec![
        Action::new("a", "Alpha", None, ActionCategory::ScriptContext),
        Action::new("b", "Beta", None, ActionCategory::ScriptContext),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // No sections on actions → no headers added
    assert_eq!(grouped.len(), 2);
}

#[test]
fn grouped_items_mixed_some_none_sections() {
    let mut a1 = Action::new("a", "Alpha", None, ActionCategory::ScriptContext);
    a1.section = Some("Group A".into());
    let a2 = Action::new("b", "Beta", None, ActionCategory::ScriptContext);
    // a2 has no section
    let actions = vec![a1, a2];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // One header for "Group A", then item, then item (no header for None section)
    assert_eq!(grouped.len(), 3);
}

#[test]
fn grouped_items_separators_never_adds_headers() {
    let mut a1 = Action::new("a", "Alpha", None, ActionCategory::ScriptContext);
    a1.section = Some("Group A".into());
    let mut a2 = Action::new("b", "Beta", None, ActionCategory::ScriptContext);
    a2.section = Some("Group B".into());
    let actions = vec![a1, a2];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    // Separators style never adds headers
    assert_eq!(grouped.len(), 2);
}

#[test]
fn grouped_items_empty_filtered_returns_empty() {
    let actions = vec![Action::new(
        "a",
        "Alpha",
        None,
        ActionCategory::ScriptContext,
    )];
    let filtered: Vec<usize> = vec![];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    assert!(grouped.is_empty());
}

// =====================================================================
// 22. coerce_action_selection: specific patterns
// =====================================================================

#[test]
fn coerce_selection_single_item() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![GroupedActionItem::Item(0)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
}

#[test]
fn coerce_selection_header_then_item() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![
        GroupedActionItem::SectionHeader("H".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn coerce_selection_item_then_header() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("H".into()),
    ];
    // On header at index 1 → search down (nothing) → search up → find item at 0
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn coerce_selection_beyond_bounds_clamped() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
    // Index 99 → clamped to len-1=1 → Item(1) → Some(1)
    assert_eq!(coerce_action_selection(&rows, 99), Some(1));
}

// =====================================================================
// 23. CommandBarConfig: close flags consistent
// =====================================================================

#[test]
fn command_bar_ai_close_on_select_true() {
    let config = CommandBarConfig::ai_style();
    assert!(config.close_on_select);
}

#[test]
fn command_bar_ai_close_on_escape_true() {
    let config = CommandBarConfig::ai_style();
    assert!(config.close_on_escape);
}

#[test]
fn command_bar_main_menu_close_on_select_true() {
    let config = CommandBarConfig::main_menu_style();
    assert!(config.close_on_select);
}

#[test]
fn command_bar_notes_close_on_escape_true() {
    let config = CommandBarConfig::notes_style();
    assert!(config.close_on_escape);
}

// =====================================================================
// 24. Script context: scriptlet reveal and copy_path details
// =====================================================================

#[test]
fn scriptlet_reveal_shortcut_cmd_shift_f() {
    let scriptlet = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_script_context_actions(&scriptlet);
    let reveal = actions
        .iter()
        .find(|a| a.id == "reveal_scriptlet_in_finder")
        .unwrap();
    assert_eq!(reveal.shortcut.as_ref().unwrap(), "⌘⇧F");
}

#[test]
fn scriptlet_reveal_desc_mentions_finder() {
    let scriptlet = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_script_context_actions(&scriptlet);
    let reveal = actions
        .iter()
        .find(|a| a.id == "reveal_scriptlet_in_finder")
        .unwrap();
    assert!(reveal.description.as_ref().unwrap().contains("Finder"));
}

#[test]
fn scriptlet_copy_path_shortcut_cmd_shift_c() {
    let scriptlet = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_script_context_actions(&scriptlet);
    let cp = actions
        .iter()
        .find(|a| a.id == "copy_scriptlet_path")
        .unwrap();
    assert_eq!(cp.shortcut.as_ref().unwrap(), "⌘⇧C");
}

#[test]
fn scriptlet_copy_path_desc_mentions_clipboard() {
    let scriptlet = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_script_context_actions(&scriptlet);
    let cp = actions
        .iter()
        .find(|a| a.id == "copy_scriptlet_path")
        .unwrap();
    assert!(cp
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("path"));
}

// =====================================================================
// 25. Score action: fuzzy match scores lower than prefix/contains
// =====================================================================

#[test]
fn score_action_fuzzy_lower_than_prefix() {
    let action = Action::new("test", "Edit Script", None, ActionCategory::ScriptContext);
    let prefix_score = ActionsDialog::score_action(&action, "edit");
    let fuzzy_score = ActionsDialog::score_action(&action, "eds"); // e-d-i-t s-c-r-i-p-t has e,d,s
    assert!(prefix_score > fuzzy_score);
}

#[test]
fn score_action_contains_lower_than_prefix() {
    let action = Action::new(
        "test",
        "My Edit Script",
        None,
        ActionCategory::ScriptContext,
    );
    let prefix_score = ActionsDialog::score_action(&action, "my");
    let contains_score = ActionsDialog::score_action(&action, "edit");
    assert!(prefix_score > contains_score);
}

#[test]
fn score_action_both_title_and_desc_match() {
    let action = Action::new(
        "test",
        "Edit Script",
        Some("Edit the script file".into()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "edit");
    // prefix(100) + desc(15) = 115
    assert!(score >= 115);
}

#[test]
fn score_action_shortcut_bonus() {
    let action =
        Action::new("test", "Zzz", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
    let score = ActionsDialog::score_action(&action, "⌘e");
    // No title match but shortcut contains "⌘e" → 10
    assert!(score >= 10);
}

// =====================================================================
// 26. fuzzy_match: additional patterns
// =====================================================================

#[test]
fn fuzzy_match_exact() {
    assert!(ActionsDialog::fuzzy_match("hello", "hello"));
}

#[test]
fn fuzzy_match_subsequence() {
    assert!(ActionsDialog::fuzzy_match("hello world", "hlwrd"));
}

#[test]
fn fuzzy_match_no_match() {
    assert!(!ActionsDialog::fuzzy_match("abc", "xyz"));
}

#[test]
fn fuzzy_match_needle_longer_than_haystack() {
    assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
}

// =====================================================================
// 27. Cross-context: all contexts produce non-empty actions
// =====================================================================

#[test]
fn cross_context_script_non_empty() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    assert!(!get_script_context_actions(&script).is_empty());
}

#[test]
fn cross_context_builtin_non_empty() {
    let builtin = ScriptInfo::builtin("Clipboard History");
    assert!(!get_script_context_actions(&builtin).is_empty());
}

#[test]
fn cross_context_scriptlet_non_empty() {
    let scriptlet = ScriptInfo::scriptlet("Open URL", "/p.md", None, None);
    assert!(!get_script_context_actions(&scriptlet).is_empty());
}

#[test]
fn cross_context_file_non_empty() {
    let f = FileInfo {
        path: "/t.txt".into(),
        name: "t.txt".into(),
        file_type: crate::file_search::FileType::File,
        is_dir: false,
    };
    assert!(!get_file_context_actions(&f).is_empty());
}

#[test]
fn cross_context_path_non_empty() {
    let p = PathInfo {
        path: "/t".into(),
        name: "t".into(),
        is_dir: false,
    };
    assert!(!get_path_context_actions(&p).is_empty());
}

#[test]
fn cross_context_ai_bar_non_empty() {
    assert!(!get_ai_command_bar_actions().is_empty());
}
