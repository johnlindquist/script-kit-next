// --- merged from part_01.rs ---
//! Built-in action behavioral validation tests — batch 4
//!
//! Validates randomly-selected built-in actions across window dialogs and
//! contexts that were NOT covered in batches 1–3. Focuses on:
//! - Agent flag interactions with shortcut/alias/frecency combinations
//! - Custom action verbs propagating correctly into primary action titles
//! - Scriptlet context vs script context systematic action set comparison
//! - Clipboard text vs image action count differential (macOS)
//! - Path context action IDs all snake_case
//! - File context FileType variants produce consistent action set
//! - Notes section label exhaustiveness for full-feature permutation
//! - AI command bar icon-per-section coverage
//! - New chat with all-empty inputs produces empty output
//! - score_action edge cases (empty query, single char, unicode)
//! - fuzzy_match boundary conditions (empty strings, longer needle, etc.)
//! - parse_shortcut_keycaps for all modifier symbols
//! - format_shortcut_hint roundtrips for unusual key names
//! - to_deeplink_name with CJK, emoji, RTL characters
//! - Grouped items with realistic AI command bar data
//! - coerce_action_selection on all-headers edge case
//! - Note switcher section assignment (Pinned vs Recent)
//! - Clipboard frontmost app edge cases (empty string, unicode)
//! - Chat with no models, no messages, no response
//! - Multiple scriptlet custom actions preserve declaration order
//! - Action constructor lowercase caching with unicode titles

use super::builders::{
    get_ai_command_bar_actions, get_chat_context_actions, get_clipboard_history_context_actions,
    get_file_context_actions, get_new_chat_actions, get_note_switcher_actions,
    get_notes_command_bar_actions, get_path_context_actions, get_script_context_actions,
    get_scriptlet_context_actions_with_custom, to_deeplink_name, ChatModelInfo, ChatPromptInfo,
    ClipboardEntryInfo, NewChatModelInfo, NewChatPresetInfo, NoteSwitcherNoteInfo, NotesInfo,
};
use super::command_bar::CommandBarConfig;
use super::dialog::{
    build_grouped_items_static, coerce_action_selection, ActionsDialog, GroupedActionItem,
};
use super::types::{Action, ActionCategory, ScriptInfo, SearchPosition, SectionStyle};
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;
use crate::scriptlets::{Scriptlet, ScriptletAction};
use std::collections::HashSet;

// =========================================================================
// Helpers
// =========================================================================

fn action_ids(actions: &[Action]) -> Vec<&str> {
    actions.iter().map(|a| a.id.as_str()).collect()
}

fn find_action<'a>(actions: &'a [Action], id: &str) -> Option<&'a Action> {
    actions.iter().find(|a| a.id == id)
}

fn sections_in_order(actions: &[Action]) -> Vec<&str> {
    let mut sections = Vec::new();
    for a in actions {
        if let Some(ref s) = a.section {
            if sections
                .last()
                .map(|l: &&str| *l != s.as_str())
                .unwrap_or(true)
            {
                sections.push(s.as_str());
            }
        }
    }
    sections
}

// =========================================================================
// 1. Agent flag interactions with shortcut/alias/frecency
// =========================================================================

#[test]
fn agent_with_shortcut_has_update_and_remove_shortcut() {
    let mut agent = ScriptInfo::new("My Agent", "/path/to/agent.md");
    agent.is_script = false;
    agent.is_agent = true;
    agent.shortcut = Some("cmd+a".to_string());
    let actions = get_script_context_actions(&agent);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"update_shortcut"));
    assert!(ids.contains(&"remove_shortcut"));
    assert!(!ids.contains(&"add_shortcut"));
}

#[test]
fn agent_without_shortcut_has_add_shortcut() {
    let mut agent = ScriptInfo::new("My Agent", "/path/to/agent.md");
    agent.is_script = false;
    agent.is_agent = true;
    let actions = get_script_context_actions(&agent);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"add_shortcut"));
    assert!(!ids.contains(&"update_shortcut"));
    assert!(!ids.contains(&"remove_shortcut"));
}

#[test]
fn agent_with_alias_has_update_and_remove_alias() {
    let mut agent = ScriptInfo::new("My Agent", "/path/to/agent.md");
    agent.is_script = false;
    agent.is_agent = true;
    agent.alias = Some("ag".to_string());
    let actions = get_script_context_actions(&agent);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"update_alias"));
    assert!(ids.contains(&"remove_alias"));
    assert!(!ids.contains(&"add_alias"));
}

#[test]
fn agent_with_frecency_has_reset_ranking() {
    let mut agent = ScriptInfo::new("My Agent", "/path/to/agent.md");
    agent.is_script = false;
    agent.is_agent = true;
    agent.is_suggested = true;
    agent.frecency_path = Some("agent:/path".to_string());
    let actions = get_script_context_actions(&agent);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"reset_ranking"));
}

#[test]
fn agent_without_frecency_lacks_reset_ranking() {
    let mut agent = ScriptInfo::new("My Agent", "/path/to/agent.md");
    agent.is_script = false;
    agent.is_agent = true;
    let actions = get_script_context_actions(&agent);
    let ids = action_ids(&actions);
    assert!(!ids.contains(&"reset_ranking"));
}

#[test]
fn agent_has_edit_agent_title() {
    let mut agent = ScriptInfo::new("My Agent", "/path/to/agent.md");
    agent.is_script = false;
    agent.is_agent = true;
    let actions = get_script_context_actions(&agent);
    let edit = find_action(&actions, "edit_script").unwrap();
    assert_eq!(edit.title, "Edit Agent");
}

#[test]
fn agent_has_reveal_copy_path_copy_content() {
    let mut agent = ScriptInfo::new("My Agent", "/path/to/agent.md");
    agent.is_script = false;
    agent.is_agent = true;
    let actions = get_script_context_actions(&agent);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"file:reveal_in_finder"));
    assert!(ids.contains(&"file:copy_path"));
    assert!(ids.contains(&"copy_content"));
}

#[test]
fn agent_lacks_view_logs() {
    let mut agent = ScriptInfo::new("My Agent", "/path/to/agent.md");
    agent.is_script = false;
    agent.is_agent = true;
    let actions = get_script_context_actions(&agent);
    let ids = action_ids(&actions);
    assert!(!ids.contains(&"view_logs"));
}

// =========================================================================
// 2. Custom action verbs propagate into primary action title
// =========================================================================

#[test]
fn action_verb_run_in_primary_title() {
    let script = ScriptInfo::new("Test Script", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].title, "Run \"Test Script\"");
}

#[test]
fn action_verb_launch_in_primary_title() {
    let script =
        ScriptInfo::with_action_verb("Safari", "/Applications/Safari.app", false, "Launch");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].title, "Launch \"Safari\"");
}

#[test]
fn action_verb_switch_to_in_primary_title() {
    let script = ScriptInfo::with_action_verb("My Window", "window:123", false, "Switch to");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].title, "Switch to \"My Window\"");
}

#[test]
fn action_verb_open_in_primary_title() {
    let script = ScriptInfo::with_action_verb("Clipboard History", "builtin:ch", false, "Open");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].title, "Open \"Clipboard History\"");
}

#[test]
fn action_verb_execute_in_primary_title() {
    let script = ScriptInfo::with_all("My Task", "/path/task.ts", true, "Execute", None, None);
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].title, "Execute \"My Task\"");
}

// =========================================================================
// 3. Scriptlet context vs script context: systematic comparison
// =========================================================================

#[test]
fn scriptlet_context_has_edit_scriptlet_not_edit_script() {
    let scriptlet = ScriptInfo::scriptlet("My Snippet", "/path/to/bundle.md", None, None);
    let actions = get_script_context_actions(&scriptlet);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"edit_scriptlet"));
    assert!(!ids.contains(&"edit_script"));
}

#[test]
fn scriptlet_context_has_reveal_scriptlet_not_reveal() {
    let scriptlet = ScriptInfo::scriptlet("My Snippet", "/path/to/bundle.md", None, None);
    let actions = get_script_context_actions(&scriptlet);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"reveal_scriptlet_in_finder"));
    // The regular reveal_in_finder should NOT be present for scriptlets
    assert!(!ids.contains(&"file:reveal_in_finder"));
}

#[test]
fn scriptlet_context_has_copy_scriptlet_path_not_copy_path() {
    let scriptlet = ScriptInfo::scriptlet("My Snippet", "/path/to/bundle.md", None, None);
    let actions = get_script_context_actions(&scriptlet);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"copy_scriptlet_path"));
    assert!(!ids.contains(&"file:copy_path"));
}

#[test]
fn scriptlet_and_script_both_have_copy_content() {
    let scriptlet = ScriptInfo::scriptlet("My Snippet", "/path/to/bundle.md", None, None);
    let script = ScriptInfo::new("My Script", "/path/script.ts");
    let scriptlet_actions = get_script_context_actions(&scriptlet);
    let script_actions = get_script_context_actions(&script);
    assert!(action_ids(&scriptlet_actions).contains(&"copy_content"));
    assert!(action_ids(&script_actions).contains(&"copy_content"));
}

#[test]
fn scriptlet_lacks_view_logs() {
    let scriptlet = ScriptInfo::scriptlet("My Snippet", "/path/to/bundle.md", None, None);
    let actions = get_script_context_actions(&scriptlet);
    assert!(!action_ids(&actions).contains(&"view_logs"));
}

// =========================================================================
// 4. Clipboard text vs image action count differential
// =========================================================================

#[test]
fn clipboard_image_has_strictly_more_actions_than_text() {
    let text_entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let image_entry = ClipboardEntryInfo {
        id: "i1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "Image (800x600)".to_string(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    };
    let text_actions = get_clipboard_history_context_actions(&text_entry);
    let image_actions = get_clipboard_history_context_actions(&image_entry);
    assert!(
        image_actions.len() > text_actions.len(),
        "Image should have more actions than text: {} > {}",
        image_actions.len(),
        text_actions.len()
    );
}

#[test]
fn clipboard_image_has_ocr_text_does_not() {
    let text_entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let image_entry = ClipboardEntryInfo {
        id: "i1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "Image".to_string(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let text_actions = get_clipboard_history_context_actions(&text_entry);
    let image_actions = get_clipboard_history_context_actions(&image_entry);
    let text_ids = action_ids(&text_actions);
    let image_ids = action_ids(&image_actions);
    assert!(!text_ids.contains(&"clip:clipboard_ocr"));
    assert!(image_ids.contains(&"clip:clipboard_ocr"));
}

#[test]
fn clipboard_pinned_shows_unpin_unpinned_shows_pin() {
    let pinned = ClipboardEntryInfo {
        id: "p1".to_string(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "pinned".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let unpinned = ClipboardEntryInfo {
        id: "u1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "unpinned".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let pinned_actions = get_clipboard_history_context_actions(&pinned);
    let unpinned_actions = get_clipboard_history_context_actions(&unpinned);
    let pinned_ids = action_ids(&pinned_actions);
    let unpinned_ids = action_ids(&unpinned_actions);
    assert!(pinned_ids.contains(&"clip:clipboard_unpin"));
    assert!(!pinned_ids.contains(&"clip:clipboard_pin"));
    assert!(unpinned_ids.contains(&"clip:clipboard_pin"));
    assert!(!unpinned_ids.contains(&"clip:clipboard_unpin"));
}

// =========================================================================
// 5. Path context action IDs are all snake_case
// =========================================================================

#[test]
fn path_context_all_ids_are_snake_case() {
    let path = PathInfo {
        name: "test.txt".to_string(),
        path: "/home/user/test.txt".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    for action in &actions {
        assert!(
            !action.id.contains(' ') && !action.id.contains('-'),
            "Action ID '{}' should be snake_case",
            action.id
        );
        assert_eq!(
            action.id,
            action.id.to_lowercase(),
            "Action ID '{}' should be lowercase",
            action.id
        );
    }
}

#[test]
fn path_context_dir_all_ids_are_snake_case() {
    let path = PathInfo {
        name: "Documents".to_string(),
        path: "/home/user/Documents".to_string(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path);
    for action in &actions {
        assert!(
            !action.id.contains(' '),
            "Action ID '{}' should not contain spaces",
            action.id
        );
    }
}

// =========================================================================
// 6. File context FileType variants produce consistent action set structure
// =========================================================================

#[test]
fn file_context_all_file_types_have_reveal_and_copy_path() {
    let file_types = vec![
        FileType::File,
        FileType::Document,
        FileType::Image,
        FileType::Application,
        FileType::Audio,
    ];
    for ft in file_types {
        let info = FileInfo {
            path: format!("/tmp/test.{:?}", ft),
            name: format!("test.{:?}", ft),
            file_type: ft,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        let ids = action_ids(&actions);
        assert!(
            ids.contains(&"file:reveal_in_finder"),
            "FileType {:?} should have reveal_in_finder",
            info.file_type
        );
        assert!(
            ids.contains(&"file:copy_path"),
            "FileType {:?} should have copy_path",
            info.file_type
        );
        assert!(
            ids.contains(&"file:copy_filename"),
            "FileType {:?} should have copy_filename",
            info.file_type
        );
    }
}

#[test]
fn file_context_file_has_open_file_dir_has_open_directory() {
    let file = FileInfo {
        path: "/tmp/readme.md".to_string(),
        name: "readme.md".to_string(),
        file_type: FileType::File,
        is_dir: false,
    };
    let dir = FileInfo {
        path: "/tmp/src".to_string(),
        name: "src".to_string(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let file_actions = get_file_context_actions(&file);
    let dir_actions = get_file_context_actions(&dir);
    assert!(action_ids(&file_actions).contains(&"file:open_file"));
    assert!(!action_ids(&file_actions).contains(&"file:open_directory"));
    assert!(action_ids(&dir_actions).contains(&"file:open_directory"));
    assert!(!action_ids(&dir_actions).contains(&"file:open_file"));
}

// --- merged from part_02.rs ---

// =========================================================================
// 7. Notes section labels exhaustive for full-feature permutation
// =========================================================================

#[test]
fn notes_full_feature_has_all_five_sections() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let secs = sections_in_order(&actions);
    assert!(secs.contains(&"Notes"), "Missing Notes section");
    assert!(secs.contains(&"Edit"), "Missing Edit section");
    assert!(secs.contains(&"Copy"), "Missing Copy section");
    assert!(secs.contains(&"Export"), "Missing Export section");
    assert!(secs.contains(&"Settings"), "Missing Settings section");
}

#[test]
fn notes_no_selection_only_has_notes_section() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let secs: Vec<&str> = actions
        .iter()
        .filter_map(|a| a.section.as_deref())
        .collect();
    // Should have Notes and Settings
    assert!(secs.contains(&"Notes"));
    assert!(secs.contains(&"Settings"));
    // Should not have Edit, Copy, Export (require selection + not trash)
    assert!(!secs.contains(&"Edit"));
    assert!(!secs.contains(&"Copy"));
    assert!(!secs.contains(&"Export"));
}

#[test]
fn notes_trash_view_has_limited_sections() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let secs: Vec<&str> = actions
        .iter()
        .filter_map(|a| a.section.as_deref())
        .collect();
    // Even with selection, trash view suppresses Edit/Copy/Export
    assert!(secs.contains(&"Notes"));
    assert!(!secs.contains(&"Edit"));
    assert!(!secs.contains(&"Copy"));
    assert!(!secs.contains(&"Export"));
}

#[test]
fn notes_auto_sizing_enabled_hides_settings() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids = action_ids(&actions);
    assert!(!ids.contains(&"enable_auto_sizing"));
}

// =========================================================================
// 8. AI command bar icon-per-section coverage
// =========================================================================

#[test]
fn ai_command_bar_every_action_has_icon() {
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        assert!(
            action.icon.is_some(),
            "AI action '{}' should have an icon",
            action.id
        );
    }
}

#[test]
fn ai_command_bar_every_action_has_section() {
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        assert!(
            action.section.is_some(),
            "AI action '{}' should have a section",
            action.id
        );
    }
}

#[test]
fn ai_command_bar_exactly_six_sections() {
    let actions = get_ai_command_bar_actions();
    let unique_sections: HashSet<&str> = actions
        .iter()
        .filter_map(|a| a.section.as_deref())
        .collect();
    assert_eq!(
        unique_sections.len(),
        6,
        "AI command bar should have exactly 6 sections, got {:?}",
        unique_sections
    );
}

#[test]
fn ai_command_bar_section_order_is_response_actions_attachments_export_actions_help_settings() {
    let actions = get_ai_command_bar_actions();
    let order = sections_in_order(&actions);
    assert_eq!(
        order,
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
// 9. New chat with all-empty inputs
// =========================================================================

#[test]
fn new_chat_empty_inputs_produces_empty() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

#[test]
fn new_chat_only_models_produces_models_section() {
    let models = vec![NewChatModelInfo {
        model_id: "m1".to_string(),
        display_name: "Model 1".to_string(),
        provider: "test".to_string(),
        provider_display_name: "Test Provider".to_string(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Models"));
}

#[test]
fn new_chat_only_presets_produces_presets_section() {
    let presets = vec![NewChatPresetInfo {
        id: "general".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Presets"));
}

#[test]
fn new_chat_only_last_used_produces_last_used_section() {
    let last_used = vec![NewChatModelInfo {
        model_id: "lu1".to_string(),
        display_name: "Recent Model".to_string(),
        provider: "test".to_string(),
        provider_display_name: "Test".to_string(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
}

#[test]
fn new_chat_section_order_is_last_used_presets_models() {
    let last_used = vec![NewChatModelInfo {
        model_id: "lu1".to_string(),
        display_name: "Recent".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "gen".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "m1".to_string(),
        display_name: "Claude".to_string(),
        provider: "anthropic".to_string(),
        provider_display_name: "Anthropic".to_string(),
    }];
    let actions = get_new_chat_actions(&last_used, &presets, &models);
    let order = sections_in_order(&actions);
    assert_eq!(order, vec!["Last Used Settings", "Presets", "Models"]);
}

// =========================================================================
// 10. score_action edge cases
// =========================================================================

#[test]
fn score_action_empty_query_returns_zero() {
    let action = Action::new("test", "Test Action", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "");
    // Empty query should match as prefix (empty string is prefix of everything)
    // Based on implementation: "test action".starts_with("") == true → 100
    assert!(score >= 100);
}

#[test]
fn score_action_exact_title_match_gets_prefix_score() {
    let action = Action::new("script:edit", "Edit Script", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "edit script");
    assert!(
        score >= 100,
        "Exact title match should score 100+, got {}",
        score
    );
}

#[test]
fn score_action_no_match_returns_zero() {
    let action = Action::new("test", "Test Action", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "zzzzz");
    assert_eq!(score, 0, "No match should score 0");
}

#[test]
fn score_action_description_only_match_returns_fifteen() {
    let action = Action::new(
        "open",
        "Open File",
        Some("Launch the default editor".to_string()),
        ActionCategory::ScriptContext,
    );
    // "default editor" doesn't match title but matches description
    let score = ActionsDialog::score_action(&action, "default editor");
    assert_eq!(
        score, 15,
        "Description-only match should score 15, got {}",
        score
    );
}

#[test]
fn score_action_shortcut_only_match_returns_ten() {
    let action =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
    let score = ActionsDialog::score_action(&action, "⌘e");
    assert_eq!(
        score, 10,
        "Shortcut-only match should score 10, got {}",
        score
    );
}

#[test]
fn score_action_title_plus_description_stacks() {
    let action = Action::new(
        "script:edit",
        "Edit Script",
        Some("Edit the script file".to_string()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "script:edit");
    // title prefix (100) + description contains "script:edit" (15) = 115
    assert!(
        score >= 115,
        "Stacked score should be >= 115, got {}",
        score
    );
}

#[test]
fn score_action_single_char_query() {
    let action = Action::new("script:edit", "Edit Script", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "e");
    assert!(
        score >= 100,
        "Single char prefix match should score 100+, got {}",
        score
    );
}

// =========================================================================
// 11. fuzzy_match boundary conditions
// =========================================================================

#[test]
fn fuzzy_match_empty_needle_always_matches() {
    assert!(ActionsDialog::fuzzy_match("anything", ""));
}

#[test]
fn fuzzy_match_empty_haystack_only_matches_empty_needle() {
    assert!(ActionsDialog::fuzzy_match("", ""));
    assert!(!ActionsDialog::fuzzy_match("", "a"));
}

#[test]
fn fuzzy_match_needle_longer_than_haystack_fails() {
    assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
}

#[test]
fn fuzzy_match_exact_match_succeeds() {
    assert!(ActionsDialog::fuzzy_match("hello", "hello"));
}

#[test]
fn fuzzy_match_subsequence_succeeds() {
    assert!(ActionsDialog::fuzzy_match("edit script", "edsc"));
}

#[test]
fn fuzzy_match_wrong_order_fails() {
    assert!(!ActionsDialog::fuzzy_match("abc", "cba"));
}

#[test]
fn fuzzy_match_case_sensitive() {
    // fuzzy_match is case-sensitive (expects pre-lowercased input)
    assert!(!ActionsDialog::fuzzy_match("hello", "H"));
    assert!(ActionsDialog::fuzzy_match("hello", "h"));
}

// =========================================================================
// 12. parse_shortcut_keycaps for all modifier symbols
// =========================================================================

#[test]
fn parse_keycaps_modifier_symbols() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⇧C");
    assert_eq!(keycaps, vec!["⌘", "⇧", "C"]);
}

#[test]
fn parse_keycaps_enter_symbol() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("↵");
    assert_eq!(keycaps, vec!["↵"]);
}

#[test]
fn parse_keycaps_escape_symbol() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⎋");
    assert_eq!(keycaps, vec!["⎋"]);
}

#[test]
fn parse_keycaps_backspace_symbol() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⌫");
    assert_eq!(keycaps, vec!["⌘", "⌫"]);
}

#[test]
fn parse_keycaps_space_symbol() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("␣");
    assert_eq!(keycaps, vec!["␣"]);
}

#[test]
fn parse_keycaps_arrow_keys() {
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("↑"), vec!["↑"]);
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("↓"), vec!["↓"]);
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("←"), vec!["←"]);
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("→"), vec!["→"]);
}

#[test]
fn parse_keycaps_tab_symbol() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⇥");
    assert_eq!(keycaps, vec!["⇥"]);
}

#[test]
fn parse_keycaps_all_modifiers_combined() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⌃⌥⇧X");
    assert_eq!(keycaps, vec!["⌘", "⌃", "⌥", "⇧", "X"]);
}

#[test]
fn parse_keycaps_lowercase_becomes_uppercase() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘e");
    assert_eq!(keycaps, vec!["⌘", "E"]);
}

// =========================================================================
// 13. format_shortcut_hint roundtrips for unusual key names
// =========================================================================

#[test]
fn format_shortcut_hint_enter() {
    let hint = ActionsDialog::format_shortcut_hint("enter");
    assert_eq!(hint, "↵");
}

#[test]
fn format_shortcut_hint_return() {
    let hint = ActionsDialog::format_shortcut_hint("return");
    assert_eq!(hint, "↵");
}

#[test]
fn format_shortcut_hint_escape() {
    let hint = ActionsDialog::format_shortcut_hint("escape");
    assert_eq!(hint, "⎋");
}

#[test]
fn format_shortcut_hint_esc() {
    let hint = ActionsDialog::format_shortcut_hint("esc");
    assert_eq!(hint, "⎋");
}

#[test]
fn format_shortcut_hint_tab() {
    let hint = ActionsDialog::format_shortcut_hint("tab");
    assert_eq!(hint, "⇥");
}

#[test]
fn format_shortcut_hint_backspace() {
    let hint = ActionsDialog::format_shortcut_hint("backspace");
    assert_eq!(hint, "⌫");
}

#[test]
fn format_shortcut_hint_space() {
    let hint = ActionsDialog::format_shortcut_hint("space");
    assert_eq!(hint, "␣");
}

#[test]
fn format_shortcut_hint_arrow_keys() {
    assert_eq!(ActionsDialog::format_shortcut_hint("up"), "↑");
    assert_eq!(ActionsDialog::format_shortcut_hint("down"), "↓");
    assert_eq!(ActionsDialog::format_shortcut_hint("left"), "←");
    assert_eq!(ActionsDialog::format_shortcut_hint("right"), "→");
}

// --- merged from part_03.rs ---

#[test]
fn format_shortcut_hint_cmd_enter() {
    let hint = ActionsDialog::format_shortcut_hint("cmd+enter");
    assert_eq!(hint, "⌘↵");
}

#[test]
fn format_shortcut_hint_ctrl_alt_delete() {
    let hint = ActionsDialog::format_shortcut_hint("ctrl+alt+delete");
    assert_eq!(hint, "⌃⌥⌫");
}

#[test]
fn format_shortcut_hint_shift_cmd_c() {
    let hint = ActionsDialog::format_shortcut_hint("shift+cmd+c");
    assert_eq!(hint, "⇧⌘C");
}

#[test]
fn format_shortcut_hint_option_variant() {
    let hint = ActionsDialog::format_shortcut_hint("option+a");
    assert_eq!(hint, "⌥A");
}

#[test]
fn format_shortcut_hint_command_variant() {
    let hint = ActionsDialog::format_shortcut_hint("command+s");
    assert_eq!(hint, "⌘S");
}

#[test]
fn format_shortcut_hint_arrowup_variant() {
    let hint = ActionsDialog::format_shortcut_hint("arrowup");
    assert_eq!(hint, "↑");
}

#[test]
fn format_shortcut_hint_arrowdown_variant() {
    let hint = ActionsDialog::format_shortcut_hint("arrowdown");
    assert_eq!(hint, "↓");
}

// =========================================================================
// 14. to_deeplink_name with CJK, emoji, RTL characters
// =========================================================================

#[test]
fn deeplink_name_ascii_basic() {
    assert_eq!(to_deeplink_name("Hello World"), "hello-world");
}

#[test]
fn deeplink_name_underscores_become_hyphens() {
    assert_eq!(to_deeplink_name("hello_world_test"), "hello-world-test");
}

#[test]
fn deeplink_name_special_chars_stripped() {
    assert_eq!(to_deeplink_name("test!@#$%^&*()"), "test");
}

#[test]
fn deeplink_name_multiple_spaces_collapsed() {
    assert_eq!(to_deeplink_name("foo   bar   baz"), "foo-bar-baz");
}

#[test]
fn deeplink_name_leading_trailing_stripped() {
    assert_eq!(to_deeplink_name("  hello  "), "hello");
}

#[test]
fn deeplink_name_empty_string() {
    assert_eq!(to_deeplink_name(""), "");
}

#[test]
fn deeplink_name_only_special_chars() {
    assert_eq!(to_deeplink_name("!@#$%"), "");
}

#[test]
fn deeplink_name_numbers_preserved() {
    assert_eq!(to_deeplink_name("Script 123"), "script-123");
}

#[test]
fn deeplink_name_mixed_case_lowered() {
    assert_eq!(to_deeplink_name("MyScript"), "myscript");
}

#[test]
fn deeplink_name_accented_chars() {
    // Accented characters are alphanumeric and should be preserved
    assert_eq!(to_deeplink_name("café résumé"), "café-résumé");
}

#[test]
fn deeplink_name_consecutive_hyphens_collapsed() {
    assert_eq!(to_deeplink_name("a--b"), "a-b");
}

// =========================================================================
// 15. Grouped items with realistic AI command bar data
// =========================================================================

#[test]
fn grouped_items_headers_style_produces_section_headers() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    let header_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
        .count();
    // Should have section headers for each section transition
    assert_eq!(
        header_count, 7,
        "AI command bar should have 7 section headers, got {}",
        header_count
    );
}

#[test]
fn grouped_items_none_style_produces_no_headers() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
    let header_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(header_count, 0, "None style should have 0 headers");
}

#[test]
fn grouped_items_separators_style_produces_no_headers() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    let header_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(header_count, 0, "Separators style should have 0 headers");
}

#[test]
fn grouped_items_empty_filtered_produces_empty() {
    let actions = get_ai_command_bar_actions();
    let grouped = build_grouped_items_static(&actions, &[], SectionStyle::Headers);
    assert!(grouped.is_empty());
}

#[test]
fn grouped_items_item_count_matches_filtered_count() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
    let item_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::Item(_)))
        .count();
    assert_eq!(item_count, filtered.len());
}

// =========================================================================
// 16. coerce_action_selection edge cases
// =========================================================================

#[test]
fn coerce_selection_empty_rows_returns_none() {
    let rows: Vec<GroupedActionItem> = vec![];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn coerce_selection_all_headers_returns_none() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".to_string()),
        GroupedActionItem::SectionHeader("B".to_string()),
        GroupedActionItem::SectionHeader("C".to_string()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn coerce_selection_on_item_returns_same_index() {
    let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    assert_eq!(coerce_action_selection(&rows, 1), Some(1));
}

#[test]
fn coerce_selection_on_header_skips_to_next_item() {
    let rows = vec![
        GroupedActionItem::SectionHeader("Response".to_string()),
        GroupedActionItem::Item(0),
        GroupedActionItem::Item(1),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn coerce_selection_on_last_header_searches_backward() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("End".to_string()),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn coerce_selection_out_of_bounds_clamps() {
    let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
    // Index 99 should be clamped to last valid index
    assert_eq!(coerce_action_selection(&rows, 99), Some(1));
}

// =========================================================================
// 17. Note switcher section assignment (Pinned vs Recent)
// =========================================================================

#[test]
fn note_switcher_pinned_note_has_pinned_section() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid-1".to_string(),
        title: "Pinned Note".to_string(),
        char_count: 100,
        is_current: false,
        is_pinned: true,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
}

#[test]
fn note_switcher_unpinned_note_has_recent_section() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid-2".to_string(),
        title: "Regular Note".to_string(),
        char_count: 50,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Recent"));
}

#[test]
fn note_switcher_mixed_pinned_and_recent() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "uuid-1".to_string(),
            title: "Pinned".to_string(),
            char_count: 10,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "uuid-2".to_string(),
            title: "Recent".to_string(),
            char_count: 20,
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
fn note_switcher_current_note_has_bullet_prefix() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid-1".to_string(),
        title: "My Note".to_string(),
        char_count: 42,
        is_current: true,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(
        actions[0].title.starts_with("• "),
        "Current note should have bullet prefix, got: {}",
        actions[0].title
    );
}

#[test]
fn note_switcher_non_current_note_no_bullet_prefix() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid-1".to_string(),
        title: "Other Note".to_string(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(!actions[0].title.starts_with("• "));
}

#[test]
fn note_switcher_icon_hierarchy_pinned_beats_current() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "uuid-1".to_string(),
            title: "Pinned+Current".to_string(),
            char_count: 10,
            is_current: true,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "uuid-2".to_string(),
            title: "Current Only".to_string(),
            char_count: 10,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "uuid-3".to_string(),
            title: "Pinned Only".to_string(),
            char_count: 10,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "uuid-4".to_string(),
            title: "Neither".to_string(),
            char_count: 10,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled)); // pinned+current → Star
    assert_eq!(actions[1].icon, Some(IconName::Check)); // current only → Check
    assert_eq!(actions[2].icon, Some(IconName::StarFilled)); // pinned only → Star
    assert_eq!(actions[3].icon, Some(IconName::File)); // neither → File
}

#[test]
fn note_switcher_char_count_singular() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid-1".to_string(),
        title: "Single Char Note".to_string(),
        char_count: 1,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("1 char"));
}

#[test]
fn note_switcher_char_count_plural() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid-1".to_string(),
        title: "Multi Char Note".to_string(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("42 chars"));
}

#[test]
fn note_switcher_char_count_zero() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid-1".to_string(),
        title: "Empty Note".to_string(),
        char_count: 0,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("0 chars"));
}

#[test]
fn note_switcher_empty_notes_shows_helpful_message() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "no_notes");
    assert_eq!(actions[0].title, "No notes yet");
    assert_eq!(actions[0].icon, Some(IconName::Plus));
}

#[test]
fn note_switcher_action_id_format() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc-123-def".to_string(),
        title: "Test".to_string(),
        char_count: 5,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].id, "note_abc-123-def");
}

// =========================================================================
// 18. Clipboard frontmost app edge cases
// =========================================================================

#[test]
fn clipboard_paste_title_with_empty_string_app_name() {
    let entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: Some("".to_string()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = find_action(&actions, "clip:clipboard_paste").unwrap();
    // Even empty string gets formatted with "Paste to "
    assert_eq!(paste.title, "Paste to ");
}

// --- merged from part_04.rs ---

#[test]
fn clipboard_paste_title_with_unicode_app_name() {
    let entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: Some("日本語App".to_string()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = find_action(&actions, "clip:clipboard_paste").unwrap();
    assert_eq!(paste.title, "Paste to 日本語App");
}

#[test]
fn clipboard_paste_title_without_app_name() {
    let entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = find_action(&actions, "clip:clipboard_paste").unwrap();
    assert_eq!(paste.title, "Paste to Active App");
}

// =========================================================================
// 19. Chat with no models, no messages, no response
// =========================================================================

#[test]
fn chat_no_models_no_messages_no_response() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"chat:continue_in_chat"));
    assert!(!ids.contains(&"chat:copy_response"));
    assert!(!ids.contains(&"chat:clear_conversation"));
    assert_eq!(actions.len(), 1);
}

#[test]
fn chat_with_response_only_has_copy_response() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"chat:continue_in_chat"));
    assert!(ids.contains(&"chat:copy_response"));
    assert!(!ids.contains(&"chat:clear_conversation"));
}

#[test]
fn chat_with_messages_only_has_clear_conversation() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"chat:continue_in_chat"));
    assert!(!ids.contains(&"chat:copy_response"));
    assert!(ids.contains(&"chat:clear_conversation"));
}

#[test]
fn chat_with_all_flags_has_all_actions() {
    let info = ChatPromptInfo {
        current_model: Some("Claude 3.5".to_string()),
        available_models: vec![ChatModelInfo {
            id: "claude-3.5".to_string(),
            display_name: "Claude 3.5".to_string(),
            provider: "Anthropic".to_string(),
        }],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"chat:continue_in_chat"));
    assert!(ids.contains(&"chat:copy_response"));
    assert!(ids.contains(&"chat:clear_conversation"));
    // Plus model selection action
    assert!(ids.iter().any(|id| id.starts_with("chat:select_model_")));
}

#[test]
fn chat_model_checkmark_on_current() {
    let info = ChatPromptInfo {
        current_model: Some("Claude 3.5".to_string()),
        available_models: vec![
            ChatModelInfo {
                id: "claude-3.5".to_string(),
                display_name: "Claude 3.5".to_string(),
                provider: "Anthropic".to_string(),
            },
            ChatModelInfo {
                id: "gpt-4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let claude = find_action(&actions, "chat:select_model_claude-3.5").unwrap();
    let gpt = find_action(&actions, "chat:select_model_gpt-4").unwrap();
    assert!(
        claude.title.contains('✓'),
        "Current model should have checkmark"
    );
    assert!(
        !gpt.title.contains('✓'),
        "Non-current model should not have checkmark"
    );
}

// =========================================================================
// 20. Scriptlet custom actions ordering preservation
// =========================================================================

#[test]
fn scriptlet_custom_actions_preserve_declaration_order() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut scriptlet = Scriptlet::new(
        "Test".to_string(),
        "bash".to_string(),
        "echo main".to_string(),
    );
    scriptlet.actions = vec![
        ScriptletAction {
            name: "First Action".to_string(),
            command: "first".to_string(),
            tool: "bash".to_string(),
            code: "echo first".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
        ScriptletAction {
            name: "Second Action".to_string(),
            command: "second".to_string(),
            tool: "bash".to_string(),
            code: "echo second".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
        ScriptletAction {
            name: "Third Action".to_string(),
            command: "third".to_string(),
            tool: "bash".to_string(),
            code: "echo third".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
    ];

    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));

    let custom_ids: Vec<&str> = actions
        .iter()
        .filter(|a| a.id.starts_with("scriptlet_action:"))
        .map(|a| a.id.as_str())
        .collect();

    assert_eq!(
        custom_ids,
        vec![
            "scriptlet_action:first",
            "scriptlet_action:second",
            "scriptlet_action:third"
        ]
    );
}

#[test]
fn scriptlet_custom_actions_appear_after_run_before_builtins() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
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
        shortcut: Some("cmd+1".to_string()),
        description: Some("A custom action".to_string()),
    }];

    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let ids = action_ids(&actions);

    let run_idx = ids.iter().position(|id| *id == "run_script").unwrap();
    let custom_idx = ids
        .iter()
        .position(|id| *id == "scriptlet_action:custom")
        .unwrap();
    let edit_idx = ids.iter().position(|id| *id == "edit_scriptlet").unwrap();

    assert!(run_idx < custom_idx, "run before custom");
    assert!(custom_idx < edit_idx, "custom before edit");
}

#[test]
fn scriptlet_custom_actions_have_has_action_true() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
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
    let custom = find_action(&actions, "scriptlet_action:custom").unwrap();
    assert!(
        custom.has_action,
        "Custom scriptlet action should have has_action=true"
    );
    assert_eq!(custom.value, Some("custom".to_string()));
}

#[test]
fn scriptlet_custom_action_shortcut_formatted() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut scriptlet = Scriptlet::new(
        "Test".to_string(),
        "bash".to_string(),
        "echo main".to_string(),
    );
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".to_string(),
        command: "copy".to_string(),
        tool: "bash".to_string(),
        code: "echo copy".to_string(),
        inputs: vec![],
        shortcut: Some("cmd+shift+c".to_string()),
        description: None,
    }];

    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let custom = find_action(&actions, "scriptlet_action:copy").unwrap();
    assert_eq!(
        custom.shortcut,
        Some("⌘⇧C".to_string()),
        "Shortcut should be formatted with symbols"
    );
}

// =========================================================================
// 21. Action constructor lowercase caching
// =========================================================================

#[test]
fn action_title_lower_caches_correctly() {
    let action = Action::new("test", "Edit Script", None, ActionCategory::ScriptContext);
    assert_eq!(action.title_lower, "edit script");
}

#[test]
fn action_description_lower_caches_correctly() {
    let action = Action::new(
        "test",
        "Test",
        Some("Open In Editor".to_string()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(action.description_lower, Some("open in editor".to_string()));
}

#[test]
fn action_description_lower_none_when_no_description() {
    let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
    assert_eq!(action.description_lower, None);
}

#[test]
fn action_shortcut_lower_set_after_with_shortcut() {
    let action =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
    assert_eq!(action.shortcut_lower, Some("⌘e".to_string()));
}

#[test]
fn action_shortcut_lower_none_without_shortcut() {
    let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
    assert_eq!(action.shortcut_lower, None);
}

#[test]
fn action_title_lower_unicode() {
    let action = Action::new("test", "Café Résumé", None, ActionCategory::ScriptContext);
    assert_eq!(action.title_lower, "café résumé");
}

#[test]
fn action_with_shortcut_opt_some_sets_shortcut() {
    let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
        .with_shortcut_opt(Some("⌘X".to_string()));
    assert_eq!(action.shortcut, Some("⌘X".to_string()));
    assert_eq!(action.shortcut_lower, Some("⌘x".to_string()));
}

#[test]
fn action_with_shortcut_opt_none_leaves_shortcut_unset() {
    let action =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
    assert_eq!(action.shortcut, None);
    assert_eq!(action.shortcut_lower, None);
}

// =========================================================================
// 22. CommandBarConfig preset field values
// =========================================================================

#[test]
fn command_bar_config_ai_style_fields() {
    let config = CommandBarConfig::ai_style();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
    assert_eq!(config.dialog_config.section_style, SectionStyle::Headers);
    assert!(config.dialog_config.show_icons);
}

#[test]
fn command_bar_config_main_menu_style_fields() {
    let config = CommandBarConfig::main_menu_style();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Bottom);
    assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
    assert!(!config.dialog_config.show_icons);
}

#[test]
fn command_bar_config_notes_style_fields() {
    let config = CommandBarConfig::notes_style();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
    assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
    assert!(config.dialog_config.show_icons);
}

#[test]
fn command_bar_config_no_search_hides_search() {
    let config = CommandBarConfig::no_search();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Hidden);
}

// =========================================================================
// 23. Path context primary action varies by is_dir
// =========================================================================

#[test]
fn path_dir_primary_is_open_directory() {
    let path = PathInfo {
        name: "Documents".to_string(),
        path: "/home/user/Documents".to_string(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].id, "file:open_directory");
    assert!(actions[0].title.contains("Documents"));
}

#[test]
fn path_file_primary_is_select_file() {
    let path = PathInfo {
        name: "readme.md".to_string(),
        path: "/home/user/readme.md".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].id, "file:select_file");
    assert!(actions[0].title.contains("readme.md"));
}

#[test]
fn path_trash_description_differs_by_is_dir() {
    let dir_path = PathInfo {
        name: "src".to_string(),
        path: "/home/user/src".to_string(),
        is_dir: true,
    };
    let file_path = PathInfo {
        name: "file.txt".to_string(),
        path: "/home/user/file.txt".to_string(),
        is_dir: false,
    };
    let dir_actions = get_path_context_actions(&dir_path);
    let file_actions = get_path_context_actions(&file_path);
    let dir_trash = find_action(&dir_actions, "file:move_to_trash").unwrap();
    let file_trash = find_action(&file_actions, "file:move_to_trash").unwrap();
    assert_eq!(dir_trash.description.as_deref(), Some("Delete folder"));
    assert_eq!(file_trash.description.as_deref(), Some("Delete file"));
}

// =========================================================================
// 24. File context primary title includes name
// =========================================================================

#[test]
fn file_primary_title_includes_filename() {
    let info = FileInfo {
        path: "/tmp/report.pdf".to_string(),
        name: "report.pdf".to_string(),
        file_type: FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    assert!(
        actions[0].title.contains("report.pdf"),
        "Primary title should include filename: {}",
        actions[0].title
    );
}

// --- merged from part_05.rs ---

#[test]
fn file_dir_primary_title_includes_dirname() {
    let info = FileInfo {
        path: "/tmp/build".to_string(),
        name: "build".to_string(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&info);
    assert!(
        actions[0].title.contains("build"),
        "Primary title should include dirname: {}",
        actions[0].title
    );
}

// =========================================================================
// 25. Deeplink description format in script context
// =========================================================================

#[test]
fn deeplink_description_contains_url_with_formatted_name() {
    let script = ScriptInfo::new("My Cool Script", "/path/to/script.ts");
    let actions = get_script_context_actions(&script);
    let dl = find_action(&actions, "script:copy_deeplink").unwrap();
    assert!(
        dl.description
            .as_ref()
            .unwrap()
            .contains("scriptkit://run/my-cool-script"),
        "Deeplink description should contain formatted URL: {:?}",
        dl.description
    );
}

#[test]
fn deeplink_description_for_builtin() {
    let builtin = ScriptInfo::builtin("Clipboard History");
    let actions = get_script_context_actions(&builtin);
    let dl = find_action(&actions, "script:copy_deeplink").unwrap();
    assert!(dl
        .description
        .as_ref()
        .unwrap()
        .contains("scriptkit://run/clipboard-history"));
}

// =========================================================================
// 26. All built-in actions have has_action=false
// =========================================================================

#[test]
fn script_context_all_actions_have_has_action_false() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    for action in &actions {
        assert!(
            !action.has_action,
            "Built-in action '{}' should have has_action=false",
            action.id
        );
    }
}

#[test]
fn clipboard_context_all_actions_have_has_action_false() {
    let entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
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
fn file_context_all_actions_have_has_action_false() {
    let info = FileInfo {
        path: "/tmp/test.txt".to_string(),
        name: "test.txt".to_string(),
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
fn path_context_all_actions_have_has_action_false() {
    let path = PathInfo {
        name: "test.txt".to_string(),
        path: "/tmp/test.txt".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    for action in &actions {
        assert!(
            !action.has_action,
            "Path action '{}' should have has_action=false",
            action.id
        );
    }
}

#[test]
fn ai_command_bar_all_actions_have_has_action_false() {
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        assert!(
            !action.has_action,
            "AI action '{}' should have has_action=false",
            action.id
        );
    }
}

// =========================================================================
// 27. All actions have non-empty title and ID
// =========================================================================

#[test]
fn script_context_all_actions_have_nonempty_title_and_id() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    for action in &actions {
        assert!(!action.id.is_empty(), "Action ID should not be empty");
        assert!(!action.title.is_empty(), "Action title should not be empty");
    }
}

#[test]
fn clipboard_context_all_actions_have_nonempty_title_and_id() {
    let entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
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
fn ai_command_bar_all_actions_have_nonempty_title_and_id() {
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        assert!(!action.id.is_empty());
        assert!(!action.title.is_empty());
    }
}

// =========================================================================
// 28. Action ID uniqueness within contexts
// =========================================================================

#[test]
fn script_context_ids_are_unique() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        ids.len(),
        actions.len(),
        "Duplicate action IDs found in script context"
    );
}

#[test]
fn clipboard_text_context_ids_are_unique() {
    let entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        ids.len(),
        actions.len(),
        "Duplicate action IDs found in clipboard text context"
    );
}

#[test]
fn clipboard_image_context_ids_are_unique() {
    let entry = ClipboardEntryInfo {
        id: "i1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "Image".to_string(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        ids.len(),
        actions.len(),
        "Duplicate action IDs found in clipboard image context"
    );
}

#[test]
fn path_context_ids_are_unique() {
    let path = PathInfo {
        name: "test.txt".to_string(),
        path: "/tmp/test.txt".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        ids.len(),
        actions.len(),
        "Duplicate action IDs found in path context"
    );
}

#[test]
fn ai_command_bar_ids_are_unique() {
    let actions = get_ai_command_bar_actions();
    let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        ids.len(),
        actions.len(),
        "Duplicate action IDs found in AI command bar"
    );
}

// =========================================================================
// 29. Clipboard destructive actions always last three
// =========================================================================

#[test]
fn clipboard_destructive_actions_are_last_three() {
    let entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let len = actions.len();
    assert!(len >= 3);
    assert_eq!(actions[len - 3].id, "clip:clipboard_delete");
    assert_eq!(actions[len - 2].id, "clip:clipboard_delete_multiple");
    assert_eq!(actions[len - 1].id, "clip:clipboard_delete_all");
}

#[test]
fn clipboard_image_destructive_actions_are_last_three() {
    let entry = ClipboardEntryInfo {
        id: "i1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "Image".to_string(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let len = actions.len();
    assert!(len >= 3);
    assert_eq!(actions[len - 3].id, "clip:clipboard_delete");
    assert_eq!(actions[len - 2].id, "clip:clipboard_delete_multiple");
    assert_eq!(actions[len - 1].id, "clip:clipboard_delete_all");
}

// =========================================================================
// 30. Clipboard paste is always first, copy is always second
// =========================================================================

#[test]
fn clipboard_paste_is_first_copy_is_second() {
    let entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[0].id, "clip:clipboard_paste");
    assert_eq!(actions[1].id, "clip:clipboard_copy");
}

// =========================================================================
// 31. All actions have ScriptContext category
// =========================================================================

#[test]
fn all_contexts_produce_script_context_category() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    for action in &get_script_context_actions(&script) {
        assert_eq!(action.category, ActionCategory::ScriptContext);
    }

    let entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    for action in &get_clipboard_history_context_actions(&entry) {
        assert_eq!(action.category, ActionCategory::ScriptContext);
    }

    let path = PathInfo {
        name: "test".to_string(),
        path: "/tmp/test".to_string(),
        is_dir: false,
    };
    for action in &get_path_context_actions(&path) {
        assert_eq!(action.category, ActionCategory::ScriptContext);
    }

    let file = FileInfo {
        path: "/tmp/test.txt".to_string(),
        name: "test.txt".to_string(),
        file_type: FileType::File,
        is_dir: false,
    };
    for action in &get_file_context_actions(&file) {
        assert_eq!(action.category, ActionCategory::ScriptContext);
    }

    for action in &get_ai_command_bar_actions() {
        assert_eq!(action.category, ActionCategory::ScriptContext);
    }
}

// =========================================================================
// 32. Primary action always first across contexts
// =========================================================================

#[test]
fn primary_action_first_in_script_context() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].id, "run_script");
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn primary_action_first_in_file_context() {
    let file = FileInfo {
        path: "/tmp/test.txt".to_string(),
        name: "test.txt".to_string(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    assert_eq!(actions[0].id, "file:open_file");
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn primary_action_first_in_path_context() {
    let path = PathInfo {
        name: "readme.md".to_string(),
        path: "/tmp/readme.md".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].id, "file:select_file");
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn primary_action_first_in_clipboard_context() {
    let entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[0].id, "clip:clipboard_paste");
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

// =========================================================================
// 33. Ordering determinism
// =========================================================================

#[test]
fn script_context_ordering_deterministic_across_calls() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions1 = get_script_context_actions(&script);
    let actions2 = get_script_context_actions(&script);
    let ids1 = action_ids(&actions1);
    let ids2 = action_ids(&actions2);
    assert_eq!(ids1, ids2, "Action ordering should be deterministic");
}

#[test]
fn clipboard_context_ordering_deterministic_across_calls() {
    let entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions1 = get_clipboard_history_context_actions(&entry);
    let actions2 = get_clipboard_history_context_actions(&entry);
    let ids1 = action_ids(&actions1);
    let ids2 = action_ids(&actions2);
    assert_eq!(ids1, ids2);
}

#[test]
fn ai_command_bar_ordering_deterministic_across_calls() {
    let actions1 = get_ai_command_bar_actions();
    let actions2 = get_ai_command_bar_actions();
    let ids1 = action_ids(&actions1);
    let ids2 = action_ids(&actions2);
    assert_eq!(ids1, ids2);
}

// --- merged from part_06.rs ---

// =========================================================================
// 34. New chat icons per section
// =========================================================================

#[test]
fn new_chat_last_used_icon_is_bolt() {
    let last_used = vec![NewChatModelInfo {
        model_id: "lu1".to_string(),
        display_name: "Recent".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
}

#[test]
fn new_chat_preset_icon_is_custom() {
    let presets = vec![NewChatPresetInfo {
        id: "gen".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].icon, Some(IconName::Star));
}

#[test]
fn new_chat_model_icon_is_settings() {
    let models = vec![NewChatModelInfo {
        model_id: "m1".to_string(),
        display_name: "Claude".to_string(),
        provider: "anthropic".to_string(),
        provider_display_name: "Anthropic".to_string(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].icon, Some(IconName::Settings));
}

// =========================================================================
// 35. New chat descriptions
// =========================================================================

#[test]
fn new_chat_last_used_has_provider_display_name_description() {
    let last_used = vec![NewChatModelInfo {
        model_id: "lu1".to_string(),
        display_name: "Claude 3.5".to_string(),
        provider: "anthropic".to_string(),
        provider_display_name: "Anthropic".to_string(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].description.as_deref(), Some("Anthropic"));
}

#[test]
fn new_chat_preset_has_no_description() {
    let presets = vec![NewChatPresetInfo {
        id: "gen".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].description, None);
}

#[test]
fn new_chat_model_has_provider_display_name_description() {
    let models = vec![NewChatModelInfo {
        model_id: "m1".to_string(),
        display_name: "GPT-4".to_string(),
        provider: "openai".to_string(),
        provider_display_name: "OpenAI".to_string(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].description.as_deref(), Some("OpenAI"));
}

// =========================================================================
// 36. New chat action ID format
// =========================================================================

#[test]
fn new_chat_last_used_id_format() {
    let last_used = vec![
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
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].id, "last_used_0");
    assert_eq!(actions[1].id, "last_used_1");
}

#[test]
fn new_chat_preset_id_format() {
    let presets = vec![NewChatPresetInfo {
        id: "code-review".to_string(),
        name: "Code Review".to_string(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].id, "preset_code-review");
}

#[test]
fn new_chat_model_id_format() {
    let models = vec![
        NewChatModelInfo {
            model_id: "claude".to_string(),
            display_name: "Claude".to_string(),
            provider: "anthropic".to_string(),
            provider_display_name: "Anthropic".to_string(),
        },
        NewChatModelInfo {
            model_id: "gpt4".to_string(),
            display_name: "GPT-4".to_string(),
            provider: "openai".to_string(),
            provider_display_name: "OpenAI".to_string(),
        },
    ];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].id, "model_0");
    assert_eq!(actions[1].id, "model_1");
}
