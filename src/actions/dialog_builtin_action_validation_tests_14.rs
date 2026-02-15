//! Batch 14 — Builtin action validation tests
//!
//! Focus areas:
//! - Path context action ordering and position guarantees
//! - Clipboard OCR/open_with mutual exclusivity between text/image
//! - ScriptInfo constructor field isolation (with_all sets exactly 6 params)
//! - AI command bar exact icon-to-section exhaustive mapping
//! - Notes command bar section-action membership
//! - Note switcher title rendering (Untitled Note, bullet prefix, pinned override)
//! - Chat context edge: zero models with both flags false
//! - Scriptlet context custom action insertion ordering with multiple H3 actions
//! - File context macOS-only action count delta
//! - Cross-context description non-emptiness and keyword matching
//! - build_grouped_items_static with mixed sections and alternating patterns
//! - coerce_action_selection with interleaved headers
//! - score_action stacking: title + description + shortcut all match
//! - fuzzy_match Unicode subsequence
//! - parse_shortcut_keycaps with slash and number inputs
//! - to_deeplink_name with numeric-only and empty-after-strip inputs
//! - Action with_shortcut_opt Some vs None chaining
//! - CommandBarConfig notes_style field completeness
//! - Clipboard destructive ordering invariant across pin states
//! - Global actions always empty

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

    // =========================================================================
    // Helper
    // =========================================================================
    fn action_ids(actions: &[Action]) -> Vec<String> {
        actions.iter().map(|a| a.id.clone()).collect()
    }

    // =========================================================================
    // 1. Path context action ordering: primary first, trash last
    // =========================================================================

    #[test]
    fn cat01_path_dir_primary_is_first_action() {
        let info = PathInfo {
            path: "/Users/test/Documents".to_string(),
            name: "Documents".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].id, "file:open_directory");
    }

    #[test]
    fn cat01_path_file_primary_is_first_action() {
        let info = PathInfo {
            path: "/Users/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].id, "file:select_file");
    }

    #[test]
    fn cat01_path_trash_is_always_last() {
        for is_dir in [true, false] {
            let info = PathInfo {
                path: "/test/item".to_string(),
                name: "item".to_string(),
                is_dir,
            };
            let actions = get_path_context_actions(&info);
            assert_eq!(
                actions.last().unwrap().id,
                "file:move_to_trash",
                "move_to_trash should be last for is_dir={}",
                is_dir
            );
        }
    }

    #[test]
    fn cat01_path_dir_and_file_same_action_count() {
        let dir_info = PathInfo {
            path: "/test/dir".to_string(),
            name: "dir".to_string(),
            is_dir: true,
        };
        let file_info = PathInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
        };
        assert_eq!(
            get_path_context_actions(&dir_info).len(),
            get_path_context_actions(&file_info).len()
        );
    }

    #[test]
    fn cat01_path_common_actions_always_present() {
        let info = PathInfo {
            path: "/test/thing".to_string(),
            name: "thing".to_string(),
            is_dir: false,
        };
        let ids = action_ids(&get_path_context_actions(&info));
        for expected in &[
            "file:copy_path",
            "file:open_in_finder",
            "file:open_in_editor",
            "file:open_in_terminal",
            "file:copy_filename",
            "file:move_to_trash",
        ] {
            assert!(
                ids.contains(&expected.to_string()),
                "Missing expected action: {}",
                expected
            );
        }
    }

    #[test]
    fn cat01_path_dir_title_contains_name() {
        let info = PathInfo {
            path: "/test/MyFolder".to_string(),
            name: "MyFolder".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        assert!(actions[0].title.contains("MyFolder"));
    }

    // =========================================================================
    // 2. Clipboard OCR only on image, not text
    // =========================================================================

    #[test]
    fn cat02_clipboard_text_has_no_ocr() {
        let entry = ClipboardEntryInfo {
            id: "t1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let ids = action_ids(&get_clipboard_history_context_actions(&entry));
        assert!(!ids.contains(&"clip:clipboard_ocr".to_string()));
    }

    #[test]
    fn cat02_clipboard_image_has_ocr() {
        let entry = ClipboardEntryInfo {
            id: "i1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "img".to_string(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let ids = action_ids(&get_clipboard_history_context_actions(&entry));
        assert!(ids.contains(&"clip:clipboard_ocr".to_string()));
    }

    #[test]
    fn cat02_clipboard_image_has_more_actions_than_text() {
        let text_entry = ClipboardEntryInfo {
            id: "t".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let img_entry = ClipboardEntryInfo {
            id: "i".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "y".to_string(),
            image_dimensions: Some((10, 10)),
            frontmost_app_name: None,
        };
        let text_count = get_clipboard_history_context_actions(&text_entry).len();
        let img_count = get_clipboard_history_context_actions(&img_entry).len();
        assert!(
            img_count > text_count,
            "Image ({}) should have more actions than text ({})",
            img_count,
            text_count
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat02_clipboard_text_no_open_with_macos() {
        let entry = ClipboardEntryInfo {
            id: "t2".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let ids = action_ids(&get_clipboard_history_context_actions(&entry));
        assert!(!ids.contains(&"clip:clipboard_open_with".to_string()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat02_clipboard_image_has_open_with_macos() {
        let entry = ClipboardEntryInfo {
            id: "i2".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "img".to_string(),
            image_dimensions: Some((50, 50)),
            frontmost_app_name: None,
        };
        let ids = action_ids(&get_clipboard_history_context_actions(&entry));
        assert!(ids.contains(&"clip:clipboard_open_with".to_string()));
    }

    // =========================================================================
    // 3. ScriptInfo::with_all sets exactly the right fields
    // =========================================================================

    #[test]
    fn cat03_with_all_sets_name_and_path() {
        let s = ScriptInfo::with_all("Foo", "/bar", true, "Run", None, None);
        assert_eq!(s.name, "Foo");
        assert_eq!(s.path, "/bar");
    }

    #[test]
    fn cat03_with_all_sets_is_script() {
        let s_true = ScriptInfo::with_all("A", "/a", true, "Run", None, None);
        assert!(s_true.is_script);
        let s_false = ScriptInfo::with_all("B", "/b", false, "Run", None, None);
        assert!(!s_false.is_script);
    }

    #[test]
    fn cat03_with_all_sets_verb() {
        let s = ScriptInfo::with_all("X", "/x", false, "Execute", None, None);
        assert_eq!(s.action_verb, "Execute");
    }

    #[test]
    fn cat03_with_all_sets_shortcut_and_alias() {
        let s = ScriptInfo::with_all(
            "Y",
            "/y",
            false,
            "Open",
            Some("cmd+y".into()),
            Some("yy".into()),
        );
        assert_eq!(s.shortcut, Some("cmd+y".to_string()));
        assert_eq!(s.alias, Some("yy".to_string()));
    }

    #[test]
    fn cat03_with_all_defaults_no_agent_no_scriptlet() {
        let s = ScriptInfo::with_all("Z", "/z", true, "Run", None, None);
        assert!(!s.is_agent);
        assert!(!s.is_scriptlet);
        assert!(!s.is_suggested);
        assert!(s.frecency_path.is_none());
    }

    // =========================================================================
    // 4. AI command bar exact icon-to-section mapping
    // =========================================================================

    #[test]
    fn cat04_ai_response_section_has_3_actions() {
        let actions = get_ai_command_bar_actions();
        let response_actions: Vec<_> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Response"))
            .collect();
        assert_eq!(response_actions.len(), 3);
    }

    #[test]
    fn cat04_ai_actions_section_has_4_actions() {
        let actions = get_ai_command_bar_actions();
        let section_actions: Vec<_> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Actions"))
            .collect();
        assert_eq!(section_actions.len(), 4);
    }

    #[test]
    fn cat04_ai_attachments_section_has_2_actions() {
        let actions = get_ai_command_bar_actions();
        let section_actions: Vec<_> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Attachments"))
            .collect();
        assert_eq!(section_actions.len(), 2);
    }

    #[test]
    fn cat04_ai_export_section_has_1_action() {
        let actions = get_ai_command_bar_actions();
        let section_actions: Vec<_> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Export"))
            .collect();
        assert_eq!(section_actions.len(), 1);
    }

    #[test]
    fn cat04_ai_help_section_has_1_action() {
        let actions = get_ai_command_bar_actions();
        let section_actions: Vec<_> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Help"))
            .collect();
        assert_eq!(section_actions.len(), 1);
    }

    #[test]
    fn cat04_ai_settings_section_has_1_action() {
        let actions = get_ai_command_bar_actions();
        let section_actions: Vec<_> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Settings"))
            .collect();
        assert_eq!(section_actions.len(), 1);
    }

    #[test]
    fn cat04_ai_copy_response_icon_is_copy() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "chat:copy_response").unwrap();
        assert_eq!(a.icon, Some(IconName::Copy));
    }

    #[test]
    fn cat04_ai_submit_icon_is_arrowup() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "chat:submit").unwrap();
        assert_eq!(a.icon, Some(IconName::ArrowUp));
    }

    #[test]
    fn cat04_ai_new_chat_icon_is_plus() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "chat:new_chat").unwrap();
        assert_eq!(a.icon, Some(IconName::Plus));
    }

    #[test]
    fn cat04_ai_delete_chat_icon_is_trash() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "chat:delete_chat").unwrap();
        assert_eq!(a.icon, Some(IconName::Trash));
    }

    #[test]
    fn cat04_ai_change_model_icon_is_settings() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "chat:change_model").unwrap();
        assert_eq!(a.icon, Some(IconName::Settings));
    }

    // =========================================================================
    // 5. Notes command bar section-action membership
    // =========================================================================

    #[test]
    fn cat05_notes_full_feature_notes_section_has_3() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Notes"))
            .count();
        assert_eq!(count, 3);
    }

    #[test]
    fn cat05_notes_full_feature_edit_section_has_2() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Edit"))
            .count();
        assert_eq!(count, 2);
    }

    #[test]
    fn cat05_notes_full_feature_copy_section_has_3() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Copy"))
            .count();
        assert_eq!(count, 3);
    }

    #[test]
    fn cat05_notes_full_feature_export_section_has_1() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Export"))
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn cat05_notes_trash_view_only_notes_section() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Only new_note and browse_notes in Notes section
        let sections: HashSet<_> = actions.iter().filter_map(|a| a.section.clone()).collect();
        assert_eq!(sections.len(), 1);
        assert!(sections.contains("Notes"));
    }

    #[test]
    fn cat05_notes_no_selection_minimal() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        // new_note, browse_notes, enable_auto_sizing
        assert_eq!(actions.len(), 3);
    }

    // =========================================================================
    // 6. Note switcher title rendering
    // =========================================================================


    // --- merged from tests_part_02.rs ---
    #[test]
    fn cat06_note_switcher_current_gets_bullet_prefix() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "My Note".into(),
            char_count: 100,
            is_current: true,
            is_pinned: false,
            preview: "content".into(),
            relative_time: "1m ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(actions[0].title.starts_with("• "));
    }

    #[test]
    fn cat06_note_switcher_non_current_no_bullet() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n2".into(),
            title: "Other Note".into(),
            char_count: 50,
            is_current: false,
            is_pinned: false,
            preview: "stuff".into(),
            relative_time: "5m ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(!actions[0].title.starts_with("• "));
        assert_eq!(actions[0].title, "Other Note");
    }

    #[test]
    fn cat06_note_switcher_pinned_overrides_current_icon() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n3".into(),
            title: "Both".into(),
            char_count: 10,
            is_current: true,
            is_pinned: true,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        // Pinned takes priority over current for icon
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    #[test]
    fn cat06_note_switcher_regular_icon_is_file() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n4".into(),
            title: "Regular".into(),
            char_count: 5,
            is_current: false,
            is_pinned: false,
            preview: "abc".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::File));
    }

    #[test]
    fn cat06_note_switcher_empty_shows_placeholder() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "no_notes");
        assert!(actions[0].title.contains("No notes"));
    }

    #[test]
    fn cat06_note_switcher_id_format() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc-123".into(),
            title: "Test".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].id, "note_abc-123");
    }

    // =========================================================================
    // 7. Chat context: zero models, both flags false
    // =========================================================================

    #[test]
    fn cat07_chat_zero_models_still_has_continue() {
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
    fn cat07_chat_no_response_no_copy() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".into()),
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let ids = action_ids(&get_chat_context_actions(&info));
        assert!(!ids.contains(&"chat:copy_response".to_string()));
    }

    #[test]
    fn cat07_chat_no_messages_no_clear() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".into()),
            available_models: vec![],
            has_messages: false,
            has_response: true,
        };
        let ids = action_ids(&get_chat_context_actions(&info));
        assert!(!ids.contains(&"chat:clear_conversation".to_string()));
    }

    #[test]
    fn cat07_chat_both_flags_true_gives_max_actions() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".into()),
            available_models: vec![ChatModelInfo {
                id: "m1".into(),
                display_name: "Claude".into(),
                provider: "Anthropic".into(),
            }],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        // 1 model + continue + copy_response + clear = 4
        assert_eq!(actions.len(), 4);
    }

    #[test]
    fn cat07_chat_current_model_gets_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".into()),
            available_models: vec![
                ChatModelInfo {
                    id: "m1".into(),
                    display_name: "Claude".into(),
                    provider: "Anthropic".into(),
                },
                ChatModelInfo {
                    id: "m2".into(),
                    display_name: "GPT-4".into(),
                    provider: "OpenAI".into(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let claude = actions.iter().find(|a| a.id == "chat:select_model_m1").unwrap();
        assert!(claude.title.contains("✓"));
        let gpt = actions.iter().find(|a| a.id == "chat:select_model_m2").unwrap();
        assert!(!gpt.title.contains("✓"));
    }

    // =========================================================================
    // 8. Scriptlet context custom action ordering with multiple H3 actions
    // =========================================================================

    #[test]
    fn cat08_scriptlet_custom_actions_appear_after_run() {
        let script = ScriptInfo::scriptlet("Test", "/test.md", None, None);
        let mut scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo main".to_string(),
        );
        scriptlet.actions = vec![
            ScriptletAction {
                name: "Alpha".into(),
                command: "alpha".into(),
                tool: "bash".into(),
                code: "echo a".into(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
            ScriptletAction {
                name: "Beta".into(),
                command: "beta".into(),
                tool: "bash".into(),
                code: "echo b".into(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
        ];
        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let ids = action_ids(&actions);
        let run_idx = ids.iter().position(|id| id == "run_script").unwrap();
        let alpha_idx = ids
            .iter()
            .position(|id| id == "scriptlet_action:alpha")
            .unwrap();
        let beta_idx = ids
            .iter()
            .position(|id| id == "scriptlet_action:beta")
            .unwrap();
        assert!(run_idx < alpha_idx);
        assert!(alpha_idx < beta_idx);
    }

    #[test]
    fn cat08_scriptlet_custom_actions_before_builtins() {
        let script = ScriptInfo::scriptlet("Test", "/test.md", None, None);
        let mut scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo main".to_string(),
        );
        scriptlet.actions = vec![ScriptletAction {
            name: "Custom".into(),
            command: "custom".into(),
            tool: "bash".into(),
            code: "echo c".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let ids = action_ids(&actions);
        let custom_idx = ids
            .iter()
            .position(|id| id == "scriptlet_action:custom")
            .unwrap();
        let edit_idx = ids.iter().position(|id| id == "edit_scriptlet").unwrap();
        assert!(custom_idx < edit_idx);
    }

    #[test]
    fn cat08_scriptlet_custom_has_action_true() {
        let mut scriptlet =
            Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());
        scriptlet.actions = vec![ScriptletAction {
            name: "Do Thing".into(),
            command: "do-thing".into(),
            tool: "bash".into(),
            code: "echo do".into(),
            inputs: vec![],
            shortcut: Some("cmd+d".into()),
            description: Some("Does the thing".into()),
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert!(actions[0].has_action);
        assert_eq!(actions[0].value, Some("do-thing".to_string()));
    }

    #[test]
    fn cat08_scriptlet_custom_shortcut_formatted() {
        let mut scriptlet =
            Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());
        scriptlet.actions = vec![ScriptletAction {
            name: "Copy".into(),
            command: "copy".into(),
            tool: "bash".into(),
            code: "echo cp".into(),
            inputs: vec![],
            shortcut: Some("cmd+shift+c".into()),
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions[0].shortcut, Some("⌘⇧C".to_string()));
    }

    #[test]
    fn cat08_scriptlet_no_actions_returns_empty() {
        let scriptlet = Scriptlet::new("Empty".to_string(), "bash".to_string(), "echo".to_string());
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert!(actions.is_empty());
    }

    // =========================================================================
    // 9. File context macOS-only action count delta
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn cat09_file_macos_has_quick_look_open_with_show_info() {
        let info = FileInfo {
            path: "/test/file.txt".into(),
            name: "file.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let ids = action_ids(&get_file_context_actions(&info));
        assert!(ids.contains(&"file:quick_look".to_string()));
        assert!(ids.contains(&"file:open_with".to_string()));
        assert!(ids.contains(&"file:show_info".to_string()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat09_file_dir_no_quick_look_macos() {
        let info = FileInfo {
            path: "/test/dir".into(),
            name: "dir".into(),
            file_type: crate::file_search::FileType::Directory,
            is_dir: true,
        };
        let ids = action_ids(&get_file_context_actions(&info));
        assert!(!ids.contains(&"file:quick_look".to_string()));
        // But still has open_with and show_info
        assert!(ids.contains(&"file:open_with".to_string()));
        assert!(ids.contains(&"file:show_info".to_string()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat09_file_macos_file_more_actions_than_dir() {
        let file_info = FileInfo {
            path: "/test/f.txt".into(),
            name: "f.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let dir_info = FileInfo {
            path: "/test/d".into(),
            name: "d".into(),
            file_type: crate::file_search::FileType::Directory,
            is_dir: true,
        };
        let file_count = get_file_context_actions(&file_info).len();
        let dir_count = get_file_context_actions(&dir_info).len();
        assert!(
            file_count > dir_count,
            "File ({}) should have more actions than dir ({}) on macOS",
            file_count,
            dir_count
        );
    }

    // =========================================================================
    // 10. Cross-context description non-emptiness
    // =========================================================================

    #[test]
    fn cat10_script_context_all_have_descriptions() {
        let script = ScriptInfo::new("test", "/test.ts");
        for action in &get_script_context_actions(&script) {
            assert!(
                action.description.is_some(),
                "Script action '{}' should have a description",
                action.id
            );
        }
    }

    #[test]
    fn cat10_ai_command_bar_all_have_descriptions() {
        for action in &get_ai_command_bar_actions() {
            assert!(
                action.description.is_some(),
                "AI action '{}' should have a description",
                action.id
            );
        }
    }

    #[test]
    fn cat10_path_context_all_have_descriptions() {
        let info = PathInfo {
            path: "/test".into(),
            name: "test".into(),
            is_dir: false,
        };
        for action in &get_path_context_actions(&info) {
            assert!(
                action.description.is_some(),
                "Path action '{}' should have a description",
                action.id
            );
        }
    }

    #[test]
    fn cat10_file_context_all_have_descriptions() {
        let info = FileInfo {
            path: "/test/f.txt".into(),
            name: "f.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        for action in &get_file_context_actions(&info) {
            assert!(
                action.description.is_some(),
                "File action '{}' should have a description",
                action.id
            );
        }
    }

    #[test]
    fn cat10_clipboard_context_all_have_descriptions() {
        let entry = ClipboardEntryInfo {
            id: "c1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        for action in &get_clipboard_history_context_actions(&entry) {
            assert!(
                action.description.is_some(),
                "Clipboard action '{}' should have a description",
                action.id
            );
        }
    }

    #[test]
    fn cat10_notes_context_all_have_descriptions() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for action in &get_notes_command_bar_actions(&info) {
            assert!(
                action.description.is_some(),
                "Notes action '{}' should have a description",
                action.id
            );
        }
    }

    // =========================================================================
    // 11. build_grouped_items_static with mixed sections
    // =========================================================================


    // --- merged from tests_part_03.rs ---
    #[test]
    fn cat11_grouped_items_headers_insert_for_each_section_change() {
        let actions = vec![
            Action::new("a1", "A1", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("a2", "A2", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("a3", "A3", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered: Vec<usize> = (0..3).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // S1 header, a1, a2, S2 header, a3
        assert_eq!(grouped.len(), 5);
        assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(_)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[2], GroupedActionItem::Item(1)));
        assert!(matches!(grouped[3], GroupedActionItem::SectionHeader(_)));
        assert!(matches!(grouped[4], GroupedActionItem::Item(2)));
    }

    #[test]
    fn cat11_grouped_items_separators_no_headers() {
        let actions = vec![
            Action::new("a1", "A1", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("a2", "A2", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        assert_eq!(grouped.len(), 2);
        assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(1)));
    }

    #[test]
    fn cat11_grouped_items_none_style_no_headers() {
        let actions = vec![
            Action::new("a1", "A1", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("a2", "A2", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        assert_eq!(grouped.len(), 2);
    }

    #[test]
    fn cat11_grouped_items_empty_input() {
        let actions: Vec<Action> = vec![];
        let filtered: Vec<usize> = vec![];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(grouped.is_empty());
    }

    #[test]
    fn cat11_grouped_items_no_section_no_header() {
        let actions = vec![
            Action::new("a1", "A1", None, ActionCategory::ScriptContext),
            Action::new("a2", "A2", None, ActionCategory::ScriptContext),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // No sections means no headers
        assert_eq!(grouped.len(), 2);
    }

    // =========================================================================
    // 12. coerce_action_selection with interleaved headers
    // =========================================================================

    #[test]
    fn cat12_coerce_empty_returns_none() {
        assert_eq!(coerce_action_selection(&[], 0), None);
    }

    #[test]
    fn cat12_coerce_on_item_returns_same() {
        let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
        assert_eq!(coerce_action_selection(&rows, 0), Some(0));
        assert_eq!(coerce_action_selection(&rows, 1), Some(1));
    }

    #[test]
    fn cat12_coerce_header_searches_down() {
        let rows = vec![
            GroupedActionItem::SectionHeader("H".into()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn cat12_coerce_trailing_header_searches_up() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("H".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(0));
    }

    #[test]
    fn cat12_coerce_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("H1".into()),
            GroupedActionItem::SectionHeader("H2".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn cat12_coerce_out_of_bounds_clamped() {
        let rows = vec![
            GroupedActionItem::SectionHeader("H".into()),
            GroupedActionItem::Item(0),
        ];
        // Index 99 should be clamped to last index (1), which is an Item
        assert_eq!(coerce_action_selection(&rows, 99), Some(1));
    }

    #[test]
    fn cat12_coerce_interleaved_headers() {
        let rows = vec![
            GroupedActionItem::SectionHeader("S1".into()),
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("S2".into()),
            GroupedActionItem::Item(1),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
        assert_eq!(coerce_action_selection(&rows, 2), Some(3));
    }

    // =========================================================================
    // 13. score_action stacking: title + description + shortcut all match
    // =========================================================================

    #[test]
    fn cat13_score_prefix_plus_desc_plus_shortcut() {
        let action = Action::new(
            "file:copy_path",
            "Copy Path",
            Some("Copy the full path".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧C");
        // "copy" matches: prefix title (100) + description (15) = 115
        let score = ActionsDialog::score_action(&action, "copy");
        assert!(
            score >= 115,
            "Expected >=115 for prefix+desc match, got {}",
            score
        );
    }

    #[test]
    fn cat13_score_contains_title_only() {
        let action = Action::new(
            "x",
            "Open Copy Path",
            Some("unrelated".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "copy");
        assert_eq!(score, 50);
    }

    #[test]
    fn cat13_score_fuzzy_title() {
        let action = Action::new(
            "x",
            "Create Proxy",
            Some("unrelated".to_string()),
            ActionCategory::ScriptContext,
        );
        // "cry" is a subsequence of "create proxy"
        let score = ActionsDialog::score_action(&action, "cry");
        assert!(score >= 25, "Expected fuzzy match >=25, got {}", score);
    }

    #[test]
    fn cat13_score_description_only() {
        let action = Action::new(
            "x",
            "Unrelated Title",
            Some("Copy the path to clipboard".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "clipboard");
        assert_eq!(score, 15);
    }

    #[test]
    fn cat13_score_no_match_zero() {
        let action = Action::new(
            "x",
            "Edit Script",
            Some("Open in editor".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘E");
        let score = ActionsDialog::score_action(&action, "zzzzz");
        assert_eq!(score, 0);
    }

    // =========================================================================
    // 14. fuzzy_match Unicode subsequence
    // =========================================================================

    #[test]
    fn cat14_fuzzy_match_ascii_subsequence() {
        assert!(ActionsDialog::fuzzy_match("hello world", "hlo"));
    }

    #[test]
    fn cat14_fuzzy_match_exact() {
        assert!(ActionsDialog::fuzzy_match("exact", "exact"));
    }

    #[test]
    fn cat14_fuzzy_match_empty_needle() {
        assert!(ActionsDialog::fuzzy_match("anything", ""));
    }

    #[test]
    fn cat14_fuzzy_match_no_match() {
        assert!(!ActionsDialog::fuzzy_match("abc", "xyz"));
    }

    #[test]
    fn cat14_fuzzy_match_needle_longer() {
        assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
    }

    #[test]
    fn cat14_fuzzy_match_both_empty() {
        assert!(ActionsDialog::fuzzy_match("", ""));
    }

    #[test]
    fn cat14_fuzzy_match_empty_haystack_nonempty_needle() {
        assert!(!ActionsDialog::fuzzy_match("", "a"));
    }

    // =========================================================================
    // 15. parse_shortcut_keycaps with slash and number inputs
    // =========================================================================

    #[test]
    fn cat15_parse_keycaps_cmd_slash() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘/");
        assert_eq!(keycaps, vec!["⌘", "/"]);
    }

    #[test]
    fn cat15_parse_keycaps_number() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘1");
        assert_eq!(keycaps, vec!["⌘", "1"]);
    }

    #[test]
    fn cat15_parse_keycaps_modifier_chain() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⇧⌥C");
        assert_eq!(keycaps, vec!["⌘", "⇧", "⌥", "C"]);
    }

    #[test]
    fn cat15_parse_keycaps_enter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("↵");
        assert_eq!(keycaps, vec!["↵"]);
    }

    #[test]
    fn cat15_parse_keycaps_escape() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⎋");
        assert_eq!(keycaps, vec!["⎋"]);
    }

    #[test]
    fn cat15_parse_keycaps_arrows() {
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("↑"), vec!["↑"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("↓"), vec!["↓"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("←"), vec!["←"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("→"), vec!["→"]);
    }

    #[test]
    fn cat15_parse_keycaps_lowercase_uppercased() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘c");
        assert_eq!(keycaps, vec!["⌘", "C"]);
    }

    // =========================================================================
    // 16. to_deeplink_name edge cases
    // =========================================================================

    #[test]
    fn cat16_deeplink_numeric_only() {
        assert_eq!(to_deeplink_name("12345"), "12345");
    }

    #[test]
    fn cat16_deeplink_all_special_returns_empty() {
        assert_eq!(to_deeplink_name("!@#$%^&*()"), "");
    }

    #[test]
    fn cat16_deeplink_mixed_case_lowered() {
        assert_eq!(to_deeplink_name("Hello World"), "hello-world");
    }

    #[test]
    fn cat16_deeplink_consecutive_specials_collapsed() {
        assert_eq!(to_deeplink_name("a!!b"), "a-b");
    }

    #[test]
    fn cat16_deeplink_leading_trailing_stripped() {
        assert_eq!(to_deeplink_name("---hello---"), "hello");
    }

    #[test]
    fn cat16_deeplink_unicode_preserved() {
        // Unicode alphanumeric chars (like CJK) should be preserved
        let result = to_deeplink_name("café");
        assert!(result.contains("caf"));
        assert!(result.contains("é"));
    }

    #[test]
    fn cat16_deeplink_underscores_become_hyphens() {
        assert_eq!(to_deeplink_name("hello_world"), "hello-world");
    }

    // =========================================================================
    // 17. Action with_shortcut_opt Some vs None
    // =========================================================================

    #[test]
    fn cat17_with_shortcut_opt_none_preserves_none() {
        let action =
            Action::new("x", "X", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
        assert!(action.shortcut.is_none());
        assert!(action.shortcut_lower.is_none());
    }

    #[test]
    fn cat17_with_shortcut_opt_some_sets_shortcut() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘C".to_string()));
        assert_eq!(action.shortcut, Some("⌘C".to_string()));
        assert!(action.shortcut_lower.is_some());
    }

    #[test]
    fn cat17_with_shortcut_sets_lower() {
        let action =
            Action::new("x", "X", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧K");
        assert_eq!(action.shortcut_lower, Some("⌘⇧k".to_string()));
    }

    #[test]
    fn cat17_action_new_no_shortcut_lower() {
        let action = Action::new("x", "Title", None, ActionCategory::ScriptContext);
        assert!(action.shortcut.is_none());
        assert!(action.shortcut_lower.is_none());
    }

    #[test]
    fn cat17_action_title_lower_cached() {
        let action = Action::new("x", "Hello World", None, ActionCategory::ScriptContext);
        assert_eq!(action.title_lower, "hello world");
    }

    #[test]
    fn cat17_action_description_lower_cached() {
        let action = Action::new(
            "x",
            "T",
            Some("Open in Editor".to_string()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.description_lower, Some("open in editor".to_string()));
    }

    // =========================================================================
    // 18. CommandBarConfig notes_style field completeness
    // =========================================================================

    #[test]
    fn cat18_notes_style_search_at_top() {
        let cfg = CommandBarConfig::notes_style();
        assert_eq!(cfg.dialog_config.search_position, SearchPosition::Top);
    }

    #[test]
    fn cat18_notes_style_separators() {
        let cfg = CommandBarConfig::notes_style();
        assert_eq!(cfg.dialog_config.section_style, SectionStyle::Separators);
    }

    #[test]
    fn cat18_notes_style_anchor_top() {
        let cfg = CommandBarConfig::notes_style();
        assert_eq!(cfg.dialog_config.anchor, AnchorPosition::Top);
    }

    #[test]
    fn cat18_notes_style_icons_enabled() {
        let cfg = CommandBarConfig::notes_style();
        assert!(cfg.dialog_config.show_icons);
    }

    #[test]
    fn cat18_notes_style_footer_enabled() {
        let cfg = CommandBarConfig::notes_style();
        assert!(cfg.dialog_config.show_footer);
    }

    #[test]
    fn cat18_notes_style_close_defaults_true() {
        let cfg = CommandBarConfig::notes_style();
        assert!(cfg.close_on_select);
        assert!(cfg.close_on_click_outside);
        assert!(cfg.close_on_escape);
    }

    #[test]
    fn cat18_ai_style_search_at_top_headers() {
        let cfg = CommandBarConfig::ai_style();
        assert_eq!(cfg.dialog_config.search_position, SearchPosition::Top);
        assert_eq!(cfg.dialog_config.section_style, SectionStyle::Headers);
    }

    #[test]
    fn cat18_main_menu_search_at_bottom() {
        let cfg = CommandBarConfig::main_menu_style();
        assert_eq!(cfg.dialog_config.search_position, SearchPosition::Bottom);
        assert_eq!(cfg.dialog_config.section_style, SectionStyle::Separators);
    }

    #[test]
    fn cat18_no_search_hidden() {
        let cfg = CommandBarConfig::no_search();
        assert_eq!(cfg.dialog_config.search_position, SearchPosition::Hidden);
    }

    // =========================================================================
    // 19. Clipboard destructive ordering invariant across pin states
    // =========================================================================


    // --- merged from tests_part_04.rs ---
    #[test]
    fn cat19_clipboard_unpinned_last_three_destructive() {
        let entry = ClipboardEntryInfo {
            id: "u1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "txt".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let len = actions.len();
        assert_eq!(actions[len - 3].id, "clip:clipboard_delete");
        assert_eq!(actions[len - 2].id, "clip:clipboard_delete_multiple");
        assert_eq!(actions[len - 1].id, "clip:clipboard_delete_all");
    }

    #[test]
    fn cat19_clipboard_pinned_last_three_destructive() {
        let entry = ClipboardEntryInfo {
            id: "p1".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "txt".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let len = actions.len();
        assert_eq!(actions[len - 3].id, "clip:clipboard_delete");
        assert_eq!(actions[len - 2].id, "clip:clipboard_delete_multiple");
        assert_eq!(actions[len - 1].id, "clip:clipboard_delete_all");
    }

    #[test]
    fn cat19_clipboard_paste_always_first() {
        for pinned in [true, false] {
            for ct in [ContentType::Text, ContentType::Image] {
                let entry = ClipboardEntryInfo {
                    id: "x".into(),
                    content_type: ct,
                    pinned,
                    preview: "p".into(),
                    image_dimensions: if ct == ContentType::Image {
                        Some((1, 1))
                    } else {
                        None
                    },
                    frontmost_app_name: None,
                };
                let actions = get_clipboard_history_context_actions(&entry);
                assert_eq!(
                    actions[0].id, "clip:clipboard_paste",
                    "Paste should be first for pinned={} type={:?}",
                    pinned, ct
                );
            }
        }
    }

    #[test]
    fn cat19_clipboard_copy_always_second() {
        let entry = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "t".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[1].id, "clip:clipboard_copy");
    }

    // =========================================================================
    // 20. Global actions always empty
    // =========================================================================

    #[test]
    fn cat20_global_actions_empty() {
        let actions = get_global_actions();
        assert!(actions.is_empty());
    }

    // =========================================================================
    // 21. New chat action structure and ID patterns
    // =========================================================================

    #[test]
    fn cat21_new_chat_empty_inputs_empty_actions() {
        let actions = get_new_chat_actions(&[], &[], &[]);
        assert!(actions.is_empty());
    }

    #[test]
    fn cat21_new_chat_last_used_id_pattern() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].id, "last_used_0");
    }

    #[test]
    fn cat21_new_chat_preset_id_pattern() {
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].id, "preset_general");
    }

    #[test]
    fn cat21_new_chat_model_id_pattern() {
        let models = vec![NewChatModelInfo {
            model_id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "openai".into(),
            provider_display_name: "OpenAI".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].id, "model_0");
    }

    #[test]
    fn cat21_new_chat_section_ordering() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "gen".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m2".into(),
            display_name: "GPT-4".into(),
            provider: "openai".into(),
            provider_display_name: "OpenAI".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
        assert_eq!(actions[1].section.as_deref(), Some("Presets"));
        assert_eq!(actions[2].section.as_deref(), Some("Models"));
    }

    #[test]
    fn cat21_new_chat_last_used_icon_bolt() {
        let last_used = vec![NewChatModelInfo {
            model_id: "x".into(),
            display_name: "X".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
    }

    #[test]
    fn cat21_new_chat_model_icon_settings() {
        let models = vec![NewChatModelInfo {
            model_id: "x".into(),
            display_name: "X".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].icon, Some(IconName::Settings));
    }

    // =========================================================================
    // 22. format_shortcut_hint additional cases
    // =========================================================================

    #[test]
    fn cat22_format_shortcut_opt_maps_option() {
        let result = ActionsDialog::format_shortcut_hint("opt+v");
        assert_eq!(result, "⌥V");
    }

    #[test]
    fn cat22_format_shortcut_triple_modifier() {
        let result = ActionsDialog::format_shortcut_hint("cmd+shift+ctrl+a");
        assert_eq!(result, "⌘⇧⌃A");
    }

    #[test]
    fn cat22_format_shortcut_space() {
        let result = ActionsDialog::format_shortcut_hint("space");
        assert_eq!(result, "␣");
    }

    #[test]
    fn cat22_format_shortcut_tab() {
        let result = ActionsDialog::format_shortcut_hint("tab");
        assert_eq!(result, "⇥");
    }

    #[test]
    fn cat22_format_shortcut_arrowup() {
        let result = ActionsDialog::format_shortcut_hint("arrowup");
        assert_eq!(result, "↑");
    }

    #[test]
    fn cat22_format_shortcut_arrowdown() {
        let result = ActionsDialog::format_shortcut_hint("arrowdown");
        assert_eq!(result, "↓");
    }

    // =========================================================================
    // 23. Agent context actions
    // =========================================================================

    #[test]
    fn cat23_agent_has_edit_agent_title() {
        let mut script = ScriptInfo::new("My Agent", "/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    #[test]
    fn cat23_agent_has_no_view_logs() {
        let mut script = ScriptInfo::new("My Agent", "/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let ids = action_ids(&get_script_context_actions(&script));
        assert!(!ids.contains(&"view_logs".to_string()));
    }

    #[test]
    fn cat23_agent_has_copy_content() {
        let mut script = ScriptInfo::new("My Agent", "/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let ids = action_ids(&get_script_context_actions(&script));
        assert!(ids.contains(&"copy_content".to_string()));
    }

    #[test]
    fn cat23_agent_has_reveal_and_copy_path() {
        let mut script = ScriptInfo::new("My Agent", "/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let ids = action_ids(&get_script_context_actions(&script));
        assert!(ids.contains(&"file:reveal_in_finder".to_string()));
        assert!(ids.contains(&"file:copy_path".to_string()));
    }

    #[test]
    fn cat23_agent_description_mentions_agent() {
        let mut script = ScriptInfo::new("My Agent", "/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert!(edit
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("agent"));
    }

    // =========================================================================
    // 24. Cross-context ID uniqueness
    // =========================================================================

    #[test]
    fn cat24_script_context_ids_unique() {
        let script = ScriptInfo::new("test", "/test.ts");
        let ids = action_ids(&get_script_context_actions(&script));
        let set: HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), set.len());
    }

    #[test]
    fn cat24_clipboard_context_ids_unique() {
        let entry = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "t".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let ids = action_ids(&get_clipboard_history_context_actions(&entry));
        let set: HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), set.len());
    }

    #[test]
    fn cat24_ai_command_bar_ids_unique() {
        let ids = action_ids(&get_ai_command_bar_actions());
        let set: HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), set.len());
    }

    #[test]
    fn cat24_path_context_ids_unique() {
        let info = PathInfo {
            path: "/test".into(),
            name: "test".into(),
            is_dir: false,
        };
        let ids = action_ids(&get_path_context_actions(&info));
        let set: HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), set.len());
    }

    #[test]
    fn cat24_notes_context_ids_unique() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let ids = action_ids(&get_notes_command_bar_actions(&info));
        let set: HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), set.len());
    }

    #[test]
    fn cat24_file_context_ids_unique() {
        let info = FileInfo {
            path: "/test/f.txt".into(),
            name: "f.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let ids = action_ids(&get_file_context_actions(&info));
        let set: HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), set.len());
    }

    // =========================================================================
    // 25. has_action=false for all built-ins
    // =========================================================================

    #[test]
    fn cat25_script_all_has_action_false() {
        let script = ScriptInfo::new("test", "/test.ts");
        for action in &get_script_context_actions(&script) {
            assert!(
                !action.has_action,
                "Action {} has_action should be false",
                action.id
            );
        }
    }

    #[test]
    fn cat25_clipboard_all_has_action_false() {
        let entry = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "t".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        for action in &get_clipboard_history_context_actions(&entry) {
            assert!(
                !action.has_action,
                "Action {} has_action should be false",
                action.id
            );
        }
    }

    #[test]
    fn cat25_ai_all_has_action_false() {
        for action in &get_ai_command_bar_actions() {
            assert!(
                !action.has_action,
                "Action {} has_action should be false",
                action.id
            );
        }
    }

    #[test]
    fn cat25_path_all_has_action_false() {
        let info = PathInfo {
            path: "/test".into(),
            name: "test".into(),
            is_dir: false,
        };
        for action in &get_path_context_actions(&info) {
            assert!(
                !action.has_action,
                "Action {} has_action should be false",
                action.id
            );
        }
    }

    #[test]
    fn cat25_file_all_has_action_false() {
        let info = FileInfo {
            path: "/test/f.txt".into(),
            name: "f.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        for action in &get_file_context_actions(&info) {
            assert!(
                !action.has_action,
                "Action {} has_action should be false",
                action.id
            );
        }
    }

    #[test]
    fn cat25_notes_all_has_action_false() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for action in &get_notes_command_bar_actions(&info) {
            assert!(
                !action.has_action,
                "Action {} has_action should be false",
                action.id
            );
        }
    }

    // =========================================================================
    // 26. Ordering determinism
    // =========================================================================


    // --- merged from tests_part_05.rs ---
    #[test]
    fn cat26_script_ordering_deterministic() {
        let script = ScriptInfo::new("test", "/test.ts");
        let ids1 = action_ids(&get_script_context_actions(&script));
        let ids2 = action_ids(&get_script_context_actions(&script));
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn cat26_clipboard_ordering_deterministic() {
        let entry = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "t".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let ids1 = action_ids(&get_clipboard_history_context_actions(&entry));
        let ids2 = action_ids(&get_clipboard_history_context_actions(&entry));
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn cat26_ai_ordering_deterministic() {
        let ids1 = action_ids(&get_ai_command_bar_actions());
        let ids2 = action_ids(&get_ai_command_bar_actions());
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn cat26_notes_ordering_deterministic() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let ids1 = action_ids(&get_notes_command_bar_actions(&info));
        let ids2 = action_ids(&get_notes_command_bar_actions(&info));
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn cat26_path_ordering_deterministic() {
        let info = PathInfo {
            path: "/test".into(),
            name: "test".into(),
            is_dir: true,
        };
        let ids1 = action_ids(&get_path_context_actions(&info));
        let ids2 = action_ids(&get_path_context_actions(&info));
        assert_eq!(ids1, ids2);
    }

    // =========================================================================
    // 27. Action builder chaining
    // =========================================================================

    #[test]
    fn cat27_with_icon_preserves_shortcut() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘X")
            .with_icon(IconName::Copy);
        assert_eq!(action.shortcut, Some("⌘X".to_string()));
        assert_eq!(action.icon, Some(IconName::Copy));
    }

    #[test]
    fn cat27_with_section_preserves_icon() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_icon(IconName::Star)
            .with_section("Test");
        assert_eq!(action.icon, Some(IconName::Star));
        assert_eq!(action.section, Some("Test".to_string()));
    }

    #[test]
    fn cat27_full_chain_preserves_all() {
        let action = Action::new(
            "test",
            "Test",
            Some("Desc".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘T")
        .with_icon(IconName::Plus)
        .with_section("Section");
        assert_eq!(action.id, "test");
        assert_eq!(action.title, "Test");
        assert_eq!(action.description, Some("Desc".to_string()));
        assert_eq!(action.shortcut, Some("⌘T".to_string()));
        assert_eq!(action.icon, Some(IconName::Plus));
        assert_eq!(action.section, Some("Section".to_string()));
    }

    // =========================================================================
    // 28. Note switcher description rendering
    // =========================================================================

    #[test]
    fn cat28_note_switcher_preview_with_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "T".into(),
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
        assert!(desc.contains("·"));
    }

    #[test]
    fn cat28_note_switcher_empty_preview_shows_char_count() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n2".into(),
            title: "T".into(),
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
    fn cat28_note_switcher_singular_char() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n3".into(),
            title: "T".into(),
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
    fn cat28_note_switcher_truncation_at_61() {
        let long_preview = "a".repeat(61);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n4".into(),
            title: "T".into(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: long_preview,
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.ends_with('…'));
    }

    #[test]
    fn cat28_note_switcher_no_truncation_at_60() {
        let exact_preview = "b".repeat(60);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n5".into(),
            title: "T".into(),
            char_count: 60,
            is_current: false,
            is_pinned: false,
            preview: exact_preview.clone(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(!desc.contains('…'));
        assert_eq!(desc.as_str(), exact_preview.as_str());
    }

    #[test]
    fn cat28_note_switcher_empty_preview_with_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n6".into(),
            title: "T".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "3h ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "3h ago");
    }

    // =========================================================================
    // 29. File context title includes quoted name
    // =========================================================================

    #[test]
    fn cat29_file_title_includes_filename() {
        let info = FileInfo {
            path: "/test/report.pdf".into(),
            name: "report.pdf".into(),
            file_type: crate::file_search::FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        assert!(actions[0].title.contains("report.pdf"));
    }

    #[test]
    fn cat29_file_dir_title_includes_dirname() {
        let info = FileInfo {
            path: "/test/MyDir".into(),
            name: "MyDir".into(),
            file_type: crate::file_search::FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&info);
        assert!(actions[0].title.contains("MyDir"));
    }

    #[test]
    fn cat29_file_title_has_quotes() {
        let info = FileInfo {
            path: "/test/file.txt".into(),
            name: "file.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        assert!(actions[0].title.contains('"'));
    }

    // =========================================================================
    // 30. Non-empty id and title for all contexts
    // =========================================================================

    #[test]
    fn cat30_script_nonempty_id_title() {
        let script = ScriptInfo::new("test", "/test.ts");
        for action in &get_script_context_actions(&script) {
            assert!(!action.id.is_empty(), "Action should have non-empty id");
            assert!(
                !action.title.is_empty(),
                "Action {} should have non-empty title",
                action.id
            );
        }
    }

    #[test]
    fn cat30_clipboard_nonempty_id_title() {
        let entry = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "t".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        for action in &get_clipboard_history_context_actions(&entry) {
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
        let info = PathInfo {
            path: "/test".into(),
            name: "test".into(),
            is_dir: false,
        };
        for action in &get_path_context_actions(&info) {
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
        let info = FileInfo {
            path: "/test/f.txt".into(),
            name: "f.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        for action in &get_file_context_actions(&info) {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

}
