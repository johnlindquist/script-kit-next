    use crate::actions::builders::*;
    use crate::actions::dialog::ActionsDialog;
    use crate::actions::types::{Action, ActionCategory, ScriptInfo};
    use crate::actions::window::count_section_headers;
    use crate::clipboard_history::ContentType;
    use crate::designs::icon_variations::IconName;
    use crate::file_search::{FileInfo, FileType};
    use crate::prompts::PathInfo;
    use crate::scriptlets::Scriptlet;

    // =========================================================================
    // 1. fuzzy_match: empty needle and empty haystack behavior
    // =========================================================================

    #[test]
    fn fuzzy_match_empty_needle_matches_anything() {
        // Empty needle should match any haystack (no characters to find)
        assert!(ActionsDialog::fuzzy_match("hello world", ""));
    }

    #[test]
    fn fuzzy_match_empty_haystack_fails_nonempty_needle() {
        // Non-empty needle can't be found in empty haystack
        assert!(!ActionsDialog::fuzzy_match("", "a"));
    }

    #[test]
    fn fuzzy_match_both_empty_matches() {
        // Both empty: trivially matches
        assert!(ActionsDialog::fuzzy_match("", ""));
    }

    #[test]
    fn fuzzy_match_single_char_in_haystack() {
        assert!(ActionsDialog::fuzzy_match("abcdef", "a"));
        assert!(ActionsDialog::fuzzy_match("abcdef", "f"));
        assert!(!ActionsDialog::fuzzy_match("abcdef", "z"));
    }

    // =========================================================================
    // 2. fuzzy_match: subsequence order enforcement
    // =========================================================================

    #[test]
    fn fuzzy_match_correct_order_matches() {
        assert!(ActionsDialog::fuzzy_match("copy path", "cp"));
    }

    #[test]
    fn fuzzy_match_reversed_order_fails() {
        assert!(!ActionsDialog::fuzzy_match("copy path", "pc"));
    }

    #[test]
    fn fuzzy_match_duplicate_chars_in_needle() {
        // "aa" should match "banana" (a at index 1, a at index 3)
        assert!(ActionsDialog::fuzzy_match("banana", "aa"));
    }

    #[test]
    fn fuzzy_match_full_string_as_subsequence() {
        assert!(ActionsDialog::fuzzy_match("edit", "edit"));
    }

    // =========================================================================
    // 3. score_action: description-only match yields exactly 15
    // =========================================================================

    #[test]
    fn score_action_desc_only_match_is_15() {
        let action = Action::new(
            "test",
            "Xyz Title",
            Some("open in editor".to_string()),
            ActionCategory::ScriptContext,
        );
        // Search "editor" won't match title "xyz title" but will match description
        let score = ActionsDialog::score_action(&action, "editor");
        assert_eq!(score, 15);
    }

    #[test]
    fn score_action_no_desc_no_match_is_0() {
        let action = Action::new("test", "Xyz Title", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "editor");
        assert_eq!(score, 0);
    }

    #[test]
    fn score_action_desc_match_plus_title_prefix() {
        let action = Action::new(
            "test",
            "Open File",
            Some("Open in editor".to_string()),
            ActionCategory::ScriptContext,
        );
        // "open" matches title prefix (100) + description contains (15)
        let score = ActionsDialog::score_action(&action, "open");
        assert_eq!(score, 115);
    }

    #[test]
    fn score_action_desc_match_plus_title_contains() {
        let action = Action::new(
            "test",
            "My Open File",
            Some("Open in editor".to_string()),
            ActionCategory::ScriptContext,
        );
        // "open" matches title contains (50) + description contains (15)
        let score = ActionsDialog::score_action(&action, "open");
        assert_eq!(score, 65);
    }

    // =========================================================================
    // 4. score_action: shortcut-only match yields exactly 10
    // =========================================================================

    #[test]
    fn score_action_shortcut_only_match_is_10() {
        let action = Action::new("test", "Xyz Title", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘E");
        // Search "⌘e" (lowercase) matches shortcut_lower "⌘e"
        let score = ActionsDialog::score_action(&action, "⌘e");
        assert_eq!(score, 10);
    }

    #[test]
    fn score_action_shortcut_no_match_is_0() {
        let action = Action::new("test", "Abc Title", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘E");
        // "q" doesn't match title "abc title", description None, or shortcut "⌘e"
        let score = ActionsDialog::score_action(&action, "q");
        assert_eq!(score, 0);
    }

    #[test]
    fn score_action_shortcut_plus_title_prefix() {
        let action = Action::new("test", "Edit Script", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘E");
        // "e" matches title prefix "edit script" (100) + shortcut_lower "⌘e" contains "e" (10)
        let score = ActionsDialog::score_action(&action, "e");
        assert_eq!(score, 110);
    }

    #[test]
    fn score_action_shortcut_plus_desc() {
        let action = Action::new(
            "test",
            "Xyz Title",
            Some("open in editor".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘E");
        // "⌘e" only matches shortcut (10), not title, not desc
        let score = ActionsDialog::score_action(&action, "⌘e");
        assert_eq!(score, 10);
    }

    // =========================================================================
    // 5. score_action: combined prefix + description + shortcut max score
    // =========================================================================

    #[test]
    fn score_action_max_combined_prefix_desc_shortcut() {
        let action = Action::new(
            "test",
            "edit script",
            Some("edit in editor".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("edit");
        // "edit" matches: title prefix (100) + desc contains (15) + shortcut contains (10) = 125
        let score = ActionsDialog::score_action(&action, "edit");
        assert_eq!(score, 125);
    }

    #[test]
    fn score_action_contains_desc_shortcut() {
        let action = Action::new(
            "test",
            "My Edit Script",
            Some("edit in editor".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("edit");
        // "edit" matches: title contains (50) + desc contains (15) + shortcut contains (10) = 75
        let score = ActionsDialog::score_action(&action, "edit");
        assert_eq!(score, 75);
    }

    #[test]
    fn score_action_no_title_match_desc_and_shortcut() {
        let action = Action::new(
            "test",
            "my xdyt script",
            Some("edit in editor".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("edit");
        // "edit": 'e' not in "my xdyt script" so no fuzzy → 0
        // desc "edit in editor" contains "edit" → +15
        // shortcut "edit" contains "edit" → +10
        // Total: 25
        let score = ActionsDialog::score_action(&action, "edit");
        assert_eq!(score, 25);
    }

    #[test]
    fn score_action_no_title_match_desc_shortcut_only() {
        let action = Action::new(
            "test",
            "Xyz Abc",
            Some("Open file in editor".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘E");
        // "⌘e" matches: no title match (0) + no desc match (0) + shortcut (10) = 10
        let score = ActionsDialog::score_action(&action, "⌘e");
        assert_eq!(score, 10);
    }

    // =========================================================================
    // 6. builders format_shortcut_hint: simpler .replace chain vs dialog
    // =========================================================================

    #[test]
    fn builders_format_shortcut_hint_cmd_c() {
        // builders::format_shortcut_hint uses simple .replace() chain
        // "cmd+c" → replace cmd→⌘, replace +→"", uppercase → "⌘C"
        let hint = format_shortcut_hint_for_test("cmd+c");
        assert_eq!(hint, "⌘C");
    }

    #[test]
    fn builders_format_shortcut_hint_ctrl_shift_x() {
        let hint = format_shortcut_hint_for_test("ctrl+shift+x");
        assert_eq!(hint, "⌃⇧X");
    }

    #[test]
    fn builders_format_shortcut_hint_alt_k() {
        let hint = format_shortcut_hint_for_test("alt+k");
        assert_eq!(hint, "⌥K");
    }

    #[test]
    fn builders_format_shortcut_hint_single_letter() {
        // Just a single letter "a" → "A"
        let hint = format_shortcut_hint_for_test("a");
        assert_eq!(hint, "A");
    }

    // Helper to call the builders-level format_shortcut_hint (private fn, test via scriptlet)
    fn format_shortcut_hint_for_test(shortcut: &str) -> String {
        // We can test this indirectly by creating a scriptlet action with a shortcut
        // and checking the resulting action's shortcut field
        let mut scriptlet = Scriptlet::new("Test".to_string(), "bash".to_string(), String::new());
        scriptlet.actions.push(crate::scriptlets::ScriptletAction {
            name: "Test Action".to_string(),
            command: "test-action".to_string(),
            description: None,
            shortcut: Some(shortcut.to_string()),
            tool: "bash".to_string(),
            code: String::new(),
            inputs: vec![],
        });
        let actions = get_scriptlet_defined_actions(&scriptlet);
        actions[0].shortcut.clone().unwrap_or_default()
    }

    // =========================================================================
    // 7. builders format_shortcut_hint: unknown keys pass through
    // =========================================================================

    #[test]
    fn builders_format_unknown_key_uppercased() {
        let hint = format_shortcut_hint_for_test("cmd+f1");
        // "cmd+f1" → "⌘" + remove + → "⌘F1"
        assert_eq!(hint, "⌘F1");
    }

    #[test]
    fn builders_format_numbers_preserved() {
        let hint = format_shortcut_hint_for_test("cmd+1");
        assert_eq!(hint, "⌘1");
    }

    #[test]
    fn builders_format_empty_shortcut() {
        let hint = format_shortcut_hint_for_test("");
        assert_eq!(hint, "");
    }

    #[test]
    fn builders_format_all_four_modifiers() {
        let hint = format_shortcut_hint_for_test("cmd+ctrl+alt+shift+k");
        assert_eq!(hint, "⌘⌃⌥⇧K");
    }

    // =========================================================================
    // 8. parse_shortcut_keycaps: all modifier symbols individually
    // =========================================================================

    #[test]
    fn parse_keycaps_command_symbol() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘");
        assert_eq!(caps, vec!["⌘"]);
    }

    #[test]
    fn parse_keycaps_all_arrows() {
        let caps = ActionsDialog::parse_shortcut_keycaps("↑↓←→");
        assert_eq!(caps, vec!["↑", "↓", "←", "→"]);
    }

    #[test]
    fn parse_keycaps_escape_tab_backspace_space() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⎋⇥⌫␣");
        assert_eq!(caps, vec!["⎋", "⇥", "⌫", "␣"]);
    }

    #[test]
    fn parse_keycaps_mixed_modifiers_and_letter() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘⇧C");
        assert_eq!(caps, vec!["⌘", "⇧", "C"]);
    }

    // =========================================================================
    // 9. parse_shortcut_keycaps: empty string produces empty vec
    // =========================================================================

    #[test]
    fn parse_keycaps_empty_string() {
        let caps = ActionsDialog::parse_shortcut_keycaps("");
        assert!(caps.is_empty());
    }

    #[test]
    fn parse_keycaps_lowercase_uppercased() {
        let caps = ActionsDialog::parse_shortcut_keycaps("a");
        assert_eq!(caps, vec!["A"]);
    }

    #[test]
    fn parse_keycaps_digit_preserved() {
        let caps = ActionsDialog::parse_shortcut_keycaps("1");
        assert_eq!(caps, vec!["1"]);
    }

    #[test]
    fn parse_keycaps_return_symbol() {
        let caps = ActionsDialog::parse_shortcut_keycaps("↵");
        assert_eq!(caps, vec!["↵"]);
    }

    // =========================================================================
    // 10. Clipboard: share action details (shortcut, title, position)
    // =========================================================================

    #[test]
    fn clipboard_share_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let share = actions.iter().find(|a| a.id == "clipboard_share").unwrap();
        assert_eq!(share.shortcut.as_deref(), Some("⇧⌘E"));
    }

    #[test]
    fn clipboard_share_title() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let share = actions.iter().find(|a| a.id == "clipboard_share").unwrap();
        assert_eq!(share.title, "Share...");
    }

    #[test]
    fn clipboard_share_position_after_paste_keep_open() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let share_idx = actions
            .iter()
            .position(|a| a.id == "clipboard_share")
            .unwrap();
        let paste_keep_idx = actions
            .iter()
            .position(|a| a.id == "clipboard_paste_keep_open")
            .unwrap();
        assert!(share_idx > paste_keep_idx);
    }

    #[test]
    fn clipboard_share_desc_mentions_share() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let share = actions.iter().find(|a| a.id == "clipboard_share").unwrap();
        assert!(share
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("share"));
    }

    // =========================================================================
    // 11. Clipboard: attach_to_ai action details
    // =========================================================================

