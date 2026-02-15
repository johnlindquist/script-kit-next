//! Batch 12: Dialog Built-in Action Validation Tests
//!
//! 150+ tests across 30 categories validating random behaviors of
//! built-in action window dialogs.
//!
//! Focus areas for this batch:
//! - CommandBarConfig preset field validation (all presets, all fields)
//! - Clipboard text vs image action count delta and exact IDs
//! - Scriptlet context with_custom shortcut/alias dynamic action switching
//! - Agent context description keyword content
//! - Note switcher multi-note sorting and section assignment
//! - AI command bar section-to-action-count mapping
//! - Path context description substring matching
//! - Chat context multi-model ID generation patterns
//! - Score_action stacking with multi-field matches
//! - build_grouped_items_static with alternating sections
//! - Cross-context action description non-empty invariant
//! - format_shortcut_hint chaining (multiple modifiers in sequence)
//! - Action builder with_icon and with_section field isolation
//! - ScriptInfo with_is_script constructor
//! - Deeplink name with mixed Unicode scripts
//! - parse_shortcut_keycaps special symbol recognition
//! - Clipboard destructive action shortcut exact values
//! - File context macOS action count (file vs dir)
//! - New chat action ID prefix patterns
//! - Notes command bar section-to-action mapping

#[cfg(test)]
mod tests {
    // --- merged from tests_part_01.rs ---
    use super::super::builders::*;
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

    // =========================================================================
    // Helper
    // =========================================================================
    fn action_ids(actions: &[Action]) -> Vec<String> {
        actions.iter().map(|a| a.id.clone()).collect()
    }

    // =========================================================================
    // 1. CommandBarConfig preset field validation — all four presets
    // =========================================================================

    #[test]
    fn cat01_command_bar_default_dialog_config() {
        let cfg = ActionsDialogConfig::default();
        assert_eq!(cfg.search_position, SearchPosition::Bottom);
        assert_eq!(cfg.section_style, SectionStyle::Separators);
        assert_eq!(cfg.anchor, AnchorPosition::Bottom);
        assert!(!cfg.show_icons);
        assert!(!cfg.show_footer);
    }

    #[test]
    fn cat01_command_bar_ai_style_dialog_config() {
        let cfg = super::super::command_bar::CommandBarConfig::ai_style();
        assert_eq!(cfg.dialog_config.search_position, SearchPosition::Top);
        assert_eq!(cfg.dialog_config.section_style, SectionStyle::Headers);
        assert_eq!(cfg.dialog_config.anchor, AnchorPosition::Top);
        assert!(cfg.dialog_config.show_icons);
        assert!(cfg.dialog_config.show_footer);
        assert!(cfg.close_on_select);
        assert!(cfg.close_on_click_outside);
        assert!(cfg.close_on_escape);
    }

    #[test]
    fn cat01_command_bar_main_menu_style_dialog_config() {
        let cfg = super::super::command_bar::CommandBarConfig::main_menu_style();
        assert_eq!(cfg.dialog_config.search_position, SearchPosition::Bottom);
        assert_eq!(cfg.dialog_config.section_style, SectionStyle::Separators);
        assert_eq!(cfg.dialog_config.anchor, AnchorPosition::Bottom);
        assert!(!cfg.dialog_config.show_icons);
        assert!(!cfg.dialog_config.show_footer);
    }

    #[test]
    fn cat01_command_bar_no_search_dialog_config() {
        let cfg = super::super::command_bar::CommandBarConfig::no_search();
        assert_eq!(cfg.dialog_config.search_position, SearchPosition::Hidden);
        assert_eq!(cfg.dialog_config.section_style, SectionStyle::Separators);
        assert_eq!(cfg.dialog_config.anchor, AnchorPosition::Bottom);
        assert!(!cfg.dialog_config.show_icons);
        assert!(!cfg.dialog_config.show_footer);
    }

    #[test]
    fn cat01_command_bar_notes_style_dialog_config() {
        let cfg = super::super::command_bar::CommandBarConfig::notes_style();
        assert_eq!(cfg.dialog_config.search_position, SearchPosition::Top);
        assert_eq!(cfg.dialog_config.section_style, SectionStyle::Separators);
        assert_eq!(cfg.dialog_config.anchor, AnchorPosition::Top);
        assert!(cfg.dialog_config.show_icons);
        assert!(cfg.dialog_config.show_footer);
    }

    #[test]
    fn cat01_command_bar_all_presets_close_defaults_true() {
        for cfg in [
            super::super::command_bar::CommandBarConfig::default(),
            super::super::command_bar::CommandBarConfig::ai_style(),
            super::super::command_bar::CommandBarConfig::main_menu_style(),
            super::super::command_bar::CommandBarConfig::no_search(),
            super::super::command_bar::CommandBarConfig::notes_style(),
        ] {
            assert!(cfg.close_on_select, "close_on_select should default true");
            assert!(
                cfg.close_on_click_outside,
                "close_on_click_outside should default true"
            );
            assert!(cfg.close_on_escape, "close_on_escape should default true");
        }
    }

    // =========================================================================
    // 2. Clipboard text vs image — exact ID difference
    // =========================================================================

    fn make_text_entry() -> ClipboardEntryInfo {
        ClipboardEntryInfo {
            id: "t1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        }
    }

    fn make_image_entry() -> ClipboardEntryInfo {
        ClipboardEntryInfo {
            id: "i1".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "Image".into(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        }
    }

    #[test]
    fn cat02_image_has_ocr_text_does_not() {
        let text_ids = action_ids(&get_clipboard_history_context_actions(&make_text_entry()));
        let image_ids = action_ids(&get_clipboard_history_context_actions(&make_image_entry()));
        assert!(!text_ids.contains(&"clip:clipboard_ocr".to_string()));
        assert!(image_ids.contains(&"clip:clipboard_ocr".to_string()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat02_image_has_open_with_and_cleanshot_text_does_not() {
        let text_ids = action_ids(&get_clipboard_history_context_actions(&make_text_entry()));
        let image_ids = action_ids(&get_clipboard_history_context_actions(&make_image_entry()));
        for id in [
            "clip:clipboard_open_with",
            "clip:clipboard_annotate_cleanshot",
            "clip:clipboard_upload_cleanshot",
        ] {
            assert!(
                !text_ids.contains(&id.to_string()),
                "text should not have {}",
                id
            );
            assert!(
                image_ids.contains(&id.to_string()),
                "image should have {}",
                id
            );
        }
    }

    #[test]
    fn cat02_image_more_actions_than_text() {
        let text_count = get_clipboard_history_context_actions(&make_text_entry()).len();
        let image_count = get_clipboard_history_context_actions(&make_image_entry()).len();
        assert!(
            image_count > text_count,
            "image {} > text {}",
            image_count,
            text_count
        );
    }

    #[test]
    fn cat02_shared_actions_between_text_and_image() {
        let text_ids: HashSet<String> =
            action_ids(&get_clipboard_history_context_actions(&make_text_entry()))
                .into_iter()
                .collect();
        let image_ids: HashSet<String> =
            action_ids(&get_clipboard_history_context_actions(&make_image_entry()))
                .into_iter()
                .collect();
        let shared: Vec<&str> = vec![
            "clip:clipboard_paste",
            "clip:clipboard_copy",
            "clip:clipboard_paste_keep_open",
            "clip:clipboard_share",
            "clip:clipboard_attach_to_ai",
            "clip:clipboard_pin",
            "clip:clipboard_save_snippet",
            "clip:clipboard_save_file",
            "clip:clipboard_delete",
            "clip:clipboard_delete_multiple",
            "clip:clipboard_delete_all",
        ];
        for id in shared {
            assert!(text_ids.contains(id), "text missing {}", id);
            assert!(image_ids.contains(id), "image missing {}", id);
        }
    }

    // =========================================================================
    // 3. Scriptlet context shortcut/alias dynamic switching
    // =========================================================================

    #[test]
    fn cat03_scriptlet_with_custom_no_shortcut_no_alias() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"add_shortcut".to_string()));
        assert!(ids.contains(&"add_alias".to_string()));
        assert!(!ids.contains(&"update_shortcut".to_string()));
        assert!(!ids.contains(&"update_alias".to_string()));
    }

    #[test]
    fn cat03_scriptlet_with_custom_has_shortcut_has_alias() {
        let script = ScriptInfo::scriptlet(
            "Test",
            "/path/test.md",
            Some("cmd+t".into()),
            Some("ts".into()),
        );
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"update_shortcut".to_string()));
        assert!(ids.contains(&"remove_shortcut".to_string()));
        assert!(ids.contains(&"update_alias".to_string()));
        assert!(ids.contains(&"remove_alias".to_string()));
        assert!(!ids.contains(&"add_shortcut".to_string()));
        assert!(!ids.contains(&"add_alias".to_string()));
    }

    #[test]
    fn cat03_scriptlet_with_custom_has_shortcut_no_alias() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", Some("cmd+t".into()), None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"update_shortcut".to_string()));
        assert!(ids.contains(&"add_alias".to_string()));
    }

    #[test]
    fn cat03_scriptlet_with_custom_no_shortcut_has_alias() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, Some("ts".into()));
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"add_shortcut".to_string()));
        assert!(ids.contains(&"update_alias".to_string()));
    }

    #[test]
    fn cat03_scriptlet_context_always_has_edit_reveal_copy() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"edit_scriptlet".to_string()));
        assert!(ids.contains(&"reveal_scriptlet_in_finder".to_string()));
        assert!(ids.contains(&"copy_scriptlet_path".to_string()));
        assert!(ids.contains(&"copy_content".to_string()));
        assert!(ids.contains(&"copy_deeplink".to_string()));
    }

    // =========================================================================
    // 4. Agent context description keyword content
    // =========================================================================

    #[test]
    fn cat04_agent_edit_title_says_agent() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    #[test]
    fn cat04_agent_edit_description_mentions_agent() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert!(edit.description.as_ref().unwrap().contains("agent"));
    }

    #[test]
    fn cat04_agent_reveal_description_mentions_agent() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
        assert!(reveal.description.as_ref().unwrap().contains("agent"));
    }

    #[test]
    fn cat04_agent_copy_path_description_mentions_agent() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
        assert!(cp.description.as_ref().unwrap().contains("agent"));
    }

    #[test]
    fn cat04_agent_no_view_logs() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    // =========================================================================
    // 5. Note switcher multi-note section assignment
    // =========================================================================

    fn make_note(id: &str, title: &str, pinned: bool, current: bool) -> NoteSwitcherNoteInfo {
        NoteSwitcherNoteInfo {
            id: id.into(),
            title: title.into(),
            char_count: 100,
            is_current: current,
            is_pinned: pinned,
            preview: "Some preview text".into(),
            relative_time: "2m ago".into(),
        }
    }

    #[test]
    fn cat05_pinned_notes_in_pinned_section() {
        let notes = vec![make_note("1", "Pinned Note", true, false)];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
    }

    #[test]
    fn cat05_unpinned_notes_in_recent_section() {
        let notes = vec![make_note("2", "Recent Note", false, false)];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn cat05_mixed_notes_correct_sections() {
        let notes = vec![
            make_note("1", "Pinned", true, false),
            make_note("2", "Recent", false, false),
            make_note("3", "Also Pinned", true, true),
        ];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
        assert_eq!(actions[1].section.as_deref(), Some("Recent"));
        assert_eq!(actions[2].section.as_deref(), Some("Pinned"));
    }

    #[test]
    fn cat05_current_note_gets_bullet_prefix() {
        let notes = vec![make_note("1", "My Note", false, true)];
        let actions = get_note_switcher_actions(&notes);
        assert!(actions[0].title.starts_with("• "));
    }

    #[test]
    fn cat05_non_current_note_no_bullet() {
        let notes = vec![make_note("1", "My Note", false, false)];
        let actions = get_note_switcher_actions(&notes);
        assert!(!actions[0].title.starts_with("• "));
    }

    #[test]
    fn cat05_note_ids_follow_pattern() {
        let notes = vec![
            make_note("abc-123", "Note A", false, false),
            make_note("def-456", "Note B", true, false),
        ];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].id, "note_abc-123");
        assert_eq!(actions[1].id, "note_def-456");
    }

    // =========================================================================
    // 6. AI command bar section-to-action-count mapping
    // =========================================================================

    #[test]
    fn cat06_ai_response_section_has_3_actions() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Response"))
            .count();
        assert_eq!(count, 3, "Response section should have 3 actions");
    }

    #[test]
    fn cat06_ai_actions_section_has_4_actions() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Actions"))
            .count();
        assert_eq!(count, 4, "Actions section should have 4 actions (submit, new_chat, delete_chat, branch_from_last)");
    }

    #[test]
    fn cat06_ai_attachments_section_has_2_actions() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Attachments"))
            .count();
        assert_eq!(count, 2, "Attachments section should have 2 actions");
    }

    #[test]
    fn cat06_ai_export_section_has_1_action() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Export"))
            .count();
        assert_eq!(count, 1, "Export section should have 1 action");
    }

    #[test]
    fn cat06_ai_help_section_has_1_action() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Help"))
            .count();
        assert_eq!(count, 1, "Help section should have 1 action");
    }

    #[test]
    fn cat06_ai_settings_section_has_1_action() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Settings"))
            .count();
        assert_eq!(count, 1, "Settings section should have 1 action");
    }

    #[test]
    fn cat06_ai_total_12_actions() {
        assert_eq!(get_ai_command_bar_actions().len(), 12);
    }


    // --- merged from tests_part_02.rs ---
    #[test]
    fn cat06_ai_6_unique_sections() {
        let actions = get_ai_command_bar_actions();
        let sections: HashSet<_> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        assert_eq!(sections.len(), 6);
    }

    // =========================================================================
    // 7. Path context description substring matching
    // =========================================================================

    #[test]
    fn cat07_path_dir_open_description_mentions_directory() {
        let info = PathInfo {
            path: "/tmp/docs".into(),
            name: "docs".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let open = actions.iter().find(|a| a.id == "file:open_directory").unwrap();
        assert!(
            open.description.as_ref().unwrap().contains("directory")
                || open.description.as_ref().unwrap().contains("Navigate")
        );
    }

    #[test]
    fn cat07_path_file_select_description_mentions_file() {
        let info = PathInfo {
            path: "/tmp/file.txt".into(),
            name: "file.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let sel = actions.iter().find(|a| a.id == "file:select_file").unwrap();
        assert!(
            sel.description.as_ref().unwrap().contains("file")
                || sel.description.as_ref().unwrap().contains("Submit")
        );
    }

    #[test]
    fn cat07_path_trash_dir_says_folder() {
        let info = PathInfo {
            path: "/tmp/docs".into(),
            name: "docs".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.iter().find(|a| a.id == "file:move_to_trash").unwrap();
        assert!(trash.description.as_ref().unwrap().contains("folder"));
    }

    #[test]
    fn cat07_path_trash_file_says_file() {
        let info = PathInfo {
            path: "/tmp/a.txt".into(),
            name: "a.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.iter().find(|a| a.id == "file:move_to_trash").unwrap();
        assert!(trash.description.as_ref().unwrap().contains("file"));
    }

    #[test]
    fn cat07_path_open_in_editor_mentions_editor() {
        let info = PathInfo {
            path: "/tmp/a.txt".into(),
            name: "a.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let ed = actions.iter().find(|a| a.id == "file:open_in_editor").unwrap();
        assert!(
            ed.description.as_ref().unwrap().contains("$EDITOR")
                || ed.description.as_ref().unwrap().contains("editor")
        );
    }

    // =========================================================================
    // 8. Chat context multi-model ID generation
    // =========================================================================

    #[test]
    fn cat08_multiple_models_generate_sequential_ids() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![
                ChatModelInfo {
                    id: "gpt-4".into(),
                    display_name: "GPT-4".into(),
                    provider: "OpenAI".into(),
                },
                ChatModelInfo {
                    id: "claude-3".into(),
                    display_name: "Claude 3".into(),
                    provider: "Anthropic".into(),
                },
                ChatModelInfo {
                    id: "gemini".into(),
                    display_name: "Gemini".into(),
                    provider: "Google".into(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "chat:select_model_gpt-4"));
        assert!(actions.iter().any(|a| a.id == "chat:select_model_claude-3"));
        assert!(actions.iter().any(|a| a.id == "chat:select_model_gemini"));
    }

    #[test]
    fn cat08_current_model_gets_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".into()),
            available_models: vec![
                ChatModelInfo {
                    id: "gpt-4".into(),
                    display_name: "GPT-4".into(),
                    provider: "OpenAI".into(),
                },
                ChatModelInfo {
                    id: "claude-3".into(),
                    display_name: "Claude 3".into(),
                    provider: "Anthropic".into(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let gpt = actions
            .iter()
            .find(|a| a.id == "chat:select_model_gpt-4")
            .unwrap();
        assert!(gpt.title.contains("✓"));
        let claude = actions
            .iter()
            .find(|a| a.id == "chat:select_model_claude-3")
            .unwrap();
        assert!(!claude.title.contains("✓"));
    }

    #[test]
    fn cat08_no_models_still_has_continue() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "chat:continue_in_chat"));
    }

    #[test]
    fn cat08_model_description_shows_provider() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "m1".into(),
                display_name: "Model 1".into(),
                provider: "Anthropic".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let m = actions.iter().find(|a| a.id == "chat:select_model_m1").unwrap();
        assert!(m.description.as_ref().unwrap().contains("Anthropic"));
    }

    #[test]
    fn cat08_has_response_adds_copy_response() {
        let with = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: true,
        };
        let without = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        assert!(get_chat_context_actions(&with)
            .iter()
            .any(|a| a.id == "chat:copy_response"));
        assert!(!get_chat_context_actions(&without)
            .iter()
            .any(|a| a.id == "chat:copy_response"));
    }

    #[test]
    fn cat08_has_messages_adds_clear_conversation() {
        let with = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let without = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        assert!(get_chat_context_actions(&with)
            .iter()
            .any(|a| a.id == "chat:clear_conversation"));
        assert!(!get_chat_context_actions(&without)
            .iter()
            .any(|a| a.id == "chat:clear_conversation"));
    }

    // =========================================================================
    // 9. Score_action stacking with multi-field matches
    // =========================================================================

    #[test]
    fn cat09_prefix_plus_description_stacks() {
        let action = Action::new(
            "script:edit",
            "Edit Script",
            Some("Edit the script file".into()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "edit");
        assert!(score >= 115, "prefix(100)+desc(15)={}", score);
    }

    #[test]
    fn cat09_prefix_plus_shortcut_stacks() {
        let action =
            Action::new("script:edit", "Edit", None, ActionCategory::ScriptContext).with_shortcut("script:edit");
        let score = ActionsDialog::score_action(&action, "edit");
        assert!(score >= 110, "prefix(100)+shortcut(10)={}", score);
    }

    #[test]
    fn cat09_contains_plus_description_stacks() {
        let action = Action::new(
            "x",
            "Copy Edit Path",
            Some("Edit mode".into()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "edit");
        assert!(score >= 65, "contains(50)+desc(15)={}", score);
    }

    #[test]
    fn cat09_no_match_returns_zero() {
        let action = Action::new(
            "x",
            "Foo",
            Some("Bar".into()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(ActionsDialog::score_action(&action, "zzz"), 0);
    }

    #[test]
    fn cat09_empty_query_matches_prefix() {
        let action = Action::new("x", "Anything", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "");
        assert!(
            score >= 100,
            "empty query is prefix of everything: {}",
            score
        );
    }

    // =========================================================================
    // 10. build_grouped_items_static with alternating sections
    // =========================================================================

    #[test]
    fn cat10_alternating_sections_produce_multiple_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Alpha"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Beta"),
            Action::new("c", "C", None, ActionCategory::ScriptContext).with_section("Alpha"),
        ];
        let filtered: Vec<usize> = (0..3).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        let header_count = grouped
            .iter()
            .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 3, "Alpha, Beta, Alpha = 3 headers");
    }

    #[test]
    fn cat10_same_section_no_duplicate_header() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Same"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Same"),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        let header_count = grouped
            .iter()
            .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 1);
    }

    #[test]
    fn cat10_separators_no_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("X"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Y"),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        let header_count = grouped
            .iter()
            .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 0);
    }

    #[test]
    fn cat10_none_style_no_headers() {
        let actions =
            vec![Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("X")];
        let filtered: Vec<usize> = vec![0];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        let header_count = grouped
            .iter()
            .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 0);
    }

    #[test]
    fn cat10_empty_filtered_returns_empty() {
        let actions = vec![Action::new("a", "A", None, ActionCategory::ScriptContext)];
        let grouped = build_grouped_items_static(&actions, &[], SectionStyle::Headers);
        assert!(grouped.is_empty());
    }

    // =========================================================================
    // 11. Cross-context action description non-empty invariant
    // =========================================================================

    #[test]
    fn cat11_script_actions_all_have_descriptions() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        for action in &get_script_context_actions(&script) {
            assert!(
                action.description.is_some(),
                "Script action '{}' missing description",
                action.id
            );
        }
    }

    #[test]
    fn cat11_clipboard_actions_all_have_descriptions() {
        for action in &get_clipboard_history_context_actions(&make_text_entry()) {
            assert!(
                action.description.is_some(),
                "Clipboard action '{}' missing description",
                action.id
            );
        }
    }

    #[test]
    fn cat11_ai_actions_all_have_descriptions() {
        for action in &get_ai_command_bar_actions() {
            assert!(
                action.description.is_some(),
                "AI action '{}' missing description",
                action.id
            );
        }
    }

    #[test]
    fn cat11_path_actions_all_have_descriptions() {
        let info = PathInfo {
            path: "/tmp/x".into(),
            name: "x".into(),
            is_dir: false,
        };
        for action in &get_path_context_actions(&info) {
            assert!(
                action.description.is_some(),
                "Path action '{}' missing description",
                action.id
            );
        }
    }

    #[test]
    fn cat11_file_actions_all_have_descriptions() {
        let info = FileInfo {
            path: "/tmp/x.txt".into(),
            name: "x.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        for action in &get_file_context_actions(&info) {
            assert!(
                action.description.is_some(),
                "File action '{}' missing description",
                action.id
            );
        }
    }

    #[test]
    fn cat11_notes_actions_all_have_descriptions() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for action in &get_notes_command_bar_actions(&info) {
            assert!(
                action.description.is_some(),
                "Notes action '{}' missing description",
                action.id
            );
        }
    }

    // =========================================================================
    // 12. format_shortcut_hint chaining — multiple modifiers
    // =========================================================================

    // format_shortcut_hint is private, but we test it indirectly through scriptlet actions

    // --- merged from tests_part_03.rs ---
    #[test]
    fn cat12_scriptlet_shortcut_cmd_shift_becomes_symbols() {
        let mut scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Act".into(),
            command: "act".into(),
            tool: "bash".into(),
            code: "echo".into(),
            inputs: vec![],
            shortcut: Some("cmd+shift+c".into()),
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions[0].shortcut.as_deref(), Some("⌘⇧C"));
    }

    #[test]
    fn cat12_scriptlet_shortcut_ctrl_alt_becomes_symbols() {
        let mut scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Act".into(),
            command: "act".into(),
            tool: "bash".into(),
            code: "echo".into(),
            inputs: vec![],
            shortcut: Some("ctrl+alt+x".into()),
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions[0].shortcut.as_deref(), Some("⌃⌥X"));
    }

    #[test]
    fn cat12_scriptlet_shortcut_no_shortcut_is_none() {
        let mut scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Act".into(),
            command: "act".into(),
            tool: "bash".into(),
            code: "echo".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert!(actions[0].shortcut.is_none());
    }

    #[test]
    fn cat12_scriptlet_shortcut_single_key() {
        let mut scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Act".into(),
            command: "act".into(),
            tool: "bash".into(),
            code: "echo".into(),
            inputs: vec![],
            shortcut: Some("a".into()),
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions[0].shortcut.as_deref(), Some("A"));
    }

    // =========================================================================
    // 13. Action builder with_icon and with_section field isolation
    // =========================================================================

    #[test]
    fn cat13_with_icon_does_not_affect_section() {
        let action =
            Action::new("x", "X", None, ActionCategory::ScriptContext).with_icon(IconName::Copy);
        assert_eq!(action.icon, Some(IconName::Copy));
        assert!(action.section.is_none());
    }

    #[test]
    fn cat13_with_section_does_not_affect_icon() {
        let action =
            Action::new("x", "X", None, ActionCategory::ScriptContext).with_section("Test");
        assert!(action.icon.is_none());
        assert_eq!(action.section.as_deref(), Some("Test"));
    }

    #[test]
    fn cat13_chaining_icon_then_section() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_icon(IconName::Star)
            .with_section("Sec");
        assert_eq!(action.icon, Some(IconName::Star));
        assert_eq!(action.section.as_deref(), Some("Sec"));
    }

    #[test]
    fn cat13_chaining_section_then_icon() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_section("Sec")
            .with_icon(IconName::Star);
        assert_eq!(action.icon, Some(IconName::Star));
        assert_eq!(action.section.as_deref(), Some("Sec"));
    }

    #[test]
    fn cat13_with_shortcut_preserves_icon_section() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_icon(IconName::Plus)
            .with_section("S")
            .with_shortcut("⌘X");
        assert_eq!(action.icon, Some(IconName::Plus));
        assert_eq!(action.section.as_deref(), Some("S"));
        assert_eq!(action.shortcut.as_deref(), Some("⌘X"));
    }

    // =========================================================================
    // 14. ScriptInfo with_is_script constructor
    // =========================================================================

    #[test]
    fn cat14_with_is_script_true() {
        let s = ScriptInfo::with_is_script("test", "/path", true);
        assert!(s.is_script);
        assert!(!s.is_scriptlet);
        assert!(!s.is_agent);
        assert_eq!(s.action_verb, "Run");
    }

    #[test]
    fn cat14_with_is_script_false() {
        let s = ScriptInfo::with_is_script("builtin", "", false);
        assert!(!s.is_script);
        assert!(!s.is_scriptlet);
        assert!(!s.is_agent);
    }

    #[test]
    fn cat14_with_is_script_false_has_limited_actions() {
        let s = ScriptInfo::with_is_script("App", "", false);
        let actions = get_script_context_actions(&s);
        let ids = action_ids(&actions);
        // Non-script, non-scriptlet, non-agent = builtin-like
        assert!(!ids.contains(&"edit_script".to_string()));
        assert!(!ids.contains(&"view_logs".to_string()));
        assert!(ids.contains(&"run_script".to_string()));
        assert!(ids.contains(&"copy_deeplink".to_string()));
    }

    // =========================================================================
    // 15. Deeplink name with mixed Unicode scripts
    // =========================================================================

    #[test]
    fn cat15_deeplink_cjk_preserved() {
        let result = to_deeplink_name("日本語テスト");
        assert!(result.contains("%E6%97%A5"));
        assert!(result.contains("%E8%AA%9E"));
    }

    #[test]
    fn cat15_deeplink_mixed_ascii_and_accents() {
        let result = to_deeplink_name("Café Script");
        assert!(result.contains("caf"));
        assert!(result.contains("%C3%A9"));
    }

    #[test]
    fn cat15_deeplink_all_special_chars() {
        let result = to_deeplink_name("!@#$%^&*()");
        assert_eq!(result, "_unnamed");
    }

    #[test]
    fn cat15_deeplink_leading_trailing_stripped() {
        let result = to_deeplink_name("  hello  ");
        assert_eq!(result, "hello");
    }

    #[test]
    fn cat15_deeplink_consecutive_specials_collapsed() {
        let result = to_deeplink_name("a---b___c");
        assert_eq!(result, "a-b-c");
    }

    // =========================================================================
    // 16. parse_shortcut_keycaps special symbol recognition
    // =========================================================================

    #[test]
    fn cat16_modifier_symbols_are_individual_keycaps() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⇧C");
        assert_eq!(keycaps, vec!["⌘", "⇧", "C"]);
    }

    #[test]
    fn cat16_enter_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("↵");
        assert_eq!(keycaps, vec!["↵"]);
    }

    #[test]
    fn cat16_escape_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⎋");
        assert_eq!(keycaps, vec!["⎋"]);
    }

    #[test]
    fn cat16_arrow_symbols() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("↑↓←→");
        assert_eq!(keycaps, vec!["↑", "↓", "←", "→"]);
    }

    #[test]
    fn cat16_backspace_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⌫");
        assert_eq!(keycaps, vec!["⌘", "⌫"]);
    }

    #[test]
    fn cat16_space_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("␣");
        assert_eq!(keycaps, vec!["␣"]);
    }

    #[test]
    fn cat16_tab_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⇥");
        assert_eq!(keycaps, vec!["⇥"]);
    }

    #[test]
    fn cat16_lowercase_uppercased() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘c");
        assert_eq!(keycaps, vec!["⌘", "C"]);
    }

    // =========================================================================
    // 17. Clipboard destructive action shortcut exact values
    // =========================================================================

    #[test]
    fn cat17_delete_shortcut() {
        let actions = get_clipboard_history_context_actions(&make_text_entry());
        let del = actions.iter().find(|a| a.id == "clip:clipboard_delete").unwrap();
        assert_eq!(del.shortcut.as_deref(), Some("⌃X"));
    }

    #[test]
    fn cat17_delete_multiple_shortcut() {
        let actions = get_clipboard_history_context_actions(&make_text_entry());
        let del = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_delete_multiple")
            .unwrap();
        assert_eq!(del.shortcut.as_deref(), Some("⇧⌘X"));
    }

    #[test]
    fn cat17_delete_all_shortcut() {
        let actions = get_clipboard_history_context_actions(&make_text_entry());
        let del = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_delete_all")
            .unwrap();
        assert_eq!(del.shortcut.as_deref(), Some("⌃⇧X"));
    }

    #[test]
    fn cat17_destructive_actions_are_last_three() {
        let actions = get_clipboard_history_context_actions(&make_text_entry());
        let len = actions.len();
        assert_eq!(actions[len - 3].id, "clip:clipboard_delete");
        assert_eq!(actions[len - 2].id, "clip:clipboard_delete_multiple");
        assert_eq!(actions[len - 1].id, "clip:clipboard_delete_all");
    }

    // =========================================================================
    // 18. File context macOS action count (file vs dir)
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn cat18_file_has_more_actions_than_dir_on_macos() {
        let file_info = FileInfo {
            path: "/tmp/a.txt".into(),
            name: "a.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let dir_info = FileInfo {
            path: "/tmp/docs".into(),
            name: "docs".into(),
            file_type: crate::file_search::FileType::Directory,
            is_dir: true,
        };
        let file_count = get_file_context_actions(&file_info).len();
        let dir_count = get_file_context_actions(&dir_info).len();
        // File has quick_look (macOS), dir doesn't
        assert!(
            file_count > dir_count,
            "file {} > dir {}",
            file_count,
            dir_count
        );
    }

    #[test]
    fn cat18_both_have_reveal_copy_path_copy_filename() {
        let file_info = FileInfo {
            path: "/tmp/a.txt".into(),
            name: "a.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let dir_info = FileInfo {
            path: "/tmp/docs".into(),
            name: "docs".into(),
            file_type: crate::file_search::FileType::Directory,
            is_dir: true,
        };
        for info in [&file_info, &dir_info] {
            let ids = action_ids(&get_file_context_actions(info));
            assert!(ids.contains(&"file:reveal_in_finder".to_string()));
            assert!(ids.contains(&"file:copy_path".to_string()));
            assert!(ids.contains(&"file:copy_filename".to_string()));
        }
    }

    // =========================================================================
    // 19. New chat action ID prefix patterns
    // =========================================================================

    #[test]
    fn cat19_last_used_ids_start_with_last_used() {
        let actions = get_new_chat_actions(
            &[NewChatModelInfo {
                model_id: "m1".into(),
                display_name: "M1".into(),
                provider: "P".into(),
                provider_display_name: "Provider".into(),
            }],
            &[],
            &[],
        );
        assert!(actions[0].id.starts_with("last_used_"));
    }

    #[test]
    fn cat19_preset_ids_start_with_preset() {
        let actions = get_new_chat_actions(
            &[],
            &[NewChatPresetInfo {
                id: "general".into(),
                name: "General".into(),
                icon: IconName::Star,
            }],
            &[],
        );
        assert!(actions[0].id.starts_with("preset_"));
    }

    #[test]
    fn cat19_model_ids_start_with_model() {
        let actions = get_new_chat_actions(
            &[],
            &[],
            &[NewChatModelInfo {
                model_id: "gpt4".into(),
                display_name: "GPT-4".into(),
                provider: "OpenAI".into(),
                provider_display_name: "OpenAI".into(),
            }],
        );
        assert!(actions[0].id.starts_with("model_"));
    }

    #[test]
    fn cat19_empty_inputs_empty_output() {
        let actions = get_new_chat_actions(&[], &[], &[]);
        assert!(actions.is_empty());
    }

    #[test]
    fn cat19_all_three_sections_present() {
        let actions = get_new_chat_actions(
            &[NewChatModelInfo {
                model_id: "m".into(),
                display_name: "M".into(),
                provider: "P".into(),
                provider_display_name: "PP".into(),
            }],
            &[NewChatPresetInfo {
                id: "p".into(),
                name: "P".into(),
                icon: IconName::Star,
            }],
            &[NewChatModelInfo {
                model_id: "x".into(),
                display_name: "X".into(),
                provider: "Q".into(),
                provider_display_name: "QQ".into(),
            }],
        );
        let sections: Vec<_> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        assert!(sections.contains(&"Last Used Settings"));
        assert!(sections.contains(&"Presets"));
        assert!(sections.contains(&"Models"));
    }

    // =========================================================================
    // 20. Notes command bar section-to-action mapping
    // =========================================================================

    #[test]
    fn cat20_full_feature_has_5_sections() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let sections: HashSet<_> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        assert_eq!(sections.len(), 5, "Expected 5 sections: {:?}", sections);
    }

    #[test]
    fn cat20_trash_view_fewer_sections() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let sections: HashSet<_> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        // Trash view hides Edit/Copy/Export, keeps only Notes
        assert!(
            sections.len() < 5,
            "Trash should have fewer sections: {:?}",
            sections
        );
    }


    // --- merged from tests_part_04.rs ---
    #[test]
    fn cat20_no_selection_minimal() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Minimal: new_note, browse_notes, possibly enable_auto_sizing
        assert!(actions.len() <= 3);
    }

    #[test]
    fn cat20_auto_sizing_disabled_adds_settings_action() {
        let disabled = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let enabled = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let d_actions = get_notes_command_bar_actions(&disabled);
        let e_actions = get_notes_command_bar_actions(&enabled);
        assert!(d_actions.iter().any(|a| a.id == "enable_auto_sizing"));
        assert!(!e_actions.iter().any(|a| a.id == "enable_auto_sizing"));
    }

    #[test]
    fn cat20_notes_section_contains_new_note_and_browse() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let notes_section: Vec<_> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Notes"))
            .collect();
        let ids: Vec<_> = notes_section.iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains(&"new_note"));
        assert!(ids.contains(&"browse_notes"));
    }

    // =========================================================================
    // 21. coerce_action_selection edge cases
    // =========================================================================

    #[test]
    fn cat21_empty_rows_returns_none() {
        assert_eq!(coerce_action_selection(&[], 0), None);
    }

    #[test]
    fn cat21_on_item_returns_same() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    }

    #[test]
    fn cat21_header_searches_down() {
        let rows = vec![
            GroupedActionItem::SectionHeader("H".into()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn cat21_trailing_header_searches_up() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("H".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(0));
    }

    #[test]
    fn cat21_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".into()),
            GroupedActionItem::SectionHeader("B".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn cat21_out_of_bounds_clamped() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 100), Some(0));
    }

    // =========================================================================
    // 22. fuzzy_match edge cases
    // =========================================================================

    #[test]
    fn cat22_empty_needle_matches() {
        assert!(ActionsDialog::fuzzy_match("anything", ""));
    }

    #[test]
    fn cat22_empty_haystack_no_match() {
        assert!(!ActionsDialog::fuzzy_match("", "x"));
    }

    #[test]
    fn cat22_both_empty_matches() {
        assert!(ActionsDialog::fuzzy_match("", ""));
    }

    #[test]
    fn cat22_exact_match() {
        assert!(ActionsDialog::fuzzy_match("hello", "hello"));
    }

    #[test]
    fn cat22_subsequence_match() {
        assert!(ActionsDialog::fuzzy_match("hello world", "hlw"));
    }

    #[test]
    fn cat22_no_subsequence() {
        assert!(!ActionsDialog::fuzzy_match("hello", "xyz"));
    }

    #[test]
    fn cat22_needle_longer_than_haystack() {
        assert!(!ActionsDialog::fuzzy_match("hi", "hello"));
    }

    // =========================================================================
    // 23. Script context exact action ordering
    // =========================================================================

    #[test]
    fn cat23_script_run_always_first() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        assert_eq!(actions[0].id, "run_script");
    }

    #[test]
    fn cat23_script_copy_deeplink_always_present() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "copy_deeplink"));
    }

    #[test]
    fn cat23_builtin_run_first() {
        let builtin = ScriptInfo::builtin("Test Builtin");
        let actions = get_script_context_actions(&builtin);
        assert_eq!(actions[0].id, "run_script");
    }

    #[test]
    fn cat23_scriptlet_run_first() {
        let scriptlet = ScriptInfo::scriptlet("Test", "/path.md", None, None);
        let actions = get_script_context_actions(&scriptlet);
        assert_eq!(actions[0].id, "run_script");
    }

    #[test]
    fn cat23_run_action_title_includes_verb_and_name() {
        let script = ScriptInfo::with_action_verb("Safari", "/app", false, "Launch");
        let actions = get_script_context_actions(&script);
        assert_eq!(actions[0].title, "Launch");
    }

    // =========================================================================
    // 24. Clipboard paste title dynamic behavior
    // =========================================================================

    #[test]
    fn cat24_paste_no_app() {
        let entry = make_text_entry();
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = actions.iter().find(|a| a.id == "clip:clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to Active App");
    }

    #[test]
    fn cat24_paste_with_app() {
        let mut entry = make_text_entry();
        entry.frontmost_app_name = Some("Safari".into());
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = actions.iter().find(|a| a.id == "clip:clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to Safari");
    }

    #[test]
    fn cat24_paste_with_unicode_app() {
        let mut entry = make_text_entry();
        entry.frontmost_app_name = Some("日本語App".into());
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = actions.iter().find(|a| a.id == "clip:clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to 日本語App");
    }

    #[test]
    fn cat24_paste_with_empty_app_string() {
        let mut entry = make_text_entry();
        entry.frontmost_app_name = Some("".into());
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = actions.iter().find(|a| a.id == "clip:clipboard_paste").unwrap();
        // Empty string still produces "Paste to " (the code uses the string as-is)
        assert_eq!(paste.title, "Paste to ");
    }

    // =========================================================================
    // 25. Action lowercase caching
    // =========================================================================

    #[test]
    fn cat25_title_lower_computed_on_creation() {
        let action = Action::new("x", "Hello World", None, ActionCategory::ScriptContext);
        assert_eq!(action.title_lower, "hello world");
    }

    #[test]
    fn cat25_description_lower_computed() {
        let action = Action::new(
            "x",
            "X",
            Some("Foo Bar".into()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.description_lower, Some("foo bar".into()));
    }

    #[test]
    fn cat25_description_lower_none_when_no_desc() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext);
        assert!(action.description_lower.is_none());
    }

    #[test]
    fn cat25_shortcut_lower_none_until_with_shortcut() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext);
        assert!(action.shortcut_lower.is_none());
        let action = action.with_shortcut("⌘C");
        assert_eq!(action.shortcut_lower, Some("⌘c".into()));
    }

    #[test]
    fn cat25_with_shortcut_opt_none_no_shortcut_lower() {
        let action =
            Action::new("x", "X", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
        assert!(action.shortcut_lower.is_none());
        assert!(action.shortcut.is_none());
    }

    // =========================================================================
    // 26. Note switcher description rendering
    // =========================================================================

    #[test]
    fn cat26_preview_with_time_uses_separator() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "Note".into(),
            char_count: 50,
            is_current: false,
            is_pinned: false,
            preview: "Some preview".into(),
            relative_time: "5m ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(
            desc.contains(" · "),
            "Expected ' · ' separator in: {}",
            desc
        );
    }

    #[test]
    fn cat26_empty_preview_empty_time_uses_char_count() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "Note".into(),
            char_count: 42,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].description.as_deref(), Some("42 chars"));
    }

    #[test]
    fn cat26_singular_char_count() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "Note".into(),
            char_count: 1,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].description.as_deref(), Some("1 char"));
    }

    #[test]
    fn cat26_zero_char_count() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "Note".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].description.as_deref(), Some("0 chars"));
    }

    #[test]
    fn cat26_preview_truncated_at_61_chars() {
        let long_preview = "a".repeat(61);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "Note".into(),
            char_count: 61,
            is_current: false,
            is_pinned: false,
            preview: long_preview,
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.ends_with('…'), "Should end with … : {}", desc);
    }

    #[test]
    fn cat26_preview_not_truncated_at_60_chars() {
        let preview = "a".repeat(60);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "Note".into(),
            char_count: 60,
            is_current: false,
            is_pinned: false,
            preview,
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(!desc.ends_with('…'), "Should not truncate at 60: {}", desc);
    }

    // =========================================================================
    // 27. Note switcher icon hierarchy
    // =========================================================================

    #[test]
    fn cat27_pinned_icon_star_filled() {
        let notes = vec![make_note("1", "N", true, false)];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    #[test]
    fn cat27_current_icon_check() {
        let notes = vec![make_note("1", "N", false, true)];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::Check));
    }

    #[test]
    fn cat27_regular_icon_file() {
        let notes = vec![make_note("1", "N", false, false)];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::File));
    }

    #[test]
    fn cat27_pinned_overrides_current() {
        let notes = vec![make_note("1", "N", true, true)];
        let actions = get_note_switcher_actions(&notes);
        // Pinned takes priority in the if-else chain
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    // =========================================================================
    // 28. Cross-context ID uniqueness
    // =========================================================================

    #[test]
    fn cat28_script_ids_unique() {
        let actions = get_script_context_actions(&ScriptInfo::new("t", "/t.ts"));
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat28_clipboard_ids_unique() {
        let actions = get_clipboard_history_context_actions(&make_text_entry());
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat28_ai_ids_unique() {
        let actions = get_ai_command_bar_actions();
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat28_path_ids_unique() {
        let info = PathInfo {
            path: "/t".into(),
            name: "t".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat28_file_ids_unique() {
        let info = FileInfo {
            path: "/t".into(),
            name: "t".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len());
    }


    // --- merged from tests_part_05.rs ---
    #[test]
    fn cat28_notes_ids_unique() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len());
    }

    // =========================================================================
    // 29. has_action=false invariant for all built-ins
    // =========================================================================

    #[test]
    fn cat29_script_has_action_false() {
        for action in &get_script_context_actions(&ScriptInfo::new("t", "/t.ts")) {
            assert!(
                !action.has_action,
                "{} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat29_clipboard_has_action_false() {
        for action in &get_clipboard_history_context_actions(&make_text_entry()) {
            assert!(
                !action.has_action,
                "{} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat29_ai_has_action_false() {
        for action in &get_ai_command_bar_actions() {
            assert!(
                !action.has_action,
                "{} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat29_path_has_action_false() {
        let info = PathInfo {
            path: "/t".into(),
            name: "t".into(),
            is_dir: false,
        };
        for action in &get_path_context_actions(&info) {
            assert!(
                !action.has_action,
                "{} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat29_file_has_action_false() {
        let info = FileInfo {
            path: "/t".into(),
            name: "t".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        for action in &get_file_context_actions(&info) {
            assert!(
                !action.has_action,
                "{} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat29_notes_has_action_false() {
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

    // =========================================================================
    // 30. Ordering determinism
    // =========================================================================

    #[test]
    fn cat30_script_ordering_deterministic() {
        let s = ScriptInfo::new("t", "/t.ts");
        let a = action_ids(&get_script_context_actions(&s));
        let b = action_ids(&get_script_context_actions(&s));
        assert_eq!(a, b);
    }

    #[test]
    fn cat30_clipboard_ordering_deterministic() {
        let e = make_text_entry();
        let a = action_ids(&get_clipboard_history_context_actions(&e));
        let b = action_ids(&get_clipboard_history_context_actions(&e));
        assert_eq!(a, b);
    }

    #[test]
    fn cat30_ai_ordering_deterministic() {
        let a = action_ids(&get_ai_command_bar_actions());
        let b = action_ids(&get_ai_command_bar_actions());
        assert_eq!(a, b);
    }

    #[test]
    fn cat30_notes_ordering_deterministic() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let a = action_ids(&get_notes_command_bar_actions(&info));
        let b = action_ids(&get_notes_command_bar_actions(&info));
        assert_eq!(a, b);
    }

    #[test]
    fn cat30_path_ordering_deterministic() {
        let info = PathInfo {
            path: "/t".into(),
            name: "t".into(),
            is_dir: false,
        };
        let a = action_ids(&get_path_context_actions(&info));
        let b = action_ids(&get_path_context_actions(&info));
        assert_eq!(a, b);
    }

}
