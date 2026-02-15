//! Batch 6: Built-in action behavioral validation tests
//!
//! 160+ tests validating action invariants NOT covered in batches 1-5.
//! Focus areas:
//! - ScriptInfo impossible flag combinations (is_script+is_scriptlet, is_script+is_agent)
//! - Action verb propagation across all contexts
//! - Deeplink description URL format for various name patterns
//! - Clipboard entry edge cases (empty preview, long preview, special app names)
//! - Chat context scaling (many models, duplicate providers, empty display names)
//! - Notes info systematic boolean combos (all 8 permutations w/ section labels)
//! - Note switcher mixed pinned/unpinned ordering and sections
//! - New chat with partial sections (empty presets, empty models, etc.)
//! - Combined score stacking (title+desc+shortcut all matching)
//! - build_grouped_items_static consecutive same-section (no duplicate headers)
//! - coerce_action_selection all-headers edge case
//! - format_shortcut_hint (ActionsDialog version) comprehensive coverage
//! - Path context long names and special chars
//! - File context all FileType variants
//! - Action builder chaining immutability
//! - CommandBarConfig default field values
//! - Scriptlet context action count vs script context action count comparison
//! - Agent ScriptInfo with full flag set (shortcut+alias+frecency)

#[cfg(test)]
mod tests {
    // --- merged from tests_part_01.rs ---
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
        // Duplicate IDs from script+agent are deduplicated in the action builder.
        let edit_count = actions.iter().filter(|a| a.id == "edit_script").count();
        assert_eq!(edit_count, 1, "edit_script is deduplicated by ID");
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
            ids.contains(&"file:reveal_in_finder"),
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
        assert!(actions.iter().any(|a| a.id == "clip:clipboard_paste"));
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
        let paste = actions.iter().find(|a| a.id == "clip:clipboard_paste").unwrap();
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
        assert!(ids.contains(&"clip:clipboard_unpin"), "pinned image has unpin");
        assert!(!ids.contains(&"clip:clipboard_pin"), "pinned image has no pin");
        assert!(ids.contains(&"clip:clipboard_ocr"), "image has OCR");
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
        assert!(!ids.contains(&"clip:clipboard_ocr"), "text has no OCR");
        assert!(ids.contains(&"clip:clipboard_unpin"), "pinned text has unpin");
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
        assert_eq!(actions[0].id, "clip:clipboard_paste", "1st: paste");
        assert_eq!(actions[1].id, "clip:clipboard_copy", "2nd: copy");
        assert_eq!(
            actions[2].id, "clip:clipboard_paste_keep_open",
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
        assert_eq!(actions[len - 3].id, "clip:clipboard_delete");
        assert_eq!(actions[len - 2].id, "clip:clipboard_delete_multiple");
        assert_eq!(actions[len - 1].id, "clip:clipboard_delete_all");
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
                action.id.starts_with("chat:select_model_"),
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
        assert_eq!(actions[0].id, "chat:continue_in_chat");
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
        assert!(ids.contains(&"chat:continue_in_chat"));
        assert!(ids.contains(&"chat:copy_response"));
        assert!(ids.contains(&"chat:clear_conversation"));
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
        assert!(ids.contains(&"chat:copy_response"));
        assert!(!ids.contains(&"chat:clear_conversation"));
    }

    // --- merged from tests_part_02.rs ---
    #[test]
    fn chat_context_has_messages_no_response() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let ids = action_ids(&actions);
        assert!(!ids.contains(&"chat:copy_response"));
        assert!(ids.contains(&"chat:clear_conversation"));
    }

    // =========================================================================
    // 6. Notes info systematic boolean combos with section labels
    // =========================================================================

    #[test]
    fn notes_all_false_has_new_note_browse_and_auto_sizing() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"new_note"));
        assert!(ids.contains(&"browse_notes"));
        assert!(ids.contains(&"enable_auto_sizing"));
        assert!(!ids.contains(&"duplicate_note"));
        assert!(!ids.contains(&"find_in_note"));
    }

    #[test]
    fn notes_selection_no_trash_no_auto_has_full_set() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"new_note"));
        assert!(ids.contains(&"duplicate_note"));
        assert!(ids.contains(&"browse_notes"));
        assert!(ids.contains(&"find_in_note"));
        assert!(ids.contains(&"format"));
        assert!(ids.contains(&"copy_note_as"));
        assert!(ids.contains(&"copy_deeplink"));
        assert!(ids.contains(&"create_quicklink"));
        assert!(ids.contains(&"export"));
        assert!(ids.contains(&"enable_auto_sizing"));
    }

    #[test]
    fn notes_selection_no_trash_auto_enabled_hides_auto_sizing() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        assert!(!ids.contains(&"enable_auto_sizing"));
        // Everything else present
        assert!(ids.contains(&"duplicate_note"));
        assert!(ids.contains(&"export"));
    }

    #[test]
    fn notes_selection_trash_hides_conditional_actions() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        // Trash view hides selection-dependent actions
        assert!(!ids.contains(&"duplicate_note"));
        assert!(!ids.contains(&"find_in_note"));
        assert!(!ids.contains(&"format"));
        assert!(!ids.contains(&"copy_note_as"));
        assert!(!ids.contains(&"export"));
        // These are always present
        assert!(ids.contains(&"new_note"));
        assert!(ids.contains(&"browse_notes"));
    }

    #[test]
    fn notes_no_selection_trash_minimal_actions() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Only new_note and browse_notes (auto_sizing_enabled=true hides that)
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].id, "new_note");
        assert_eq!(actions[1].id, "browse_notes");
    }

    #[test]
    fn notes_section_labels_present_for_full_set() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Verify section labels
        let sections: Vec<&str> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        assert!(sections.contains(&"Notes"));
        assert!(sections.contains(&"Edit"));
        assert!(sections.contains(&"Copy"));
        assert!(sections.contains(&"Export"));
        assert!(sections.contains(&"Settings"));
    }

    #[test]
    fn notes_icons_present_for_all_actions() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for action in &actions {
            assert!(
                action.icon.is_some(),
                "Action '{}' should have an icon",
                action.id
            );
        }
    }

    // =========================================================================
    // 7. Note switcher mixed pinned/unpinned section assignment
    // =========================================================================

    #[test]
    fn note_switcher_pinned_notes_in_pinned_section() {
        let notes = vec![
            NoteSwitcherNoteInfo {
                id: "p1".into(),
                title: "Pinned Note".into(),
                char_count: 50,
                is_current: false,
                is_pinned: true,
                preview: "pinned content".into(),
                relative_time: "1h ago".into(),
            },
            NoteSwitcherNoteInfo {
                id: "r1".into(),
                title: "Recent Note".into(),
                char_count: 30,
                is_current: false,
                is_pinned: false,
                preview: "recent content".into(),
                relative_time: "5m ago".into(),
            },
        ];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
        assert_eq!(actions[1].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn note_switcher_current_pinned_gets_star_icon() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "cp".into(),
            title: "Current Pinned".into(),
            char_count: 10,
            is_current: true,
            is_pinned: true,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        // Pinned takes precedence over current for icon
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
        // But current still gets bullet prefix
        assert!(actions[0].title.starts_with("• "));
    }

    #[test]
    fn note_switcher_current_not_pinned_gets_check_icon() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "cn".into(),
            title: "Current Note".into(),
            char_count: 10,
            is_current: true,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::Check));
    }

    #[test]
    fn note_switcher_regular_note_gets_file_icon() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "rn".into(),
            title: "Regular Note".into(),
            char_count: 10,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::File));
    }

    #[test]
    fn note_switcher_id_format_is_note_prefix() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc-123".into(),
            title: "Test".into(),
            char_count: 5,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].id, "note_abc-123");
    }

    #[test]
    fn note_switcher_empty_shows_no_notes_message() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "no_notes");
        assert_eq!(actions[0].title, "No notes yet");
        assert_eq!(actions[0].section.as_deref(), Some("Notes"));
    }

    #[test]
    fn note_switcher_char_count_singular() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "s1".into(),
            title: "One Char".into(),
            char_count: 1,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "1 char");
    }

    #[test]
    fn note_switcher_char_count_plural() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "s2".into(),
            title: "Many Chars".into(),
            char_count: 42,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "42 chars");
    }

    #[test]
    fn note_switcher_char_count_zero() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "s0".into(),
            title: "Empty Note".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "0 chars");
    }

    #[test]
    fn note_switcher_preview_exactly_60_chars_not_truncated() {
        let preview = "a".repeat(60);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "t60".into(),
            title: "Exact 60".into(),
            char_count: 60,
            is_current: false,
            is_pinned: false,
            preview: preview.clone(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, &preview);
        assert!(!desc.contains("…"), "60 chars should not be truncated");
    }

    #[test]
    fn note_switcher_preview_61_chars_is_truncated() {
        let preview = "a".repeat(61);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "t61".into(),
            title: "Over 60".into(),
            char_count: 61,
            is_current: false,
            is_pinned: false,
            preview,
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.ends_with("…"), "61 chars should be truncated with …");
    }

    #[test]
    fn note_switcher_relative_time_only_no_preview() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "to".into(),
            title: "Time Only".into(),
            char_count: 10,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "3d ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "3d ago");
    }

    // =========================================================================
    // 8. New chat with partial sections
    // =========================================================================

    #[test]
    fn new_chat_no_last_used_only_presets_and_models() {
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Settings,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "openai".into(),
            provider_display_name: "OpenAI".into(),
        }];
        let actions = get_new_chat_actions(&[], &presets, &models);
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].section.as_deref(), Some("Presets"));
        assert_eq!(actions[1].section.as_deref(), Some("Models"));
    }

    #[test]
    fn new_chat_only_models_no_presets_no_last_used() {
        let models = vec![
            NewChatModelInfo {
                model_id: "m1".into(),
                display_name: "Model 1".into(),
                provider: "p1".into(),
                provider_display_name: "Provider 1".into(),
            },
            NewChatModelInfo {
                model_id: "m2".into(),
                display_name: "Model 2".into(),
                provider: "p2".into(),
                provider_display_name: "Provider 2".into(),
            },
        ];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].id, "model_0");
        assert_eq!(actions[1].id, "model_1");
    }

    #[test]
    fn new_chat_only_last_used() {
        let last_used = vec![NewChatModelInfo {
            model_id: "lu1".into(),
            display_name: "Last Used".into(),
            provider: "p".into(),
            provider_display_name: "Provider".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "last_used_0");
        assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
        assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
    }

    #[test]
    fn new_chat_all_three_sections_have_correct_section_labels() {
        let last_used = vec![NewChatModelInfo {
            model_id: "lu".into(),
            display_name: "LU".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "pr".into(),
            name: "Preset".into(),
            icon: IconName::Code,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m".into(),
            display_name: "Model".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
        assert_eq!(actions[1].section.as_deref(), Some("Presets"));
        assert_eq!(actions[2].section.as_deref(), Some("Models"));
    }

    #[test]
    fn new_chat_preset_has_no_description() {
        let presets = vec![NewChatPresetInfo {
            id: "gen".into(),
            name: "General".into(),
            icon: IconName::Settings,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert!(actions[0].description.is_none());
    }


    // --- merged from tests_part_03.rs ---
    #[test]
    fn new_chat_model_has_provider_description() {
        let models = vec![NewChatModelInfo {
            model_id: "m".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].description.as_ref().unwrap(), "Anthropic");
    }

    // =========================================================================
    // 9. Score stacking (title+desc+shortcut all matching)
    // =========================================================================

    #[test]
    fn score_action_prefix_title_only() {
        let action = Action::new(
            "test",
            "Copy Path",
            Some("Copy the path".into()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "copy");
        assert!(score >= 100, "Prefix match: {}", score);
    }

    #[test]
    fn score_action_title_prefix_plus_description_match() {
        let action = Action::new(
            "test",
            "Copy Path",
            Some("Copy the path to clipboard".into()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "copy");
        // prefix(100) + description contains "copy"(15) = 115
        assert!(score >= 115, "Prefix + desc: {}", score);
    }

    #[test]
    fn score_action_title_prefix_plus_desc_plus_shortcut() {
        let action = Action::new(
            "test",
            "Copy Path",
            Some("Copy the path to clipboard".into()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘COPY");
        let score = ActionsDialog::score_action(&action, "copy");
        // prefix(100) + desc(15) + shortcut(10) = 125
        assert!(score >= 125, "Prefix + desc + shortcut: {}", score);
    }

    #[test]
    fn score_action_no_match_returns_zero() {
        let action = Action::new(
            "test",
            "Open File",
            Some("Open the file".into()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "xyz123");
        assert_eq!(score, 0);
    }

    #[test]
    fn score_action_contains_only_no_prefix() {
        let action = Action::new(
            "test",
            "Reset Copy Path",
            None,
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "copy");
        // Contains only: 50
        assert!(score >= 50, "Contains: {}", score);
        assert!(score < 100, "Should not be prefix: {}", score);
    }

    #[test]
    fn score_action_fuzzy_only() {
        let action = Action::new("test", "Extract Data", None, ActionCategory::ScriptContext);
        // "eda" matches E-x-t-r-a-c-t-D-A-t-a as subsequence e...d...a
        let score = ActionsDialog::score_action(&action, "eda");
        assert!(score >= 25, "Fuzzy match: {}", score);
        assert!(score < 50, "Should not be contains: {}", score);
    }

    #[test]
    fn score_action_description_only_match() {
        let action = Action::new(
            "test",
            "Open File",
            Some("Navigate to the editor".into()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "editor");
        // Description only: 15
        assert_eq!(score, 15, "Description-only match");
    }

    #[test]
    fn score_action_shortcut_only_match() {
        let action = Action::new(
            "test",
            "Open File",
            Some("Open the file".into()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘Z");
        let score = ActionsDialog::score_action(&action, "⌘z");
        // Shortcut contains: 10
        assert!(score >= 10, "Shortcut match: {}", score);
    }

    // =========================================================================
    // 10. build_grouped_items_static edge cases
    // =========================================================================

    #[test]
    fn grouped_items_same_section_no_duplicate_headers() {
        let actions = vec![
            Action::new("a1", "Action 1", None, ActionCategory::ScriptContext)
                .with_section("Response"),
            Action::new("a2", "Action 2", None, ActionCategory::ScriptContext)
                .with_section("Response"),
            Action::new("a3", "Action 3", None, ActionCategory::ScriptContext)
                .with_section("Response"),
        ];
        let filtered: Vec<usize> = (0..3).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Should be: 1 header + 3 items = 4
        assert_eq!(grouped.len(), 4);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 1, "Same section = 1 header");
    }

    #[test]
    fn grouped_items_alternating_sections_get_headers() {
        let actions = vec![
            Action::new("a1", "A1", None, ActionCategory::ScriptContext).with_section("Alpha"),
            Action::new("a2", "A2", None, ActionCategory::ScriptContext).with_section("Beta"),
            Action::new("a3", "A3", None, ActionCategory::ScriptContext).with_section("Alpha"),
        ];
        let filtered: Vec<usize> = (0..3).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Alpha header, A1, Beta header, A2, Alpha header again, A3 = 6
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 3, "Each section change = new header");
    }

    #[test]
    fn grouped_items_none_style_no_headers() {
        let actions = vec![
            Action::new("a1", "A1", None, ActionCategory::ScriptContext).with_section("Section"),
            Action::new("a2", "A2", None, ActionCategory::ScriptContext).with_section("Other"),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 0, "None style = no headers");
    }

    #[test]
    fn grouped_items_separators_style_no_headers() {
        let actions = vec![
            Action::new("a1", "A1", None, ActionCategory::ScriptContext).with_section("Section"),
            Action::new("a2", "A2", None, ActionCategory::ScriptContext).with_section("Other"),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 0, "Separators style = no headers");
    }

    #[test]
    fn grouped_items_empty_filtered_returns_empty() {
        let actions = vec![Action::new("a1", "A1", None, ActionCategory::ScriptContext)];
        let grouped = build_grouped_items_static(&actions, &[], SectionStyle::Headers);
        assert!(grouped.is_empty());
    }

    #[test]
    fn grouped_items_no_section_actions_with_headers_style() {
        let actions = vec![
            Action::new("a1", "A1", None, ActionCategory::ScriptContext),
            Action::new("a2", "A2", None, ActionCategory::ScriptContext),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // No sections means no headers, just items
        assert_eq!(grouped.len(), 2);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 0, "No sections = no headers");
    }

    // =========================================================================
    // 11. coerce_action_selection edge cases
    // =========================================================================

    #[test]
    fn coerce_selection_empty_rows_returns_none() {
        assert_eq!(coerce_action_selection(&[], 0), None);
    }

    #[test]
    fn coerce_selection_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".into()),
            GroupedActionItem::SectionHeader("B".into()),
            GroupedActionItem::SectionHeader("C".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
        assert_eq!(coerce_action_selection(&rows, 1), None);
        assert_eq!(coerce_action_selection(&rows, 2), None);
    }

    #[test]
    fn coerce_selection_on_item_returns_same_index() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".into()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(1));
    }

    #[test]
    fn coerce_selection_on_header_searches_down_first() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".into()),
            GroupedActionItem::Item(0),
            GroupedActionItem::Item(1),
        ];
        // Landing on header at 0, should go down to item at 1
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn coerce_selection_on_trailing_header_searches_up() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::Item(1),
            GroupedActionItem::SectionHeader("A".into()),
        ];
        // Landing on header at 2, no items below, should go up to item at 1
        assert_eq!(coerce_action_selection(&rows, 2), Some(1));
    }

    #[test]
    fn coerce_selection_out_of_bounds_clamped() {
        let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
        // Index 10 should be clamped to last index (1)
        assert_eq!(coerce_action_selection(&rows, 10), Some(1));
    }

    // =========================================================================
    // 12. ActionsDialog::format_shortcut_hint comprehensive
    // =========================================================================

    #[test]
    fn format_hint_cmd_c() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+c"), "⌘C");
    }

    #[test]
    fn format_hint_ctrl_shift_escape() {
        assert_eq!(
            ActionsDialog::format_shortcut_hint("ctrl+shift+escape"),
            "⌃⇧⎋"
        );
    }

    #[test]
    fn format_hint_alt_backspace() {
        assert_eq!(ActionsDialog::format_shortcut_hint("alt+backspace"), "⌥⌫");
    }

    #[test]
    fn format_hint_command_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("command+n"), "⌘N");
    }

    #[test]
    fn format_hint_meta_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("meta+n"), "⌘N");
    }

    #[test]
    fn format_hint_option_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("option+n"), "⌥N");
    }

    #[test]
    fn format_hint_control_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("control+x"), "⌃X");
    }

    #[test]
    fn format_hint_enter_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+enter"), "⌘↵");
    }

    #[test]
    fn format_hint_return_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+return"), "⌘↵");
    }

    #[test]
    fn format_hint_tab_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("shift+tab"), "⇧⇥");
    }

    #[test]
    fn format_hint_space_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+space"), "⌘␣");
    }

    #[test]
    fn format_hint_arrow_keys() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+up"), "⌘↑");
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+down"), "⌘↓");
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+left"), "⌘←");
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+right"), "⌘→");
    }

    #[test]
    fn format_hint_arrowup_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+arrowup"), "⌘↑");
    }

    #[test]
    fn format_hint_delete_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("cmd+delete"), "⌘⌫");
    }

    #[test]
    fn format_hint_esc_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("esc"), "⎋");
    }

    #[test]
    fn format_hint_super_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("super+k"), "⌘K");
    }

    #[test]
    fn format_hint_opt_alias() {
        assert_eq!(ActionsDialog::format_shortcut_hint("opt+k"), "⌥K");
    }

    // =========================================================================
    // 13. Path context edge cases
    // =========================================================================

    #[test]
    fn path_context_dir_primary_title_includes_name() {
        let info = PathInfo {
            path: "/Users/test/Documents".into(),
            name: "Documents".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].title, "Open \"Documents\"");
        assert_eq!(actions[0].id, "file:open_directory");
    }

    #[test]
    fn path_context_file_primary_title_includes_name() {
        let info = PathInfo {
            path: "/Users/test/file.txt".into(),
            name: "file.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].title, "Select \"file.txt\"");
        assert_eq!(actions[0].id, "file:select_file");
    }

    #[test]
    fn path_context_trash_description_dir() {
        let info = PathInfo {
            path: "/Users/test/Documents".into(),
            name: "Documents".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.iter().find(|a| a.id == "file:move_to_trash").unwrap();
        assert_eq!(trash.description.as_ref().unwrap(), "Delete folder");
    }

    #[test]
    fn path_context_trash_description_file() {
        let info = PathInfo {
            path: "/Users/test/file.txt".into(),
            name: "file.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.iter().find(|a| a.id == "file:move_to_trash").unwrap();
        assert_eq!(trash.description.as_ref().unwrap(), "Delete file");
    }

    #[test]
    fn path_context_always_has_copy_path_and_copy_filename() {
        let info = PathInfo {
            path: "/Users/test/file.txt".into(),
            name: "file.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"file:copy_path"));
        assert!(ids.contains(&"file:copy_filename"));
    }

    #[test]
    fn path_context_has_open_in_editor_and_terminal() {
        let info = PathInfo {
            path: "/Users/test/file.txt".into(),
            name: "file.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"file:open_in_editor"));
        assert!(ids.contains(&"file:open_in_terminal"));
    }


    // --- merged from tests_part_04.rs ---
    #[test]
    fn path_context_action_count_dir_vs_file() {
        let dir_info = PathInfo {
            path: "/test/dir".into(),
            name: "dir".into(),
            is_dir: true,
        };
        let file_info = PathInfo {
            path: "/test/file.txt".into(),
            name: "file.txt".into(),
            is_dir: false,
        };
        let dir_actions = get_path_context_actions(&dir_info);
        let file_actions = get_path_context_actions(&file_info);
        // Both should have same count (primary + copy_path + open_in_finder + open_in_editor
        // + open_in_terminal + copy_filename + move_to_trash = 7)
        assert_eq!(dir_actions.len(), file_actions.len());
        assert_eq!(dir_actions.len(), 7);
    }

    // =========================================================================
    // 14. File context FileType variants
    // =========================================================================

    #[test]
    fn file_context_document_type() {
        let info = FileInfo {
            path: "/test/doc.pdf".into(),
            name: "doc.pdf".into(),
            file_type: FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        assert_eq!(actions[0].id, "file:open_file");
        assert!(actions[0].title.contains("doc.pdf"));
    }

    #[test]
    fn file_context_image_type() {
        let info = FileInfo {
            path: "/test/photo.jpg".into(),
            name: "photo.jpg".into(),
            file_type: FileType::Image,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        assert_eq!(actions[0].id, "file:open_file");
    }

    #[test]
    fn file_context_audio_type() {
        let info = FileInfo {
            path: "/test/song.mp3".into(),
            name: "song.mp3".into(),
            file_type: FileType::Audio,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        assert_eq!(actions[0].id, "file:open_file");
    }

    #[test]
    fn file_context_video_type() {
        let info = FileInfo {
            path: "/test/movie.mp4".into(),
            name: "movie.mp4".into(),
            file_type: FileType::Video,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        assert_eq!(actions[0].id, "file:open_file");
    }

    #[test]
    fn file_context_application_type() {
        let info = FileInfo {
            path: "/Applications/Safari.app".into(),
            name: "Safari.app".into(),
            file_type: FileType::Application,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        assert_eq!(actions[0].id, "file:open_file");
        assert!(actions[0].title.contains("Safari.app"));
    }

    #[test]
    fn file_context_directory_type() {
        let info = FileInfo {
            path: "/test/folder".into(),
            name: "folder".into(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&info);
        assert_eq!(actions[0].id, "file:open_directory");
        assert!(actions[0].title.contains("folder"));
    }

    #[test]
    fn file_context_other_type() {
        let info = FileInfo {
            path: "/test/unknown.xyz".into(),
            name: "unknown.xyz".into(),
            file_type: FileType::Other,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        assert_eq!(actions[0].id, "file:open_file");
    }

    // =========================================================================
    // 15. Action builder chaining immutability
    // =========================================================================

    #[test]
    fn action_with_shortcut_preserves_other_fields() {
        let action = Action::new(
            "test",
            "Test Title",
            Some("Test Description".into()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘T");
        assert_eq!(action.id, "test");
        assert_eq!(action.title, "Test Title");
        assert_eq!(action.description, Some("Test Description".into()));
        assert_eq!(action.shortcut, Some("⌘T".into()));
    }

    #[test]
    fn action_with_icon_preserves_other_fields() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘T")
            .with_icon(IconName::Copy);
        assert_eq!(action.shortcut, Some("⌘T".into()));
        assert_eq!(action.icon, Some(IconName::Copy));
    }

    #[test]
    fn action_with_section_preserves_other_fields() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘T")
            .with_icon(IconName::Copy)
            .with_section("MySection");
        assert_eq!(action.shortcut, Some("⌘T".into()));
        assert_eq!(action.icon, Some(IconName::Copy));
        assert_eq!(action.section, Some("MySection".into()));
    }

    #[test]
    fn action_with_shortcut_opt_none_leaves_shortcut_none() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(None);
        assert!(action.shortcut.is_none());
        assert!(action.shortcut_lower.is_none());
    }

    #[test]
    fn action_with_shortcut_opt_some_sets_shortcut() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘K".into()));
        assert_eq!(action.shortcut, Some("⌘K".into()));
        assert_eq!(action.shortcut_lower, Some("⌘k".into()));
    }

    // =========================================================================
    // 16. Action lowercase cache correctness
    // =========================================================================

    #[test]
    fn title_lower_matches_title_to_lowercase() {
        let action = Action::new(
            "test",
            "Copy Path To Clipboard",
            None,
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.title_lower, "copy path to clipboard");
    }

    #[test]
    fn description_lower_matches_description_to_lowercase() {
        let action = Action::new(
            "test",
            "Test",
            Some("Open In $EDITOR".into()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.description_lower, Some("open in $editor".into()));
    }

    #[test]
    fn description_lower_none_when_no_description() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
        assert!(action.description_lower.is_none());
    }

    #[test]
    fn shortcut_lower_set_by_with_shortcut() {
        let action =
            Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
        assert_eq!(action.shortcut_lower, Some("⌘⇧c".into()));
    }

    #[test]
    fn shortcut_lower_none_when_no_shortcut() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
        assert!(action.shortcut_lower.is_none());
    }

    // =========================================================================
    // 17. CommandBarConfig default field values
    // =========================================================================

    #[test]
    fn commandbar_default_close_flags_all_true() {
        let config = CommandBarConfig::default();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    #[test]
    fn commandbar_ai_style_search_at_top() {
        let config = CommandBarConfig::ai_style();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
        assert_eq!(config.dialog_config.section_style, SectionStyle::Headers);
        assert!(config.dialog_config.show_icons);
        assert!(config.dialog_config.show_footer);
    }

    #[test]
    fn commandbar_main_menu_search_at_bottom() {
        let config = CommandBarConfig::main_menu_style();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Bottom);
        assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
        assert!(!config.dialog_config.show_icons);
        assert!(!config.dialog_config.show_footer);
    }

    #[test]
    fn commandbar_no_search_hidden() {
        let config = CommandBarConfig::no_search();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Hidden);
    }

    #[test]
    fn commandbar_notes_style_search_at_top_with_separators() {
        let config = CommandBarConfig::notes_style();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
        assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
        assert!(config.dialog_config.show_icons);
        assert!(config.dialog_config.show_footer);
    }

    // =========================================================================
    // 18. Scriptlet context vs script context action comparison
    // =========================================================================

    #[test]
    fn scriptlet_context_has_edit_scriptlet_not_edit_script() {
        let info = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"edit_scriptlet"));
        assert!(!ids.contains(&"edit_script"));
    }

    #[test]
    fn scriptlet_context_has_reveal_scriptlet_not_reveal() {
        let info = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"reveal_scriptlet_in_finder"));
        assert!(!ids.contains(&"file:reveal_in_finder"));
    }

    #[test]
    fn scriptlet_context_has_copy_scriptlet_path_not_copy_path() {
        let info = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"copy_scriptlet_path"));
        assert!(!ids.contains(&"file:copy_path"));
    }

    #[test]
    fn scriptlet_context_has_copy_content() {
        let info = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"copy_content"));
    }

    #[test]
    fn scriptlet_context_with_custom_actions_interleaved() {
        let info = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo test".into());
        scriptlet.actions = vec![
            ScriptletAction {
                name: "Action A".into(),
                command: "action-a".into(),
                tool: "bash".into(),
                code: "echo a".into(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
            ScriptletAction {
                name: "Action B".into(),
                command: "action-b".into(),
                tool: "bash".into(),
                code: "echo b".into(),
                inputs: vec![],
                shortcut: Some("cmd+b".into()),
                description: Some("Do B".into()),
            },
        ];
        let actions = get_scriptlet_context_actions_with_custom(&info, Some(&scriptlet));

        // run_script first
        assert_eq!(actions[0].id, "run_script");
        // Then custom actions
        assert_eq!(actions[1].id, "scriptlet_action:action-a");
        assert_eq!(actions[2].id, "scriptlet_action:action-b");
        // Custom actions have has_action=true
        assert!(actions[1].has_action);
        assert!(actions[2].has_action);
        // Custom action B has shortcut formatted
        assert!(actions[2].shortcut.is_some());
        // Custom action B has description
        assert_eq!(actions[2].description.as_deref(), Some("Do B"));
    }

    #[test]
    fn scriptlet_context_with_shortcut_and_alias() {
        let info = ScriptInfo::scriptlet(
            "Test",
            "/path/test.md",
            Some("cmd+t".into()),
            Some("ts".into()),
        );
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"update_shortcut"));
        assert!(ids.contains(&"remove_shortcut"));
        assert!(ids.contains(&"update_alias"));
        assert!(ids.contains(&"remove_alias"));
        assert!(!ids.contains(&"add_shortcut"));
        assert!(!ids.contains(&"add_alias"));
    }

    #[test]
    fn scriptlet_context_with_frecency() {
        let info = ScriptInfo::scriptlet("Test", "/path/test.md", None, None)
            .with_frecency(true, Some("scriptlet:Test".into()));
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"reset_ranking"));
    }

    // =========================================================================
    // 19. AI command bar actions detailed validation
    // =========================================================================

    #[test]
    fn ai_command_bar_has_exactly_12_actions() {
        let actions = get_ai_command_bar_actions();
        assert_eq!(actions.len(), 12);
    }

    #[test]
    fn ai_command_bar_response_section_actions() {
        let actions = get_ai_command_bar_actions();
        let response_actions: Vec<&Action> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Response"))
            .collect();
        assert_eq!(response_actions.len(), 3);
        let ids: Vec<&str> = response_actions.iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains(&"chat:copy_response"));
        assert!(ids.contains(&"chat:copy_chat"));
        assert!(ids.contains(&"chat:copy_last_code"));
    }

    #[test]
    fn ai_command_bar_actions_section_actions() {
        let actions = get_ai_command_bar_actions();
        let action_section: Vec<&Action> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Actions"))
            .collect();
        assert_eq!(action_section.len(), 4);
        let ids: Vec<&str> = action_section.iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains(&"chat:submit"));
        assert!(ids.contains(&"chat:new_chat"));
        assert!(ids.contains(&"chat:delete_chat"));
        assert!(ids.contains(&"chat:branch_from_last"));
    }

    #[test]
    fn ai_command_bar_attachments_section_actions() {
        let actions = get_ai_command_bar_actions();
        let att_actions: Vec<&Action> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Attachments"))
            .collect();
        assert_eq!(att_actions.len(), 2);
    }

    #[test]
    fn ai_command_bar_settings_section_actions() {
        let actions = get_ai_command_bar_actions();
        let settings_actions: Vec<&Action> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Settings"))
            .collect();
        assert_eq!(settings_actions.len(), 1);
        assert_eq!(settings_actions[0].id, "chat:change_model");
    }

    #[test]
    fn ai_command_bar_all_actions_have_icons() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                action.icon.is_some(),
                "AI action '{}' should have icon",
                action.id
            );
        }
    }


    // --- merged from tests_part_05.rs ---
    #[test]
    fn ai_command_bar_section_order_correct() {
        let actions = get_ai_command_bar_actions();
        let sections: Vec<&str> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        // Order: Response(3), Actions(3), Attachments(2), Export(1), Actions(1), Help(1), Settings(1)
        let unique_order: Vec<&str> = {
            let mut result = vec![];
            let mut prev: Option<&str> = None;
            for s in &sections {
                if prev != Some(s) {
                    result.push(*s);
                    prev = Some(s);
                }
            }
            result
        };
        assert_eq!(
            unique_order,
            vec![
                "Response",
                "Actions",
                "Attachments",
                "Export",
                "Actions",
                "Help",
                "Settings"
            ]
        );
    }

    // =========================================================================
    // 20. fuzzy_match edge cases
    // =========================================================================

    #[test]
    fn fuzzy_match_empty_needle_matches_anything() {
        assert!(ActionsDialog::fuzzy_match("hello", ""));
    }

    #[test]
    fn fuzzy_match_empty_haystack_no_match() {
        assert!(!ActionsDialog::fuzzy_match("", "a"));
    }

    #[test]
    fn fuzzy_match_both_empty_matches() {
        assert!(ActionsDialog::fuzzy_match("", ""));
    }

    #[test]
    fn fuzzy_match_exact_match() {
        assert!(ActionsDialog::fuzzy_match("hello", "hello"));
    }

    #[test]
    fn fuzzy_match_subsequence() {
        assert!(ActionsDialog::fuzzy_match("hello world", "hlwrd"));
    }

    #[test]
    fn fuzzy_match_no_subsequence() {
        assert!(!ActionsDialog::fuzzy_match("hello", "xyz"));
    }

    #[test]
    fn fuzzy_match_needle_longer_than_haystack() {
        assert!(!ActionsDialog::fuzzy_match("hi", "hello"));
    }

    #[test]
    fn fuzzy_match_single_char() {
        assert!(ActionsDialog::fuzzy_match("hello", "h"));
        assert!(ActionsDialog::fuzzy_match("hello", "o"));
        assert!(!ActionsDialog::fuzzy_match("hello", "z"));
    }

    // =========================================================================
    // 21. parse_shortcut_keycaps edge cases
    // =========================================================================

    #[test]
    fn parse_keycaps_modifier_plus_letter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘C");
        assert_eq!(keycaps, vec!["⌘", "C"]);
    }

    #[test]
    fn parse_keycaps_two_modifiers() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⇧C");
        assert_eq!(keycaps, vec!["⌘", "⇧", "C"]);
    }

    #[test]
    fn parse_keycaps_enter_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("↵");
        assert_eq!(keycaps, vec!["↵"]);
    }

    #[test]
    fn parse_keycaps_arrow_keys() {
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("↑"), vec!["↑"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("↓"), vec!["↓"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("←"), vec!["←"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("→"), vec!["→"]);
    }

    #[test]
    fn parse_keycaps_escape_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⎋");
        assert_eq!(keycaps, vec!["⎋"]);
    }

    #[test]
    fn parse_keycaps_space_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("␣");
        assert_eq!(keycaps, vec!["␣"]);
    }

    #[test]
    fn parse_keycaps_lowercase_uppercased() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘e");
        assert_eq!(keycaps, vec!["⌘", "E"]);
    }

    // =========================================================================
    // 22. to_deeplink_name edge cases
    // =========================================================================

    #[test]
    fn deeplink_name_basic() {
        assert_eq!(to_deeplink_name("My Script"), "my-script");
    }

    #[test]
    fn deeplink_name_underscores_to_hyphens() {
        assert_eq!(to_deeplink_name("hello_world"), "hello-world");
    }

    #[test]
    fn deeplink_name_special_chars_stripped() {
        assert_eq!(to_deeplink_name("Hello!@#$World"), "hello-world");
    }

    #[test]
    fn deeplink_name_multiple_spaces_collapsed() {
        assert_eq!(to_deeplink_name("My   Script"), "my-script");
    }

    #[test]
    fn deeplink_name_leading_trailing_stripped() {
        assert_eq!(to_deeplink_name("  My Script  "), "my-script");
    }

    #[test]
    fn deeplink_name_numbers_preserved() {
        assert_eq!(to_deeplink_name("Script 123"), "script-123");
    }

    #[test]
    fn deeplink_name_all_special_chars_empty() {
        assert_eq!(to_deeplink_name("!@#$%^&*"), "");
    }

    #[test]
    fn deeplink_name_already_hyphenated() {
        assert_eq!(to_deeplink_name("already-hyphenated"), "already-hyphenated");
    }

    #[test]
    fn deeplink_name_mixed_case() {
        assert_eq!(to_deeplink_name("CamelCaseScript"), "camelcasescript");
    }

    // =========================================================================
    // 23. Agent ScriptInfo with full flag set
    // =========================================================================

    #[test]
    fn agent_with_shortcut_alias_frecency() {
        let mut info = ScriptInfo::with_all(
            "My Agent",
            "/path/agent.md",
            false,
            "Run",
            Some("cmd+a".into()),
            Some("ma".into()),
        );
        info.is_agent = true;
        let info = info.with_frecency(true, Some("agent:/path".into()));

        let actions = get_script_context_actions(&info);
        let ids = action_ids(&actions);

        // Agent-specific actions
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");

        // Has update/remove for shortcut and alias
        assert!(ids.contains(&"update_shortcut"));
        assert!(ids.contains(&"remove_shortcut"));
        assert!(ids.contains(&"update_alias"));
        assert!(ids.contains(&"remove_alias"));

        // Has frecency reset
        assert!(ids.contains(&"reset_ranking"));

        // Has agent copy actions
        assert!(ids.contains(&"file:copy_path"));
        assert!(ids.contains(&"copy_content"));
        assert!(ids.contains(&"file:reveal_in_finder"));
    }

    // =========================================================================
    // 24. Global actions always empty
    // =========================================================================

    #[test]
    fn global_actions_is_empty() {
        assert!(get_global_actions().is_empty());
    }

    // =========================================================================
    // 25. Ordering determinism across repeated calls
    // =========================================================================

    #[test]
    fn script_actions_deterministic() {
        let info = ScriptInfo::new("Test", "/path/test.ts");
        let a1 = get_script_context_actions(&info);
        let a2 = get_script_context_actions(&info);
        let ids1 = action_ids(&a1);
        let ids2 = action_ids(&a2);
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn clipboard_actions_deterministic() {
        let entry = ClipboardEntryInfo {
            id: "det".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let a1 = get_clipboard_history_context_actions(&entry);
        let a2 = get_clipboard_history_context_actions(&entry);
        let ids1 = action_ids(&a1);
        let ids2 = action_ids(&a2);
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn ai_actions_deterministic() {
        let a1 = get_ai_command_bar_actions();
        let a2 = get_ai_command_bar_actions();
        let ids1 = action_ids(&a1);
        let ids2 = action_ids(&a2);
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn notes_actions_deterministic() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let a1 = get_notes_command_bar_actions(&info);
        let a2 = get_notes_command_bar_actions(&info);
        let ids1 = action_ids(&a1);
        let ids2 = action_ids(&a2);
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn path_actions_deterministic() {
        let info = PathInfo {
            path: "/test/dir".into(),
            name: "dir".into(),
            is_dir: true,
        };
        let a1 = get_path_context_actions(&info);
        let a2 = get_path_context_actions(&info);
        let ids1 = action_ids(&a1);
        let ids2 = action_ids(&a2);
        assert_eq!(ids1, ids2);
    }

    // =========================================================================
    // 26. has_action invariant across contexts
    // =========================================================================

    #[test]
    fn script_context_all_has_action_false() {
        let info = ScriptInfo::new("Test", "/path/test.ts");
        let actions = get_script_context_actions(&info);
        for action in &actions {
            assert!(
                !action.has_action,
                "Script action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn clipboard_context_all_has_action_false() {
        let entry = ClipboardEntryInfo {
            id: "ha".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        for action in &actions {
            assert!(
                !action.has_action,
                "Clipboard action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn path_context_all_has_action_false() {
        let info = PathInfo {
            path: "/test/file.txt".into(),
            name: "file.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        for action in &actions {
            assert!(
                !action.has_action,
                "Path action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn file_context_all_has_action_false() {
        let info = FileInfo {
            path: "/test/file.txt".into(),
            name: "file.txt".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        for action in &actions {
            assert!(
                !action.has_action,
                "File action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn ai_command_bar_all_has_action_false() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                !action.has_action,
                "AI action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn notes_command_bar_all_has_action_false() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for action in &actions {
            assert!(
                !action.has_action,
                "Notes action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn chat_context_builtin_actions_has_action_false() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        for action in &actions {
            assert!(
                !action.has_action,
                "Chat action '{}' should have has_action=false",
                action.id
            );
        }
    }

    // =========================================================================
    // 27. Scriptlet defined actions have has_action=true
    // =========================================================================

    #[test]
    fn scriptlet_defined_actions_all_have_has_action_true() {
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo test".into());
        scriptlet.actions = vec![
            ScriptletAction {
                name: "Action 1".into(),
                command: "action-1".into(),
                tool: "bash".into(),
                code: "echo 1".into(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
            ScriptletAction {
                name: "Action 2".into(),
                command: "action-2".into(),
                tool: "bash".into(),
                code: "echo 2".into(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
        ];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        for action in &actions {
            assert!(
                action.has_action,
                "Scriptlet defined action '{}' should have has_action=true",
                action.id
            );
        }
    }


    // --- merged from tests_part_06.rs ---
    #[test]
    fn scriptlet_defined_actions_have_values() {
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo test".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Custom".into(),
            command: "custom-cmd".into(),
            tool: "bash".into(),
            code: "echo custom".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions[0].value, Some("custom-cmd".into()));
    }

    #[test]
    fn scriptlet_defined_action_id_format() {
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo test".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "My Custom".into(),
            command: "my-custom".into(),
            tool: "bash".into(),
            code: "echo custom".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions[0].id, "scriptlet_action:my-custom");
    }

    // =========================================================================
    // 28. Action ID uniqueness within contexts
    // =========================================================================

    #[test]
    fn script_context_ids_unique() {
        let info = ScriptInfo::new("Test", "/path/test.ts");
        let actions = get_script_context_actions(&info);
        let ids = action_ids(&actions);
        let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "Script IDs should be unique");
    }

    #[test]
    fn clipboard_text_ids_unique() {
        let entry = ClipboardEntryInfo {
            id: "uniq".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "Clipboard IDs should be unique");
    }

    #[test]
    fn path_context_ids_unique() {
        let info = PathInfo {
            path: "/test".into(),
            name: "test".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let ids = action_ids(&actions);
        let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "Path IDs should be unique");
    }

    #[test]
    fn file_context_ids_unique() {
        let info = FileInfo {
            path: "/test/file.txt".into(),
            name: "file.txt".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        let ids = action_ids(&actions);
        let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "File IDs should be unique");
    }

    #[test]
    fn ai_command_bar_ids_unique() {
        let actions = get_ai_command_bar_actions();
        let ids = action_ids(&actions);
        let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "AI IDs should be unique");
    }

    #[test]
    fn notes_command_bar_ids_unique() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "Notes IDs should be unique");
    }

    // =========================================================================
    // 29. All actions have non-empty title and ID
    // =========================================================================

    #[test]
    fn all_script_actions_nonempty_title_and_id() {
        let info = ScriptInfo::new("Test", "/path/test.ts");
        let actions = get_script_context_actions(&info);
        for action in &actions {
            assert!(!action.id.is_empty(), "Action ID should not be empty");
            assert!(!action.title.is_empty(), "Action title should not be empty");
        }
    }

    #[test]
    fn all_clipboard_actions_nonempty_title_and_id() {
        let entry = ClipboardEntryInfo {
            id: "ne".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        for action in &actions {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    #[test]
    fn all_ai_actions_nonempty_title_and_id() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    // =========================================================================
    // 30. ActionCategory on all built-in actions
    // =========================================================================

    #[test]
    fn all_script_actions_are_script_context_category() {
        let info = ScriptInfo::new("Test", "/path/test.ts");
        let actions = get_script_context_actions(&info);
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
    fn all_clipboard_actions_are_script_context_category() {
        let entry = ClipboardEntryInfo {
            id: "cat".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        for action in &actions {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn all_path_actions_are_script_context_category() {
        let info = PathInfo {
            path: "/test".into(),
            name: "test".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        for action in &actions {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn all_file_actions_are_script_context_category() {
        let info = FileInfo {
            path: "/test/file.txt".into(),
            name: "file.txt".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        for action in &actions {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

}
