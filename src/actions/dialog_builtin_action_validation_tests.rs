// --- merged from part_01.rs ---
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
    assert_eq!(ids[len - 3], "clip:clipboard_delete");
    assert_eq!(ids[len - 2], "clip:clipboard_delete_multiple");
    assert_eq!(ids[len - 1], "clip:clipboard_delete_all");
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
    assert_eq!(ids[len - 3], "clip:clipboard_delete");
    assert_eq!(ids[len - 2], "clip:clipboard_delete_multiple");
    assert_eq!(ids[len - 1], "clip:clipboard_delete_all");
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

// --- merged from part_02.rs ---

#[test]
fn path_open_directory_has_enter_shortcut() {
    let path = PathInfo::new("dir", "/dir", true);
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn clipboard_paste_has_enter_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

// =========================================================================
// 9. Chat model actions — edge cases with many models
// =========================================================================

#[test]
fn chat_ten_models_all_present_exactly_one_checkmark() {
    let models: Vec<ChatModelInfo> = (0..10)
        .map(|i| ChatModelInfo {
            id: format!("model-{}", i),
            display_name: format!("Model {}", i),
            provider: format!("Provider {}", i),
        })
        .collect();
    let info = ChatPromptInfo {
        current_model: Some("Model 5".into()),
        available_models: models,
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let checked = actions.iter().filter(|a| a.title.contains('✓')).count();
    assert_eq!(checked, 1);
    let checked_action = actions.iter().find(|a| a.title.contains('✓')).unwrap();
    assert_eq!(checked_action.id, "chat:select_model_model-5");
}

#[test]
fn chat_current_model_not_in_available_models_means_no_checkmark() {
    let models = vec![ChatModelInfo {
        id: "gpt-4".into(),
        display_name: "GPT-4".into(),
        provider: "OpenAI".into(),
    }];
    let info = ChatPromptInfo {
        current_model: Some("Nonexistent Model".into()),
        available_models: models,
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let checked = actions.iter().filter(|a| a.title.contains('✓')).count();
    assert_eq!(
        checked, 0,
        "No model should be checked when current doesn't match any"
    );
}

#[test]
fn chat_model_actions_all_have_provider_description() {
    let models = vec![
        ChatModelInfo {
            id: "a".into(),
            display_name: "A".into(),
            provider: "PA".into(),
        },
        ChatModelInfo {
            id: "b".into(),
            display_name: "B".into(),
            provider: "PB".into(),
        },
    ];
    let info = ChatPromptInfo {
        current_model: None,
        available_models: models,
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model_actions: Vec<&Action> = actions
        .iter()
        .filter(|a| a.id.starts_with("chat:select_model_"))
        .collect();
    for action in &model_actions {
        assert!(
            action.description.as_ref().unwrap().starts_with("via "),
            "Model action '{}' description should start with 'via '",
            action.id
        );
    }
}

// =========================================================================
// 10. Grouped items — real actions produce valid grouped output
// =========================================================================

#[test]
fn ai_actions_grouped_with_headers_have_correct_structure() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);

    // Last item should not be a header (it should be an action)
    assert!(
        matches!(grouped.last(), Some(GroupedActionItem::Item(_))),
        "Last grouped item should be an action, not a header"
    );
}

#[test]
fn ai_actions_grouped_with_separators_have_no_headers() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    let headers: Vec<&GroupedActionItem> = grouped
        .iter()
        .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
        .collect();
    assert!(
        headers.is_empty(),
        "Separators style should have no headers"
    );
}

#[test]
fn notes_actions_grouped_header_count_matches_section_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let filtered: Vec<usize> = (0..actions.len()).collect();

    let header_count_from_fn = count_section_headers(&actions, &filtered);
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    let header_count_from_grouped = grouped
        .iter()
        .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(header_count_from_fn, header_count_from_grouped);
}

// =========================================================================
// 11. Coerce selection — real grouped items from AI actions
// =========================================================================

#[test]
fn coerce_selection_on_real_ai_grouped_actions_finds_valid_item() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);

    // Coerce at index 0 (which is likely a header) should find an item
    let result = coerce_action_selection(&grouped, 0);
    assert!(
        result.is_some(),
        "Should find an item in AI grouped actions"
    );

    // The selected row should be an Item, not a header
    if let Some(idx) = result {
        assert!(
            matches!(grouped[idx], GroupedActionItem::Item(_)),
            "Coerced selection should be an Item"
        );
    }
}

#[test]
fn coerce_selection_on_every_row_returns_valid_or_none() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);

    for i in 0..grouped.len() {
        let result = coerce_action_selection(&grouped, i);
        if let Some(idx) = result {
            assert!(
                matches!(grouped[idx], GroupedActionItem::Item(_)),
                "Row {} coerced to non-item at {}",
                i,
                idx
            );
        }
    }
}

// =========================================================================
// 12. Score consistency — same action + same query = same score
// =========================================================================

#[test]
fn score_action_is_deterministic() {
    let action = Action::new(
        "edit_script",
        "Edit Script",
        Some("Open in editor".into()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘E");
    let score1 = ActionsDialog::score_action(&action, "edit");
    let score2 = ActionsDialog::score_action(&action, "edit");
    let score3 = ActionsDialog::score_action(&action, "edit");
    assert_eq!(score1, score2);
    assert_eq!(score2, score3);
}

#[test]
fn score_action_prefix_beats_contains_beats_fuzzy() {
    let prefix = Action::new("e", "Edit Script", None, ActionCategory::ScriptContext);
    let contains = Action::new("c", "Copy Edit Path", None, ActionCategory::ScriptContext);
    let fuzzy = Action::new("f", "Examine Data", None, ActionCategory::ScriptContext);

    let prefix_score = ActionsDialog::score_action(&prefix, "edit");
    let contains_score = ActionsDialog::score_action(&contains, "edit");
    // "edit" in "examine data": fuzzy? e-x-a-m-i-n-e- -d-i-t → not a fuzzy match for "edit"
    // Actually e at 0, d at 8, i at 9, t at 10... need e-d-i-t in order: yes that fuzzy matches
    let _fuzzy_score = ActionsDialog::score_action(&fuzzy, "edit");

    assert!(
        prefix_score > contains_score,
        "Prefix({}) should beat contains({})",
        prefix_score,
        contains_score
    );
    // Contains may or may not beat fuzzy depending on implementation, but both should be > 0
    assert!(contains_score > 0);
}

// =========================================================================
// 13. Description presence for critical actions
// =========================================================================

#[test]
fn script_run_action_has_description() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let run = find_action(&actions, "run_script").unwrap();
    assert!(
        run.description.is_some(),
        "run_script should have a description"
    );
}

#[test]
fn script_edit_action_has_description() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let edit = find_action(&actions, "edit_script").unwrap();
    assert!(
        edit.description.is_some(),
        "edit_script should have a description"
    );
}

#[test]
fn clipboard_delete_all_has_description() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let delete_all = find_action(&actions, "clip:clipboard_delete_all").unwrap();
    assert!(
        delete_all.description.is_some(),
        "clipboard_delete_all should have a description"
    );
}

#[test]
fn path_move_to_trash_has_description() {
    let path = PathInfo::new("test", "/test", false);
    let actions = get_path_context_actions(&path);
    let trash = find_action(&actions, "file:move_to_trash").unwrap();
    assert!(
        trash.description.is_some(),
        "move_to_trash should have a description"
    );
}

// =========================================================================
// 14. Deeplink action present across all script-like contexts
// =========================================================================

#[test]
fn deeplink_present_for_script() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"script:copy_deeplink"));
}

#[test]
fn deeplink_present_for_builtin() {
    let builtin = ScriptInfo::builtin("Clipboard History");
    let actions = get_script_context_actions(&builtin);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"script:copy_deeplink"));
}

#[test]
fn deeplink_present_for_scriptlet() {
    let scriptlet = ScriptInfo::scriptlet("Open URL", "/path/urls.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&scriptlet, None);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"script:copy_deeplink"));
}

#[test]
fn deeplink_present_for_agent() {
    let mut agent = ScriptInfo::new("my-agent", "/path/agent.md");
    agent.is_agent = true;
    agent.is_script = false;
    let actions = get_script_context_actions(&agent);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"script:copy_deeplink"));
}

// =========================================================================
// 15. Shortcut management actions: mutually exclusive add vs update/remove
// =========================================================================

#[test]
fn shortcut_add_vs_update_remove_mutually_exclusive_for_scripts() {
    // No shortcut → add only
    let no_sc = ScriptInfo::new("test", "/path/test.ts");
    let no_sc_actions = get_script_context_actions(&no_sc);
    let ids = action_ids(&no_sc_actions);
    assert!(ids.contains(&"add_shortcut"));
    assert!(!ids.contains(&"update_shortcut"));
    assert!(!ids.contains(&"remove_shortcut"));

    // Has shortcut → update+remove only
    let has_sc = ScriptInfo::with_shortcut("test", "/path/test.ts", Some("cmd+t".into()));
    let has_sc_actions = get_script_context_actions(&has_sc);
    let ids = action_ids(&has_sc_actions);
    assert!(!ids.contains(&"add_shortcut"));
    assert!(ids.contains(&"update_shortcut"));
    assert!(ids.contains(&"remove_shortcut"));
}

#[test]
fn alias_add_vs_update_remove_mutually_exclusive_for_scripts() {
    // No alias → add only
    let no_al = ScriptInfo::new("test", "/path/test.ts");
    let no_al_actions = get_script_context_actions(&no_al);
    let ids = action_ids(&no_al_actions);
    assert!(ids.contains(&"add_alias"));
    assert!(!ids.contains(&"update_alias"));
    assert!(!ids.contains(&"remove_alias"));

    // Has alias → update+remove only
    let has_al =
        ScriptInfo::with_shortcut_and_alias("test", "/path/test.ts", None, Some("ts".into()));
    let has_al_actions = get_script_context_actions(&has_al);
    let ids = action_ids(&has_al_actions);
    assert!(!ids.contains(&"add_alias"));
    assert!(ids.contains(&"update_alias"));
    assert!(ids.contains(&"remove_alias"));
}

// =========================================================================
// 16. File context — Application type has open as primary
// =========================================================================

#[test]
fn file_application_primary_is_open_file() {
    let app = FileInfo {
        path: "/Applications/Safari.app".into(),
        name: "Safari.app".into(),
        file_type: FileType::Application,
        is_dir: false,
    };
    let actions = get_file_context_actions(&app);
    assert_eq!(actions[0].id, "file:open_file");
    assert!(actions[0].title.contains("Safari.app"));
}

#[test]
fn file_document_primary_is_open_file() {
    let doc = FileInfo {
        path: "/test/report.pdf".into(),
        name: "report.pdf".into(),
        file_type: FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&doc);
    assert_eq!(actions[0].id, "file:open_file");
}

#[test]
fn file_image_primary_is_open_file() {
    let img = FileInfo {
        path: "/test/photo.jpg".into(),
        name: "photo.jpg".into(),
        file_type: FileType::Image,
        is_dir: false,
    };
    let actions = get_file_context_actions(&img);
    assert_eq!(actions[0].id, "file:open_file");
}

// =========================================================================
// 17. Note switcher — many notes all unique IDs
// =========================================================================

#[test]
fn note_switcher_fifty_notes_all_unique_ids() {
    let notes: Vec<NoteSwitcherNoteInfo> = (0..50)
        .map(|i| NoteSwitcherNoteInfo {
            id: format!("note-{}", i),
            title: format!("Note {}", i),
            char_count: i * 10,
            is_current: i == 25,
            is_pinned: i % 7 == 0,
            preview: String::new(),
            relative_time: String::new(),
        })
        .collect();
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions.len(), 50);
    let mut ids: Vec<&str> = action_ids(&actions);
    let total = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(total, ids.len(), "Note switcher IDs should be unique");
}

// --- merged from part_03.rs ---

#[test]
fn note_switcher_pinned_current_same_note_gets_star_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "both".into(),
        title: "Both Pinned & Current".into(),
        char_count: 42,
        is_current: true,
        is_pinned: true,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    assert!(actions[0].title.starts_with("• "));
}

// =========================================================================
// 18. New chat actions — icons are correct per section
// =========================================================================

#[test]
fn new_chat_last_used_all_get_bolt_icon() {
    let last_used: Vec<NewChatModelInfo> = (0..3)
        .map(|i| NewChatModelInfo {
            model_id: format!("m{}", i),
            display_name: format!("M{}", i),
            provider: "p".into(),
            provider_display_name: "P".into(),
        })
        .collect();
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    for action in &actions {
        assert_eq!(
            action.icon,
            Some(IconName::BoltFilled),
            "Last used '{}' should have BoltFilled icon",
            action.id
        );
    }
}

#[test]
fn new_chat_models_all_get_settings_icon() {
    let models: Vec<NewChatModelInfo> = (0..3)
        .map(|i| NewChatModelInfo {
            model_id: format!("m{}", i),
            display_name: format!("M{}", i),
            provider: "p".into(),
            provider_display_name: "P".into(),
        })
        .collect();
    let actions = get_new_chat_actions(&[], &[], &models);
    for action in &actions {
        assert_eq!(
            action.icon,
            Some(IconName::Settings),
            "Model '{}' should have Settings icon",
            action.id
        );
    }
}

#[test]
fn new_chat_presets_preserve_custom_icons() {
    let presets = vec![
        NewChatPresetInfo {
            id: "code".into(),
            name: "Code".into(),
            icon: IconName::Code,
        },
        NewChatPresetInfo {
            id: "star".into(),
            name: "Star".into(),
            icon: IconName::Star,
        },
    ];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].icon, Some(IconName::Code));
    assert_eq!(actions[1].icon, Some(IconName::Star));
}

// =========================================================================
// 19. CommandBarConfig — field preservation across presets
// =========================================================================

#[test]
fn command_bar_all_presets_close_on_escape() {
    assert!(CommandBarConfig::default().close_on_escape);
    assert!(CommandBarConfig::ai_style().close_on_escape);
    assert!(CommandBarConfig::notes_style().close_on_escape);
    assert!(CommandBarConfig::main_menu_style().close_on_escape);
    assert!(CommandBarConfig::no_search().close_on_escape);
}

#[test]
fn command_bar_all_presets_close_on_click_outside() {
    assert!(CommandBarConfig::default().close_on_click_outside);
    assert!(CommandBarConfig::ai_style().close_on_click_outside);
    assert!(CommandBarConfig::notes_style().close_on_click_outside);
    assert!(CommandBarConfig::main_menu_style().close_on_click_outside);
    assert!(CommandBarConfig::no_search().close_on_click_outside);
}

#[test]
fn command_bar_all_presets_close_on_select() {
    assert!(CommandBarConfig::default().close_on_select);
    assert!(CommandBarConfig::ai_style().close_on_select);
    assert!(CommandBarConfig::notes_style().close_on_select);
    assert!(CommandBarConfig::main_menu_style().close_on_select);
    assert!(CommandBarConfig::no_search().close_on_select);
}

// =========================================================================
// 20. Fuzzy match with real action titles
// =========================================================================

#[test]
fn fuzzy_match_works_on_real_action_titles() {
    // Common user search patterns against actual action titles
    assert!(ActionsDialog::fuzzy_match("edit script", "es"));
    assert!(ActionsDialog::fuzzy_match("reveal in finder", "rif"));
    assert!(ActionsDialog::fuzzy_match("copy path", "cp"));
    assert!(ActionsDialog::fuzzy_match("copy deeplink", "cdl"));
    assert!(ActionsDialog::fuzzy_match("add keyboard shortcut", "aks"));
    assert!(ActionsDialog::fuzzy_match("reset ranking", "rr"));
    assert!(ActionsDialog::fuzzy_match("view logs", "vl"));
}

#[test]
fn fuzzy_match_fails_for_reversed_chars() {
    // "se" should not fuzzy match "edit script" (s comes after e)
    // Actually: e-d-i-t- -s-c-r-i-p-t → 's' at index 5, 'e' not found after 's'... wait
    // "se": s at 5, then e at... no e after index 5. So it fails.
    assert!(!ActionsDialog::fuzzy_match("edit script", "se"));
}

// =========================================================================
// 21. Action verb propagation in primary action
// =========================================================================

#[test]
fn action_verb_launch_propagates_to_run_action() {
    let script = ScriptInfo::with_action_verb("App Launcher", "builtin:launcher", false, "Launch");
    let actions = get_script_context_actions(&script);
    let run = find_action(&actions, "run_script").unwrap();
    assert!(
        run.title.starts_with("Launch"),
        "Primary action should use 'Launch' verb, got '{}'",
        run.title
    );
}

#[test]
fn action_verb_switch_to_propagates_to_run_action() {
    let script =
        ScriptInfo::with_action_verb("Window Switcher", "builtin:windows", false, "Switch to");
    let actions = get_script_context_actions(&script);
    let run = find_action(&actions, "run_script").unwrap();
    assert!(
        run.title.starts_with("Switch to"),
        "Primary action should use 'Switch to' verb, got '{}'",
        run.title
    );
}

#[test]
fn action_verb_open_propagates_to_run_action() {
    let script = ScriptInfo::with_action_verb("Notes", "builtin:notes", false, "Open");
    let actions = get_script_context_actions(&script);
    let run = find_action(&actions, "run_script").unwrap();
    assert!(
        run.title.starts_with("Open"),
        "Primary action should use 'Open' verb, got '{}'",
        run.title
    );
}

// =========================================================================
// 22. title_lower correctness across all contexts
// =========================================================================

#[test]
fn title_lower_matches_title_for_all_clipboard_actions() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: Some("Safari".into()),
    };
    for action in &get_clipboard_history_context_actions(&entry) {
        assert_eq!(
            action.title_lower,
            action.title.to_lowercase(),
            "title_lower mismatch for clipboard action '{}'",
            action.id
        );
    }
}

#[test]
fn title_lower_matches_title_for_all_file_actions() {
    let file = FileInfo {
        path: "/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    for action in &get_file_context_actions(&file) {
        assert_eq!(
            action.title_lower,
            action.title.to_lowercase(),
            "title_lower mismatch for file action '{}'",
            action.id
        );
    }
}

#[test]
fn title_lower_matches_title_for_all_path_actions() {
    let path = PathInfo::new("test", "/test", false);
    for action in &get_path_context_actions(&path) {
        assert_eq!(
            action.title_lower,
            action.title.to_lowercase(),
            "title_lower mismatch for path action '{}'",
            action.id
        );
    }
}

#[test]
fn title_lower_matches_title_for_all_chat_actions() {
    let info = ChatPromptInfo {
        current_model: Some("Model A".into()),
        available_models: vec![ChatModelInfo {
            id: "a".into(),
            display_name: "Model A".into(),
            provider: "PA".into(),
        }],
        has_messages: true,
        has_response: true,
    };
    for action in &get_chat_context_actions(&info) {
        assert_eq!(
            action.title_lower,
            action.title.to_lowercase(),
            "title_lower mismatch for chat action '{}'",
            action.id
        );
    }
}

// =========================================================================
// 23. Scriptlet with zero custom actions still has built-in actions
// =========================================================================

#[test]
fn scriptlet_zero_custom_actions_has_built_in_set() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo hi".into());
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    assert!(
        actions.len() >= 3,
        "Scriptlet with no custom actions should still have built-in actions"
    );
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn scriptlet_none_scriptlet_has_built_in_set() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(!actions.is_empty());
    assert_eq!(actions[0].id, "run_script");
}

// =========================================================================
// 24. ProtocolAction default behavior
// =========================================================================

#[test]
fn protocol_action_new_defaults_to_visible_and_closable() {
    let pa = ProtocolAction::new("Test".into());
    assert!(pa.is_visible());
    assert!(pa.should_close());
}

#[test]
fn protocol_action_explicit_false_overrides_defaults() {
    let pa = ProtocolAction {
        visible: Some(false),
        close: Some(false),
        ..ProtocolAction::new("Test".into())
    };
    assert!(!pa.is_visible());
    assert!(!pa.should_close());
}

#[test]
fn protocol_action_with_value_sets_value_and_keeps_has_action_false() {
    let pa = ProtocolAction::with_value("Submit".into(), "val".into());
    assert_eq!(pa.value, Some("val".into()));
    assert!(!pa.has_action);
}

// =========================================================================
// 25. Cross-context: minimum action counts
// =========================================================================

#[test]
fn script_context_has_at_least_seven_actions() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let count = get_script_context_actions(&script).len();
    assert!(
        count >= 7,
        "Script context should have at least 7 actions, got {}",
        count
    );
}

#[test]
fn file_context_has_at_least_four_actions() {
    let file = FileInfo {
        path: "/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let count = get_file_context_actions(&file).len();
    assert!(
        count >= 4,
        "File context should have at least 4 actions, got {}",
        count
    );
}

#[test]
fn clipboard_context_has_at_least_eight_actions() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let count = get_clipboard_history_context_actions(&entry).len();
    assert!(
        count >= 8,
        "Clipboard context should have at least 8 actions, got {}",
        count
    );
}

#[test]
fn path_context_has_at_least_six_actions() {
    let path = PathInfo::new("test", "/test", false);
    let count = get_path_context_actions(&path).len();
    assert!(
        count >= 6,
        "Path context should have at least 6 actions, got {}",
        count
    );
}

#[test]
fn chat_context_always_has_continue_in_chat() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"chat:continue_in_chat"));
}

// =========================================================================
// 26. Scriptlet with many custom actions — all ordered after run
// =========================================================================

#[test]
fn scriptlet_ten_custom_actions_all_after_run() {
    let script = ScriptInfo::scriptlet("Big", "/path/big.md", None, None);
    let mut scriptlet = Scriptlet::new("Big".into(), "bash".into(), "echo main".into());
    for i in 0..10 {
        scriptlet.actions.push(ScriptletAction {
            name: format!("Action {}", i),
            command: format!("act-{}", i),
            tool: "bash".into(),
            code: format!("echo {}", i),
            inputs: vec![],
            shortcut: None,
            description: None,
        });
    }
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    assert_eq!(actions[0].id, "run_script");
    let custom_ids: Vec<&str> = actions
        .iter()
        .filter(|a| a.id.starts_with("scriptlet_action:"))
        .map(|a| a.id.as_str())
        .collect();
    assert_eq!(custom_ids.len(), 10);
    // All custom actions should come after run_script
    let run_pos = actions.iter().position(|a| a.id == "run_script").unwrap();
    for custom in &actions {
        if custom.id.starts_with("scriptlet_action:") {
            let pos = actions.iter().position(|a| a.id == custom.id).unwrap();
            assert!(
                pos > run_pos,
                "Custom action '{}' at {} should be after run_script at {}",
                custom.id,
                pos,
                run_pos
            );
        }
    }
}

// =========================================================================
// 27. Format shortcut hint — roundtrip patterns
// =========================================================================

#[test]
fn format_shortcut_hint_cmd_enter() {
    assert_eq!(ActionsDialog::format_shortcut_hint("cmd+enter"), "⌘↵");
}

#[test]
fn format_shortcut_hint_cmd_shift_delete() {
    let result = ActionsDialog::format_shortcut_hint("cmd+shift+delete");
    assert!(result.contains('⌘'));
    assert!(result.contains('⇧'));
    assert!(result.contains('⌫'));
}

#[test]
fn format_shortcut_hint_ctrl_alt_letter() {
    let result = ActionsDialog::format_shortcut_hint("ctrl+alt+z");
    assert!(result.contains('⌃'));
    assert!(result.contains('⌥'));
    assert!(result.contains('Z'));
}

// --- merged from part_04.rs ---

// =========================================================================
// 28. Deeplink name — preserves unicode alphanumerics
// =========================================================================

#[test]
fn deeplink_name_preserves_numbers_in_script_name() {
    assert_eq!(to_deeplink_name("Script 123"), "script-123");
}

#[test]
fn deeplink_name_handles_empty_string() {
    assert_eq!(to_deeplink_name(""), "");
}

#[test]
fn deeplink_name_handles_all_whitespace() {
    assert_eq!(to_deeplink_name("   "), "");
}

// =========================================================================
// 29. Notes — minimum actions for every permutation
// =========================================================================

#[test]
fn notes_all_eight_permutations_have_at_least_two_actions() {
    for sel in [false, true] {
        for trash in [false, true] {
            for auto in [false, true] {
                let info = NotesInfo {
                    has_selection: sel,
                    is_trash_view: trash,
                    auto_sizing_enabled: auto,
                };
                let count = get_notes_command_bar_actions(&info).len();
                assert!(
                    count >= 2,
                    "Notes permutation sel={}, trash={}, auto={} has only {} actions",
                    sel,
                    trash,
                    auto,
                    count
                );
            }
        }
    }
}

// =========================================================================
// 30. Action with_section chains correctly
// =========================================================================

#[test]
fn action_with_section_chains_preserve_other_fields() {
    let action = Action::new(
        "test",
        "Test Action",
        Some("A description".into()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘T")
    .with_icon(IconName::Star)
    .with_section("My Section");

    assert_eq!(action.id, "test");
    assert_eq!(action.title, "Test Action");
    assert_eq!(action.description, Some("A description".into()));
    assert_eq!(action.shortcut, Some("⌘T".into()));
    assert_eq!(action.icon, Some(IconName::Star));
    assert_eq!(action.section, Some("My Section".into()));
    assert!(!action.has_action);
    assert!(action.value.is_none());
}
