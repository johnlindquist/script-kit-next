
#[test]
fn batch32_parse_shortcut_keycaps_space_symbol() {
    let caps = ActionsDialog::parse_shortcut_keycaps("␣");
    assert_eq!(caps, vec!["␣"]);
}

// ---------------------------------------------------------------------------
// 21. score_action: empty search returns zero
// ---------------------------------------------------------------------------

#[test]
fn batch32_score_action_empty_search_returns_zero() {
    let action = Action::new("test", "Test Action", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "");
    // Empty string is a prefix of everything, so prefix match gives 100
    assert!(
        score >= 100,
        "Empty search should prefix-match, got {}",
        score
    );
}

#[test]
fn batch32_score_action_prefix_match_100_plus() {
    let action = Action::new(
        "edit",
        "Edit Script",
        Some("Open in editor".into()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘E");
    let score = ActionsDialog::score_action(&action, "edit");
    // prefix (100) + desc contains "edit" (15) = 115
    assert!(score >= 100, "Prefix match should be 100+, got {}", score);
}

#[test]
fn batch32_score_action_no_match_returns_zero() {
    let action = Action::new("test", "Test Action", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "xyz");
    assert_eq!(score, 0);
}

#[test]
fn batch32_score_action_desc_bonus_stacks() {
    let action = Action::new(
        "open",
        "Open File",
        Some("Open in editor".into()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "open");
    // prefix (100) + desc contains "open" (15) = 115
    assert_eq!(score, 115);
}

// ---------------------------------------------------------------------------
// 22. fuzzy_match: edge cases
// ---------------------------------------------------------------------------

#[test]
fn batch32_fuzzy_match_empty_needle_matches() {
    assert!(ActionsDialog::fuzzy_match("anything", ""));
}

#[test]
fn batch32_fuzzy_match_empty_haystack_empty_needle() {
    assert!(ActionsDialog::fuzzy_match("", ""));
}

#[test]
fn batch32_fuzzy_match_empty_haystack_nonempty_needle() {
    assert!(!ActionsDialog::fuzzy_match("", "a"));
}

#[test]
fn batch32_fuzzy_match_subsequence() {
    assert!(ActionsDialog::fuzzy_match("hello world", "hlo"));
}

#[test]
fn batch32_fuzzy_match_no_subsequence() {
    assert!(!ActionsDialog::fuzzy_match("hello", "ba"));
}

// ---------------------------------------------------------------------------
// 23. build_grouped_items_static: Headers style adds section headers
// ---------------------------------------------------------------------------

#[test]
fn batch32_grouped_items_headers_style_adds_header() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Sec1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Sec1"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // Should be: Header("Sec1"), Item(0), Item(1)
    assert_eq!(grouped.len(), 3);
    assert!(matches!(&grouped[0], GroupedActionItem::SectionHeader(s) if s == "Sec1"));
}

#[test]
fn batch32_grouped_items_separators_no_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Sec1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Sec2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    // No headers, just items
    assert_eq!(grouped.len(), 2);
    assert!(matches!(&grouped[0], GroupedActionItem::Item(_)));
}

#[test]
fn batch32_grouped_items_headers_two_sections() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // Header("S1"), Item(0), Header("S2"), Item(1)
    assert_eq!(grouped.len(), 4);
    assert!(matches!(&grouped[0], GroupedActionItem::SectionHeader(s) if s == "S1"));
    assert!(matches!(&grouped[2], GroupedActionItem::SectionHeader(s) if s == "S2"));
}

#[test]
fn batch32_grouped_items_empty_returns_empty() {
    let actions: Vec<Action> = vec![];
    let filtered: Vec<usize> = vec![];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    assert!(grouped.is_empty());
}

// ---------------------------------------------------------------------------
// 24. coerce_action_selection: various patterns
// ---------------------------------------------------------------------------

#[test]
fn batch32_coerce_on_item_stays() {
    let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
}

#[test]
fn batch32_coerce_on_header_jumps_down_to_item() {
    let rows = vec![
        GroupedActionItem::SectionHeader("H".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn batch32_coerce_trailing_header_jumps_up() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("H".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn batch32_coerce_all_headers_returns_none() {
    let rows = vec![
        GroupedActionItem::SectionHeader("H1".into()),
        GroupedActionItem::SectionHeader("H2".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn batch32_coerce_empty_returns_none() {
    let rows: Vec<GroupedActionItem> = vec![];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

// ---------------------------------------------------------------------------
// 25. CommandBarConfig: ai_style vs main_menu_style differences
// ---------------------------------------------------------------------------

#[test]
fn batch32_config_ai_style_show_icons_true() {
    let config = CommandBarConfig::ai_style();
    assert!(config.dialog_config.show_icons);
}

#[test]
fn batch32_config_main_menu_show_icons_false() {
    let config = CommandBarConfig::main_menu_style();
    assert!(!config.dialog_config.show_icons);
}

#[test]
fn batch32_config_ai_style_show_footer_true() {
    let config = CommandBarConfig::ai_style();
    assert!(config.dialog_config.show_footer);
}

#[test]
fn batch32_config_main_menu_show_footer_false() {
    let config = CommandBarConfig::main_menu_style();
    assert!(!config.dialog_config.show_footer);
}

// ---------------------------------------------------------------------------
// 26. Script context: with_action_verb propagates to run_script title
// ---------------------------------------------------------------------------

#[test]
fn batch32_script_custom_verb_launch() {
    let script =
        crate::actions::types::ScriptInfo::with_action_verb("App", "/p/app", true, "Launch");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].title, "Launch \"App\"");
}

#[test]
fn batch32_script_custom_verb_switch_to() {
    let script =
        crate::actions::types::ScriptInfo::with_action_verb("Window", "/p/w", false, "Switch to");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].title, "Switch to \"Window\"");
}

#[test]
fn batch32_script_custom_verb_desc_uses_verb() {
    let script =
        crate::actions::types::ScriptInfo::with_action_verb("Foo", "/p/foo", true, "Execute");
    let actions = get_script_context_actions(&script);
    assert!(
        actions[0].description.as_ref().unwrap().contains("Execute"),
        "run desc should use verb, got: {:?}",
        actions[0].description
    );
}

// ---------------------------------------------------------------------------
// 27. Script context: deeplink URL format in copy_deeplink description
// ---------------------------------------------------------------------------

#[test]
fn batch32_script_deeplink_url_format() {
    let script = crate::actions::types::ScriptInfo::new("My Script", "/p/my-script.ts");
    let actions = get_script_context_actions(&script);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert!(
        dl.description
            .as_ref()
            .unwrap()
            .contains("scriptkit://run/my-script"),
        "deeplink should contain URL, got: {:?}",
        dl.description
    );
}

#[test]
fn batch32_script_deeplink_shortcut() {
    let script = crate::actions::types::ScriptInfo::new("Test", "/p/test.ts");
    let actions = get_script_context_actions(&script);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert_eq!(dl.shortcut.as_deref(), Some("⌘⇧D"));
}

#[test]
fn batch32_builtin_deeplink_url_format() {
    let builtin = crate::actions::types::ScriptInfo::builtin("Clipboard History");
    let actions = get_script_context_actions(&builtin);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert!(dl
        .description
        .as_ref()
        .unwrap()
        .contains("scriptkit://run/clipboard-history"),);
}

// ---------------------------------------------------------------------------
// 28. Clipboard: save_snippet and save_file always present
// ---------------------------------------------------------------------------

#[test]
fn batch32_clipboard_text_has_save_snippet() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_save_snippet"));
}

#[test]
fn batch32_clipboard_image_has_save_snippet() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_save_snippet"));
}

#[test]
fn batch32_clipboard_text_has_save_file() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_save_file"));
}

// ---------------------------------------------------------------------------
// 29. Action builder: cached lowercase fields
// ---------------------------------------------------------------------------

#[test]
fn batch32_action_title_lower_is_precomputed() {
    let action = Action::new("id", "Hello World", None, ActionCategory::ScriptContext);
    assert_eq!(action.title_lower, "hello world");
}

#[test]
fn batch32_action_description_lower_is_precomputed() {
    let action = Action::new(
        "id",
        "T",
        Some("Open In EDITOR".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(action.description_lower.as_deref(), Some("open in editor"));
}

#[test]
fn batch32_action_no_description_lower_is_none() {
    let action = Action::new("id", "T", None, ActionCategory::ScriptContext);
    assert!(action.description_lower.is_none());
}

#[test]
fn batch32_action_shortcut_lower_after_with_shortcut() {
    let action = Action::new("id", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
    assert_eq!(action.shortcut_lower.as_deref(), Some("⌘⇧c"));
}

// ---------------------------------------------------------------------------
// 30. Cross-context: all clipboard actions have ActionCategory::ScriptContext
// ---------------------------------------------------------------------------

#[test]
fn batch32_all_clipboard_actions_are_script_context() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    for a in &actions {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "clipboard action {} should be ScriptContext",
            a.id
        );
    }
}

#[test]
fn batch32_all_file_actions_are_script_context() {
    let info = FileInfo {
        path: "/p/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    for a in &actions {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "file action {} should be ScriptContext",
            a.id
        );
    }
}

#[test]
fn batch32_all_path_actions_are_script_context() {
    let info = PathInfo::new("test.txt", "/p/test.txt", false);
    let actions = get_path_context_actions(&info);
    for a in &actions {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "path action {} should be ScriptContext",
            a.id
        );
    }
}

#[test]
fn batch32_all_notes_actions_are_script_context() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    for a in &actions {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "notes action {} should be ScriptContext",
            a.id
        );
    }
}

#[test]
fn batch32_all_ai_bar_actions_are_script_context() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "AI action {} should be ScriptContext",
            a.id
        );
    }
}
