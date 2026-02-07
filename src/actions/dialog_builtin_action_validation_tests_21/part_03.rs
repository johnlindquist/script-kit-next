
// ============================================================
// 16. Clipboard context: frontmost_app_name edge cases
// ============================================================

#[test]
fn batch21_clipboard_paste_empty_string_app() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: Some("".into()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let a = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
    // Empty string still triggers Some branch: "Paste to "
    assert_eq!(a.title, "Paste to ");
}

#[test]
fn batch21_clipboard_paste_unicode_app() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: Some("Xcode \u{2013} Beta".into()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let a = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
    assert_eq!(a.title, "Paste to Xcode \u{2013} Beta");
}

// ============================================================
// 17. CommandBarConfig preset field matrix
// ============================================================

#[test]
fn batch21_config_default_search_bottom() {
    let c = CommandBarConfig::default();
    assert!(matches!(
        c.dialog_config.search_position,
        SearchPosition::Bottom
    ));
}

#[test]
fn batch21_config_ai_style_anchor_top() {
    let c = CommandBarConfig::ai_style();
    assert!(matches!(c.dialog_config.anchor, AnchorPosition::Top));
}

#[test]
fn batch21_config_main_menu_anchor_bottom() {
    let c = CommandBarConfig::main_menu_style();
    assert!(matches!(c.dialog_config.anchor, AnchorPosition::Bottom));
}

#[test]
fn batch21_config_notes_style_icons_true() {
    let c = CommandBarConfig::notes_style();
    assert!(c.dialog_config.show_icons);
}

#[test]
fn batch21_config_notes_style_footer_true() {
    let c = CommandBarConfig::notes_style();
    assert!(c.dialog_config.show_footer);
}

// ============================================================
// 18. ActionsDialogConfig default values
// ============================================================

#[test]
fn batch21_dialog_config_default_search_bottom() {
    let c = ActionsDialogConfig::default();
    assert!(matches!(c.search_position, SearchPosition::Bottom));
}

#[test]
fn batch21_dialog_config_default_section_separators() {
    let c = ActionsDialogConfig::default();
    assert!(matches!(c.section_style, SectionStyle::Separators));
}

#[test]
fn batch21_dialog_config_default_anchor_bottom() {
    let c = ActionsDialogConfig::default();
    assert!(matches!(c.anchor, AnchorPosition::Bottom));
}

#[test]
fn batch21_dialog_config_default_no_icons() {
    let c = ActionsDialogConfig::default();
    assert!(!c.show_icons);
}

#[test]
fn batch21_dialog_config_default_no_footer() {
    let c = ActionsDialogConfig::default();
    assert!(!c.show_footer);
}

// ============================================================
// 19. Action with_shortcut caching behavior
// ============================================================

#[test]
fn batch21_action_with_shortcut_sets_shortcut_lower() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
    assert_eq!(a.shortcut_lower, Some("⌘e".into()));
}

#[test]
fn batch21_action_no_shortcut_lower_is_none() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext);
    assert!(a.shortcut_lower.is_none());
}

#[test]
fn batch21_action_title_lower_precomputed() {
    let a = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
    assert_eq!(a.title_lower, "edit script");
}

#[test]
fn batch21_action_description_lower_precomputed() {
    let a = Action::new(
        "id",
        "T",
        Some("Open in $EDITOR".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(a.description_lower, Some("open in $editor".into()));
}

#[test]
fn batch21_action_description_none_lower_none() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext);
    assert!(a.description_lower.is_none());
}

// ============================================================
// 20. Action builder chaining: with_icon, with_section
// ============================================================

#[test]
fn batch21_action_with_icon_preserves_shortcut() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext)
        .with_shortcut("⌘X")
        .with_icon(IconName::Copy);
    assert_eq!(a.shortcut.as_deref(), Some("⌘X"));
    assert_eq!(a.icon, Some(IconName::Copy));
}

#[test]
fn batch21_action_with_section_preserves_icon() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext)
        .with_icon(IconName::Star)
        .with_section("MySection");
    assert_eq!(a.icon, Some(IconName::Star));
    assert_eq!(a.section.as_deref(), Some("MySection"));
}

#[test]
fn batch21_action_full_chain_all_fields() {
    let a = Action::new(
        "test_id",
        "Test Title",
        Some("Test Desc".into()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘T")
    .with_icon(IconName::Plus)
    .with_section("TestSection");

    assert_eq!(a.id, "test_id");
    assert_eq!(a.title, "Test Title");
    assert_eq!(a.description, Some("Test Desc".into()));
    assert_eq!(a.shortcut, Some("⌘T".into()));
    assert_eq!(a.icon, Some(IconName::Plus));
    assert_eq!(a.section, Some("TestSection".into()));
}

#[test]
fn batch21_action_with_shortcut_opt_none_preserves() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext)
        .with_shortcut("⌘X")
        .with_shortcut_opt(None);
    assert_eq!(a.shortcut.as_deref(), Some("⌘X"));
}

#[test]
fn batch21_action_with_shortcut_opt_some_sets() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext)
        .with_shortcut_opt(Some("⌘Y".into()));
    assert_eq!(a.shortcut.as_deref(), Some("⌘Y"));
}

// ============================================================
// 21. build_grouped_items_static: section transitions
// ============================================================

#[test]
fn batch21_grouped_items_headers_two_sections() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // S1 header, item0, S2 header, item1
    assert_eq!(grouped.len(), 4);
    assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(_)));
    assert!(matches!(grouped[1], GroupedActionItem::Item(0)));
    assert!(matches!(grouped[2], GroupedActionItem::SectionHeader(_)));
    assert!(matches!(grouped[3], GroupedActionItem::Item(1)));
}

#[test]
fn batch21_grouped_items_headers_same_section_no_dup() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // S header, item0, item1
    assert_eq!(grouped.len(), 3);
    assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(_)));
}

#[test]
fn batch21_grouped_items_separators_no_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    // Just items, no headers
    assert_eq!(grouped.len(), 2);
    assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
    assert!(matches!(grouped[1], GroupedActionItem::Item(1)));
}

#[test]
fn batch21_grouped_items_empty_filtered() {
    let actions = vec![Action::new("a", "A", None, ActionCategory::ScriptContext)];
    let filtered: Vec<usize> = vec![];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    assert!(grouped.is_empty());
}

// ============================================================
// 22. coerce_action_selection: header skipping
// ============================================================

#[test]
fn batch21_coerce_on_item_stays() {
    let rows = vec![
        GroupedActionItem::SectionHeader("H".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(1));
}

#[test]
fn batch21_coerce_on_header_jumps_down() {
    let rows = vec![
        GroupedActionItem::SectionHeader("H".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn batch21_coerce_trailing_header_jumps_up() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("H".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn batch21_coerce_all_headers_none() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".into()),
        GroupedActionItem::SectionHeader("B".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn batch21_coerce_empty_none() {
    let rows: Vec<GroupedActionItem> = vec![];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

// ============================================================
// 23. ScriptInfo constructor defaults
// ============================================================

#[test]
fn batch21_scriptinfo_new_defaults() {
    let s = ScriptInfo::new("n", "/p");
    assert!(s.is_script);
    assert!(!s.is_scriptlet);
    assert!(!s.is_agent);
    assert_eq!(s.action_verb, "Run");
    assert!(!s.is_suggested);
    assert!(s.frecency_path.is_none());
}

#[test]
fn batch21_scriptinfo_builtin_path_empty() {
    let s = ScriptInfo::builtin("B");
    assert!(s.path.is_empty());
    assert!(!s.is_script);
}

#[test]
fn batch21_scriptinfo_scriptlet_flags() {
    let s = ScriptInfo::scriptlet("S", "/p", None, None);
    assert!(!s.is_script);
    assert!(s.is_scriptlet);
    assert!(!s.is_agent);
}

#[test]
fn batch21_scriptinfo_with_frecency_chaining() {
    let s = ScriptInfo::new("n", "/p").with_frecency(true, Some("fp".into()));
    assert!(s.is_suggested);
    assert_eq!(s.frecency_path, Some("fp".into()));
    // Original fields preserved
    assert!(s.is_script);
}

// ============================================================
// 24. Script context: copy_content shortcut consistent
// ============================================================

#[test]
fn batch21_script_copy_content_shortcut() {
    let s = ScriptInfo::new("s", "/p");
    let actions = get_script_context_actions(&s);
    let a = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘⌥C"));
}

#[test]
fn batch21_scriptlet_copy_content_shortcut() {
    let s = ScriptInfo::scriptlet("s", "/p", None, None);
    let actions = get_script_context_actions(&s);
    let a = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘⌥C"));
}

#[test]
fn batch21_agent_copy_content_shortcut() {
    let mut s = ScriptInfo::new("a", "/p");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    let a = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘⌥C"));
}

#[test]
fn batch21_scriptlet_with_custom_copy_content_shortcut() {
    let s = ScriptInfo::scriptlet("s", "/p", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    let a = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘⌥C"));
}

// ============================================================
// 25. File vs path context: primary action IDs differ
// ============================================================

#[test]
fn batch21_file_file_primary_is_open_file() {
    let fi = FileInfo {
        path: "/f".into(),
        name: "f".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    assert_eq!(actions[0].id, "open_file");
}

#[test]
fn batch21_path_file_primary_is_select_file_in_file_vs_path() {
    let pi = PathInfo {
        path: "/f".into(),
        name: "f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    assert_eq!(actions[0].id, "select_file");
}

#[test]
fn batch21_file_dir_and_path_dir_same_primary_id() {
    let fi = FileInfo {
        path: "/d".into(),
        name: "d".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let pi = PathInfo {
        path: "/d".into(),
        name: "d".into(),
        is_dir: true,
    };
    assert_eq!(
        get_file_context_actions(&fi)[0].id,
        get_path_context_actions(&pi)[0].id
    );
}

// ============================================================
// 26. Path context: move_to_trash always last
// ============================================================

#[test]
fn batch21_path_trash_last_for_dir() {
    let pi = PathInfo {
        path: "/d".into(),
        name: "d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    assert_eq!(actions.last().unwrap().id, "move_to_trash");
}
