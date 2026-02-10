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
    // Category 01: Agent context is_agent via mutation — action set correctness
    // =========================================================================

    #[test]
    fn cat01_agent_via_mutation_has_edit_script() {
        let mut script = ScriptInfo::new("My Agent", "/path/to/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    #[test]
    fn cat01_agent_via_mutation_has_reveal_in_finder() {
        let mut script = ScriptInfo::new("My Agent", "/path/to/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
    }

    #[test]
    fn cat01_agent_via_mutation_has_copy_path() {
        let mut script = ScriptInfo::new("My Agent", "/path/to/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "copy_path"));
    }

    #[test]
    fn cat01_agent_via_mutation_has_copy_content() {
        let mut script = ScriptInfo::new("My Agent", "/path/to/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "copy_content"));
    }

    #[test]
    fn cat01_agent_via_mutation_no_view_logs() {
        let mut script = ScriptInfo::new("My Agent", "/path/to/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    #[test]
    fn cat01_agent_edit_description_mentions_agent() {
        let mut script = ScriptInfo::new("My Agent", "/path/to/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert!(edit.description.as_ref().unwrap().contains("agent"));
    }

    // =========================================================================
    // Category 02: Scriptlet context with_custom — reset_ranking placement
    // =========================================================================

    #[test]
    fn cat02_scriptlet_custom_with_frecency_has_reset_ranking() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None)
            .with_frecency(true, Some("scriptlet:Test".to_string()));
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    }

    #[test]
    fn cat02_scriptlet_custom_reset_ranking_is_last() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None)
            .with_frecency(true, Some("scriptlet:Test".to_string()));
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let last = actions.last().unwrap();
        assert_eq!(last.id, "reset_ranking");
    }

    #[test]
    fn cat02_scriptlet_custom_no_frecency_no_reset() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(!actions.iter().any(|a| a.id == "reset_ranking"));
    }

    #[test]
    fn cat02_scriptlet_custom_with_custom_actions_and_frecency() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None)
            .with_frecency(true, Some("scriptlet:Test".to_string()));
        let mut scriptlet =
            Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());
        scriptlet.actions = vec![ScriptletAction {
            name: "Custom".to_string(),
            command: "custom".to_string(),
            tool: "bash".to_string(),
            code: "echo custom".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        assert!(actions.iter().any(|a| a.id == "scriptlet_action:custom"));
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    }

    // =========================================================================
    // Category 03: Clipboard OCR action details
    // =========================================================================

    #[test]
    fn cat03_ocr_shortcut_is_shift_cmd_c() {
        let entry = ClipboardEntryInfo {
            id: "img1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "Image".to_string(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ocr = actions.iter().find(|a| a.id == "clipboard_ocr").unwrap();
        assert_eq!(ocr.shortcut.as_ref().unwrap(), "⇧⌘C");
    }

    #[test]
    fn cat03_ocr_title_is_copy_text_from_image() {
        let entry = ClipboardEntryInfo {
            id: "img1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "Image".to_string(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ocr = actions.iter().find(|a| a.id == "clipboard_ocr").unwrap();
        assert_eq!(ocr.title, "Copy Text from Image");
    }

    #[test]
    fn cat03_ocr_description_mentions_ocr() {
        let entry = ClipboardEntryInfo {
            id: "img1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "Image".to_string(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ocr = actions.iter().find(|a| a.id == "clipboard_ocr").unwrap();
        assert!(ocr.description.as_ref().unwrap().contains("OCR"));
    }

    #[test]
    fn cat03_text_entry_has_no_ocr() {
        let entry = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(!actions.iter().any(|a| a.id == "clipboard_ocr"));
    }

    // =========================================================================
    // Category 04: Clipboard CleanShot actions (macOS only)
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn cat04_annotate_cleanshot_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "img1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "Image".to_string(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let annotate = actions
            .iter()
            .find(|a| a.id == "clipboard_annotate_cleanshot")
            .unwrap();
        assert_eq!(annotate.shortcut.as_ref().unwrap(), "⇧⌘A");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat04_upload_cleanshot_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "img1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "Image".to_string(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let upload = actions
            .iter()
            .find(|a| a.id == "clipboard_upload_cleanshot")
            .unwrap();
        assert_eq!(upload.shortcut.as_ref().unwrap(), "⇧⌘U");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat04_text_entry_no_cleanshot_actions() {
        let entry = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(!actions
            .iter()
            .any(|a| a.id == "clipboard_annotate_cleanshot"));
        assert!(!actions.iter().any(|a| a.id == "clipboard_upload_cleanshot"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat04_image_has_open_with_text_does_not() {
        let img = ClipboardEntryInfo {
            id: "img1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "Image".to_string(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let txt = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let img_actions = get_clipboard_history_context_actions(&img);
        let txt_actions = get_clipboard_history_context_actions(&txt);
        assert!(img_actions.iter().any(|a| a.id == "clipboard_open_with"));
        assert!(!txt_actions.iter().any(|a| a.id == "clipboard_open_with"));
    }

    // =========================================================================
    // Category 05: Chat context both flags false — minimal action set
    // =========================================================================

    #[test]
    fn cat05_both_flags_false_zero_models_only_continue() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "continue_in_chat");
    }

    #[test]
    fn cat05_both_flags_true_adds_copy_and_clear() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions.len(), 3);
        assert!(actions.iter().any(|a| a.id == "continue_in_chat"));
        assert!(actions.iter().any(|a| a.id == "copy_response"));
        assert!(actions.iter().any(|a| a.id == "clear_conversation"));
    }

    #[test]
    fn cat05_has_response_only_adds_copy_response() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions.len(), 2);
        assert!(actions.iter().any(|a| a.id == "copy_response"));
        assert!(!actions.iter().any(|a| a.id == "clear_conversation"));
    }

    #[test]
    fn cat05_has_messages_only_adds_clear() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions.len(), 2);
        assert!(actions.iter().any(|a| a.id == "clear_conversation"));
        assert!(!actions.iter().any(|a| a.id == "copy_response"));
    }

    // =========================================================================
    // Category 06: Chat context model checkmark matching
    // =========================================================================

    #[test]
    fn cat06_current_model_gets_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("Claude 3.5 Sonnet".to_string()),
            available_models: vec![ChatModelInfo {
                id: "claude-3-5-sonnet".to_string(),
                display_name: "Claude 3.5 Sonnet".to_string(),
                provider: "Anthropic".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = actions
            .iter()
            .find(|a| a.id == "select_model_claude-3-5-sonnet")
            .unwrap();
        assert!(model_action.title.contains('✓'));
    }

    #[test]
    fn cat06_non_current_model_no_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".to_string()),
            available_models: vec![ChatModelInfo {
                id: "claude-3-5-sonnet".to_string(),
                display_name: "Claude 3.5 Sonnet".to_string(),
                provider: "Anthropic".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = actions
            .iter()
            .find(|a| a.id == "select_model_claude-3-5-sonnet")
            .unwrap();
        assert!(!model_action.title.contains('✓'));
    }

    #[test]
    fn cat06_model_description_has_via_provider() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "gpt-4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = actions
            .iter()
            .find(|a| a.id == "select_model_gpt-4")
            .unwrap();
        assert_eq!(model_action.description.as_ref().unwrap(), "via OpenAI");
    }

    #[test]
    fn cat06_multiple_models_ordering_preserved() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![
                ChatModelInfo {
                    id: "m1".to_string(),
                    display_name: "Model A".to_string(),
                    provider: "P1".to_string(),
                },
                ChatModelInfo {
                    id: "m2".to_string(),
                    display_name: "Model B".to_string(),
                    provider: "P2".to_string(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let m1_idx = actions
            .iter()
            .position(|a| a.id == "select_model_m1")
            .unwrap();
        let m2_idx = actions
            .iter()
            .position(|a| a.id == "select_model_m2")
            .unwrap();
        assert!(m1_idx < m2_idx);
    }

    // =========================================================================
    // Category 07: New chat action provider_display_name propagation
    // =========================================================================

    #[test]
    fn cat07_last_used_description_uses_provider_display_name() {
        let last_used = vec![NewChatModelInfo {
            model_id: "claude-3".to_string(),
            display_name: "Claude 3".to_string(),
            provider: "anthropic".to_string(),
            provider_display_name: "Anthropic".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        let lu = actions.iter().find(|a| a.id == "last_used_0").unwrap();
        assert_eq!(lu.description.as_ref().unwrap(), "Anthropic");
    }

