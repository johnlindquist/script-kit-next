
#[test]
fn cat18_empty_preview_with_time_uses_time() {
    let note = NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 0,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "1h ago".into(),
    };
    let actions = get_note_switcher_actions(&[note]);
    assert_eq!(actions[0].description, Some("1h ago".to_string()));
}

#[test]
fn cat18_empty_preview_empty_time_uses_char_count() {
    let note = NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 0,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    };
    let actions = get_note_switcher_actions(&[note]);
    assert_eq!(actions[0].description, Some("0 chars".to_string()));
}

#[test]
fn cat18_singular_char_count() {
    let note = NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 1,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    };
    let actions = get_note_switcher_actions(&[note]);
    assert_eq!(actions[0].description, Some("1 char".to_string()));
}

#[test]
fn cat18_preview_truncated_at_61_chars() {
    let long_preview = "a".repeat(61);
    let note = NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 61,
        is_current: false,
        is_pinned: false,
        preview: long_preview,
        relative_time: "".into(),
    };
    let actions = get_note_switcher_actions(&[note]);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(desc.ends_with('…'), "Should be truncated: {}", desc);
}

#[test]
fn cat18_preview_not_truncated_at_60_chars() {
    let exact_preview = "b".repeat(60);
    let note = NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 60,
        is_current: false,
        is_pinned: false,
        preview: exact_preview,
        relative_time: "".into(),
    };
    let actions = get_note_switcher_actions(&[note]);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(
        !desc.ends_with('…'),
        "Should NOT be truncated at exactly 60"
    );
}

// ============================================================================
// 19. Note switcher — empty state fallback
// ============================================================================

#[test]
fn cat19_empty_notes_shows_placeholder() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "no_notes");
    assert!(actions[0].title.contains("No notes yet"));
}

#[test]
fn cat19_empty_placeholder_has_plus_icon() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(
        actions[0].icon,
        Some(crate::designs::icon_variations::IconName::Plus)
    );
}

#[test]
fn cat19_empty_placeholder_description_mentions_cmd_n() {
    let actions = get_note_switcher_actions(&[]);
    assert!(actions[0].description.as_ref().unwrap().contains("⌘N"));
}

// ============================================================================
// 20. Chat context — model selection and conditional actions
// ============================================================================

#[test]
fn cat20_no_models_still_has_continue_in_chat() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "continue_in_chat");
}

#[test]
fn cat20_current_model_gets_checkmark() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".into()),
        available_models: vec![
            ChatModelInfo {
                id: "gpt4".into(),
                display_name: "GPT-4".into(),
                provider: "OpenAI".into(),
            },
            ChatModelInfo {
                id: "claude".into(),
                display_name: "Claude".into(),
                provider: "Anthropic".into(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let gpt4 = actions
        .iter()
        .find(|a| a.id == "select_model_gpt4")
        .unwrap();
    assert!(gpt4.title.contains('✓'), "Current should have ✓");
    let claude = actions
        .iter()
        .find(|a| a.id == "select_model_claude")
        .unwrap();
    assert!(!claude.title.contains('✓'), "Non-current should not have ✓");
}

#[test]
fn cat20_copy_response_only_when_has_response() {
    let no_resp = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let with_resp = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    assert!(!get_chat_context_actions(&no_resp)
        .iter()
        .any(|a| a.id == "copy_response"));
    assert!(get_chat_context_actions(&with_resp)
        .iter()
        .any(|a| a.id == "copy_response"));
}

#[test]
fn cat20_clear_conversation_only_when_has_messages() {
    let no_msgs = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let with_msgs = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    assert!(!get_chat_context_actions(&no_msgs)
        .iter()
        .any(|a| a.id == "clear_conversation"));
    assert!(get_chat_context_actions(&with_msgs)
        .iter()
        .any(|a| a.id == "clear_conversation"));
}

// ============================================================================
// 21. to_deeplink_name — edge cases
// ============================================================================

#[test]
fn cat21_basic_conversion() {
    assert_eq!(to_deeplink_name("My Script"), "my-script");
}

#[test]
fn cat21_underscores_become_hyphens() {
    assert_eq!(to_deeplink_name("hello_world"), "hello-world");
}

#[test]
fn cat21_special_chars_stripped() {
    assert_eq!(to_deeplink_name("test!@#$%"), "test");
}

#[test]
fn cat21_consecutive_specials_collapsed() {
    assert_eq!(to_deeplink_name("a---b"), "a-b");
}

#[test]
fn cat21_unicode_alphanumeric_preserved() {
    assert_eq!(to_deeplink_name("café"), "café");
}

#[test]
fn cat21_leading_trailing_stripped() {
    assert_eq!(to_deeplink_name("  hello  "), "hello");
}

#[test]
fn cat21_numbers_preserved() {
    assert_eq!(to_deeplink_name("v2 script"), "v2-script");
}

// ============================================================================
// 22. fuzzy_match edge cases
// ============================================================================

#[test]
fn cat22_empty_needle_matches() {
    assert!(ActionsDialog::fuzzy_match("anything", ""));
}

#[test]
fn cat22_empty_haystack_with_needle_fails() {
    assert!(!ActionsDialog::fuzzy_match("", "x"));
}

#[test]
fn cat22_both_empty_matches() {
    assert!(ActionsDialog::fuzzy_match("", ""));
}

#[test]
fn cat22_exact_match() {
    assert!(ActionsDialog::fuzzy_match("hello", "hello"));
}

#[test]
fn cat22_subsequence_match() {
    assert!(ActionsDialog::fuzzy_match("hello world", "hlo"));
}

#[test]
fn cat22_no_subsequence() {
    assert!(!ActionsDialog::fuzzy_match("hello", "xyz"));
}

#[test]
fn cat22_needle_longer_than_haystack() {
    assert!(!ActionsDialog::fuzzy_match("hi", "hello"));
}

// ============================================================================
// 23. score_action boundary thresholds
// ============================================================================

#[test]
fn cat23_prefix_match_gives_100() {
    let a = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
    assert!(ActionsDialog::score_action(&a, "edit") >= 100);
}

#[test]
fn cat23_contains_match_gives_50() {
    let a = Action::new("id", "My Edit Tool", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&a, "edit");
    assert!(
        (50..100).contains(&score),
        "Contains should be 50-99: {}",
        score
    );
}

#[test]
fn cat23_fuzzy_match_gives_25() {
    let a = Action::new("id", "Elephant", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&a, "ept");
    assert!(
        (25..50).contains(&score),
        "Fuzzy should be 25-49: {}",
        score
    );
}

#[test]
fn cat23_description_bonus_15() {
    let a = Action::new(
        "id",
        "Open File",
        Some("Edit in editor".into()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&a, "editor");
    assert!(
        score >= 15,
        "Description match should give >= 15: {}",
        score
    );
}

#[test]
fn cat23_no_match_gives_0() {
    let a = Action::new("id", "Run Script", None, ActionCategory::ScriptContext);
    assert_eq!(ActionsDialog::score_action(&a, "xyz"), 0);
}

#[test]
fn cat23_prefix_plus_desc_stacks() {
    let a = Action::new(
        "id",
        "Edit Script",
        Some("Edit the script in editor".into()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&a, "edit");
    assert!(score >= 115, "Prefix(100) + Desc(15) = 115: {}", score);
}

// ============================================================================
// 24. parse_shortcut_keycaps
// ============================================================================

#[test]
fn cat24_modifier_plus_letter() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘C");
    assert_eq!(caps, vec!["⌘", "C"]);
}

#[test]
fn cat24_two_modifiers() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘⇧C");
    assert_eq!(caps, vec!["⌘", "⇧", "C"]);
}

#[test]
fn cat24_enter_symbol() {
    let caps = ActionsDialog::parse_shortcut_keycaps("↵");
    assert_eq!(caps, vec!["↵"]);
}

#[test]
fn cat24_arrow_keys() {
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("↑"), vec!["↑"]);
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("↓"), vec!["↓"]);
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("←"), vec!["←"]);
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("→"), vec!["→"]);
}

#[test]
fn cat24_escape_and_space() {
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("⎋"), vec!["⎋"]);
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("␣"), vec!["␣"]);
}

#[test]
fn cat24_lowercase_uppercased() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘x");
    assert_eq!(caps, vec!["⌘", "X"]);
}

// ============================================================================
// 25. build_grouped_items_static behavior
// ============================================================================

#[test]
fn cat25_empty_filtered_returns_empty() {
    let actions: Vec<Action> = vec![];
    let result = build_grouped_items_static(&actions, &[], SectionStyle::Headers);
    assert!(result.is_empty());
}

#[test]
fn cat25_headers_inserts_section_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered = vec![0, 1];
    let result = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // Should have: header S1, item 0, header S2, item 1
    assert_eq!(result.len(), 4);
}

#[test]
fn cat25_separators_no_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered = vec![0, 1];
    let result = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    // Should have: item 0, item 1 (no headers)
    assert_eq!(result.len(), 2);
}

#[test]
fn cat25_none_style_no_headers() {
    let actions =
        vec![Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1")];
    let filtered = vec![0];
    let result = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
    assert_eq!(result.len(), 1);
}

#[test]
fn cat25_same_section_no_duplicate_header() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S"),
    ];
    let filtered = vec![0, 1];
    let result = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // Should have: header S, item 0, item 1
    assert_eq!(result.len(), 3);
}
