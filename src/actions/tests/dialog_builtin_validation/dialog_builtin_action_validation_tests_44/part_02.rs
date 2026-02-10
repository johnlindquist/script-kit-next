
// =========== 9. Path context: all actions have ScriptContext category ===========

#[test]
fn path_file_all_script_context() {
    let p = PathInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&p);
    assert!(actions
        .iter()
        .all(|a| a.category == ActionCategory::ScriptContext));
}

#[test]
fn path_dir_all_script_context() {
    let p = PathInfo {
        path: "/test/dir".into(),
        name: "dir".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&p);
    assert!(actions
        .iter()
        .all(|a| a.category == ActionCategory::ScriptContext));
}

#[test]
fn path_file_primary_is_first() {
    let p = PathInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&p);
    assert_eq!(actions[0].id, "select_file");
}

#[test]
fn path_dir_primary_is_first() {
    let p = PathInfo {
        path: "/test/dir".into(),
        name: "dir".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&p);
    assert_eq!(actions[0].id, "open_directory");
}

// =========== 10. Script context: run_script title includes verb and quoted name ===========

#[test]
fn script_run_title_includes_verb() {
    let s = ScriptInfo::with_action_verb("Test", "/p", true, "Launch");
    let actions = get_script_context_actions(&s);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert!(run.title.starts_with("Launch"));
}

#[test]
fn script_run_title_includes_quoted_name() {
    let s = ScriptInfo::new("My Script", "/p");
    let actions = get_script_context_actions(&s);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert!(run.title.contains("\"My Script\""));
}

#[test]
fn script_run_desc_includes_verb() {
    let s = ScriptInfo::with_action_verb("X", "/p", true, "Execute");
    let actions = get_script_context_actions(&s);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert!(run.description.as_ref().unwrap().contains("Execute"));
}

#[test]
fn script_run_shortcut_enter() {
    let s = ScriptInfo::new("X", "/p");
    let actions = get_script_context_actions(&s);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert_eq!(run.shortcut, Some("↵".to_string()));
}

// =========== 11. Script context: copy_deeplink desc uses to_deeplink_name ===========

#[test]
fn script_deeplink_desc_has_correct_url() {
    let s = ScriptInfo::new("My Cool Script", "/p");
    let actions = get_script_context_actions(&s);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert!(dl
        .description
        .as_ref()
        .unwrap()
        .contains("scriptkit://run/my-cool-script"));
}

#[test]
fn script_deeplink_shortcut() {
    let s = ScriptInfo::new("X", "/p");
    let actions = get_script_context_actions(&s);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert_eq!(dl.shortcut, Some("⌘⇧D".to_string()));
}

#[test]
fn script_deeplink_title() {
    let s = ScriptInfo::new("X", "/p");
    let actions = get_script_context_actions(&s);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert_eq!(dl.title, "Copy Deeplink");
}

#[test]
fn scriptlet_deeplink_desc_has_slugified_name() {
    let s = ScriptInfo::scriptlet("Open GitHub PR", "/path.md", None, None);
    let actions = get_script_context_actions(&s);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert!(dl.description.as_ref().unwrap().contains("open-github-pr"));
}

// =========== 12. Script context: agent actions have agent-specific descriptions ===========

#[test]
fn agent_edit_desc_mentions_agent() {
    let mut s = ScriptInfo::new("My Agent", "/p");
    s.is_script = false;
    s.is_agent = true;
    let actions = get_script_context_actions(&s);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert!(edit.description.as_ref().unwrap().contains("agent"));
}

#[test]
fn agent_reveal_desc_mentions_agent() {
    let mut s = ScriptInfo::new("My Agent", "/p");
    s.is_script = false;
    s.is_agent = true;
    let actions = get_script_context_actions(&s);
    let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
    assert!(reveal.description.as_ref().unwrap().contains("agent"));
}

#[test]
fn agent_copy_path_desc_mentions_agent() {
    let mut s = ScriptInfo::new("My Agent", "/p");
    s.is_script = false;
    s.is_agent = true;
    let actions = get_script_context_actions(&s);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert!(cp.description.as_ref().unwrap().contains("agent"));
}

#[test]
fn agent_no_view_logs() {
    let mut s = ScriptInfo::new("My Agent", "/p");
    s.is_script = false;
    s.is_agent = true;
    let actions = get_script_context_actions(&s);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

// =========== 13. Scriptlet with_custom: run_script title format ===========

#[test]
fn scriptlet_with_custom_run_title_includes_name() {
    let s = ScriptInfo::scriptlet("My Snippet", "/path.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert!(run.title.contains("\"My Snippet\""));
}

#[test]
fn scriptlet_with_custom_run_title_starts_with_verb() {
    let s = ScriptInfo::scriptlet("X", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert!(run.title.starts_with("Run"));
}

#[test]
fn scriptlet_with_custom_edit_desc_mentions_editor() {
    let s = ScriptInfo::scriptlet("X", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    let edit = actions.iter().find(|a| a.id == "edit_scriptlet").unwrap();
    assert!(edit.description.as_ref().unwrap().contains("$EDITOR"));
}

#[test]
fn scriptlet_with_custom_reveal_desc_mentions_finder() {
    let s = ScriptInfo::scriptlet("X", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    let reveal = actions
        .iter()
        .find(|a| a.id == "reveal_scriptlet_in_finder")
        .unwrap();
    assert!(reveal.description.as_ref().unwrap().contains("Finder"));
}

// =========== 14. Scriptlet defined actions: has_action and value set ===========

#[test]
fn scriptlet_defined_action_has_action_true() {
    let mut scriptlet = Scriptlet::new("T".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".to_string(),
        command: "copy".to_string(),
        tool: "bash".to_string(),
        code: "pbcopy".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert!(actions[0].has_action);
}

#[test]
fn scriptlet_defined_action_value_is_command() {
    let mut scriptlet = Scriptlet::new("T".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".to_string(),
        command: "copy-text".to_string(),
        tool: "bash".to_string(),
        code: "pbcopy".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert_eq!(actions[0].value, Some("copy-text".to_string()));
}

#[test]
fn scriptlet_defined_action_id_uses_prefix() {
    let mut scriptlet = Scriptlet::new("T".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Open".to_string(),
        command: "open-link".to_string(),
        tool: "open".to_string(),
        code: "https://example.com".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert_eq!(actions[0].id, "scriptlet_action:open-link");
}

#[test]
fn scriptlet_defined_action_shortcut_formatted() {
    let mut scriptlet = Scriptlet::new("T".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".to_string(),
        command: "copy".to_string(),
        tool: "bash".to_string(),
        code: "pbcopy".to_string(),
        inputs: vec![],
        shortcut: Some("cmd+c".to_string()),
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert_eq!(actions[0].shortcut, Some("⌘C".to_string()));
}

// =========== 15. AI bar: copy_last_code details ===========

#[test]
fn ai_bar_copy_last_code_shortcut() {
    let actions = get_ai_command_bar_actions();
    let clc = actions.iter().find(|a| a.id == "copy_last_code").unwrap();
    assert_eq!(clc.shortcut, Some("⌥⌘C".to_string()));
}

#[test]
fn ai_bar_copy_last_code_icon() {
    let actions = get_ai_command_bar_actions();
    let clc = actions.iter().find(|a| a.id == "copy_last_code").unwrap();
    assert_eq!(clc.icon, Some(IconName::Code));
}

#[test]
fn ai_bar_copy_last_code_section() {
    let actions = get_ai_command_bar_actions();
    let clc = actions.iter().find(|a| a.id == "copy_last_code").unwrap();
    assert_eq!(clc.section, Some("Response".to_string()));
}

#[test]
fn ai_bar_copy_last_code_desc_mentions_code() {
    let actions = get_ai_command_bar_actions();
    let clc = actions.iter().find(|a| a.id == "copy_last_code").unwrap();
    assert!(clc
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("code"));
}

// =========== 16. AI bar: submit action details ===========

#[test]
fn ai_bar_submit_shortcut() {
    let actions = get_ai_command_bar_actions();
    let sub = actions.iter().find(|a| a.id == "submit").unwrap();
    assert_eq!(sub.shortcut, Some("↵".to_string()));
}

#[test]
fn ai_bar_submit_icon() {
    let actions = get_ai_command_bar_actions();
    let sub = actions.iter().find(|a| a.id == "submit").unwrap();
    assert_eq!(sub.icon, Some(IconName::ArrowUp));
}

#[test]
fn ai_bar_submit_section_actions() {
    let actions = get_ai_command_bar_actions();
    let sub = actions.iter().find(|a| a.id == "submit").unwrap();
    assert_eq!(sub.section, Some("Actions".to_string()));
}

#[test]
fn ai_bar_submit_desc_mentions_send() {
    let actions = get_ai_command_bar_actions();
    let sub = actions.iter().find(|a| a.id == "submit").unwrap();
    assert!(sub
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("send"));
}

// =========== 17. AI bar: export_markdown details ===========

#[test]
fn ai_bar_export_markdown_shortcut() {
    let actions = get_ai_command_bar_actions();
    let em = actions.iter().find(|a| a.id == "export_markdown").unwrap();
    assert_eq!(em.shortcut, Some("⇧⌘E".to_string()));
}

#[test]
fn ai_bar_export_markdown_icon() {
    let actions = get_ai_command_bar_actions();
    let em = actions.iter().find(|a| a.id == "export_markdown").unwrap();
    assert_eq!(em.icon, Some(IconName::FileCode));
}

#[test]
fn ai_bar_export_markdown_section() {
    let actions = get_ai_command_bar_actions();
    let em = actions.iter().find(|a| a.id == "export_markdown").unwrap();
    assert_eq!(em.section, Some("Export".to_string()));
}

#[test]
fn ai_bar_export_markdown_title() {
    let actions = get_ai_command_bar_actions();
    let em = actions.iter().find(|a| a.id == "export_markdown").unwrap();
    assert_eq!(em.title, "Export as Markdown");
}

// =========== 18. Notes: find_in_note details ===========

#[test]
fn notes_find_in_note_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let fin = actions.iter().find(|a| a.id == "find_in_note").unwrap();
    assert_eq!(fin.shortcut, Some("⌘F".to_string()));
}

#[test]
fn notes_find_in_note_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let fin = actions.iter().find(|a| a.id == "find_in_note").unwrap();
    assert_eq!(fin.icon, Some(IconName::MagnifyingGlass));
}

#[test]
fn notes_find_in_note_section() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let fin = actions.iter().find(|a| a.id == "find_in_note").unwrap();
    assert_eq!(fin.section, Some("Edit".to_string()));
}

#[test]
fn notes_find_in_note_absent_without_selection() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "find_in_note"));
}

// =========== 19. Notes: duplicate_note details ===========

#[test]
fn notes_duplicate_note_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let dup = actions.iter().find(|a| a.id == "duplicate_note").unwrap();
    assert_eq!(dup.shortcut, Some("⌘D".to_string()));
}

#[test]
fn notes_duplicate_note_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let dup = actions.iter().find(|a| a.id == "duplicate_note").unwrap();
    assert_eq!(dup.icon, Some(IconName::Copy));
}
