
#[test]
fn file_context_file_quick_look_desc() {
    let fi = FileInfo {
        name: "demo.txt".into(),
        path: "/path/demo.txt".into(),
        is_dir: false,
        file_type: FileType::Document,
    };
    let actions = get_file_context_actions(&fi);
    if let Some(ql) = actions.iter().find(|a| a.id == "quick_look") {
        assert!(ql.description.as_deref().unwrap().contains("Quick Look"));
    }
}

// =====================================================================
// 9. File context: copy_path shortcut ⌘⇧C
// =====================================================================

#[test]
fn file_context_copy_path_shortcut() {
    let fi = FileInfo {
        name: "file.rs".into(),
        path: "/path/file.rs".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&fi);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert_eq!(cp.shortcut.as_deref(), Some("⌘⇧C"));
}

#[test]
fn file_context_copy_path_desc_mentions_full_path() {
    let fi = FileInfo {
        name: "file.rs".into(),
        path: "/path/file.rs".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&fi);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert!(cp.description.as_deref().unwrap().contains("full path"));
}

#[test]
fn file_context_copy_filename_shortcut_cmd_c() {
    let fi = FileInfo {
        name: "main.rs".into(),
        path: "/path/main.rs".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&fi);
    let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert_eq!(cf.shortcut.as_deref(), Some("⌘C"));
}

// =====================================================================
// 10. Path context: open_in_terminal shortcut ⌘T
// =====================================================================

#[test]
fn path_context_open_in_terminal_shortcut() {
    let pi = PathInfo {
        name: "project".into(),
        path: "/path/project".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    let ot = actions.iter().find(|a| a.id == "open_in_terminal").unwrap();
    assert_eq!(ot.shortcut.as_deref(), Some("⌘T"));
}

#[test]
fn path_context_open_in_terminal_desc() {
    let pi = PathInfo {
        name: "project".into(),
        path: "/path/project".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    let ot = actions.iter().find(|a| a.id == "open_in_terminal").unwrap();
    assert!(ot.description.as_deref().unwrap().contains("terminal"));
}

#[test]
fn path_context_open_in_terminal_title() {
    let pi = PathInfo {
        name: "src".into(),
        path: "/path/src".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    let ot = actions.iter().find(|a| a.id == "open_in_terminal").unwrap();
    assert_eq!(ot.title, "Open in Terminal");
}

#[test]
fn path_context_open_in_terminal_present_for_file() {
    let pi = PathInfo {
        name: "main.rs".into(),
        path: "/path/main.rs".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    assert!(actions.iter().any(|a| a.id == "open_in_terminal"));
}

// =====================================================================
// 11. Script context: view_logs shortcut ⌘L
// =====================================================================

#[test]
fn script_context_view_logs_shortcut() {
    let script = ScriptInfo::new("my-script", "/path/my-script.ts");
    let actions = get_script_context_actions(&script);
    let vl = actions.iter().find(|a| a.id == "view_logs").unwrap();
    assert_eq!(vl.shortcut.as_deref(), Some("⌘L"));
}

#[test]
fn script_context_view_logs_title() {
    let script = ScriptInfo::new("my-script", "/path/my-script.ts");
    let actions = get_script_context_actions(&script);
    let vl = actions.iter().find(|a| a.id == "view_logs").unwrap();
    assert_eq!(vl.title, "View Logs");
}

#[test]
fn script_context_view_logs_desc_mentions_logs() {
    let script = ScriptInfo::new("my-script", "/path/my-script.ts");
    let actions = get_script_context_actions(&script);
    let vl = actions.iter().find(|a| a.id == "view_logs").unwrap();
    assert!(vl.description.as_deref().unwrap().contains("logs"));
}

#[test]
fn script_context_view_logs_absent_for_builtin() {
    let script = ScriptInfo::builtin("Clipboard History");
    let actions = get_script_context_actions(&script);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

// =====================================================================
// 12. Script context: all IDs unique within context
// =====================================================================

#[test]
fn script_context_ids_unique_basic() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len(), "Duplicate IDs found: {:?}", ids);
}

#[test]
fn script_context_ids_unique_with_shortcut_and_alias() {
    let script = ScriptInfo::with_shortcut_and_alias(
        "test",
        "/path/test.ts",
        Some("cmd+t".into()),
        Some("ts".into()),
    );
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len());
}

#[test]
fn scriptlet_context_ids_unique() {
    let script = ScriptInfo::scriptlet("Open URL", "/path/url.md", None, None);
    let scriptlet = Scriptlet::new("Open URL".into(), "bash".into(), "echo hi".into());
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len());
}

#[test]
fn ai_command_bar_ids_unique() {
    let actions = get_ai_command_bar_actions();
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len());
}

// =====================================================================
// 13. Script context: action count increases with shortcut+alias+suggestion
// =====================================================================

#[test]
fn script_context_base_count_no_extras() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    // run + add_shortcut + add_alias + edit + view_logs + reveal + copy_path + copy_content + copy_deeplink = 9
    assert_eq!(actions.len(), 9);
}

#[test]
fn script_context_with_shortcut_adds_one() {
    let script = ScriptInfo::with_shortcut("test", "/path/test.ts", Some("cmd+t".into()));
    let actions = get_script_context_actions(&script);
    // run + update_shortcut + remove_shortcut + add_alias + edit + view_logs + reveal + copy_path + copy_content + copy_deeplink = 10
    assert_eq!(actions.len(), 10);
}

#[test]
fn script_context_with_both_adds_two() {
    let script = ScriptInfo::with_shortcut_and_alias(
        "test",
        "/path/test.ts",
        Some("cmd+t".into()),
        Some("ts".into()),
    );
    let actions = get_script_context_actions(&script);
    // run + update_shortcut + remove_shortcut + update_alias + remove_alias + edit + view_logs + reveal + copy_path + copy_content + copy_deeplink = 11
    assert_eq!(actions.len(), 11);
}

#[test]
fn script_context_with_suggestion_adds_reset_ranking() {
    let script =
        ScriptInfo::new("test", "/path/test.ts").with_frecency(true, Some("/path/test.ts".into()));
    let actions = get_script_context_actions(&script);
    // 9 + reset_ranking = 10
    assert_eq!(actions.len(), 10);
    assert!(actions.iter().any(|a| a.id == "reset_ranking"));
}

// =====================================================================
// 14. Scriptlet context: identical shortcut/alias dynamic behavior
// =====================================================================

#[test]
fn scriptlet_no_shortcut_has_add_shortcut() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "add_shortcut"));
}

#[test]
fn scriptlet_with_shortcut_has_update_and_remove() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", Some("cmd+t".into()), None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "update_shortcut"));
    assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
    assert!(!actions.iter().any(|a| a.id == "add_shortcut"));
}

#[test]
fn scriptlet_no_alias_has_add_alias() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "add_alias"));
}

#[test]
fn scriptlet_with_alias_has_update_and_remove() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, Some("ts".into()));
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "update_alias"));
    assert!(actions.iter().any(|a| a.id == "remove_alias"));
    assert!(!actions.iter().any(|a| a.id == "add_alias"));
}

// =====================================================================
// 15. Agent context: no view_logs but has edit/reveal/copy_path/copy_content
// =====================================================================

#[test]
fn agent_has_edit_with_agent_title() {
    let mut script = ScriptInfo::new("my-agent", "/path/agent.md");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.title, "Edit Agent");
}

#[test]
fn agent_has_reveal_in_finder() {
    let mut script = ScriptInfo::new("my-agent", "/path/agent.md");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
}

#[test]
fn agent_has_copy_path_and_copy_content() {
    let mut script = ScriptInfo::new("my-agent", "/path/agent.md");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    assert!(actions.iter().any(|a| a.id == "copy_path"));
    assert!(actions.iter().any(|a| a.id == "copy_content"));
}

#[test]
fn agent_no_view_logs() {
    let mut script = ScriptInfo::new("my-agent", "/path/agent.md");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

// =====================================================================
// 16. AI command bar: branch_from_last has no shortcut
// =====================================================================

#[test]
fn ai_bar_branch_from_last_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let bfl = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
    assert!(bfl.shortcut.is_none());
}

#[test]
fn ai_bar_branch_from_last_section_actions() {
    let actions = get_ai_command_bar_actions();
    let bfl = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
    assert_eq!(bfl.section.as_deref(), Some("Actions"));
}

#[test]
fn ai_bar_branch_from_last_icon_arrowright() {
    let actions = get_ai_command_bar_actions();
    let bfl = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
    assert_eq!(bfl.icon, Some(IconName::ArrowRight));
}

#[test]
fn ai_bar_branch_from_last_desc_mentions_branch() {
    let actions = get_ai_command_bar_actions();
    let bfl = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
    assert!(bfl.description.as_deref().unwrap().contains("branch"));
}

// =====================================================================
// 17. AI command bar: section ordering
// =====================================================================

#[test]
fn ai_bar_first_section_is_response() {
    let actions = get_ai_command_bar_actions();
    assert_eq!(actions[0].section.as_deref(), Some("Response"));
}

#[test]
fn ai_bar_response_section_has_3_actions() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .count();
    assert_eq!(count, 3);
}

#[test]
fn ai_bar_actions_section_has_4_actions() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Actions"))
        .count();
    assert_eq!(count, 4);
}

#[test]
fn ai_bar_total_is_12() {
    let actions = get_ai_command_bar_actions();
    assert_eq!(actions.len(), 12);
}

// =====================================================================
// 18. Notes: auto_sizing_enabled=true hides enable_auto_sizing
// =====================================================================

#[test]
fn notes_auto_sizing_enabled_hides_action() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "enable_auto_sizing"));
}

#[test]
fn notes_auto_sizing_disabled_shows_action() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "enable_auto_sizing"));
}

#[test]
fn notes_auto_sizing_action_shortcut_cmd_a() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let action = actions
        .iter()
        .find(|a| a.id == "enable_auto_sizing")
        .unwrap();
    assert_eq!(action.shortcut.as_deref(), Some("⌘A"));
}

#[test]
fn notes_auto_sizing_action_section_settings() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let action = actions
        .iter()
        .find(|a| a.id == "enable_auto_sizing")
        .unwrap();
    assert_eq!(action.section.as_deref(), Some("Settings"));
}

// =====================================================================
// 19. Notes: full selection action set
// =====================================================================

#[test]
fn notes_full_selection_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note + duplicate + browse_notes + find_in_note + format + copy_note_as + copy_deeplink + create_quicklink + export + enable_auto_sizing = 10
    assert_eq!(actions.len(), 10);
}
