
// =====================================================================
// 20. Action::with_shortcut_opt with None leaves shortcut unset
// =====================================================================

#[test]
fn action_with_shortcut_opt_none_leaves_none() {
    let action =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
    assert!(action.shortcut.is_none());
    assert!(action.shortcut_lower.is_none());
}

#[test]
fn action_with_shortcut_opt_some_sets_both() {
    let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
        .with_shortcut_opt(Some("⌘X".into()));
    assert_eq!(action.shortcut.as_deref(), Some("⌘X"));
    assert_eq!(action.shortcut_lower.as_deref(), Some("⌘x"));
}

#[test]
fn action_with_shortcut_sets_shortcut_lower() {
    let action =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut("⇧⌘K");
    assert_eq!(action.shortcut.as_deref(), Some("⇧⌘K"));
    assert_eq!(action.shortcut_lower.as_deref(), Some("⇧⌘k"));
}

#[test]
fn action_new_has_no_shortcut_by_default() {
    let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
    assert!(action.shortcut.is_none());
    assert!(action.shortcut_lower.is_none());
}

// =====================================================================
// 21. Note switcher: section assignment based on pinned status
// =====================================================================

#[test]
fn note_switcher_pinned_section_is_pinned() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid-1".into(),
        title: "Pinned Note".into(),
        char_count: 100,
        is_current: false,
        is_pinned: true,
        preview: "Some text".into(),
        relative_time: "1m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
}

#[test]
fn note_switcher_unpinned_section_is_recent() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid-2".into(),
        title: "Regular Note".into(),
        char_count: 50,
        is_current: false,
        is_pinned: false,
        preview: "Some text".into(),
        relative_time: "2h ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Recent"));
}

#[test]
fn note_switcher_mixed_pinned_and_recent() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "a".into(),
            title: "A".into(),
            char_count: 10,
            is_current: false,
            is_pinned: true,
            preview: "".into(),
            relative_time: "".into(),
        },
        NoteSwitcherNoteInfo {
            id: "b".into(),
            title: "B".into(),
            char_count: 20,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
    assert_eq!(actions[1].section.as_deref(), Some("Recent"));
}

// =====================================================================
// 22. Clipboard: pin/unpin share the same shortcut ⇧⌘P
// =====================================================================

#[test]
fn clipboard_pin_shortcut_shift_cmd_p() {
    let entry = ClipboardEntryInfo {
        id: "pin-1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let pin = actions.iter().find(|a| a.id == "clipboard_pin").unwrap();
    assert_eq!(pin.shortcut.as_deref(), Some("⇧⌘P"));
}

#[test]
fn clipboard_unpin_shortcut_shift_cmd_p() {
    let entry = ClipboardEntryInfo {
        id: "pin-2".into(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "test".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let unpin = actions.iter().find(|a| a.id == "clipboard_unpin").unwrap();
    assert_eq!(unpin.shortcut.as_deref(), Some("⇧⌘P"));
}

#[test]
fn clipboard_pin_title_is_pin_entry() {
    let entry = ClipboardEntryInfo {
        id: "pin-3".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let pin = actions.iter().find(|a| a.id == "clipboard_pin").unwrap();
    assert_eq!(pin.title, "Pin Entry");
}

#[test]
fn clipboard_unpin_title_is_unpin_entry() {
    let entry = ClipboardEntryInfo {
        id: "pin-4".into(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "test".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let unpin = actions.iter().find(|a| a.id == "clipboard_unpin").unwrap();
    assert_eq!(unpin.title, "Unpin Entry");
}

// =====================================================================
// 23. Script context: agent action set details
// =====================================================================

#[test]
fn agent_edit_title_is_edit_agent() {
    let mut script = ScriptInfo::new("MyAgent", "/path/agent.md");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.title, "Edit Agent");
}

#[test]
fn agent_edit_desc_mentions_agent_file() {
    let mut script = ScriptInfo::new("MyAgent", "/path/agent.md");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert!(edit.description.as_ref().unwrap().contains("agent"));
}

#[test]
fn agent_has_reveal_in_finder() {
    let mut script = ScriptInfo::new("MyAgent", "/path/agent.md");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
}

#[test]
fn agent_has_no_view_logs() {
    let mut script = ScriptInfo::new("MyAgent", "/path/agent.md");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

// =====================================================================
// 24. New chat: preset icon is preserved
// =====================================================================

#[test]
fn new_chat_preset_icon_preserved() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].icon, Some(IconName::Star));
}

#[test]
fn new_chat_preset_section_is_presets() {
    let presets = vec![NewChatPresetInfo {
        id: "code".into(),
        name: "Code".into(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].section.as_deref(), Some("Presets"));
}

#[test]
fn new_chat_preset_desc_is_none() {
    let presets = vec![NewChatPresetInfo {
        id: "code".into(),
        name: "Code".into(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert!(actions[0].description.is_none());
}

#[test]
fn new_chat_model_desc_is_provider_display_name() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt-4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].description.as_deref(), Some("OpenAI"));
}

// =====================================================================
// 25. Note switcher: empty preview uses relative_time or char count
// =====================================================================

#[test]
fn note_switcher_empty_preview_with_time_shows_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n1".into(),
        title: "Note".into(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "5m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("5m ago"));
}

#[test]
fn note_switcher_empty_preview_empty_time_shows_chars() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n2".into(),
        title: "Note".into(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("42 chars"));
}

#[test]
fn note_switcher_preview_with_time_has_separator() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n3".into(),
        title: "Note".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "Hello world".into(),
        relative_time: "1h ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(desc.contains(" · "));
    assert!(desc.contains("Hello world"));
    assert!(desc.contains("1h ago"));
}

#[test]
fn note_switcher_preview_without_time_no_separator() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n4".into(),
        title: "Note".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "Hello world".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(!desc.contains(" · "));
    assert_eq!(desc, "Hello world");
}

// =====================================================================
// 26. Clipboard: upload_cleanshot shortcut is ⇧⌘U (macOS only)
// =====================================================================

#[cfg(target_os = "macos")]
#[test]
fn clipboard_upload_cleanshot_shortcut_shift_cmd_u() {
    let entry = ClipboardEntryInfo {
        id: "uc-1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let upload = actions
        .iter()
        .find(|a| a.id == "clipboard_upload_cleanshot")
        .unwrap();
    assert_eq!(upload.shortcut.as_deref(), Some("⇧⌘U"));
}

#[cfg(target_os = "macos")]
#[test]
fn clipboard_upload_cleanshot_title() {
    let entry = ClipboardEntryInfo {
        id: "uc-2".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let upload = actions
        .iter()
        .find(|a| a.id == "clipboard_upload_cleanshot")
        .unwrap();
    assert_eq!(upload.title, "Upload to CleanShot X");
}

#[cfg(target_os = "macos")]
#[test]
fn clipboard_upload_cleanshot_desc_mentions_cloud() {
    let entry = ClipboardEntryInfo {
        id: "uc-3".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let upload = actions
        .iter()
        .find(|a| a.id == "clipboard_upload_cleanshot")
        .unwrap();
    assert!(upload.description.as_ref().unwrap().contains("Cloud"));
}

#[cfg(target_os = "macos")]
#[test]
fn clipboard_upload_cleanshot_absent_for_text() {
    let entry = ClipboardEntryInfo {
        id: "uc-4".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_upload_cleanshot"));
}

// =====================================================================
// 27. Path context: open_in_editor and open_in_finder details
// =====================================================================

#[test]
fn path_open_in_editor_shortcut_cmd_e() {
    let path_info = PathInfo {
        path: "/tmp/code.rs".into(),
        name: "code.rs".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let editor = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
    assert_eq!(editor.shortcut.as_deref(), Some("⌘E"));
}

#[test]
fn path_open_in_editor_desc_mentions_editor() {
    let path_info = PathInfo {
        path: "/tmp/code.rs".into(),
        name: "code.rs".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let editor = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
    assert!(editor.description.as_ref().unwrap().contains("$EDITOR"));
}

#[test]
fn path_open_in_finder_shortcut_cmd_shift_f() {
    let path_info = PathInfo {
        path: "/tmp/code.rs".into(),
        name: "code.rs".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let finder = actions.iter().find(|a| a.id == "open_in_finder").unwrap();
    assert_eq!(finder.shortcut.as_deref(), Some("⌘⇧F"));
}

#[test]
fn path_open_in_finder_desc_mentions_finder() {
    let path_info = PathInfo {
        path: "/tmp/code.rs".into(),
        name: "code.rs".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let finder = actions.iter().find(|a| a.id == "open_in_finder").unwrap();
    assert!(finder.description.as_ref().unwrap().contains("Finder"));
}
