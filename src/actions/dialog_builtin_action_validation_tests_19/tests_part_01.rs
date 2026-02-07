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
    use std::collections::HashSet;

    fn action_ids(actions: &[Action]) -> Vec<String> {
        actions.iter().map(|a| a.id.clone()).collect()
    }

    // =========================================================================
    // Category 01: Scriptlet context shortcut/alias toggle symmetry
    // Verifies that scriptlet context actions correctly toggle add vs
    // update/remove based on shortcut and alias presence, mirroring script context.
    // =========================================================================

    #[test]
    fn cat01_scriptlet_no_shortcut_has_add_shortcut() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(actions.iter().any(|a| a.id == "add_shortcut"));
        assert!(!actions.iter().any(|a| a.id == "update_shortcut"));
    }

    #[test]
    fn cat01_scriptlet_with_shortcut_has_update_remove() {
        let script =
            ScriptInfo::scriptlet("Test", "/path/test.md", Some("cmd+t".to_string()), None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(!actions.iter().any(|a| a.id == "add_shortcut"));
        assert!(actions.iter().any(|a| a.id == "update_shortcut"));
        assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
    }

    #[test]
    fn cat01_scriptlet_no_alias_has_add_alias() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(actions.iter().any(|a| a.id == "add_alias"));
        assert!(!actions.iter().any(|a| a.id == "update_alias"));
    }

    #[test]
    fn cat01_scriptlet_with_alias_has_update_remove_alias() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, Some("ts".to_string()));
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(!actions.iter().any(|a| a.id == "add_alias"));
        assert!(actions.iter().any(|a| a.id == "update_alias"));
        assert!(actions.iter().any(|a| a.id == "remove_alias"));
    }

    #[test]
    fn cat01_scriptlet_both_shortcut_and_alias_set() {
        let script = ScriptInfo::scriptlet(
            "Test",
            "/path/test.md",
            Some("cmd+t".to_string()),
            Some("ts".to_string()),
        );
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(actions.iter().any(|a| a.id == "update_shortcut"));
        assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
        assert!(actions.iter().any(|a| a.id == "update_alias"));
        assert!(actions.iter().any(|a| a.id == "remove_alias"));
    }

    // =========================================================================
    // Category 02: Script context deeplink description contains URL
    // Verifies the copy_deeplink action description format across script types.
    // =========================================================================

    #[test]
    fn cat02_script_deeplink_desc_contains_url_pattern() {
        let script = ScriptInfo::new("My Cool Script", "/path/script.ts");
        let actions = get_script_context_actions(&script);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert!(dl
            .description
            .as_ref()
            .unwrap()
            .contains("scriptkit://run/"));
    }

    #[test]
    fn cat02_script_deeplink_desc_contains_deeplink_name() {
        let script = ScriptInfo::new("My Cool Script", "/path/script.ts");
        let actions = get_script_context_actions(&script);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert!(dl.description.as_ref().unwrap().contains("my-cool-script"));
    }

    #[test]
    fn cat02_builtin_deeplink_desc_contains_url() {
        let script = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&script);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert!(dl
            .description
            .as_ref()
            .unwrap()
            .contains("clipboard-history"));
    }

    #[test]
    fn cat02_scriptlet_context_deeplink_desc_contains_url() {
        let script = ScriptInfo::scriptlet("Open GitHub", "/path/urls.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert!(dl.description.as_ref().unwrap().contains("open-github"));
    }

    // =========================================================================
    // Category 03: Clipboard frontmost_app_name dynamic paste title
    // Tests that paste action title changes based on the frontmost app.
    // =========================================================================

    #[test]
    fn cat03_clipboard_paste_title_no_app() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to Active App");
    }

    #[test]
    fn cat03_clipboard_paste_title_with_app() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: Some("Safari".to_string()),
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to Safari");
    }

    #[test]
    fn cat03_clipboard_paste_title_unicode_app() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: Some("日本語エディタ".to_string()),
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to 日本語エディタ");
    }

    #[test]
    fn cat03_clipboard_paste_title_empty_app_string() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: Some("".to_string()),
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
        // Empty string still formats as "Paste to "
        assert_eq!(paste.title, "Paste to ");
    }

    // =========================================================================
    // Category 04: Notes command bar action presence per flag combination
    // Explores specific actions' conditional visibility across flag combos.
    // =========================================================================

    #[test]
    fn cat04_notes_new_note_always_present() {
        for (sel, trash, auto) in [
            (false, false, false),
            (true, true, true),
            (false, true, false),
        ] {
            let info = NotesInfo {
                has_selection: sel,
                is_trash_view: trash,
                auto_sizing_enabled: auto,
            };
            let actions = get_notes_command_bar_actions(&info);
            assert!(
                actions.iter().any(|a| a.id == "new_note"),
                "new_note absent with sel={sel} trash={trash} auto={auto}"
            );
        }
    }

    #[test]
    fn cat04_notes_browse_notes_always_present() {
        for (sel, trash, auto) in [
            (false, false, false),
            (true, true, true),
            (true, false, true),
        ] {
            let info = NotesInfo {
                has_selection: sel,
                is_trash_view: trash,
                auto_sizing_enabled: auto,
            };
            let actions = get_notes_command_bar_actions(&info);
            assert!(
                actions.iter().any(|a| a.id == "browse_notes"),
                "browse_notes absent with sel={sel} trash={trash} auto={auto}"
            );
        }
    }

    #[test]
    fn cat04_notes_enable_auto_sizing_conditional() {
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
        let actions_disabled = get_notes_command_bar_actions(&disabled);
        let actions_enabled = get_notes_command_bar_actions(&enabled);
        assert!(actions_disabled
            .iter()
            .any(|a| a.id == "enable_auto_sizing"));
        assert!(!actions_enabled.iter().any(|a| a.id == "enable_auto_sizing"));
    }

    #[test]
    fn cat04_notes_copy_section_requires_selection_and_no_trash() {
        let valid = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let no_sel = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let trash = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        assert!(get_notes_command_bar_actions(&valid)
            .iter()
            .any(|a| a.id == "copy_note_as"));
        assert!(!get_notes_command_bar_actions(&no_sel)
            .iter()
            .any(|a| a.id == "copy_note_as"));
        assert!(!get_notes_command_bar_actions(&trash)
            .iter()
            .any(|a| a.id == "copy_note_as"));
    }

    #[test]
    fn cat04_notes_create_quicklink_requires_selection_no_trash() {
        let valid = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let invalid = NotesInfo {
            has_selection: false,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        assert!(get_notes_command_bar_actions(&valid)
            .iter()
            .any(|a| a.id == "create_quicklink"));
        assert!(!get_notes_command_bar_actions(&invalid)
            .iter()
            .any(|a| a.id == "create_quicklink"));
    }

    // =========================================================================
    // Category 05: Chat context action count boundary states
    // Validates exact action count under different ChatPromptInfo combos.
    // =========================================================================

    #[test]
    fn cat05_chat_zero_models_no_flags_one_action() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions.len(), 1); // only continue_in_chat
        assert_eq!(actions[0].id, "continue_in_chat");
    }

    #[test]
    fn cat05_chat_zero_models_both_flags_three_actions() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions.len(), 3);
    }

    #[test]
    fn cat05_chat_two_models_no_flags_three_actions() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".to_string()),
            available_models: vec![
                ChatModelInfo {
                    id: "gpt-4".to_string(),
                    display_name: "GPT-4".to_string(),
                    provider: "OpenAI".to_string(),
                },
                ChatModelInfo {
                    id: "claude-3".to_string(),
                    display_name: "Claude 3".to_string(),
                    provider: "Anthropic".to_string(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions.len(), 3); // 2 models + continue_in_chat
    }

    #[test]
    fn cat05_chat_two_models_both_flags_five_actions() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".to_string()),
            available_models: vec![
                ChatModelInfo {
                    id: "gpt-4".to_string(),
                    display_name: "GPT-4".to_string(),
                    provider: "OpenAI".to_string(),
                },
                ChatModelInfo {
                    id: "claude-3".to_string(),
                    display_name: "Claude 3".to_string(),
                    provider: "Anthropic".to_string(),
                },
            ],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions.len(), 5); // 2 models + continue + copy + clear
    }

    // =========================================================================
    // Category 06: New chat actions section assignment correctness
    // Validates that each action lands in the right section.
    // =========================================================================

    #[test]
    fn cat06_new_chat_last_used_section() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".to_string(),
            display_name: "Model 1".to_string(),
            provider: "P1".to_string(),
            provider_display_name: "Provider 1".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
    }

    #[test]
    fn cat06_new_chat_preset_section() {
        let presets = vec![NewChatPresetInfo {
            id: "general".to_string(),
            name: "General".to_string(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].section.as_deref(), Some("Presets"));
    }

    #[test]
    fn cat06_new_chat_model_section() {
        let models = vec![NewChatModelInfo {
            model_id: "m1".to_string(),
            display_name: "Model 1".to_string(),
            provider: "P1".to_string(),
            provider_display_name: "Provider 1".to_string(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].section.as_deref(), Some("Models"));
    }

    #[test]
    fn cat06_new_chat_mixed_sections_correct_order() {
        let last_used = vec![NewChatModelInfo {
            model_id: "lu".to_string(),
            display_name: "LU".to_string(),
            provider: "P".to_string(),
            provider_display_name: "Provider".to_string(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "p1".to_string(),
            name: "Preset".to_string(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m1".to_string(),
            display_name: "Model".to_string(),
            provider: "P".to_string(),
            provider_display_name: "Provider".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        assert_eq!(actions.len(), 3);
        assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
        assert_eq!(actions[1].section.as_deref(), Some("Presets"));
        assert_eq!(actions[2].section.as_deref(), Some("Models"));
    }

    #[test]
    fn cat06_new_chat_empty_all_returns_empty() {
        let actions = get_new_chat_actions(&[], &[], &[]);
        assert!(actions.is_empty());
    }

    // =========================================================================
    // Category 07: Note switcher description edge cases with preview+time
    // Tests various combinations of preview and relative_time.
    // =========================================================================

