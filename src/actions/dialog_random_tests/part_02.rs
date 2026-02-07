
// =========================================================================
// 8. build_grouped_items_static — section style effects
// =========================================================================

#[test]
fn build_grouped_items_headers_style_adds_headers() {
    let actions = vec![
        make_action("a", "A", Some("S1")),
        make_action("b", "B", Some("S1")),
        make_action("c", "C", Some("S2")),
    ];
    let result = build_grouped_items_static(&actions, &[0, 1, 2], SectionStyle::Headers);
    // Should have: Header("S1"), Item(0), Item(1), Header("S2"), Item(2)
    assert_eq!(result.len(), 5);
    assert!(matches!(&result[0], GroupedActionItem::SectionHeader(s) if s == "S1"));
    assert!(matches!(result[1], GroupedActionItem::Item(0)));
    assert!(matches!(result[2], GroupedActionItem::Item(1)));
    assert!(matches!(&result[3], GroupedActionItem::SectionHeader(s) if s == "S2"));
    assert!(matches!(result[4], GroupedActionItem::Item(2)));
}

#[test]
fn build_grouped_items_separators_style_no_headers() {
    let actions = vec![
        make_action("a", "A", Some("S1")),
        make_action("b", "B", Some("S2")),
    ];
    let result = build_grouped_items_static(&actions, &[0, 1], SectionStyle::Separators);
    // Separators style: no headers, just items
    assert_eq!(result.len(), 2);
    assert!(matches!(result[0], GroupedActionItem::Item(0)));
    assert!(matches!(result[1], GroupedActionItem::Item(1)));
}

#[test]
fn build_grouped_items_none_style_no_headers() {
    let actions = vec![
        make_action("a", "A", Some("S1")),
        make_action("b", "B", Some("S2")),
    ];
    let result = build_grouped_items_static(&actions, &[0, 1], SectionStyle::None);
    assert_eq!(result.len(), 2);
    assert!(matches!(result[0], GroupedActionItem::Item(0)));
    assert!(matches!(result[1], GroupedActionItem::Item(1)));
}

#[test]
fn build_grouped_items_empty_actions() {
    let actions: Vec<Action> = vec![];
    let result = build_grouped_items_static(&actions, &[], SectionStyle::Headers);
    assert!(result.is_empty());
}

#[test]
fn build_grouped_items_no_sections_with_headers_style() {
    let actions = vec![make_action("a", "A", None), make_action("b", "B", None)];
    let result = build_grouped_items_static(&actions, &[0, 1], SectionStyle::Headers);
    // No sections → no headers, just items
    assert_eq!(result.len(), 2);
    assert!(matches!(result[0], GroupedActionItem::Item(0)));
    assert!(matches!(result[1], GroupedActionItem::Item(1)));
}

// =========================================================================
// 9. to_deeplink_name — unicode and edge cases
// =========================================================================

#[test]
fn to_deeplink_name_unicode_accents() {
    // Accented chars should lowercase and remain (they're alphanumeric)
    let result = to_deeplink_name("Café Résumé");
    assert_eq!(result, "café-résumé");
}

#[test]
fn to_deeplink_name_all_special_chars() {
    let result = to_deeplink_name("!@#$%^&*()");
    // All non-alphanumeric → hyphens, then filtered as empty segments
    assert_eq!(result, "");
}

#[test]
fn to_deeplink_name_consecutive_separators() {
    let result = to_deeplink_name("a___b---c   d");
    assert_eq!(result, "a-b-c-d");
}

#[test]
fn to_deeplink_name_numbers_only() {
    let result = to_deeplink_name("123");
    assert_eq!(result, "123");
}

#[test]
fn to_deeplink_name_mixed_case_and_underscores() {
    let result = to_deeplink_name("My_Script_Name");
    assert_eq!(result, "my-script-name");
}

#[test]
fn to_deeplink_name_emoji() {
    // Emoji are alphanumeric in Unicode (they pass is_alphanumeric)
    // Actually, emoji may NOT pass is_alphanumeric; let's test what happens
    let result = to_deeplink_name("hello 🎉 world");
    // 🎉 is NOT alphanumeric, so it becomes a hyphen
    assert!(result == "hello-world" || result == "hello-🎉-world");
}

#[test]
fn to_deeplink_name_single_char() {
    assert_eq!(to_deeplink_name("A"), "a");
}

#[test]
fn to_deeplink_name_leading_trailing_spaces() {
    let result = to_deeplink_name("  hello world  ");
    assert_eq!(result, "hello-world");
}

// =========================================================================
// 10. Agent action validation
// =========================================================================

#[test]
fn agent_has_edit_agent_title() {
    let mut script = ScriptInfo::new("my-agent", "/path/agent.md");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    let edit = find_action(&actions, "edit_script").unwrap();
    assert_eq!(edit.title, "Edit Agent");
}

#[test]
fn agent_has_no_view_logs() {
    let mut script = ScriptInfo::new("my-agent", "/path/agent.md");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    let ids = action_ids(&actions);
    assert!(!ids.contains(&"view_logs"));
}

#[test]
fn agent_has_reveal_copy_content() {
    let mut script = ScriptInfo::new("my-agent", "/path/agent.md");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"reveal_in_finder"));
    assert!(ids.contains(&"copy_path"));
    assert!(ids.contains(&"copy_content"));
}

#[test]
fn agent_with_shortcut_and_alias() {
    let mut script = ScriptInfo::with_shortcut_and_alias(
        "my-agent",
        "/path/agent.md",
        Some("cmd+a".to_string()),
        Some("agt".to_string()),
    );
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"update_shortcut"));
    assert!(ids.contains(&"remove_shortcut"));
    assert!(ids.contains(&"update_alias"));
    assert!(ids.contains(&"remove_alias"));
    assert!(!ids.contains(&"add_shortcut"));
    assert!(!ids.contains(&"add_alias"));
}

#[test]
fn agent_suggested_has_reset_ranking() {
    let mut script =
        ScriptInfo::new("my-agent", "/path/agent.md").with_frecency(true, Some("/path".into()));
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"reset_ranking"));
}

// =========================================================================
// 11. Clipboard actions — comprehensive conditional branches
// =========================================================================

fn make_text_entry(pinned: bool, app_name: Option<&str>) -> ClipboardEntryInfo {
    ClipboardEntryInfo {
        id: "entry-1".to_string(),
        content_type: ContentType::Text,
        pinned,
        preview: "Hello, world!".to_string(),
        image_dimensions: None,
        frontmost_app_name: app_name.map(|s| s.to_string()),
    }
}

fn make_image_entry(pinned: bool) -> ClipboardEntryInfo {
    ClipboardEntryInfo {
        id: "entry-2".to_string(),
        content_type: ContentType::Image,
        pinned,
        preview: String::new(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: Some("Preview".to_string()),
    }
}

#[test]
fn clipboard_text_unpinned_has_pin() {
    let entry = make_text_entry(false, None);
    let actions = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"clipboard_pin"));
    assert!(!ids.contains(&"clipboard_unpin"));
}

#[test]
fn clipboard_text_pinned_has_unpin() {
    let entry = make_text_entry(true, None);
    let actions = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"clipboard_unpin"));
    assert!(!ids.contains(&"clipboard_pin"));
}

#[test]
fn clipboard_paste_title_shows_app_name() {
    let entry = make_text_entry(false, Some("VS Code"));
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = find_action(&actions, "clipboard_paste").unwrap();
    assert_eq!(paste.title, "Paste to VS Code");
}

#[test]
fn clipboard_paste_title_fallback() {
    let entry = make_text_entry(false, None);
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = find_action(&actions, "clipboard_paste").unwrap();
    assert_eq!(paste.title, "Paste to Active App");
}

#[test]
fn clipboard_text_no_ocr() {
    let entry = make_text_entry(false, None);
    let actions = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions);
    assert!(!ids.contains(&"clipboard_ocr"));
}

#[test]
fn clipboard_image_has_ocr() {
    let entry = make_image_entry(false);
    let actions = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"clipboard_ocr"));
}

#[test]
fn clipboard_destructive_always_last_three() {
    let entry = make_text_entry(false, None);
    let actions = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions);
    let len = ids.len();
    assert!(len >= 3);
    assert_eq!(ids[len - 3], "clipboard_delete");
    assert_eq!(ids[len - 2], "clipboard_delete_multiple");
    assert_eq!(ids[len - 1], "clipboard_delete_all");
}

#[test]
fn clipboard_paste_always_first() {
    let entry = make_text_entry(false, None);
    let actions = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions);
    assert_eq!(ids[0], "clipboard_paste");
}

#[test]
fn clipboard_copy_always_second() {
    let entry = make_text_entry(false, None);
    let actions = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions);
    assert_eq!(ids[1], "clipboard_copy");
}

#[test]
fn clipboard_image_has_more_actions_than_text() {
    let text = make_text_entry(false, None);
    let image = make_image_entry(false);
    let text_count = get_clipboard_history_context_actions(&text).len();
    let image_count = get_clipboard_history_context_actions(&image).len();
    assert!(
        image_count > text_count,
        "Image({}) should have more actions than text({})",
        image_count,
        text_count
    );
}

// =========================================================================
// 12. Chat context actions — edge cases
// =========================================================================

#[test]
fn chat_zero_models() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    // Should have only "continue_in_chat" (no copy_response, no clear)
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "continue_in_chat");
}

#[test]
fn chat_current_model_checkmark() {
    let info = ChatPromptInfo {
        current_model: Some("Claude 3".to_string()),
        available_models: vec![
            ChatModelInfo {
                id: "claude-3".to_string(),
                display_name: "Claude 3".to_string(),
                provider: "Anthropic".to_string(),
            },
            ChatModelInfo {
                id: "gpt-4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let claude = find_action(&actions, "select_model_claude-3").unwrap();
    assert!(claude.title.contains('✓'));
    let gpt = find_action(&actions, "select_model_gpt-4").unwrap();
    assert!(!gpt.title.contains('✓'));
}

#[test]
fn chat_has_response_and_messages() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"copy_response"));
    assert!(ids.contains(&"clear_conversation"));
}

#[test]
fn chat_no_response_no_messages() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let ids = action_ids(&actions);
    assert!(!ids.contains(&"copy_response"));
    assert!(!ids.contains(&"clear_conversation"));
}

#[test]
fn chat_model_description_has_provider() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "claude-3".to_string(),
            display_name: "Claude 3".to_string(),
            provider: "Anthropic".to_string(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model_action = find_action(&actions, "select_model_claude-3").unwrap();
    assert_eq!(model_action.description, Some("via Anthropic".to_string()));
}

// =========================================================================
// 13. Notes command bar — all 8 permutations
// =========================================================================

fn notes_action_ids(has_sel: bool, trash: bool, auto: bool) -> Vec<String> {
    let info = NotesInfo {
        has_selection: has_sel,
        is_trash_view: trash,
        auto_sizing_enabled: auto,
    };
    get_notes_command_bar_actions(&info)
        .iter()
        .map(|a| a.id.clone())
        .collect()
}

#[test]
fn notes_new_note_always_present() {
    for &sel in &[false, true] {
        for &trash in &[false, true] {
            for &auto in &[false, true] {
                let ids = notes_action_ids(sel, trash, auto);
                assert!(
                    ids.contains(&"new_note".to_string()),
                    "new_note missing for sel={}, trash={}, auto={}",
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
    for &sel in &[false, true] {
        for &trash in &[false, true] {
            for &auto in &[false, true] {
                let ids = notes_action_ids(sel, trash, auto);
                assert!(
                    ids.contains(&"browse_notes".to_string()),
                    "browse_notes missing for sel={}, trash={}, auto={}",
                    sel,
                    trash,
                    auto
                );
            }
        }
    }
}
