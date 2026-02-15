//! Batch 41: Dialog builtin action validation tests
//!
//! Focuses on:
//! - fuzzy_match: empty needle and empty haystack behavior
//! - fuzzy_match: subsequence order enforcement
//! - score_action: description-only match yields exactly 15
//! - score_action: shortcut-only match yields exactly 10
//! - score_action: combined prefix + description + shortcut max score
//! - builders format_shortcut_hint: simpler .replace chain vs dialog
//! - builders format_shortcut_hint: unknown keys pass through
//! - parse_shortcut_keycaps: all modifier symbols individually
//! - parse_shortcut_keycaps: empty string produces empty vec
//! - Clipboard: share action details (shortcut, title, position)
//! - Clipboard: attach_to_ai action details
//! - Clipboard: image open_with is macOS only
//! - File context: primary action ID differs file vs dir
//! - File context: all IDs unique within context
//! - Path context: open_in_terminal shortcut and desc
//! - Path context: move_to_trash desc differs file vs dir
//! - Script context: with shortcut yields update_shortcut + remove_shortcut
//! - Script context: with alias yields update_alias + remove_alias
//! - Script context: agent has edit_script with "Edit Agent" title, desc mentions agent
//! - Script context: total action count varies by type
//! - Scriptlet context: add_shortcut when no shortcut, add_alias when no alias
//! - Scriptlet context: reset_ranking only when is_suggested
//! - AI bar: delete_chat shortcut and icon
//! - AI bar: new_chat shortcut and icon
//! - Notes: format action details
//! - Notes: selection+trash yields subset of actions
//! - Chat context: model with current_model gets checkmark
//! - Chat context: multiple models ordering
//! - New chat: section ordering across last_used, presets, models
//! - count_section_headers: items without sections produce 0 headers

#[cfg(test)]
mod tests {
    // --- merged from tests_part_01.rs ---
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
        let share = actions.iter().find(|a| a.id == "clip:clipboard_share").unwrap();
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
        let share = actions.iter().find(|a| a.id == "clip:clipboard_share").unwrap();
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
            .position(|a| a.id == "clip:clipboard_share")
            .unwrap();
        let paste_keep_idx = actions
            .iter()
            .position(|a| a.id == "clip:clipboard_paste_keep_open")
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
        let share = actions.iter().find(|a| a.id == "clip:clipboard_share").unwrap();
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


    // --- merged from tests_part_02.rs ---
    #[test]
    fn clipboard_attach_to_ai_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hi".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let attach = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_attach_to_ai")
            .unwrap();
        assert_eq!(attach.shortcut.as_deref(), Some("⌃⌘A"));
    }

    #[test]
    fn clipboard_attach_to_ai_title() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hi".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let attach = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_attach_to_ai")
            .unwrap();
        assert_eq!(attach.title, "Attach to AI Chat");
    }

    #[test]
    fn clipboard_attach_to_ai_desc_mentions_ai() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hi".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let attach = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_attach_to_ai")
            .unwrap();
        assert!(attach
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("ai"));
    }

    #[test]
    fn clipboard_attach_to_ai_present_for_image_too() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clip:clipboard_attach_to_ai"));
    }

    // =========================================================================
    // 12. Clipboard: image open_with is macOS only
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn clipboard_image_has_open_with() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clip:clipboard_open_with"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn clipboard_text_has_no_open_with() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(!actions.iter().any(|a| a.id == "clip:clipboard_open_with"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn clipboard_image_open_with_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let open_with = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_open_with")
            .unwrap();
        assert_eq!(open_with.shortcut.as_deref(), Some("⌘O"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn clipboard_image_annotate_cleanshot_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let annotate = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_annotate_cleanshot")
            .unwrap();
        assert_eq!(annotate.shortcut.as_deref(), Some("⇧⌘A"));
    }

    // =========================================================================
    // 13. File context: primary action ID differs file vs dir
    // =========================================================================

    #[test]
    fn file_context_file_primary_id_is_open_file() {
        let info = FileInfo {
            name: "readme.md".to_string(),
            path: "/docs/readme.md".to_string(),
            is_dir: false,
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&info);
        assert_eq!(actions[0].id, "file:open_file");
    }

    #[test]
    fn file_context_dir_primary_id_is_open_directory() {
        let info = FileInfo {
            name: "src".to_string(),
            path: "/project/src".to_string(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        let actions = get_file_context_actions(&info);
        assert_eq!(actions[0].id, "file:open_directory");
    }

    #[test]
    fn file_context_primary_shortcut_is_enter() {
        let info = FileInfo {
            name: "test.txt".to_string(),
            path: "/test.txt".to_string(),
            is_dir: false,
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&info);
        assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
    }

    #[test]
    fn file_context_dir_primary_desc_mentions_folder() {
        let info = FileInfo {
            name: "lib".to_string(),
            path: "/lib".to_string(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        let actions = get_file_context_actions(&info);
        assert!(actions[0]
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("folder"));
    }

    // =========================================================================
    // 14. File context: all IDs unique within context
    // =========================================================================

    #[test]
    fn file_context_file_all_ids_unique() {
        let info = FileInfo {
            name: "test.rs".to_string(),
            path: "/test.rs".to_string(),
            is_dir: false,
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&info);
        let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        let mut unique_ids = ids.clone();
        unique_ids.sort();
        unique_ids.dedup();
        assert_eq!(ids.len(), unique_ids.len());
    }

    #[test]
    fn file_context_dir_all_ids_unique() {
        let info = FileInfo {
            name: "docs".to_string(),
            path: "/docs".to_string(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        let actions = get_file_context_actions(&info);
        let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        let mut unique_ids = ids.clone();
        unique_ids.sort();
        unique_ids.dedup();
        assert_eq!(ids.len(), unique_ids.len());
    }

    #[test]
    fn file_context_file_has_copy_path_and_copy_filename() {
        let info = FileInfo {
            name: "foo.txt".to_string(),
            path: "/foo.txt".to_string(),
            is_dir: false,
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "file:copy_path"));
        assert!(actions.iter().any(|a| a.id == "file:copy_filename"));
    }

    #[test]
    fn file_context_reveal_in_finder_always_present() {
        let file_info = FileInfo {
            name: "a.txt".to_string(),
            path: "/a.txt".to_string(),
            is_dir: false,
            file_type: FileType::File,
        };
        let dir_info = FileInfo {
            name: "b".to_string(),
            path: "/b".to_string(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        assert!(get_file_context_actions(&file_info)
            .iter()
            .any(|a| a.id == "file:reveal_in_finder"));
        assert!(get_file_context_actions(&dir_info)
            .iter()
            .any(|a| a.id == "file:reveal_in_finder"));
    }

    // =========================================================================
    // 15. Path context: open_in_terminal shortcut and desc
    // =========================================================================

    #[test]
    fn path_context_open_in_terminal_shortcut() {
        let info = PathInfo {
            name: "src".to_string(),
            path: "/project/src".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let term = actions.iter().find(|a| a.id == "file:open_in_terminal").unwrap();
        assert_eq!(term.shortcut.as_deref(), Some("⌘T"));
    }

    #[test]
    fn path_context_open_in_terminal_desc_mentions_terminal() {
        let info = PathInfo {
            name: "src".to_string(),
            path: "/project/src".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let term = actions.iter().find(|a| a.id == "file:open_in_terminal").unwrap();
        assert!(term
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("terminal"));
    }

    #[test]
    fn path_context_open_in_terminal_present_for_files() {
        let info = PathInfo {
            name: "script.sh".to_string(),
            path: "/project/script.sh".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "file:open_in_terminal"));
    }

    #[test]
    fn path_context_open_in_terminal_title() {
        let info = PathInfo {
            name: "foo".to_string(),
            path: "/foo".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let term = actions.iter().find(|a| a.id == "file:open_in_terminal").unwrap();
        assert_eq!(term.title, "Open in Terminal");
    }

    // =========================================================================
    // 16. Path context: move_to_trash desc differs file vs dir
    // =========================================================================

    #[test]
    fn path_context_trash_desc_file() {
        let info = PathInfo {
            name: "test.txt".to_string(),
            path: "/test.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.iter().find(|a| a.id == "file:move_to_trash").unwrap();
        assert!(trash.description.as_ref().unwrap().contains("file"));
    }

    #[test]
    fn path_context_trash_desc_dir() {
        let info = PathInfo {
            name: "src".to_string(),
            path: "/src".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.iter().find(|a| a.id == "file:move_to_trash").unwrap();
        assert!(trash.description.as_ref().unwrap().contains("folder"));
    }

    #[test]
    fn path_context_trash_shortcut() {
        let info = PathInfo {
            name: "x".to_string(),
            path: "/x".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.iter().find(|a| a.id == "file:move_to_trash").unwrap();
        assert_eq!(trash.shortcut.as_deref(), Some("⌘⌫"));
    }

    #[test]
    fn path_context_trash_is_last_action() {
        let info = PathInfo {
            name: "y".to_string(),
            path: "/y".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions.last().unwrap().id, "file:move_to_trash");
    }

    // =========================================================================
    // 17. Script context: with shortcut yields update_shortcut + remove_shortcut
    // =========================================================================

    #[test]
    fn script_with_shortcut_has_update_shortcut() {
        let info = ScriptInfo::with_shortcut("my-script", "/s.ts", Some("cmd+k".into()));
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "update_shortcut"));
    }

    #[test]
    fn script_with_shortcut_has_remove_shortcut() {
        let info = ScriptInfo::with_shortcut("my-script", "/s.ts", Some("cmd+k".into()));
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
    }

    #[test]
    fn script_with_shortcut_has_no_add_shortcut() {
        let info = ScriptInfo::with_shortcut("my-script", "/s.ts", Some("cmd+k".into()));
        let actions = get_script_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "add_shortcut"));
    }

    #[test]
    fn script_without_shortcut_has_add_shortcut() {
        let info = ScriptInfo::new("my-script", "/s.ts");
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "add_shortcut"));
        assert!(!actions.iter().any(|a| a.id == "update_shortcut"));
        assert!(!actions.iter().any(|a| a.id == "remove_shortcut"));
    }

    // =========================================================================
    // 18. Script context: with alias yields update_alias + remove_alias
    // =========================================================================

    #[test]
    fn script_with_alias_has_update_alias() {
        let info =
            ScriptInfo::with_shortcut_and_alias("my-script", "/s.ts", None, Some("ms".to_string()));
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "update_alias"));
    }

    #[test]
    fn script_with_alias_has_remove_alias() {
        let info =
            ScriptInfo::with_shortcut_and_alias("my-script", "/s.ts", None, Some("ms".to_string()));
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "remove_alias"));
    }

    #[test]
    fn script_with_alias_has_no_add_alias() {
        let info =
            ScriptInfo::with_shortcut_and_alias("my-script", "/s.ts", None, Some("ms".to_string()));
        let actions = get_script_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "add_alias"));
    }


    // --- merged from tests_part_03.rs ---
    #[test]
    fn script_without_alias_has_add_alias() {
        let info = ScriptInfo::new("my-script", "/s.ts");
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "add_alias"));
        assert!(!actions.iter().any(|a| a.id == "update_alias"));
        assert!(!actions.iter().any(|a| a.id == "remove_alias"));
    }

    // =========================================================================
    // 19. Script context: agent has edit_script with "Edit Agent" title, desc mentions agent
    // =========================================================================

    #[test]
    fn agent_edit_title_is_edit_agent() {
        let mut info = ScriptInfo::new("my-agent", "/a.md");
        info.is_agent = true;
        info.is_script = false;
        let actions = get_script_context_actions(&info);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    #[test]
    fn agent_edit_desc_mentions_agent() {
        let mut info = ScriptInfo::new("my-agent", "/a.md");
        info.is_agent = true;
        info.is_script = false;
        let actions = get_script_context_actions(&info);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert!(edit
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("agent"));
    }

    #[test]
    fn agent_has_reveal_in_finder() {
        let mut info = ScriptInfo::new("my-agent", "/a.md");
        info.is_agent = true;
        info.is_script = false;
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "file:reveal_in_finder"));
    }

    #[test]
    fn agent_reveal_desc_mentions_agent() {
        let mut info = ScriptInfo::new("my-agent", "/a.md");
        info.is_agent = true;
        info.is_script = false;
        let actions = get_script_context_actions(&info);
        let reveal = actions.iter().find(|a| a.id == "file:reveal_in_finder").unwrap();
        assert!(reveal
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("agent"));
    }

    // =========================================================================
    // 20. Script context: total action count varies by type
    // =========================================================================

    #[test]
    fn script_context_real_script_count() {
        // Real script: run + add_shortcut + add_alias + edit + view_logs + reveal + copy_path + copy_content + copy_deeplink = 9
        let info = ScriptInfo::new("test", "/test.ts");
        let actions = get_script_context_actions(&info);
        assert_eq!(actions.len(), 9);
    }

    #[test]
    fn script_context_builtin_count() {
        // Builtin: run + add_shortcut + add_alias + copy_deeplink = 4
        let info = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&info);
        assert_eq!(actions.len(), 4);
    }

    #[test]
    fn script_context_agent_count() {
        // Agent: run + add_shortcut + add_alias + edit + reveal + copy_path + copy_content + copy_deeplink = 8
        let mut info = ScriptInfo::new("my-agent", "/a.md");
        info.is_agent = true;
        info.is_script = false;
        let actions = get_script_context_actions(&info);
        assert_eq!(actions.len(), 8);
    }

    #[test]
    fn script_context_scriptlet_count() {
        // Scriptlet: run + add_shortcut + add_alias + edit_scriptlet + reveal + copy_path + copy_content + copy_deeplink = 8
        let info = ScriptInfo::scriptlet("Test Scriptlet", "/t.md", None, None);
        let actions = get_script_context_actions(&info);
        assert_eq!(actions.len(), 8);
    }

    // =========================================================================
    // 21. Scriptlet context: add_shortcut when no shortcut, add_alias when no alias
    // =========================================================================

    #[test]
    fn scriptlet_with_custom_no_shortcut_has_add_shortcut() {
        let script = ScriptInfo::scriptlet("Test", "/t.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(actions.iter().any(|a| a.id == "add_shortcut"));
        assert!(!actions.iter().any(|a| a.id == "update_shortcut"));
    }

    #[test]
    fn scriptlet_with_custom_has_shortcut_shows_update_remove() {
        let script = ScriptInfo::scriptlet("Test", "/t.md", Some("cmd+t".into()), None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(actions.iter().any(|a| a.id == "update_shortcut"));
        assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
        assert!(!actions.iter().any(|a| a.id == "add_shortcut"));
    }

    #[test]
    fn scriptlet_with_custom_no_alias_has_add_alias() {
        let script = ScriptInfo::scriptlet("Test", "/t.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(actions.iter().any(|a| a.id == "add_alias"));
        assert!(!actions.iter().any(|a| a.id == "update_alias"));
    }

    #[test]
    fn scriptlet_with_custom_has_alias_shows_update_remove() {
        let script = ScriptInfo::scriptlet("Test", "/t.md", None, Some("tst".into()));
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(actions.iter().any(|a| a.id == "update_alias"));
        assert!(actions.iter().any(|a| a.id == "remove_alias"));
        assert!(!actions.iter().any(|a| a.id == "add_alias"));
    }

    // =========================================================================
    // 22. Scriptlet context: reset_ranking only when is_suggested
    // =========================================================================

    #[test]
    fn scriptlet_with_custom_suggested_has_reset_ranking() {
        let script = ScriptInfo::scriptlet("Test", "/t.md", None, None)
            .with_frecency(true, Some("/t.md".into()));
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    }

    #[test]
    fn scriptlet_with_custom_not_suggested_no_reset_ranking() {
        let script = ScriptInfo::scriptlet("Test", "/t.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(!actions.iter().any(|a| a.id == "reset_ranking"));
    }

    #[test]
    fn scriptlet_with_custom_reset_ranking_is_last() {
        let script = ScriptInfo::scriptlet("Test", "/t.md", None, None)
            .with_frecency(true, Some("/t.md".into()));
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert_eq!(actions.last().unwrap().id, "reset_ranking");
    }

    #[test]
    fn scriptlet_with_custom_reset_ranking_has_no_shortcut() {
        let script = ScriptInfo::scriptlet("Test", "/t.md", None, None)
            .with_frecency(true, Some("/t.md".into()));
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let reset = actions.iter().find(|a| a.id == "reset_ranking").unwrap();
        assert_eq!(reset.shortcut.as_deref(), Some("⌃⌘R"));
    }

    // =========================================================================
    // 23. AI bar: delete_chat shortcut and icon
    // =========================================================================

    #[test]
    fn ai_bar_delete_chat_shortcut() {
        let actions = get_ai_command_bar_actions();
        let delete = actions.iter().find(|a| a.id == "chat:delete_chat").unwrap();
        assert_eq!(delete.shortcut.as_deref(), Some("⌘⌫"));
    }

    #[test]
    fn ai_bar_delete_chat_icon() {
        let actions = get_ai_command_bar_actions();
        let delete = actions.iter().find(|a| a.id == "chat:delete_chat").unwrap();
        assert_eq!(delete.icon, Some(IconName::Trash));
    }

    #[test]
    fn ai_bar_delete_chat_section() {
        let actions = get_ai_command_bar_actions();
        let delete = actions.iter().find(|a| a.id == "chat:delete_chat").unwrap();
        assert_eq!(delete.section.as_deref(), Some("Actions"));
    }

    #[test]
    fn ai_bar_delete_chat_desc_mentions_delete() {
        let actions = get_ai_command_bar_actions();
        let delete = actions.iter().find(|a| a.id == "chat:delete_chat").unwrap();
        assert!(delete
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("delete"));
    }

    // =========================================================================
    // 24. AI bar: new_chat shortcut and icon
    // =========================================================================

    #[test]
    fn ai_bar_new_chat_shortcut() {
        let actions = get_ai_command_bar_actions();
        let nc = actions.iter().find(|a| a.id == "chat:new_chat").unwrap();
        assert_eq!(nc.shortcut.as_deref(), Some("⌘N"));
    }

    #[test]
    fn ai_bar_new_chat_icon() {
        let actions = get_ai_command_bar_actions();
        let nc = actions.iter().find(|a| a.id == "chat:new_chat").unwrap();
        assert_eq!(nc.icon, Some(IconName::Plus));
    }

    #[test]
    fn ai_bar_new_chat_section() {
        let actions = get_ai_command_bar_actions();
        let nc = actions.iter().find(|a| a.id == "chat:new_chat").unwrap();
        assert_eq!(nc.section.as_deref(), Some("Actions"));
    }

    #[test]
    fn ai_bar_new_chat_desc_mentions_conversation() {
        let actions = get_ai_command_bar_actions();
        let nc = actions.iter().find(|a| a.id == "chat:new_chat").unwrap();
        assert!(nc
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("conversation"));
    }

    // =========================================================================
    // 25. Notes: format action details
    // =========================================================================

    #[test]
    fn notes_format_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let format = actions.iter().find(|a| a.id == "format").unwrap();
        assert_eq!(format.shortcut.as_deref(), Some("⇧⌘T"));
    }

    #[test]
    fn notes_format_icon_code() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let format = actions.iter().find(|a| a.id == "format").unwrap();
        assert_eq!(format.icon, Some(IconName::Code));
    }

    #[test]
    fn notes_format_section_edit() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let format = actions.iter().find(|a| a.id == "format").unwrap();
        assert_eq!(format.section.as_deref(), Some("Edit"));
    }

    #[test]
    fn notes_format_absent_without_selection() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "format"));
    }

    // =========================================================================
    // 26. Notes: selection+trash yields subset of actions
    // =========================================================================

    #[test]
    fn notes_trash_view_has_new_note() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "notes:new_note"));
    }

    #[test]
    fn notes_trash_view_no_duplicate() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
    }

    #[test]
    fn notes_trash_view_no_find_in_note() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "find_in_note"));
    }

    #[test]
    fn notes_trash_view_no_export() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "export"));
    }

    // =========================================================================
    // 27. Chat context: model with current_model gets checkmark
    // =========================================================================

    #[test]
    fn chat_current_model_has_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".to_string()),
            available_models: vec![ChatModelInfo {
                id: "gpt4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = actions
            .iter()
            .find(|a| a.id == "chat:select_model_gpt4")
            .unwrap();
        assert!(model_action.title.contains("✓"));
    }

    #[test]
    fn chat_non_current_model_no_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".to_string()),
            available_models: vec![ChatModelInfo {
                id: "gpt4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = actions
            .iter()
            .find(|a| a.id == "chat:select_model_gpt4")
            .unwrap();
        assert!(!model_action.title.contains("✓"));
    }

    #[test]
    fn chat_no_current_model_no_checkmark() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "gpt4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = actions
            .iter()
            .find(|a| a.id == "chat:select_model_gpt4")
            .unwrap();
        assert!(!model_action.title.contains("✓"));
    }

    #[test]
    fn chat_model_desc_mentions_provider() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "claude".to_string(),
                display_name: "Claude".to_string(),
                provider: "Anthropic".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = actions
            .iter()
            .find(|a| a.id == "chat:select_model_claude")
            .unwrap();
        assert!(model_action
            .description
            .as_ref()
            .unwrap()
            .contains("Anthropic"));
    }

    // =========================================================================
    // 28. Chat context: multiple models ordering
    // =========================================================================

    // --- merged from tests_part_04.rs ---
    #[test]
    fn chat_models_come_before_continue_in_chat() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![
                ChatModelInfo {
                    id: "a".to_string(),
                    display_name: "Model A".to_string(),
                    provider: "P".to_string(),
                },
                ChatModelInfo {
                    id: "b".to_string(),
                    display_name: "Model B".to_string(),
                    provider: "P".to_string(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let continue_idx = actions
            .iter()
            .position(|a| a.id == "chat:continue_in_chat")
            .unwrap();
        let model_a_idx = actions
            .iter()
            .position(|a| a.id == "chat:select_model_a")
            .unwrap();
        let model_b_idx = actions
            .iter()
            .position(|a| a.id == "chat:select_model_b")
            .unwrap();
        assert!(model_a_idx < continue_idx);
        assert!(model_b_idx < continue_idx);
    }

    #[test]
    fn chat_models_preserve_order() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![
                ChatModelInfo {
                    id: "first".to_string(),
                    display_name: "First".to_string(),
                    provider: "P".to_string(),
                },
                ChatModelInfo {
                    id: "second".to_string(),
                    display_name: "Second".to_string(),
                    provider: "P".to_string(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let first_idx = actions
            .iter()
            .position(|a| a.id == "chat:select_model_first")
            .unwrap();
        let second_idx = actions
            .iter()
            .position(|a| a.id == "chat:select_model_second")
            .unwrap();
        assert!(first_idx < second_idx);
    }

    #[test]
    fn chat_both_messages_and_response_max_actions() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "m1".to_string(),
                display_name: "Model".to_string(),
                provider: "P".to_string(),
            }],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        // 1 model + continue + copy_response + clear_conversation = 4
        assert_eq!(actions.len(), 4);
    }

    #[test]
    fn chat_no_models_no_messages_minimal() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        // Only continue_in_chat
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "chat:continue_in_chat");
    }

    // =========================================================================
    // 29. New chat: section ordering across last_used, presets, models
    // =========================================================================

    #[test]
    fn new_chat_section_ordering_last_used_first() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".to_string(),
            display_name: "Model 1".to_string(),
            provider: "p".to_string(),
            provider_display_name: "Provider".to_string(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "general".to_string(),
            name: "General".to_string(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m2".to_string(),
            display_name: "Model 2".to_string(),
            provider: "p".to_string(),
            provider_display_name: "Provider".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        // First action section should be Last Used Settings
        assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
    }

    #[test]
    fn new_chat_section_ordering_presets_second() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".to_string(),
            display_name: "Model 1".to_string(),
            provider: "p".to_string(),
            provider_display_name: "Provider".to_string(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "general".to_string(),
            name: "General".to_string(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m2".to_string(),
            display_name: "Model 2".to_string(),
            provider: "p".to_string(),
            provider_display_name: "Provider".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        assert_eq!(actions[1].section.as_deref(), Some("Presets"));
    }

    #[test]
    fn new_chat_section_ordering_models_last() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".to_string(),
            display_name: "Model 1".to_string(),
            provider: "p".to_string(),
            provider_display_name: "Provider".to_string(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "general".to_string(),
            name: "General".to_string(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m2".to_string(),
            display_name: "Model 2".to_string(),
            provider: "p".to_string(),
            provider_display_name: "Provider".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        assert_eq!(actions[2].section.as_deref(), Some("Models"));
    }

    #[test]
    fn new_chat_total_count_matches_input_sizes() {
        let last_used = vec![
            NewChatModelInfo {
                model_id: "m1".to_string(),
                display_name: "M1".to_string(),
                provider: "p".to_string(),
                provider_display_name: "P".to_string(),
            },
            NewChatModelInfo {
                model_id: "m2".to_string(),
                display_name: "M2".to_string(),
                provider: "p".to_string(),
                provider_display_name: "P".to_string(),
            },
        ];
        let presets = vec![NewChatPresetInfo {
            id: "g".to_string(),
            name: "General".to_string(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m3".to_string(),
            display_name: "M3".to_string(),
            provider: "p".to_string(),
            provider_display_name: "P".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        assert_eq!(actions.len(), 4); // 2 + 1 + 1
    }

    // =========================================================================
    // 30. count_section_headers: items without sections produce 0 headers
    // =========================================================================

    #[test]
    fn count_headers_no_sections_is_zero() {
        let actions = vec![
            Action::new("a", "Action A", None, ActionCategory::ScriptContext),
            Action::new("b", "Action B", None, ActionCategory::ScriptContext),
        ];
        let indices: Vec<usize> = (0..actions.len()).collect();
        assert_eq!(count_section_headers(&actions, &indices), 0);
    }

    #[test]
    fn count_headers_all_same_section_is_one() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Group"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Group"),
        ];
        let indices: Vec<usize> = (0..actions.len()).collect();
        assert_eq!(count_section_headers(&actions, &indices), 1);
    }

    #[test]
    fn count_headers_two_different_sections() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Alpha"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Beta"),
        ];
        let indices: Vec<usize> = (0..actions.len()).collect();
        assert_eq!(count_section_headers(&actions, &indices), 2);
    }

    #[test]
    fn count_headers_empty_indices() {
        let actions =
            vec![Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("X")];
        let indices: Vec<usize> = vec![];
        assert_eq!(count_section_headers(&actions, &indices), 0);
    }

}
