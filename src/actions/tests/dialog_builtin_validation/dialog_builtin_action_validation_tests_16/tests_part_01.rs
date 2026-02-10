    use super::super::builders::*;
    use super::super::command_bar::CommandBarConfig;
    use super::super::dialog::{
        build_grouped_items_static, coerce_action_selection, ActionsDialog, GroupedActionItem,
    };
    use super::super::types::*;
    use crate::clipboard_history::ContentType;
    use crate::designs::icon_variations::IconName;
    use crate::file_search::FileInfo;
    use crate::prompts::PathInfo;
    use crate::scriptlets::{Scriptlet, ScriptletAction};
    use std::collections::HashSet;

    fn action_ids(actions: &[Action]) -> Vec<String> {
        actions.iter().map(|a| a.id.clone()).collect()
    }

    // =========================================================================
    // cat01: Scriptlet context custom actions ordering relative to built-ins
    // =========================================================================

    #[test]
    fn cat01_scriptlet_custom_after_run_before_edit() {
        let script = ScriptInfo::scriptlet("Test", "/p/test.md", None, None);
        let mut scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo hi".to_string(),
        );
        scriptlet.actions = vec![ScriptletAction {
            name: "Custom One".to_string(),
            command: "custom-one".to_string(),
            tool: "bash".to_string(),
            code: "echo 1".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let run_idx = actions.iter().position(|a| a.id == "run_script").unwrap();
        let custom_idx = actions
            .iter()
            .position(|a| a.id == "scriptlet_action:custom-one")
            .unwrap();
        let edit_idx = actions
            .iter()
            .position(|a| a.id == "edit_scriptlet")
            .unwrap();
        assert!(run_idx < custom_idx, "custom after run");
        assert!(custom_idx < edit_idx, "custom before edit_scriptlet");
    }

    #[test]
    fn cat01_scriptlet_multiple_customs_preserve_order() {
        let script = ScriptInfo::scriptlet("T", "/p/t.md", None, None);
        let mut scriptlet = Scriptlet::new("T".to_string(), "bash".to_string(), "echo".to_string());
        scriptlet.actions = vec![
            ScriptletAction {
                name: "Alpha".to_string(),
                command: "alpha".to_string(),
                tool: "bash".to_string(),
                code: "echo a".to_string(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
            ScriptletAction {
                name: "Beta".to_string(),
                command: "beta".to_string(),
                tool: "bash".to_string(),
                code: "echo b".to_string(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
        ];
        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let a_idx = actions
            .iter()
            .position(|a| a.id == "scriptlet_action:alpha")
            .unwrap();
        let b_idx = actions
            .iter()
            .position(|a| a.id == "scriptlet_action:beta")
            .unwrap();
        assert!(a_idx < b_idx, "customs preserve source order");
    }

    #[test]
    fn cat01_scriptlet_custom_has_action_true() {
        let script = ScriptInfo::scriptlet("T", "/p/t.md", None, None);
        let mut scriptlet = Scriptlet::new("T".to_string(), "bash".to_string(), "echo".to_string());
        scriptlet.actions = vec![ScriptletAction {
            name: "Act".to_string(),
            command: "act-cmd".to_string(),
            tool: "bash".to_string(),
            code: "echo".to_string(),
            inputs: vec![],
            shortcut: Some("cmd+1".to_string()),
            description: Some("Do a thing".to_string()),
        }];
        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let custom = actions
            .iter()
            .find(|a| a.id == "scriptlet_action:act-cmd")
            .unwrap();
        assert!(
            custom.has_action,
            "scriptlet custom actions have has_action=true"
        );
        assert_eq!(custom.value, Some("act-cmd".to_string()));
        assert_eq!(custom.shortcut, Some("⌘1".to_string()));
    }

    #[test]
    fn cat01_scriptlet_no_custom_still_has_builtins() {
        let script = ScriptInfo::scriptlet("T", "/p/t.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"run_script".to_string()));
        assert!(ids.contains(&"edit_scriptlet".to_string()));
        assert!(ids.contains(&"copy_deeplink".to_string()));
        assert!(!ids.iter().any(|id| id.starts_with("scriptlet_action:")));
    }

    #[test]
    fn cat01_scriptlet_custom_description_propagated() {
        let script = ScriptInfo::scriptlet("T", "/p/t.md", None, None);
        let mut scriptlet = Scriptlet::new("T".to_string(), "bash".to_string(), "echo".to_string());
        scriptlet.actions = vec![ScriptletAction {
            name: "Described".to_string(),
            command: "desc-cmd".to_string(),
            tool: "bash".to_string(),
            code: "echo".to_string(),
            inputs: vec![],
            shortcut: None,
            description: Some("My description".to_string()),
        }];
        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let custom = actions
            .iter()
            .find(|a| a.id == "scriptlet_action:desc-cmd")
            .unwrap();
        assert_eq!(custom.description, Some("My description".to_string()));
    }

    // =========================================================================
    // cat02: Clipboard share and attach_to_ai universality
    // =========================================================================

    #[test]
    fn cat02_clipboard_text_has_share() {
        let entry = ClipboardEntryInfo {
            id: "t1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hi".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let ids = action_ids(&get_clipboard_history_context_actions(&entry));
        assert!(ids.contains(&"clipboard_share".to_string()));
    }

    #[test]
    fn cat02_clipboard_image_has_share() {
        let entry = ClipboardEntryInfo {
            id: "i1".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: Some((10, 10)),
            frontmost_app_name: None,
        };
        let ids = action_ids(&get_clipboard_history_context_actions(&entry));
        assert!(ids.contains(&"clipboard_share".to_string()));
    }

    #[test]
    fn cat02_clipboard_text_has_attach_to_ai() {
        let entry = ClipboardEntryInfo {
            id: "t2".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "data".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let ids = action_ids(&get_clipboard_history_context_actions(&entry));
        assert!(ids.contains(&"clipboard_attach_to_ai".to_string()));
    }

    #[test]
    fn cat02_clipboard_image_has_attach_to_ai() {
        let entry = ClipboardEntryInfo {
            id: "i2".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let ids = action_ids(&get_clipboard_history_context_actions(&entry));
        assert!(ids.contains(&"clipboard_attach_to_ai".to_string()));
    }

    #[test]
    fn cat02_clipboard_share_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "t3".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let share = actions.iter().find(|a| a.id == "clipboard_share").unwrap();
        assert_eq!(share.shortcut.as_deref(), Some("⇧⌘E"));
    }

    #[test]
    fn cat02_clipboard_attach_to_ai_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "t4".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let attach = actions
            .iter()
            .find(|a| a.id == "clipboard_attach_to_ai")
            .unwrap();
        assert_eq!(attach.shortcut.as_deref(), Some("⌃⌘A"));
    }

    // =========================================================================
    // cat03: Agent context copy_content description mentions "agent"
    // =========================================================================

    #[test]
    fn cat03_agent_edit_title_is_edit_agent() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    #[test]
    fn cat03_agent_copy_content_desc_mentions_agent() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let copy = actions.iter().find(|a| a.id == "copy_content").unwrap();
        assert!(copy
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("file"));
    }

    #[test]
    fn cat03_agent_has_reveal_and_copy_path() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"reveal_in_finder".to_string()));
        assert!(ids.contains(&"copy_path".to_string()));
    }

    #[test]
    fn cat03_agent_no_view_logs() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let ids = action_ids(&actions);
        assert!(!ids.contains(&"view_logs".to_string()));
    }

    #[test]
    fn cat03_agent_copy_path_shortcut() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
        assert_eq!(cp.shortcut.as_deref(), Some("⌘⇧C"));
    }

    // =========================================================================
    // cat04: Path context shortcut values for open_in_terminal and open_in_editor
    // =========================================================================

    #[test]
    fn cat04_path_open_in_terminal_shortcut() {
        let info = PathInfo {
            name: "project".into(),
            path: "/home/project".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let terminal = actions.iter().find(|a| a.id == "open_in_terminal").unwrap();
        assert_eq!(terminal.shortcut.as_deref(), Some("⌘T"));
    }

    #[test]
    fn cat04_path_open_in_editor_shortcut() {
        let info = PathInfo {
            name: "file.rs".into(),
            path: "/home/file.rs".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let editor = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
        assert_eq!(editor.shortcut.as_deref(), Some("⌘E"));
    }

    #[test]
    fn cat04_path_open_in_finder_shortcut() {
        let info = PathInfo {
            name: "docs".into(),
            path: "/home/docs".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let finder = actions.iter().find(|a| a.id == "open_in_finder").unwrap();
        assert_eq!(finder.shortcut.as_deref(), Some("⌘⇧F"));
    }

    #[test]
    fn cat04_path_copy_path_shortcut() {
        let info = PathInfo {
            name: "test.txt".into(),
            path: "/test.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
        assert_eq!(cp.shortcut.as_deref(), Some("⌘⇧C"));
    }

    #[test]
    fn cat04_path_move_to_trash_shortcut() {
        let info = PathInfo {
            name: "old".into(),
            path: "/old".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
        assert_eq!(trash.shortcut.as_deref(), Some("⌘⌫"));
    }

    // =========================================================================
    // cat05: File context shortcut consistency between file and directory
    // =========================================================================

    #[test]
    fn cat05_file_and_dir_both_have_reveal_shortcut() {
        let file = FileInfo {
            path: "/a/b.txt".into(),
            name: "b.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let dir = FileInfo {
            path: "/a/c".into(),
            name: "c".into(),
            file_type: crate::file_search::FileType::Directory,
            is_dir: true,
        };
        let f_reveal = get_file_context_actions(&file)
            .into_iter()
            .find(|a| a.id == "reveal_in_finder")
            .unwrap();
        let d_reveal = get_file_context_actions(&dir)
            .into_iter()
            .find(|a| a.id == "reveal_in_finder")
            .unwrap();
        assert_eq!(f_reveal.shortcut, d_reveal.shortcut, "same shortcut");
    }

    #[test]
    fn cat05_file_and_dir_copy_path_same_shortcut() {
        let file = FileInfo {
            path: "/a.txt".into(),
            name: "a.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let dir = FileInfo {
            path: "/b".into(),
            name: "b".into(),
            file_type: crate::file_search::FileType::Directory,
            is_dir: true,
        };
        let f_cp = get_file_context_actions(&file)
            .into_iter()
            .find(|a| a.id == "copy_path")
            .unwrap();
        let d_cp = get_file_context_actions(&dir)
            .into_iter()
            .find(|a| a.id == "copy_path")
            .unwrap();
        assert_eq!(f_cp.shortcut, d_cp.shortcut);
    }

    #[test]
    fn cat05_file_primary_has_enter_shortcut() {
        let file = FileInfo {
            path: "/x.rs".into(),
            name: "x.rs".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let primary = &actions[0];
        assert_eq!(primary.shortcut.as_deref(), Some("↵"));
    }

    #[test]
    fn cat05_dir_primary_has_enter_shortcut() {
        let dir = FileInfo {
            path: "/mydir".into(),
            name: "mydir".into(),
            file_type: crate::file_search::FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&dir);
        let primary = &actions[0];
        assert_eq!(primary.shortcut.as_deref(), Some("↵"));
    }

