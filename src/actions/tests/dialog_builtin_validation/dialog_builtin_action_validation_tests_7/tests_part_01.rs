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
            "run",
            "run script",
            Some("run the script now".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "run");
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
        assert!(ids.contains(&"clipboard_pin"));
        assert!(!ids.contains(&"clipboard_unpin"));
        assert!(!ids.contains(&"clipboard_ocr"), "Text should not have OCR");
    }

    #[test]
    fn clipboard_text_pinned_has_unpin() {
        let entry = make_text_entry(true, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"clipboard_unpin"));
        assert!(!ids.contains(&"clipboard_pin"));
    }

    #[test]
    fn clipboard_image_unpinned_has_ocr_and_pin() {
        let entry = make_image_entry(false);
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"clipboard_ocr"), "Image should have OCR");
        assert!(ids.contains(&"clipboard_pin"));
        assert!(!ids.contains(&"clipboard_unpin"));
    }

    #[test]
    fn clipboard_image_pinned_has_ocr_and_unpin() {
        let entry = make_image_entry(true);
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"clipboard_ocr"), "Image should have OCR");
        assert!(ids.contains(&"clipboard_unpin"));
        assert!(!ids.contains(&"clipboard_pin"));
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
        assert!(actions.iter().any(|a| a.id == "select_model_gpt-4"));
        assert!(actions.iter().any(|a| a.id == "select_model_claude-3"));
    }

