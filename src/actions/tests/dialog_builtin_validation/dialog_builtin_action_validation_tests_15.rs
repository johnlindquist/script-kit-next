//! Batch 15 – Dialog Builtin Action Validation Tests
//!
//! 30 categories, ~170 tests covering fresh angles:
//! - to_deeplink_name with non-Latin Unicode (Arabic, Thai, Devanagari)
//! - Clipboard image macOS-exclusive actions
//! - Notes combined-flag interactions
//! - Chat context boundary states
//! - New chat section guarantees
//! - Note switcher description fallback hierarchy
//! - AI command bar per-section ID enumeration
//! - Action builder overwrite semantics
//! - CommandBarConfig preset field comparison matrix
//! - Cross-context category uniformity
//! - Clipboard exact action counts on macOS
//! - Path primary-action insertion position
//! - File title quoting
//! - ScriptInfo::with_all field completeness
//! - Ordering idempotency (double-call determinism)

#[cfg(test)]
mod tests {
    // --- merged from tests_part_01.rs ---
    use super::super::builders::*;
    use super::super::command_bar::CommandBarConfig;
    use super::super::types::*;
    use crate::clipboard_history::ContentType;
    use crate::designs::icon_variations::IconName;
    use crate::file_search::FileInfo;
    use crate::prompts::PathInfo;
    use std::collections::HashSet;

    fn action_ids(actions: &[Action]) -> Vec<String> {
        actions.iter().map(|a| a.id.clone()).collect()
    }

    // =========================================================================
    // cat01: to_deeplink_name with non-Latin Unicode scripts
    // =========================================================================

    #[test]
    fn cat01_deeplink_arabic_preserved() {
        // Arabic alphanumeric chars should pass is_alphanumeric()
        let result = to_deeplink_name("مرحبا");
        assert!(!result.is_empty(), "Arabic should be preserved");
        assert!(
            result.chars().all(|c| c.is_alphanumeric() || c == '-'),
            "Result should only have alphanumeric or hyphens: {}",
            result
        );
    }

    #[test]
    fn cat01_deeplink_thai_preserved() {
        let result = to_deeplink_name("สวัสดี");
        assert!(!result.is_empty(), "Thai should be preserved");
    }

    #[test]
    fn cat01_deeplink_devanagari_preserved() {
        let result = to_deeplink_name("नमस्ते");
        assert!(!result.is_empty(), "Devanagari should be preserved");
    }

    #[test]
    fn cat01_deeplink_mixed_scripts() {
        // "Hello-مرحبا" — mixed Latin and Arabic
        let result = to_deeplink_name("Hello مرحبا");
        assert!(result.contains("hello"), "Latin part lowercased");
        // Arabic and Latin separated by space → hyphen
        assert!(result.contains('-'), "Space becomes hyphen");
    }

    #[test]
    fn cat01_deeplink_empty_string() {
        assert_eq!(to_deeplink_name(""), "");
    }

    #[test]
    fn cat01_deeplink_only_specials() {
        assert_eq!(to_deeplink_name("!@#$%^&*()"), "");
    }

    #[test]
    fn cat01_deeplink_single_char() {
        assert_eq!(to_deeplink_name("a"), "a");
    }

    // =========================================================================
    // cat02: Clipboard image macOS-exclusive action set
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn cat02_clipboard_image_macos_has_open_with() {
        let entry = ClipboardEntryInfo {
            id: "img-1".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"clipboard_open_with".to_string()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat02_clipboard_image_macos_has_annotate_cleanshot() {
        let entry = ClipboardEntryInfo {
            id: "img-2".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: Some((200, 200)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"clipboard_annotate_cleanshot".to_string()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat02_clipboard_image_macos_has_upload_cleanshot() {
        let entry = ClipboardEntryInfo {
            id: "img-3".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"clipboard_upload_cleanshot".to_string()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat02_clipboard_text_macos_no_annotate() {
        let entry = ClipboardEntryInfo {
            id: "txt-1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        assert!(!ids.contains(&"clipboard_annotate_cleanshot".to_string()));
        assert!(!ids.contains(&"clipboard_upload_cleanshot".to_string()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat02_clipboard_image_has_ocr_text_does_not() {
        let img = ClipboardEntryInfo {
            id: "i1".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let txt = ClipboardEntryInfo {
            id: "t1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let img_ids = action_ids(&get_clipboard_history_context_actions(&img));
        let txt_ids = action_ids(&get_clipboard_history_context_actions(&txt));
        assert!(img_ids.contains(&"clipboard_ocr".to_string()));
        assert!(!txt_ids.contains(&"clipboard_ocr".to_string()));
    }

    // =========================================================================
    // cat03: Notes combined-flag interactions
    // =========================================================================

    #[test]
    fn cat03_notes_all_true_trash_blocks_selection() {
        // has_selection=true + is_trash_view=true → selection-dependent actions hidden
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        // Trash view blocks: duplicate, find, format, copy_note_as, copy_deeplink, create_quicklink, export
        assert!(!ids.contains(&"duplicate_note".to_string()));
        assert!(!ids.contains(&"find_in_note".to_string()));
        assert!(!ids.contains(&"export".to_string()));
    }

    #[test]
    fn cat03_notes_no_selection_no_trash() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        // Always present: new_note, browse_notes
        assert!(ids.contains(&"new_note".to_string()));
        assert!(ids.contains(&"browse_notes".to_string()));
        // No selection → no duplicate, find, format, etc.
        assert!(!ids.contains(&"duplicate_note".to_string()));
        // auto_sizing disabled → enable_auto_sizing present
        assert!(ids.contains(&"enable_auto_sizing".to_string()));
    }

    #[test]
    fn cat03_notes_full_feature_set() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Should have max actions: new_note, duplicate, browse, find, format,
        // copy_note_as, copy_deeplink, create_quicklink, export, enable_auto_sizing
        assert_eq!(actions.len(), 10, "Full feature set should be 10 actions");
    }

    #[test]
    fn cat03_notes_auto_sizing_enabled_hides_action() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        assert!(!ids.contains(&"enable_auto_sizing".to_string()));
        // Full set minus enable_auto_sizing = 9
        assert_eq!(actions.len(), 9);
    }

    #[test]
    fn cat03_notes_trash_view_minimal() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Only new_note and browse_notes (auto_sizing enabled, so no enable_auto_sizing)
        assert_eq!(actions.len(), 2);
    }

    // =========================================================================
    // cat04: Chat context boundary states
    // =========================================================================

    #[test]
    fn cat04_chat_zero_models_both_flags_false() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        // Only continue_in_chat
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "continue_in_chat");
    }

    #[test]
    fn cat04_chat_zero_models_both_flags_true() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        // continue_in_chat + copy_response + clear_conversation = 3
        assert_eq!(actions.len(), 3);
    }

    #[test]
    fn cat04_chat_model_id_format() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "gpt-4o".into(),
                display_name: "GPT-4o".into(),
                provider: "OpenAI".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions[0].id, "select_model_gpt-4o");
    }

    #[test]
    fn cat04_chat_current_model_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("Claude 3.5".into()),
            available_models: vec![
                ChatModelInfo {
                    id: "claude-35".into(),
                    display_name: "Claude 3.5".into(),
                    provider: "Anthropic".into(),
                },
                ChatModelInfo {
                    id: "gpt-4".into(),
                    display_name: "GPT-4".into(),
                    provider: "OpenAI".into(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(
            actions[0].title.contains("✓"),
            "Current model gets checkmark"
        );
        assert!(
            !actions[1].title.contains("✓"),
            "Non-current model no checkmark"
        );
    }

    #[test]
    fn cat04_chat_continue_shortcut() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions[0].shortcut.as_deref(), Some("⌘↵"));
    }

    #[test]
    fn cat04_chat_model_description_via_provider() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "m1".into(),
                display_name: "Model One".into(),
                provider: "Acme Corp".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions[0].description.as_deref(), Some("via Acme Corp"));
    }

    // =========================================================================
    // cat05: New chat section ordering guarantees
    // =========================================================================

    #[test]
    fn cat05_new_chat_empty_all_sections() {
        let actions = get_new_chat_actions(&[], &[], &[]);
        assert!(actions.is_empty());
    }

    #[test]
    fn cat05_new_chat_section_ordering() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "Model 1".into(),
            provider: "P".into(),
            provider_display_name: "Provider".into(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m2".into(),
            display_name: "Model 2".into(),
            provider: "P".into(),
            provider_display_name: "Provider".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        assert_eq!(actions.len(), 3);
        assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
        assert_eq!(actions[1].section.as_deref(), Some("Presets"));
        assert_eq!(actions[2].section.as_deref(), Some("Models"));
    }

    #[test]
    fn cat05_new_chat_preset_no_description() {
        let presets = vec![NewChatPresetInfo {
            id: "code".into(),
            name: "Code".into(),
            icon: IconName::Code,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].description, None);
    }

    #[test]
    fn cat05_new_chat_model_has_provider_description() {
        let models = vec![NewChatModelInfo {
            model_id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "openai".into(),
            provider_display_name: "OpenAI".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].description.as_deref(), Some("OpenAI"));
    }

    #[test]
    fn cat05_new_chat_last_used_icon_bolt() {
        let last_used = vec![NewChatModelInfo {
            model_id: "x".into(),
            display_name: "X".into(),
            provider: "P".into(),
            provider_display_name: "PP".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
    }

    #[test]
    fn cat05_new_chat_model_icon_settings() {
        let models = vec![NewChatModelInfo {
            model_id: "m".into(),
            display_name: "M".into(),
            provider: "P".into(),
            provider_display_name: "PP".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].icon, Some(IconName::Settings));
    }

    // =========================================================================
    // cat06: Note switcher description fallback hierarchy
    // =========================================================================

    #[test]
    fn cat06_note_switcher_preview_and_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "Test".into(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "Hello world".into(),
            relative_time: "5m ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_deref().unwrap();
        assert!(desc.contains("Hello world"));
        assert!(desc.contains("5m ago"));
        assert!(desc.contains("·"), "Separator between preview and time");
    }


    // --- merged from tests_part_02.rs ---
    #[test]
    fn cat06_note_switcher_preview_no_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n2".into(),
            title: "Test".into(),
            char_count: 50,
            is_current: false,
            is_pinned: false,
            preview: "Some content".into(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_deref().unwrap();
        assert_eq!(desc, "Some content");
        assert!(!desc.contains("·"));
    }

    #[test]
    fn cat06_note_switcher_no_preview_with_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n3".into(),
            title: "Test".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: "1h ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_deref().unwrap();
        assert_eq!(desc, "1h ago");
    }

    #[test]
    fn cat06_note_switcher_no_preview_no_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n4".into(),
            title: "Test".into(),
            char_count: 42,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_deref().unwrap();
        assert_eq!(desc, "42 chars");
    }

    #[test]
    fn cat06_note_switcher_singular_char() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n5".into(),
            title: "T".into(),
            char_count: 1,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_deref().unwrap();
        assert_eq!(desc, "1 char");
    }

    #[test]
    fn cat06_note_switcher_zero_chars() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n6".into(),
            title: "Empty".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_deref().unwrap();
        assert_eq!(desc, "0 chars");
    }

    #[test]
    fn cat06_note_switcher_preview_exactly_60_no_ellipsis() {
        let preview: String = "a".repeat(60);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n7".into(),
            title: "T".into(),
            char_count: 60,
            is_current: false,
            is_pinned: false,
            preview,
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_deref().unwrap();
        assert!(!desc.contains('…'), "60 chars should not be truncated");
    }

    #[test]
    fn cat06_note_switcher_preview_61_has_ellipsis() {
        let preview: String = "b".repeat(61);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n8".into(),
            title: "T".into(),
            char_count: 61,
            is_current: false,
            is_pinned: false,
            preview,
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_deref().unwrap();
        assert!(desc.contains('…'), "61 chars should be truncated with …");
    }

    // =========================================================================
    // cat07: AI command bar per-section ID enumeration
    // =========================================================================

    #[test]
    fn cat07_ai_response_section_ids() {
        let actions = get_ai_command_bar_actions();
        let response_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Response"))
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(
            response_ids,
            vec!["copy_response", "copy_chat", "copy_last_code"]
        );
    }

    #[test]
    fn cat07_ai_actions_section_ids() {
        let actions = get_ai_command_bar_actions();
        let action_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Actions"))
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(
            action_ids,
            vec!["submit", "new_chat", "delete_chat", "branch_from_last"]
        );
    }

    #[test]
    fn cat07_ai_attachments_section_ids() {
        let actions = get_ai_command_bar_actions();
        let att_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Attachments"))
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(att_ids, vec!["add_attachment", "paste_image"]);
    }

    #[test]
    fn cat07_ai_export_section_ids() {
        let actions = get_ai_command_bar_actions();
        let export_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Export"))
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(export_ids, vec!["export_markdown"]);
    }

    #[test]
    fn cat07_ai_help_section_ids() {
        let actions = get_ai_command_bar_actions();
        let help_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Help"))
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(help_ids, vec!["toggle_shortcuts_help"]);
    }

    #[test]
    fn cat07_ai_settings_section_ids() {
        let actions = get_ai_command_bar_actions();
        let settings_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Settings"))
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(settings_ids, vec!["change_model"]);
    }

    // =========================================================================
    // cat08: Action builder overwrite semantics
    // =========================================================================

    #[test]
    fn cat08_with_shortcut_overwrites_previous() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘A")
            .with_shortcut("⌘B");
        assert_eq!(action.shortcut.as_deref(), Some("⌘B"));
    }

    #[test]
    fn cat08_with_icon_overwrites_previous() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_icon(IconName::Star)
            .with_icon(IconName::Trash);
        assert_eq!(action.icon, Some(IconName::Trash));
    }

    #[test]
    fn cat08_with_section_overwrites_previous() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_section("A")
            .with_section("B");
        assert_eq!(action.section.as_deref(), Some("B"));
    }

    #[test]
    fn cat08_with_shortcut_opt_none_preserves() {
        // with_shortcut_opt(None) does NOT clear existing shortcut
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘A")
            .with_shortcut_opt(None);
        assert_eq!(
            action.shortcut.as_deref(),
            Some("⌘A"),
            "None does not clear existing shortcut"
        );
    }

    #[test]
    fn cat08_with_shortcut_opt_some_sets() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘Z".to_string()));
        assert_eq!(action.shortcut.as_deref(), Some("⌘Z"));
    }

    // =========================================================================
    // cat09: CommandBarConfig preset field comparison matrix
    // =========================================================================

    #[test]
    fn cat09_default_vs_ai_style() {
        let def = CommandBarConfig::default();
        let ai = CommandBarConfig::ai_style();
        // AI style uses Headers, default uses Separators
        assert_eq!(ai.dialog_config.section_style, SectionStyle::Headers);
        assert_eq!(def.dialog_config.section_style, SectionStyle::Separators);
    }

    #[test]
    fn cat09_notes_style_search_top() {
        let notes = CommandBarConfig::notes_style();
        assert_eq!(notes.dialog_config.search_position, SearchPosition::Top);
    }

    #[test]
    fn cat09_no_search_hidden() {
        let ns = CommandBarConfig::no_search();
        assert_eq!(ns.dialog_config.search_position, SearchPosition::Hidden);
    }

    #[test]
    fn cat09_main_menu_bottom() {
        let mm = CommandBarConfig::main_menu_style();
        assert_eq!(mm.dialog_config.search_position, SearchPosition::Bottom);
    }

    #[test]
    fn cat09_all_presets_close_on_select_true() {
        assert!(CommandBarConfig::default().close_on_select);
        assert!(CommandBarConfig::ai_style().close_on_select);
        assert!(CommandBarConfig::main_menu_style().close_on_select);
        assert!(CommandBarConfig::no_search().close_on_select);
        assert!(CommandBarConfig::notes_style().close_on_select);
    }

    #[test]
    fn cat09_all_presets_close_on_escape_true() {
        assert!(CommandBarConfig::default().close_on_escape);
        assert!(CommandBarConfig::ai_style().close_on_escape);
        assert!(CommandBarConfig::main_menu_style().close_on_escape);
        assert!(CommandBarConfig::no_search().close_on_escape);
        assert!(CommandBarConfig::notes_style().close_on_escape);
    }

    // =========================================================================
    // cat10: Cross-context category uniformity
    // =========================================================================

    #[test]
    fn cat10_script_actions_all_script_context() {
        let script = ScriptInfo::new("test", "/p");
        for action in &get_script_context_actions(&script) {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn cat10_clipboard_actions_all_script_context() {
        let entry = ClipboardEntryInfo {
            id: "c1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        for action in &get_clipboard_history_context_actions(&entry) {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn cat10_ai_actions_all_script_context() {
        for action in &get_ai_command_bar_actions() {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn cat10_path_actions_all_script_context() {
        let pi = PathInfo {
            name: "dir".into(),
            path: "/tmp/dir".into(),
            is_dir: true,
        };
        for action in &get_path_context_actions(&pi) {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn cat10_notes_actions_all_script_context() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for action in &get_notes_command_bar_actions(&info) {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn cat10_chat_actions_all_script_context() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: true,
        };
        for action in &get_chat_context_actions(&info) {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    // =========================================================================
    // cat11: Clipboard exact action counts on macOS
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn cat11_clipboard_text_count_macos() {
        let entry = ClipboardEntryInfo {
            id: "t1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hi".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        // paste, copy, paste_keep_open, share, attach, quick_look,
        // pin, save_snippet, save_file, delete, delete_multiple, delete_all = 12
        assert_eq!(actions.len(), 12, "Text on macOS: {}", actions.len());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat11_clipboard_image_count_macos() {
        let entry = ClipboardEntryInfo {
            id: "i1".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        // paste, copy, paste_keep_open, share, attach, quick_look,
        // open_with, annotate_cleanshot, upload_cleanshot,
        // pin, ocr, save_snippet, save_file, delete, delete_multiple, delete_all = 16
        assert_eq!(actions.len(), 16, "Image on macOS: {}", actions.len());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat11_clipboard_image_more_than_text_macos() {
        let img = ClipboardEntryInfo {
            id: "i".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let txt = ClipboardEntryInfo {
            id: "t".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let img_count = get_clipboard_history_context_actions(&img).len();
        let txt_count = get_clipboard_history_context_actions(&txt).len();
        assert!(
            img_count > txt_count,
            "Image ({}) should have more actions than text ({})",
            img_count,
            txt_count
        );
    }

    // =========================================================================
    // cat12: Path primary-action insertion position
    // =========================================================================

    #[test]
    fn cat12_path_dir_primary_first() {
        let pi = PathInfo {
            name: "mydir".into(),
            path: "/tmp/mydir".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&pi);
        assert_eq!(actions[0].id, "open_directory");
    }


    // --- merged from tests_part_03.rs ---
    #[test]
    fn cat12_path_file_primary_first() {
        let pi = PathInfo {
            name: "file.txt".into(),
            path: "/tmp/file.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&pi);
        assert_eq!(actions[0].id, "select_file");
    }

    #[test]
    fn cat12_path_trash_always_last() {
        let pi = PathInfo {
            name: "x".into(),
            path: "/tmp/x".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&pi);
        assert_eq!(actions.last().unwrap().id, "move_to_trash");
    }

    #[test]
    fn cat12_path_dir_and_file_same_count() {
        let dir = PathInfo {
            name: "d".into(),
            path: "/tmp/d".into(),
            is_dir: true,
        };
        let file = PathInfo {
            name: "f".into(),
            path: "/tmp/f".into(),
            is_dir: false,
        };
        assert_eq!(
            get_path_context_actions(&dir).len(),
            get_path_context_actions(&file).len()
        );
    }

    // =========================================================================
    // cat13: File title quoting
    // =========================================================================

    #[test]
    fn cat13_file_title_contains_quoted_name() {
        let fi = FileInfo {
            name: "report.pdf".into(),
            path: "/docs/report.pdf".into(),
            is_dir: false,
            file_type: crate::file_search::FileType::Document,
        };
        let actions = get_file_context_actions(&fi);
        assert!(
            actions[0].title.contains("\"report.pdf\""),
            "Title should contain quoted filename: {}",
            actions[0].title
        );
    }

    #[test]
    fn cat13_dir_title_contains_quoted_name() {
        let fi = FileInfo {
            name: "build".into(),
            path: "/project/build".into(),
            is_dir: true,
            file_type: crate::file_search::FileType::Directory,
        };
        let actions = get_file_context_actions(&fi);
        assert!(
            actions[0].title.contains("\"build\""),
            "Title should contain quoted dirname: {}",
            actions[0].title
        );
    }

    #[test]
    fn cat13_file_primary_is_open_file() {
        let fi = FileInfo {
            name: "x".into(),
            path: "/x".into(),
            is_dir: false,
            file_type: crate::file_search::FileType::Document,
        };
        let actions = get_file_context_actions(&fi);
        assert_eq!(actions[0].id, "open_file");
    }

    #[test]
    fn cat13_dir_primary_is_open_directory() {
        let fi = FileInfo {
            name: "y".into(),
            path: "/y".into(),
            is_dir: true,
            file_type: crate::file_search::FileType::Directory,
        };
        let actions = get_file_context_actions(&fi);
        assert_eq!(actions[0].id, "open_directory");
    }

    // =========================================================================
    // cat14: ScriptInfo::with_all field completeness
    // =========================================================================

    #[test]
    fn cat14_with_all_name_path() {
        let s = ScriptInfo::with_all("MyScript", "/path/my.ts", true, "Execute", None, None);
        assert_eq!(s.name, "MyScript");
        assert_eq!(s.path, "/path/my.ts");
    }

    #[test]
    fn cat14_with_all_is_script() {
        let s = ScriptInfo::with_all("S", "/p", true, "Run", None, None);
        assert!(s.is_script);
        let s2 = ScriptInfo::with_all("S", "/p", false, "Run", None, None);
        assert!(!s2.is_script);
    }

    #[test]
    fn cat14_with_all_verb() {
        let s = ScriptInfo::with_all("S", "/p", true, "Launch", None, None);
        assert_eq!(s.action_verb, "Launch");
    }

    #[test]
    fn cat14_with_all_shortcut_and_alias() {
        let s = ScriptInfo::with_all(
            "S",
            "/p",
            true,
            "Run",
            Some("cmd+k".into()),
            Some("sk".into()),
        );
        assert_eq!(s.shortcut, Some("cmd+k".to_string()));
        assert_eq!(s.alias, Some("sk".to_string()));
    }

    #[test]
    fn cat14_with_all_no_agent_no_scriptlet() {
        let s = ScriptInfo::with_all("S", "/p", true, "Run", None, None);
        assert!(!s.is_agent);
        assert!(!s.is_scriptlet);
        assert!(!s.is_suggested);
    }

    // =========================================================================
    // cat15: Script context run title includes verb + name
    // =========================================================================

    #[test]
    fn cat15_run_title_default_verb() {
        let s = ScriptInfo::new("My Script", "/p");
        let actions = get_script_context_actions(&s);
        assert!(
            actions[0].title.contains("Run"),
            "Default verb is Run: {}",
            actions[0].title
        );
        assert!(
            actions[0].title.contains("My Script"),
            "Title includes name: {}",
            actions[0].title
        );
    }

    #[test]
    fn cat15_run_title_custom_verb() {
        let s = ScriptInfo::with_action_verb("Windows", "/p", true, "Switch to");
        let actions = get_script_context_actions(&s);
        assert!(
            actions[0].title.contains("Switch to"),
            "Custom verb: {}",
            actions[0].title
        );
    }

    #[test]
    fn cat15_run_title_builtin() {
        let s = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&s);
        assert!(
            actions[0].title.contains("Clipboard History"),
            "Builtin title: {}",
            actions[0].title
        );
    }

    // =========================================================================
    // cat16: Ordering idempotency (double-call determinism)
    // =========================================================================

    #[test]
    fn cat16_script_actions_idempotent() {
        let s = ScriptInfo::new("test", "/p");
        let a1 = action_ids(&get_script_context_actions(&s));
        let a2 = action_ids(&get_script_context_actions(&s));
        assert_eq!(a1, a2);
    }

    #[test]
    fn cat16_clipboard_actions_idempotent() {
        let e = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hi".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let a1 = action_ids(&get_clipboard_history_context_actions(&e));
        let a2 = action_ids(&get_clipboard_history_context_actions(&e));
        assert_eq!(a1, a2);
    }

    #[test]
    fn cat16_ai_actions_idempotent() {
        let a1 = action_ids(&get_ai_command_bar_actions());
        let a2 = action_ids(&get_ai_command_bar_actions());
        assert_eq!(a1, a2);
    }

    #[test]
    fn cat16_notes_actions_idempotent() {
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
    fn cat16_path_actions_idempotent() {
        let pi = PathInfo {
            name: "x".into(),
            path: "/x".into(),
            is_dir: true,
        };
        let a1 = action_ids(&get_path_context_actions(&pi));
        let a2 = action_ids(&get_path_context_actions(&pi));
        assert_eq!(a1, a2);
    }

    // =========================================================================
    // cat17: Note switcher icon hierarchy
    // =========================================================================

    #[test]
    fn cat17_pinned_overrides_current() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "Both".into(),
            char_count: 10,
            is_current: true,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    #[test]
    fn cat17_current_only_check() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n2".into(),
            title: "Current".into(),
            char_count: 10,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::Check));
    }

    #[test]
    fn cat17_regular_file_icon() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n3".into(),
            title: "Regular".into(),
            char_count: 10,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::File));
    }

    #[test]
    fn cat17_pinned_not_current_star() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n4".into(),
            title: "Pinned".into(),
            char_count: 10,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    // =========================================================================
    // cat18: Note switcher section assignment
    // =========================================================================

    #[test]
    fn cat18_pinned_in_pinned_section() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "p1".into(),
            title: "P".into(),
            char_count: 1,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
    }

    #[test]
    fn cat18_unpinned_in_recent_section() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "r1".into(),
            title: "R".into(),
            char_count: 1,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn cat18_mixed_sections() {
        let notes = vec![
            NoteSwitcherNoteInfo {
                id: "p".into(),
                title: "Pinned".into(),
                char_count: 1,
                is_current: false,
                is_pinned: true,
                preview: String::new(),
                relative_time: String::new(),
            },
            NoteSwitcherNoteInfo {
                id: "r".into(),
                title: "Recent".into(),
                char_count: 1,
                is_current: false,
                is_pinned: false,
                preview: String::new(),
                relative_time: String::new(),
            },
        ];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
        assert_eq!(actions[1].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn cat18_empty_shows_notes_section() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].section.as_deref(), Some("Notes"));
    }

    // =========================================================================
    // cat19: Note switcher current bullet prefix
    // =========================================================================

    #[test]
    fn cat19_current_has_bullet() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "c1".into(),
            title: "My Note".into(),
            char_count: 10,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(
            actions[0].title.starts_with("• "),
            "Current note should have bullet prefix: {}",
            actions[0].title
        );
    }

    #[test]
    fn cat19_non_current_no_bullet() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "c2".into(),
            title: "Other Note".into(),
            char_count: 10,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(
            !actions[0].title.starts_with("• "),
            "Non-current should not have bullet: {}",
            actions[0].title
        );
    }

    #[test]
    fn cat19_current_pinned_has_bullet() {
        // Even pinned+current gets bullet prefix
        let notes = vec![NoteSwitcherNoteInfo {
            id: "c3".into(),
            title: "Pinned Current".into(),
            char_count: 10,
            is_current: true,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(actions[0].title.starts_with("• "));
    }

    // =========================================================================
    // cat20: Clipboard paste title dynamic behavior
    // =========================================================================


    // --- merged from tests_part_04.rs ---
    #[test]
    fn cat20_paste_no_app() {
        let entry = ClipboardEntryInfo {
            id: "p1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[0].title, "Paste to Active App");
    }

    #[test]
    fn cat20_paste_with_app() {
        let entry = ClipboardEntryInfo {
            id: "p2".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: Some("Safari".into()),
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[0].title, "Paste to Safari");
    }

    #[test]
    fn cat20_paste_with_unicode_app() {
        let entry = ClipboardEntryInfo {
            id: "p3".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: Some("日本語エディタ".into()),
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[0].title, "Paste to 日本語エディタ");
    }

    #[test]
    fn cat20_paste_with_empty_string_app() {
        let entry = ClipboardEntryInfo {
            id: "p4".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: Some(String::new()),
        };
        let actions = get_clipboard_history_context_actions(&entry);
        // Some("") → "Paste to " (empty name)
        assert_eq!(actions[0].title, "Paste to ");
    }

    // =========================================================================
    // cat21: Clipboard pin/unpin toggle
    // =========================================================================

    #[test]
    fn cat21_unpinned_shows_pin() {
        let entry = ClipboardEntryInfo {
            id: "u1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"clipboard_pin".to_string()));
        assert!(!ids.contains(&"clipboard_unpin".to_string()));
    }

    #[test]
    fn cat21_pinned_shows_unpin() {
        let entry = ClipboardEntryInfo {
            id: "u2".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"clipboard_unpin".to_string()));
        assert!(!ids.contains(&"clipboard_pin".to_string()));
    }

    #[test]
    fn cat21_pin_unpin_same_shortcut() {
        let pin_entry = ClipboardEntryInfo {
            id: "s1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let unpin_entry = ClipboardEntryInfo {
            id: "s2".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let pin_actions = get_clipboard_history_context_actions(&pin_entry);
        let unpin_actions = get_clipboard_history_context_actions(&unpin_entry);
        let pin_sc = pin_actions
            .iter()
            .find(|a| a.id == "clipboard_pin")
            .unwrap()
            .shortcut
            .as_deref();
        let unpin_sc = unpin_actions
            .iter()
            .find(|a| a.id == "clipboard_unpin")
            .unwrap()
            .shortcut
            .as_deref();
        assert_eq!(pin_sc, unpin_sc, "Pin and Unpin share same shortcut");
        assert_eq!(pin_sc, Some("⇧⌘P"));
    }

    // =========================================================================
    // cat22: Clipboard destructive actions always last three
    // =========================================================================

    #[test]
    fn cat22_text_last_three_destructive() {
        let entry = ClipboardEntryInfo {
            id: "d1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let len = actions.len();
        assert_eq!(actions[len - 3].id, "clipboard_delete");
        assert_eq!(actions[len - 2].id, "clipboard_delete_multiple");
        assert_eq!(actions[len - 1].id, "clipboard_delete_all");
    }

    #[test]
    fn cat22_image_last_three_destructive() {
        let entry = ClipboardEntryInfo {
            id: "d2".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let len = actions.len();
        assert_eq!(actions[len - 3].id, "clipboard_delete");
        assert_eq!(actions[len - 2].id, "clipboard_delete_multiple");
        assert_eq!(actions[len - 1].id, "clipboard_delete_all");
    }

    #[test]
    fn cat22_paste_always_first() {
        let entry = ClipboardEntryInfo {
            id: "d3".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[0].id, "clipboard_paste");
    }

    #[test]
    fn cat22_copy_always_second() {
        let entry = ClipboardEntryInfo {
            id: "d4".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[1].id, "clipboard_copy");
    }

    // =========================================================================
    // cat23: Action lowercase caching
    // =========================================================================

    #[test]
    fn cat23_title_lower_cached() {
        let action = Action::new("test", "My Title", None, ActionCategory::ScriptContext);
        assert_eq!(action.title_lower, "my title");
    }

    #[test]
    fn cat23_description_lower_cached() {
        let action = Action::new(
            "test",
            "T",
            Some("My Description".into()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.description_lower.as_deref(), Some("my description"));
    }

    #[test]
    fn cat23_description_none_lower_none() {
        let action = Action::new("test", "T", None, ActionCategory::ScriptContext);
        assert_eq!(action.description_lower, None);
    }

    #[test]
    fn cat23_shortcut_lower_none_initially() {
        let action = Action::new("test", "T", None, ActionCategory::ScriptContext);
        assert_eq!(action.shortcut_lower, None);
    }

    #[test]
    fn cat23_shortcut_lower_set_after_with_shortcut() {
        let action =
            Action::new("test", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
        assert_eq!(action.shortcut_lower.as_deref(), Some("⌘⇧c"));
    }

    #[test]
    fn cat23_title_lower_unicode() {
        let action = Action::new("test", "Café Résumé", None, ActionCategory::ScriptContext);
        assert_eq!(action.title_lower, "café résumé");
    }

    // =========================================================================
    // cat24: AI command bar total count and all have icons
    // =========================================================================

    #[test]
    fn cat24_ai_total_12() {
        assert_eq!(get_ai_command_bar_actions().len(), 12);
    }

    #[test]
    fn cat24_ai_all_have_icons() {
        for action in &get_ai_command_bar_actions() {
            assert!(
                action.icon.is_some(),
                "AI action {} should have icon",
                action.id
            );
        }
    }

    #[test]
    fn cat24_ai_all_have_sections() {
        for action in &get_ai_command_bar_actions() {
            assert!(
                action.section.is_some(),
                "AI action {} should have section",
                action.id
            );
        }
    }

    #[test]
    fn cat24_ai_6_unique_sections() {
        let actions = get_ai_command_bar_actions();
        let sections: HashSet<&str> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        assert_eq!(sections.len(), 6);
    }

    #[test]
    fn cat24_ai_ids_unique() {
        let actions = get_ai_command_bar_actions();
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len(), "AI action IDs should be unique");
    }

    // =========================================================================
    // cat25: Notes format action shortcut and icon
    // =========================================================================

    #[test]
    fn cat25_notes_format_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let format = actions.iter().find(|a| a.id == "format").unwrap();
        assert_eq!(format.shortcut.as_deref(), Some("⇧⌘T"));
    }

    #[test]
    fn cat25_notes_format_icon_code() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let format = actions.iter().find(|a| a.id == "format").unwrap();
        assert_eq!(format.icon, Some(IconName::Code));
    }

    #[test]
    fn cat25_notes_format_section_edit() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let format = actions.iter().find(|a| a.id == "format").unwrap();
        assert_eq!(format.section.as_deref(), Some("Edit"));
    }

    // =========================================================================
    // cat26: File context common actions always present
    // =========================================================================

    #[test]
    fn cat26_file_has_reveal() {
        let fi = FileInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
            file_type: crate::file_search::FileType::Document,
        };
        let ids = action_ids(&get_file_context_actions(&fi));
        assert!(ids.contains(&"reveal_in_finder".to_string()));
    }

    #[test]
    fn cat26_file_has_copy_path() {
        let fi = FileInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
            file_type: crate::file_search::FileType::Document,
        };
        let ids = action_ids(&get_file_context_actions(&fi));
        assert!(ids.contains(&"copy_path".to_string()));
    }

    #[test]
    fn cat26_file_has_copy_filename() {
        let fi = FileInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
            file_type: crate::file_search::FileType::Document,
        };
        let ids = action_ids(&get_file_context_actions(&fi));
        assert!(ids.contains(&"copy_filename".to_string()));
    }

    #[test]
    fn cat26_dir_has_reveal() {
        let fi = FileInfo {
            name: "d".into(),
            path: "/d".into(),
            is_dir: true,
            file_type: crate::file_search::FileType::Directory,
        };
        let ids = action_ids(&get_file_context_actions(&fi));
        assert!(ids.contains(&"reveal_in_finder".to_string()));
    }

    #[test]
    fn cat26_dir_has_copy_path() {
        let fi = FileInfo {
            name: "d".into(),
            path: "/d".into(),
            is_dir: true,
            file_type: crate::file_search::FileType::Directory,
        };
        let ids = action_ids(&get_file_context_actions(&fi));
        assert!(ids.contains(&"copy_path".to_string()));
    }

    // =========================================================================
    // cat27: Script context shortcut/alias dynamic action count
    // =========================================================================

    #[test]
    fn cat27_no_shortcut_no_alias_count() {
        let s = ScriptInfo::new("test", "/p");
        let actions = get_script_context_actions(&s);
        // run, edit, add_shortcut, add_alias, view_logs, reveal, copy_path, copy_content, copy_deeplink = 9
        assert_eq!(actions.len(), 9);
    }

    #[test]
    fn cat27_with_shortcut_count() {
        let s = ScriptInfo::with_shortcut("test", "/p", Some("cmd+t".into()));
        let actions = get_script_context_actions(&s);
        // run, edit, update_shortcut, remove_shortcut, add_alias, view_logs, reveal, copy_path, copy_content, copy_deeplink = 10
        assert_eq!(actions.len(), 10);
    }

    #[test]
    fn cat27_with_both_count() {
        let s = ScriptInfo::with_shortcut_and_alias(
            "test",
            "/p",
            Some("cmd+t".into()),
            Some("ts".into()),
        );
        let actions = get_script_context_actions(&s);
        // run, edit, update_shortcut, remove_shortcut, update_alias, remove_alias, view_logs, reveal, copy_path, copy_content, copy_deeplink = 11
        assert_eq!(actions.len(), 11);
    }

    #[test]
    fn cat27_frecency_adds_one() {
        let s = ScriptInfo::new("test", "/p").with_frecency(true, Some("/f".into()));
        let actions = get_script_context_actions(&s);
        // 9 + 1 (reset_ranking) = 10
        assert_eq!(actions.len(), 10);
    }

    // =========================================================================
    // cat28: has_action=false invariant for all built-ins
    // =========================================================================

    #[test]
    fn cat28_script_has_action_false() {
        let s = ScriptInfo::new("t", "/p");
        for action in &get_script_context_actions(&s) {
            assert!(
                !action.has_action,
                "Script action {} should have has_action=false",
                action.id
            );
        }
    }


    // --- merged from tests_part_05.rs ---
    #[test]
    fn cat28_clipboard_has_action_false() {
        let e = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        for action in &get_clipboard_history_context_actions(&e) {
            assert!(
                !action.has_action,
                "Clipboard action {} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat28_ai_has_action_false() {
        for action in &get_ai_command_bar_actions() {
            assert!(
                !action.has_action,
                "AI action {} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat28_path_has_action_false() {
        let pi = PathInfo {
            name: "x".into(),
            path: "/x".into(),
            is_dir: false,
        };
        for action in &get_path_context_actions(&pi) {
            assert!(
                !action.has_action,
                "Path action {} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat28_notes_has_action_false() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for action in &get_notes_command_bar_actions(&info) {
            assert!(
                !action.has_action,
                "Notes action {} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat28_file_has_action_false() {
        let fi = FileInfo {
            name: "f".into(),
            path: "/f".into(),
            is_dir: false,
            file_type: crate::file_search::FileType::Document,
        };
        for action in &get_file_context_actions(&fi) {
            assert!(
                !action.has_action,
                "File action {} should have has_action=false",
                action.id
            );
        }
    }

    // =========================================================================
    // cat29: ID uniqueness across contexts
    // =========================================================================

    #[test]
    fn cat29_script_ids_unique() {
        let s =
            ScriptInfo::with_shortcut_and_alias("t", "/p", Some("cmd+t".into()), Some("al".into()));
        let actions = get_script_context_actions(&s);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat29_clipboard_ids_unique() {
        let e = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&e);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat29_path_ids_unique() {
        let pi = PathInfo {
            name: "x".into(),
            path: "/x".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&pi);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat29_file_ids_unique() {
        let fi = FileInfo {
            name: "f".into(),
            path: "/f".into(),
            is_dir: false,
            file_type: crate::file_search::FileType::Document,
        };
        let actions = get_file_context_actions(&fi);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat29_notes_ids_unique() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat29_note_switcher_ids_unique() {
        let notes = vec![
            NoteSwitcherNoteInfo {
                id: "a".into(),
                title: "A".into(),
                char_count: 1,
                is_current: false,
                is_pinned: false,
                preview: String::new(),
                relative_time: String::new(),
            },
            NoteSwitcherNoteInfo {
                id: "b".into(),
                title: "B".into(),
                char_count: 1,
                is_current: false,
                is_pinned: false,
                preview: String::new(),
                relative_time: String::new(),
            },
        ];
        let actions = get_note_switcher_actions(&notes);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    // =========================================================================
    // cat30: Non-empty id and title invariant
    // =========================================================================

    #[test]
    fn cat30_script_nonempty_id_title() {
        let s = ScriptInfo::new("t", "/p");
        for action in &get_script_context_actions(&s) {
            assert!(!action.id.is_empty(), "ID should not be empty");
            assert!(!action.title.is_empty(), "Title should not be empty");
        }
    }

    #[test]
    fn cat30_clipboard_nonempty_id_title() {
        let e = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        for action in &get_clipboard_history_context_actions(&e) {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    #[test]
    fn cat30_ai_nonempty_id_title() {
        for action in &get_ai_command_bar_actions() {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    #[test]
    fn cat30_path_nonempty_id_title() {
        let pi = PathInfo {
            name: "x".into(),
            path: "/x".into(),
            is_dir: false,
        };
        for action in &get_path_context_actions(&pi) {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    #[test]
    fn cat30_notes_nonempty_id_title() {
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
    fn cat30_file_nonempty_id_title() {
        let fi = FileInfo {
            name: "f".into(),
            path: "/f".into(),
            is_dir: false,
            file_type: crate::file_search::FileType::Document,
        };
        for action in &get_file_context_actions(&fi) {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

}
