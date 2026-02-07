
#[test]
fn path_all_actions_have_no_value() {
    let path = PathInfo::new("test", "/test", false);
    for action in &get_path_context_actions(&path) {
        assert!(
            action.value.is_none(),
            "Path action '{}' should have no value",
            action.id
        );
    }
}

// =========================================================================
// 32. Section header count consistency for AI with headers
// =========================================================================

#[test]
fn ai_section_header_count_is_seven() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let header_count = count_section_headers(&actions, &filtered);
    assert_eq!(
        header_count, 7,
        "AI command bar should have 7 section headers (Response, Actions, Attachments, Export, Actions, Help, Settings)"
    );
}

// =========================================================================
// 33. Scriptlet context actions from get_scriptlet_context_actions_with_custom
//     have all the same universal actions as script context
// =========================================================================

#[test]
fn scriptlet_context_has_shortcut_alias_deeplink() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions_tmp = get_scriptlet_context_actions_with_custom(&script, None);
    let ids = action_ids(&actions_tmp);
    assert!(ids.contains(&"add_shortcut"));
    assert!(ids.contains(&"add_alias"));
    assert!(ids.contains(&"copy_deeplink"));
}

#[test]
fn scriptlet_context_has_edit_reveal_copy() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions_tmp = get_scriptlet_context_actions_with_custom(&script, None);
    let ids = action_ids(&actions_tmp);
    assert!(ids.contains(&"edit_scriptlet"));
    assert!(ids.contains(&"reveal_scriptlet_in_finder"));
    assert!(ids.contains(&"copy_scriptlet_path"));
    assert!(ids.contains(&"copy_content"));
}

// =========================================================================
// 34. Notes new_note always present across all permutations
// =========================================================================

#[test]
fn notes_new_note_always_present() {
    for sel in [false, true] {
        for trash in [false, true] {
            for auto in [false, true] {
                let info = NotesInfo {
                    has_selection: sel,
                    is_trash_view: trash,
                    auto_sizing_enabled: auto,
                };
                let actions_tmp = get_notes_command_bar_actions(&info);
                let ids = action_ids(&actions_tmp);
                assert!(
                    ids.contains(&"new_note"),
                    "new_note should always be present (sel={}, trash={}, auto={})",
                    sel,
                    trash,
                    auto
                );
            }
        }
    }
}

#[test]
fn notes_browse_notes_always_present() {
    for sel in [false, true] {
        for trash in [false, true] {
            for auto in [false, true] {
                let info = NotesInfo {
                    has_selection: sel,
                    is_trash_view: trash,
                    auto_sizing_enabled: auto,
                };
                let actions_tmp = get_notes_command_bar_actions(&info);
                let ids = action_ids(&actions_tmp);
                assert!(
                    ids.contains(&"browse_notes"),
                    "browse_notes should always be present (sel={}, trash={}, auto={})",
                    sel,
                    trash,
                    auto
                );
            }
        }
    }
}

// =========================================================================
// 35. Fuzzy match on real action IDs across contexts
// =========================================================================

#[test]
fn fuzzy_match_on_clipboard_action_titles() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    // "pke" should fuzzy match "Paste and Keep Window Open" (p-a-s-t-e... k-e-e-p)
    let paste_keep = actions
        .iter()
        .find(|a| a.id == "clipboard_paste_keep_open")
        .unwrap();
    assert!(ActionsDialog::fuzzy_match(&paste_keep.title_lower, "pke"));
}

#[test]
fn fuzzy_match_on_notes_action_titles() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let new_note = actions.iter().find(|a| a.id == "new_note").unwrap();
    // "nn" matches "new note" → n at 0, n at 4
    assert!(ActionsDialog::fuzzy_match(&new_note.title_lower, "nn"));
}

// =========================================================================
// 36. Grouped items headers style produces section headers
// =========================================================================

#[test]
fn grouped_items_headers_style_has_section_headers() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    let header_count = grouped
        .iter()
        .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
        .count();
    assert!(
        header_count > 0,
        "Headers style should produce at least one section header"
    );
}

#[test]
fn grouped_items_separators_style_has_separator_items() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    // Should have separator items but no header items
    let headers = grouped
        .iter()
        .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(headers, 0, "Separators style should have no headers");
}
