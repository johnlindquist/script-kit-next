    use super::super::builders::*;
    use super::super::command_bar::CommandBarConfig;
    use super::super::dialog::{
        build_grouped_items_static, coerce_action_selection, ActionsDialog, GroupedActionItem,
    };
    use super::super::types::*;
    use crate::clipboard_history::ContentType;
    use crate::designs::icon_variations::IconName;
    use crate::file_search::{FileInfo, FileType};
    use crate::prompts::PathInfo;
    use crate::scriptlets::{Scriptlet, ScriptletAction};
    use std::collections::HashSet;

    fn action_ids(actions: &[Action]) -> Vec<String> {
        actions.iter().map(|a| a.id.clone()).collect()
    }

    // =========================================================================
    // Category 01: Script context agent vs script edit action description diff
    // Verifies that the edit action description differs between agent and
    // script contexts—agent says "agent file", script says "$EDITOR".
    // =========================================================================

    #[test]
    fn cat01_script_edit_desc_mentions_editor() {
        let script = ScriptInfo::new("Test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert!(
            edit.description.as_ref().unwrap().contains("$EDITOR"),
            "Script edit should mention $EDITOR"
        );
    }

    #[test]
    fn cat01_agent_edit_desc_mentions_agent() {
        let mut script = ScriptInfo::new("Agent", "/path/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert!(
            edit.description.as_ref().unwrap().contains("agent"),
            "Agent edit should mention 'agent'"
        );
    }

    #[test]
    fn cat01_agent_edit_title_says_edit_agent() {
        let mut script = ScriptInfo::new("Agent", "/path/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    #[test]
    fn cat01_script_edit_title_says_edit_script() {
        let script = ScriptInfo::new("Test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Script");
    }

    // =========================================================================
    // Category 02: Scriptlet context copy_content always present
    // Verifies copy_content exists in scriptlet context (via both
    // get_script_context_actions and get_scriptlet_context_actions_with_custom).
    // =========================================================================

    #[test]
    fn cat02_scriptlet_context_has_copy_content() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "copy_content"));
    }

    #[test]
    fn cat02_scriptlet_with_custom_has_copy_content() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(actions.iter().any(|a| a.id == "copy_content"));
    }

    #[test]
    fn cat02_scriptlet_copy_content_shortcut() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
        assert_eq!(cc.shortcut.as_ref().unwrap(), "⌘⌥C");
    }

    #[test]
    fn cat02_scriptlet_copy_content_desc_mentions_file() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
        assert!(cc
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("file"));
    }

    // =========================================================================
    // Category 03: Path context open title includes quoted name
    // Verifies the primary action title includes the file/directory name in quotes.
    // =========================================================================

    #[test]
    fn cat03_path_dir_title_includes_name() {
        let path_info = PathInfo {
            path: "/Users/test/Documents".to_string(),
            name: "Documents".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path_info);
        let primary = actions.first().unwrap();
        assert!(primary.title.contains("Documents"));
        assert!(primary.title.contains('"'));
    }

    #[test]
    fn cat03_path_file_title_includes_name() {
        let path_info = PathInfo {
            path: "/Users/test/readme.txt".to_string(),
            name: "readme.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        let primary = actions.first().unwrap();
        assert!(primary.title.contains("readme.txt"));
    }

    #[test]
    fn cat03_path_dir_primary_is_open_directory() {
        let path_info = PathInfo {
            path: "/Users/test/src".to_string(),
            name: "src".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path_info);
        assert_eq!(actions[0].id, "open_directory");
    }

    #[test]
    fn cat03_path_file_primary_is_select_file() {
        let path_info = PathInfo {
            path: "/Users/test/file.rs".to_string(),
            name: "file.rs".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path_info);
        assert_eq!(actions[0].id, "select_file");
    }

    #[test]
    fn cat03_file_context_title_includes_name() {
        let file_info = FileInfo {
            path: "/test/photo.jpg".to_string(),
            name: "photo.jpg".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        let primary = actions.first().unwrap();
        assert!(primary.title.contains("photo.jpg"));
    }

    // =========================================================================
    // Category 04: Clipboard delete_all description mentions pinned
    // Verifies the delete_all action description mentions "pinned" items
    // are excluded from the clear operation.
    // =========================================================================

    #[test]
    fn cat04_delete_all_desc_mentions_pinned() {
        let entry = ClipboardEntryInfo {
            id: "t".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let da = actions
            .iter()
            .find(|a| a.id == "clipboard_delete_all")
            .unwrap();
        assert!(da
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("pinned"));
    }

    #[test]
    fn cat04_delete_entry_desc_mentions_remove() {
        let entry = ClipboardEntryInfo {
            id: "t".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let d = actions.iter().find(|a| a.id == "clipboard_delete").unwrap();
        assert!(d
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("remove"));
    }

    #[test]
    fn cat04_delete_multiple_desc_mentions_filter() {
        let entry = ClipboardEntryInfo {
            id: "t".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let dm = actions
            .iter()
            .find(|a| a.id == "clipboard_delete_multiple")
            .unwrap();
        assert!(
            dm.description
                .as_ref()
                .unwrap()
                .to_lowercase()
                .contains("filter")
                || dm
                    .description
                    .as_ref()
                    .unwrap()
                    .to_lowercase()
                    .contains("entries")
        );
    }

    #[test]
    fn cat04_destructive_actions_order() {
        let entry = ClipboardEntryInfo {
            id: "t".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        let del_idx = ids.iter().position(|id| id == "clipboard_delete").unwrap();
        let del_multi = ids
            .iter()
            .position(|id| id == "clipboard_delete_multiple")
            .unwrap();
        let del_all = ids
            .iter()
            .position(|id| id == "clipboard_delete_all")
            .unwrap();
        // Destructive actions in order: delete < delete_multiple < delete_all
        assert!(del_idx < del_multi);
        assert!(del_multi < del_all);
    }

    // =========================================================================
    // Category 05: AI command bar branch_from_last has no shortcut
    // Verifies that branch_from_last and change_model lack shortcuts.
    // =========================================================================

    #[test]
    fn cat05_branch_from_last_no_shortcut() {
        let actions = get_ai_command_bar_actions();
        let bfl = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
        assert!(bfl.shortcut.is_none());
    }

    #[test]
    fn cat05_change_model_no_shortcut() {
        let actions = get_ai_command_bar_actions();
        let cm = actions.iter().find(|a| a.id == "change_model").unwrap();
        assert!(cm.shortcut.is_none());
    }

    #[test]
    fn cat05_submit_has_shortcut_enter() {
        let actions = get_ai_command_bar_actions();
        let s = actions.iter().find(|a| a.id == "submit").unwrap();
        assert_eq!(s.shortcut.as_ref().unwrap(), "↵");
    }

    #[test]
    fn cat05_new_chat_shortcut_cmd_n() {
        let actions = get_ai_command_bar_actions();
        let nc = actions.iter().find(|a| a.id == "new_chat").unwrap();
        assert_eq!(nc.shortcut.as_ref().unwrap(), "⌘N");
    }

    #[test]
    fn cat05_delete_chat_shortcut_cmd_delete() {
        let actions = get_ai_command_bar_actions();
        let dc = actions.iter().find(|a| a.id == "delete_chat").unwrap();
        assert_eq!(dc.shortcut.as_ref().unwrap(), "⌘⌫");
    }

    // =========================================================================
    // Category 06: Notes command bar new_note always present
    // Verifies new_note and browse_notes appear regardless of flag combos.
    // =========================================================================

    #[test]
    fn cat06_new_note_always_present_full() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "new_note"));
    }

    #[test]
    fn cat06_new_note_always_present_trash() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "new_note"));
    }

    #[test]
    fn cat06_browse_notes_always_present() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "browse_notes"));
    }

    #[test]
    fn cat06_new_note_shortcut_cmd_n() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let nn = actions.iter().find(|a| a.id == "new_note").unwrap();
        assert_eq!(nn.shortcut.as_ref().unwrap(), "⌘N");
    }

    #[test]
    fn cat06_browse_notes_icon_folder() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let bn = actions.iter().find(|a| a.id == "browse_notes").unwrap();
        assert_eq!(bn.icon, Some(IconName::FolderOpen));
    }

    // =========================================================================
    // Category 07: Chat context model action ordering matches input
    // Verifies model selection actions appear in the same order as input.
    // =========================================================================

    #[test]
    fn cat07_model_ordering_preserved() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![
                ChatModelInfo {
                    id: "gpt-4".into(),
                    display_name: "GPT-4".into(),
                    provider: "OpenAI".into(),
                },
                ChatModelInfo {
                    id: "claude".into(),
                    display_name: "Claude".into(),
                    provider: "Anthropic".into(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions[0].id, "select_model_gpt-4");
        assert_eq!(actions[1].id, "select_model_claude");
    }

    #[test]
    fn cat07_continue_after_models() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "m1".into(),
                display_name: "M1".into(),
                provider: "P".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_idx = actions
            .iter()
            .position(|a| a.id == "select_model_m1")
            .unwrap();
        let cont_idx = actions
            .iter()
            .position(|a| a.id == "continue_in_chat")
            .unwrap();
        assert!(cont_idx > model_idx);
    }

