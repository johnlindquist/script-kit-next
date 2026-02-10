
#[test]
fn batch22_note_switcher_no_preview_uses_char_count() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert_eq!(desc, "42 chars");
}

#[test]
fn batch22_note_switcher_one_char_singular() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 1,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert_eq!(desc, "1 char");
}

#[test]
fn batch22_note_switcher_preview_truncation_at_60() {
    let long_preview = "a".repeat(70);
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 70,
        is_current: false,
        is_pinned: false,
        preview: long_preview,
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(desc.ends_with('…'));
}

// ============================================================
// 18. Note switcher: current bullet prefix
// ============================================================

#[test]
fn batch22_note_switcher_current_has_bullet() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "My Note".into(),
        char_count: 10,
        is_current: true,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(actions[0].title.starts_with("• "));
}

#[test]
fn batch22_note_switcher_non_current_no_bullet() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "My Note".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(!actions[0].title.starts_with("• "));
}

// ============================================================
// 19. Note switcher: section assignment by pin state
// ============================================================

#[test]
fn batch22_note_switcher_pinned_section() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 10,
        is_current: false,
        is_pinned: true,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
}

#[test]
fn batch22_note_switcher_unpinned_section() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Recent"));
}

// ============================================================
// 20. File context: file vs dir primary action IDs
// ============================================================

#[test]
fn batch22_file_context_file_primary_is_open_file() {
    let fi = FileInfo {
        name: "test.txt".into(),
        path: "/p/test.txt".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&fi);
    assert_eq!(actions[0].id, "open_file");
}

#[test]
fn batch22_file_context_dir_primary_is_open_directory() {
    let fi = FileInfo {
        name: "docs".into(),
        path: "/p/docs".into(),
        is_dir: true,
        file_type: FileType::Directory,
    };
    let actions = get_file_context_actions(&fi);
    assert_eq!(actions[0].id, "open_directory");
}

#[test]
fn batch22_file_context_always_has_reveal_in_finder() {
    let fi = FileInfo {
        name: "f".into(),
        path: "/p/f".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&fi);
    assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
}

#[test]
fn batch22_file_context_always_has_copy_path() {
    let fi = FileInfo {
        name: "f".into(),
        path: "/p/f".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&fi);
    assert!(actions.iter().any(|a| a.id == "copy_path"));
}

#[test]
fn batch22_file_context_copy_filename_has_shortcut() {
    let fi = FileInfo {
        name: "f".into(),
        path: "/p/f".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&fi);
    let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert_eq!(cf.shortcut.as_deref(), Some("⌘C"));
}

// ============================================================
// 21. Path context: dir vs file primary action, trash always last
// ============================================================

#[test]
fn batch22_path_dir_primary_is_open_directory() {
    let pi = PathInfo {
        name: "src".into(),
        path: "/src".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    assert_eq!(actions[0].id, "open_directory");
}

#[test]
fn batch22_path_file_primary_is_select_file() {
    let pi = PathInfo {
        name: "f.rs".into(),
        path: "/f.rs".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    assert_eq!(actions[0].id, "select_file");
}

#[test]
fn batch22_path_trash_is_always_last() {
    let pi_dir = PathInfo {
        name: "d".into(),
        path: "/d".into(),
        is_dir: true,
    };
    let pi_file = PathInfo {
        name: "f".into(),
        path: "/f".into(),
        is_dir: false,
    };
    let da = get_path_context_actions(&pi_dir);
    let fa = get_path_context_actions(&pi_file);
    assert_eq!(da.last().unwrap().id, "move_to_trash");
    assert_eq!(fa.last().unwrap().id, "move_to_trash");
}

#[test]
fn batch22_path_trash_desc_dir_says_folder() {
    let pi = PathInfo {
        name: "d".into(),
        path: "/d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert!(trash.description.as_ref().unwrap().contains("folder"));
}

#[test]
fn batch22_path_trash_desc_file_says_file() {
    let pi = PathInfo {
        name: "f".into(),
        path: "/f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert!(trash.description.as_ref().unwrap().contains("file"));
}

// ============================================================
// 22. Path context: copy_filename has no shortcut
// ============================================================

#[test]
fn batch22_path_copy_filename_no_shortcut() {
    let pi = PathInfo {
        name: "f".into(),
        path: "/f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert!(cf.shortcut.is_none());
}

#[test]
fn batch22_path_open_in_terminal_shortcut() {
    let pi = PathInfo {
        name: "f".into(),
        path: "/f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    let ot = actions.iter().find(|a| a.id == "open_in_terminal").unwrap();
    assert_eq!(ot.shortcut.as_deref(), Some("⌘T"));
}

// ============================================================
// 23. CommandBarConfig preset values
// ============================================================

#[test]
fn batch22_command_bar_default_bottom_search() {
    let cfg = CommandBarConfig::default();
    assert_eq!(cfg.dialog_config.search_position, SearchPosition::Bottom);
}

#[test]
fn batch22_command_bar_ai_top_search() {
    let cfg = CommandBarConfig::ai_style();
    assert_eq!(cfg.dialog_config.search_position, SearchPosition::Top);
    assert!(cfg.dialog_config.show_icons);
    assert!(cfg.dialog_config.show_footer);
}

#[test]
fn batch22_command_bar_no_search_hidden() {
    let cfg = CommandBarConfig::no_search();
    assert_eq!(cfg.dialog_config.search_position, SearchPosition::Hidden);
}

#[test]
fn batch22_command_bar_notes_separators() {
    let cfg = CommandBarConfig::notes_style();
    assert_eq!(cfg.dialog_config.section_style, SectionStyle::Separators);
    assert!(cfg.dialog_config.show_icons);
}

#[test]
fn batch22_command_bar_main_menu_no_icons() {
    let cfg = CommandBarConfig::main_menu_style();
    assert!(!cfg.dialog_config.show_icons);
    assert!(!cfg.dialog_config.show_footer);
}

// ============================================================
// 24. Action builder chaining preserves all fields
// ============================================================

#[test]
fn batch22_action_chain_shortcut_icon_section() {
    let action = Action::new(
        "id",
        "Title",
        Some("Desc".into()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘T")
    .with_icon(IconName::Star)
    .with_section("Sec");
    assert_eq!(action.shortcut.as_deref(), Some("⌘T"));
    assert_eq!(action.icon, Some(IconName::Star));
    assert_eq!(action.section.as_deref(), Some("Sec"));
    assert_eq!(action.title, "Title");
    assert_eq!(action.description.as_deref(), Some("Desc"));
}

#[test]
fn batch22_action_with_shortcut_opt_none_preserves() {
    let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
        .with_shortcut("⌘A")
        .with_shortcut_opt(None);
    // with_shortcut_opt(None) should NOT clear existing shortcut
    assert_eq!(action.shortcut.as_deref(), Some("⌘A"));
}

#[test]
fn batch22_action_with_shortcut_opt_some_sets() {
    let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
        .with_shortcut_opt(Some("⌘B".into()));
    assert_eq!(action.shortcut.as_deref(), Some("⌘B"));
}

#[test]
fn batch22_action_defaults_no_icon_no_section() {
    let action = Action::new("id", "T", None, ActionCategory::ScriptContext);
    assert!(action.icon.is_none());
    assert!(action.section.is_none());
    assert!(action.shortcut.is_none());
    assert!(!action.has_action);
    assert!(action.value.is_none());
}

// ============================================================
// 25. Action lowercase caching correctness
// ============================================================

#[test]
fn batch22_action_title_lower_precomputed() {
    let action = Action::new("id", "Hello World", None, ActionCategory::ScriptContext);
    assert_eq!(action.title_lower, "hello world");
}

#[test]
fn batch22_action_description_lower_precomputed() {
    let action = Action::new(
        "id",
        "T",
        Some("Open In Editor".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(action.description_lower.as_deref(), Some("open in editor"));
}

#[test]
fn batch22_action_shortcut_lower_after_with_shortcut() {
    let action = Action::new("id", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
    assert_eq!(action.shortcut_lower.as_deref(), Some("⌘e"));
}

#[test]
fn batch22_action_no_shortcut_lower_is_none() {
    let action = Action::new("id", "T", None, ActionCategory::ScriptContext);
    assert!(action.shortcut_lower.is_none());
}

// ============================================================
// 26. Scriptlet context with custom actions via get_scriptlet_context_actions_with_custom
// ============================================================

#[test]
fn batch22_scriptlet_custom_run_is_first() {
    let script = ScriptInfo::scriptlet("My Script", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn batch22_scriptlet_custom_has_edit_scriptlet() {
    let script = ScriptInfo::scriptlet("My Script", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "edit_scriptlet"));
}

#[test]
fn batch22_scriptlet_custom_has_copy_content() {
    let script = ScriptInfo::scriptlet("My Script", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "copy_content"));
}

#[test]
fn batch22_scriptlet_custom_frecency_adds_reset() {
    let script = ScriptInfo::scriptlet("My Script", "/p.md", None, None)
        .with_frecency(true, Some("/frec".into()));
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    // Reset ranking should be last
    assert_eq!(actions.last().unwrap().id, "reset_ranking");
}

// ============================================================
// 27. Cross-context: all actions have non-empty ID and title
// ============================================================

#[test]
fn batch22_cross_script_non_empty_ids_titles() {
    let s = ScriptInfo::new("s", "/p");
    for a in get_script_context_actions(&s) {
        assert!(!a.id.is_empty(), "Action ID should not be empty");
        assert!(!a.title.is_empty(), "Action title should not be empty");
    }
}
