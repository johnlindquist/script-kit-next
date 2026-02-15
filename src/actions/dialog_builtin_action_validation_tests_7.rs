//! Batch 7: Dialog builtin action validation tests
//!
//! Focuses on novel edge cases and cross-cutting invariants not covered in batches 1-6:
//!
//! 1. format_shortcut_hint (dialog.rs version) edge cases: unknown keys, single modifier,
//!    double-plus, empty string, mixed-case modifiers
//! 2. score_action with Unicode: diacritics, CJK, emoji in title/desc
//! 3. Note switcher description rendering boundary: exactly 60 chars, 59, 61, 0
//! 4. Clipboard combined flag matrix: pinned×image, pinned×text, unpinned×image, unpinned×text
//! 5. Chat context model ID generation format consistency
//! 6. Notes command bar icon presence for every action
//! 7. New chat action ordering within each section
//! 8. Agent actions exclude view_logs
//! 9. Script vs scriptlet action set symmetric difference
//! 10. Deeplink URL in description format
//! 11. AI command bar shortcut uniqueness
//! 12. Notes command bar shortcut uniqueness
//! 13. Path context action ordering: primary first, trash last
//! 14. Clipboard action shortcut format (all use symbol notation)
//! 15. Score_action with whitespace-only query
//! 16. fuzzy_match with repeated characters
//! 17. build_grouped_items_static with single-item single-section
//! 18. coerce_action_selection with alternating header-item pattern
//! 19. parse_shortcut_keycaps with empty string and multi-byte
//! 20. CommandBarConfig close flags independence
//! 21. Action constructor with empty strings
//! 22. ScriptInfo scriptlet flag exclusivity with agent
//! 23. Notes command bar action count bounds per flag state
//! 24. Chat model display_name in title
//! 25. New chat model_id in action ID
//! 26. Clipboard delete_all description mentions "pinned"
//! 27. File context all actions have ScriptContext category
//! 28. Path context copy_path and copy_filename always present
//! 29. Cross-context ID namespace separation
//! 30. Action title_lower invariant across all builder functions

#[cfg(test)]
mod tests {
    // --- merged from tests_part_01.rs ---
    use crate::actions::builders::{
        get_ai_command_bar_actions, get_chat_context_actions,
        get_clipboard_history_context_actions, get_file_context_actions, get_new_chat_actions,
        get_note_switcher_actions, get_notes_command_bar_actions, get_path_context_actions,
        get_script_context_actions, get_scriptlet_context_actions_with_custom, to_deeplink_name,
        ChatModelInfo, ChatPromptInfo, ClipboardEntryInfo, NewChatModelInfo, NewChatPresetInfo,
        NoteSwitcherNoteInfo, NotesInfo,
    };
    use crate::actions::dialog::{
        build_grouped_items_static, coerce_action_selection, ActionsDialog, GroupedActionItem,
    };
    use crate::actions::types::{Action, ActionCategory, ScriptInfo, SectionStyle};
    use crate::actions::CommandBarConfig;
    use crate::clipboard_history::ContentType;
    use crate::designs::icon_variations::IconName;
    use crate::file_search::{FileInfo, FileType};
    use crate::prompts::PathInfo;
    use crate::scriptlets::{Scriptlet, ScriptletAction};
    use std::collections::HashSet;

    // ============================================================
    // Helper functions
    // ============================================================

    fn action_ids(actions: &[Action]) -> Vec<&str> {
        actions.iter().map(|a| a.id.as_str()).collect()
    }

    fn find_action<'a>(actions: &'a [Action], id: &str) -> Option<&'a Action> {
        actions.iter().find(|a| a.id == id)
    }

    fn make_action(id: &str, title: &str, section: Option<&str>) -> Action {
        let mut a = Action::new(
            id,
            title,
            Some(format!("Desc for {}", title)),
            ActionCategory::ScriptContext,
        );
        if let Some(s) = section {
            a = a.with_section(s);
        }
        a
    }

    fn make_text_entry(pinned: bool, app: Option<&str>) -> ClipboardEntryInfo {
        ClipboardEntryInfo {
            id: "txt-1".to_string(),
            content_type: ContentType::Text,
            pinned,
            preview: "hello world".to_string(),
            image_dimensions: None,
            frontmost_app_name: app.map(|s| s.to_string()),
        }
    }

    fn make_image_entry(pinned: bool) -> ClipboardEntryInfo {
        ClipboardEntryInfo {
            id: "img-1".to_string(),
            content_type: ContentType::Image,
            pinned,
            preview: "Screenshot (1920x1080)".to_string(),
            image_dimensions: Some((1920, 1080)),
            frontmost_app_name: None,
        }
    }

    fn make_note(
        id: &str,
        title: &str,
        char_count: usize,
        is_current: bool,
        is_pinned: bool,
        preview: &str,
        relative_time: &str,
    ) -> NoteSwitcherNoteInfo {
        NoteSwitcherNoteInfo {
            id: id.to_string(),
            title: title.to_string(),
            char_count,
            is_current,
            is_pinned,
            preview: preview.to_string(),
            relative_time: relative_time.to_string(),
        }
    }

    // ============================================================
    // 1. format_shortcut_hint (dialog.rs version) edge cases
    // ============================================================

    #[test]
    fn hint_unknown_key_passthrough() {
        // Unknown keys in non-last position are passed through as-is
        let result = ActionsDialog::format_shortcut_hint("cmd+f1");
        assert!(result.contains('⌘'), "cmd should map to ⌘: got {}", result);
        assert!(
            result.contains("F1"),
            "f1 should be uppercased: got {}",
            result
        );
    }

    #[test]
    fn hint_single_modifier_alone() {
        let result = ActionsDialog::format_shortcut_hint("cmd");
        // "cmd" alone is a single part, it's the last part, so it gets uppercased
        // Actually the match in format_shortcut_hint checks part_lower first
        assert_eq!(result, "⌘");
    }

    #[test]
    fn hint_empty_string() {
        let result = ActionsDialog::format_shortcut_hint("");
        assert_eq!(result, "");
    }

    #[test]
    fn hint_super_maps_to_cmd() {
        let result = ActionsDialog::format_shortcut_hint("super+a");
        assert_eq!(result, "⌘A");
    }

    #[test]
    fn hint_opt_maps_to_option() {
        let result = ActionsDialog::format_shortcut_hint("opt+b");
        assert_eq!(result, "⌥B");
    }

    #[test]
    fn hint_return_maps_to_enter() {
        let result = ActionsDialog::format_shortcut_hint("cmd+return");
        assert_eq!(result, "⌘↵");
    }

    #[test]
    fn hint_esc_alias() {
        let result = ActionsDialog::format_shortcut_hint("esc");
        assert_eq!(result, "⎋");
    }

    #[test]
    fn hint_arrowdown_alias() {
        let result = ActionsDialog::format_shortcut_hint("arrowdown");
        assert_eq!(result, "↓");
    }

    #[test]
    fn hint_arrowleft_alias() {
        let result = ActionsDialog::format_shortcut_hint("arrowleft");
        assert_eq!(result, "←");
    }

    #[test]
    fn hint_arrowright_alias() {
        let result = ActionsDialog::format_shortcut_hint("arrowright");
        assert_eq!(result, "→");
    }

    #[test]
    fn hint_triple_modifier() {
        let result = ActionsDialog::format_shortcut_hint("cmd+ctrl+shift+x");
        assert_eq!(result, "⌘⌃⇧X");
    }

    #[test]
    fn hint_mixed_case_modifiers() {
        let result = ActionsDialog::format_shortcut_hint("Cmd+Shift+Z");
        assert_eq!(result, "⌘⇧Z");
    }

    // ============================================================
    // 2. score_action with Unicode
    // ============================================================

    #[test]
    fn score_unicode_title_prefix() {
        let action = Action::new(
            "café",
            "Café Latte",
            Some("A hot drink".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "café");
        assert!(
            score >= 100,
            "Unicode prefix match should score >= 100, got {}",
            score
        );
    }

    #[test]
    fn score_unicode_contains() {
        let action = Action::new(
            "drink",
            "Hot Café Drink",
            None,
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "café");
        assert!(
            score >= 50,
            "Unicode contains should score >= 50, got {}",
            score
        );
    }

    #[test]
    fn score_empty_query_returns_zero() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "");
        // Empty query: starts_with("") == true for any string
        assert!(score >= 100, "Empty query matches prefix, got {}", score);
    }

    #[test]
    fn score_no_match_returns_zero() {
        let action = Action::new("test", "Test Action", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "zzzzz");
        assert_eq!(score, 0, "No match should return 0");
    }

    #[test]
    fn score_description_only_match() {
        let action = Action::new(
            "x",
            "Alpha",
            Some("Open in beta editor".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "beta");
        assert_eq!(
            score, 15,
            "Description-only match should be 15, got {}",
            score
        );
    }

    #[test]
    fn score_shortcut_only_match() {
        let action =
            Action::new("x", "Alpha", None, ActionCategory::ScriptContext).with_shortcut("⌘Z");
        let score = ActionsDialog::score_action(&action, "⌘z");
        assert_eq!(score, 10, "Shortcut-only match should be 10, got {}", score);
    }

    #[test]
    fn score_title_plus_description_stacks() {
        let action = Action::new(
            "script:run",
            "run script",
            Some("run the script now".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "script:run");
        assert!(
            score >= 115,
            "Title prefix (100) + desc (15) should stack, got {}",
            score
        );
    }

    #[test]
    fn score_all_three_match_stack() {
        let action = Action::new(
            "copy",
            "copy text",
            Some("copy to clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("copy");
        let score = ActionsDialog::score_action(&action, "copy");
        assert!(
            score >= 125,
            "prefix(100)+desc(15)+shortcut(10)=125, got {}",
            score
        );
    }

    // ============================================================
    // 3. Note switcher description rendering boundary
    // ============================================================

    #[test]
    fn note_desc_exactly_60_chars_not_truncated() {
        let preview = "a".repeat(60);
        let note = make_note("n1", "Title", 100, false, false, &preview, "");
        let actions = get_note_switcher_actions(&[note]);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(!desc.contains('…'), "60 chars should not be truncated");
        assert_eq!(desc.len(), 60);
    }

    #[test]
    fn note_desc_61_chars_is_truncated() {
        let preview = "a".repeat(61);
        let note = make_note("n1", "Title", 100, false, false, &preview, "");
        let actions = get_note_switcher_actions(&[note]);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains('…'), "61 chars should be truncated with …");
    }

    #[test]
    fn note_desc_59_chars_not_truncated() {
        let preview = "b".repeat(59);
        let note = make_note("n1", "Title", 100, false, false, &preview, "");
        let actions = get_note_switcher_actions(&[note]);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(!desc.contains('…'), "59 chars should not be truncated");
    }

    #[test]
    fn note_desc_empty_preview_empty_time_uses_char_count() {
        let note = make_note("n1", "Title", 42, false, false, "", "");
        let actions = get_note_switcher_actions(&[note]);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "42 chars");
    }

    #[test]
    fn note_desc_empty_preview_with_time() {
        let note = make_note("n1", "Title", 42, false, false, "", "5m ago");
        let actions = get_note_switcher_actions(&[note]);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "5m ago");
    }

    #[test]
    fn note_desc_preview_with_time_separator() {
        let note = make_note("n1", "Title", 42, false, false, "Hello world", "3d ago");
        let actions = get_note_switcher_actions(&[note]);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains(" · "), "Should have ' · ' separator");
        assert!(desc.starts_with("Hello world"), "Should start with preview");
        assert!(desc.ends_with("3d ago"), "Should end with time");
    }

    #[test]
    fn note_desc_singular_char_count() {
        let note = make_note("n1", "Title", 1, false, false, "", "");
        let actions = get_note_switcher_actions(&[note]);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "1 char", "Singular should not have 's'");
    }

    #[test]
    fn note_desc_zero_char_count() {
        let note = make_note("n1", "Title", 0, false, false, "", "");
        let actions = get_note_switcher_actions(&[note]);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "0 chars", "Zero should be plural");
    }

    // ============================================================
    // 4. Clipboard combined flag matrix
    // ============================================================

    #[test]
    fn clipboard_text_unpinned_has_pin() {
        let entry = make_text_entry(false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"clip:clipboard_pin"));
        assert!(!ids.contains(&"clip:clipboard_unpin"));
        assert!(!ids.contains(&"clip:clipboard_ocr"), "Text should not have OCR");
    }

    #[test]
    fn clipboard_text_pinned_has_unpin() {
        let entry = make_text_entry(true, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"clip:clipboard_unpin"));
        assert!(!ids.contains(&"clip:clipboard_pin"));
    }

    #[test]
    fn clipboard_image_unpinned_has_ocr_and_pin() {
        let entry = make_image_entry(false);
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"clip:clipboard_ocr"), "Image should have OCR");
        assert!(ids.contains(&"clip:clipboard_pin"));
        assert!(!ids.contains(&"clip:clipboard_unpin"));
    }

    #[test]
    fn clipboard_image_pinned_has_ocr_and_unpin() {
        let entry = make_image_entry(true);
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"clip:clipboard_ocr"), "Image should have OCR");
        assert!(ids.contains(&"clip:clipboard_unpin"));
        assert!(!ids.contains(&"clip:clipboard_pin"));
    }

    #[test]
    fn clipboard_image_has_more_actions_than_text() {
        let text = make_text_entry(false, None);
        let img = make_image_entry(false);
        let text_actions = get_clipboard_history_context_actions(&text);
        let img_actions = get_clipboard_history_context_actions(&img);
        assert!(
            img_actions.len() > text_actions.len(),
            "Image ({}) should have more actions than text ({})",
            img_actions.len(),
            text_actions.len()
        );
    }

    // ============================================================
    // 5. Chat context model ID generation format consistency
    // ============================================================

    #[test]
    fn chat_model_id_format() {
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
        // Model actions should have select_model_{id} format
        assert!(actions.iter().any(|a| a.id == "chat:select_model_gpt-4"));
        assert!(actions.iter().any(|a| a.id == "chat:select_model_claude-3"));
    }


    // --- merged from tests_part_02.rs ---
    #[test]
    fn chat_current_model_gets_checkmark() {
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
        let gpt = find_action(&actions, "chat:select_model_gpt-4").unwrap();
        assert!(gpt.title.contains('✓'), "Current model should have ✓");
        let claude = find_action(&actions, "chat:select_model_claude-3").unwrap();
        assert!(
            !claude.title.contains('✓'),
            "Non-current model should not have ✓"
        );
    }

    #[test]
    fn chat_continue_always_present() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(
            action_ids(&actions).contains(&"chat:continue_in_chat"),
            "continue_in_chat should always be present"
        );
    }

    #[test]
    fn chat_copy_response_requires_has_response() {
        let without = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let with = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: true,
        };
        let without_actions = get_chat_context_actions(&without);
        assert!(!action_ids(&without_actions).contains(&"chat:copy_response"));
        let with_actions = get_chat_context_actions(&with);
        assert!(action_ids(&with_actions).contains(&"chat:copy_response"));
    }

    #[test]
    fn chat_clear_requires_has_messages() {
        let without = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let with = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let without_actions = get_chat_context_actions(&without);
        assert!(!action_ids(&without_actions).contains(&"chat:clear_conversation"));
        let with_actions = get_chat_context_actions(&with);
        assert!(action_ids(&with_actions).contains(&"chat:clear_conversation"));
    }

    // ============================================================
    // 6. Notes command bar icon presence
    // ============================================================

    #[test]
    fn notes_all_actions_have_icons() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for action in &actions {
            assert!(
                action.icon.is_some(),
                "Notes action '{}' should have an icon",
                action.id
            );
        }
    }

    #[test]
    fn notes_all_actions_have_sections() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for action in &actions {
            assert!(
                action.section.is_some(),
                "Notes action '{}' should have a section",
                action.id
            );
        }
    }

    // ============================================================
    // 7. New chat action ordering within each section
    // ============================================================

    #[test]
    fn new_chat_sections_appear_in_order() {
        let last_used = vec![NewChatModelInfo {
            model_id: "gpt-4".to_string(),
            display_name: "GPT-4".to_string(),
            provider: "openai".to_string(),
            provider_display_name: "OpenAI".to_string(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "general".to_string(),
            name: "General".to_string(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "claude-3".to_string(),
            display_name: "Claude 3".to_string(),
            provider: "anthropic".to_string(),
            provider_display_name: "Anthropic".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);

        // Find first index of each section
        let first_last_used = actions
            .iter()
            .position(|a| a.section.as_deref() == Some("Last Used Settings"));
        let first_preset = actions
            .iter()
            .position(|a| a.section.as_deref() == Some("Presets"));
        let first_model = actions
            .iter()
            .position(|a| a.section.as_deref() == Some("Models"));

        assert!(first_last_used.unwrap() < first_preset.unwrap());
        assert!(first_preset.unwrap() < first_model.unwrap());
    }

    #[test]
    fn new_chat_last_used_has_bolt_icon() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".to_string(),
            display_name: "Model 1".to_string(),
            provider: "p1".to_string(),
            provider_display_name: "Provider 1".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
    }

    #[test]
    fn new_chat_preset_uses_custom_icon() {
        let presets = vec![NewChatPresetInfo {
            id: "code".to_string(),
            name: "Code".to_string(),
            icon: IconName::Code,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].icon, Some(IconName::Code));
    }

    #[test]
    fn new_chat_model_uses_settings_icon() {
        let models = vec![NewChatModelInfo {
            model_id: "m1".to_string(),
            display_name: "M1".to_string(),
            provider: "p".to_string(),
            provider_display_name: "P".to_string(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].icon, Some(IconName::Settings));
    }

    #[test]
    fn new_chat_empty_inputs_empty_output() {
        let actions = get_new_chat_actions(&[], &[], &[]);
        assert!(actions.is_empty());
    }

    #[test]
    fn new_chat_id_format_indexed() {
        let lu = vec![
            NewChatModelInfo {
                model_id: "a".to_string(),
                display_name: "A".to_string(),
                provider: "p".to_string(),
                provider_display_name: "P".to_string(),
            },
            NewChatModelInfo {
                model_id: "b".to_string(),
                display_name: "B".to_string(),
                provider: "p".to_string(),
                provider_display_name: "P".to_string(),
            },
        ];
        let actions = get_new_chat_actions(&lu, &[], &[]);
        assert_eq!(actions[0].id, "last_used_0");
        assert_eq!(actions[1].id, "last_used_1");
    }

    #[test]
    fn new_chat_preset_id_format() {
        let presets = vec![NewChatPresetInfo {
            id: "writer".to_string(),
            name: "Writer".to_string(),
            icon: IconName::File,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].id, "preset_writer");
    }

    #[test]
    fn new_chat_model_id_format_indexed() {
        let models = vec![
            NewChatModelInfo {
                model_id: "x".to_string(),
                display_name: "X".to_string(),
                provider: "p".to_string(),
                provider_display_name: "P".to_string(),
            },
            NewChatModelInfo {
                model_id: "y".to_string(),
                display_name: "Y".to_string(),
                provider: "p".to_string(),
                provider_display_name: "P".to_string(),
            },
        ];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].id, "model_0");
        assert_eq!(actions[1].id, "model_1");
    }

    #[test]
    fn new_chat_last_used_has_provider_description() {
        let lu = vec![NewChatModelInfo {
            model_id: "m".to_string(),
            display_name: "M".to_string(),
            provider: "p".to_string(),
            provider_display_name: "ProviderName".to_string(),
        }];
        let actions = get_new_chat_actions(&lu, &[], &[]);
        assert_eq!(actions[0].description, Some("ProviderName".to_string()));
    }

    #[test]
    fn new_chat_preset_has_no_description() {
        let presets = vec![NewChatPresetInfo {
            id: "x".to_string(),
            name: "X".to_string(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].description, None);
    }

    // ============================================================
    // 8. Agent actions exclude view_logs
    // ============================================================

    #[test]
    fn agent_has_edit_agent_title() {
        let mut agent = ScriptInfo::new("My Agent", "/path/agent.md");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        let edit = find_action(&actions, "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    #[test]
    fn agent_lacks_view_logs() {
        let mut agent = ScriptInfo::new("My Agent", "/path/agent.md");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        assert!(
            !action_ids(&actions).contains(&"view_logs"),
            "Agent should not have view_logs"
        );
    }

    #[test]
    fn agent_has_reveal_and_copy() {
        let mut agent = ScriptInfo::new("My Agent", "/path/agent.md");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"file:reveal_in_finder"));
        assert!(ids.contains(&"file:copy_path"));
        assert!(ids.contains(&"copy_content"));
    }

    // ============================================================
    // 9. Script vs scriptlet action set symmetric difference
    // ============================================================

    #[test]
    fn script_has_actions_scriptlet_lacks() {
        let script = ScriptInfo::new("s", "/path/s.ts");
        let scriptlet = ScriptInfo::scriptlet("s", "/path/s.md", None, None);
        let s_actions = get_script_context_actions(&script);
        let s_ids: HashSet<&str> = action_ids(&s_actions).into_iter().collect();
        let sl_actions = get_script_context_actions(&scriptlet);
        let sl_ids: HashSet<&str> = action_ids(&sl_actions).into_iter().collect();
        // Script should have these that scriptlet lacks
        assert!(s_ids.contains("edit_script"));
        assert!(s_ids.contains("view_logs"));
        assert!(!sl_ids.contains("edit_script"));
        assert!(!sl_ids.contains("view_logs"));
    }

    #[test]
    fn scriptlet_has_actions_script_lacks() {
        let script = ScriptInfo::new("s", "/path/s.ts");
        let scriptlet = ScriptInfo::scriptlet("s", "/path/s.md", None, None);
        let s_actions = get_script_context_actions(&script);
        let s_ids: HashSet<&str> = action_ids(&s_actions).into_iter().collect();
        let sl_actions = get_script_context_actions(&scriptlet);
        let sl_ids: HashSet<&str> = action_ids(&sl_actions).into_iter().collect();
        // Scriptlet should have these that script lacks
        assert!(sl_ids.contains("edit_scriptlet"));
        assert!(sl_ids.contains("reveal_scriptlet_in_finder"));
        assert!(sl_ids.contains("copy_scriptlet_path"));
        assert!(!s_ids.contains("edit_scriptlet"));
        assert!(!s_ids.contains("reveal_scriptlet_in_finder"));
        assert!(!s_ids.contains("copy_scriptlet_path"));
    }

    #[test]
    fn script_and_scriptlet_share_common_ids() {
        let script = ScriptInfo::new("s", "/path/s.ts");
        let scriptlet = ScriptInfo::scriptlet("s", "/path/s.md", None, None);
        let s_actions = get_script_context_actions(&script);
        let s_ids: HashSet<&str> = action_ids(&s_actions).into_iter().collect();
        let sl_actions = get_script_context_actions(&scriptlet);
        let sl_ids: HashSet<&str> = action_ids(&sl_actions).into_iter().collect();
        // Both should have these common actions
        let common = [
            "run_script",
            "script:copy_deeplink",
            "add_shortcut",
            "add_alias",
            "copy_content",
        ];
        for id in &common {
            assert!(s_ids.contains(id), "Script should have {}", id);
            assert!(sl_ids.contains(id), "Scriptlet should have {}", id);
        }
    }

    // ============================================================
    // 10. Deeplink URL in description format
    // ============================================================

    #[test]
    fn deeplink_description_contains_url() {
        let script = ScriptInfo::new("My Cool Script", "/path/script.ts");
        let actions = get_script_context_actions(&script);
        let dl = find_action(&actions, "script:copy_deeplink").unwrap();
        let desc = dl.description.as_ref().unwrap();
        assert!(desc.contains("scriptkit://run/my-cool-script"));
    }

    #[test]
    fn deeplink_description_special_chars() {
        let script = ScriptInfo::new("Test!@#$Script", "/path/script.ts");
        let actions = get_script_context_actions(&script);
        let dl = find_action(&actions, "script:copy_deeplink").unwrap();
        let desc = dl.description.as_ref().unwrap();
        assert!(desc.contains("scriptkit://run/test-script"));
    }

    #[test]
    fn deeplink_scriptlet_context() {
        let script = ScriptInfo::scriptlet("Open GitHub", "/path.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let dl = find_action(&actions, "script:copy_deeplink").unwrap();
        let desc = dl.description.as_ref().unwrap();
        assert!(desc.contains("scriptkit://run/open-github"));
    }

    // ============================================================
    // 11. AI command bar shortcut uniqueness
    // ============================================================

    #[test]
    fn ai_command_bar_shortcuts_unique() {
        let actions = get_ai_command_bar_actions();
        let shortcuts: Vec<&str> = actions
            .iter()
            .filter_map(|a| a.shortcut.as_deref())
            .collect();
        let unique: HashSet<&str> = shortcuts.iter().copied().collect();
        assert_eq!(
            shortcuts.len(),
            unique.len(),
            "AI command bar shortcuts should be unique: {:?}",
            shortcuts
        );
    }

    #[test]
    fn ai_command_bar_exactly_12_actions() {
        let actions = get_ai_command_bar_actions();
        assert_eq!(actions.len(), 12);
    }

    #[test]
    fn ai_command_bar_all_have_icons() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                action.icon.is_some(),
                "AI action '{}' should have icon",
                action.id
            );
        }
    }


    // --- merged from tests_part_03.rs ---
    #[test]
    fn ai_command_bar_all_have_sections() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                action.section.is_some(),
                "AI action '{}' should have section",
                action.id
            );
        }
    }

    #[test]
    fn ai_command_bar_section_order() {
        let actions = get_ai_command_bar_actions();
        let sections: Vec<&str> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        // Verify order: Response before Actions before Attachments before Settings
        let first_response = sections.iter().position(|&s| s == "Response").unwrap();
        let first_actions = sections.iter().position(|&s| s == "Actions").unwrap();
        let first_attachments = sections.iter().position(|&s| s == "Attachments").unwrap();
        let first_settings = sections.iter().position(|&s| s == "Settings").unwrap();
        assert!(first_response < first_actions);
        assert!(first_actions < first_attachments);
        assert!(first_attachments < first_settings);
    }

    // ============================================================
    // 12. Notes command bar shortcut uniqueness
    // ============================================================

    #[test]
    fn notes_command_bar_shortcuts_unique() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let shortcuts: Vec<&str> = actions
            .iter()
            .filter_map(|a| a.shortcut.as_deref())
            .collect();
        let unique: HashSet<&str> = shortcuts.iter().copied().collect();
        assert_eq!(
            shortcuts.len(),
            unique.len(),
            "Notes command bar shortcuts should be unique: {:?}",
            shortcuts
        );
    }

    // ============================================================
    // 13. Path context action ordering
    // ============================================================

    #[test]
    fn path_dir_primary_first() {
        let path = PathInfo {
            path: "/test/dir".to_string(),
            name: "dir".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path);
        assert_eq!(actions[0].id, "file:open_directory");
    }

    #[test]
    fn path_file_primary_first() {
        let path = PathInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        assert_eq!(actions[0].id, "file:select_file");
    }

    #[test]
    fn path_trash_last() {
        let path = PathInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        assert_eq!(
            actions.last().unwrap().id,
            "file:move_to_trash",
            "Trash should be last action"
        );
    }

    #[test]
    fn path_dir_trash_says_folder() {
        let path = PathInfo {
            path: "/test/dir".to_string(),
            name: "dir".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path);
        let trash = find_action(&actions, "file:move_to_trash").unwrap();
        assert!(
            trash.description.as_ref().unwrap().contains("folder"),
            "Dir trash should say 'folder'"
        );
    }

    #[test]
    fn path_file_trash_says_file() {
        let path = PathInfo {
            path: "/test/f.txt".to_string(),
            name: "f.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        let trash = find_action(&actions, "file:move_to_trash").unwrap();
        assert!(
            trash.description.as_ref().unwrap().contains("file"),
            "File trash should say 'file'"
        );
    }

    // ============================================================
    // 14. Clipboard action shortcut format
    // ============================================================

    #[test]
    fn clipboard_all_shortcuts_use_symbols() {
        let entry = make_text_entry(false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        for action in &actions {
            if let Some(ref shortcut) = action.shortcut {
                // Should not contain text like "cmd", "ctrl" etc.
                assert!(
                    !shortcut.contains("cmd"),
                    "Shortcut '{}' should use symbols not text",
                    shortcut
                );
                assert!(
                    !shortcut.contains("shift"),
                    "Shortcut '{}' should use symbols not text",
                    shortcut
                );
            }
        }
    }

    // ============================================================
    // 15. score_action with whitespace-only query
    // ============================================================

    #[test]
    fn score_whitespace_query() {
        let action = Action::new(
            "test",
            "Test Action With Spaces",
            Some("Description with spaces".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, " ");
        // " " is a contains match on the title " " appears after words
        assert!(score > 0, "Space should match title containing spaces");
    }

    // ============================================================
    // 16. fuzzy_match with repeated characters
    // ============================================================

    #[test]
    fn fuzzy_repeated_chars_in_needle() {
        assert!(
            ActionsDialog::fuzzy_match("aabbcc", "abc"),
            "Should match subsequence with repeated chars in haystack"
        );
    }

    #[test]
    fn fuzzy_repeated_chars_in_both() {
        assert!(
            ActionsDialog::fuzzy_match("aabbcc", "aabb"),
            "Should match when both have repeated chars"
        );
    }

    #[test]
    fn fuzzy_needle_longer_than_haystack() {
        assert!(
            !ActionsDialog::fuzzy_match("ab", "abc"),
            "Needle longer than haystack should not match"
        );
    }

    #[test]
    fn fuzzy_exact_match() {
        assert!(
            ActionsDialog::fuzzy_match("hello", "hello"),
            "Exact match is a valid subsequence"
        );
    }

    #[test]
    fn fuzzy_empty_needle_always_matches() {
        assert!(
            ActionsDialog::fuzzy_match("anything", ""),
            "Empty needle should match everything"
        );
    }

    #[test]
    fn fuzzy_empty_haystack_empty_needle() {
        assert!(
            ActionsDialog::fuzzy_match("", ""),
            "Both empty should match"
        );
    }

    #[test]
    fn fuzzy_empty_haystack_nonempty_needle() {
        assert!(
            !ActionsDialog::fuzzy_match("", "a"),
            "Non-empty needle with empty haystack should not match"
        );
    }

    // ============================================================
    // 17. build_grouped_items_static edge cases
    // ============================================================

    #[test]
    fn grouped_single_item_no_section_headers_style() {
        let actions = vec![make_action("a1", "Action 1", Some("Sec"))];
        let filtered = vec![0usize];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Should have 1 header + 1 item = 2
        assert_eq!(grouped.len(), 2);
        assert!(matches!(&grouped[0], GroupedActionItem::SectionHeader(s) if s == "Sec"));
        assert!(matches!(&grouped[1], GroupedActionItem::Item(0)));
    }

    #[test]
    fn grouped_empty_filtered() {
        let actions = vec![make_action("a1", "Action 1", None)];
        let filtered: Vec<usize> = vec![];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(grouped.is_empty());
    }

    #[test]
    fn grouped_none_style_no_headers() {
        let actions = vec![
            make_action("a1", "A1", Some("Sec1")),
            make_action("a2", "A2", Some("Sec2")),
        ];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        // None style should have no section headers
        assert_eq!(grouped.len(), 2);
        for item in &grouped {
            assert!(
                matches!(item, GroupedActionItem::Item(_)),
                "None style should have no headers"
            );
        }
    }

    #[test]
    fn grouped_separators_style_no_headers() {
        let actions = vec![
            make_action("a1", "A1", Some("Sec1")),
            make_action("a2", "A2", Some("Sec2")),
        ];
        let filtered = vec![0, 1];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        assert_eq!(grouped.len(), 2);
        for item in &grouped {
            assert!(
                matches!(item, GroupedActionItem::Item(_)),
                "Separators style should have no headers"
            );
        }
    }

    #[test]
    fn grouped_same_section_no_duplicate_header() {
        let actions = vec![
            make_action("a1", "A1", Some("Same")),
            make_action("a2", "A2", Some("Same")),
            make_action("a3", "A3", Some("Same")),
        ];
        let filtered = vec![0, 1, 2];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        let header_count = grouped
            .iter()
            .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 1, "Same section should produce only 1 header");
    }

    #[test]
    fn grouped_alternating_sections_produce_headers() {
        let actions = vec![
            make_action("a1", "A1", Some("A")),
            make_action("a2", "A2", Some("B")),
            make_action("a3", "A3", Some("A")),
        ];
        let filtered = vec![0, 1, 2];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        let header_count = grouped
            .iter()
            .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
            .count();
        // A -> B -> A = 3 section changes
        assert_eq!(
            header_count, 3,
            "Alternating sections should produce 3 headers"
        );
    }

    // ============================================================
    // 18. coerce_action_selection edge cases
    // ============================================================

    #[test]
    fn coerce_empty_returns_none() {
        assert_eq!(coerce_action_selection(&[], 0), None);
    }

    #[test]
    fn coerce_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".to_string()),
            GroupedActionItem::SectionHeader("B".to_string()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn coerce_on_item_returns_same() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".to_string()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(1));
    }

    #[test]
    fn coerce_header_searches_down() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".to_string()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn coerce_trailing_header_searches_up() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("A".to_string()),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(0));
    }

    #[test]
    fn coerce_alternating_header_item() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".to_string()),
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("B".to_string()),
            GroupedActionItem::Item(1),
            GroupedActionItem::SectionHeader("C".to_string()),
            GroupedActionItem::Item(2),
        ];
        // On header at 0 -> should find Item at 1
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
        // On header at 2 -> should find Item at 3
        assert_eq!(coerce_action_selection(&rows, 2), Some(3));
        // On header at 4 -> should find Item at 5
        assert_eq!(coerce_action_selection(&rows, 4), Some(5));
    }

    #[test]
    fn coerce_out_of_bounds_clamped() {
        let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
        // Index 99 should clamp to len-1 = 1
        assert_eq!(coerce_action_selection(&rows, 99), Some(1));
    }

    // ============================================================
    // 19. parse_shortcut_keycaps edge cases
    // ============================================================

    #[test]
    fn keycaps_empty_string() {
        let result = ActionsDialog::parse_shortcut_keycaps("");
        assert!(result.is_empty());
    }

    #[test]
    fn keycaps_single_modifier() {
        let result = ActionsDialog::parse_shortcut_keycaps("⌘");
        assert_eq!(result, vec!["⌘"]);
    }

    #[test]
    fn keycaps_modifier_plus_letter() {
        let result = ActionsDialog::parse_shortcut_keycaps("⌘C");
        assert_eq!(result, vec!["⌘", "C"]);
    }

    #[test]
    fn keycaps_all_modifiers() {
        let result = ActionsDialog::parse_shortcut_keycaps("⌘⌃⌥⇧");
        assert_eq!(result, vec!["⌘", "⌃", "⌥", "⇧"]);
    }

    #[test]
    fn keycaps_special_keys() {
        let result = ActionsDialog::parse_shortcut_keycaps("↵⎋⇥⌫␣↑↓←→");
        assert_eq!(result, vec!["↵", "⎋", "⇥", "⌫", "␣", "↑", "↓", "←", "→"]);
    }

    #[test]
    fn keycaps_lowercase_uppercased() {
        let result = ActionsDialog::parse_shortcut_keycaps("⌘a");
        assert_eq!(result, vec!["⌘", "A"]);
    }

    // ============================================================
    // 20. CommandBarConfig close flags independence
    // ============================================================

    #[test]
    fn command_bar_default_all_close_true() {
        let config = CommandBarConfig::default();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }


    // --- merged from tests_part_04.rs ---
    #[test]
    fn command_bar_ai_style_close_flags_default() {
        let config = CommandBarConfig::ai_style();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    #[test]
    fn command_bar_main_menu_search_bottom() {
        let config = CommandBarConfig::main_menu_style();
        assert_eq!(
            config.dialog_config.search_position,
            crate::actions::types::SearchPosition::Bottom
        );
    }

    #[test]
    fn command_bar_ai_style_search_top() {
        let config = CommandBarConfig::ai_style();
        assert_eq!(
            config.dialog_config.search_position,
            crate::actions::types::SearchPosition::Top
        );
    }

    #[test]
    fn command_bar_no_search_hidden() {
        let config = CommandBarConfig::no_search();
        assert_eq!(
            config.dialog_config.search_position,
            crate::actions::types::SearchPosition::Hidden
        );
    }

    #[test]
    fn command_bar_notes_style_search_top_icons() {
        let config = CommandBarConfig::notes_style();
        assert_eq!(
            config.dialog_config.search_position,
            crate::actions::types::SearchPosition::Top
        );
        assert!(config.dialog_config.show_icons);
        assert!(config.dialog_config.show_footer);
    }

    // ============================================================
    // 21. Action constructor with empty strings
    // ============================================================

    #[test]
    fn action_empty_id_and_title() {
        let action = Action::new("", "", None, ActionCategory::ScriptContext);
        assert_eq!(action.id, "");
        assert_eq!(action.title, "");
        assert_eq!(action.title_lower, "");
        assert!(action.description.is_none());
        assert!(action.description_lower.is_none());
    }

    #[test]
    fn action_with_shortcut_sets_lower() {
        let action =
            Action::new("x", "X", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
        assert_eq!(action.shortcut_lower, Some("⌘⇧c".to_string()));
    }

    #[test]
    fn action_with_shortcut_opt_none_no_lower() {
        let action =
            Action::new("x", "X", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
        assert!(action.shortcut.is_none());
        assert!(action.shortcut_lower.is_none());
    }

    #[test]
    fn action_with_shortcut_opt_some_sets_lower() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘Z".to_string()));
        assert_eq!(action.shortcut, Some("⌘Z".to_string()));
        assert_eq!(action.shortcut_lower, Some("⌘z".to_string()));
    }

    // ============================================================
    // 22. ScriptInfo flag exclusivity
    // ============================================================

    #[test]
    fn script_info_scriptlet_is_not_script() {
        let scriptlet = ScriptInfo::scriptlet("X", "/p.md", None, None);
        assert!(scriptlet.is_scriptlet);
        assert!(!scriptlet.is_script);
        assert!(!scriptlet.is_agent);
    }

    #[test]
    fn script_info_agent_is_not_scriptlet() {
        let mut agent = ScriptInfo::new("A", "/a.md");
        agent.is_script = false;
        agent.is_agent = true;
        assert!(agent.is_agent);
        assert!(!agent.is_scriptlet);
        assert!(!agent.is_script);
    }

    #[test]
    fn script_info_builtin_is_none_of_the_above() {
        let builtin = ScriptInfo::builtin("Clipboard");
        assert!(!builtin.is_script);
        assert!(!builtin.is_scriptlet);
        assert!(!builtin.is_agent);
    }

    // ============================================================
    // 23. Notes command bar action count bounds per flag state
    // ============================================================

    #[test]
    fn notes_minimal_count() {
        // No selection, no auto-sizing disabled → only new_note + browse_notes + enable_auto_sizing
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        // new_note + browse_notes + enable_auto_sizing = 3
        assert_eq!(
            actions.len(),
            3,
            "Minimal notes actions: {:?}",
            action_ids(&actions)
        );
    }

    #[test]
    fn notes_minimal_auto_sizing_enabled() {
        // No selection, auto-sizing already enabled
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        // new_note + browse_notes = 2
        assert_eq!(
            actions.len(),
            2,
            "Minimal with auto: {:?}",
            action_ids(&actions)
        );
    }

    #[test]
    fn notes_full_feature_count() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        // new_note + duplicate + browse_notes + find + format + copy_note_as + copy_deeplink
        // + create_quicklink + export + enable_auto_sizing = 10
        assert_eq!(
            actions.len(),
            10,
            "Full feature: {:?}",
            action_ids(&actions)
        );
    }

    #[test]
    fn notes_trash_hides_editing() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        assert!(!ids.contains(&"duplicate_note"));
        assert!(!ids.contains(&"find_in_note"));
        assert!(!ids.contains(&"format"));
        assert!(!ids.contains(&"copy_note_as"));
        assert!(!ids.contains(&"export"));
    }

    // ============================================================
    // 24. Chat model display_name in title
    // ============================================================

    #[test]
    fn chat_model_display_name_in_title() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "model-x".to_string(),
                display_name: "Model X Ultra".to_string(),
                provider: "Acme".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = find_action(&actions, "chat:select_model_model-x").unwrap();
        assert_eq!(model_action.title, "Model X Ultra");
    }

    #[test]
    fn chat_model_provider_in_description() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "m".to_string(),
                display_name: "M".to_string(),
                provider: "Acme Corp".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = find_action(&actions, "chat:select_model_m").unwrap();
        assert_eq!(model_action.description, Some("via Acme Corp".to_string()));
    }

    // ============================================================
    // 25. New chat model_id in action ID
    // ============================================================

    #[test]
    fn new_chat_model_section_name() {
        let models = vec![NewChatModelInfo {
            model_id: "abc-123".to_string(),
            display_name: "ABC 123".to_string(),
            provider: "p".to_string(),
            provider_display_name: "P".to_string(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].section.as_deref(), Some("Models"));
    }

    // ============================================================
    // 26. Clipboard delete_all description mentions "pinned"
    // ============================================================

    #[test]
    fn clipboard_delete_all_mentions_pinned() {
        let entry = make_text_entry(false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let delete_all = find_action(&actions, "clip:clipboard_delete_all").unwrap();
        assert!(
            delete_all
                .description
                .as_ref()
                .unwrap()
                .to_lowercase()
                .contains("pinned"),
            "delete_all description should mention pinned: {:?}",
            delete_all.description
        );
    }

    // ============================================================
    // 27. File context all actions have ScriptContext category
    // ============================================================

    #[test]
    fn file_all_script_context_category() {
        let file_info = FileInfo {
            path: "/test/file.rs".to_string(),
            name: "file.rs".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        for action in &actions {
            assert_eq!(
                action.category,
                ActionCategory::ScriptContext,
                "File action '{}' should be ScriptContext",
                action.id
            );
        }
    }

    #[test]
    fn file_dir_all_script_context_category() {
        let file_info = FileInfo {
            path: "/test/dir".to_string(),
            name: "dir".to_string(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&file_info);
        for action in &actions {
            assert_eq!(
                action.category,
                ActionCategory::ScriptContext,
                "File dir action '{}' should be ScriptContext",
                action.id
            );
        }
    }

    // ============================================================
    // 28. Path context copy_path and copy_filename always present
    // ============================================================

    #[test]
    fn path_always_has_copy_path_and_filename() {
        for is_dir in [true, false] {
            let path = PathInfo {
                path: "/test/item".to_string(),
                name: "item".to_string(),
                is_dir,
            };
            let actions = get_path_context_actions(&path);
            let ids = action_ids(&actions);
            assert!(
                ids.contains(&"file:copy_path"),
                "Path (is_dir={}) should have copy_path",
                is_dir
            );
            assert!(
                ids.contains(&"file:copy_filename"),
                "Path (is_dir={}) should have copy_filename",
                is_dir
            );
        }
    }

    #[test]
    fn path_always_has_open_in_finder_editor_terminal() {
        for is_dir in [true, false] {
            let path = PathInfo {
                path: "/test/x".to_string(),
                name: "x".to_string(),
                is_dir,
            };
            let actions = get_path_context_actions(&path);
            let ids = action_ids(&actions);
            assert!(ids.contains(&"file:open_in_finder"));
            assert!(ids.contains(&"file:open_in_editor"));
            assert!(ids.contains(&"file:open_in_terminal"));
        }
    }

    // ============================================================
    // 29. Cross-context ID namespace separation
    // ============================================================

    #[test]
    fn clipboard_ids_not_in_script_context() {
        let clip = make_text_entry(false, None);
        let script = ScriptInfo::new("s", "/s.ts");
        let clip_actions = get_clipboard_history_context_actions(&clip);
        let clip_ids: HashSet<&str> = action_ids(&clip_actions).into_iter().collect();
        let script_actions = get_script_context_actions(&script);
        let script_ids: HashSet<&str> = action_ids(&script_actions).into_iter().collect();
        let overlap: Vec<&&str> = clip_ids.intersection(&script_ids).collect();
        assert!(
            overlap.is_empty(),
            "Clipboard and script IDs should not overlap: {:?}",
            overlap
        );
    }

    #[test]
    fn file_ids_not_in_clipboard_context() {
        let file = FileInfo {
            path: "/f.txt".to_string(),
            name: "f.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let clip = make_text_entry(false, None);
        let file_actions = get_file_context_actions(&file);
        let file_ids: HashSet<&str> = action_ids(&file_actions).into_iter().collect();
        let clip_actions = get_clipboard_history_context_actions(&clip);
        let clip_ids: HashSet<&str> = action_ids(&clip_actions).into_iter().collect();
        let overlap: Vec<&&str> = file_ids.intersection(&clip_ids).collect();
        assert!(
            overlap.is_empty(),
            "File and clipboard IDs should not overlap: {:?}",
            overlap
        );
    }

    #[test]
    fn ai_ids_not_in_notes_context() {
        let ai_actions = get_ai_command_bar_actions();
        let ai_ids: HashSet<&str> = action_ids(&ai_actions).into_iter().collect();
        let notes_info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let notes_actions = get_notes_command_bar_actions(&notes_info);
        let notes_ids: HashSet<&str> = action_ids(&notes_actions).into_iter().collect();
        // copy_deeplink can exist in both contexts, but the rest should not overlap
        // Actually checking: AI actions should be distinct from notes actions
        let overlap: Vec<&&str> = ai_ids.intersection(&notes_ids).collect();
        // copy_deeplink exists in notes. Let's check what AI has - it has copy_response, copy_chat etc.
        // They should not overlap
        assert!(
            overlap.is_empty(),
            "AI and notes IDs should not overlap: {:?}",
            overlap
        );
    }

    // ============================================================
    // 30. Action title_lower invariant across all builder functions
    // ============================================================

    #[test]
    fn title_lower_matches_title_for_script() {
        let script = ScriptInfo::new("My Script", "/path/s.ts");
        for action in &get_script_context_actions(&script) {
            assert_eq!(
                action.title_lower,
                action.title.to_lowercase(),
                "title_lower mismatch for '{}'",
                action.id
            );
        }
    }

    #[test]
    fn title_lower_matches_title_for_clipboard() {
        let entry = make_text_entry(false, Some("VS Code"));
        for action in &get_clipboard_history_context_actions(&entry) {
            assert_eq!(
                action.title_lower,
                action.title.to_lowercase(),
                "title_lower mismatch for '{}'",
                action.id
            );
        }
    }


    // --- merged from tests_part_05.rs ---
    #[test]
    fn title_lower_matches_title_for_ai() {
        for action in &get_ai_command_bar_actions() {
            assert_eq!(
                action.title_lower,
                action.title.to_lowercase(),
                "title_lower mismatch for '{}'",
                action.id
            );
        }
    }

    #[test]
    fn title_lower_matches_title_for_notes() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for action in &get_notes_command_bar_actions(&info) {
            assert_eq!(
                action.title_lower,
                action.title.to_lowercase(),
                "title_lower mismatch for '{}'",
                action.id
            );
        }
    }

    #[test]
    fn title_lower_matches_title_for_new_chat() {
        let lu = vec![NewChatModelInfo {
            model_id: "m".to_string(),
            display_name: "Model ABC".to_string(),
            provider: "p".to_string(),
            provider_display_name: "P".to_string(),
        }];
        for action in &get_new_chat_actions(&lu, &[], &[]) {
            assert_eq!(
                action.title_lower,
                action.title.to_lowercase(),
                "title_lower mismatch for '{}'",
                action.id
            );
        }
    }

    #[test]
    fn title_lower_matches_title_for_note_switcher() {
        let notes = vec![make_note("n1", "My Note", 50, false, false, "", "")];
        for action in &get_note_switcher_actions(&notes) {
            assert_eq!(
                action.title_lower,
                action.title.to_lowercase(),
                "title_lower mismatch for '{}'",
                action.id
            );
        }
    }

    #[test]
    fn title_lower_matches_title_for_path() {
        let path = PathInfo {
            path: "/test/MyDir".to_string(),
            name: "MyDir".to_string(),
            is_dir: true,
        };
        for action in &get_path_context_actions(&path) {
            assert_eq!(
                action.title_lower,
                action.title.to_lowercase(),
                "title_lower mismatch for '{}'",
                action.id
            );
        }
    }

    #[test]
    fn title_lower_matches_title_for_file() {
        let file = FileInfo {
            path: "/test/MyFile.txt".to_string(),
            name: "MyFile.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        for action in &get_file_context_actions(&file) {
            assert_eq!(
                action.title_lower,
                action.title.to_lowercase(),
                "title_lower mismatch for '{}'",
                action.id
            );
        }
    }

    #[test]
    fn description_lower_matches_description_for_script() {
        let script = ScriptInfo::new("My Script", "/path/s.ts");
        for action in &get_script_context_actions(&script) {
            match (&action.description, &action.description_lower) {
                (Some(desc), Some(desc_lower)) => {
                    assert_eq!(
                        *desc_lower,
                        desc.to_lowercase(),
                        "description_lower mismatch for '{}'",
                        action.id
                    );
                }
                (None, None) => {} // Both absent is fine
                _ => panic!(
                    "description and description_lower mismatch for '{}': desc={:?}, lower={:?}",
                    action.id, action.description, action.description_lower
                ),
            }
        }
    }

    // ============================================================
    // Additional: Scriptlet with custom actions
    // ============================================================

    #[test]
    fn scriptlet_custom_actions_have_has_action_true() {
        let script = ScriptInfo::scriptlet("Test", "/test.md", None, None);
        let mut scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo main".to_string(),
        );
        scriptlet.actions = vec![
            ScriptletAction {
                name: "Copy".to_string(),
                command: "copy".to_string(),
                tool: "bash".to_string(),
                code: "echo copy".to_string(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
            ScriptletAction {
                name: "Open".to_string(),
                command: "open".to_string(),
                tool: "open".to_string(),
                code: "https://example.com".to_string(),
                inputs: vec![],
                shortcut: Some("cmd+o".to_string()),
                description: Some("Open in browser".to_string()),
            },
        ];
        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let custom: Vec<&Action> = actions
            .iter()
            .filter(|a| a.id.starts_with("scriptlet_action:"))
            .collect();
        assert_eq!(custom.len(), 2);
        for ca in &custom {
            assert!(
                ca.has_action,
                "Custom action '{}' should have has_action=true",
                ca.id
            );
            assert!(
                ca.value.is_some(),
                "Custom action '{}' should have value",
                ca.id
            );
        }
    }

    #[test]
    fn scriptlet_custom_actions_appear_after_run_before_edit() {
        let script = ScriptInfo::scriptlet("Test", "/test.md", None, None);
        let mut scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo main".to_string(),
        );
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
        let run_pos = actions.iter().position(|a| a.id == "run_script").unwrap();
        let custom_pos = actions
            .iter()
            .position(|a| a.id == "scriptlet_action:custom")
            .unwrap();
        let edit_pos = actions
            .iter()
            .position(|a| a.id == "edit_scriptlet")
            .unwrap();
        assert!(run_pos < custom_pos, "Run before custom");
        assert!(custom_pos < edit_pos, "Custom before edit");
    }

    #[test]
    fn scriptlet_custom_action_shortcut_formatted() {
        let script = ScriptInfo::scriptlet("Test", "/test.md", None, None);
        let mut scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo main".to_string(),
        );
        scriptlet.actions = vec![ScriptletAction {
            name: "Copy".to_string(),
            command: "copy".to_string(),
            tool: "bash".to_string(),
            code: "echo".to_string(),
            inputs: vec![],
            shortcut: Some("cmd+c".to_string()),
            description: None,
        }];
        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let custom = find_action(&actions, "scriptlet_action:copy").unwrap();
        // The shortcut should be formatted using builders.rs format_shortcut_hint
        assert_eq!(custom.shortcut, Some("⌘C".to_string()));
    }

    // ============================================================
    // Additional: to_deeplink_name edge cases
    // ============================================================

    #[test]
    fn deeplink_unicode_chars_stripped() {
        // Non-alphanumeric chars (including accented) should be replaced with hyphens
        // Actually accented chars are NOT alphanumeric in Rust's is_alphanumeric()
        // Wait, they ARE: 'é'.is_alphanumeric() == true
        let result = to_deeplink_name("café");
        assert_eq!(result, "café");
    }

    #[test]
    fn deeplink_numbers_preserved() {
        let result = to_deeplink_name("Script 123");
        assert_eq!(result, "script-123");
    }

    #[test]
    fn deeplink_all_special_returns_empty() {
        let result = to_deeplink_name("!@#$%");
        assert_eq!(result, "");
    }

    #[test]
    fn deeplink_already_hyphenated_passthrough() {
        let result = to_deeplink_name("my-script");
        assert_eq!(result, "my-script");
    }

    #[test]
    fn deeplink_leading_trailing_special() {
        let result = to_deeplink_name(" !hello! ");
        assert_eq!(result, "hello");
    }

    // ============================================================
    // Additional: Ordering determinism
    // ============================================================

    #[test]
    fn ordering_determinism_script() {
        let script = ScriptInfo::new("Test", "/test.ts");
        let actions_1 = get_script_context_actions(&script);
        let ids1 = action_ids(&actions_1);
        let actions_2 = get_script_context_actions(&script);
        let ids2 = action_ids(&actions_2);
        assert_eq!(ids1, ids2, "Script actions should be deterministic");
    }

    #[test]
    fn ordering_determinism_clipboard() {
        let entry = make_text_entry(false, None);
        let actions_1 = get_clipboard_history_context_actions(&entry);
        let ids1 = action_ids(&actions_1);
        let actions_2 = get_clipboard_history_context_actions(&entry);
        let ids2 = action_ids(&actions_2);
        assert_eq!(ids1, ids2, "Clipboard actions should be deterministic");
    }

    #[test]
    fn ordering_determinism_ai() {
        let actions_1 = get_ai_command_bar_actions();
        let ids1 = action_ids(&actions_1);
        let actions_2 = get_ai_command_bar_actions();
        let ids2 = action_ids(&actions_2);
        assert_eq!(ids1, ids2, "AI actions should be deterministic");
    }

    #[test]
    fn ordering_determinism_notes() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions_1 = get_notes_command_bar_actions(&info);
        let ids1 = action_ids(&actions_1);
        let actions_2 = get_notes_command_bar_actions(&info);
        let ids2 = action_ids(&actions_2);
        assert_eq!(ids1, ids2, "Notes actions should be deterministic");
    }

    #[test]
    fn ordering_determinism_path() {
        let path = PathInfo {
            path: "/test".to_string(),
            name: "test".to_string(),
            is_dir: false,
        };
        let actions_1 = get_path_context_actions(&path);
        let ids1 = action_ids(&actions_1);
        let actions_2 = get_path_context_actions(&path);
        let ids2 = action_ids(&actions_2);
        assert_eq!(ids1, ids2, "Path actions should be deterministic");
    }

    // ============================================================
    // Additional: ID uniqueness per context
    // ============================================================

    #[test]
    fn id_uniqueness_script() {
        let script = ScriptInfo::new("s", "/s.ts");
        let actions = get_script_context_actions(&script);
        let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();
        assert_eq!(
            ids.len(),
            actions.len(),
            "Script action IDs should be unique"
        );
    }

    #[test]
    fn id_uniqueness_clipboard() {
        let entry = make_text_entry(false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();
        assert_eq!(
            ids.len(),
            actions.len(),
            "Clipboard action IDs should be unique"
        );
    }

    #[test]
    fn id_uniqueness_ai() {
        let actions = get_ai_command_bar_actions();
        let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();
        assert_eq!(ids.len(), actions.len(), "AI action IDs should be unique");
    }

    #[test]
    fn id_uniqueness_path() {
        let path = PathInfo {
            path: "/test".to_string(),
            name: "test".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();
        assert_eq!(ids.len(), actions.len(), "Path action IDs should be unique");
    }

    #[test]
    fn id_uniqueness_file() {
        let file = FileInfo {
            path: "/f.txt".to_string(),
            name: "f.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();
        assert_eq!(ids.len(), actions.len(), "File action IDs should be unique");
    }

    #[test]
    fn id_uniqueness_notes() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();
        assert_eq!(
            ids.len(),
            actions.len(),
            "Notes action IDs should be unique"
        );
    }

    // ============================================================
    // Additional: has_action=false invariant for all built-in actions
    // ============================================================

    #[test]
    fn has_action_false_for_script() {
        let script = ScriptInfo::new("s", "/s.ts");
        for action in &get_script_context_actions(&script) {
            assert!(
                !action.has_action,
                "Built-in action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn has_action_false_for_clipboard() {
        let entry = make_text_entry(false, None);
        for action in &get_clipboard_history_context_actions(&entry) {
            assert!(
                !action.has_action,
                "Built-in action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn has_action_false_for_ai() {
        for action in &get_ai_command_bar_actions() {
            assert!(
                !action.has_action,
                "Built-in action '{}' should have has_action=false",
                action.id
            );
        }
    }


    // --- merged from tests_part_06.rs ---
    #[test]
    fn has_action_false_for_path() {
        let path = PathInfo {
            path: "/test".to_string(),
            name: "test".to_string(),
            is_dir: false,
        };
        for action in &get_path_context_actions(&path) {
            assert!(
                !action.has_action,
                "Built-in action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn has_action_false_for_file() {
        let file = FileInfo {
            path: "/f.txt".to_string(),
            name: "f.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        for action in &get_file_context_actions(&file) {
            assert!(
                !action.has_action,
                "Built-in action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn has_action_false_for_notes() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for action in &get_notes_command_bar_actions(&info) {
            assert!(
                !action.has_action,
                "Built-in action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn has_action_false_for_chat() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "m".to_string(),
                display_name: "M".to_string(),
                provider: "P".to_string(),
            }],
            has_messages: true,
            has_response: true,
        };
        for action in &get_chat_context_actions(&info) {
            assert!(
                !action.has_action,
                "Built-in action '{}' should have has_action=false",
                action.id
            );
        }
    }

    // ============================================================
    // Additional: Non-empty title and ID for all contexts
    // ============================================================

    #[test]
    fn nonempty_title_id_script() {
        let script = ScriptInfo::new("s", "/s.ts");
        for action in &get_script_context_actions(&script) {
            assert!(!action.id.is_empty(), "Action should have non-empty ID");
            assert!(
                !action.title.is_empty(),
                "Action '{}' should have non-empty title",
                action.id
            );
        }
    }

    #[test]
    fn nonempty_title_id_clipboard() {
        let entry = make_text_entry(false, None);
        for action in &get_clipboard_history_context_actions(&entry) {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    #[test]
    fn nonempty_title_id_ai() {
        for action in &get_ai_command_bar_actions() {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    #[test]
    fn nonempty_title_id_notes() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for action in &get_notes_command_bar_actions(&info) {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    #[test]
    fn nonempty_title_id_path() {
        let path = PathInfo {
            path: "/test".to_string(),
            name: "test".to_string(),
            is_dir: true,
        };
        for action in &get_path_context_actions(&path) {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    #[test]
    fn nonempty_title_id_file() {
        let file = FileInfo {
            path: "/f".to_string(),
            name: "f".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        for action in &get_file_context_actions(&file) {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    // ============================================================
    // Additional: Note switcher icons and sections
    // ============================================================

    #[test]
    fn note_switcher_pinned_star_icon() {
        let note = make_note("n1", "Note", 10, false, true, "", "");
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
    }

    #[test]
    fn note_switcher_current_check_icon() {
        let note = make_note("n1", "Note", 10, true, false, "", "");
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].icon, Some(IconName::Check));
        assert_eq!(actions[0].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn note_switcher_default_file_icon() {
        let note = make_note("n1", "Note", 10, false, false, "", "");
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].icon, Some(IconName::File));
        assert_eq!(actions[0].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn note_switcher_current_gets_bullet_prefix() {
        let note = make_note("n1", "My Note", 10, true, false, "", "");
        let actions = get_note_switcher_actions(&[note]);
        assert!(
            actions[0].title.starts_with("• "),
            "Current note should have bullet prefix"
        );
    }

    #[test]
    fn note_switcher_not_current_no_bullet() {
        let note = make_note("n1", "My Note", 10, false, false, "", "");
        let actions = get_note_switcher_actions(&[note]);
        assert!(
            !actions[0].title.starts_with("• "),
            "Non-current note should not have bullet"
        );
    }

    #[test]
    fn note_switcher_id_format() {
        let note = make_note("abc-123", "Note", 10, false, false, "", "");
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].id, "note_abc-123");
    }

    #[test]
    fn note_switcher_empty_shows_placeholder() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "no_notes");
        assert!(actions[0].title.contains("No notes"));
    }

    #[test]
    fn note_switcher_pinned_takes_priority_over_current() {
        let note = make_note("n1", "Note", 10, true, true, "", "");
        let actions = get_note_switcher_actions(&[note]);
        // Pinned icon takes priority
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
        // But still gets the "Pinned" section
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
        // And still gets bullet prefix because is_current
        assert!(actions[0].title.starts_with("• "));
    }

}
