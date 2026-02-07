//! Built-in action behavioral validation tests
//!
//! Validates randomly-selected built-in actions across window dialogs and
//! contexts to ensure invariants hold: ordering, value/has_action correctness,
//! section label consistency, description presence, icon assignment, and
//! cross-context guarantees.

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
use super::types::{Action, ActionCategory, ScriptInfo, SectionStyle};
use super::window::count_section_headers;
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

// =========================================================================
// 1. Primary action is always first across ALL script-like contexts
// =========================================================================

#[test]
fn run_script_always_first_for_basic_script() {
    let script = ScriptInfo::new("hello", "/path/hello.ts");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].id, "run_script");
    assert!(actions[0].title.starts_with("Run"));
}

#[test]
fn run_script_always_first_for_script_with_shortcut_alias_frecency() {
    let script = ScriptInfo::with_shortcut_and_alias(
        "full",
        "/path/full.ts",
        Some("cmd+f".into()),
        Some("fl".into()),
    )
    .with_frecency(true, Some("/path/full.ts".into()));
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn run_script_always_first_for_builtin() {
    let builtin = ScriptInfo::builtin("Clipboard History");
    let actions = get_script_context_actions(&builtin);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn run_script_always_first_for_scriptlet() {
    let scriptlet = ScriptInfo::scriptlet("Open URL", "/path/urls.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&scriptlet, None);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn run_script_always_first_for_agent() {
    let mut agent = ScriptInfo::new("my-agent", "/path/agent.md");
    agent.is_agent = true;
    agent.is_script = false;
    let actions = get_script_context_actions(&agent);
    assert_eq!(actions[0].id, "run_script");
}

// =========================================================================
// 2. Built-in actions never have has_action=true
// =========================================================================

#[test]
fn script_context_built_in_actions_have_has_action_false() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    for action in &get_script_context_actions(&script) {
        assert!(
            !action.has_action,
            "Built-in action '{}' should have has_action=false",
            action.id
        );
    }
}

#[test]
fn file_context_built_in_actions_have_has_action_false() {
    let file = FileInfo {
        path: "/f.txt".into(),
        name: "f.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    for action in &get_file_context_actions(&file) {
        assert!(
            !action.has_action,
            "File action '{}' should have has_action=false",
            action.id
        );
    }
}

#[test]
fn clipboard_context_built_in_actions_have_has_action_false() {
    let entry = ClipboardEntryInfo {
        id: "c1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    for action in &get_clipboard_history_context_actions(&entry) {
        assert!(
            !action.has_action,
            "Clipboard action '{}' should have has_action=false",
            action.id
        );
    }
}

#[test]
fn path_context_built_in_actions_have_has_action_false() {
    let path = PathInfo::new("test", "/test", false);
    for action in &get_path_context_actions(&path) {
        assert!(
            !action.has_action,
            "Path action '{}' should have has_action=false",
            action.id
        );
    }
}

#[test]
fn ai_command_bar_built_in_actions_have_has_action_false() {
    for action in &get_ai_command_bar_actions() {
        assert!(
            !action.has_action,
            "AI action '{}' should have has_action=false",
            action.id
        );
    }
}

#[test]
fn notes_command_bar_built_in_actions_have_has_action_false() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for action in &get_notes_command_bar_actions(&info) {
        assert!(
            !action.has_action,
            "Notes action '{}' should have has_action=false",
            action.id
        );
    }
}

// =========================================================================
// 3. Built-in actions have no value field (value is for SDK routing)
// =========================================================================

#[test]
fn script_context_built_in_actions_have_no_value() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    for action in &get_script_context_actions(&script) {
        assert!(
            action.value.is_none(),
            "Built-in action '{}' should have no value",
            action.id
        );
    }
}

#[test]
fn file_context_built_in_actions_have_no_value() {
    let file = FileInfo {
        path: "/f.txt".into(),
        name: "f.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    for action in &get_file_context_actions(&file) {
        assert!(
            action.value.is_none(),
            "File action '{}' should have no value",
            action.id
        );
    }
}

// =========================================================================
// 4. Scriptlet custom actions DO have has_action=true and value
// =========================================================================

#[test]
fn scriptlet_custom_actions_have_has_action_and_value() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo hi".into());
    scriptlet.actions.push(ScriptletAction {
        name: "Copy Output".into(),
        command: "copy-output".into(),
        tool: "bash".into(),
        code: "echo output | pbcopy".into(),
        inputs: vec![],
        shortcut: None,
        description: Some("Copy the output".into()),
    });
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let custom: Vec<&Action> = actions
        .iter()
        .filter(|a| a.id.starts_with("scriptlet_action:"))
        .collect();
    assert_eq!(custom.len(), 1);
    assert!(custom[0].has_action);
    assert!(custom[0].value.is_some());
}

#[test]
fn scriptlet_built_in_actions_still_have_no_has_action() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo hi".into());
    scriptlet.actions.push(ScriptletAction {
        name: "Custom".into(),
        command: "custom".into(),
        tool: "bash".into(),
        code: "echo".into(),
        inputs: vec![],
        shortcut: None,
        description: None,
    });
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let built_in: Vec<&Action> = actions
        .iter()
        .filter(|a| !a.id.starts_with("scriptlet_action:"))
        .collect();
    for action in &built_in {
        assert!(
            !action.has_action,
            "Built-in scriptlet action '{}' should have has_action=false",
            action.id
        );
    }
}

// =========================================================================
// 5. Destructive clipboard actions always appear last
// =========================================================================

#[test]
fn clipboard_destructive_actions_last_for_text_unpinned() {
    let entry = ClipboardEntryInfo {
        id: "e1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".into(),
        image_dimensions: None,
        frontmost_app_name: Some("Terminal".into()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions);
    let len = ids.len();
    assert_eq!(ids[len - 3], "clipboard_delete");
    assert_eq!(ids[len - 2], "clipboard_delete_multiple");
    assert_eq!(ids[len - 1], "clipboard_delete_all");
}

#[test]
fn clipboard_destructive_actions_last_for_image_pinned() {
    let entry = ClipboardEntryInfo {
        id: "e2".into(),
        content_type: ContentType::Image,
        pinned: true,
        preview: "".into(),
        image_dimensions: Some((640, 480)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions);
    let len = ids.len();
    assert_eq!(ids[len - 3], "clipboard_delete");
    assert_eq!(ids[len - 2], "clipboard_delete_multiple");
    assert_eq!(ids[len - 1], "clipboard_delete_all");
}

// =========================================================================
// 6. Section label consistency — no typos, same spelling across contexts
// =========================================================================

#[test]
fn ai_command_bar_section_labels_are_known() {
    let known = [
        "Response",
        "Actions",
        "Attachments",
        "Export",
        "Help",
        "Settings",
    ];
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        let section = action.section.as_deref().unwrap();
        assert!(
            known.contains(&section),
            "Unknown AI section label: '{}' in action '{}'",
            section,
            action.id
        );
    }
}

#[test]
fn notes_command_bar_section_labels_are_known() {
    let known = ["Notes", "Edit", "Copy", "Export", "Settings"];
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    for action in &actions {
        let section = action.section.as_deref().unwrap();
        assert!(
            known.contains(&section),
            "Unknown Notes section label: '{}' in action '{}'",
            section,
            action.id
        );
    }
}

#[test]
fn new_chat_section_labels_are_known() {
    let known = ["Last Used Settings", "Presets", "Models"];
    let last = vec![NewChatModelInfo {
        model_id: "m".into(),
        display_name: "M".into(),
        provider: "p".into(),
        provider_display_name: "P".into(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "p".into(),
        name: "P".into(),
        icon: IconName::Star,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "mo".into(),
        display_name: "MO".into(),
        provider: "mp".into(),
        provider_display_name: "MP".into(),
    }];
    let actions = get_new_chat_actions(&last, &presets, &models);
    for action in &actions {
        let section = action.section.as_deref().unwrap();
        assert!(
            known.contains(&section),
            "Unknown new chat section: '{}'",
            section
        );
    }
}

// =========================================================================
// 7. Action count stability — deterministic for same input
// =========================================================================

#[test]
fn script_context_action_count_is_deterministic() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let count1 = get_script_context_actions(&script).len();
    let count2 = get_script_context_actions(&script).len();
    let count3 = get_script_context_actions(&script).len();
    assert_eq!(count1, count2);
    assert_eq!(count2, count3);
}

#[test]
fn clipboard_action_count_is_deterministic() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let count1 = get_clipboard_history_context_actions(&entry).len();
    let count2 = get_clipboard_history_context_actions(&entry).len();
    assert_eq!(count1, count2);
}

#[test]
fn ai_command_bar_action_count_is_exactly_twelve() {
    assert_eq!(get_ai_command_bar_actions().len(), 12);
}

// =========================================================================
// 8. Enter shortcut on primary actions across contexts
// =========================================================================

#[test]
fn file_open_file_has_enter_shortcut() {
    let file = FileInfo {
        path: "/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn file_open_directory_has_enter_shortcut() {
    let dir = FileInfo {
        path: "/test".into(),
        name: "test".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&dir);
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn path_select_file_has_enter_shortcut() {
    let path = PathInfo::new("file.txt", "/file.txt", false);
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}
