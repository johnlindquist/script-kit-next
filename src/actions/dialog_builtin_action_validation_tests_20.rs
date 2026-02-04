// =============================================================================
// Dialog Built-in Action Validation Tests — Batch 20
//
// 30 categories of tests validating random built-in actions from dialog windows.
// Each category tests a specific behavior, field, or invariant.
//
// Run with:
//   cargo test --lib actions::dialog_builtin_action_validation_tests_20
// =============================================================================

#[cfg(test)]
mod tests {
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

    #[test]
    fn cat07_model_title_checkmark_current() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".into()),
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
        let gpt4 = actions
            .iter()
            .find(|a| a.id == "select_model_gpt-4")
            .unwrap();
        assert!(gpt4.title.contains('✓'));
        let claude = actions
            .iter()
            .find(|a| a.id == "select_model_claude")
            .unwrap();
        assert!(!claude.title.contains('✓'));
    }

    #[test]
    fn cat07_model_description_via_provider() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "m".into(),
                display_name: "M".into(),
                provider: "Acme".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let m = actions.iter().find(|a| a.id == "select_model_m").unwrap();
        assert_eq!(m.description.as_ref().unwrap(), "via Acme");
    }

    // =========================================================================
    // Category 08: New chat last_used section icon is BoltFilled
    // Verifies icon and section assignments in get_new_chat_actions.
    // =========================================================================

    #[test]
    fn cat08_last_used_icon_bolt() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m".into(),
            display_name: "M".into(),
            provider: "P".into(),
            provider_display_name: "Provider".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
    }

    #[test]
    fn cat08_last_used_section_name() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m".into(),
            display_name: "M".into(),
            provider: "P".into(),
            provider_display_name: "Provider".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].section.as_ref().unwrap(), "Last Used Settings");
    }

    #[test]
    fn cat08_model_section_name() {
        let models = vec![NewChatModelInfo {
            model_id: "m".into(),
            display_name: "M".into(),
            provider: "P".into(),
            provider_display_name: "Provider".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].section.as_ref().unwrap(), "Models");
    }

    #[test]
    fn cat08_model_icon_settings() {
        let models = vec![NewChatModelInfo {
            model_id: "m".into(),
            display_name: "M".into(),
            provider: "P".into(),
            provider_display_name: "Provider".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].icon, Some(IconName::Settings));
    }

    #[test]
    fn cat08_preset_section_name() {
        let presets = vec![NewChatPresetInfo {
            id: "gen".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].section.as_ref().unwrap(), "Presets");
    }

    #[test]
    fn cat08_preset_icon_preserved() {
        let presets = vec![NewChatPresetInfo {
            id: "gen".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].icon, Some(IconName::Star));
    }

    // =========================================================================
    // Category 09: Note switcher description with preview + relative_time
    // Verifies the "preview · time" format and char count fallback.
    // =========================================================================

    #[test]
    fn cat09_preview_and_time_format() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "Note".into(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "Hello world".into(),
            relative_time: "2m ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains("Hello world"));
        assert!(desc.contains("2m ago"));
        assert!(desc.contains(" · "));
    }

    #[test]
    fn cat09_no_preview_uses_char_count() {
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
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains("42 chars"));
    }

    #[test]
    fn cat09_char_count_singular() {
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
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains("1 char"));
        assert!(!desc.contains("chars"));
    }

    #[test]
    fn cat09_preview_only_no_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "Note".into(),
            char_count: 5,
            is_current: false,
            is_pinned: false,
            preview: "Some text".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "Some text");
    }

    #[test]
    fn cat09_time_only_no_preview() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "Note".into(),
            char_count: 5,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "5d ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "5d ago");
    }

    // =========================================================================
    // Category 10: to_deeplink_name CJK and accented character preservation
    // Verifies Unicode alphanumeric chars are kept, not stripped.
    // =========================================================================

    #[test]
    fn cat10_cjk_chars_preserved() {
        let result = to_deeplink_name("测试脚本");
        assert!(result.contains("测试脚本"));
    }

    #[test]
    fn cat10_accented_chars_preserved() {
        let result = to_deeplink_name("Résumé Editor");
        assert!(result.contains("résumé"));
    }

    #[test]
    fn cat10_mixed_case_lowered() {
        assert_eq!(to_deeplink_name("MyScript"), "myscript");
    }

    #[test]
    fn cat10_mixed_special_and_alpha() {
        assert_eq!(to_deeplink_name("My -- Script!"), "my-script");
    }

    #[test]
    fn cat10_empty_string() {
        assert_eq!(to_deeplink_name(""), "");
    }

    // =========================================================================
    // Category 11: Action::new pre-computes lowercase fields correctly
    // =========================================================================

    #[test]
    fn cat11_title_lower_matches() {
        let action = Action::new("id", "Hello World", None, ActionCategory::ScriptContext);
        assert_eq!(action.title_lower, "hello world");
    }

    #[test]
    fn cat11_description_lower_matches() {
        let action = Action::new(
            "id",
            "T",
            Some("Hello DESC".to_string()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.description_lower, Some("hello desc".to_string()));
    }

    #[test]
    fn cat11_description_lower_none() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext);
        assert!(action.description_lower.is_none());
    }

    #[test]
    fn cat11_shortcut_lower_none_initially() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext);
        assert!(action.shortcut_lower.is_none());
    }

    #[test]
    fn cat11_shortcut_lower_set_after_with() {
        let action =
            Action::new("id", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
        assert_eq!(action.shortcut_lower, Some("⌘⇧c".to_string()));
    }

    // =========================================================================
    // Category 12: score_action scoring tiers via ActionsDialog
    // =========================================================================

    #[test]
    fn cat12_prefix_scores_100() {
        let action = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "edit");
        assert_eq!(score, 100);
    }

    #[test]
    fn cat12_contains_scores_50() {
        let action = Action::new("id", "Copy Edit Path", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "edit");
        assert_eq!(score, 50);
    }

    #[test]
    fn cat12_no_match_scores_0() {
        let action = Action::new("id", "Run Script", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "xyz");
        assert_eq!(score, 0);
    }

    #[test]
    fn cat12_desc_bonus_15() {
        let action = Action::new(
            "id",
            "Open File",
            Some("Edit in editor".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "editor");
        assert!(
            score >= 15,
            "Description match should add at least 15 points, got {}",
            score
        );
    }

    #[test]
    fn cat12_shortcut_bonus_10() {
        let action =
            Action::new("id", "Test", None, ActionCategory::ScriptContext).with_shortcut("⌘X");
        let score = ActionsDialog::score_action(&action, "⌘x");
        assert!(
            score >= 10,
            "Shortcut match should score 10+, got {}",
            score
        );
    }

    // =========================================================================
    // Category 13: fuzzy_match subsequence behavior
    // =========================================================================

    #[test]
    fn cat13_exact_match() {
        assert!(ActionsDialog::fuzzy_match("hello", "hello"));
    }

    #[test]
    fn cat13_subsequence_match() {
        assert!(ActionsDialog::fuzzy_match("hello world", "hlwrd"));
    }

    #[test]
    fn cat13_no_match() {
        assert!(!ActionsDialog::fuzzy_match("hello", "xyz"));
    }

    #[test]
    fn cat13_empty_needle_matches() {
        assert!(ActionsDialog::fuzzy_match("hello", ""));
    }

    #[test]
    fn cat13_empty_haystack_no_match() {
        assert!(!ActionsDialog::fuzzy_match("", "a"));
    }

    #[test]
    fn cat13_both_empty() {
        assert!(ActionsDialog::fuzzy_match("", ""));
    }

    #[test]
    fn cat13_needle_longer_fails() {
        assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
    }

    // =========================================================================
    // Category 14: parse_shortcut_keycaps splits correctly
    // =========================================================================

    #[test]
    fn cat14_cmd_c_two_caps() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘C");
        assert_eq!(caps.len(), 2);
        assert_eq!(caps[0], "⌘");
        assert_eq!(caps[1], "C");
    }

    #[test]
    fn cat14_cmd_shift_c_three_caps() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘⇧C");
        assert_eq!(caps.len(), 3);
    }

    #[test]
    fn cat14_enter_single() {
        let caps = ActionsDialog::parse_shortcut_keycaps("↵");
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0], "↵");
    }

    #[test]
    fn cat14_arrows() {
        let caps = ActionsDialog::parse_shortcut_keycaps("↑↓←→");
        assert_eq!(caps.len(), 4);
    }

    #[test]
    fn cat14_escape() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⎋");
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0], "⎋");
    }

    // =========================================================================
    // Category 15: build_grouped_items_static with Headers style
    // =========================================================================

    #[test]
    fn cat15_headers_add_section_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Should have: Header("S1"), Item(0), Header("S2"), Item(1)
        assert_eq!(grouped.len(), 4);
        assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(_)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[2], GroupedActionItem::SectionHeader(_)));
        assert!(matches!(grouped[3], GroupedActionItem::Item(1)));
    }

    #[test]
    fn cat15_same_section_no_duplicate_header() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S1"),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Should have: Header("S1"), Item(0), Item(1) — only one header
        assert_eq!(grouped.len(), 3);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 1);
    }

    #[test]
    fn cat15_separators_no_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 0);
    }

    #[test]
    fn cat15_empty_filtered_empty_result() {
        let actions = vec![Action::new("a", "A", None, ActionCategory::ScriptContext)];
        let filtered: Vec<usize> = vec![];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(grouped.is_empty());
    }

    #[test]
    fn cat15_none_style_no_headers() {
        let actions =
            vec![Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1")];
        let filtered: Vec<usize> = vec![0];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 0);
    }

    // =========================================================================
    // Category 16: coerce_action_selection skips headers
    // =========================================================================

    #[test]
    fn cat16_item_stays() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    }

    #[test]
    fn cat16_header_skips_down() {
        let rows = vec![
            GroupedActionItem::SectionHeader("S".into()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn cat16_trailing_header_skips_up() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("S".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(0));
    }

    #[test]
    fn cat16_all_headers_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("S1".into()),
            GroupedActionItem::SectionHeader("S2".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn cat16_empty_rows_none() {
        let rows: Vec<GroupedActionItem> = vec![];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    // =========================================================================
    // Category 17: CommandBarConfig preset field matrix
    // =========================================================================

    #[test]
    fn cat17_default_close_on_select() {
        let config = CommandBarConfig::default();
        assert!(config.close_on_select);
        assert!(config.close_on_escape);
    }

    #[test]
    fn cat17_ai_style_top_search() {
        let config = CommandBarConfig::ai_style();
        assert!(matches!(
            config.dialog_config.search_position,
            SearchPosition::Top
        ));
    }

    #[test]
    fn cat17_main_menu_bottom_search() {
        let config = CommandBarConfig::main_menu_style();
        assert!(matches!(
            config.dialog_config.search_position,
            SearchPosition::Bottom
        ));
    }

    #[test]
    fn cat17_no_search_hidden() {
        let config = CommandBarConfig::no_search();
        assert!(matches!(
            config.dialog_config.search_position,
            SearchPosition::Hidden
        ));
    }

    #[test]
    fn cat17_notes_style_separators() {
        let config = CommandBarConfig::notes_style();
        assert!(matches!(
            config.dialog_config.section_style,
            SectionStyle::Separators
        ));
        assert!(config.dialog_config.show_icons);
        assert!(config.dialog_config.show_footer);
    }

    // =========================================================================
    // Category 18: Action builder with_shortcut_opt behavior
    // =========================================================================

    #[test]
    fn cat18_with_shortcut_opt_none_preserves() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘A")
            .with_shortcut_opt(None);
        // None should not clear the existing shortcut
        assert_eq!(action.shortcut, Some("⌘A".to_string()));
    }

    #[test]
    fn cat18_with_shortcut_opt_some_sets() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘B".to_string()));
        assert_eq!(action.shortcut, Some("⌘B".to_string()));
    }

    #[test]
    fn cat18_with_icon_preserves_shortcut() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘C")
            .with_icon(IconName::Copy);
        assert_eq!(action.shortcut, Some("⌘C".to_string()));
        assert_eq!(action.icon, Some(IconName::Copy));
    }

    #[test]
    fn cat18_with_section_preserves_all() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘D")
            .with_icon(IconName::Plus)
            .with_section("MySection");
        assert_eq!(action.shortcut, Some("⌘D".to_string()));
        assert_eq!(action.icon, Some(IconName::Plus));
        assert_eq!(action.section, Some("MySection".to_string()));
    }

    // =========================================================================
    // Category 19: ScriptInfo with_is_script vs with_action_verb
    // =========================================================================

    #[test]
    fn cat19_with_is_script_true() {
        let s = ScriptInfo::with_is_script("test", "/path", true);
        assert!(s.is_script);
        assert_eq!(s.action_verb, "Run");
    }

    #[test]
    fn cat19_with_is_script_false() {
        let s = ScriptInfo::with_is_script("app", "/app", false);
        assert!(!s.is_script);
    }

    #[test]
    fn cat19_with_action_verb_custom() {
        let s = ScriptInfo::with_action_verb("Window", "w:1", false, "Switch to");
        assert_eq!(s.action_verb, "Switch to");
    }

    #[test]
    fn cat19_with_action_verb_and_shortcut() {
        let s = ScriptInfo::with_action_verb_and_shortcut(
            "W",
            "w:1",
            false,
            "Launch",
            Some("cmd+l".to_string()),
        );
        assert_eq!(s.action_verb, "Launch");
        assert_eq!(s.shortcut, Some("cmd+l".to_string()));
    }

    // =========================================================================
    // Category 20: Notes command bar conditional actions per flags
    // =========================================================================

    #[test]
    fn cat20_duplicate_note_requires_selection_no_trash() {
        let yes = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&yes);
        assert!(actions.iter().any(|a| a.id == "duplicate_note"));

        let no_sel = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions2 = get_notes_command_bar_actions(&no_sel);
        assert!(!actions2.iter().any(|a| a.id == "duplicate_note"));
    }

    #[test]
    fn cat20_find_in_note_absent_in_trash() {
        let trash = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&trash);
        assert!(!actions.iter().any(|a| a.id == "find_in_note"));
    }

    #[test]
    fn cat20_export_requires_selection_no_trash() {
        let yes = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&yes);
        assert!(actions.iter().any(|a| a.id == "export"));

        let no = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions2 = get_notes_command_bar_actions(&no);
        assert!(!actions2.iter().any(|a| a.id == "export"));
    }

    #[test]
    fn cat20_auto_sizing_absent_when_enabled() {
        let enabled = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&enabled);
        assert!(!actions.iter().any(|a| a.id == "enable_auto_sizing"));
    }

    #[test]
    fn cat20_auto_sizing_present_when_disabled() {
        let disabled = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&disabled);
        assert!(actions.iter().any(|a| a.id == "enable_auto_sizing"));
    }

    // =========================================================================
    // Category 21: Clipboard share and attach_to_ai universality
    // Both text and image entries should have these actions.
    // =========================================================================

    #[test]
    fn cat21_text_has_share() {
        let entry = ClipboardEntryInfo {
            id: "t".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hi".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_share"));
    }

    #[test]
    fn cat21_image_has_share() {
        let entry = ClipboardEntryInfo {
            id: "i".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "img".into(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_share"));
    }

    #[test]
    fn cat21_text_has_attach_to_ai() {
        let entry = ClipboardEntryInfo {
            id: "t".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hi".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_attach_to_ai"));
    }

    #[test]
    fn cat21_image_has_attach_to_ai() {
        let entry = ClipboardEntryInfo {
            id: "i".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "img".into(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_attach_to_ai"));
    }

    #[test]
    fn cat21_share_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "t".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hi".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let s = actions.iter().find(|a| a.id == "clipboard_share").unwrap();
        assert_eq!(s.shortcut.as_ref().unwrap(), "⇧⌘E");
    }

    // =========================================================================
    // Category 22: File context directory lacks quick_look (macOS)
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn cat22_file_dir_no_quick_look() {
        let file_info = FileInfo {
            path: "/test/Dir".into(),
            name: "Dir".into(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&file_info);
        assert!(!actions.iter().any(|a| a.id == "quick_look"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat22_file_file_has_quick_look() {
        let file_info = FileInfo {
            path: "/test/f.txt".into(),
            name: "f.txt".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        assert!(actions.iter().any(|a| a.id == "quick_look"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat22_file_dir_has_open_with() {
        let file_info = FileInfo {
            path: "/test/Dir".into(),
            name: "Dir".into(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&file_info);
        assert!(actions.iter().any(|a| a.id == "open_with"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat22_file_dir_has_show_info() {
        let file_info = FileInfo {
            path: "/test/Dir".into(),
            name: "Dir".into(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&file_info);
        assert!(actions.iter().any(|a| a.id == "show_info"));
    }

    // =========================================================================
    // Category 23: Scriptlet defined actions from H3 headers
    // =========================================================================

    #[test]
    fn cat23_empty_scriptlet_no_actions() {
        let scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert!(actions.is_empty());
    }

    #[test]
    fn cat23_single_action_has_action_true() {
        let mut scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Copy".into(),
            command: "copy".into(),
            tool: "bash".into(),
            code: "echo | pbcopy".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions.len(), 1);
        assert!(actions[0].has_action);
    }

    #[test]
    fn cat23_action_id_prefix() {
        let mut scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Do Thing".into(),
            command: "do-thing".into(),
            tool: "bash".into(),
            code: "echo".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert!(actions[0].id.starts_with("scriptlet_action:"));
    }

    #[test]
    fn cat23_action_value_is_command() {
        let mut scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Do".into(),
            command: "my-cmd".into(),
            tool: "bash".into(),
            code: "echo".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions[0].value, Some("my-cmd".to_string()));
    }

    #[test]
    fn cat23_action_shortcut_formatted() {
        let mut scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Copy".into(),
            command: "copy".into(),
            tool: "bash".into(),
            code: "echo".into(),
            inputs: vec![],
            shortcut: Some("cmd+c".into()),
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions[0].shortcut, Some("⌘C".to_string()));
    }

    // =========================================================================
    // Category 24: Script context run_script title includes action_verb
    // =========================================================================

    #[test]
    fn cat24_default_verb_run() {
        let script = ScriptInfo::new("Test", "/test.ts");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert!(run.title.starts_with("Run"));
    }

    #[test]
    fn cat24_custom_verb_launch() {
        let script = ScriptInfo::with_action_verb("Safari", "/app", false, "Launch");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert!(run.title.starts_with("Launch"));
    }

    #[test]
    fn cat24_custom_verb_switch_to() {
        let script = ScriptInfo::with_action_verb("Window", "w:1", false, "Switch to");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert!(run.title.starts_with("Switch to"));
    }

    #[test]
    fn cat24_title_contains_name_in_quotes() {
        let script = ScriptInfo::new("My Script", "/test.ts");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert!(run.title.contains("\"My Script\""));
    }

    // =========================================================================
    // Category 25: Notes command bar section assignments
    // =========================================================================

    #[test]
    fn cat25_new_note_section_notes() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let nn = actions.iter().find(|a| a.id == "new_note").unwrap();
        assert_eq!(nn.section.as_ref().unwrap(), "Notes");
    }

    #[test]
    fn cat25_find_in_note_section_edit() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let fin = actions.iter().find(|a| a.id == "find_in_note").unwrap();
        assert_eq!(fin.section.as_ref().unwrap(), "Edit");
    }

    #[test]
    fn cat25_format_section_edit() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let fmt = actions.iter().find(|a| a.id == "format").unwrap();
        assert_eq!(fmt.section.as_ref().unwrap(), "Edit");
    }

    #[test]
    fn cat25_copy_note_as_section_copy() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let cna = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
        assert_eq!(cna.section.as_ref().unwrap(), "Copy");
    }

    #[test]
    fn cat25_export_section_export() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let exp = actions.iter().find(|a| a.id == "export").unwrap();
        assert_eq!(exp.section.as_ref().unwrap(), "Export");
    }

    // =========================================================================
    // Category 26: AI command bar icon assignments
    // =========================================================================

    #[test]
    fn cat26_copy_response_icon_copy() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "copy_response").unwrap();
        assert_eq!(a.icon, Some(IconName::Copy));
    }

    #[test]
    fn cat26_submit_icon_arrow_up() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "submit").unwrap();
        assert_eq!(a.icon, Some(IconName::ArrowUp));
    }

    #[test]
    fn cat26_new_chat_icon_plus() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "new_chat").unwrap();
        assert_eq!(a.icon, Some(IconName::Plus));
    }

    #[test]
    fn cat26_delete_chat_icon_trash() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "delete_chat").unwrap();
        assert_eq!(a.icon, Some(IconName::Trash));
    }

    #[test]
    fn cat26_export_markdown_icon_filecode() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "export_markdown").unwrap();
        assert_eq!(a.icon, Some(IconName::FileCode));
    }

    // =========================================================================
    // Category 27: Note switcher icon priority hierarchy
    // pinned > current > regular
    // =========================================================================

    #[test]
    fn cat27_pinned_icon_star_filled() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "N".into(),
            char_count: 0,
            is_current: false,
            is_pinned: true,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    #[test]
    fn cat27_current_icon_check() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "N".into(),
            char_count: 0,
            is_current: true,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::Check));
    }

    #[test]
    fn cat27_regular_icon_file() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "N".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::File));
    }

    #[test]
    fn cat27_pinned_and_current_icon_star() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "N".into(),
            char_count: 0,
            is_current: true,
            is_pinned: true,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        // Pinned takes priority over current
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    // =========================================================================
    // Category 28: Note switcher empty state placeholder
    // =========================================================================

    #[test]
    fn cat28_empty_notes_single_placeholder() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions.len(), 1);
    }

    #[test]
    fn cat28_empty_notes_id_no_notes() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions[0].id, "no_notes");
    }

    #[test]
    fn cat28_empty_notes_title() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions[0].title, "No notes yet");
    }

    #[test]
    fn cat28_empty_notes_icon_plus() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions[0].icon, Some(IconName::Plus));
    }

    #[test]
    fn cat28_empty_notes_section_notes() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions[0].section.as_ref().unwrap(), "Notes");
    }

    // =========================================================================
    // Category 29: Cross-context all descriptions non-empty
    // =========================================================================

    #[test]
    fn cat29_script_all_have_descriptions() {
        let script = ScriptInfo::new("T", "/t.ts");
        let actions = get_script_context_actions(&script);
        for a in &actions {
            assert!(
                a.description.is_some(),
                "Script action '{}' missing description",
                a.id
            );
        }
    }

    #[test]
    fn cat29_clipboard_all_have_descriptions() {
        let entry = ClipboardEntryInfo {
            id: "t".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hi".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        for a in &actions {
            assert!(
                a.description.is_some(),
                "Clipboard action '{}' missing description",
                a.id
            );
        }
    }

    #[test]
    fn cat29_ai_all_have_descriptions() {
        let actions = get_ai_command_bar_actions();
        for a in &actions {
            assert!(
                a.description.is_some(),
                "AI action '{}' missing description",
                a.id
            );
        }
    }

    #[test]
    fn cat29_path_all_have_descriptions() {
        let path = PathInfo {
            path: "/t".into(),
            name: "t".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        for a in &actions {
            assert!(
                a.description.is_some(),
                "Path action '{}' missing description",
                a.id
            );
        }
    }

    #[test]
    fn cat29_file_all_have_descriptions() {
        let file = FileInfo {
            path: "/t".into(),
            name: "t".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        for a in &actions {
            assert!(
                a.description.is_some(),
                "File action '{}' missing description",
                a.id
            );
        }
    }

    #[test]
    fn cat29_notes_all_have_descriptions() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for a in &actions {
            assert!(
                a.description.is_some(),
                "Notes action '{}' missing description",
                a.id
            );
        }
    }

    // =========================================================================
    // Category 30: Cross-context ID uniqueness and snake_case invariant
    // =========================================================================

    #[test]
    fn cat30_script_ids_unique() {
        let script = ScriptInfo::new("T", "/t.ts");
        let actions = get_script_context_actions(&script);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat30_clipboard_text_ids_unique() {
        let entry = ClipboardEntryInfo {
            id: "t".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hi".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat30_ai_ids_unique() {
        let actions = get_ai_command_bar_actions();
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat30_path_ids_unique() {
        let path = PathInfo {
            path: "/t".into(),
            name: "t".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat30_builtin_ids_snake_case() {
        let script = ScriptInfo::new("T", "/t.ts");
        let actions = get_script_context_actions(&script);
        for a in &actions {
            assert!(
                !a.id.contains(' ') && !a.id.contains('-'),
                "Action ID '{}' should be snake_case",
                a.id
            );
        }
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
        assert_eq!(ids.len(), actions.len());
    }
}
