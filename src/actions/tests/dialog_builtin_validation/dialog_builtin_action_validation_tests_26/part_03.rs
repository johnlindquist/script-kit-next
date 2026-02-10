
#[test]
fn cat26_16_builtin_no_view_logs() {
    let b = ScriptInfo::builtin("B");
    let actions = get_script_context_actions(&b);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

#[test]
fn cat26_16_scriptlet_no_view_logs() {
    let s = ScriptInfo::scriptlet("S", "/s.md", None, None);
    let actions = get_script_context_actions(&s);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

#[test]
fn cat26_16_agent_no_view_logs() {
    let mut s = ScriptInfo::new("a", "/a.md");
    s.is_script = false;
    s.is_agent = true;
    let actions = get_script_context_actions(&s);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

// ─────────────────────────────────────────────
// 17. Script context: copy_deeplink always present
// ─────────────────────────────────────────────

#[test]
fn cat26_17_script_has_copy_deeplink() {
    let s = ScriptInfo::new("s", "/s.ts");
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "copy_deeplink"));
}

#[test]
fn cat26_17_builtin_has_copy_deeplink() {
    let b = ScriptInfo::builtin("B");
    let actions = get_script_context_actions(&b);
    assert!(actions.iter().any(|a| a.id == "copy_deeplink"));
}

#[test]
fn cat26_17_scriptlet_has_copy_deeplink() {
    let s = ScriptInfo::scriptlet("S", "/s.md", None, None);
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "copy_deeplink"));
}

// ─────────────────────────────────────────────
// 18. File context: reveal_in_finder always present
// ─────────────────────────────────────────────

#[test]
fn cat26_18_file_reveal_always_present_file() {
    let info = FileInfo {
        path: "/f.txt".into(),
        name: "f.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
}

#[test]
fn cat26_18_file_reveal_always_present_dir() {
    let info = FileInfo {
        path: "/d".into(),
        name: "d".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
}

#[test]
fn cat26_18_file_reveal_shortcut() {
    let info = FileInfo {
        path: "/f.txt".into(),
        name: "f.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
    assert_eq!(reveal.shortcut.as_deref(), Some("⌘↵"));
}

// ─────────────────────────────────────────────
// 19. Path context: open_in_terminal and open_in_editor always present
// ─────────────────────────────────────────────

#[test]
fn cat26_19_path_has_open_in_terminal_for_dir() {
    let info = PathInfo {
        name: "d".into(),
        path: "/d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "open_in_terminal"));
}

#[test]
fn cat26_19_path_has_open_in_terminal_for_file() {
    let info = PathInfo {
        name: "f".into(),
        path: "/f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "open_in_terminal"));
}

#[test]
fn cat26_19_path_has_open_in_editor_for_file() {
    let info = PathInfo {
        name: "f".into(),
        path: "/f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "open_in_editor"));
}

#[test]
fn cat26_19_path_open_in_editor_shortcut() {
    let info = PathInfo {
        name: "f".into(),
        path: "/f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    let editor = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
    assert_eq!(editor.shortcut.as_deref(), Some("⌘E"));
}

// ─────────────────────────────────────────────
// 20. build_grouped_items_static: empty actions list
// ─────────────────────────────────────────────

#[test]
fn cat26_20_build_grouped_empty_actions_empty_result() {
    let result = build_grouped_items_static(&[], &[], SectionStyle::Headers);
    assert!(result.is_empty());
}

#[test]
fn cat26_20_build_grouped_no_filtered_indices() {
    let actions = vec![Action::new(
        "a",
        "Action",
        Some("desc".into()),
        ActionCategory::ScriptContext,
    )];
    let result = build_grouped_items_static(&actions, &[], SectionStyle::Headers);
    assert!(result.is_empty());
}

#[test]
fn cat26_20_build_grouped_single_action_no_section_no_header() {
    let actions = vec![Action::new(
        "a",
        "Action",
        Some("desc".into()),
        ActionCategory::ScriptContext,
    )];
    let result = build_grouped_items_static(&actions, &[0], SectionStyle::Headers);
    // No section on action, so no header added
    assert_eq!(result.len(), 1);
    assert!(matches!(result[0], GroupedActionItem::Item(0)));
}

#[test]
fn cat26_20_build_grouped_single_action_with_section_has_header() {
    let actions = vec![Action::new(
        "a",
        "Action",
        Some("desc".into()),
        ActionCategory::ScriptContext,
    )
    .with_section("MySection")];
    let result = build_grouped_items_static(&actions, &[0], SectionStyle::Headers);
    assert_eq!(result.len(), 2);
    assert!(matches!(&result[0], GroupedActionItem::SectionHeader(s) if s == "MySection"));
    assert!(matches!(result[1], GroupedActionItem::Item(0)));
}

// ─────────────────────────────────────────────
// 21. coerce_action_selection: mixed header/item patterns
// ─────────────────────────────────────────────

#[test]
fn cat26_21_coerce_item_header_item_on_header_goes_down() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("S".into()),
        GroupedActionItem::Item(1),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(2));
}

#[test]
fn cat26_21_coerce_header_item_on_header_goes_down() {
    let rows = vec![
        GroupedActionItem::SectionHeader("S".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn cat26_21_coerce_item_header_on_header_goes_up() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("S".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn cat26_21_coerce_all_headers_returns_none() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".into()),
        GroupedActionItem::SectionHeader("B".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

// ─────────────────────────────────────────────
// 22. Action: title_lower and description_lower caching
// ─────────────────────────────────────────────

#[test]
fn cat26_22_action_title_lower_precomputed() {
    let a = Action::new("id", "Hello World", None, ActionCategory::ScriptContext);
    assert_eq!(a.title_lower, "hello world");
}

#[test]
fn cat26_22_action_description_lower_precomputed() {
    let a = Action::new(
        "id",
        "T",
        Some("My Description".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(a.description_lower.as_deref(), Some("my description"));
}

#[test]
fn cat26_22_action_shortcut_lower_set_by_with_shortcut() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
    assert_eq!(a.shortcut_lower.as_deref(), Some("⌘⇧c"));
}

#[test]
fn cat26_22_action_no_shortcut_lower_is_none() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext);
    assert!(a.shortcut_lower.is_none());
}

// ─────────────────────────────────────────────
// 23. score_action: combined bonus stacking variations
// ─────────────────────────────────────────────

#[test]
fn cat26_23_score_prefix_match_at_least_100() {
    let a = Action::new("id", "Copy Path", None, ActionCategory::ScriptContext);
    let score = super::ActionsDialog::score_action(&a, "copy");
    assert!(score >= 100);
}

#[test]
fn cat26_23_score_contains_match_50_to_99() {
    let a = Action::new("id", "My Copy Path", None, ActionCategory::ScriptContext);
    let score = super::ActionsDialog::score_action(&a, "copy");
    assert!(score >= 50);
    // It's a contains match not a prefix match
    assert!(score < 100 || a.title_lower.starts_with("copy"));
}

#[test]
fn cat26_23_score_no_match_zero() {
    let a = Action::new("id", "Delete", None, ActionCategory::ScriptContext);
    let score = super::ActionsDialog::score_action(&a, "xyz");
    assert_eq!(score, 0);
}

#[test]
fn cat26_23_score_empty_search_is_prefix() {
    let a = Action::new("id", "Anything", None, ActionCategory::ScriptContext);
    let score = super::ActionsDialog::score_action(&a, "");
    assert!(score >= 100, "Empty search should match as prefix");
}

// ─────────────────────────────────────────────
// 24. fuzzy_match: various patterns
// ─────────────────────────────────────────────

#[test]
fn cat26_24_fuzzy_exact_match() {
    assert!(super::ActionsDialog::fuzzy_match("hello", "hello"));
}

#[test]
fn cat26_24_fuzzy_subsequence_match() {
    assert!(super::ActionsDialog::fuzzy_match("hello world", "hlo"));
}

#[test]
fn cat26_24_fuzzy_no_match() {
    assert!(!super::ActionsDialog::fuzzy_match("abc", "abd"));
}

#[test]
fn cat26_24_fuzzy_empty_needle_matches() {
    assert!(super::ActionsDialog::fuzzy_match("anything", ""));
}

// ─────────────────────────────────────────────
// 25. Clipboard: paste description mentions clipboard
// ─────────────────────────────────────────────

#[test]
fn cat26_25_paste_desc_mentions_clipboard() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hi".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
    assert!(paste
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("clipboard"));
}

#[test]
fn cat26_25_copy_desc_mentions_clipboard() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hi".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let copy = actions.iter().find(|a| a.id == "clipboard_copy").unwrap();
    assert!(copy
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("clipboard"));
}

#[test]
fn cat26_25_paste_keep_open_desc_mentions_keep() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hi".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let keep = actions
        .iter()
        .find(|a| a.id == "clipboard_paste_keep_open")
        .unwrap();
    assert!(keep
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("keep"));
}

// ─────────────────────────────────────────────
// 26. Script context: shortcut toggle (add vs update/remove)
// ─────────────────────────────────────────────

#[test]
fn cat26_26_no_shortcut_shows_add_shortcut() {
    let s = ScriptInfo::new("s", "/s.ts");
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "add_shortcut"));
    assert!(!actions.iter().any(|a| a.id == "update_shortcut"));
    assert!(!actions.iter().any(|a| a.id == "remove_shortcut"));
}

#[test]
fn cat26_26_with_shortcut_shows_update_and_remove() {
    let s = ScriptInfo::with_shortcut("s", "/s.ts", Some("cmd+s".into()));
    let actions = get_script_context_actions(&s);
    assert!(!actions.iter().any(|a| a.id == "add_shortcut"));
    assert!(actions.iter().any(|a| a.id == "update_shortcut"));
    assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
}

#[test]
fn cat26_26_no_alias_shows_add_alias() {
    let s = ScriptInfo::new("s", "/s.ts");
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "add_alias"));
    assert!(!actions.iter().any(|a| a.id == "update_alias"));
    assert!(!actions.iter().any(|a| a.id == "remove_alias"));
}

#[test]
fn cat26_26_with_alias_shows_update_and_remove() {
    let s = ScriptInfo::with_shortcut_and_alias("s", "/s.ts", None, Some("al".into()));
    let actions = get_script_context_actions(&s);
    assert!(!actions.iter().any(|a| a.id == "add_alias"));
    assert!(actions.iter().any(|a| a.id == "update_alias"));
    assert!(actions.iter().any(|a| a.id == "remove_alias"));
}

// ─────────────────────────────────────────────
// 27. Notes command bar: find_in_note section and icon
// ─────────────────────────────────────────────

#[test]
fn cat26_27_notes_find_in_note_section() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let find = actions.iter().find(|a| a.id == "find_in_note").unwrap();
    assert_eq!(find.section.as_deref(), Some("Edit"));
}
