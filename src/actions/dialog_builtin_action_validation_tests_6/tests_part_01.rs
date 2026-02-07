    use crate::actions::builders::*;
    use crate::actions::dialog::{
        build_grouped_items_static, coerce_action_selection, ActionsDialog, GroupedActionItem,
    };
    use crate::actions::types::*;
    use crate::actions::CommandBarConfig;
    use crate::clipboard_history::ContentType;
    use crate::designs::icon_variations::IconName;
    use crate::file_search::{FileInfo, FileType};
    use crate::prompts::PathInfo;
    use crate::scriptlets::{Scriptlet, ScriptletAction};

    // =========================================================================
    // Helper: collect action IDs from a Vec<Action>
    // =========================================================================
    fn action_ids(actions: &[Action]) -> Vec<&str> {
        actions.iter().map(|a| a.id.as_str()).collect()
    }

    fn action_titles(actions: &[Action]) -> Vec<&str> {
        actions.iter().map(|a| a.title.as_str()).collect()
    }

    // =========================================================================
    // 1. ScriptInfo impossible flag combinations
    //    The constructors don't prevent these, so verify behavior is reasonable.
    // =========================================================================

    #[test]
    fn script_and_scriptlet_flags_both_true_gets_both_action_sets() {
        // Manually create a ScriptInfo with both is_script and is_scriptlet true.
        // The builder won't do this, but it's worth validating behavior.
        let mut info = ScriptInfo::new("Hybrid", "/path/hybrid.ts");
        info.is_scriptlet = true;
        // is_script=true AND is_scriptlet=true
        let actions = get_script_context_actions(&info);
        let ids = action_ids(&actions);
        // Should have both script actions AND scriptlet actions
        assert!(
            ids.contains(&"edit_script"),
            "has edit_script from is_script"
        );
        assert!(
            ids.contains(&"edit_scriptlet"),
            "has edit_scriptlet from is_scriptlet"
        );
        assert!(
            ids.contains(&"view_logs"),
            "has view_logs from is_script block"
        );
    }

    #[test]
    fn script_and_agent_flags_both_true_gets_both_action_sets() {
        let mut info = ScriptInfo::new("HybridAgent", "/path/agent.ts");
        info.is_agent = true;
        // is_script=true AND is_agent=true
        let actions = get_script_context_actions(&info);
        let _ids = action_ids(&actions);
        // Both script edit and agent edit should appear (agent uses same "edit_script" ID)
        let edit_count = actions.iter().filter(|a| a.id == "edit_script").count();
        assert!(
            edit_count >= 2,
            "Both script and agent blocks add edit_script: count={}",
            edit_count
        );
    }

    #[test]
    fn agent_without_is_script_has_no_view_logs() {
        let mut info = ScriptInfo::new("PureAgent", "/path/agent.md");
        info.is_script = false;
        info.is_agent = true;
        let actions = get_script_context_actions(&info);
        let ids = action_ids(&actions);
        assert!(
            !ids.contains(&"view_logs"),
            "Agent without is_script should lack view_logs"
        );
        // But should have agent edit
        let edit_action = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit_action.title, "Edit Agent");
    }

    #[test]
    fn all_three_flags_true_produces_actions_from_all_three_blocks() {
        let mut info = ScriptInfo::new("Triple", "/path/triple.ts");
        info.is_scriptlet = true;
        info.is_agent = true;
        let actions = get_script_context_actions(&info);
        let ids = action_ids(&actions);
        // Should have actions from all three blocks
        assert!(ids.contains(&"view_logs"), "script block: view_logs");
        assert!(
            ids.contains(&"edit_scriptlet"),
            "scriptlet block: edit_scriptlet"
        );
        assert!(
            ids.contains(&"reveal_in_finder"),
            "agent block: reveal_in_finder"
        );
    }

    // =========================================================================
    // 2. Action verb propagation in primary action title
    // =========================================================================

    #[test]
    fn action_verb_run_in_script_title() {
        let info = ScriptInfo::new("My Script", "/path/script.ts");
        let actions = get_script_context_actions(&info);
        assert_eq!(actions[0].title, "Run \"My Script\"");
    }

    #[test]
    fn action_verb_launch_in_app_title() {
        let info =
            ScriptInfo::with_action_verb("Safari", "/Applications/Safari.app", false, "Launch");
        let actions = get_script_context_actions(&info);
        assert_eq!(actions[0].title, "Launch \"Safari\"");
    }

    #[test]
    fn action_verb_switch_to_in_window_title() {
        let info = ScriptInfo::with_action_verb("My Document", "window:123", false, "Switch to");
        let actions = get_script_context_actions(&info);
        assert_eq!(actions[0].title, "Switch to \"My Document\"");
    }

    #[test]
    fn action_verb_execute_custom_in_title() {
        let info = ScriptInfo::with_action_verb("Task", "/path/task.ts", true, "Execute");
        let actions = get_script_context_actions(&info);
        assert_eq!(actions[0].title, "Execute \"Task\"");
    }

    #[test]
    fn action_verb_open_in_builtin_title() {
        let info =
            ScriptInfo::with_action_verb("Clipboard History", "builtin:clipboard", false, "Open");
        let actions = get_script_context_actions(&info);
        assert_eq!(actions[0].title, "Open \"Clipboard History\"");
    }

    #[test]
    fn action_verb_description_matches_verb() {
        let info = ScriptInfo::with_action_verb("Test", "/path/test.ts", true, "Execute");
        let actions = get_script_context_actions(&info);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "Execute this item");
    }

    #[test]
    fn scriptlet_context_action_verb_propagation() {
        let info = ScriptInfo::scriptlet("My Snippet", "/path/snippet.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        assert_eq!(actions[0].title, "Run \"My Snippet\"");
        assert_eq!(actions[0].description.as_ref().unwrap(), "Run this item");
    }

    // =========================================================================
    // 3. Deeplink description URL format validation
    // =========================================================================

    #[test]
    fn deeplink_description_contains_url_for_simple_name() {
        let info = ScriptInfo::new("Hello World", "/path/hello.ts");
        let actions = get_script_context_actions(&info);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        let desc = dl.description.as_ref().unwrap();
        assert!(desc.contains("scriptkit://run/hello-world"));
    }

    #[test]
    fn deeplink_description_contains_url_for_special_chars() {
        let info = ScriptInfo::builtin("Open !@# File");
        let actions = get_script_context_actions(&info);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        let desc = dl.description.as_ref().unwrap();
        assert!(desc.contains("scriptkit://run/open-file"));
    }

    #[test]
    fn deeplink_description_contains_url_for_underscores() {
        let info = ScriptInfo::new("hello_world_test", "/path/test.ts");
        let actions = get_script_context_actions(&info);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        let desc = dl.description.as_ref().unwrap();
        assert!(desc.contains("scriptkit://run/hello-world-test"));
    }

    #[test]
    fn deeplink_description_for_scriptlet_context() {
        let info = ScriptInfo::scriptlet("Open URL", "/path/urls.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        let desc = dl.description.as_ref().unwrap();
        assert!(desc.contains("scriptkit://run/open-url"));
    }

    // =========================================================================
    // 4. Clipboard entry edge cases
    // =========================================================================

    #[test]
    fn clipboard_empty_preview_text_entry() {
        let entry = ClipboardEntryInfo {
            id: "e1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        // Should still produce valid actions even with empty preview
        assert!(!actions.is_empty());
        assert!(actions.iter().any(|a| a.id == "clipboard_paste"));
    }

    #[test]
    fn clipboard_very_long_app_name_in_paste_title() {
        let entry = ClipboardEntryInfo {
            id: "e2".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: Some("Super Long Application Name That Goes On And On".to_string()),
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
        assert_eq!(
            paste.title,
            "Paste to Super Long Application Name That Goes On And On"
        );
    }

    #[test]
    fn clipboard_image_pinned_has_unpin_and_ocr() {
        let entry = ClipboardEntryInfo {
            id: "img1".into(),
            content_type: ContentType::Image,
            pinned: true,
            preview: "Image".into(),
            image_dimensions: Some((1920, 1080)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"clipboard_unpin"), "pinned image has unpin");
        assert!(!ids.contains(&"clipboard_pin"), "pinned image has no pin");
        assert!(ids.contains(&"clipboard_ocr"), "image has OCR");
    }

    #[test]
    fn clipboard_text_pinned_has_no_ocr() {
        let entry = ClipboardEntryInfo {
            id: "txt1".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "Hello".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        assert!(!ids.contains(&"clipboard_ocr"), "text has no OCR");
        assert!(ids.contains(&"clipboard_unpin"), "pinned text has unpin");
    }

    #[test]
    fn clipboard_text_action_order_first_three() {
        let entry = ClipboardEntryInfo {
            id: "ord1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[0].id, "clipboard_paste", "1st: paste");
        assert_eq!(actions[1].id, "clipboard_copy", "2nd: copy");
        assert_eq!(
            actions[2].id, "clipboard_paste_keep_open",
            "3rd: paste_keep_open"
        );
    }

    #[test]
    fn clipboard_destructive_actions_are_last_three() {
        let entry = ClipboardEntryInfo {
            id: "del1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let len = actions.len();
        assert_eq!(actions[len - 3].id, "clipboard_delete");
        assert_eq!(actions[len - 2].id, "clipboard_delete_multiple");
        assert_eq!(actions[len - 1].id, "clipboard_delete_all");
    }

    // =========================================================================
    // 5. Chat context scaling and edge cases
    // =========================================================================

    #[test]
    fn chat_context_many_models_all_get_select_prefix() {
        let models: Vec<ChatModelInfo> = (0..20)
            .map(|i| ChatModelInfo {
                id: format!("model-{}", i),
                display_name: format!("Model {}", i),
                provider: "TestProvider".into(),
            })
            .collect();
        let info = ChatPromptInfo {
            current_model: None,
            available_models: models,
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        // 20 models + continue_in_chat = 21
        assert_eq!(actions.len(), 21);
        for action in actions.iter().take(20) {
            assert!(
                action.id.starts_with("select_model_"),
                "Model action ID should start with select_model_: {}",
                action.id
            );
        }
    }

    #[test]
    fn chat_context_duplicate_provider_names_in_descriptions() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![
                ChatModelInfo {
                    id: "gpt4".into(),
                    display_name: "GPT-4".into(),
                    provider: "OpenAI".into(),
                },
                ChatModelInfo {
                    id: "gpt3".into(),
                    display_name: "GPT-3.5".into(),
                    provider: "OpenAI".into(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        // Both should have "via OpenAI" description
        assert_eq!(actions[0].description.as_ref().unwrap(), "via OpenAI");
        assert_eq!(actions[1].description.as_ref().unwrap(), "via OpenAI");
    }

    #[test]
    fn chat_context_current_model_checkmark_only_on_matching() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".into()),
            available_models: vec![
                ChatModelInfo {
                    id: "gpt4".into(),
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
        assert!(
            actions[0].title.contains("✓"),
            "Current model should have checkmark"
        );
        assert!(
            !actions[1].title.contains("✓"),
            "Non-current model should not have checkmark"
        );
    }

    #[test]
    fn chat_context_no_models_only_continue() {
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
    fn chat_context_all_flags_true_has_all_actions() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".into()),
            available_models: vec![ChatModelInfo {
                id: "gpt4".into(),
                display_name: "GPT-4".into(),
                provider: "OpenAI".into(),
            }],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"continue_in_chat"));
        assert!(ids.contains(&"copy_response"));
        assert!(ids.contains(&"clear_conversation"));
        assert_eq!(actions.len(), 4); // 1 model + 3 actions
    }

    #[test]
    fn chat_context_has_response_no_messages() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"copy_response"));
        assert!(!ids.contains(&"clear_conversation"));
    }

