
#[test]
fn cat27_18_coerce_only_headers_returns_none() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".into()),
        GroupedActionItem::SectionHeader("B".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

// ─────────────────────────────────────────────
// 19. Action: with_shortcut sets shortcut_lower
// ─────────────────────────────────────────────

#[test]
fn cat27_19_with_shortcut_sets_shortcut_lower() {
    let action = Action::new("t", "Test", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
    assert_eq!(action.shortcut_lower, Some("⌘e".into()));
}

#[test]
fn cat27_19_no_shortcut_shortcut_lower_is_none() {
    let action = Action::new("t", "Test", None, ActionCategory::ScriptContext);
    assert!(action.shortcut_lower.is_none());
}

#[test]
fn cat27_19_title_lower_is_precomputed() {
    let action = Action::new("t", "Edit Script", None, ActionCategory::ScriptContext);
    assert_eq!(action.title_lower, "edit script");
}

#[test]
fn cat27_19_description_lower_is_precomputed() {
    let action = Action::new(
        "t",
        "Test",
        Some("Open in $EDITOR".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(action.description_lower, Some("open in $editor".into()));
}

// ─────────────────────────────────────────────
// 20. Script context: scriptlet vs script edit action IDs differ
// ─────────────────────────────────────────────

#[test]
fn cat27_20_script_edit_id_is_edit_script() {
    let script = ScriptInfo::new("test", "/p.ts");
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"edit_script"));
    assert!(!ids.contains(&"edit_scriptlet"));
}

#[test]
fn cat27_20_scriptlet_edit_id_is_edit_scriptlet() {
    let script = ScriptInfo::scriptlet("s", "/p.md", None, None);
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"edit_scriptlet"));
    assert!(!ids.contains(&"edit_script"));
}

#[test]
fn cat27_20_agent_edit_id_is_edit_script() {
    let mut script = ScriptInfo::builtin("agent");
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"edit_script"));
}

#[test]
fn cat27_20_agent_edit_title_says_agent() {
    let mut script = ScriptInfo::builtin("agent");
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert!(edit.title.contains("Agent"));
}

// ─────────────────────────────────────────────
// 21. Notes command bar: copy section requires selection+not trash
// ─────────────────────────────────────────────

#[test]
fn cat27_21_notes_copy_section_present_with_selection() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"copy_note_as"));
    assert!(ids.contains(&"copy_deeplink"));
    assert!(ids.contains(&"create_quicklink"));
}

#[test]
fn cat27_21_notes_copy_section_absent_without_selection() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(!ids.contains(&"copy_note_as"));
    assert!(!ids.contains(&"copy_deeplink"));
}

#[test]
fn cat27_21_notes_copy_section_absent_in_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(!ids.contains(&"copy_note_as"));
}

#[test]
fn cat27_21_notes_create_quicklink_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ql = actions.iter().find(|a| a.id == "create_quicklink").unwrap();
    assert_eq!(ql.shortcut.as_deref(), Some("⇧⌘L"));
}

// ─────────────────────────────────────────────
// 22. AI command bar: section counts
// ─────────────────────────────────────────────

#[test]
fn cat27_22_ai_response_section_has_3_actions() {
    let actions = get_ai_command_bar_actions();
    let response_count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .count();
    assert_eq!(response_count, 3);
}

#[test]
fn cat27_22_ai_actions_section_has_4_actions() {
    let actions = get_ai_command_bar_actions();
    let action_count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Actions"))
        .count();
    assert_eq!(action_count, 4);
}

#[test]
fn cat27_22_ai_attachments_section_has_2_actions() {
    let actions = get_ai_command_bar_actions();
    let attach_count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Attachments"))
        .count();
    assert_eq!(attach_count, 2);
}

#[test]
fn cat27_22_ai_total_is_12() {
    let actions = get_ai_command_bar_actions();
    assert_eq!(actions.len(), 12);
}

// ─────────────────────────────────────────────
// 23. parse_shortcut_keycaps: various combos
// ─────────────────────────────────────────────

#[test]
fn cat27_23_parse_keycaps_cmd_e() {
    let caps = super::dialog::ActionsDialog::parse_shortcut_keycaps("⌘E");
    assert_eq!(caps, vec!["⌘", "E"]);
}

#[test]
fn cat27_23_parse_keycaps_all_modifiers_and_key() {
    let caps = super::dialog::ActionsDialog::parse_shortcut_keycaps("⌘⌃⌥⇧A");
    assert_eq!(caps, vec!["⌘", "⌃", "⌥", "⇧", "A"]);
}

#[test]
fn cat27_23_parse_keycaps_enter_alone() {
    let caps = super::dialog::ActionsDialog::parse_shortcut_keycaps("↵");
    assert_eq!(caps, vec!["↵"]);
}

#[test]
fn cat27_23_parse_keycaps_lowercase_uppercased() {
    let caps = super::dialog::ActionsDialog::parse_shortcut_keycaps("⌘c");
    assert_eq!(caps, vec!["⌘", "C"]);
}

// ─────────────────────────────────────────────
// 24. fuzzy_match: various patterns
// ─────────────────────────────────────────────

#[test]
fn cat27_24_fuzzy_match_exact() {
    assert!(super::dialog::ActionsDialog::fuzzy_match("edit", "edit"));
}

#[test]
fn cat27_24_fuzzy_match_subsequence() {
    assert!(super::dialog::ActionsDialog::fuzzy_match(
        "edit script",
        "es"
    ));
}

#[test]
fn cat27_24_fuzzy_match_no_match() {
    assert!(!super::dialog::ActionsDialog::fuzzy_match("edit", "z"));
}

#[test]
fn cat27_24_fuzzy_match_empty_needle() {
    assert!(super::dialog::ActionsDialog::fuzzy_match("anything", ""));
}

#[test]
fn cat27_24_fuzzy_match_needle_longer_fails() {
    assert!(!super::dialog::ActionsDialog::fuzzy_match("ab", "abc"));
}

// ─────────────────────────────────────────────
// 25. Script context: view_logs exclusive to is_script
// ─────────────────────────────────────────────

#[test]
fn cat27_25_script_has_view_logs() {
    let script = ScriptInfo::new("test", "/p.ts");
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"view_logs"));
}

#[test]
fn cat27_25_scriptlet_no_view_logs() {
    let script = ScriptInfo::scriptlet("s", "/p.md", None, None);
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(!ids.contains(&"view_logs"));
}

#[test]
fn cat27_25_builtin_no_view_logs() {
    let script = ScriptInfo::builtin("Clipboard");
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(!ids.contains(&"view_logs"));
}

#[test]
fn cat27_25_agent_no_view_logs() {
    let mut script = ScriptInfo::builtin("agent");
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(!ids.contains(&"view_logs"));
}

// ─────────────────────────────────────────────
// 26. Clipboard: delete_all desc mentions pinned exception
// ─────────────────────────────────────────────

#[test]
fn cat27_26_clipboard_delete_all_desc_mentions_pinned() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let del_all = actions
        .iter()
        .find(|a| a.id == "clipboard_delete_all")
        .unwrap();
    assert!(del_all
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("pinned"));
}

#[test]
fn cat27_26_clipboard_delete_multiple_desc_mentions_filter() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let del_multi = actions
        .iter()
        .find(|a| a.id == "clipboard_delete_multiple")
        .unwrap();
    assert!(del_multi
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("filter"));
}

#[test]
fn cat27_26_clipboard_delete_shortcut_is_ctrl_x() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let del = actions.iter().find(|a| a.id == "clipboard_delete").unwrap();
    assert_eq!(del.shortcut.as_deref(), Some("⌃X"));
}

// ─────────────────────────────────────────────
// 27. Note switcher: preview with time uses separator
// ─────────────────────────────────────────────

#[test]
fn cat27_27_note_switcher_preview_with_time_has_separator() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".into(),
        title: "T".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "Hello world".into(),
        relative_time: "3m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(desc.contains(" · "));
    assert!(desc.contains("Hello world"));
    assert!(desc.contains("3m ago"));
}

#[test]
fn cat27_27_note_switcher_long_preview_truncated() {
    let long_preview = "a".repeat(80);
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".into(),
        title: "T".into(),
        char_count: 80,
        is_current: false,
        is_pinned: false,
        preview: long_preview,
        relative_time: "1h ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(desc.contains("…"));
}

#[test]
fn cat27_27_note_switcher_exactly_60_chars_no_truncation() {
    let exact = "b".repeat(60);
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".into(),
        title: "T".into(),
        char_count: 60,
        is_current: false,
        is_pinned: false,
        preview: exact,
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(!desc.contains("…"));
}

// ─────────────────────────────────────────────
// 28. ScriptInfo: with_frecency builder chain
// ─────────────────────────────────────────────

#[test]
fn cat27_28_with_frecency_sets_is_suggested() {
    let script = ScriptInfo::new("test", "/p.ts").with_frecency(true, Some("/p".into()));
    assert!(script.is_suggested);
    assert_eq!(script.frecency_path, Some("/p".into()));
}

#[test]
fn cat27_28_with_frecency_false_not_suggested() {
    let script = ScriptInfo::new("test", "/p.ts").with_frecency(false, None);
    assert!(!script.is_suggested);
    assert!(script.frecency_path.is_none());
}

#[test]
fn cat27_28_with_frecency_preserves_other_fields() {
    let script = ScriptInfo::new("test", "/p.ts").with_frecency(true, None);
    assert!(script.is_script);
    assert_eq!(script.action_verb, "Run");
    assert_eq!(script.name, "test");
}

// ─────────────────────────────────────────────
// 29. CommandBarConfig: anchor positions
// ─────────────────────────────────────────────

#[test]
fn cat27_29_default_config_anchor_bottom() {
    let cfg = CommandBarConfig::default();
    assert_eq!(cfg.dialog_config.anchor, AnchorPosition::Bottom);
}

#[test]
fn cat27_29_ai_style_anchor_top() {
    let cfg = CommandBarConfig::ai_style();
    assert_eq!(cfg.dialog_config.anchor, AnchorPosition::Top);
}

#[test]
fn cat27_29_main_menu_anchor_bottom() {
    let cfg = CommandBarConfig::main_menu_style();
    assert_eq!(cfg.dialog_config.anchor, AnchorPosition::Bottom);
}

#[test]
fn cat27_29_notes_style_anchor_top() {
    let cfg = CommandBarConfig::notes_style();
    assert_eq!(cfg.dialog_config.anchor, AnchorPosition::Top);
}
