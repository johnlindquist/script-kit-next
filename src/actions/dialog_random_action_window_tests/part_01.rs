//! Random action/window/dialog behavior validation tests
//!
//! Tests validate randomly-selected behaviors across action builders,
//! confirm dialog constants, window configs, and action property invariants.
//! Each test picks a specific scenario and validates expected behavior.

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
use super::types::{
    Action, ActionCategory, AnchorPosition, ScriptInfo, SearchPosition, SectionStyle,
};
use super::window::{count_section_headers, WindowPosition};
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;
use crate::protocol::ProtocolAction;
use crate::scriptlets::{Scriptlet, ScriptletAction};

// =========================================================================
// Helpers
// =========================================================================

fn action_ids(actions: &[Action]) -> Vec<&str> {
    actions.iter().map(|a| a.id.as_str()).collect()
}

fn find_action<'a>(actions: &'a [Action], id: &str) -> Option<&'a Action> {
    actions.iter().find(|a| a.id == id)
}

fn make_action(id: &str, title: &str, section: Option<&str>) -> Action {
    let mut a = Action::new(id, title, None, ActionCategory::ScriptContext);
    if let Some(s) = section {
        a = a.with_section(s);
    }
    a
}

// =========================================================================
// 1. Action shortcut uniqueness within each context
// =========================================================================

#[test]
fn script_context_shortcuts_are_unique() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let shortcuts: Vec<&str> = actions
        .iter()
        .filter_map(|a| a.shortcut.as_deref())
        .collect();
    let total = shortcuts.len();
    let mut deduped = shortcuts.clone();
    deduped.sort();
    deduped.dedup();
    assert_eq!(
        total,
        deduped.len(),
        "Duplicate shortcuts found in script context: {:?}",
        shortcuts
    );
}

#[test]
fn file_context_shortcuts_are_unique() {
    let file = FileInfo {
        path: "/f.txt".into(),
        name: "f.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let shortcuts: Vec<&str> = actions
        .iter()
        .filter_map(|a| a.shortcut.as_deref())
        .collect();
    let total = shortcuts.len();
    let mut deduped = shortcuts.clone();
    deduped.sort();
    deduped.dedup();
    assert_eq!(
        total,
        deduped.len(),
        "Duplicate shortcuts found in file context: {:?}",
        shortcuts
    );
}

#[test]
fn clipboard_context_shortcuts_are_unique() {
    let entry = ClipboardEntryInfo {
        id: "c".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "c".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let shortcuts: Vec<&str> = actions
        .iter()
        .filter_map(|a| a.shortcut.as_deref())
        .collect();
    let total = shortcuts.len();
    let mut deduped = shortcuts.clone();
    deduped.sort();
    deduped.dedup();
    assert_eq!(
        total,
        deduped.len(),
        "Duplicate shortcuts found in clipboard context: {:?}",
        shortcuts
    );
}

#[test]
fn ai_command_bar_shortcuts_are_unique() {
    let actions = get_ai_command_bar_actions();
    let shortcuts: Vec<&str> = actions
        .iter()
        .filter_map(|a| a.shortcut.as_deref())
        .collect();
    let total = shortcuts.len();
    let mut deduped = shortcuts.clone();
    deduped.sort();
    deduped.dedup();
    assert_eq!(
        total,
        deduped.len(),
        "Duplicate shortcuts found in AI command bar: {:?}",
        shortcuts
    );
}

// =========================================================================
// 2. Notes command bar sections have correct icons
// =========================================================================

#[test]
fn notes_command_bar_actions_have_icons() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // All notes command bar actions should have icons
    for action in &actions {
        assert!(
            action.icon.is_some(),
            "Notes action '{}' should have an icon",
            action.id
        );
    }
}

#[test]
fn ai_command_bar_actions_have_icons() {
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
fn ai_command_bar_actions_have_sections() {
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        assert!(
            action.section.is_some(),
            "AI action '{}' should have a section",
            action.id
        );
    }
}

// =========================================================================
// 3. Random script type permutations — shortcut+alias+frecency combos
// =========================================================================

#[test]
fn script_no_shortcut_no_alias_no_frecency() {
    let script = ScriptInfo::new("vanilla", "/path/vanilla.ts");
    let actions = get_script_context_actions(&script);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"add_shortcut"));
    assert!(ids.contains(&"add_alias"));
    assert!(!ids.contains(&"reset_ranking"));
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn script_has_shortcut_has_alias_has_frecency() {
    let script = ScriptInfo::with_shortcut_and_alias(
        "full",
        "/path/full.ts",
        Some("cmd+f".into()),
        Some("fu".into()),
    )
    .with_frecency(true, Some("/path/full.ts".into()));
    let actions = get_script_context_actions(&script);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"update_shortcut"));
    assert!(ids.contains(&"remove_shortcut"));
    assert!(ids.contains(&"update_alias"));
    assert!(ids.contains(&"remove_alias"));
    assert!(ids.contains(&"reset_ranking"));
    assert!(!ids.contains(&"add_shortcut"));
    assert!(!ids.contains(&"add_alias"));
}

#[test]
fn script_has_shortcut_no_alias_no_frecency() {
    let script = ScriptInfo::with_shortcut("shortcut-only", "/path/s.ts", Some("cmd+s".into()));
    let actions = get_script_context_actions(&script);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"update_shortcut"));
    assert!(ids.contains(&"remove_shortcut"));
    assert!(ids.contains(&"add_alias"));
    assert!(!ids.contains(&"add_shortcut"));
    assert!(!ids.contains(&"reset_ranking"));
}

#[test]
fn script_no_shortcut_has_alias_has_frecency() {
    let script =
        ScriptInfo::with_shortcut_and_alias("alias-frec", "/path/af.ts", None, Some("af".into()))
            .with_frecency(true, Some("/path/af.ts".into()));
    let actions = get_script_context_actions(&script);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"add_shortcut"));
    assert!(ids.contains(&"update_alias"));
    assert!(ids.contains(&"remove_alias"));
    assert!(ids.contains(&"reset_ranking"));
}

#[test]
fn builtin_has_shortcut_has_alias() {
    let builtin = ScriptInfo::with_all(
        "Test Builtin",
        "builtin:test",
        false,
        "Open",
        Some("cmd+b".into()),
        Some("tb".into()),
    );
    let actions = get_script_context_actions(&builtin);
    let ids = action_ids(&actions);
    // Builtins get run + shortcut/alias mgmt + deeplink
    assert!(ids.contains(&"run_script"));
    assert!(ids.contains(&"update_shortcut"));
    assert!(ids.contains(&"remove_shortcut"));
    assert!(ids.contains(&"update_alias"));
    assert!(ids.contains(&"remove_alias"));
    assert!(ids.contains(&"copy_deeplink"));
    // No script-only actions
    assert!(!ids.contains(&"edit_script"));
    assert!(!ids.contains(&"view_logs"));
}

#[test]
fn scriptlet_has_shortcut_no_alias_with_frecency() {
    let script = ScriptInfo::scriptlet("Test SL", "/path/test.md", Some("cmd+t".into()), None)
        .with_frecency(true, Some("scriptlet:Test SL".into()));
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"update_shortcut"));
    assert!(ids.contains(&"remove_shortcut"));
    assert!(ids.contains(&"add_alias"));
    assert!(ids.contains(&"reset_ranking"));
    assert!(!ids.contains(&"add_shortcut"));
}

// =========================================================================
// 4. Agent flag edge cases
// =========================================================================

#[test]
fn agent_no_shortcut_no_alias_not_suggested() {
    let mut agent = ScriptInfo::new("clean-agent", "/path/agent.md");
    agent.is_agent = true;
    agent.is_script = false;
    let actions = get_script_context_actions(&agent);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"add_shortcut"));
    assert!(ids.contains(&"add_alias"));
    assert!(!ids.contains(&"reset_ranking"));
    // Agent-specific
    let edit = find_action(&actions, "edit_script").unwrap();
    assert_eq!(edit.title, "Edit Agent");
}

#[test]
fn agent_frecency_adds_reset() {
    let mut agent = ScriptInfo::new("frec-agent", "/path/agent.md")
        .with_frecency(true, Some("agent:frec".into()));
    agent.is_agent = true;
    agent.is_script = false;
    let actions = get_script_context_actions(&agent);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"reset_ranking"));
}

// =========================================================================
// 5. Clipboard action random permutations
// =========================================================================

#[test]
fn clipboard_text_pinned_with_app_name() {
    let entry = ClipboardEntryInfo {
        id: "t1".into(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "pinned text".into(),
        image_dimensions: None,
        frontmost_app_name: Some("Safari".into()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions);
    // Pinned → unpin
    assert!(ids.contains(&"clipboard_unpin"));
    assert!(!ids.contains(&"clipboard_pin"));
    // App name in title
    let paste = find_action(&actions, "clipboard_paste").unwrap();
    assert_eq!(paste.title, "Paste to Safari");
    // Destructive still last
    let len = ids.len();
    assert_eq!(ids[len - 1], "clipboard_delete_all");
}

#[test]
fn clipboard_image_unpinned_no_app() {
    let entry = ClipboardEntryInfo {
        id: "i1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "".into(),
        image_dimensions: Some((1920, 1080)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"clipboard_pin"));
    assert!(!ids.contains(&"clipboard_unpin"));
    assert!(ids.contains(&"clipboard_ocr"));
    let paste = find_action(&actions, "clipboard_paste").unwrap();
    assert_eq!(paste.title, "Paste to Active App");
}

#[test]
fn clipboard_all_entries_have_core_actions() {
    for content_type in [ContentType::Text, ContentType::Image] {
        for pinned in [false, true] {
            for app in [None, Some("Terminal".to_string())] {
                let entry = ClipboardEntryInfo {
                    id: "any".into(),
                    content_type,
                    pinned,
                    preview: "any".into(),
                    image_dimensions: if content_type == ContentType::Image {
                        Some((100, 100))
                    } else {
                        None
                    },
                    frontmost_app_name: app.clone(),
                };
                let actions = get_clipboard_history_context_actions(&entry);
                let ids = action_ids(&actions);
                // Core actions always present
                assert!(
                    ids.contains(&"clipboard_paste"),
                    "Missing paste for {:?}/{}/{:?}",
                    content_type,
                    pinned,
                    app
                );
                assert!(
                    ids.contains(&"clipboard_copy"),
                    "Missing copy for {:?}/{}/{:?}",
                    content_type,
                    pinned,
                    app
                );
                assert!(
                    ids.contains(&"clipboard_delete"),
                    "Missing delete for {:?}/{}/{:?}",
                    content_type,
                    pinned,
                    app
                );
                assert!(
                    ids.contains(&"clipboard_delete_all"),
                    "Missing delete_all for {:?}/{}/{:?}",
                    content_type,
                    pinned,
                    app
                );
                // Pin/unpin mutually exclusive
                assert!(
                    ids.contains(&"clipboard_pin") ^ ids.contains(&"clipboard_unpin"),
                    "Pin/unpin should be mutually exclusive for {:?}/{}/{:?}",
                    content_type,
                    pinned,
                    app
                );
            }
        }
    }
}

// =========================================================================
// 6. Chat context — all flag combinations
// =========================================================================

#[test]
fn chat_single_model_current_checkmark() {
    let info = ChatPromptInfo {
        current_model: Some("Test Model".into()),
        available_models: vec![ChatModelInfo {
            id: "tm".into(),
            display_name: "Test Model".into(),
            provider: "TestCo".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model = find_action(&actions, "select_model_tm").unwrap();
    assert!(model.title.contains('✓'));
    assert_eq!(model.description.as_deref(), Some("via TestCo"));
}
