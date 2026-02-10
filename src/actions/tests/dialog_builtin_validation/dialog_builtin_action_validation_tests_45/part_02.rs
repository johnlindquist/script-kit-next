
// =========== 10. File context: all ScriptContext category ===========

#[test]
fn file_context_file_all_script_context() {
    let file_info = FileInfo {
        name: "test.txt".into(),
        path: "/tmp/test.txt".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&file_info);
    assert!(actions
        .iter()
        .all(|a| a.category == ActionCategory::ScriptContext));
}

#[test]
fn file_context_dir_all_script_context() {
    let file_info = FileInfo {
        name: "mydir".into(),
        path: "/tmp/mydir".into(),
        is_dir: true,
        file_type: FileType::Directory,
    };
    let actions = get_file_context_actions(&file_info);
    assert!(actions
        .iter()
        .all(|a| a.category == ActionCategory::ScriptContext));
}

#[test]
fn file_context_no_script_ops() {
    let file_info = FileInfo {
        name: "test.txt".into(),
        path: "/tmp/test.txt".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&file_info);
    assert!(!actions
        .iter()
        .any(|a| a.category == ActionCategory::ScriptOps));
}

#[test]
fn file_context_no_global_ops() {
    let file_info = FileInfo {
        name: "test.txt".into(),
        path: "/tmp/test.txt".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&file_info);
    assert!(!actions
        .iter()
        .any(|a| a.category == ActionCategory::GlobalOps));
}

// =========== 11. Path context: primary at index 0 ===========

#[test]
fn path_file_primary_at_index_0() {
    let path_info = PathInfo {
        name: "file.txt".into(),
        path: "/tmp/file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    assert_eq!(actions[0].id, "select_file");
}

#[test]
fn path_dir_primary_at_index_0() {
    let path_info = PathInfo {
        name: "mydir".into(),
        path: "/tmp/mydir".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    assert_eq!(actions[0].id, "open_directory");
}

#[test]
fn path_file_copy_path_at_index_1() {
    let path_info = PathInfo {
        name: "file.txt".into(),
        path: "/tmp/file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    assert_eq!(actions[1].id, "copy_path");
}

#[test]
fn path_dir_copy_path_at_index_1() {
    let path_info = PathInfo {
        name: "mydir".into(),
        path: "/tmp/mydir".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    assert_eq!(actions[1].id, "copy_path");
}

// =========== 12. Path context: dir has all 7 IDs ===========

#[test]
fn path_dir_has_open_directory() {
    let path_info = PathInfo {
        name: "d".into(),
        path: "/d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    assert!(actions.iter().any(|a| a.id == "open_directory"));
}

#[test]
fn path_dir_has_copy_path_and_copy_filename() {
    let path_info = PathInfo {
        name: "d".into(),
        path: "/d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    assert!(actions.iter().any(|a| a.id == "copy_path"));
    assert!(actions.iter().any(|a| a.id == "copy_filename"));
}

#[test]
fn path_dir_has_open_in_finder_and_editor() {
    let path_info = PathInfo {
        name: "d".into(),
        path: "/d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    assert!(actions.iter().any(|a| a.id == "open_in_finder"));
    assert!(actions.iter().any(|a| a.id == "open_in_editor"));
}

#[test]
fn path_dir_has_terminal_and_trash() {
    let path_info = PathInfo {
        name: "d".into(),
        path: "/d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    assert!(actions.iter().any(|a| a.id == "open_in_terminal"));
    assert!(actions.iter().any(|a| a.id == "move_to_trash"));
}

// =========== 13. Script: shortcut+alias yields update+remove for both ===========

#[test]
fn script_shortcut_alias_has_update_shortcut() {
    let s =
        ScriptInfo::with_shortcut_and_alias("test", "/p", Some("cmd+t".into()), Some("ts".into()));
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "update_shortcut"));
}

#[test]
fn script_shortcut_alias_has_update_alias() {
    let s =
        ScriptInfo::with_shortcut_and_alias("test", "/p", Some("cmd+t".into()), Some("ts".into()));
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "update_alias"));
}

#[test]
fn script_shortcut_alias_has_remove_shortcut() {
    let s =
        ScriptInfo::with_shortcut_and_alias("test", "/p", Some("cmd+t".into()), Some("ts".into()));
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
}

#[test]
fn script_shortcut_alias_has_remove_alias() {
    let s =
        ScriptInfo::with_shortcut_and_alias("test", "/p", Some("cmd+t".into()), Some("ts".into()));
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "remove_alias"));
}

// =========== 14. Script: agent exactly 8 actions ===========

#[test]
fn agent_has_8_actions() {
    let mut s = ScriptInfo::new("my-agent", "/agents/my-agent.md");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    assert_eq!(actions.len(), 8);
}

#[test]
fn agent_has_edit_script() {
    let mut s = ScriptInfo::new("my-agent", "/agents/my-agent.md");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "edit_script"));
}

#[test]
fn agent_has_copy_content() {
    let mut s = ScriptInfo::new("my-agent", "/agents/my-agent.md");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "copy_content"));
}

#[test]
fn agent_has_copy_deeplink() {
    let mut s = ScriptInfo::new("my-agent", "/agents/my-agent.md");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "copy_deeplink"));
}

// =========== 15. Script: get_global_actions empty ===========

#[test]
fn global_actions_returns_empty() {
    let actions = get_global_actions();
    assert!(actions.is_empty());
}

#[test]
fn global_actions_len_zero() {
    let actions = get_global_actions();
    assert_eq!(actions.len(), 0);
}

#[test]
fn global_actions_no_script_context() {
    let actions = get_global_actions();
    assert!(!actions
        .iter()
        .any(|a| a.category == ActionCategory::ScriptContext));
}

#[test]
fn global_actions_no_global_ops() {
    let actions = get_global_actions();
    assert!(!actions
        .iter()
        .any(|a| a.category == ActionCategory::GlobalOps));
}

// =========== 16. Scriptlet with_custom: None scriptlet → no has_action=true ===========

#[test]
fn scriptlet_with_custom_none_first_is_run() {
    let s = ScriptInfo::scriptlet("My Scriptlet", "/s.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn scriptlet_with_custom_none_all_has_action_false() {
    let s = ScriptInfo::scriptlet("My Scriptlet", "/s.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    assert!(actions.iter().all(|a| !a.has_action));
}

#[test]
fn scriptlet_with_custom_none_no_scriptlet_action_ids() {
    let s = ScriptInfo::scriptlet("My Scriptlet", "/s.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    assert!(!actions
        .iter()
        .any(|a| a.id.starts_with("scriptlet_action:")));
}

#[test]
fn scriptlet_with_custom_none_has_edit_scriptlet() {
    let s = ScriptInfo::scriptlet("My Scriptlet", "/s.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    assert!(actions.iter().any(|a| a.id == "edit_scriptlet"));
}

// =========== 17. Scriptlet with_custom: copy_content desc ===========

#[test]
fn scriptlet_with_custom_copy_content_desc() {
    let s = ScriptInfo::scriptlet("My Scriptlet", "/s.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    let action = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert!(action
        .description
        .as_ref()
        .unwrap()
        .contains("entire file content"));
}

#[test]
fn scriptlet_with_custom_copy_content_shortcut() {
    let s = ScriptInfo::scriptlet("My Scriptlet", "/s.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    let action = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(action.shortcut.as_deref(), Some("⌘⌥C"));
}

#[test]
fn scriptlet_with_custom_copy_content_title() {
    let s = ScriptInfo::scriptlet("My Scriptlet", "/s.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    let action = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(action.title, "Copy Content");
}

#[test]
fn scriptlet_with_custom_copy_content_present() {
    let s = ScriptInfo::scriptlet("My Scriptlet", "/s.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    assert!(actions.iter().any(|a| a.id == "copy_content"));
}

// =========== 18. Scriptlet defined: empty actions → empty result ===========

#[test]
fn scriptlet_defined_empty_returns_empty() {
    let scriptlet = Scriptlet::new(
        "test".to_string(),
        "bash".to_string(),
        "echo hi".to_string(),
    );
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert!(actions.is_empty());
}

#[test]
fn scriptlet_defined_empty_len_zero() {
    let scriptlet = Scriptlet::new(
        "test".to_string(),
        "bash".to_string(),
        "echo hi".to_string(),
    );
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert_eq!(actions.len(), 0);
}

#[test]
fn scriptlet_defined_empty_no_has_action() {
    let scriptlet = Scriptlet::new(
        "test".to_string(),
        "bash".to_string(),
        "echo hi".to_string(),
    );
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert!(!actions.iter().any(|a| a.has_action));
}

#[test]
fn scriptlet_defined_empty_no_ids() {
    let scriptlet = Scriptlet::new(
        "test".to_string(),
        "bash".to_string(),
        "echo hi".to_string(),
    );
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert!(!actions
        .iter()
        .any(|a| a.id.starts_with("scriptlet_action:")));
}

// =========== 19. Scriptlet defined: action with description preserved ===========

#[test]
fn scriptlet_defined_preserves_description() {
    let mut scriptlet = Scriptlet::new(
        "test".to_string(),
        "bash".to_string(),
        "echo hi".to_string(),
    );
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".to_string(),
        command: "pbcopy".to_string(),
        tool: "bash".to_string(),
        code: "echo hi | pbcopy".to_string(),
        description: Some("Copy to clipboard".to_string()),
        shortcut: None,
        inputs: vec![],
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert_eq!(
        actions[0].description,
        Some("Copy to clipboard".to_string())
    );
}

#[test]
fn scriptlet_defined_has_action_true() {
    let mut scriptlet = Scriptlet::new(
        "test".to_string(),
        "bash".to_string(),
        "echo hi".to_string(),
    );
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".to_string(),
        command: "pbcopy".to_string(),
        tool: "bash".to_string(),
        code: "echo hi | pbcopy".to_string(),
        description: None,
        shortcut: None,
        inputs: vec![],
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert!(actions[0].has_action);
}

#[test]
fn scriptlet_defined_value_is_command() {
    let mut scriptlet = Scriptlet::new(
        "test".to_string(),
        "bash".to_string(),
        "echo hi".to_string(),
    );
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".to_string(),
        command: "pbcopy".to_string(),
        tool: "bash".to_string(),
        code: "echo hi | pbcopy".to_string(),
        description: None,
        shortcut: None,
        inputs: vec![],
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert_eq!(actions[0].value, Some("pbcopy".to_string()));
}
