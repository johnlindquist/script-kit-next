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
    assert!(ids.contains(&"reveal_in_finder"));
    assert!(ids.contains(&"copy_path"));
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
    assert!(!ids.contains(&"reveal_in_finder"));
}

#[test]
fn scriptlet_context_has_copy_scriptlet_path_not_copy_path() {
    let scriptlet = ScriptInfo::scriptlet("My Snippet", "/path/to/bundle.md", None, None);
    let actions = get_script_context_actions(&scriptlet);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"copy_scriptlet_path"));
    assert!(!ids.contains(&"copy_path"));
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
    assert!(!text_ids.contains(&"clipboard_ocr"));
    assert!(image_ids.contains(&"clipboard_ocr"));
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
    assert!(pinned_ids.contains(&"clipboard_unpin"));
    assert!(!pinned_ids.contains(&"clipboard_pin"));
    assert!(unpinned_ids.contains(&"clipboard_pin"));
    assert!(!unpinned_ids.contains(&"clipboard_unpin"));
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
            ids.contains(&"reveal_in_finder"),
            "FileType {:?} should have reveal_in_finder",
            info.file_type
        );
        assert!(
            ids.contains(&"copy_path"),
            "FileType {:?} should have copy_path",
            info.file_type
        );
        assert!(
            ids.contains(&"copy_filename"),
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
    assert!(action_ids(&file_actions).contains(&"open_file"));
    assert!(!action_ids(&file_actions).contains(&"open_directory"));
    assert!(action_ids(&dir_actions).contains(&"open_directory"));
    assert!(!action_ids(&dir_actions).contains(&"open_file"));
}
