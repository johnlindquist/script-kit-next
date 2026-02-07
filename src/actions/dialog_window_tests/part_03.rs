
#[test]
fn all_file_action_ids_are_snake_case() {
    let file = FileInfo {
        name: "test.txt".to_string(),
        path: "/test.txt".to_string(),
        is_dir: false,
        file_type: crate::file_search::FileType::File,
    };
    let actions = get_file_context_actions(&file);

    for action in &actions {
        assert!(
            !action.id.contains(' '),
            "File action ID '{}' should not contain spaces",
            action.id
        );
        assert_eq!(
            action.id,
            action.id.to_lowercase(),
            "File action ID '{}' should be lowercase",
            action.id
        );
    }
}

// =========================================================================
// Path context: action count and common actions
// =========================================================================

#[test]
fn path_context_directory_has_all_expected_actions() {
    let path = PathInfo {
        name: "Downloads".to_string(),
        path: "/Users/test/Downloads".to_string(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    // Directory should have: open_directory + 6 common
    assert!(ids.contains(&"open_directory"));
    assert!(ids.contains(&"copy_path"));
    assert!(ids.contains(&"open_in_finder"));
    assert!(ids.contains(&"open_in_editor"));
    assert!(ids.contains(&"open_in_terminal"));
    assert!(ids.contains(&"copy_filename"));
    assert!(ids.contains(&"move_to_trash"));
    assert_eq!(actions.len(), 7);
}

#[test]
fn path_context_file_has_all_expected_actions() {
    let path = PathInfo {
        name: "doc.txt".to_string(),
        path: "/Users/test/doc.txt".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    // File should have: select_file + 6 common
    assert!(ids.contains(&"select_file"));
    assert!(ids.contains(&"copy_path"));
    assert!(ids.contains(&"open_in_finder"));
    assert!(ids.contains(&"open_in_editor"));
    assert!(ids.contains(&"open_in_terminal"));
    assert!(ids.contains(&"copy_filename"));
    assert!(ids.contains(&"move_to_trash"));
    assert_eq!(actions.len(), 7);
}

#[test]
fn path_trash_description_folder_vs_file() {
    let dir_path = PathInfo {
        name: "Docs".to_string(),
        path: "/Docs".to_string(),
        is_dir: true,
    };
    let dir_actions = get_path_context_actions(&dir_path);
    let dir_trash = dir_actions
        .iter()
        .find(|a| a.id == "move_to_trash")
        .unwrap();
    assert_eq!(dir_trash.description, Some("Delete folder".to_string()));

    let file_path = PathInfo {
        name: "test.txt".to_string(),
        path: "/test.txt".to_string(),
        is_dir: false,
    };
    let file_actions = get_path_context_actions(&file_path);
    let file_trash = file_actions
        .iter()
        .find(|a| a.id == "move_to_trash")
        .unwrap();
    assert_eq!(file_trash.description, Some("Delete file".to_string()));
}

// =========================================================================
// File context: directory Quick Look exclusion
// =========================================================================

#[cfg(target_os = "macos")]
#[test]
fn file_directory_excludes_quick_look_includes_open_with() {
    let dir = FileInfo {
        name: "Folder".to_string(),
        path: "/Folder".to_string(),
        is_dir: true,
        file_type: crate::file_search::FileType::Directory,
    };
    let actions = get_file_context_actions(&dir);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    assert!(
        !ids.contains(&"quick_look"),
        "Dir should NOT have Quick Look"
    );
    assert!(ids.contains(&"open_with"), "Dir should have Open With");
    assert!(ids.contains(&"show_info"), "Dir should have Show Info");
}

// =========================================================================
// Scriptlet with custom actions: ordering guarantee
// =========================================================================

#[test]
fn scriptlet_custom_actions_appear_between_run_and_builtins() {
    use crate::scriptlets::{Scriptlet, ScriptletAction};

    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut scriptlet = Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![
        ScriptletAction {
            name: "Custom A".to_string(),
            command: "cmd-a".to_string(),
            tool: "bash".to_string(),
            code: "echo a".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
        ScriptletAction {
            name: "Custom B".to_string(),
            command: "cmd-b".to_string(),
            tool: "bash".to_string(),
            code: "echo b".to_string(),
            inputs: vec![],
            shortcut: Some("cmd+b".to_string()),
            description: None,
        },
    ];

    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));

    // Find positions
    let run_pos = actions.iter().position(|a| a.id == "run_script").unwrap();
    let custom_a_pos = actions
        .iter()
        .position(|a| a.id == "scriptlet_action:cmd-a")
        .unwrap();
    let custom_b_pos = actions
        .iter()
        .position(|a| a.id == "scriptlet_action:cmd-b")
        .unwrap();
    let edit_pos = actions
        .iter()
        .position(|a| a.id == "edit_scriptlet")
        .unwrap();

    assert_eq!(run_pos, 0, "run_script must be first");
    assert!(custom_a_pos > run_pos, "Custom A after run");
    assert!(custom_b_pos > custom_a_pos, "Custom B after Custom A");
    assert!(
        edit_pos > custom_b_pos,
        "Built-in edit after custom actions"
    );

    // Custom actions should have has_action=true
    let ca = actions
        .iter()
        .find(|a| a.id == "scriptlet_action:cmd-a")
        .unwrap();
    assert!(ca.has_action, "Custom actions should have has_action=true");

    // Custom B should have shortcut formatted
    let cb = actions
        .iter()
        .find(|a| a.id == "scriptlet_action:cmd-b")
        .unwrap();
    assert_eq!(cb.shortcut, Some("⌘B".to_string()));
}

// =========================================================================
// ProtocolAction: edge case combinations
// =========================================================================

#[test]
fn protocol_action_with_all_fields() {
    let pa = ProtocolAction {
        name: "Full Action".to_string(),
        description: Some("A complete action".to_string()),
        shortcut: Some("cmd+shift+f".to_string()),
        value: Some("full-value".to_string()),
        has_action: true,
        visible: Some(true),
        close: Some(false),
    };

    assert_eq!(pa.name, "Full Action");
    assert_eq!(pa.description, Some("A complete action".to_string()));
    assert_eq!(pa.shortcut, Some("cmd+shift+f".to_string()));
    assert_eq!(pa.value, Some("full-value".to_string()));
    assert!(pa.has_action);
    assert!(pa.is_visible());
    assert!(!pa.should_close());
}

#[test]
fn protocol_action_hidden_but_closes() {
    let pa = ProtocolAction {
        name: "Hidden Closer".to_string(),
        description: None,
        shortcut: None,
        value: None,
        has_action: false,
        visible: Some(false),
        close: Some(true),
    };
    assert!(!pa.is_visible());
    assert!(pa.should_close());
}

// =========================================================================
// Score action: combined scoring
// =========================================================================

#[test]
fn score_action_title_prefix_plus_description_match() {
    let action = Action::new(
        "copy_path",
        "Copy Path",
        Some("Copy the full path to clipboard".to_string()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "copy");
    // Prefix match (100) + description contains "copy" (15) = 115
    assert_eq!(score, 115, "Prefix + description match should score 115");
}

#[test]
fn score_action_title_contains_plus_shortcut_match() {
    let action = Action::new(
        "reveal",
        "Reveal in Finder",
        None,
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘⇧F");
    let score = ActionsDialog::score_action(&action, "f");
    // Contains "f" in "reveal in finder" (50) + shortcut contains "f" in "⌘⇧f" (10) = 60
    assert_eq!(
        score, 60,
        "Contains + shortcut match should score 60, got {}",
        score
    );
}

#[test]
fn score_action_no_match_returns_zero() {
    let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "xyz");
    assert_eq!(score, 0);
}

// =========================================================================
// ScriptInfo constructor invariants
// =========================================================================

#[test]
fn script_info_new_always_is_script_true() {
    let s = ScriptInfo::new("a", "/a");
    assert!(s.is_script);
    assert!(!s.is_scriptlet);
    assert!(!s.is_agent);
}

#[test]
fn script_info_builtin_never_is_script() {
    let s = ScriptInfo::builtin("X");
    assert!(!s.is_script);
    assert!(!s.is_scriptlet);
    assert!(!s.is_agent);
}

#[test]
fn script_info_scriptlet_never_is_script() {
    let s = ScriptInfo::scriptlet("X", "/x.md", None, None);
    assert!(!s.is_script);
    assert!(s.is_scriptlet);
    assert!(!s.is_agent);
}

#[test]
fn script_info_default_action_verb_is_run() {
    assert_eq!(ScriptInfo::new("a", "/a").action_verb, "Run");
    assert_eq!(ScriptInfo::builtin("X").action_verb, "Run");
    assert_eq!(
        ScriptInfo::scriptlet("X", "/x.md", None, None).action_verb,
        "Run"
    );
    assert_eq!(
        ScriptInfo::with_shortcut("X", "/x", None).action_verb,
        "Run"
    );
}

// =========================================================================
// No duplicate IDs: scriptlet context, path context, file context
// =========================================================================

#[test]
fn no_duplicate_ids_in_scriptlet_context() {
    let script = ScriptInfo::scriptlet(
        "Test",
        "/test.md",
        Some("cmd+t".to_string()),
        Some("ts".to_string()),
    )
    .with_frecency(true, Some("s:Test".to_string()));

    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let mut ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let total = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(total, ids.len(), "Scriptlet context should have no dups");
}

#[test]
fn no_duplicate_ids_in_path_context() {
    let path = PathInfo {
        name: "test".to_string(),
        path: "/test".to_string(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path);
    let mut ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let total = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(total, ids.len(), "Path context should have no dups");
}

#[test]
fn no_duplicate_ids_in_file_context() {
    let file = FileInfo {
        name: "f.txt".to_string(),
        path: "/f.txt".to_string(),
        is_dir: false,
        file_type: crate::file_search::FileType::File,
    };
    let actions = get_file_context_actions(&file);
    let mut ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let total = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(total, ids.len(), "File context should have no dups");
}

// =========================================================================
// All actions have categories
// =========================================================================

#[test]
fn all_script_actions_use_script_context_category() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    for action in &actions {
        assert_eq!(
            action.category,
            ActionCategory::ScriptContext,
            "Action '{}' should be ScriptContext",
            action.id
        );
    }
}

#[test]
fn all_clipboard_actions_use_script_context_category() {
    let entry = ClipboardEntryInfo {
        id: "t".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    for action in &actions {
        assert_eq!(
            action.category,
            ActionCategory::ScriptContext,
            "Clipboard action '{}' should be ScriptContext",
            action.id
        );
    }
}

// =========================================================================
// Clipboard: text vs image action count difference
// =========================================================================

#[test]
fn clipboard_image_has_more_actions_than_text() {
    let text = ClipboardEntryInfo {
        id: "t".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "txt".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let img = ClipboardEntryInfo {
        id: "i".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".to_string(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };

    let text_actions = get_clipboard_history_context_actions(&text);
    let img_actions = get_clipboard_history_context_actions(&img);

    assert!(
        img_actions.len() > text_actions.len(),
        "Image should have more actions ({}) than text ({})",
        img_actions.len(),
        text_actions.len()
    );
}
