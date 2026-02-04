//! Built-in action behavioral validation tests — batch 3
//!
//! Validates randomly-selected built-in actions across window dialogs and
//! contexts that were NOT covered in batch 1 or batch 2. Focuses on:
//! - Shortcut uniqueness within each context (no duplicate hotkeys)
//! - Action ordering stability / determinism across repeated calls
//! - Cross-context action exclusivity (clipboard IDs never in file context etc.)
//! - with_shortcut_opt builder correctness
//! - Section ordering in AI, Notes, and New Chat command bars
//! - Scriptlet with multiple custom H3 actions: ordering and ID format
//! - Action title formatting with varied action_verbs
//! - Path context shortcut assignments completeness
//! - Clipboard ordering invariant (paste first, deletes last)
//! - Mixed flag combinations on ScriptInfo
//! - Note switcher icon hierarchy for all is_current × is_pinned combos
//! - to_deeplink_name with unicode / emoji edge cases
//! - Score stacking (title + description bonuses accumulate)
//! - File context primary title includes filename
//! - Scriptlet context action order: run > custom > shortcut > built-in > deeplink
//! - Chat model checkmark only on current model
//! - Notes conditional section counts across all 8 permutations
//! - CommandBarConfig notes_style specifics

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
// 1. Shortcut uniqueness within context — no two actions share a hotkey
// =========================================================================

#[test]
fn script_context_shortcuts_are_unique() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let mut seen = HashSet::new();
    for a in &actions {
        if let Some(ref s) = a.shortcut {
            assert!(
                seen.insert(s.as_str()),
                "Duplicate shortcut '{}' on action '{}' in script context",
                s,
                a.id
            );
        }
    }
}

#[test]
fn clipboard_context_text_shortcuts_are_unique() {
    let entry = ClipboardEntryInfo {
        id: "e1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let mut seen = HashSet::new();
    for a in &actions {
        if let Some(ref s) = a.shortcut {
            assert!(
                seen.insert(s.as_str()),
                "Duplicate shortcut '{}' on action '{}' in clipboard text context",
                s,
                a.id
            );
        }
    }
}

#[test]
fn file_context_shortcuts_are_unique() {
    let file = FileInfo {
        path: "/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let mut seen = HashSet::new();
    for a in &actions {
        if let Some(ref s) = a.shortcut {
            assert!(
                seen.insert(s.as_str()),
                "Duplicate shortcut '{}' on action '{}' in file context",
                s,
                a.id
            );
        }
    }
}

#[test]
fn path_context_shortcuts_are_unique() {
    let path = PathInfo {
        path: "/usr/local".into(),
        name: "local".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path);
    let mut seen = HashSet::new();
    for a in &actions {
        if let Some(ref s) = a.shortcut {
            assert!(
                seen.insert(s.as_str()),
                "Duplicate shortcut '{}' on action '{}' in path context",
                s,
                a.id
            );
        }
    }
}

#[test]
fn ai_command_bar_shortcuts_are_unique() {
    let actions = get_ai_command_bar_actions();
    let mut seen = HashSet::new();
    for a in &actions {
        if let Some(ref s) = a.shortcut {
            assert!(
                seen.insert(s.as_str()),
                "Duplicate shortcut '{}' on action '{}' in AI command bar",
                s,
                a.id
            );
        }
    }
}

// =========================================================================
// 2. Action ordering stability — same inputs always produce same output
// =========================================================================

#[test]
fn script_context_ordering_is_deterministic() {
    let script = ScriptInfo::with_shortcut_and_alias(
        "stable",
        "/path/stable.ts",
        Some("cmd+s".into()),
        Some("st".into()),
    )
    .with_frecency(true, Some("/path/stable.ts".into()));

    let a1 = get_script_context_actions(&script);
    let a2 = get_script_context_actions(&script);
    let a3 = get_script_context_actions(&script);
    let ids_1 = action_ids(&a1);
    let ids_2 = action_ids(&a2);
    let ids_3 = action_ids(&a3);

    assert_eq!(
        ids_1, ids_2,
        "Action ordering should be stable across calls"
    );
    assert_eq!(
        ids_2, ids_3,
        "Action ordering should be stable across calls"
    );
}

#[test]
fn clipboard_ordering_is_deterministic() {
    let entry = ClipboardEntryInfo {
        id: "det".into(),
        content_type: ContentType::Image,
        pinned: true,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: Some("Safari".into()),
    };
    let a1 = get_clipboard_history_context_actions(&entry);
    let a2 = get_clipboard_history_context_actions(&entry);
    let ids_1 = action_ids(&a1);
    let ids_2 = action_ids(&a2);
    assert_eq!(ids_1, ids_2);
}

#[test]
fn notes_command_bar_ordering_is_deterministic() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let a1 = get_notes_command_bar_actions(&info);
    let a2 = get_notes_command_bar_actions(&info);
    let ids_1 = action_ids(&a1);
    let ids_2 = action_ids(&a2);
    assert_eq!(ids_1, ids_2);
}

// =========================================================================
// 3. Cross-context action exclusivity
// =========================================================================

#[test]
fn clipboard_ids_never_appear_in_file_context() {
    let file = FileInfo {
        path: "/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let file_actions = get_file_context_actions(&file);
    let file_ids: HashSet<&str> = action_ids(&file_actions).into_iter().collect();

    // Clipboard-specific action IDs
    let clipboard_only = [
        "clipboard_paste",
        "clipboard_copy",
        "clipboard_pin",
        "clipboard_unpin",
        "clipboard_ocr",
        "clipboard_delete",
        "clipboard_delete_all",
        "clipboard_save_snippet",
        "clipboard_share",
        "clipboard_attach_to_ai",
    ];

    for id in &clipboard_only {
        assert!(
            !file_ids.contains(id),
            "File context should not contain clipboard action '{}'",
            id
        );
    }
}

#[test]
fn file_ids_never_appear_in_clipboard_context() {
    let entry = ClipboardEntryInfo {
        id: "c1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let clip_actions = get_clipboard_history_context_actions(&entry);
    let clip_ids: HashSet<&str> = action_ids(&clip_actions).into_iter().collect();

    let file_only = [
        "open_file",
        "open_directory",
        "open_with",
        "show_info",
        "copy_filename",
    ];

    for id in &file_only {
        assert!(
            !clip_ids.contains(id),
            "Clipboard context should not contain file action '{}'",
            id
        );
    }
}

#[test]
fn script_ids_never_appear_in_path_context() {
    let path = PathInfo {
        path: "/usr/bin".into(),
        name: "bin".into(),
        is_dir: true,
    };
    let path_actions = get_path_context_actions(&path);
    let path_ids: HashSet<&str> = action_ids(&path_actions).into_iter().collect();

    let script_only = [
        "run_script",
        "edit_script",
        "view_logs",
        "add_shortcut",
        "add_alias",
        "copy_deeplink",
        "reset_ranking",
    ];

    for id in &script_only {
        assert!(
            !path_ids.contains(id),
            "Path context should not contain script action '{}'",
            id
        );
    }
}

// =========================================================================
// 4. with_shortcut_opt builder correctness
// =========================================================================

#[test]
fn with_shortcut_opt_some_sets_shortcut() {
    let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
        .with_shortcut_opt(Some("⌘T".to_string()));
    assert_eq!(action.shortcut, Some("⌘T".to_string()));
    assert_eq!(action.shortcut_lower, Some("⌘t".to_string()));
}

#[test]
fn with_shortcut_opt_none_leaves_shortcut_none() {
    let action =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
    assert!(action.shortcut.is_none());
    assert!(action.shortcut_lower.is_none());
}

// =========================================================================
// 5. AI command bar section ordering: Response > Actions > Attachments > Settings
// =========================================================================

#[test]
fn ai_command_bar_section_order_is_correct() {
    let actions = get_ai_command_bar_actions();
    let sections = sections_in_order(&actions);
    assert_eq!(
        sections,
        vec![
            "Response",
            "Actions",
            "Attachments",
            "Export",
            "Actions",
            "Help",
            "Settings"
        ],
        "AI command bar sections should be in correct order"
    );
}

#[test]
fn ai_command_bar_response_section_has_three_actions() {
    let actions = get_ai_command_bar_actions();
    let response_count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .count();
    assert_eq!(response_count, 3, "Response section should have 3 actions");
}

#[test]
fn ai_command_bar_actions_section_has_four_actions() {
    let actions = get_ai_command_bar_actions();
    let actions_count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Actions"))
        .count();
    assert_eq!(actions_count, 4, "Actions section should have 4 actions");
}

#[test]
fn ai_command_bar_attachments_section_has_two_actions() {
    let actions = get_ai_command_bar_actions();
    let attachments_count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Attachments"))
        .count();
    assert_eq!(
        attachments_count, 2,
        "Attachments section should have 2 actions"
    );
}

#[test]
fn ai_command_bar_settings_section_has_one_action() {
    let actions = get_ai_command_bar_actions();
    let settings_count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Settings"))
        .count();
    assert_eq!(settings_count, 1, "Settings section should have 1 action");
}

// =========================================================================
// 6. Notes command bar section ordering: Notes > Edit > Copy > Export > Settings
// =========================================================================

#[test]
fn notes_command_bar_section_order_full() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let sections = sections_in_order(&actions);
    assert_eq!(
        sections,
        vec!["Notes", "Edit", "Copy", "Export", "Settings"],
        "Notes command bar sections should be in correct order"
    );
}

#[test]
fn notes_command_bar_section_order_no_selection() {
    // Without selection, only Notes and Settings sections should appear
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let sections = sections_in_order(&actions);
    assert_eq!(
        sections,
        vec!["Notes", "Settings"],
        "Notes without selection should only have Notes and Settings"
    );
}

#[test]
fn notes_command_bar_section_order_trash_view() {
    // In trash view, even with selection, only Notes appears (plus Settings if not auto-sizing)
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let sections = sections_in_order(&actions);
    assert_eq!(
        sections,
        vec!["Notes", "Settings"],
        "Notes in trash view should only have Notes and Settings"
    );
}

#[test]
fn notes_command_bar_auto_sizing_enabled_hides_settings() {
    // With auto-sizing already enabled, Settings section should be absent
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let sections = sections_in_order(&actions);
    assert_eq!(
        sections,
        vec!["Notes"],
        "With auto-sizing on and no selection, only Notes section"
    );
}

// =========================================================================
// 7. New chat section ordering: Last Used Settings > Presets > Models
// =========================================================================

#[test]
fn new_chat_section_order_all_populated() {
    let last_used = vec![NewChatModelInfo {
        model_id: "claude-3".into(),
        display_name: "Claude 3".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Settings,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "gpt-4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &presets, &models);
    let sections = sections_in_order(&actions);
    assert_eq!(sections, vec!["Last Used Settings", "Presets", "Models"],);
}

#[test]
fn new_chat_section_order_no_last_used() {
    let presets = vec![NewChatPresetInfo {
        id: "code".into(),
        name: "Code".into(),
        icon: IconName::Code,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "gpt-4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&[], &presets, &models);
    let sections = sections_in_order(&actions);
    assert_eq!(sections, vec!["Presets", "Models"]);
}

#[test]
fn new_chat_all_empty_returns_no_actions() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

// =========================================================================
// 8. Scriptlet with multiple custom H3 actions
// =========================================================================

#[test]
fn scriptlet_custom_actions_maintain_order() {
    let script = ScriptInfo::scriptlet("Multi Action", "/path/multi.md", None, None);
    let mut scriptlet = Scriptlet::new(
        "Multi Action".to_string(),
        "bash".to_string(),
        "echo main".to_string(),
    );
    scriptlet.actions = vec![
        ScriptletAction {
            name: "Alpha".to_string(),
            command: "alpha-cmd".to_string(),
            tool: "bash".to_string(),
            code: "echo alpha".to_string(),
            inputs: vec![],
            shortcut: Some("cmd+1".to_string()),
            description: Some("First action".to_string()),
        },
        ScriptletAction {
            name: "Beta".to_string(),
            command: "beta-cmd".to_string(),
            tool: "bash".to_string(),
            code: "echo beta".to_string(),
            inputs: vec![],
            shortcut: Some("cmd+2".to_string()),
            description: Some("Second action".to_string()),
        },
        ScriptletAction {
            name: "Gamma".to_string(),
            command: "gamma-cmd".to_string(),
            tool: "bash".to_string(),
            code: "echo gamma".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
    ];

    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let ids = action_ids(&actions);

    // run_script must be first
    assert_eq!(ids[0], "run_script");

    // Custom actions follow run in declaration order
    let alpha_idx = ids
        .iter()
        .position(|id| *id == "scriptlet_action:alpha-cmd")
        .unwrap();
    let beta_idx = ids
        .iter()
        .position(|id| *id == "scriptlet_action:beta-cmd")
        .unwrap();
    let gamma_idx = ids
        .iter()
        .position(|id| *id == "scriptlet_action:gamma-cmd")
        .unwrap();

    assert_eq!(alpha_idx, 1);
    assert_eq!(beta_idx, 2);
    assert_eq!(gamma_idx, 3);

    // Custom actions all have has_action=true
    for id in &[
        "scriptlet_action:alpha-cmd",
        "scriptlet_action:beta-cmd",
        "scriptlet_action:gamma-cmd",
    ] {
        let a = find_action(&actions, id).unwrap();
        assert!(
            a.has_action,
            "Custom action '{}' should have has_action=true",
            id
        );
        assert!(
            a.value.is_some(),
            "Custom action '{}' should have a value",
            id
        );
    }
}

#[test]
fn scriptlet_custom_action_id_format() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut scriptlet = Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Do Something".to_string(),
        command: "do-something".to_string(),
        tool: "bash".to_string(),
        code: "echo do".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];

    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let custom = find_action(&actions, "scriptlet_action:do-something").unwrap();
    assert!(custom.id.starts_with("scriptlet_action:"));
    assert_eq!(custom.title, "Do Something");
}

// =========================================================================
// 9. Action title formatting with varied action_verbs
// =========================================================================

#[test]
fn action_verb_appears_in_primary_title() {
    let verbs = ["Run", "Launch", "Switch to", "Open", "Execute"];
    for verb in &verbs {
        let script = ScriptInfo::with_action_verb("MyItem", "/path/item", false, *verb);
        let actions = get_script_context_actions(&script);
        let primary = &actions[0];
        assert!(
            primary.title.starts_with(verb),
            "Primary action title '{}' should start with verb '{}'",
            primary.title,
            verb
        );
        assert!(
            primary.title.contains("MyItem"),
            "Primary action title '{}' should contain the item name",
            primary.title
        );
    }
}

#[test]
fn scriptlet_primary_uses_action_verb() {
    let script = ScriptInfo::scriptlet("Open URL", "/path/url.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let primary = &actions[0];
    assert!(
        primary.title.starts_with("Run"),
        "Scriptlet primary should use 'Run' verb"
    );
    assert!(primary.title.contains("Open URL"));
}

// =========================================================================
// 10. Path context shortcut assignments
// =========================================================================

#[test]
fn path_file_has_enter_on_primary() {
    let path = PathInfo {
        path: "/usr/file.txt".into(),
        name: "file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].id, "select_file");
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn path_dir_has_enter_on_primary() {
    let path = PathInfo {
        path: "/usr/local".into(),
        name: "local".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].id, "open_directory");
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn path_context_has_trash_shortcut() {
    let path = PathInfo {
        path: "/tmp/file.txt".into(),
        name: "file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    let trash = find_action(&actions, "move_to_trash").unwrap();
    assert_eq!(trash.shortcut.as_deref(), Some("⌘⌫"));
}

#[test]
fn path_context_has_all_expected_actions() {
    let path = PathInfo {
        path: "/tmp/file.txt".into(),
        name: "file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();

    let expected = [
        "select_file",
        "copy_path",
        "open_in_finder",
        "open_in_editor",
        "open_in_terminal",
        "copy_filename",
        "move_to_trash",
    ];
    for id in &expected {
        assert!(
            ids.contains(id),
            "Path file context should have action '{}'",
            id
        );
    }
}

#[test]
fn path_dir_context_has_open_directory_not_select_file() {
    let path = PathInfo {
        path: "/usr/local".into(),
        name: "local".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path);
    let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();
    assert!(ids.contains("open_directory"));
    assert!(!ids.contains("select_file"));
}

// =========================================================================
// 11. Clipboard ordering invariant: paste first, deletes last
// =========================================================================

#[test]
fn clipboard_paste_always_first_text() {
    let entry = ClipboardEntryInfo {
        id: "t1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[0].id, "clipboard_paste");
}

#[test]
fn clipboard_paste_always_first_image() {
    let entry = ClipboardEntryInfo {
        id: "i1".into(),
        content_type: ContentType::Image,
        pinned: true,
        preview: "img".into(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: Some("Figma".into()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[0].id, "clipboard_paste");
}

#[test]
fn clipboard_delete_actions_always_last_three() {
    let entry = ClipboardEntryInfo {
        id: "d1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let len = actions.len();
    assert!(len >= 3);

    let last_three_ids: Vec<&str> = actions[len - 3..].iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        last_three_ids,
        vec![
            "clipboard_delete",
            "clipboard_delete_multiple",
            "clipboard_delete_all"
        ],
        "Last 3 clipboard actions should be the destructive ones in order"
    );
}

#[test]
fn clipboard_delete_actions_always_last_three_image() {
    let entry = ClipboardEntryInfo {
        id: "di".into(),
        content_type: ContentType::Image,
        pinned: true,
        preview: "img".into(),
        image_dimensions: Some((1920, 1080)),
        frontmost_app_name: Some("Preview".into()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let len = actions.len();

    let last_three_ids: Vec<&str> = actions[len - 3..].iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        last_three_ids,
        vec![
            "clipboard_delete",
            "clipboard_delete_multiple",
            "clipboard_delete_all"
        ],
    );
}

// =========================================================================
// 12. Mixed flag combinations on ScriptInfo
// =========================================================================

#[test]
fn script_with_both_shortcut_and_alias_has_update_remove_for_both() {
    let script = ScriptInfo::with_shortcut_and_alias(
        "full",
        "/path/full.ts",
        Some("cmd+f".into()),
        Some("fl".into()),
    );
    let actions = get_script_context_actions(&script);
    let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();

    assert!(ids.contains("update_shortcut"));
    assert!(ids.contains("remove_shortcut"));
    assert!(!ids.contains("add_shortcut"));
    assert!(ids.contains("update_alias"));
    assert!(ids.contains("remove_alias"));
    assert!(!ids.contains("add_alias"));
}

#[test]
fn builtin_with_frecency_has_reset_ranking_and_no_edit() {
    let builtin = ScriptInfo::builtin("Clipboard History")
        .with_frecency(true, Some("builtin:clipboard".into()));
    let actions = get_script_context_actions(&builtin);
    let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();

    assert!(ids.contains("reset_ranking"));
    assert!(ids.contains("run_script"));
    assert!(ids.contains("copy_deeplink"));
    assert!(!ids.contains("edit_script"));
    assert!(!ids.contains("view_logs"));
}

#[test]
fn agent_with_shortcut_shows_update_not_add() {
    let mut agent = ScriptInfo::new("my-agent", "/path/agent.md");
    agent.is_agent = true;
    agent.is_script = false;
    agent.shortcut = Some("cmd+a".into());

    let actions = get_script_context_actions(&agent);
    let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();
    assert!(ids.contains("update_shortcut"));
    assert!(!ids.contains("add_shortcut"));
    assert!(ids.contains("edit_script")); // agent gets edit_script with title "Edit Agent"
}

// =========================================================================
// 13. Note switcher icon hierarchy for all is_current × is_pinned combos
// =========================================================================

#[test]
fn note_switcher_pinned_current_gets_star_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n1".into(),
        title: "Note 1".into(),
        char_count: 100,
        is_current: true,
        is_pinned: true,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn note_switcher_pinned_not_current_gets_star_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n2".into(),
        title: "Note 2".into(),
        char_count: 50,
        is_current: false,
        is_pinned: true,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn note_switcher_current_not_pinned_gets_check_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n3".into(),
        title: "Note 3".into(),
        char_count: 25,
        is_current: true,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::Check));
}

#[test]
fn note_switcher_neither_pinned_nor_current_gets_file_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n4".into(),
        title: "Note 4".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::File));
}

#[test]
fn note_switcher_current_note_has_bullet_prefix() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "a".into(),
            title: "Current Note".into(),
            char_count: 100,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "b".into(),
            title: "Other Note".into(),
            char_count: 50,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    assert!(
        actions[0].title.starts_with("• "),
        "Current note should have bullet prefix"
    );
    assert!(
        !actions[1].title.starts_with("• "),
        "Non-current note should not have bullet prefix"
    );
}

#[test]
fn note_switcher_char_count_plural() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "p0".into(),
            title: "Zero".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "p1".into(),
            title: "One".into(),
            char_count: 1,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "p2".into(),
            title: "Many".into(),
            char_count: 42,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("0 chars"));
    assert_eq!(actions[1].description.as_deref(), Some("1 char"));
    assert_eq!(actions[2].description.as_deref(), Some("42 chars"));
}

#[test]
fn note_switcher_empty_shows_no_notes_message() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "no_notes");
    assert_eq!(actions[0].title, "No notes yet");
    assert_eq!(actions[0].icon, Some(IconName::Plus));
}

// =========================================================================
// 14. to_deeplink_name with unicode / emoji edge cases
// =========================================================================

#[test]
fn deeplink_name_with_accented_chars() {
    // to_deeplink_name lowercases and replaces non-alphanumeric with hyphens,
    // but accented latin chars like 'é' are alphanumeric in Unicode
    assert_eq!(to_deeplink_name("café"), "café");
}

#[test]
fn deeplink_name_with_numbers() {
    assert_eq!(to_deeplink_name("Script123"), "script123");
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
fn deeplink_name_leading_trailing_spaces() {
    assert_eq!(to_deeplink_name("  hello  "), "hello");
}

#[test]
fn deeplink_name_consecutive_hyphens_collapsed() {
    assert_eq!(to_deeplink_name("a---b"), "a-b");
}

#[test]
fn deeplink_name_mixed_case_numbers_symbols() {
    assert_eq!(to_deeplink_name("My Script (v2.0)"), "my-script-v2-0");
}

// =========================================================================
// 15. Score stacking — title + description bonuses accumulate
// =========================================================================

#[test]
fn score_prefix_match_is_100() {
    let action = Action::new(
        "edit_script",
        "Edit Script",
        Some("Open in editor".to_string()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "edit");
    assert_eq!(
        score,
        100 + 15,
        "Prefix 'edit' should get 100 for title + 15 for description containing 'edit'"
    );
}

#[test]
fn score_contains_match_is_50() {
    let action = Action::new(
        "copy_path",
        "Copy Path",
        Some("Copy to clipboard".to_string()),
        ActionCategory::ScriptContext,
    );
    // "path" is contained but not a prefix
    let score = ActionsDialog::score_action(&action, "path");
    assert!(
        score >= 50,
        "Contains match should be at least 50, got {}",
        score
    );
}

#[test]
fn score_description_only_match() {
    let action = Action::new(
        "open_file",
        "Open File",
        Some("Launch with default application".to_string()),
        ActionCategory::ScriptContext,
    );
    // "launch" is in description but not title
    let score = ActionsDialog::score_action(&action, "launch");
    assert_eq!(score, 15, "'launch' only in description should give 15");
}

#[test]
fn score_shortcut_only_match() {
    let action = Action::new(
        "edit_script",
        "Edit Script",
        Some("Open in editor".to_string()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘E");
    // "⌘e" matches shortcut but not title or description
    let score = ActionsDialog::score_action(&action, "⌘e");
    assert!(
        score >= 10,
        "Shortcut match should give at least 10, got {}",
        score
    );
}

#[test]
fn score_no_match_is_zero() {
    let action = Action::new(
        "run_script",
        "Run Script",
        Some("Execute this item".to_string()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "zzzzz");
    assert_eq!(score, 0, "No match should score 0");
}

#[test]
fn score_prefix_plus_description_stack() {
    let action = Action::new(
        "copy_path",
        "Copy Path",
        Some("Copy the full path to clipboard".to_string()),
        ActionCategory::ScriptContext,
    );
    // "copy" is a prefix of title AND contained in description
    let score = ActionsDialog::score_action(&action, "copy");
    assert_eq!(
        score,
        100 + 15,
        "Prefix + description match should stack: 100 + 15 = 115, got {}",
        score
    );
}

// =========================================================================
// 16. File context primary title includes filename
// =========================================================================

#[test]
fn file_context_primary_title_includes_filename() {
    let file = FileInfo {
        path: "/Users/test/document.pdf".into(),
        name: "document.pdf".into(),
        file_type: FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    assert!(
        actions[0].title.contains("document.pdf"),
        "File primary title '{}' should include filename",
        actions[0].title
    );
}

#[test]
fn file_context_dir_primary_title_includes_dirname() {
    let file = FileInfo {
        path: "/Users/test/Documents".into(),
        name: "Documents".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&file);
    assert!(
        actions[0].title.contains("Documents"),
        "Dir primary title '{}' should include dirname",
        actions[0].title
    );
}

// =========================================================================
// 17. Chat model checkmark only on current model
// =========================================================================

#[test]
fn chat_model_checkmark_on_current_only() {
    let info = ChatPromptInfo {
        current_model: Some("Claude 3".to_string()),
        available_models: vec![
            ChatModelInfo {
                id: "claude-3".into(),
                display_name: "Claude 3".into(),
                provider: "Anthropic".into(),
            },
            ChatModelInfo {
                id: "gpt-4".into(),
                display_name: "GPT-4".into(),
                provider: "OpenAI".into(),
            },
            ChatModelInfo {
                id: "gemini".into(),
                display_name: "Gemini".into(),
                provider: "Google".into(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);

    // Claude 3 should have checkmark
    let claude = find_action(&actions, "select_model_claude-3").unwrap();
    assert!(
        claude.title.contains('✓'),
        "Current model should have checkmark"
    );

    // Others should not
    let gpt = find_action(&actions, "select_model_gpt-4").unwrap();
    assert!(
        !gpt.title.contains('✓'),
        "Non-current model should not have checkmark"
    );

    let gemini = find_action(&actions, "select_model_gemini").unwrap();
    assert!(
        !gemini.title.contains('✓'),
        "Non-current model should not have checkmark"
    );
}

#[test]
fn chat_no_current_model_no_checkmarks() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![
            ChatModelInfo {
                id: "m1".into(),
                display_name: "Model 1".into(),
                provider: "P1".into(),
            },
            ChatModelInfo {
                id: "m2".into(),
                display_name: "Model 2".into(),
                provider: "P2".into(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    for a in &actions {
        if a.id.starts_with("select_model_") {
            assert!(
                !a.title.contains('✓'),
                "No model should have checkmark when current_model is None"
            );
        }
    }
}

#[test]
fn chat_continue_in_chat_always_present() {
    // Even with no models, continue_in_chat should be present
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(
        actions.iter().any(|a| a.id == "continue_in_chat"),
        "continue_in_chat should always be present"
    );
}

#[test]
fn chat_copy_response_only_with_response() {
    let without = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions_without = get_chat_context_actions(&without);
    assert!(
        !actions_without.iter().any(|a| a.id == "copy_response"),
        "copy_response should be absent without response"
    );

    let with = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: true,
    };
    let actions_with = get_chat_context_actions(&with);
    assert!(
        actions_with.iter().any(|a| a.id == "copy_response"),
        "copy_response should be present with response"
    );
}

#[test]
fn chat_clear_conversation_only_with_messages() {
    let without = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions_without = get_chat_context_actions(&without);
    assert!(!actions_without.iter().any(|a| a.id == "clear_conversation"),);

    let with = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions_with = get_chat_context_actions(&with);
    assert!(actions_with.iter().any(|a| a.id == "clear_conversation"));
}

// =========================================================================
// 18. Notes conditional action counts across all 8 permutations
//     (has_selection × is_trash × auto_sizing)
// =========================================================================

#[test]
fn notes_8_permutations_action_counts() {
    let bools = [false, true];
    for &sel in &bools {
        for &trash in &bools {
            for &auto in &bools {
                let info = NotesInfo {
                    has_selection: sel,
                    is_trash_view: trash,
                    auto_sizing_enabled: auto,
                };
                let actions = get_notes_command_bar_actions(&info);

                // new_note and browse_notes always present
                assert!(
                    actions.iter().any(|a| a.id == "new_note"),
                    "new_note always present (sel={}, trash={}, auto={})",
                    sel,
                    trash,
                    auto
                );
                assert!(
                    actions.iter().any(|a| a.id == "browse_notes"),
                    "browse_notes always present (sel={}, trash={}, auto={})",
                    sel,
                    trash,
                    auto
                );

                // Conditional: duplicate, find, format, copy, export
                // only when has_selection && !is_trash_view
                let has_conditionals = sel && !trash;
                let conditional_ids = [
                    "duplicate_note",
                    "find_in_note",
                    "format",
                    "copy_note_as",
                    "copy_deeplink",
                    "create_quicklink",
                    "export",
                ];
                for id in &conditional_ids {
                    assert_eq!(
                        actions.iter().any(|a| a.id == *id),
                        has_conditionals,
                        "Action '{}' should {} when sel={}, trash={}, auto={}",
                        id,
                        if has_conditionals {
                            "be present"
                        } else {
                            "be absent"
                        },
                        sel,
                        trash,
                        auto
                    );
                }

                // enable_auto_sizing only when auto_sizing_enabled is false
                assert_eq!(
                    actions.iter().any(|a| a.id == "enable_auto_sizing"),
                    !auto,
                    "enable_auto_sizing should {} when auto={}",
                    if !auto { "be present" } else { "be absent" },
                    auto
                );
            }
        }
    }
}

// =========================================================================
// 19. CommandBarConfig notes_style specifics
// =========================================================================

#[test]
fn command_bar_notes_style_search_top_separators_icons() {
    let config = CommandBarConfig::notes_style();
    assert!(
        matches!(config.dialog_config.search_position, SearchPosition::Top),
        "notes_style should have search at top"
    );
    assert!(
        matches!(config.dialog_config.section_style, SectionStyle::Separators),
        "notes_style should use Separators"
    );
    assert!(
        config.dialog_config.show_icons,
        "notes_style should show icons"
    );
    assert!(
        config.dialog_config.show_footer,
        "notes_style should show footer"
    );
    assert!(config.close_on_escape);
    assert!(config.close_on_select);
    assert!(config.close_on_click_outside);
}

// =========================================================================
// 20. Grouped items build correctness
// =========================================================================

#[test]
fn grouped_items_headers_style_produces_section_headers() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);

    // Should contain at least one SectionHeader
    let header_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
        .count();
    assert!(
        header_count >= 7,
        "Headers style should produce at least 7 section headers, got {}",
        header_count
    );
}

#[test]
fn grouped_items_none_style_has_no_headers() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);

    let header_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(
        header_count, 0,
        "None style should produce no section headers"
    );
}

#[test]
fn grouped_items_separators_style_has_no_headers() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);

    let header_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(
        header_count, 0,
        "Separators style should produce no section headers"
    );
}

#[test]
fn grouped_items_empty_filtered_returns_empty() {
    let actions = get_ai_command_bar_actions();
    let grouped = build_grouped_items_static(&actions, &[], SectionStyle::Headers);
    assert!(grouped.is_empty());
}

// =========================================================================
// 21. Coerce action selection correctness
// =========================================================================

#[test]
fn coerce_selection_on_item_returns_same_index() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::Item(1),
        GroupedActionItem::Item(2),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(1));
}

#[test]
fn coerce_selection_on_header_skips_to_next_item() {
    let rows = vec![
        GroupedActionItem::SectionHeader("Section".into()),
        GroupedActionItem::Item(0),
        GroupedActionItem::Item(1),
    ];
    // Index 0 is a header, should coerce to index 1
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn coerce_selection_on_trailing_header_goes_up() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("Section".into()),
    ];
    // Index 1 is a header at the end, should coerce back to index 0
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn coerce_selection_empty_returns_none() {
    assert_eq!(coerce_action_selection(&[], 0), None);
}

#[test]
fn coerce_selection_all_headers_returns_none() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".into()),
        GroupedActionItem::SectionHeader("B".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

// =========================================================================
// 22. Action cached lowercase fields consistency
// =========================================================================

#[test]
fn action_title_lower_matches_title() {
    let action = Action::new(
        "test",
        "My Title With CAPS",
        Some("Description HERE".to_string()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘⇧C");

    assert_eq!(action.title_lower, "my title with caps");
    assert_eq!(
        action.description_lower,
        Some("description here".to_string())
    );
    assert_eq!(action.shortcut_lower, Some("⌘⇧c".to_string()));
}

#[test]
fn all_script_actions_have_consistent_lowercase_caches() {
    let script = ScriptInfo::new("Test Script", "/path/test.ts");
    for a in &get_script_context_actions(&script) {
        assert_eq!(
            a.title_lower,
            a.title.to_lowercase(),
            "title_lower mismatch for '{}'",
            a.id
        );
        if let Some(ref desc) = a.description {
            assert_eq!(
                a.description_lower.as_deref(),
                Some(desc.to_lowercase()).as_deref(),
                "description_lower mismatch for '{}'",
                a.id
            );
        }
        if let Some(ref sc) = a.shortcut {
            assert_eq!(
                a.shortcut_lower.as_deref(),
                Some(sc.to_lowercase()).as_deref(),
                "shortcut_lower mismatch for '{}'",
                a.id
            );
        }
    }
}

#[test]
fn all_clipboard_actions_have_consistent_lowercase_caches() {
    let entry = ClipboardEntryInfo {
        id: "lc".into(),
        content_type: ContentType::Image,
        pinned: true,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: Some("Arc".into()),
    };
    for a in &get_clipboard_history_context_actions(&entry) {
        assert_eq!(
            a.title_lower,
            a.title.to_lowercase(),
            "title_lower mismatch for '{}'",
            a.id
        );
    }
}

#[test]
fn all_ai_command_bar_actions_have_consistent_lowercase_caches() {
    for a in &get_ai_command_bar_actions() {
        assert_eq!(a.title_lower, a.title.to_lowercase());
        if let Some(ref desc) = a.description {
            assert_eq!(
                a.description_lower.as_deref(),
                Some(desc.to_lowercase()).as_deref()
            );
        }
    }
}

// =========================================================================
// 23. New chat action descriptions
// =========================================================================

#[test]
fn new_chat_last_used_has_provider_description() {
    let last_used = vec![NewChatModelInfo {
        model_id: "claude-3".into(),
        display_name: "Claude 3".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    let a = &actions[0];
    assert_eq!(a.description.as_deref(), Some("Anthropic"));
}

#[test]
fn new_chat_presets_have_no_description() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Settings,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    let a = &actions[0];
    assert!(
        a.description.is_none(),
        "Presets should have no description"
    );
}

#[test]
fn new_chat_models_have_provider_description() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt-4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    let a = &actions[0];
    assert_eq!(a.description.as_deref(), Some("OpenAI"));
}

// =========================================================================
// 24. New chat action ID format
// =========================================================================

#[test]
fn new_chat_last_used_ids_are_indexed() {
    let last_used = vec![
        NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "M1".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        },
        NewChatModelInfo {
            model_id: "m2".into(),
            display_name: "M2".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        },
    ];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].id, "last_used_0");
    assert_eq!(actions[1].id, "last_used_1");
}

#[test]
fn new_chat_preset_ids_use_preset_id() {
    let presets = vec![
        NewChatPresetInfo {
            id: "gen".into(),
            name: "General".into(),
            icon: IconName::Settings,
        },
        NewChatPresetInfo {
            id: "code".into(),
            name: "Code".into(),
            icon: IconName::Code,
        },
    ];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].id, "preset_gen");
    assert_eq!(actions[1].id, "preset_code");
}

#[test]
fn new_chat_model_ids_are_indexed() {
    let models = vec![
        NewChatModelInfo {
            model_id: "claude".into(),
            display_name: "Claude".into(),
            provider: "a".into(),
            provider_display_name: "Anthropic".into(),
        },
        NewChatModelInfo {
            model_id: "gpt".into(),
            display_name: "GPT".into(),
            provider: "o".into(),
            provider_display_name: "OpenAI".into(),
        },
    ];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].id, "model_0");
    assert_eq!(actions[1].id, "model_1");
}

// =========================================================================
// 25. All AI command bar actions have icon and section
// =========================================================================

#[test]
fn ai_command_bar_all_have_icon() {
    for a in &get_ai_command_bar_actions() {
        assert!(a.icon.is_some(), "AI action '{}' should have an icon", a.id);
    }
}

#[test]
fn ai_command_bar_all_have_section() {
    for a in &get_ai_command_bar_actions() {
        assert!(
            a.section.is_some(),
            "AI action '{}' should have a section",
            a.id
        );
    }
}

// =========================================================================
// 26. Notes command bar conditional icons
// =========================================================================

#[test]
fn notes_command_bar_all_have_icons_when_full() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for a in &get_notes_command_bar_actions(&info) {
        assert!(
            a.icon.is_some(),
            "Notes action '{}' should have an icon",
            a.id
        );
    }
}

#[test]
fn notes_command_bar_all_have_sections_when_full() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for a in &get_notes_command_bar_actions(&info) {
        assert!(
            a.section.is_some(),
            "Notes action '{}' should have a section",
            a.id
        );
    }
}

// =========================================================================
// 27. Clipboard attach_to_ai action present
// =========================================================================

#[test]
fn clipboard_text_has_attach_to_ai() {
    let entry = ClipboardEntryInfo {
        id: "ai".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_attach_to_ai"));
}

#[test]
fn clipboard_image_has_attach_to_ai() {
    let entry = ClipboardEntryInfo {
        id: "ai2".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_attach_to_ai"));
}

// =========================================================================
// 28. Scriptlet context built-in action set
// =========================================================================

#[test]
fn scriptlet_context_has_expected_builtin_ids() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();

    let expected = [
        "run_script",
        "add_shortcut",
        "add_alias",
        "edit_scriptlet",
        "reveal_scriptlet_in_finder",
        "copy_scriptlet_path",
        "copy_content",
        "copy_deeplink",
    ];
    for id in &expected {
        assert!(ids.contains(id), "Scriptlet context should have '{}'", id);
    }
}

#[test]
fn scriptlet_context_action_order_run_before_builtin() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let ids = action_ids(&actions);

    let run_idx = ids.iter().position(|id| *id == "run_script").unwrap();
    let edit_idx = ids.iter().position(|id| *id == "edit_scriptlet").unwrap();
    let deeplink_idx = ids.iter().position(|id| *id == "copy_deeplink").unwrap();

    assert!(run_idx < edit_idx, "run should come before edit_scriptlet");
    assert!(
        edit_idx < deeplink_idx,
        "edit_scriptlet should come before copy_deeplink"
    );
}

// =========================================================================
// 29. Path context trash description varies by is_dir
// =========================================================================

#[test]
fn path_trash_description_says_file_for_file() {
    let path = PathInfo {
        path: "/tmp/file.txt".into(),
        name: "file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    let trash = find_action(&actions, "move_to_trash").unwrap();
    assert_eq!(trash.description.as_deref(), Some("Delete file"),);
}

#[test]
fn path_trash_description_says_folder_for_dir() {
    let path = PathInfo {
        path: "/tmp/mydir".into(),
        name: "mydir".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path);
    let trash = find_action(&actions, "move_to_trash").unwrap();
    assert_eq!(trash.description.as_deref(), Some("Delete folder"),);
}

// =========================================================================
// 30. Note switcher all notes have "Notes" section
// =========================================================================

#[test]
fn note_switcher_all_actions_have_notes_section() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "a".into(),
            title: "Note A".into(),
            char_count: 10,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "b".into(),
            title: "Note B".into(),
            char_count: 20,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    for a in &actions {
        let section = a.section.as_deref();
        assert!(
            section == Some("Pinned") || section == Some("Recent"),
            "Note switcher action '{}' should have 'Pinned' or 'Recent' section, got {:?}",
            a.id,
            section
        );
    }
}

#[test]
fn note_switcher_empty_state_has_notes_section() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions[0].section.as_deref(), Some("Notes"));
}

// =========================================================================
// 31. New chat action icons
// =========================================================================

#[test]
fn new_chat_last_used_has_bolt_icon() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "M1".into(),
        provider: "p".into(),
        provider_display_name: "P".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
}

#[test]
fn new_chat_models_have_settings_icon() {
    let models = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "M1".into(),
        provider: "p".into(),
        provider_display_name: "P".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].icon, Some(IconName::Settings));
}

#[test]
fn new_chat_preset_uses_custom_icon() {
    let presets = vec![NewChatPresetInfo {
        id: "code".into(),
        name: "Code".into(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].icon, Some(IconName::Code));
}

// =========================================================================
// 32. Clipboard save actions have correct shortcuts
// =========================================================================

#[test]
fn clipboard_save_snippet_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "ss".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let save = find_action(&actions, "clipboard_save_snippet").unwrap();
    assert_eq!(save.shortcut.as_deref(), Some("⇧⌘S"));
}

#[test]
fn clipboard_save_file_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "sf".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let save = find_action(&actions, "clipboard_save_file").unwrap();
    assert_eq!(save.shortcut.as_deref(), Some("⌥⇧⌘S"));
}

// =========================================================================
// 33. Script context deeplink description format
// =========================================================================

#[test]
fn script_deeplink_description_contains_url() {
    let script = ScriptInfo::new("My Cool Script", "/path/script.ts");
    let actions = get_script_context_actions(&script);
    let deeplink = find_action(&actions, "copy_deeplink").unwrap();
    assert!(
        deeplink
            .description
            .as_ref()
            .unwrap()
            .contains("scriptkit://run/my-cool-script"),
        "Deeplink description should contain the URL"
    );
}

#[test]
fn scriptlet_deeplink_description_contains_url() {
    let script = ScriptInfo::scriptlet("Open GitHub", "/path/url.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let deeplink = find_action(&actions, "copy_deeplink").unwrap();
    assert!(deeplink
        .description
        .as_ref()
        .unwrap()
        .contains("scriptkit://run/open-github"),);
}

// =========================================================================
// 34. All built-in actions have ActionCategory::ScriptContext
// =========================================================================

#[test]
fn script_context_all_actions_are_script_context_category() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    for a in &get_script_context_actions(&script) {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "Action '{}' should be ScriptContext",
            a.id
        );
    }
}

#[test]
fn clipboard_all_actions_are_script_context_category() {
    let entry = ClipboardEntryInfo {
        id: "c".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    for a in &get_clipboard_history_context_actions(&entry) {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "Action '{}' should be ScriptContext",
            a.id
        );
    }
}

#[test]
fn file_all_actions_are_script_context_category() {
    let file = FileInfo {
        path: "/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    for a in &get_file_context_actions(&file) {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "Action '{}' should be ScriptContext",
            a.id
        );
    }
}

#[test]
fn path_all_actions_are_script_context_category() {
    let path = PathInfo {
        path: "/tmp".into(),
        name: "tmp".into(),
        is_dir: true,
    };
    for a in &get_path_context_actions(&path) {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "Action '{}' should be ScriptContext",
            a.id
        );
    }
}

#[test]
fn ai_command_bar_all_actions_are_script_context_category() {
    for a in &get_ai_command_bar_actions() {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "Action '{}' should be ScriptContext",
            a.id
        );
    }
}

#[test]
fn notes_command_bar_all_actions_are_script_context_category() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for a in &get_notes_command_bar_actions(&info) {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "Action '{}' should be ScriptContext",
            a.id
        );
    }
}

// =========================================================================
// 35. Action count bounds
// =========================================================================

#[test]
fn script_context_has_at_least_5_actions() {
    // Any script should have at minimum: run, shortcut, alias, deeplink, + edit/view/reveal/copy
    let script = ScriptInfo::new("test", "/path/test.ts");
    let count = get_script_context_actions(&script).len();
    assert!(
        count >= 5,
        "Script context should have at least 5 actions, got {}",
        count
    );
}

#[test]
fn builtin_context_has_at_least_4_actions() {
    // Built-in: run, add_shortcut, add_alias, copy_deeplink
    let builtin = ScriptInfo::builtin("Test");
    let count = get_script_context_actions(&builtin).len();
    assert!(
        count >= 4,
        "Builtin context should have at least 4 actions, got {}",
        count
    );
}

#[test]
fn clipboard_text_has_at_least_10_actions() {
    let entry = ClipboardEntryInfo {
        id: "t".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let count = get_clipboard_history_context_actions(&entry).len();
    assert!(
        count >= 10,
        "Clipboard text context should have at least 10 actions, got {}",
        count
    );
}

#[test]
fn ai_command_bar_has_exactly_12_actions() {
    let count = get_ai_command_bar_actions().len();
    assert_eq!(
        count, 12,
        "AI command bar should have exactly 12 actions, got {}",
        count
    );
}

// =========================================================================
// 36. Score fuzzy match
// =========================================================================

#[test]
fn score_fuzzy_match_subsequence() {
    let action = Action::new(
        "reveal_in_finder",
        "Reveal in Finder",
        Some("Show in Finder".to_string()),
        ActionCategory::ScriptContext,
    );
    // "rif" is a subsequence of "reveal in finder"
    let score = ActionsDialog::score_action(&action, "rif");
    assert!(
        score > 0,
        "Fuzzy subsequence 'rif' should match 'reveal in finder', got score {}",
        score
    );
}

#[test]
fn score_fuzzy_no_match() {
    let action = Action::new(
        "run_script",
        "Run Script",
        None,
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "xyz");
    assert_eq!(score, 0);
}
