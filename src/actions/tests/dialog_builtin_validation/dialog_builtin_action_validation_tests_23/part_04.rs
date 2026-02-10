
#[test]
fn batch23_score_fuzzy_low() {
    let action = Action::new(
        "a",
        "Configure Options Pretty",
        None,
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "cop");
    // "cop" is a subsequence of "configure options pretty"
    assert!(score >= 25);
}

#[test]
fn batch23_score_no_match_zero() {
    let action = Action::new("a", "Delete", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "xyz");
    assert_eq!(score, 0);
}

#[test]
fn batch23_score_empty_search_prefix() {
    let action = Action::new("a", "Anything", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "");
    assert!(score >= 100);
}

// ============================================================
// 26. fuzzy_match: edge cases
// ============================================================

#[test]
fn batch23_fuzzy_exact_match() {
    assert!(ActionsDialog::fuzzy_match("hello", "hello"));
}

#[test]
fn batch23_fuzzy_subsequence() {
    assert!(ActionsDialog::fuzzy_match("hello world", "hwd"));
}

#[test]
fn batch23_fuzzy_no_match() {
    assert!(!ActionsDialog::fuzzy_match("abc", "xyz"));
}

#[test]
fn batch23_fuzzy_empty_needle_matches() {
    assert!(ActionsDialog::fuzzy_match("anything", ""));
}

#[test]
fn batch23_fuzzy_needle_longer_fails() {
    assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
}

// ============================================================
// 27. build_grouped_items_static: headers style adds section labels
// ============================================================

#[test]
fn batch23_grouped_headers_two_sections() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // S1 header + item A + S2 header + item B = 4
    assert_eq!(grouped.len(), 4);
    assert!(matches!(&grouped[0], GroupedActionItem::SectionHeader(s) if s == "S1"));
    assert!(matches!(&grouped[2], GroupedActionItem::SectionHeader(s) if s == "S2"));
}

#[test]
fn batch23_grouped_headers_same_section_no_dup() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S1"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // S1 header + item A + item B = 3 (no second header)
    assert_eq!(grouped.len(), 3);
}

#[test]
fn batch23_grouped_separators_no_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    // No headers with Separators, just items
    assert_eq!(grouped.len(), 2);
    assert!(matches!(&grouped[0], GroupedActionItem::Item(_)));
    assert!(matches!(&grouped[1], GroupedActionItem::Item(_)));
}

#[test]
fn batch23_grouped_empty_filtered() {
    let actions = vec![Action::new("a", "A", None, ActionCategory::ScriptContext)];
    let filtered: Vec<usize> = vec![];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    assert!(grouped.is_empty());
}

// ============================================================
// 28. coerce_action_selection: various scenarios
// ============================================================

#[test]
fn batch23_coerce_empty_returns_none() {
    assert_eq!(coerce_action_selection(&[], 0), None);
}

#[test]
fn batch23_coerce_item_returns_same() {
    let rows = vec![GroupedActionItem::Item(0)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
}

#[test]
fn batch23_coerce_header_skips_to_item() {
    let rows = vec![
        GroupedActionItem::SectionHeader("H".to_string()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn batch23_coerce_header_at_end_searches_up() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("H".to_string()),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn batch23_coerce_all_headers_returns_none() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".to_string()),
        GroupedActionItem::SectionHeader("B".to_string()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

// ============================================================
// 29. Action builder: has_action defaults to false
// ============================================================

#[test]
fn batch23_action_default_has_action_false() {
    let action = Action::new("id", "title", None, ActionCategory::ScriptContext);
    assert!(!action.has_action);
}

#[test]
fn batch23_action_default_value_none() {
    let action = Action::new("id", "title", None, ActionCategory::ScriptContext);
    assert!(action.value.is_none());
}

#[test]
fn batch23_action_default_icon_none() {
    let action = Action::new("id", "title", None, ActionCategory::ScriptContext);
    assert!(action.icon.is_none());
}

#[test]
fn batch23_action_default_section_none() {
    let action = Action::new("id", "title", None, ActionCategory::ScriptContext);
    assert!(action.section.is_none());
}

#[test]
fn batch23_action_with_all_builders() {
    let action = Action::new(
        "id",
        "Title",
        Some("Desc".to_string()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘A")
    .with_icon(IconName::Star)
    .with_section("S1");
    assert_eq!(action.shortcut.as_ref().unwrap(), "⌘A");
    assert_eq!(action.icon, Some(IconName::Star));
    assert_eq!(action.section.as_ref().unwrap(), "S1");
    assert_eq!(action.title_lower, "title");
    assert_eq!(action.description_lower.as_ref().unwrap(), "desc");
    assert_eq!(action.shortcut_lower.as_ref().unwrap(), "⌘a");
}

// ============================================================
// 30. Cross-context: all contexts produce at least one action
// ============================================================

#[test]
fn batch23_cross_script_has_actions() {
    let script = ScriptInfo::new("t", "/t.ts");
    assert!(!get_script_context_actions(&script).is_empty());
}

#[test]
fn batch23_cross_builtin_has_actions() {
    let b = ScriptInfo::builtin("B");
    assert!(!get_script_context_actions(&b).is_empty());
}

#[test]
fn batch23_cross_clipboard_text_has_actions() {
    let e = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    assert!(!get_clipboard_history_context_actions(&e).is_empty());
}

#[test]
fn batch23_cross_clipboard_image_has_actions() {
    let e = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".to_string(),
        image_dimensions: Some((1, 1)),
        frontmost_app_name: None,
    };
    assert!(!get_clipboard_history_context_actions(&e).is_empty());
}

#[test]
fn batch23_cross_path_has_actions() {
    let p = PathInfo::new("t", "/t", false);
    assert!(!get_path_context_actions(&p).is_empty());
}

#[test]
fn batch23_cross_file_has_actions() {
    let f = FileInfo {
        path: "/t".to_string(),
        name: "t".to_string(),
        file_type: FileType::File,
        is_dir: false,
    };
    assert!(!get_file_context_actions(&f).is_empty());
}

#[test]
fn batch23_cross_ai_has_actions() {
    assert!(!get_ai_command_bar_actions().is_empty());
}

#[test]
fn batch23_cross_notes_has_actions() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    assert!(!get_notes_command_bar_actions(&info).is_empty());
}

#[test]
fn batch23_cross_chat_has_actions() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    assert!(!get_chat_context_actions(&info).is_empty());
}

#[test]
fn batch23_cross_note_switcher_empty_has_placeholder() {
    assert!(!get_note_switcher_actions(&[]).is_empty());
}
