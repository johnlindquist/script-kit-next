
// =====================================================================
// 13. CommandBarConfig: show_icons and show_footer presets
// =====================================================================

#[test]
fn command_bar_ai_shows_icons_and_footer() {
    let config = CommandBarConfig::ai_style();
    assert!(config.dialog_config.show_icons);
    assert!(config.dialog_config.show_footer);
}

#[test]
fn command_bar_main_menu_hides_icons_and_footer() {
    let config = CommandBarConfig::main_menu_style();
    assert!(!config.dialog_config.show_icons);
    assert!(!config.dialog_config.show_footer);
}

#[test]
fn command_bar_notes_shows_icons_and_footer() {
    let config = CommandBarConfig::notes_style();
    assert!(config.dialog_config.show_icons);
    assert!(config.dialog_config.show_footer);
}

#[test]
fn command_bar_no_search_hides_icons_and_footer() {
    let config = CommandBarConfig::no_search();
    assert!(!config.dialog_config.show_icons);
    assert!(!config.dialog_config.show_footer);
}

// =====================================================================
// 14. CommandBarConfig: close flag defaults
// =====================================================================

#[test]
fn command_bar_default_close_flags_all_true() {
    let config = CommandBarConfig::default();
    assert!(config.close_on_select);
    assert!(config.close_on_click_outside);
    assert!(config.close_on_escape);
}

#[test]
fn command_bar_ai_close_flags_inherited() {
    let config = CommandBarConfig::ai_style();
    assert!(config.close_on_select);
    assert!(config.close_on_click_outside);
    assert!(config.close_on_escape);
}

#[test]
fn command_bar_main_menu_close_flags_inherited() {
    let config = CommandBarConfig::main_menu_style();
    assert!(config.close_on_select);
    assert!(config.close_on_click_outside);
    assert!(config.close_on_escape);
}

#[test]
fn command_bar_notes_close_flags_inherited() {
    let config = CommandBarConfig::notes_style();
    assert!(config.close_on_select);
    assert!(config.close_on_click_outside);
    assert!(config.close_on_escape);
}

// =====================================================================
// 15. AI command bar: paste_image details
// =====================================================================

#[test]
fn ai_command_bar_paste_image_shortcut() {
    let actions = get_ai_command_bar_actions();
    let action = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert_eq!(action.shortcut.as_ref().unwrap(), "⌘V");
}

#[test]
fn ai_command_bar_paste_image_icon() {
    let actions = get_ai_command_bar_actions();
    let action = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert_eq!(action.icon, Some(IconName::File));
}

#[test]
fn ai_command_bar_paste_image_section() {
    let actions = get_ai_command_bar_actions();
    let action = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert_eq!(action.section.as_deref(), Some("Attachments"));
}

#[test]
fn ai_command_bar_paste_image_desc_mentions_clipboard() {
    let actions = get_ai_command_bar_actions();
    let action = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert!(action
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("clipboard"));
}

// =====================================================================
// 16. AI command bar: section distribution (count per section)
// =====================================================================

#[test]
fn ai_command_bar_response_section_has_3_actions() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .count();
    assert_eq!(count, 3);
}

#[test]
fn ai_command_bar_actions_section_has_4_actions() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Actions"))
        .count();
    assert_eq!(count, 4);
}

#[test]
fn ai_command_bar_attachments_section_has_2_actions() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Attachments"))
        .count();
    assert_eq!(count, 2);
}

#[test]
fn ai_command_bar_total_is_12() {
    let actions = get_ai_command_bar_actions();
    assert_eq!(actions.len(), 12);
}

// =====================================================================
// 17. to_deeplink_name: edge cases with unicode and special chars
// =====================================================================

#[test]
fn to_deeplink_name_with_parentheses_and_ampersand() {
    assert_eq!(to_deeplink_name("Copy & Paste (v2)"), "copy-paste-v2");
}

#[test]
fn to_deeplink_name_with_dots_and_slashes() {
    assert_eq!(to_deeplink_name("file.txt/path"), "file-txt-path");
}

#[test]
fn to_deeplink_name_only_special_chars() {
    assert_eq!(to_deeplink_name("!@#$%^&*()"), "");
}

#[test]
fn to_deeplink_name_already_hyphenated() {
    assert_eq!(to_deeplink_name("my-script"), "my-script");
}

// =====================================================================
// 18. Script context: exact action ordering for plain script
// =====================================================================

#[test]
fn script_context_first_action_is_run_script() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn script_context_last_action_is_copy_deeplink_without_suggestion() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions.last().unwrap().id, "copy_deeplink");
}

#[test]
fn script_context_last_action_is_reset_ranking_with_suggestion() {
    let script = ScriptInfo::new("test", "/path/test.ts")
        .with_frecency(true, Some("/path/test.ts".to_string()));
    let actions = get_script_context_actions(&script);
    assert_eq!(actions.last().unwrap().id, "reset_ranking");
}

#[test]
fn script_context_action_count_no_shortcut_no_alias() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    // run + add_shortcut + add_alias + edit + view_logs + reveal + copy_path + copy_content + copy_deeplink = 9
    assert_eq!(actions.len(), 9);
}

// =====================================================================
// 19. Script context: agent-specific descriptions mention "agent"
// =====================================================================

#[test]
fn agent_edit_title_is_edit_agent() {
    let mut agent = ScriptInfo::new("My Agent", "/path/agent.ts");
    agent.is_script = false;
    agent.is_agent = true;
    let actions = get_script_context_actions(&agent);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.title, "Edit Agent");
}

#[test]
fn agent_edit_desc_mentions_agent_file() {
    let mut agent = ScriptInfo::new("My Agent", "/path/agent.ts");
    agent.is_script = false;
    agent.is_agent = true;
    let actions = get_script_context_actions(&agent);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert!(edit
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("agent"));
}

#[test]
fn agent_reveal_desc_mentions_agent() {
    let mut agent = ScriptInfo::new("My Agent", "/path/agent.ts");
    agent.is_script = false;
    agent.is_agent = true;
    let actions = get_script_context_actions(&agent);
    let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
    assert!(reveal
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("agent"));
}

#[test]
fn agent_has_no_view_logs() {
    let mut agent = ScriptInfo::new("My Agent", "/path/agent.ts");
    agent.is_script = false;
    agent.is_agent = true;
    let actions = get_script_context_actions(&agent);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

// =====================================================================
// 20. Clipboard: share shortcut and section for both text and image
// =====================================================================

#[test]
fn clipboard_share_shortcut_is_shift_cmd_e() {
    let entry = ClipboardEntryInfo {
        id: "t".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let share = actions.iter().find(|a| a.id == "clipboard_share").unwrap();
    assert_eq!(share.shortcut.as_ref().unwrap(), "⇧⌘E");
}

#[test]
fn clipboard_share_title_is_share() {
    let entry = ClipboardEntryInfo {
        id: "t".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let share = actions.iter().find(|a| a.id == "clipboard_share").unwrap();
    assert_eq!(share.title, "Share...");
}

#[test]
fn clipboard_share_present_for_image() {
    let entry = ClipboardEntryInfo {
        id: "i".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".to_string(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_share"));
}

#[test]
fn clipboard_share_desc_mentions_share() {
    let entry = ClipboardEntryInfo {
        id: "t".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let share = actions.iter().find(|a| a.id == "clipboard_share").unwrap();
    assert!(share
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("share"));
}

// =====================================================================
// 21. Note switcher: char count singular vs plural
// =====================================================================

#[test]
fn note_switcher_zero_chars_plural() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "id1".to_string(),
        title: "Note".to_string(),
        char_count: 0,
        is_current: false,
        is_pinned: false,
        preview: "".to_string(),
        relative_time: "".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("0 chars"));
}

#[test]
fn note_switcher_one_char_singular() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "id1".to_string(),
        title: "Note".to_string(),
        char_count: 1,
        is_current: false,
        is_pinned: false,
        preview: "".to_string(),
        relative_time: "".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("1 char"));
}

#[test]
fn note_switcher_many_chars_plural() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "id1".to_string(),
        title: "Note".to_string(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: "".to_string(),
        relative_time: "".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("42 chars"));
}

// =====================================================================
// 22. Note switcher: preview with relative time separator
// =====================================================================

#[test]
fn note_switcher_preview_with_time_has_dot_separator() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "id1".to_string(),
        title: "Note".to_string(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "Hello world".to_string(),
        relative_time: "2m ago".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(
        actions[0].description.as_deref(),
        Some("Hello world · 2m ago")
    );
}

#[test]
fn note_switcher_preview_without_time_no_separator() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "id1".to_string(),
        title: "Note".to_string(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "Hello world".to_string(),
        relative_time: "".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("Hello world"));
}

#[test]
fn note_switcher_no_preview_with_time_shows_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "id1".to_string(),
        title: "Note".to_string(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "".to_string(),
        relative_time: "5d ago".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("5d ago"));
}

// =====================================================================
// 23. Notes command bar: conditional action presence (selection + trash)
// =====================================================================

#[test]
fn notes_cmd_bar_no_selection_has_only_3_actions() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note + browse_notes + enable_auto_sizing = 3
    assert_eq!(actions.len(), 3);
}
