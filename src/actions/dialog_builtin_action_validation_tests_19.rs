// =============================================================================
// Dialog Built-in Action Validation Tests — Batch 19
//
// 30 categories of tests validating random built-in actions from dialog windows.
// Each category tests a specific behavior, field, or invariant.
//
// Run with:
//   cargo test --lib actions::dialog_builtin_action_validation_tests_19
// =============================================================================

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
    use crate::file_search::{FileInfo, FileType};
    use crate::prompts::PathInfo;
    use std::collections::HashSet;

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
        let dl = actions.iter().find(|a| a.id == "script:copy_deeplink").unwrap();
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
        let dl = actions.iter().find(|a| a.id == "script:copy_deeplink").unwrap();
        assert!(dl.description.as_ref().unwrap().contains("my-cool-script"));
    }

    #[test]
    fn cat02_builtin_deeplink_desc_contains_url() {
        let script = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&script);
        let dl = actions.iter().find(|a| a.id == "script:copy_deeplink").unwrap();
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
        let dl = actions.iter().find(|a| a.id == "script:copy_deeplink").unwrap();
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
        let paste = actions.iter().find(|a| a.id == "clip:clipboard_paste").unwrap();
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
        let paste = actions.iter().find(|a| a.id == "clip:clipboard_paste").unwrap();
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
        let paste = actions.iter().find(|a| a.id == "clip:clipboard_paste").unwrap();
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
        let paste = actions.iter().find(|a| a.id == "clip:clipboard_paste").unwrap();
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
                actions.iter().any(|a| a.id == "notes:new_note"),
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
        assert_eq!(actions[0].id, "chat:continue_in_chat");
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


    // --- merged from tests_part_02.rs ---
    #[test]
    fn cat07_note_switcher_preview_and_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".to_string(),
            title: "Test".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "Hello world".to_string(),
            relative_time: "2m ago".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains("Hello world"));
        assert!(desc.contains("2m ago"));
        assert!(desc.contains("·"));
    }

    #[test]
    fn cat07_note_switcher_preview_no_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".to_string(),
            title: "Test".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "Hello world".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "Hello world");
    }

    #[test]
    fn cat07_note_switcher_no_preview_with_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".to_string(),
            title: "Test".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "".to_string(),
            relative_time: "5d ago".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "5d ago");
    }

    #[test]
    fn cat07_note_switcher_no_preview_no_time_char_count() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".to_string(),
            title: "Test".to_string(),
            char_count: 42,
            is_current: false,
            is_pinned: false,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "42 chars");
    }

    #[test]
    fn cat07_note_switcher_zero_chars_fallback() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".to_string(),
            title: "Test".to_string(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "0 chars");
    }

    #[test]
    fn cat07_note_switcher_one_char_singular() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".to_string(),
            title: "Test".to_string(),
            char_count: 1,
            is_current: false,
            is_pinned: false,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "1 char");
    }

    // =========================================================================
    // Category 08: Script context action ordering invariant — run_script first
    // Validates that run_script is always the very first action.
    // =========================================================================

    #[test]
    fn cat08_script_run_first_basic() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        assert_eq!(actions[0].id, "run_script");
    }

    #[test]
    fn cat08_script_run_first_with_shortcut() {
        let script = ScriptInfo::with_shortcut("test", "/path/test.ts", Some("cmd+t".to_string()));
        let actions = get_script_context_actions(&script);
        assert_eq!(actions[0].id, "run_script");
    }

    #[test]
    fn cat08_script_run_first_builtin() {
        let script = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&script);
        assert_eq!(actions[0].id, "run_script");
    }

    #[test]
    fn cat08_scriptlet_run_first() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert_eq!(actions[0].id, "run_script");
    }

    #[test]
    fn cat08_script_run_shortcut_is_enter() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
    }

    // =========================================================================
    // Category 09: Path context directory vs file primary action distinction
    // Verifies the different primary action IDs and titles for dirs vs files.
    // =========================================================================

    #[test]
    fn cat09_path_dir_primary_is_open_directory() {
        let info = PathInfo {
            path: "/users/test/Downloads".to_string(),
            is_dir: true,
            name: "Downloads".to_string(),
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].id, "file:open_directory");
    }

    #[test]
    fn cat09_path_file_primary_is_select_file() {
        let info = PathInfo {
            path: "/users/test/file.txt".to_string(),
            is_dir: false,
            name: "file.txt".to_string(),
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].id, "file:select_file");
    }

    #[test]
    fn cat09_path_dir_primary_shortcut_enter() {
        let info = PathInfo {
            path: "/users/test/Downloads".to_string(),
            is_dir: true,
            name: "Downloads".to_string(),
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
    }

    #[test]
    fn cat09_path_file_primary_shortcut_enter() {
        let info = PathInfo {
            path: "/users/test/file.txt".to_string(),
            is_dir: false,
            name: "file.txt".to_string(),
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
    }

    #[test]
    fn cat09_path_trash_always_last() {
        let info = PathInfo {
            path: "/users/test/Downloads".to_string(),
            is_dir: true,
            name: "Downloads".to_string(),
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions.last().unwrap().id, "file:move_to_trash");
    }

    // =========================================================================
    // Category 10: File context macOS-only action presence
    // Validates macOS-specific actions exist for file context on macOS.
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn cat10_file_macos_has_quick_look() {
        let info = FileInfo {
            path: "/users/test/photo.jpg".to_string(),
            is_dir: false,
            name: "photo.jpg".to_string(),
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "file:quick_look"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat10_file_macos_has_open_with() {
        let info = FileInfo {
            path: "/users/test/photo.jpg".to_string(),
            is_dir: false,
            name: "photo.jpg".to_string(),
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "file:open_with"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat10_file_macos_dir_no_quick_look() {
        let info = FileInfo {
            path: "/users/test/folder".to_string(),
            is_dir: true,
            name: "folder".to_string(),
            file_type: FileType::Directory,
        };
        let actions = get_file_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "file:quick_look"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat10_file_macos_has_show_info() {
        let info = FileInfo {
            path: "/users/test/photo.jpg".to_string(),
            is_dir: false,
            name: "photo.jpg".to_string(),
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "file:show_info"));
    }

    // =========================================================================
    // Category 11: AI command bar action section membership — exact IDs
    // Validates exact action IDs within each section of the AI command bar.
    // =========================================================================

    #[test]
    fn cat11_ai_response_section_ids() {
        let actions = get_ai_command_bar_actions();
        let response_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Response"))
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(
            response_ids,
            vec!["chat:copy_response", "chat:copy_chat", "chat:copy_last_code"]
        );
    }

    #[test]
    fn cat11_ai_actions_section_ids() {
        let actions = get_ai_command_bar_actions();
        let action_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Actions"))
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(
            action_ids,
            vec!["chat:submit", "chat:new_chat", "chat:delete_chat", "chat:branch_from_last"]
        );
    }

    #[test]
    fn cat11_ai_attachments_section_ids() {
        let actions = get_ai_command_bar_actions();
        let attachment_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Attachments"))
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(attachment_ids, vec!["chat:add_attachment", "chat:paste_image"]);
    }

    #[test]
    fn cat11_ai_export_section_ids() {
        let actions = get_ai_command_bar_actions();
        let export_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Export"))
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(export_ids, vec!["chat:export_markdown"]);
    }

    #[test]
    fn cat11_ai_help_and_settings_section_ids() {
        let actions = get_ai_command_bar_actions();
        let help_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Help"))
            .map(|a| a.id.as_str())
            .collect();
        let settings_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Settings"))
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(help_ids, vec!["chat:toggle_shortcuts_help"]);
        assert_eq!(settings_ids, vec!["chat:change_model"]);
    }

    // =========================================================================
    // Category 12: to_deeplink_name special character handling
    // Tests edge cases for special character replacement and collapsing.
    // =========================================================================

    #[test]
    fn cat12_deeplink_spaces_to_hyphens() {
        assert_eq!(to_deeplink_name("Hello World"), "hello-world");
    }

    #[test]
    fn cat12_deeplink_consecutive_specials_collapsed() {
        assert_eq!(to_deeplink_name("a--b__c  d"), "a-b-c-d");
    }

    #[test]
    fn cat12_deeplink_leading_trailing_stripped() {
        assert_eq!(to_deeplink_name("---hello---"), "hello");
    }

    #[test]
    fn cat12_deeplink_all_specials_returns_empty() {
        assert_eq!(to_deeplink_name("!@#$%^&*"), "");
    }

    #[test]
    fn cat12_deeplink_unicode_preserved() {
        let result = to_deeplink_name("日本語テスト");
        assert!(result.contains('日'));
        assert!(result.contains('語'));
    }

    #[test]
    fn cat12_deeplink_mixed_case_lowered() {
        assert_eq!(to_deeplink_name("MyScript"), "myscript");
    }

    // =========================================================================
    // Category 13: format_shortcut_hint modifier replacement
    // Tests that modifier keys are correctly replaced with symbols.
    // (format_shortcut_hint is private, so we test it indirectly via action shortcuts)
    // =========================================================================

    #[test]
    fn cat13_script_add_shortcut_uses_formatted_hint() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        let add_shortcut = actions.iter().find(|a| a.id == "add_shortcut").unwrap();
        // Expected: ⌘⇧K (cmd+shift+k formatted)
        assert_eq!(add_shortcut.shortcut.as_deref(), Some("⌘⇧K"));
    }

    #[test]
    fn cat13_script_edit_shortcut_formatted() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.shortcut.as_deref(), Some("⌘E"));
    }

    #[test]
    fn cat13_script_view_logs_shortcut_formatted() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        let logs = actions.iter().find(|a| a.id == "view_logs").unwrap();
        assert_eq!(logs.shortcut.as_deref(), Some("⌘L"));
    }

    #[test]
    fn cat13_clipboard_delete_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let del = actions.iter().find(|a| a.id == "clip:clipboard_delete").unwrap();
        assert_eq!(del.shortcut.as_deref(), Some("⌃X"));
    }

    #[test]
    fn cat13_clipboard_delete_all_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let del_all = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_delete_all")
            .unwrap();
        assert_eq!(del_all.shortcut.as_deref(), Some("⌃⇧X"));
    }

    // =========================================================================
    // Category 14: score_action prefix vs contains vs fuzzy scoring
    // Tests the scoring function used for filtering actions.
    // =========================================================================

    #[test]
    fn cat14_score_prefix_match_100_plus() {
        let action = Action::new(
            "edit_script",
            "Edit Script",
            Some("Open in editor".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "script:edit");
        assert!(score >= 100, "Prefix match should be 100+, got {}", score);
    }


    // --- merged from tests_part_03.rs ---
    #[test]
    fn cat14_score_contains_match_50() {
        let action = Action::new(
            "file:copy_path",
            "Copy Path",
            Some("Copy to clipboard".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "path");
        assert!(
            (50..100).contains(&score),
            "Contains match should be 50-99, got {}",
            score
        );
    }

    #[test]
    fn cat14_score_no_match_zero() {
        let action = Action::new(
            "edit_script",
            "Edit Script",
            None,
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "zzz");
        assert_eq!(score, 0, "No match should be 0, got {}", score);
    }

    #[test]
    fn cat14_score_empty_query_prefix() {
        let action = Action::new(
            "edit_script",
            "Edit Script",
            None,
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "");
        assert!(
            score >= 100,
            "Empty query is prefix match (100+), got {}",
            score
        );
    }

    #[test]
    fn cat14_score_description_bonus() {
        let action = Action::new(
            "file:open_file",
            "Open File",
            Some("Edit the file in your editor".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "editor");
        assert!(
            score >= 15,
            "Description match should give 15+, got {}",
            score
        );
    }

    // =========================================================================
    // Category 15: fuzzy_match subsequence correctness
    // Tests the fuzzy matching helper function.
    // =========================================================================

    #[test]
    fn cat15_fuzzy_exact_match() {
        assert!(ActionsDialog::fuzzy_match("hello", "hello"));
    }

    #[test]
    fn cat15_fuzzy_subsequence_match() {
        assert!(ActionsDialog::fuzzy_match("hello world", "hlwrd"));
    }

    #[test]
    fn cat15_fuzzy_no_match() {
        assert!(!ActionsDialog::fuzzy_match("hello", "xyz"));
    }

    #[test]
    fn cat15_fuzzy_empty_needle() {
        assert!(ActionsDialog::fuzzy_match("hello", ""));
    }

    #[test]
    fn cat15_fuzzy_empty_haystack() {
        assert!(!ActionsDialog::fuzzy_match("", "a"));
    }

    #[test]
    fn cat15_fuzzy_needle_longer_than_haystack() {
        assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
    }

    #[test]
    fn cat15_fuzzy_both_empty() {
        assert!(ActionsDialog::fuzzy_match("", ""));
    }

    // =========================================================================
    // Category 16: build_grouped_items_static section style behavior
    // Tests that different section styles produce correct GroupedActionItem layouts.
    // =========================================================================

    #[test]
    fn cat16_grouped_headers_style_adds_section_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered = vec![0, 1];
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Should have: Header(S1), Item(0), Header(S2), Item(1)
        assert_eq!(items.len(), 4);
        assert!(matches!(items[0], GroupedActionItem::SectionHeader(_)));
        assert!(matches!(items[1], GroupedActionItem::Item(0)));
        assert!(matches!(items[2], GroupedActionItem::SectionHeader(_)));
        assert!(matches!(items[3], GroupedActionItem::Item(1)));
    }

    #[test]
    fn cat16_grouped_separators_style_no_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered = vec![0, 1];
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        // No headers, just items
        assert_eq!(items.len(), 2);
        assert!(matches!(items[0], GroupedActionItem::Item(0)));
        assert!(matches!(items[1], GroupedActionItem::Item(1)));
    }

    #[test]
    fn cat16_grouped_none_style_no_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered = vec![0, 1];
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        assert_eq!(items.len(), 2);
        assert!(matches!(items[0], GroupedActionItem::Item(0)));
    }

    #[test]
    fn cat16_grouped_empty_filtered_returns_empty() {
        let actions =
            vec![Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1")];
        let filtered: Vec<usize> = vec![];
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(items.is_empty());
    }

    #[test]
    fn cat16_grouped_same_section_one_header() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S1"),
        ];
        let filtered = vec![0, 1];
        let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Header(S1), Item(0), Item(1) — no duplicate header
        assert_eq!(items.len(), 3);
        let header_count = items
            .iter()
            .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 1);
    }

    // =========================================================================
    // Category 17: coerce_action_selection header skipping
    // Tests that selection correctly skips over section headers.
    // =========================================================================

    #[test]
    fn cat17_coerce_on_item_stays() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    }

    #[test]
    fn cat17_coerce_on_header_moves_down() {
        let rows = vec![
            GroupedActionItem::SectionHeader("S1".to_string()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn cat17_coerce_trailing_header_moves_up() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("S1".to_string()),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(0));
    }

    #[test]
    fn cat17_coerce_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("S1".to_string()),
            GroupedActionItem::SectionHeader("S2".to_string()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn cat17_coerce_empty_returns_none() {
        let rows: Vec<GroupedActionItem> = vec![];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    // =========================================================================
    // Category 18: CommandBarConfig preset defaults — close flags
    // Validates that all presets have consistent close behavior defaults.
    // =========================================================================

    #[test]
    fn cat18_default_config_close_on_select() {
        let config = CommandBarConfig::default();
        assert!(config.close_on_select);
    }

    #[test]
    fn cat18_default_config_close_on_escape() {
        let config = CommandBarConfig::default();
        assert!(config.close_on_escape);
    }

    #[test]
    fn cat18_ai_style_close_on_select() {
        let config = CommandBarConfig::ai_style();
        assert!(config.close_on_select);
    }

    #[test]
    fn cat18_main_menu_close_on_select() {
        let config = CommandBarConfig::main_menu_style();
        assert!(config.close_on_select);
    }

    #[test]
    fn cat18_notes_style_close_on_select() {
        let config = CommandBarConfig::notes_style();
        assert!(config.close_on_select);
    }

    #[test]
    fn cat18_no_search_close_on_escape() {
        let config = CommandBarConfig::no_search();
        assert!(config.close_on_escape);
    }

    // =========================================================================
    // Category 19: Action lowercase caching correctness
    // Verifies that title_lower, description_lower, and shortcut_lower
    // are correctly pre-computed when constructing an Action.
    // =========================================================================

    #[test]
    fn cat19_title_lower_computed() {
        let action = Action::new("test", "Edit Script", None, ActionCategory::ScriptContext);
        assert_eq!(action.title_lower, "edit script");
    }

    #[test]
    fn cat19_description_lower_computed() {
        let action = Action::new(
            "test",
            "Test",
            Some("Open In Editor".to_string()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.description_lower, Some("open in editor".to_string()));
    }

    #[test]
    fn cat19_description_lower_none_when_no_desc() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
        assert!(action.description_lower.is_none());
    }

    #[test]
    fn cat19_shortcut_lower_none_initially() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
        assert!(action.shortcut_lower.is_none());
    }

    #[test]
    fn cat19_shortcut_lower_set_after_with_shortcut() {
        let action =
            Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
        assert_eq!(action.shortcut_lower, Some("⌘e".to_string()));
    }

    #[test]
    fn cat19_unicode_title_lower() {
        let action = Action::new("test", "ÜBER SCRIPT", None, ActionCategory::ScriptContext);
        assert_eq!(action.title_lower, "über script");
    }

    // =========================================================================
    // Category 20: Action builder chaining — field preservation
    // Verifies that chaining with_shortcut, with_icon, with_section
    // preserves previously set fields.
    // =========================================================================

    #[test]
    fn cat20_with_shortcut_then_icon_preserves_shortcut() {
        let action = Action::new("t", "T", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘E")
            .with_icon(IconName::Copy);
        assert_eq!(action.shortcut.as_deref(), Some("⌘E"));
        assert_eq!(action.icon, Some(IconName::Copy));
    }

    #[test]
    fn cat20_with_icon_then_section_preserves_icon() {
        let action = Action::new("t", "T", None, ActionCategory::ScriptContext)
            .with_icon(IconName::Star)
            .with_section("Help");
        assert_eq!(action.icon, Some(IconName::Star));
        assert_eq!(action.section.as_deref(), Some("Help"));
    }

    #[test]
    fn cat20_full_chain_preserves_all() {
        let action = Action::new(
            "t",
            "T",
            Some("Desc".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘E")
        .with_icon(IconName::Settings)
        .with_section("Settings");
        assert_eq!(action.shortcut.as_deref(), Some("⌘E"));
        assert_eq!(action.icon, Some(IconName::Settings));
        assert_eq!(action.section.as_deref(), Some("Settings"));
        assert_eq!(action.description.as_deref(), Some("Desc"));
    }

    #[test]
    fn cat20_with_shortcut_opt_none_preserves_existing() {
        let action = Action::new("t", "T", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘E")
            .with_shortcut_opt(None);
        // with_shortcut_opt(None) preserves the existing shortcut
        assert_eq!(action.shortcut.as_deref(), Some("⌘E"));
    }

    #[test]
    fn cat20_with_shortcut_opt_some_sets() {
        let action = Action::new("t", "T", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘F".to_string()));
        assert_eq!(action.shortcut.as_deref(), Some("⌘F"));
    }

    // =========================================================================
    // Category 21: ScriptInfo constructor defaults and mutability
    // Validates default field values across constructors and mutability of flags.
    // =========================================================================

    #[test]
    fn cat21_new_defaults() {
        let s = ScriptInfo::new("test", "/path/test.ts");
        assert!(s.is_script);
        assert!(!s.is_scriptlet);
        assert!(!s.is_agent);
        assert_eq!(s.action_verb, "Run");
        assert!(s.shortcut.is_none());
        assert!(s.alias.is_none());
        assert!(!s.is_suggested);
    }

    #[test]
    fn cat21_builtin_defaults() {
        let s = ScriptInfo::builtin("Test");
        assert!(!s.is_script);
        assert!(!s.is_scriptlet);
        assert!(!s.is_agent);
        assert!(s.path.is_empty());
    }

    #[test]
    fn cat21_scriptlet_defaults() {
        let s = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        assert!(!s.is_script);
        assert!(s.is_scriptlet);
        assert!(!s.is_agent);
    }

    #[test]
    fn cat21_agent_via_mutation() {
        let mut s = ScriptInfo::new("Agent", "/path/agent.md");
        s.is_agent = true;
        s.is_script = false;
        assert!(s.is_agent);
        assert!(!s.is_script);
    }

    #[test]
    fn cat21_with_frecency_builder() {
        let s = ScriptInfo::new("test", "/path/test.ts")
            .with_frecency(true, Some("/frecency".to_string()));
        assert!(s.is_suggested);
        assert_eq!(s.frecency_path, Some("/frecency".to_string()));
    }

    // =========================================================================
    // Category 22: Clipboard pin/unpin toggle — exact action details
    // Validates that pin/unpin toggle produces correct titles/descriptions.
    // =========================================================================

    #[test]
    fn cat22_unpinned_shows_pin() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clip:clipboard_pin"));
        assert!(!actions.iter().any(|a| a.id == "clip:clipboard_unpin"));
    }

    #[test]
    fn cat22_pinned_shows_unpin() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(!actions.iter().any(|a| a.id == "clip:clipboard_pin"));
        assert!(actions.iter().any(|a| a.id == "clip:clipboard_unpin"));
    }


    // --- merged from tests_part_04.rs ---
    #[test]
    fn cat22_pin_unpin_same_shortcut() {
        let pinned_entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let unpinned_entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let pinned_actions = get_clipboard_history_context_actions(&pinned_entry);
        let unpinned_actions = get_clipboard_history_context_actions(&unpinned_entry);
        let unpin = pinned_actions
            .iter()
            .find(|a| a.id == "clip:clipboard_unpin")
            .unwrap();
        let pin = unpinned_actions
            .iter()
            .find(|a| a.id == "clip:clipboard_pin")
            .unwrap();
        assert_eq!(unpin.shortcut, pin.shortcut);
        assert_eq!(pin.shortcut.as_deref(), Some("⇧⌘P"));
    }

    #[test]
    fn cat22_pin_title_and_description() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let pin = actions.iter().find(|a| a.id == "clip:clipboard_pin").unwrap();
        assert_eq!(pin.title, "Pin Entry");
        assert!(pin
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("pin"));
    }

    // =========================================================================
    // Category 23: Clipboard save actions — snippet and file shortcuts
    // Validates save snippet and save file action details.
    // =========================================================================

    #[test]
    fn cat23_save_snippet_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let snippet = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_save_snippet")
            .unwrap();
        assert_eq!(snippet.shortcut.as_deref(), Some("⇧⌘S"));
    }

    #[test]
    fn cat23_save_file_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let file = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_save_file")
            .unwrap();
        assert_eq!(file.shortcut.as_deref(), Some("⌥⇧⌘S"));
    }

    #[test]
    fn cat23_save_snippet_title() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let snippet = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_save_snippet")
            .unwrap();
        assert_eq!(snippet.title, "Save Text as Snippet");
    }

    #[test]
    fn cat23_save_file_title() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let file = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_save_file")
            .unwrap();
        assert_eq!(file.title, "Save as File...");
    }

    // =========================================================================
    // Category 24: Script context shortcut count — add vs update/remove
    // Validates exact action count difference between no-shortcut and with-shortcut.
    // =========================================================================

    #[test]
    fn cat24_no_shortcut_has_add_only() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        let shortcut_actions: Vec<&Action> = actions
            .iter()
            .filter(|a| {
                a.id == "add_shortcut" || a.id == "update_shortcut" || a.id == "remove_shortcut"
            })
            .collect();
        assert_eq!(shortcut_actions.len(), 1);
        assert_eq!(shortcut_actions[0].id, "add_shortcut");
    }

    #[test]
    fn cat24_with_shortcut_has_update_and_remove() {
        let script = ScriptInfo::with_shortcut("test", "/path/test.ts", Some("cmd+t".to_string()));
        let actions = get_script_context_actions(&script);
        let shortcut_actions: Vec<&Action> = actions
            .iter()
            .filter(|a| {
                a.id == "add_shortcut" || a.id == "update_shortcut" || a.id == "remove_shortcut"
            })
            .collect();
        assert_eq!(shortcut_actions.len(), 2);
        let ids: HashSet<&str> = shortcut_actions.iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains("update_shortcut"));
        assert!(ids.contains("remove_shortcut"));
    }

    #[test]
    fn cat24_with_shortcut_one_more_action() {
        let no_shortcut = ScriptInfo::new("test", "/path/test.ts");
        let with_shortcut =
            ScriptInfo::with_shortcut("test", "/path/test.ts", Some("cmd+t".to_string()));
        let count_no = get_script_context_actions(&no_shortcut).len();
        let count_with = get_script_context_actions(&with_shortcut).len();
        assert_eq!(count_with, count_no + 1); // update + remove = add + 1
    }

    #[test]
    fn cat24_same_pattern_for_alias() {
        let no_alias = ScriptInfo::new("test", "/path/test.ts");
        let with_alias = ScriptInfo::with_shortcut_and_alias(
            "test",
            "/path/test.ts",
            None,
            Some("ts".to_string()),
        );
        let count_no = get_script_context_actions(&no_alias).len();
        let count_with = get_script_context_actions(&with_alias).len();
        assert_eq!(count_with, count_no + 1);
    }

    // =========================================================================
    // Category 25: Note switcher icon assignment — pinned > current > default
    // Verifies the icon priority: StarFilled > Check > File.
    // =========================================================================

    #[test]
    fn cat25_pinned_gets_star_filled() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "Test".to_string(),
            char_count: 10,
            is_current: false,
            is_pinned: true,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    #[test]
    fn cat25_current_gets_check() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "Test".to_string(),
            char_count: 10,
            is_current: true,
            is_pinned: false,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::Check));
    }

    #[test]
    fn cat25_regular_gets_file() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "Test".to_string(),
            char_count: 10,
            is_current: false,
            is_pinned: false,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::File));
    }

    #[test]
    fn cat25_pinned_and_current_prefers_star() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "Test".to_string(),
            char_count: 10,
            is_current: true,
            is_pinned: true,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    #[test]
    fn cat25_note_switcher_empty_placeholder_icon() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions[0].icon, Some(IconName::Plus));
    }

    // =========================================================================
    // Category 26: Note switcher section assignment — Pinned vs Recent
    // Validates that pinned notes go to "Pinned" section and others to "Recent".
    // =========================================================================

    #[test]
    fn cat26_pinned_section() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "Pinned Note".to_string(),
            char_count: 10,
            is_current: false,
            is_pinned: true,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
    }

    #[test]
    fn cat26_unpinned_section() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "Recent Note".to_string(),
            char_count: 10,
            is_current: false,
            is_pinned: false,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn cat26_mixed_notes_correct_sections() {
        let notes = vec![
            NoteSwitcherNoteInfo {
                id: "a".to_string(),
                title: "Pinned".to_string(),
                char_count: 10,
                is_current: false,
                is_pinned: true,
                preview: "".to_string(),
                relative_time: "".to_string(),
            },
            NoteSwitcherNoteInfo {
                id: "b".to_string(),
                title: "Recent".to_string(),
                char_count: 20,
                is_current: false,
                is_pinned: false,
                preview: "".to_string(),
                relative_time: "".to_string(),
            },
        ];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
        assert_eq!(actions[1].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn cat26_empty_notes_placeholder_section() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions[0].section.as_deref(), Some("Notes"));
    }

    // =========================================================================
    // Category 27: Note switcher current indicator — bullet prefix
    // Validates the "• " prefix for current notes.
    // =========================================================================

    #[test]
    fn cat27_current_note_has_bullet() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "My Note".to_string(),
            char_count: 10,
            is_current: true,
            is_pinned: false,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(actions[0].title.starts_with("• "));
    }

    #[test]
    fn cat27_non_current_no_bullet() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "My Note".to_string(),
            char_count: 10,
            is_current: false,
            is_pinned: false,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(!actions[0].title.starts_with("• "));
        assert_eq!(actions[0].title, "My Note");
    }

    #[test]
    fn cat27_current_and_pinned_has_bullet() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "My Note".to_string(),
            char_count: 10,
            is_current: true,
            is_pinned: true,
            preview: "".to_string(),
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(actions[0].title.starts_with("• "));
    }

    // =========================================================================
    // Category 28: Note switcher preview truncation boundary
    // Tests exact truncation at the 60-character boundary with ellipsis.
    // =========================================================================

    #[test]
    fn cat28_preview_exactly_60_no_ellipsis() {
        let preview = "a".repeat(60);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "Test".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview,
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(!desc.contains('…'));
        assert_eq!(desc.len(), 60);
    }

    #[test]
    fn cat28_preview_61_has_ellipsis() {
        let preview = "a".repeat(61);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "Test".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview,
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains('…'));
    }

    #[test]
    fn cat28_preview_59_no_ellipsis() {
        let preview = "b".repeat(59);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "Test".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview,
            relative_time: "".to_string(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(!desc.contains('…'));
    }

    // =========================================================================
    // Category 29: Cross-context has_action=false invariant
    // Validates that all built-in actions have has_action=false.
    // =========================================================================


    // --- merged from tests_part_05.rs ---
    #[test]
    fn cat29_script_actions_has_action_false() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        for action in &get_script_context_actions(&script) {
            assert!(
                !action.has_action,
                "Script action {} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat29_clipboard_actions_has_action_false() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        for action in &get_clipboard_history_context_actions(&entry) {
            assert!(
                !action.has_action,
                "Clipboard action {} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat29_ai_command_bar_has_action_false() {
        for action in &get_ai_command_bar_actions() {
            assert!(
                !action.has_action,
                "AI action {} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat29_path_actions_has_action_false() {
        let info = PathInfo {
            path: "/test".to_string(),
            is_dir: true,
            name: "test".to_string(),
        };
        for action in &get_path_context_actions(&info) {
            assert!(
                !action.has_action,
                "Path action {} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat29_notes_actions_has_action_false() {
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
    fn cat29_file_actions_has_action_false() {
        let info = FileInfo {
            path: "/test/file.txt".to_string(),
            is_dir: false,
            name: "file.txt".to_string(),
            file_type: FileType::File,
        };
        for action in &get_file_context_actions(&info) {
            assert!(
                !action.has_action,
                "File action {} should have has_action=false",
                action.id
            );
        }
    }

    // =========================================================================
    // Category 30: Cross-context ID uniqueness invariant
    // Validates that all action IDs within a single context are unique.
    // =========================================================================

    #[test]
    fn cat30_script_ids_unique() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len(), "Script action IDs not unique");
    }

    #[test]
    fn cat30_clipboard_text_ids_unique() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len(), "Clipboard text IDs not unique");
    }

    #[test]
    fn cat30_clipboard_image_ids_unique() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "".to_string(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len(), "Clipboard image IDs not unique");
    }

    #[test]
    fn cat30_ai_command_bar_ids_unique() {
        let actions = get_ai_command_bar_actions();
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len(), "AI command bar IDs not unique");
    }

    #[test]
    fn cat30_notes_ids_unique() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len(), "Notes action IDs not unique");
    }

    #[test]
    fn cat30_path_ids_unique() {
        let info = PathInfo {
            path: "/test".to_string(),
            is_dir: true,
            name: "test".to_string(),
        };
        let actions = get_path_context_actions(&info);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len(), "Path action IDs not unique");
    }

}
