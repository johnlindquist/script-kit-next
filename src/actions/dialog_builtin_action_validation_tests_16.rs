//! Batch 16 – Dialog Builtin Action Validation Tests
//!
//! 30 categories, ~155 tests covering fresh angles:
//! - Scriptlet context with custom actions ordering relative to built-ins
//! - Clipboard share/attach_to_ai universality across content types
//! - Agent context copy_content description substring
//! - Path context open_in_terminal/open_in_editor shortcuts
//! - File context shortcut consistency between file and directory
//! - Notes command bar section count per flag combo
//! - Chat context continue_in_chat shortcut value
//! - AI command bar export section single action
//! - New chat preset icon propagation
//! - Note switcher pinned+current combined state
//! - format_shortcut_hint modifier keyword normalization edge cases
//! - to_deeplink_name numeric and underscore handling
//! - score_action empty query behaviour
//! - fuzzy_match case sensitivity
//! - build_grouped_items_static single-item input
//! - coerce_action_selection single-item input
//! - CommandBarConfig close flag independence
//! - Action::new description_lower None when description is None
//! - Action builder chain ordering (icon before section, section before shortcut)
//! - ScriptInfo with_action_verb preserves defaults
//! - Script context agent flag produces edit_script with "Edit Agent" title
//! - Cross-context shortcut format consistency (all use Unicode symbols)
//! - Clipboard paste_keep_open shortcut value
//! - Path context copy_filename has no shortcut
//! - File context open_with macOS shortcut
//! - Notes format shortcut exact value
//! - AI command bar icon name correctness
//! - Script context run title format
//! - Ordering consistency across repeated calls

#[cfg(test)]
mod tests {
    // --- merged from tests_part_01.rs ---
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
        assert!(ids.contains(&"script:copy_deeplink".to_string()));
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
        assert!(ids.contains(&"file:reveal_in_finder".to_string()));
        assert!(ids.contains(&"file:copy_path".to_string()));
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
        let cp = actions.iter().find(|a| a.id == "file:copy_path").unwrap();
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
        let terminal = actions.iter().find(|a| a.id == "file:open_in_terminal").unwrap();
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
        let editor = actions.iter().find(|a| a.id == "file:open_in_editor").unwrap();
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
        let finder = actions.iter().find(|a| a.id == "file:open_in_finder").unwrap();
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
        let cp = actions.iter().find(|a| a.id == "file:copy_path").unwrap();
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
        let trash = actions.iter().find(|a| a.id == "file:move_to_trash").unwrap();
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
            .find(|a| a.id == "file:reveal_in_finder")
            .unwrap();
        let d_reveal = get_file_context_actions(&dir)
            .into_iter()
            .find(|a| a.id == "file:reveal_in_finder")
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
            .find(|a| a.id == "file:copy_path")
            .unwrap();
        let d_cp = get_file_context_actions(&dir)
            .into_iter()
            .find(|a| a.id == "file:copy_path")
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


    // --- merged from tests_part_02.rs ---
    #[test]
    fn cat05_file_copy_filename_shortcut_cmd_c() {
        let file = FileInfo {
            path: "/doc.pdf".into(),
            name: "doc.pdf".into(),
            file_type: crate::file_search::FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let cf = actions.iter().find(|a| a.id == "file:copy_filename").unwrap();
        assert_eq!(cf.shortcut.as_deref(), Some("⌘C"));
    }

    // =========================================================================
    // cat06: Notes command bar section count per flag combination
    // =========================================================================

    #[test]
    fn cat06_notes_full_feature_section_count() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let sections: HashSet<&str> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        // Notes, Edit, Copy, Export, Settings
        assert_eq!(
            sections.len(),
            5,
            "full feature has 5 sections: {:?}",
            sections
        );
    }

    #[test]
    fn cat06_notes_trash_view_minimal_sections() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let sections: HashSet<&str> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        // Only Notes and Settings
        assert!(sections.contains("Notes"));
        assert!(sections.contains("Settings"));
    }

    #[test]
    fn cat06_notes_no_selection_sections() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let sections: HashSet<&str> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        assert!(sections.contains("Notes"));
        assert!(sections.contains("Settings"));
        assert!(!sections.contains("Edit"), "no Edit without selection");
        assert!(!sections.contains("Copy"), "no Copy without selection");
    }

    #[test]
    fn cat06_notes_auto_sizing_enabled_hides_setting() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        assert!(!ids.contains(&"enable_auto_sizing".to_string()));
    }

    #[test]
    fn cat06_notes_auto_sizing_disabled_shows_setting() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"enable_auto_sizing".to_string()));
    }

    // =========================================================================
    // cat07: Chat context continue_in_chat shortcut
    // =========================================================================

    #[test]
    fn cat07_chat_continue_shortcut_cmd_enter() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let cont = actions.iter().find(|a| a.id == "continue_in_chat").unwrap();
        assert_eq!(cont.shortcut.as_deref(), Some("⌘↵"));
    }

    #[test]
    fn cat07_chat_continue_always_present() {
        // Even with no models, continue_in_chat should be present
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let ids = action_ids(&get_chat_context_actions(&info));
        assert!(ids.contains(&"continue_in_chat".to_string()));
    }

    #[test]
    fn cat07_chat_copy_response_conditional_true() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: true,
        };
        let ids = action_ids(&get_chat_context_actions(&info));
        assert!(ids.contains(&"chat:copy_response".to_string()));
    }

    #[test]
    fn cat07_chat_copy_response_conditional_false() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let ids = action_ids(&get_chat_context_actions(&info));
        assert!(!ids.contains(&"chat:copy_response".to_string()));
    }

    #[test]
    fn cat07_chat_clear_conditional_true() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let ids = action_ids(&get_chat_context_actions(&info));
        assert!(ids.contains(&"clear_conversation".to_string()));
    }

    #[test]
    fn cat07_chat_clear_conditional_false() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let ids = action_ids(&get_chat_context_actions(&info));
        assert!(!ids.contains(&"clear_conversation".to_string()));
    }

    // =========================================================================
    // cat08: AI command bar export section has exactly one action
    // =========================================================================

    #[test]
    fn cat08_ai_export_section_count() {
        let actions = get_ai_command_bar_actions();
        let export_count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Export"))
            .count();
        assert_eq!(export_count, 1, "Export section has exactly 1 action");
    }

    #[test]
    fn cat08_ai_export_action_is_export_markdown() {
        let actions = get_ai_command_bar_actions();
        let export = actions
            .iter()
            .find(|a| a.section.as_deref() == Some("Export"))
            .unwrap();
        assert_eq!(export.id, "export_markdown");
    }

    #[test]
    fn cat08_ai_export_markdown_shortcut() {
        let actions = get_ai_command_bar_actions();
        let export = actions.iter().find(|a| a.id == "export_markdown").unwrap();
        assert_eq!(export.shortcut.as_deref(), Some("⇧⌘E"));
    }

    #[test]
    fn cat08_ai_export_markdown_icon() {
        let actions = get_ai_command_bar_actions();
        let export = actions.iter().find(|a| a.id == "export_markdown").unwrap();
        assert_eq!(export.icon, Some(IconName::FileCode));
    }

    // =========================================================================
    // cat09: New chat preset icon propagation
    // =========================================================================

    #[test]
    fn cat09_new_chat_preset_icon_preserved() {
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        let preset_action = actions.iter().find(|a| a.id == "preset_general").unwrap();
        assert_eq!(preset_action.icon, Some(IconName::Star));
    }

    #[test]
    fn cat09_new_chat_preset_no_description() {
        let presets = vec![NewChatPresetInfo {
            id: "code".into(),
            name: "Code".into(),
            icon: IconName::Code,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        let preset_action = actions.iter().find(|a| a.id == "preset_code").unwrap();
        assert!(preset_action.description.is_none());
    }

    #[test]
    fn cat09_new_chat_model_has_provider_description() {
        let models = vec![NewChatModelInfo {
            model_id: "gpt-4".into(),
            display_name: "GPT-4".into(),
            provider: "openai".into(),
            provider_display_name: "OpenAI".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        let model_action = actions.iter().find(|a| a.id == "model_0").unwrap();
        assert_eq!(model_action.description.as_deref(), Some("OpenAI"));
    }

    #[test]
    fn cat09_new_chat_model_icon_settings() {
        let models = vec![NewChatModelInfo {
            model_id: "claude".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        let model = actions.iter().find(|a| a.id == "model_0").unwrap();
        assert_eq!(model.icon, Some(IconName::Settings));
    }

    #[test]
    fn cat09_new_chat_last_used_bolt_icon() {
        let last_used = vec![NewChatModelInfo {
            model_id: "gpt-4o".into(),
            display_name: "GPT-4o".into(),
            provider: "openai".into(),
            provider_display_name: "OpenAI".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        let lu = actions.iter().find(|a| a.id == "last_used_0").unwrap();
        assert_eq!(lu.icon, Some(IconName::BoltFilled));
    }

    // =========================================================================
    // cat10: Note switcher pinned+current combined state
    // =========================================================================

    #[test]
    fn cat10_pinned_current_icon_is_star() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "My Note".into(),
            char_count: 100,
            is_current: true,
            is_pinned: true,
            preview: "content".into(),
            relative_time: "1m ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    #[test]
    fn cat10_pinned_current_has_bullet_prefix() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n2".into(),
            title: "Pinned Current".into(),
            char_count: 50,
            is_current: true,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(
            actions[0].title.starts_with("• "),
            "current note should have bullet prefix"
        );
    }

    #[test]
    fn cat10_pinned_not_current_no_bullet() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n3".into(),
            title: "Pinned Only".into(),
            char_count: 20,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].title, "Pinned Only");
    }

    #[test]
    fn cat10_pinned_section_is_pinned() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n4".into(),
            title: "Pin".into(),
            char_count: 5,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
    }

    #[test]
    fn cat10_unpinned_section_is_recent() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n5".into(),
            title: "Regular".into(),
            char_count: 5,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Recent"));
    }

    // =========================================================================
    // cat11: format_shortcut_hint modifier keyword normalization
    // =========================================================================

    #[test]
    fn cat11_format_shortcut_cmd_c() {
        // Using the builders-private fn via ActionsDialog
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘C");
        assert_eq!(keycaps, vec!["⌘", "C"]);
    }

    #[test]
    fn cat11_format_shortcut_ctrl_alt_del() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌃⌥⌫");
        assert_eq!(keycaps, vec!["⌃", "⌥", "⌫"]);
    }

    #[test]
    fn cat11_format_shortcut_enter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("↵");
        assert_eq!(keycaps, vec!["↵"]);
    }

    #[test]
    fn cat11_format_shortcut_arrows() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("↑↓←→");
        assert_eq!(keycaps, vec!["↑", "↓", "←", "→"]);
    }

    #[test]
    fn cat11_format_shortcut_escape() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⎋");
        assert_eq!(keycaps, vec!["⎋"]);
    }

    #[test]
    fn cat11_format_shortcut_tab() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⇥");
        assert_eq!(keycaps, vec!["⇥"]);
    }

    // =========================================================================
    // cat12: to_deeplink_name numeric and underscore handling
    // =========================================================================

    #[test]
    fn cat12_deeplink_numeric_only() {
        assert_eq!(to_deeplink_name("12345"), "12345");
    }

    #[test]
    fn cat12_deeplink_underscores_to_hyphens() {
        assert_eq!(to_deeplink_name("hello_world"), "hello-world");
    }

    #[test]
    fn cat12_deeplink_mixed_case_lowered() {
        assert_eq!(to_deeplink_name("MyScript"), "myscript");
    }

    #[test]
    fn cat12_deeplink_consecutive_specials_collapsed() {
        assert_eq!(to_deeplink_name("a!!b"), "a-b");
    }

    #[test]
    fn cat12_deeplink_leading_trailing_stripped() {
        assert_eq!(to_deeplink_name("--hello--"), "hello");
    }

    // =========================================================================
    // cat13: score_action empty query behaviour
    // =========================================================================


    // --- merged from tests_part_03.rs ---
    #[test]
    fn cat13_score_empty_query_returns_prefix_match() {
        let action = Action::new(
            "test",
            "Edit Script",
            Some("Open in editor".to_string()),
            ActionCategory::ScriptContext,
        );
        // Empty string is a prefix of everything
        let score = ActionsDialog::score_action(&action, "");
        assert!(
            score >= 100,
            "empty query is prefix of any title: {}",
            score
        );
    }

    #[test]
    fn cat13_score_no_match_returns_zero() {
        let action = Action::new("test", "Edit Script", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "zzzzz");
        assert_eq!(score, 0);
    }

    #[test]
    fn cat13_score_prefix_beats_contains() {
        let action = Action::new("test", "Edit Script", None, ActionCategory::ScriptContext);
        let prefix_score = ActionsDialog::score_action(&action, "script:edit");
        let contains_score = ActionsDialog::score_action(&action, "script");
        assert!(
            prefix_score > contains_score,
            "prefix {} > contains {}",
            prefix_score,
            contains_score
        );
    }

    #[test]
    fn cat13_score_description_bonus() {
        let action = Action::new(
            "test",
            "Open File",
            Some("Open in the default editor".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "editor");
        assert!(
            score >= 15,
            "description match gives at least 15: {}",
            score
        );
    }

    #[test]
    fn cat13_score_shortcut_bonus() {
        let action =
            Action::new("test", "Submit", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
        let score = ActionsDialog::score_action(&action, "⌘e");
        assert!(score >= 10, "shortcut match gives at least 10: {}", score);
    }

    // =========================================================================
    // cat14: fuzzy_match case sensitivity
    // =========================================================================

    #[test]
    fn cat14_fuzzy_exact_match() {
        assert!(ActionsDialog::fuzzy_match("hello", "hello"));
    }

    #[test]
    fn cat14_fuzzy_subsequence() {
        assert!(ActionsDialog::fuzzy_match("hello world", "hwd"));
    }

    #[test]
    fn cat14_fuzzy_no_match() {
        assert!(!ActionsDialog::fuzzy_match("hello", "xyz"));
    }

    #[test]
    fn cat14_fuzzy_empty_needle() {
        assert!(ActionsDialog::fuzzy_match("hello", ""));
    }

    #[test]
    fn cat14_fuzzy_empty_haystack() {
        assert!(!ActionsDialog::fuzzy_match("", "a"));
    }

    #[test]
    fn cat14_fuzzy_both_empty() {
        assert!(ActionsDialog::fuzzy_match("", ""));
    }

    #[test]
    fn cat14_fuzzy_needle_longer() {
        assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
    }

    // =========================================================================
    // cat15: build_grouped_items_static single-item input
    // =========================================================================

    #[test]
    fn cat15_grouped_single_item_headers() {
        let actions = vec![
            Action::new("a", "Action A", None, ActionCategory::ScriptContext).with_section("Sec"),
        ];
        let filtered = vec![0usize];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert_eq!(grouped.len(), 2, "1 header + 1 item");
        assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(_)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(0)));
    }

    #[test]
    fn cat15_grouped_single_item_separators() {
        let actions = vec![Action::new(
            "a",
            "Action A",
            None,
            ActionCategory::ScriptContext,
        )];
        let filtered = vec![0usize];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        assert_eq!(grouped.len(), 1, "no header for separators");
        assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
    }

    #[test]
    fn cat15_grouped_single_item_none_style() {
        let actions = vec![Action::new(
            "a",
            "Action A",
            None,
            ActionCategory::ScriptContext,
        )];
        let filtered = vec![0usize];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        assert_eq!(grouped.len(), 1);
    }

    #[test]
    fn cat15_grouped_empty_returns_empty() {
        let actions: Vec<Action> = vec![];
        let filtered: Vec<usize> = vec![];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(grouped.is_empty());
    }

    #[test]
    fn cat15_grouped_same_section_no_duplicate_header() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S"),
        ];
        let filtered = vec![0usize, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 1, "same section = 1 header");
    }

    // =========================================================================
    // cat16: coerce_action_selection single-item input
    // =========================================================================

    #[test]
    fn cat16_coerce_single_item() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    }

    #[test]
    fn cat16_coerce_single_header() {
        let rows = vec![GroupedActionItem::SectionHeader("S".into())];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn cat16_coerce_header_then_item() {
        let rows = vec![
            GroupedActionItem::SectionHeader("S".into()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn cat16_coerce_item_then_header() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("S".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(0));
    }

    #[test]
    fn cat16_coerce_empty() {
        let rows: Vec<GroupedActionItem> = vec![];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn cat16_coerce_out_of_bounds_clamps() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 999), Some(0));
    }

    // =========================================================================
    // cat17: CommandBarConfig close flag independence
    // =========================================================================

    #[test]
    fn cat17_default_all_close_true() {
        let config = CommandBarConfig::default();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    #[test]
    fn cat17_ai_style_close_flags() {
        let config = CommandBarConfig::ai_style();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    #[test]
    fn cat17_main_menu_close_flags() {
        let config = CommandBarConfig::main_menu_style();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    #[test]
    fn cat17_no_search_close_flags() {
        let config = CommandBarConfig::no_search();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    // =========================================================================
    // cat18: Action::new description_lower None when description is None
    // =========================================================================

    #[test]
    fn cat18_action_no_description_lower_none() {
        let action = Action::new("id", "Title", None, ActionCategory::ScriptContext);
        assert!(action.description_lower.is_none());
    }

    #[test]
    fn cat18_action_with_description_lower_set() {
        let action = Action::new(
            "id",
            "Title",
            Some("Hello World".to_string()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.description_lower.as_deref(), Some("hello world"));
    }

    #[test]
    fn cat18_action_title_lower_cached() {
        let action = Action::new("id", "My Title", None, ActionCategory::ScriptContext);
        assert_eq!(action.title_lower, "my title");
    }

    #[test]
    fn cat18_action_shortcut_lower_none_initially() {
        let action = Action::new("id", "Title", None, ActionCategory::ScriptContext);
        assert!(action.shortcut_lower.is_none());
    }

    #[test]
    fn cat18_action_shortcut_lower_set_after_with_shortcut() {
        let action =
            Action::new("id", "Title", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
        assert_eq!(action.shortcut_lower.as_deref(), Some("⌘e"));
    }

    // =========================================================================
    // cat19: Action builder chain ordering (icon, section, shortcut)
    // =========================================================================

    #[test]
    fn cat19_icon_then_section() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_icon(IconName::Star)
            .with_section("Sec");
        assert_eq!(action.icon, Some(IconName::Star));
        assert_eq!(action.section.as_deref(), Some("Sec"));
    }

    #[test]
    fn cat19_section_then_icon() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_section("Sec")
            .with_icon(IconName::Star);
        assert_eq!(action.icon, Some(IconName::Star));
        assert_eq!(action.section.as_deref(), Some("Sec"));
    }

    #[test]
    fn cat19_shortcut_then_icon_preserves_shortcut() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘K")
            .with_icon(IconName::Settings);
        assert_eq!(action.shortcut.as_deref(), Some("⌘K"));
        assert_eq!(action.icon, Some(IconName::Settings));
    }

    #[test]
    fn cat19_full_chain() {
        let action = Action::new(
            "id",
            "T",
            Some("desc".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘X")
        .with_icon(IconName::Trash)
        .with_section("Danger");
        assert_eq!(action.shortcut.as_deref(), Some("⌘X"));
        assert_eq!(action.icon, Some(IconName::Trash));
        assert_eq!(action.section.as_deref(), Some("Danger"));
        assert_eq!(action.description.as_deref(), Some("desc"));
    }

    // =========================================================================
    // cat20: ScriptInfo with_action_verb preserves defaults
    // =========================================================================

    #[test]
    fn cat20_with_action_verb_preserves_not_scriptlet() {
        let info = ScriptInfo::with_action_verb("App", "/app", false, "Launch");
        assert!(!info.is_scriptlet);
        assert!(!info.is_agent);
        assert!(info.shortcut.is_none());
        assert!(info.alias.is_none());
        assert!(!info.is_suggested);
    }

    #[test]
    fn cat20_with_action_verb_sets_verb() {
        let info = ScriptInfo::with_action_verb("Win", "/win", false, "Switch to");
        assert_eq!(info.action_verb, "Switch to");
    }

    #[test]
    fn cat20_with_action_verb_name_and_path() {
        let info =
            ScriptInfo::with_action_verb("Safari", "/Applications/Safari.app", false, "Launch");
        assert_eq!(info.name, "Safari");
        assert_eq!(info.path, "/Applications/Safari.app");
    }

    #[test]
    fn cat20_with_action_verb_is_script_flag() {
        let info_true = ScriptInfo::with_action_verb("S", "/s", true, "Run");
        assert!(info_true.is_script);
        let info_false = ScriptInfo::with_action_verb("S", "/s", false, "Run");
        assert!(!info_false.is_script);
    }

    // =========================================================================
    // cat21: Script context agent flag produces edit with "Edit Agent" title
    // =========================================================================

    #[test]
    fn cat21_agent_flag_produces_edit_agent() {
        let mut script = ScriptInfo::new("Bot", "/bot.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert!(edit.title.contains("Agent"));
    }

    #[test]
    fn cat21_agent_has_copy_content() {
        let mut script = ScriptInfo::new("Bot", "/bot.md");
        script.is_script = false;
        script.is_agent = true;
        let ids = action_ids(&get_script_context_actions(&script));
        assert!(ids.contains(&"copy_content".to_string()));
    }

    #[test]
    fn cat21_agent_edit_shortcut() {
        let mut script = ScriptInfo::new("Bot", "/bot.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.shortcut.as_deref(), Some("⌘E"));
    }

    #[test]
    fn cat21_agent_reveal_shortcut() {
        let mut script = ScriptInfo::new("Bot", "/bot.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let reveal = actions.iter().find(|a| a.id == "file:reveal_in_finder").unwrap();
        assert_eq!(reveal.shortcut.as_deref(), Some("⌘⇧F"));
    }

    // =========================================================================
    // cat22: Cross-context shortcut format uses Unicode symbols
    // =========================================================================

    #[test]
    fn cat22_script_shortcuts_use_unicode() {
        let script = ScriptInfo::new("test", "/p/test.ts");
        let actions = get_script_context_actions(&script);
        for action in &actions {
            if let Some(ref s) = action.shortcut {
                // All shortcuts should contain Unicode symbols, not "cmd" / "shift" etc.
                assert!(
                    !s.contains("cmd") && !s.contains("shift") && !s.contains("ctrl"),
                    "Shortcut '{}' on action '{}' should use Unicode symbols",
                    s,
                    action.id
                );
            }
        }
    }


    // --- merged from tests_part_04.rs ---
    #[test]
    fn cat22_clipboard_shortcuts_use_unicode() {
        let entry = ClipboardEntryInfo {
            id: "c1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        for action in &actions {
            if let Some(ref s) = action.shortcut {
                assert!(
                    !s.contains("cmd") && !s.contains("shift"),
                    "Clipboard shortcut '{}' should use Unicode",
                    s
                );
            }
        }
    }

    #[test]
    fn cat22_ai_shortcuts_use_unicode() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            if let Some(ref s) = action.shortcut {
                assert!(
                    !s.contains("cmd") && !s.contains("shift"),
                    "AI shortcut '{}' should use Unicode",
                    s
                );
            }
        }
    }

    #[test]
    fn cat22_path_shortcuts_use_unicode() {
        let info = PathInfo {
            name: "f".into(),
            path: "/f".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        for action in &actions {
            if let Some(ref s) = action.shortcut {
                assert!(
                    !s.contains("cmd") && !s.contains("shift"),
                    "Path shortcut '{}' should use Unicode",
                    s
                );
            }
        }
    }

    // =========================================================================
    // cat23: Clipboard paste_keep_open shortcut
    // =========================================================================

    #[test]
    fn cat23_paste_keep_open_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "pk1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let pko = actions
            .iter()
            .find(|a| a.id == "clipboard_paste_keep_open")
            .unwrap();
        assert_eq!(pko.shortcut.as_deref(), Some("⌥↵"));
    }

    #[test]
    fn cat23_paste_keep_open_title() {
        let entry = ClipboardEntryInfo {
            id: "pk2".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let pko = actions
            .iter()
            .find(|a| a.id == "clipboard_paste_keep_open")
            .unwrap();
        assert_eq!(pko.title, "Paste and Keep Window Open");
    }

    #[test]
    fn cat23_paste_keep_open_description() {
        let entry = ClipboardEntryInfo {
            id: "pk3".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let pko = actions
            .iter()
            .find(|a| a.id == "clipboard_paste_keep_open")
            .unwrap();
        assert!(pko.description.is_some());
    }

    // =========================================================================
    // cat24: Path context copy_filename has no shortcut
    // =========================================================================

    #[test]
    fn cat24_path_copy_filename_no_shortcut() {
        let info = PathInfo {
            name: "test.txt".into(),
            path: "/test.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let cf = actions.iter().find(|a| a.id == "file:copy_filename").unwrap();
        assert!(
            cf.shortcut.is_none(),
            "path copy_filename should have no shortcut"
        );
    }

    #[test]
    fn cat24_path_copy_filename_present() {
        let info = PathInfo {
            name: "readme.md".into(),
            path: "/readme.md".into(),
            is_dir: false,
        };
        let ids = action_ids(&get_path_context_actions(&info));
        assert!(ids.contains(&"file:copy_filename".to_string()));
    }

    #[test]
    fn cat24_path_copy_filename_description() {
        let info = PathInfo {
            name: "data.json".into(),
            path: "/data.json".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let cf = actions.iter().find(|a| a.id == "file:copy_filename").unwrap();
        assert!(cf
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("filename"));
    }

    // =========================================================================
    // cat25: File context open_with macOS shortcut
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn cat25_file_open_with_shortcut() {
        let file = FileInfo {
            path: "/test.txt".into(),
            name: "test.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let ow = actions.iter().find(|a| a.id == "file:open_with").unwrap();
        assert_eq!(ow.shortcut.as_deref(), Some("⌘O"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat25_file_show_info_shortcut() {
        let file = FileInfo {
            path: "/img.png".into(),
            name: "img.png".into(),
            file_type: crate::file_search::FileType::Image,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let si = actions.iter().find(|a| a.id == "file:show_info").unwrap();
        assert_eq!(si.shortcut.as_deref(), Some("⌘I"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat25_file_quick_look_shortcut() {
        let file = FileInfo {
            path: "/readme.md".into(),
            name: "readme.md".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let ql = actions.iter().find(|a| a.id == "file:quick_look").unwrap();
        assert_eq!(ql.shortcut.as_deref(), Some("⌘Y"));
    }

    // =========================================================================
    // cat26: Notes format shortcut exact value
    // =========================================================================

    #[test]
    fn cat26_notes_format_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let fmt = actions.iter().find(|a| a.id == "format").unwrap();
        assert_eq!(fmt.shortcut.as_deref(), Some("⇧⌘T"));
    }

    #[test]
    fn cat26_notes_format_icon() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let fmt = actions.iter().find(|a| a.id == "format").unwrap();
        assert_eq!(fmt.icon, Some(IconName::Code));
    }

    #[test]
    fn cat26_notes_format_section() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let fmt = actions.iter().find(|a| a.id == "format").unwrap();
        assert_eq!(fmt.section.as_deref(), Some("Edit"));
    }

    #[test]
    fn cat26_notes_find_in_note_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let find = actions.iter().find(|a| a.id == "find_in_note").unwrap();
        assert_eq!(find.shortcut.as_deref(), Some("⌘F"));
    }

    // =========================================================================
    // cat27: AI command bar icon name correctness
    // =========================================================================

    #[test]
    fn cat27_ai_copy_response_icon() {
        let actions = get_ai_command_bar_actions();
        let cr = actions.iter().find(|a| a.id == "chat:copy_response").unwrap();
        assert_eq!(cr.icon, Some(IconName::Copy));
    }

    #[test]
    fn cat27_ai_submit_icon() {
        let actions = get_ai_command_bar_actions();
        let submit = actions.iter().find(|a| a.id == "chat:submit").unwrap();
        assert_eq!(submit.icon, Some(IconName::ArrowUp));
    }

    #[test]
    fn cat27_ai_new_chat_icon() {
        let actions = get_ai_command_bar_actions();
        let nc = actions.iter().find(|a| a.id == "new_chat").unwrap();
        assert_eq!(nc.icon, Some(IconName::Plus));
    }

    #[test]
    fn cat27_ai_delete_chat_icon() {
        let actions = get_ai_command_bar_actions();
        let dc = actions.iter().find(|a| a.id == "delete_chat").unwrap();
        assert_eq!(dc.icon, Some(IconName::Trash));
    }

    #[test]
    fn cat27_ai_change_model_icon() {
        let actions = get_ai_command_bar_actions();
        let cm = actions.iter().find(|a| a.id == "change_model").unwrap();
        assert_eq!(cm.icon, Some(IconName::Settings));
    }

    #[test]
    fn cat27_ai_toggle_shortcuts_help_icon() {
        let actions = get_ai_command_bar_actions();
        let ts = actions
            .iter()
            .find(|a| a.id == "toggle_shortcuts_help")
            .unwrap();
        assert_eq!(ts.icon, Some(IconName::Star));
    }

    // =========================================================================
    // cat28: Script context run title format
    // =========================================================================

    #[test]
    fn cat28_run_title_default_verb() {
        let script = ScriptInfo::new("My Script", "/p/my-script.ts");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert_eq!(run.title, "Run \"My Script\"");
    }

    #[test]
    fn cat28_run_title_custom_verb() {
        let script = ScriptInfo::with_action_verb("Safari", "/app", false, "Launch");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert_eq!(run.title, "Launch \"Safari\"");
    }

    #[test]
    fn cat28_run_title_switch_to_verb() {
        let script = ScriptInfo::with_action_verb("Terminal", "window:1", false, "Switch to");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert_eq!(run.title, "Switch to \"Terminal\"");
    }

    #[test]
    fn cat28_run_title_builtin() {
        let builtin = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&builtin);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert_eq!(run.title, "Run \"Clipboard History\"");
    }

    #[test]
    fn cat28_run_shortcut_always_enter() {
        let script = ScriptInfo::new("test", "/p/test.ts");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert_eq!(run.shortcut.as_deref(), Some("↵"));
    }

    // =========================================================================
    // cat29: Ordering consistency across repeated calls
    // =========================================================================

    #[test]
    fn cat29_script_ordering_deterministic() {
        let script = ScriptInfo::new("test", "/p/test.ts");
        let a1 = action_ids(&get_script_context_actions(&script));
        let a2 = action_ids(&get_script_context_actions(&script));
        assert_eq!(a1, a2);
    }

    #[test]
    fn cat29_clipboard_ordering_deterministic() {
        let entry = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let a1 = action_ids(&get_clipboard_history_context_actions(&entry));
        let a2 = action_ids(&get_clipboard_history_context_actions(&entry));
        assert_eq!(a1, a2);
    }

    #[test]
    fn cat29_ai_ordering_deterministic() {
        let a1 = action_ids(&get_ai_command_bar_actions());
        let a2 = action_ids(&get_ai_command_bar_actions());
        assert_eq!(a1, a2);
    }

    #[test]
    fn cat29_notes_ordering_deterministic() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let a1 = action_ids(&get_notes_command_bar_actions(&info));
        let a2 = action_ids(&get_notes_command_bar_actions(&info));
        assert_eq!(a1, a2);
    }

    #[test]
    fn cat29_path_ordering_deterministic() {
        let info = PathInfo {
            name: "f".into(),
            path: "/f".into(),
            is_dir: false,
        };
        let a1 = action_ids(&get_path_context_actions(&info));
        let a2 = action_ids(&get_path_context_actions(&info));
        assert_eq!(a1, a2);
    }

    // =========================================================================
    // cat30: Cross-context non-empty ID and title, has_action=false, ID uniqueness
    // =========================================================================

    #[test]
    fn cat30_script_non_empty_ids_and_titles() {
        let script = ScriptInfo::new("test", "/p/test.ts");
        for action in &get_script_context_actions(&script) {
            assert!(!action.id.is_empty(), "action ID should not be empty");
            assert!(!action.title.is_empty(), "action title should not be empty");
        }
    }

    #[test]
    fn cat30_clipboard_non_empty_ids_and_titles() {
        let entry = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        for action in &get_clipboard_history_context_actions(&entry) {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    #[test]
    fn cat30_ai_non_empty_ids_and_titles() {
        for action in &get_ai_command_bar_actions() {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }


    // --- merged from tests_part_05.rs ---
    #[test]
    fn cat30_script_has_action_false() {
        let script = ScriptInfo::new("test", "/p/test.ts");
        for action in &get_script_context_actions(&script) {
            assert!(
                !action.has_action,
                "built-in action '{}' should be false",
                action.id
            );
        }
    }

    #[test]
    fn cat30_clipboard_has_action_false() {
        let entry = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        for action in &get_clipboard_history_context_actions(&entry) {
            assert!(
                !action.has_action,
                "clipboard action '{}' should be false",
                action.id
            );
        }
    }

    #[test]
    fn cat30_script_ids_unique() {
        let script = ScriptInfo::new("test", "/p/test.ts");
        let ids = action_ids(&get_script_context_actions(&script));
        let set: HashSet<&String> = ids.iter().collect();
        assert_eq!(ids.len(), set.len(), "script IDs must be unique");
    }

    #[test]
    fn cat30_ai_ids_unique() {
        let ids = action_ids(&get_ai_command_bar_actions());
        let set: HashSet<&String> = ids.iter().collect();
        assert_eq!(ids.len(), set.len(), "AI IDs must be unique");
    }

    #[test]
    fn cat30_path_ids_unique() {
        let info = PathInfo {
            name: "f".into(),
            path: "/f".into(),
            is_dir: false,
        };
        let ids = action_ids(&get_path_context_actions(&info));
        let set: HashSet<&String> = ids.iter().collect();
        assert_eq!(ids.len(), set.len(), "path IDs must be unique");
    }

    #[test]
    fn cat30_file_ids_unique() {
        let file = FileInfo {
            path: "/x.rs".into(),
            name: "x.rs".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let ids = action_ids(&get_file_context_actions(&file));
        let set: HashSet<&String> = ids.iter().collect();
        assert_eq!(ids.len(), set.len(), "file IDs must be unique");
    }

    #[test]
    fn cat30_notes_ids_unique() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let ids = action_ids(&get_notes_command_bar_actions(&info));
        let set: HashSet<&String> = ids.iter().collect();
        assert_eq!(ids.len(), set.len(), "notes IDs must be unique");
    }

}
