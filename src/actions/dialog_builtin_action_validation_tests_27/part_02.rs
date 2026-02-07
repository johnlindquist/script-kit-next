
#[test]
fn cat27_08_note_switcher_singular_char_count() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "T".into(),
        char_count: 1,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("1 char"));
}

#[test]
fn cat27_08_note_switcher_zero_chars() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Empty".into(),
        char_count: 0,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("0 chars"));
}

// ─────────────────────────────────────────────
// 9. File context: copy_filename vs copy_path shortcut
// ─────────────────────────────────────────────

#[test]
fn cat27_09_file_copy_filename_shortcut_is_cmd_c() {
    let file = FileInfo {
        name: "test.txt".into(),
        path: "/tmp/test.txt".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&file);
    let copy_fn = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert_eq!(copy_fn.shortcut.as_deref(), Some("⌘C"));
}

#[test]
fn cat27_09_file_copy_path_shortcut_is_cmd_shift_c() {
    let file = FileInfo {
        name: "test.txt".into(),
        path: "/tmp/test.txt".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&file);
    let copy_p = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert_eq!(copy_p.shortcut.as_deref(), Some("⌘⇧C"));
}

#[test]
fn cat27_09_file_dir_copy_filename_also_cmd_c() {
    let dir = FileInfo {
        name: "mydir".into(),
        path: "/tmp/mydir".into(),
        is_dir: true,
        file_type: FileType::Directory,
    };
    let actions = get_file_context_actions(&dir);
    let copy_fn = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert_eq!(copy_fn.shortcut.as_deref(), Some("⌘C"));
}

// ─────────────────────────────────────────────
// 10. Path context: copy_filename has no shortcut
// ─────────────────────────────────────────────

#[test]
fn cat27_10_path_copy_filename_no_shortcut() {
    let path = PathInfo {
        name: "file.rs".into(),
        path: "/tmp/file.rs".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    let copy_fn = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert!(copy_fn.shortcut.is_none());
}

#[test]
fn cat27_10_path_copy_path_has_shortcut() {
    let path = PathInfo {
        name: "file.rs".into(),
        path: "/tmp/file.rs".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    let copy_p = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert_eq!(copy_p.shortcut.as_deref(), Some("⌘⇧C"));
}

#[test]
fn cat27_10_path_dir_copy_filename_still_no_shortcut() {
    let path = PathInfo {
        name: "src".into(),
        path: "/tmp/src".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path);
    let copy_fn = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert!(copy_fn.shortcut.is_none());
}

// ─────────────────────────────────────────────
// 11. format_shortcut_hint: intermediate modifier handling
// ─────────────────────────────────────────────

#[test]
fn cat27_11_format_hint_cmd_shift_c() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("cmd+shift+c"),
        "⌘⇧C"
    );
}

#[test]
fn cat27_11_format_hint_ctrl_alt_delete() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("ctrl+alt+delete"),
        "⌃⌥⌫"
    );
}

#[test]
fn cat27_11_format_hint_single_key_enter() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("enter"),
        "↵"
    );
}

#[test]
fn cat27_11_format_hint_super_alias() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("super+k"),
        "⌘K"
    );
}

#[test]
fn cat27_11_format_hint_option_space() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("option+space"),
        "⌥␣"
    );
}

// ─────────────────────────────────────────────
// 12. to_deeplink_name: mixed punctuation collapses
// ─────────────────────────────────────────────

#[test]
fn cat27_12_deeplink_mixed_punctuation_collapses() {
    assert_eq!(to_deeplink_name("a...b"), "a-b");
}

#[test]
fn cat27_12_deeplink_parens_and_brackets() {
    assert_eq!(to_deeplink_name("foo (bar) [baz]"), "foo-bar-baz");
}

#[test]
fn cat27_12_deeplink_ampersand_and_at() {
    assert_eq!(to_deeplink_name("copy & paste @ home"), "copy-paste-home");
}

#[test]
fn cat27_12_deeplink_slash_and_backslash() {
    assert_eq!(to_deeplink_name("path/to\\file"), "path-to-file");
}

// ─────────────────────────────────────────────
// 13. score_action: fuzzy bonus value
// ─────────────────────────────────────────────

#[test]
fn cat27_13_score_fuzzy_match_gives_25() {
    // "rn" is a subsequence of "run script" but not prefix or contains
    let action = Action::new(
        "run_script",
        "Run Script",
        None,
        ActionCategory::ScriptContext,
    );
    // "rp" → r...(u)(n)(space)(s)(c)(r)(i)(p) - subsequence r,p
    let score = super::dialog::ActionsDialog::score_action(&action, "rp");
    // fuzzy match gives 25
    assert_eq!(score, 25);
}

#[test]
fn cat27_13_score_prefix_gives_at_least_100() {
    let action = Action::new("edit", "Edit Script", None, ActionCategory::ScriptContext);
    let score = super::dialog::ActionsDialog::score_action(&action, "edit");
    assert!(score >= 100);
}

#[test]
fn cat27_13_score_contains_gives_50() {
    let action = Action::new("test", "Open Editor", None, ActionCategory::ScriptContext);
    let score = super::dialog::ActionsDialog::score_action(&action, "editor");
    // "editor" is contained in "open editor" but not a prefix
    assert!(score >= 50);
}

#[test]
fn cat27_13_score_no_match_gives_0() {
    let action = Action::new("test", "Run Script", None, ActionCategory::ScriptContext);
    let score = super::dialog::ActionsDialog::score_action(&action, "xyz");
    assert_eq!(score, 0);
}

// ─────────────────────────────────────────────
// 14. build_grouped_items_static: None section in Headers mode
// ─────────────────────────────────────────────

#[test]
fn cat27_14_headers_mode_no_section_skips_header() {
    let actions = vec![
        Action::new("a", "Alpha", None, ActionCategory::ScriptContext),
        Action::new("b", "Beta", None, ActionCategory::ScriptContext),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // No sections → no headers, just items
    assert_eq!(grouped.len(), 2);
    assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
    assert!(matches!(grouped[1], GroupedActionItem::Item(1)));
}

#[test]
fn cat27_14_headers_mode_with_section_adds_header() {
    let actions = vec![
        Action::new("a", "Alpha", None, ActionCategory::ScriptContext).with_section("Group A"),
        Action::new("b", "Beta", None, ActionCategory::ScriptContext).with_section("Group A"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // 1 header + 2 items
    assert_eq!(grouped.len(), 3);
    assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(_)));
}

#[test]
fn cat27_14_headers_mode_two_sections_two_headers() {
    let actions = vec![
        Action::new("a", "Alpha", None, ActionCategory::ScriptContext).with_section("X"),
        Action::new("b", "Beta", None, ActionCategory::ScriptContext).with_section("Y"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // 2 headers + 2 items
    assert_eq!(grouped.len(), 4);
    let header_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(header_count, 2);
}

#[test]
fn cat27_14_separators_mode_never_adds_headers() {
    let actions = vec![
        Action::new("a", "Alpha", None, ActionCategory::ScriptContext).with_section("X"),
        Action::new("b", "Beta", None, ActionCategory::ScriptContext).with_section("Y"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    // No headers in Separators mode
    assert_eq!(grouped.len(), 2);
}

// ─────────────────────────────────────────────
// 15. CommandBarConfig: notes_style uses Separators
// ─────────────────────────────────────────────

#[test]
fn cat27_15_notes_style_uses_separators() {
    let cfg = CommandBarConfig::notes_style();
    assert_eq!(cfg.dialog_config.section_style, SectionStyle::Separators);
}

#[test]
fn cat27_15_ai_style_uses_headers() {
    let cfg = CommandBarConfig::ai_style();
    assert_eq!(cfg.dialog_config.section_style, SectionStyle::Headers);
}

#[test]
fn cat27_15_main_menu_uses_separators() {
    let cfg = CommandBarConfig::main_menu_style();
    assert_eq!(cfg.dialog_config.section_style, SectionStyle::Separators);
}

#[test]
fn cat27_15_no_search_uses_hidden() {
    let cfg = CommandBarConfig::no_search();
    assert_eq!(cfg.dialog_config.search_position, SearchPosition::Hidden);
}

// ─────────────────────────────────────────────
// 16. Cross-context: first action shortcut is ↵
// ─────────────────────────────────────────────

#[test]
fn cat27_16_script_first_shortcut_is_enter() {
    let script = ScriptInfo::new("test", "/p.ts");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn cat27_16_clipboard_first_shortcut_is_enter() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn cat27_16_path_file_first_shortcut_is_enter() {
    let path = PathInfo {
        name: "f.txt".into(),
        path: "/f.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn cat27_16_file_context_first_shortcut_is_enter() {
    let file = FileInfo {
        name: "a.txt".into(),
        path: "/a.txt".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&file);
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

// ─────────────────────────────────────────────
// 17. Clipboard: image text action difference (image has OCR)
// ─────────────────────────────────────────────

#[test]
fn cat27_17_clipboard_image_has_ocr() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"clipboard_ocr"));
}

#[test]
fn cat27_17_clipboard_text_no_ocr() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(!ids.contains(&"clipboard_ocr"));
}

#[test]
fn cat27_17_clipboard_image_more_actions_than_text() {
    let text_entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let img_entry = ClipboardEntryInfo {
        id: "2".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "".into(),
        image_dimensions: Some((10, 10)),
        frontmost_app_name: None,
    };
    let text_actions = get_clipboard_history_context_actions(&text_entry);
    let img_actions = get_clipboard_history_context_actions(&img_entry);
    assert!(img_actions.len() > text_actions.len());
}

// ─────────────────────────────────────────────
// 18. coerce_action_selection: all items selectable
// ─────────────────────────────────────────────

#[test]
fn cat27_18_coerce_all_items_stays_at_index() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::Item(1),
        GroupedActionItem::Item(2),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(1));
}

#[test]
fn cat27_18_coerce_index_beyond_len_clamps() {
    let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
    // ix=10 → clamped to len-1 = 1
    assert_eq!(coerce_action_selection(&rows, 10), Some(1));
}

#[test]
fn cat27_18_coerce_header_at_0_jumps_to_1() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}
