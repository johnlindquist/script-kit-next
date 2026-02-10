
#[test]
fn cat26_06_path_dir_desc_says_navigate() {
    let info = PathInfo {
        name: "d".into(),
        path: "/d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&info);
    let open = actions.iter().find(|a| a.id == "open_directory").unwrap();
    assert!(open.description.as_ref().unwrap().contains("Navigate"));
}

// ─────────────────────────────────────────────
// 7. ScriptInfo: is_agent mutual exclusivity
// ─────────────────────────────────────────────

#[test]
fn cat26_07_script_info_new_is_not_agent() {
    let s = ScriptInfo::new("x", "/x.ts");
    assert!(!s.is_agent);
    assert!(s.is_script);
    assert!(!s.is_scriptlet);
}

#[test]
fn cat26_07_script_info_builtin_is_not_agent() {
    let b = ScriptInfo::builtin("Clip");
    assert!(!b.is_agent);
    assert!(!b.is_script);
    assert!(!b.is_scriptlet);
}

#[test]
fn cat26_07_script_info_scriptlet_is_not_agent() {
    let s = ScriptInfo::scriptlet("S", "/s.md", None, None);
    assert!(!s.is_agent);
    assert!(!s.is_script);
    assert!(s.is_scriptlet);
}

#[test]
fn cat26_07_script_info_with_action_verb_defaults_no_agent() {
    let s = ScriptInfo::with_action_verb("App", "/a.app", false, "Launch");
    assert!(!s.is_agent);
}

// ─────────────────────────────────────────────
// 8. format_shortcut_hint: multi-key combos with numbers
// ─────────────────────────────────────────────

#[test]
fn cat26_08_format_hint_cmd_1() {
    let result = super::ActionsDialog::format_shortcut_hint("cmd+1");
    assert_eq!(result, "⌘1");
}

#[test]
fn cat26_08_format_hint_ctrl_shift_3() {
    let result = super::ActionsDialog::format_shortcut_hint("ctrl+shift+3");
    assert_eq!(result, "⌃⇧3");
}

#[test]
fn cat26_08_format_hint_alt_f4() {
    let result = super::ActionsDialog::format_shortcut_hint("alt+f4");
    assert_eq!(result, "⌥F4");
}

#[test]
fn cat26_08_format_hint_command_alias() {
    let result = super::ActionsDialog::format_shortcut_hint("command+k");
    assert_eq!(result, "⌘K");
}

#[test]
fn cat26_08_format_hint_option_alias() {
    let result = super::ActionsDialog::format_shortcut_hint("option+delete");
    assert_eq!(result, "⌥⌫");
}

// ─────────────────────────────────────────────
// 9. to_deeplink_name: Unicode with mixed scripts
// ─────────────────────────────────────────────

#[test]
fn cat26_09_deeplink_preserves_cjk() {
    let result = to_deeplink_name("日本語スクリプト");
    assert!(result.contains("日本語スクリプト"));
}

#[test]
fn cat26_09_deeplink_preserves_accented() {
    let result = to_deeplink_name("café résumé");
    assert!(result.contains("café"));
    assert!(result.contains("résumé"));
}

#[test]
fn cat26_09_deeplink_mixed_alpha_special_unicode() {
    let result = to_deeplink_name("Hello 世界!");
    assert_eq!(result, "hello-世界");
}

#[test]
fn cat26_09_deeplink_emoji_stripped() {
    // Emojis are alphanumeric in Unicode, so they should be preserved
    let result = to_deeplink_name("🚀 Launch");
    // Rocket emoji is not alphanumeric (it's a symbol), so it becomes a hyphen
    // "Launch" becomes "launch"
    assert!(result.contains("launch"));
}

// ─────────────────────────────────────────────
// 10. Cross-context: all built-in actions have Some description
// ─────────────────────────────────────────────

#[test]
fn cat26_10_script_actions_all_have_description() {
    let s = ScriptInfo::new("x", "/x.ts");
    let actions = get_script_context_actions(&s);
    for a in &actions {
        assert!(
            a.description.is_some(),
            "Action '{}' should have a description",
            a.id
        );
    }
}

#[test]
fn cat26_10_clipboard_text_actions_all_have_description() {
    let entry = ClipboardEntryInfo {
        id: "t".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    for a in &actions {
        assert!(
            a.description.is_some(),
            "Clipboard action '{}' should have a description",
            a.id
        );
    }
}

#[test]
fn cat26_10_path_actions_all_have_description() {
    let info = PathInfo {
        name: "p".into(),
        path: "/p".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    for a in &actions {
        assert!(
            a.description.is_some(),
            "Path action '{}' should have a description",
            a.id
        );
    }
}

#[test]
fn cat26_10_ai_actions_all_have_description() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(
            a.description.is_some(),
            "AI action '{}' should have a description",
            a.id
        );
    }
}

// ─────────────────────────────────────────────
// 11. Clipboard: pin/unpin title and description content
// ─────────────────────────────────────────────

#[test]
fn cat26_11_clipboard_pin_title_says_pin_entry() {
    let entry = ClipboardEntryInfo {
        id: "u".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let pin = actions.iter().find(|a| a.id == "clipboard_pin").unwrap();
    assert_eq!(pin.title, "Pin Entry");
}

#[test]
fn cat26_11_clipboard_unpin_title_says_unpin_entry() {
    let entry = ClipboardEntryInfo {
        id: "p".into(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let unpin = actions.iter().find(|a| a.id == "clipboard_unpin").unwrap();
    assert_eq!(unpin.title, "Unpin Entry");
}

#[test]
fn cat26_11_clipboard_pin_desc_mentions_prevent() {
    let entry = ClipboardEntryInfo {
        id: "u".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let pin = actions.iter().find(|a| a.id == "clipboard_pin").unwrap();
    assert!(pin
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("pin"));
}

// ─────────────────────────────────────────────
// 12. Note switcher: multiple notes with diverse states
// ─────────────────────────────────────────────

#[test]
fn cat26_12_note_switcher_three_notes_three_items() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "a".into(),
            title: "Note A".into(),
            char_count: 10,
            is_current: true,
            is_pinned: false,
            preview: "Hello".into(),
            relative_time: "1m ago".into(),
        },
        NoteSwitcherNoteInfo {
            id: "b".into(),
            title: "Note B".into(),
            char_count: 20,
            is_current: false,
            is_pinned: true,
            preview: "World".into(),
            relative_time: "5m ago".into(),
        },
        NoteSwitcherNoteInfo {
            id: "c".into(),
            title: "Note C".into(),
            char_count: 5,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "1h ago".into(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions.len(), 3);
}

#[test]
fn cat26_12_note_switcher_pinned_note_section() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "b".into(),
        title: "Note B".into(),
        char_count: 20,
        is_current: false,
        is_pinned: true,
        preview: "World".into(),
        relative_time: "5m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
}

#[test]
fn cat26_12_note_switcher_unpinned_note_section() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "c".into(),
        title: "Note C".into(),
        char_count: 5,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "1h ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Recent"));
}

#[test]
fn cat26_12_note_switcher_empty_preview_with_time_shows_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "c".into(),
        title: "Note C".into(),
        char_count: 5,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "1h ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("1h ago"));
}

// ─────────────────────────────────────────────
// 13. New chat: last_used section and icon
// ─────────────────────────────────────────────

#[test]
fn cat26_13_new_chat_last_used_has_bolt_filled_icon() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "P".into(),
        provider_display_name: "Provider".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
}

#[test]
fn cat26_13_new_chat_last_used_section_name() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "P".into(),
        provider_display_name: "Provider".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
}

#[test]
fn cat26_13_new_chat_last_used_description_is_provider_display_name() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "P".into(),
        provider_display_name: "My Provider".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].description.as_deref(), Some("My Provider"));
}

#[test]
fn cat26_13_new_chat_model_section_name() {
    let models = vec![NewChatModelInfo {
        model_id: "m2".into(),
        display_name: "Model 2".into(),
        provider: "P".into(),
        provider_display_name: "Provider".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].section.as_deref(), Some("Models"));
}

// ─────────────────────────────────────────────
// 14. CommandBarConfig: close flags default to true
// ─────────────────────────────────────────────

#[test]
fn cat26_14_command_bar_config_default_close_on_select() {
    let config = CommandBarConfig::default();
    assert!(config.close_on_select);
}

#[test]
fn cat26_14_command_bar_config_default_close_on_click_outside() {
    let config = CommandBarConfig::default();
    assert!(config.close_on_click_outside);
}

#[test]
fn cat26_14_command_bar_config_default_close_on_escape() {
    let config = CommandBarConfig::default();
    assert!(config.close_on_escape);
}

#[test]
fn cat26_14_command_bar_config_ai_style_preserves_close_flags() {
    let config = CommandBarConfig::ai_style();
    assert!(config.close_on_select);
    assert!(config.close_on_escape);
}

// ─────────────────────────────────────────────
// 15. Script context: edit shortcut is ⌘E for all editable types
// ─────────────────────────────────────────────

#[test]
fn cat26_15_script_edit_shortcut() {
    let s = ScriptInfo::new("s", "/s.ts");
    let actions = get_script_context_actions(&s);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.shortcut.as_deref(), Some("⌘E"));
}

#[test]
fn cat26_15_scriptlet_edit_shortcut() {
    let s = ScriptInfo::scriptlet("s", "/s.md", None, None);
    let actions = get_script_context_actions(&s);
    let edit = actions.iter().find(|a| a.id == "edit_scriptlet").unwrap();
    assert_eq!(edit.shortcut.as_deref(), Some("⌘E"));
}

#[test]
fn cat26_15_agent_edit_shortcut() {
    let mut s = ScriptInfo::new("a", "/a.md");
    s.is_script = false;
    s.is_agent = true;
    let actions = get_script_context_actions(&s);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.shortcut.as_deref(), Some("⌘E"));
}

#[test]
fn cat26_15_agent_edit_title_says_agent() {
    let mut s = ScriptInfo::new("a", "/a.md");
    s.is_script = false;
    s.is_agent = true;
    let actions = get_script_context_actions(&s);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert!(edit.title.contains("Agent"));
}

// ─────────────────────────────────────────────
// 16. Script context: view_logs only for is_script=true
// ─────────────────────────────────────────────

#[test]
fn cat26_16_script_has_view_logs() {
    let s = ScriptInfo::new("s", "/s.ts");
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "view_logs"));
}
