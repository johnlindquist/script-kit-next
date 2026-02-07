
// ============================================================================
// 26. coerce_action_selection
// ============================================================================

#[test]
fn cat26_empty_returns_none() {
    assert!(coerce_action_selection(&[], 0).is_none());
}

#[test]
fn cat26_on_item_returns_same() {
    let rows = vec![GroupedActionItem::Item(0)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
}

#[test]
fn cat26_header_searches_down() {
    let rows = vec![
        GroupedActionItem::SectionHeader("S".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn cat26_trailing_header_searches_up() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("S".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn cat26_all_headers_returns_none() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".into()),
        GroupedActionItem::SectionHeader("B".into()),
    ];
    assert!(coerce_action_selection(&rows, 0).is_none());
}

#[test]
fn cat26_out_of_bounds_clamped() {
    let rows = vec![GroupedActionItem::Item(0)];
    assert_eq!(coerce_action_selection(&rows, 100), Some(0));
}

// ============================================================================
// 27. Cross-context ID namespace collision avoidance
// ============================================================================

#[test]
fn cat27_script_and_clipboard_no_id_overlap() {
    let script_actions = get_script_context_actions(&ScriptInfo::new("test", "/test.ts"));
    let clip_actions = get_clipboard_history_context_actions(&text_entry());
    let script_ids: HashSet<String> = action_ids(&script_actions).into_iter().collect();
    let clip_ids: HashSet<String> = action_ids(&clip_actions).into_iter().collect();
    let overlap: Vec<&String> = script_ids.intersection(&clip_ids).collect();
    assert!(
        overlap.is_empty(),
        "Script/Clipboard ID overlap: {:?}",
        overlap
    );
}

#[test]
fn cat27_ai_and_notes_no_id_overlap() {
    let ai_actions = get_ai_command_bar_actions();
    let notes_actions = get_notes_command_bar_actions(&NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    });
    let ai_ids: HashSet<String> = action_ids(&ai_actions).into_iter().collect();
    let notes_ids: HashSet<String> = action_ids(&notes_actions).into_iter().collect();
    // copy_deeplink exists in both contexts by design - that's OK since they
    // are in different command bars and never shown together. But check for
    // unexpected collisions.
    let overlap: Vec<&String> = ai_ids.intersection(&notes_ids).collect();
    // Allow known shared IDs
    let unexpected: Vec<&&String> = overlap
        .iter()
        .filter(|id| !["copy_deeplink", "new_chat"].contains(&id.as_str()))
        .collect();
    assert!(
        unexpected.is_empty(),
        "Unexpected AI/Notes ID overlap: {:?}",
        unexpected
    );
}

#[test]
fn cat27_path_and_file_some_shared_ids() {
    // Path and file contexts are related — they share some IDs by design
    let path_actions = get_path_context_actions(&path_dir());
    let file_actions = get_file_context_actions(&file_info_dir());
    let path_ids: HashSet<String> = action_ids(&path_actions).into_iter().collect();
    let file_ids: HashSet<String> = action_ids(&file_actions).into_iter().collect();
    let shared: Vec<&String> = path_ids.intersection(&file_ids).collect();
    // copy_path, copy_filename, open_directory should be shared
    assert!(
        shared.len() >= 2,
        "Path/File should share some IDs: {:?}",
        shared
    );
}

// ============================================================================
// 28. All actions have non-empty id and title
// ============================================================================

#[test]
fn cat28_script_actions_nonempty_id_title() {
    for action in &get_script_context_actions(&ScriptInfo::new("t", "/t.ts")) {
        assert!(!action.id.is_empty(), "Empty ID");
        assert!(!action.title.is_empty(), "Empty title for {}", action.id);
    }
}

#[test]
fn cat28_clipboard_actions_nonempty_id_title() {
    for action in &get_clipboard_history_context_actions(&text_entry()) {
        assert!(!action.id.is_empty(), "Empty ID");
        assert!(!action.title.is_empty(), "Empty title for {}", action.id);
    }
}

#[test]
fn cat28_ai_actions_nonempty_id_title() {
    for action in &get_ai_command_bar_actions() {
        assert!(!action.id.is_empty(), "Empty ID");
        assert!(!action.title.is_empty(), "Empty title for {}", action.id);
    }
}

#[test]
fn cat28_notes_actions_nonempty_id_title() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for action in &get_notes_command_bar_actions(&info) {
        assert!(!action.id.is_empty(), "Empty ID");
        assert!(!action.title.is_empty(), "Empty title for {}", action.id);
    }
}

#[test]
fn cat28_path_actions_nonempty_id_title() {
    for action in &get_path_context_actions(&path_dir()) {
        assert!(!action.id.is_empty(), "Empty ID");
        assert!(!action.title.is_empty(), "Empty title for {}", action.id);
    }
}

#[test]
fn cat28_file_actions_nonempty_id_title() {
    for action in &get_file_context_actions(&file_info_file()) {
        assert!(!action.id.is_empty(), "Empty ID");
        assert!(!action.title.is_empty(), "Empty title for {}", action.id);
    }
}

// ============================================================================
// 29. has_action = false for all built-in actions
// ============================================================================

#[test]
fn cat29_script_actions_has_action_false() {
    for action in &get_script_context_actions(&ScriptInfo::new("t", "/t.ts")) {
        assert!(
            !action.has_action,
            "{} should have has_action=false",
            action.id
        );
    }
}

#[test]
fn cat29_clipboard_actions_has_action_false() {
    for action in &get_clipboard_history_context_actions(&text_entry()) {
        assert!(
            !action.has_action,
            "{} should have has_action=false",
            action.id
        );
    }
}

#[test]
fn cat29_ai_actions_has_action_false() {
    for action in &get_ai_command_bar_actions() {
        assert!(
            !action.has_action,
            "{} should have has_action=false",
            action.id
        );
    }
}

#[test]
fn cat29_path_actions_has_action_false() {
    for action in &get_path_context_actions(&path_dir()) {
        assert!(
            !action.has_action,
            "{} should have has_action=false",
            action.id
        );
    }
}

#[test]
fn cat29_file_actions_has_action_false() {
    for action in &get_file_context_actions(&file_info_file()) {
        assert!(
            !action.has_action,
            "{} should have has_action=false",
            action.id
        );
    }
}

#[test]
fn cat29_notes_actions_has_action_false() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for action in &get_notes_command_bar_actions(&info) {
        assert!(
            !action.has_action,
            "{} should have has_action=false",
            action.id
        );
    }
}

// ============================================================================
// 30. Scriptlet defined actions — has_action=true and value set
// ============================================================================

#[test]
fn cat30_scriptlet_actions_have_has_action_true() {
    let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo test".into());
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".into(),
        command: "copy-cmd".into(),
        tool: "bash".into(),
        code: "pbcopy".into(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert!(
        actions[0].has_action,
        "Scriptlet action should have has_action=true"
    );
}

#[test]
fn cat30_scriptlet_actions_have_value() {
    let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo test".into());
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".into(),
        command: "copy-cmd".into(),
        tool: "bash".into(),
        code: "pbcopy".into(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert_eq!(actions[0].value, Some("copy-cmd".to_string()));
}

#[test]
fn cat30_scriptlet_action_id_format() {
    let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo test".into());
    scriptlet.actions = vec![ScriptletAction {
        name: "Open Browser".into(),
        command: "open-browser".into(),
        tool: "bash".into(),
        code: "open".into(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert_eq!(actions[0].id, "scriptlet_action:open-browser");
}

#[test]
fn cat30_scriptlet_with_shortcut_formatted() {
    let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo test".into());
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".into(),
        command: "copy".into(),
        tool: "bash".into(),
        code: "pbcopy".into(),
        inputs: vec![],
        shortcut: Some("cmd+c".into()),
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert_eq!(actions[0].shortcut, Some("⌘C".to_string()));
}

#[test]
fn cat30_scriptlet_empty_actions_returns_empty() {
    let scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo test".into());
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert!(actions.is_empty());
}

// ============================================================================
// Bonus: Ordering determinism — repeated calls produce same result
// ============================================================================

#[test]
fn bonus_script_actions_deterministic() {
    let s = ScriptInfo::new("test", "/test.ts");
    let a1 = action_ids(&get_script_context_actions(&s));
    let a2 = action_ids(&get_script_context_actions(&s));
    assert_eq!(a1, a2);
}

#[test]
fn bonus_clipboard_actions_deterministic() {
    let a1 = action_ids(&get_clipboard_history_context_actions(&text_entry()));
    let a2 = action_ids(&get_clipboard_history_context_actions(&text_entry()));
    assert_eq!(a1, a2);
}

#[test]
fn bonus_ai_actions_deterministic() {
    let a1 = action_ids(&get_ai_command_bar_actions());
    let a2 = action_ids(&get_ai_command_bar_actions());
    assert_eq!(a1, a2);
}

#[test]
fn bonus_notes_actions_deterministic() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let a1 = action_ids(&get_notes_command_bar_actions(&info));
    let a2 = action_ids(&get_notes_command_bar_actions(&info));
    assert_eq!(a1, a2);
}

// ============================================================================
// Bonus: ActionCategory PartialEq
// ============================================================================

#[test]
fn bonus_action_category_equality() {
    assert_eq!(ActionCategory::ScriptContext, ActionCategory::ScriptContext);
    assert_ne!(ActionCategory::ScriptContext, ActionCategory::ScriptOps);
    assert_ne!(ActionCategory::GlobalOps, ActionCategory::Terminal);
}

// ============================================================================
// Bonus: title_lower invariant across contexts
// ============================================================================

#[test]
fn bonus_title_lower_matches_lowercase() {
    // Script context
    for action in &get_script_context_actions(&ScriptInfo::new("Test", "/t.ts")) {
        assert_eq!(action.title_lower, action.title.to_lowercase());
    }
    // Clipboard context
    for action in &get_clipboard_history_context_actions(&text_entry()) {
        assert_eq!(action.title_lower, action.title.to_lowercase());
    }
    // AI command bar
    for action in &get_ai_command_bar_actions() {
        assert_eq!(action.title_lower, action.title.to_lowercase());
    }
}

// ============================================================================
// Bonus: All ScriptContext category
// ============================================================================

#[test]
fn bonus_all_script_actions_are_script_context() {
    for a in &get_script_context_actions(&ScriptInfo::new("t", "/t.ts")) {
        assert_eq!(a.category, ActionCategory::ScriptContext);
    }
}

#[test]
fn bonus_all_clipboard_actions_are_script_context() {
    for a in &get_clipboard_history_context_actions(&text_entry()) {
        assert_eq!(a.category, ActionCategory::ScriptContext);
    }
}

#[test]
fn bonus_all_ai_actions_are_script_context() {
    for a in &get_ai_command_bar_actions() {
        assert_eq!(a.category, ActionCategory::ScriptContext);
    }
}

#[test]
fn bonus_all_path_actions_are_script_context() {
    for a in &get_path_context_actions(&path_dir()) {
        assert_eq!(a.category, ActionCategory::ScriptContext);
    }
}
