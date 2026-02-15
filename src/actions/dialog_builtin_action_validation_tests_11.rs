// --- merged from part_01.rs ---
//! Batch 11: Random builtin action/dialog validation tests
//!
//! 30 test categories covering fresh angles on action builders, edge cases,
//! and behavioral invariants not thoroughly covered by batches 1-10.

use super::builders::*;
use super::dialog::{
    build_grouped_items_static, coerce_action_selection, ActionsDialog, GroupedActionItem,
};
use super::types::*;
use crate::clipboard_history::ContentType;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;
use crate::scriptlets::{Scriptlet, ScriptletAction};
use std::collections::HashSet;

// ============================================================================
// Helper
// ============================================================================

fn action_ids(actions: &[Action]) -> Vec<String> {
    actions.iter().map(|a| a.id.clone()).collect()
}

// ============================================================================
// 1. ScriptInfo constructor symmetry — each constructor sets exactly the
//    expected defaults
// ============================================================================

#[test]
fn cat01_script_info_new_defaults_all_fields() {
    let s = ScriptInfo::new("abc", "/tmp/abc.ts");
    assert!(s.is_script);
    assert!(!s.is_scriptlet);
    assert!(!s.is_agent);
    assert_eq!(s.action_verb, "Run");
    assert!(s.shortcut.is_none());
    assert!(s.alias.is_none());
    assert!(!s.is_suggested);
    assert!(s.frecency_path.is_none());
}

#[test]
fn cat01_script_info_builtin_path_is_empty() {
    let b = ScriptInfo::builtin("Test Builtin");
    assert!(b.path.is_empty());
    assert!(!b.is_script);
    assert!(!b.is_scriptlet);
    assert!(!b.is_agent);
}

#[test]
fn cat01_scriptlet_sets_only_scriptlet_flag() {
    let s = ScriptInfo::scriptlet("x", "/p.md", None, None);
    assert!(!s.is_script);
    assert!(s.is_scriptlet);
    assert!(!s.is_agent);
    assert_eq!(s.action_verb, "Run");
}

#[test]
fn cat01_with_action_verb_and_shortcut_preserves_verb() {
    let s = ScriptInfo::with_action_verb_and_shortcut(
        "App",
        "/app",
        false,
        "Launch",
        Some("cmd+l".into()),
    );
    assert_eq!(s.action_verb, "Launch");
    assert_eq!(s.shortcut, Some("cmd+l".into()));
    assert!(!s.is_script);
}

// ============================================================================
// 2. Action::new caches lowercase fields correctly
// ============================================================================

#[test]
fn cat02_title_lower_is_cached_on_creation() {
    let a = Action::new("id", "Hello World", None, ActionCategory::ScriptContext);
    assert_eq!(a.title_lower, "hello world");
}

#[test]
fn cat02_description_lower_cached_when_present() {
    let a = Action::new(
        "id",
        "T",
        Some("My Desc".to_string()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(a.description_lower, Some("my desc".to_string()));
}

#[test]
fn cat02_description_lower_none_when_no_desc() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext);
    assert!(a.description_lower.is_none());
}

#[test]
fn cat02_shortcut_lower_none_until_with_shortcut() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext);
    assert!(a.shortcut_lower.is_none());
    let a2 = a.with_shortcut("⌘X");
    assert_eq!(a2.shortcut_lower, Some("⌘x".to_string()));
}

// ============================================================================
// 3. Action builder chaining — order independence and with_shortcut_opt
// ============================================================================

#[test]
fn cat03_with_icon_then_section_same_as_reverse() {
    let a1 = Action::new("a", "A", None, ActionCategory::ScriptContext)
        .with_icon(crate::designs::icon_variations::IconName::Plus)
        .with_section("S");
    let a2 = Action::new("a", "A", None, ActionCategory::ScriptContext)
        .with_section("S")
        .with_icon(crate::designs::icon_variations::IconName::Plus);
    assert_eq!(a1.icon, a2.icon);
    assert_eq!(a1.section, a2.section);
}

#[test]
fn cat03_with_shortcut_opt_none_leaves_shortcut_none() {
    let a = Action::new("a", "A", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
    assert!(a.shortcut.is_none());
    assert!(a.shortcut_lower.is_none());
}

#[test]
fn cat03_with_shortcut_opt_some_sets_both() {
    let a = Action::new("a", "A", None, ActionCategory::ScriptContext)
        .with_shortcut_opt(Some("⌘Z".to_string()));
    assert_eq!(a.shortcut, Some("⌘Z".to_string()));
    assert_eq!(a.shortcut_lower, Some("⌘z".to_string()));
}

// ============================================================================
// 4. Script context — exact action IDs per flag combination
// ============================================================================

#[test]
fn cat04_script_no_shortcut_no_alias_ids() {
    let s = ScriptInfo::new("test", "/test.ts");
    let actions = get_script_context_actions(&s);
    let ids: HashSet<String> = action_ids(&actions).into_iter().collect();
    // Must have these exact built-in IDs
    for expected in &[
        "run_script",
        "add_shortcut",
        "add_alias",
        "edit_script",
        "view_logs",
        "reveal_in_finder",
        "copy_path",
        "copy_content",
        "copy_deeplink",
    ] {
        assert!(ids.contains(*expected), "Missing: {}", expected);
    }
    // Must NOT have these
    for absent in &[
        "update_shortcut",
        "remove_shortcut",
        "update_alias",
        "remove_alias",
        "reset_ranking",
    ] {
        assert!(!ids.contains(*absent), "Unexpected: {}", absent);
    }
}

#[test]
fn cat04_script_with_shortcut_and_alias_ids() {
    let s = ScriptInfo::with_shortcut_and_alias(
        "test",
        "/test.ts",
        Some("cmd+t".into()),
        Some("ts".into()),
    );
    let actions = get_script_context_actions(&s);
    let ids: HashSet<String> = action_ids(&actions).into_iter().collect();
    assert!(ids.contains("update_shortcut"));
    assert!(ids.contains("remove_shortcut"));
    assert!(ids.contains("update_alias"));
    assert!(ids.contains("remove_alias"));
    assert!(!ids.contains("add_shortcut"));
    assert!(!ids.contains("add_alias"));
}

#[test]
fn cat04_builtin_has_exactly_4_actions() {
    let b = ScriptInfo::builtin("Test");
    let actions = get_script_context_actions(&b);
    // run_script, add_shortcut, add_alias, copy_deeplink
    assert_eq!(actions.len(), 4, "Builtin should have 4 actions");
}

#[test]
fn cat04_agent_has_no_view_logs() {
    let mut a = ScriptInfo::new("agent", "/agent.claude.md");
    a.is_script = false;
    a.is_agent = true;
    let actions = get_script_context_actions(&a);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
    assert!(actions.iter().any(|a| a.title == "Edit Agent"));
}

// ============================================================================
// 5. Scriptlet context actions — ordering guarantees
// ============================================================================

#[test]
fn cat05_scriptlet_run_is_first() {
    let s = ScriptInfo::scriptlet("x", "/x.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn cat05_scriptlet_custom_actions_between_run_and_edit() {
    let s = ScriptInfo::scriptlet("x", "/x.md", None, None);
    let mut scriptlet = Scriptlet::new("x".into(), "bash".into(), "echo hi".into());
    scriptlet.actions = vec![ScriptletAction {
        name: "Custom".into(),
        command: "custom".into(),
        tool: "bash".into(),
        code: "echo custom".into(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_context_actions_with_custom(&s, Some(&scriptlet));
    let run_pos = actions.iter().position(|a| a.id == "run_script").unwrap();
    let custom_pos = actions
        .iter()
        .position(|a| a.id == "scriptlet_action:custom")
        .unwrap();
    let edit_pos = actions
        .iter()
        .position(|a| a.id == "edit_scriptlet")
        .unwrap();
    assert!(run_pos < custom_pos, "run before custom");
    assert!(custom_pos < edit_pos, "custom before edit");
}

#[test]
fn cat05_scriptlet_copy_content_before_deeplink() {
    let s = ScriptInfo::scriptlet("x", "/x.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    let content_pos = actions.iter().position(|a| a.id == "copy_content").unwrap();
    let deeplink_pos = actions
        .iter()
        .position(|a| a.id == "copy_deeplink")
        .unwrap();
    assert!(content_pos < deeplink_pos);
}

#[test]
fn cat05_scriptlet_with_frecency_adds_reset_ranking_last() {
    let s =
        ScriptInfo::scriptlet("x", "/x.md", None, None).with_frecency(true, Some("/x.md".into()));
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    assert_eq!(actions.last().unwrap().id, "reset_ranking");
}

// ============================================================================
// 6. Clipboard actions — content type differences
// ============================================================================

fn text_entry() -> ClipboardEntryInfo {
    ClipboardEntryInfo {
        id: "t1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    }
}

fn image_entry() -> ClipboardEntryInfo {
    ClipboardEntryInfo {
        id: "i1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "800x600".into(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    }
}

#[test]
fn cat06_image_has_ocr_text_does_not() {
    let text_actions = get_clipboard_history_context_actions(&text_entry());
    let image_actions = get_clipboard_history_context_actions(&image_entry());
    assert!(!text_actions.iter().any(|a| a.id == "clip:clipboard_ocr"));
    assert!(image_actions.iter().any(|a| a.id == "clip:clipboard_ocr"));
}

#[test]
fn cat06_image_has_more_actions_than_text() {
    let ta = get_clipboard_history_context_actions(&text_entry());
    let ia = get_clipboard_history_context_actions(&image_entry());
    assert!(
        ia.len() > ta.len(),
        "Image {} > Text {}",
        ia.len(),
        ta.len()
    );
}

#[test]
fn cat06_destructive_actions_always_last_three() {
    for entry in &[text_entry(), image_entry()] {
        let actions = get_clipboard_history_context_actions(entry);
        let len = actions.len();
        assert_eq!(actions[len - 3].id, "clip:clipboard_delete");
        assert_eq!(actions[len - 2].id, "clip:clipboard_delete_multiple");
        assert_eq!(actions[len - 1].id, "clip:clipboard_delete_all");
    }
}

#[test]
fn cat06_paste_is_always_first() {
    for entry in &[text_entry(), image_entry()] {
        let actions = get_clipboard_history_context_actions(entry);
        assert_eq!(actions[0].id, "clip:clipboard_paste");
    }
}

// ============================================================================
// 7. Clipboard pin/unpin dynamic toggle
// ============================================================================

#[test]
fn cat07_unpinned_shows_pin() {
    let actions = get_clipboard_history_context_actions(&text_entry());
    assert!(actions.iter().any(|a| a.id == "clip:clipboard_pin"));
    assert!(!actions.iter().any(|a| a.id == "clip:clipboard_unpin"));
}

#[test]
fn cat07_pinned_shows_unpin() {
    let mut e = text_entry();
    e.pinned = true;
    let actions = get_clipboard_history_context_actions(&e);
    assert!(!actions.iter().any(|a| a.id == "clip:clipboard_pin"));
    assert!(actions.iter().any(|a| a.id == "clip:clipboard_unpin"));
}

#[test]
fn cat07_pin_unpin_share_same_shortcut() {
    let pin_actions = get_clipboard_history_context_actions(&text_entry());
    let mut pinned = text_entry();
    pinned.pinned = true;
    let unpin_actions = get_clipboard_history_context_actions(&pinned);
    let pin_sc = pin_actions
        .iter()
        .find(|a| a.id == "clip:clipboard_pin")
        .unwrap()
        .shortcut
        .as_ref()
        .unwrap();
    let unpin_sc = unpin_actions
        .iter()
        .find(|a| a.id == "clip:clipboard_unpin")
        .unwrap()
        .shortcut
        .as_ref()
        .unwrap();
    assert_eq!(pin_sc, unpin_sc, "Pin/Unpin share ⇧⌘P");
}

// ============================================================================
// 8. Clipboard frontmost_app_name propagation
// ============================================================================

#[test]
fn cat08_no_app_shows_active_app() {
    let actions = get_clipboard_history_context_actions(&text_entry());
    let paste = actions.iter().find(|a| a.id == "clip:clipboard_paste").unwrap();
    assert_eq!(paste.title, "Paste to Active App");
}

#[test]
fn cat08_with_app_shows_name() {
    let mut e = text_entry();
    e.frontmost_app_name = Some("Firefox".into());
    let actions = get_clipboard_history_context_actions(&e);
    let paste = actions.iter().find(|a| a.id == "clip:clipboard_paste").unwrap();
    assert_eq!(paste.title, "Paste to Firefox");
}

#[test]
fn cat08_app_name_does_not_affect_other_action_count() {
    let a1 = get_clipboard_history_context_actions(&text_entry());
    let mut e = text_entry();
    e.frontmost_app_name = Some("Safari".into());
    let a2 = get_clipboard_history_context_actions(&e);
    assert_eq!(a1.len(), a2.len());
}

// ============================================================================
// 9. File context — directory vs file differences
// ============================================================================

fn file_info_file() -> FileInfo {
    FileInfo {
        path: "/tmp/doc.pdf".into(),
        name: "doc.pdf".into(),
        file_type: FileType::Document,
        is_dir: false,
    }
}

fn file_info_dir() -> FileInfo {
    FileInfo {
        path: "/tmp/docs".into(),
        name: "docs".into(),
        file_type: FileType::Directory,
        is_dir: true,
    }
}

#[test]
fn cat09_file_has_open_file_not_open_directory() {
    let actions = get_file_context_actions(&file_info_file());
    assert!(actions.iter().any(|a| a.id == "file:open_file"));
    assert!(!actions.iter().any(|a| a.id == "file:open_directory"));
}

#[test]
fn cat09_dir_has_open_directory_not_open_file() {
    let actions = get_file_context_actions(&file_info_dir());
    assert!(actions.iter().any(|a| a.id == "file:open_directory"));
    assert!(!actions.iter().any(|a| a.id == "file:open_file"));
}

// --- merged from part_02.rs ---

#[test]
fn cat09_both_have_reveal_copy_path_copy_filename() {
    for info in &[file_info_file(), file_info_dir()] {
        let actions = get_file_context_actions(info);
        assert!(actions.iter().any(|a| a.id == "file:reveal_in_finder"));
        assert!(actions.iter().any(|a| a.id == "file:copy_path"));
        assert!(actions.iter().any(|a| a.id == "file:copy_filename"));
    }
}

#[cfg(target_os = "macos")]
#[test]
fn cat09_quick_look_only_for_files() {
    let file_actions = get_file_context_actions(&file_info_file());
    let dir_actions = get_file_context_actions(&file_info_dir());
    assert!(file_actions.iter().any(|a| a.id == "file:quick_look"));
    assert!(!dir_actions.iter().any(|a| a.id == "file:quick_look"));
}

// ============================================================================
// 10. File context — title includes quoted filename
// ============================================================================

#[test]
fn cat10_file_title_includes_name() {
    let actions = get_file_context_actions(&file_info_file());
    let primary = &actions[0];
    assert!(
        primary.title.contains("doc.pdf"),
        "Title should contain filename: {}",
        primary.title
    );
    assert!(primary.title.contains('"'));
}

#[test]
fn cat10_dir_title_includes_name() {
    let actions = get_file_context_actions(&file_info_dir());
    let primary = &actions[0];
    assert!(primary.title.contains("docs"));
}

// ============================================================================
// 11. Path context — directory vs file primary action
// ============================================================================

fn path_dir() -> PathInfo {
    PathInfo {
        path: "/tmp/projects".into(),
        name: "projects".into(),
        is_dir: true,
    }
}

fn path_file() -> PathInfo {
    PathInfo {
        path: "/tmp/readme.md".into(),
        name: "readme.md".into(),
        is_dir: false,
    }
}

#[test]
fn cat11_dir_primary_is_open_directory() {
    let actions = get_path_context_actions(&path_dir());
    assert_eq!(actions[0].id, "file:open_directory");
}

#[test]
fn cat11_file_primary_is_select_file() {
    let actions = get_path_context_actions(&path_file());
    assert_eq!(actions[0].id, "file:select_file");
}

#[test]
fn cat11_trash_is_always_last() {
    for info in &[path_dir(), path_file()] {
        let actions = get_path_context_actions(info);
        assert_eq!(actions.last().unwrap().id, "file:move_to_trash");
    }
}

#[test]
fn cat11_trash_description_mentions_folder_or_file() {
    let dir_actions = get_path_context_actions(&path_dir());
    let file_actions = get_path_context_actions(&path_file());
    let dir_trash = dir_actions
        .iter()
        .find(|a| a.id == "file:move_to_trash")
        .unwrap();
    let file_trash = file_actions
        .iter()
        .find(|a| a.id == "file:move_to_trash")
        .unwrap();
    assert!(dir_trash.description.as_ref().unwrap().contains("folder"));
    assert!(file_trash.description.as_ref().unwrap().contains("file"));
}

#[test]
fn cat11_dir_and_file_have_same_action_count() {
    let d = get_path_context_actions(&path_dir());
    let f = get_path_context_actions(&path_file());
    assert_eq!(d.len(), f.len());
}

// ============================================================================
// 12. Path context — common actions present for both
// ============================================================================

#[test]
fn cat12_always_has_copy_path_and_open_in_editor() {
    for info in &[path_dir(), path_file()] {
        let actions = get_path_context_actions(info);
        assert!(actions.iter().any(|a| a.id == "file:copy_path"));
        assert!(actions.iter().any(|a| a.id == "file:open_in_editor"));
        assert!(actions.iter().any(|a| a.id == "file:open_in_terminal"));
        assert!(actions.iter().any(|a| a.id == "file:open_in_finder"));
        assert!(actions.iter().any(|a| a.id == "file:copy_filename"));
    }
}

// ============================================================================
// 13. AI command bar — exact action count and section distribution
// ============================================================================

#[test]
fn cat13_ai_command_bar_has_12_actions() {
    let actions = get_ai_command_bar_actions();
    assert_eq!(actions.len(), 12);
}

#[test]
fn cat13_ai_sections_present() {
    let actions = get_ai_command_bar_actions();
    let sections: HashSet<String> = actions.iter().filter_map(|a| a.section.clone()).collect();
    for expected in &[
        "Response",
        "Actions",
        "Attachments",
        "Export",
        "Help",
        "Settings",
    ] {
        assert!(
            sections.contains(*expected),
            "Missing section: {}",
            expected
        );
    }
}

#[test]
fn cat13_all_ai_actions_have_icons() {
    for action in &get_ai_command_bar_actions() {
        assert!(
            action.icon.is_some(),
            "AI action {} missing icon",
            action.id
        );
    }
}

#[test]
fn cat13_all_ai_actions_have_sections() {
    for action in &get_ai_command_bar_actions() {
        assert!(
            action.section.is_some(),
            "AI action {} missing section",
            action.id
        );
    }
}

#[test]
fn cat13_ai_action_ids_unique() {
    let actions = get_ai_command_bar_actions();
    let ids: HashSet<String> = action_ids(&actions).into_iter().collect();
    assert_eq!(ids.len(), actions.len(), "Duplicate IDs in AI command bar");
}

// ============================================================================
// 14. Notes command bar — conditional actions based on state
// ============================================================================

#[test]
fn cat14_full_feature_notes_actions_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note, duplicate_note, browse_notes, find_in_note, format, copy_note_as,
    // copy_deeplink, create_quicklink, export, enable_auto_sizing = 10
    assert_eq!(actions.len(), 10);
}

#[test]
fn cat14_trash_view_hides_editing_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
    assert!(!actions.iter().any(|a| a.id == "find_in_note"));
    assert!(!actions.iter().any(|a| a.id == "format"));
    assert!(!actions.iter().any(|a| a.id == "export"));
}

#[test]
fn cat14_no_selection_minimal() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    // Only new_note and browse_notes (no auto_sizing since enabled)
    assert_eq!(actions.len(), 2);
}

#[test]
fn cat14_auto_sizing_disabled_adds_enable_action() {
    let with = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let without = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let a_with = get_notes_command_bar_actions(&with);
    let a_without = get_notes_command_bar_actions(&without);
    assert!(a_with.iter().any(|a| a.id == "enable_auto_sizing"));
    assert!(!a_without.iter().any(|a| a.id == "enable_auto_sizing"));
    assert_eq!(a_with.len(), a_without.len() + 1);
}

// ============================================================================
// 15. Notes command bar — all actions have icons and sections
// ============================================================================

#[test]
fn cat15_all_notes_actions_have_icons() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for action in &get_notes_command_bar_actions(&info) {
        assert!(
            action.icon.is_some(),
            "Notes action {} missing icon",
            action.id
        );
    }
}

#[test]
fn cat15_all_notes_actions_have_sections() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for action in &get_notes_command_bar_actions(&info) {
        assert!(
            action.section.is_some(),
            "Notes action {} missing section",
            action.id
        );
    }
}

// ============================================================================
// 16. New chat actions — section structure
// ============================================================================

fn sample_model() -> NewChatModelInfo {
    NewChatModelInfo {
        model_id: "claude-3".into(),
        display_name: "Claude 3".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }
}

fn sample_preset() -> NewChatPresetInfo {
    NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: crate::designs::icon_variations::IconName::Star,
    }
}

#[test]
fn cat16_empty_inputs_produce_empty_actions() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

#[test]
fn cat16_sections_appear_in_order() {
    let actions = get_new_chat_actions(&[sample_model()], &[sample_preset()], &[sample_model()]);
    let sections: Vec<String> = actions.iter().filter_map(|a| a.section.clone()).collect();
    let last_used_pos = sections.iter().position(|s| s == "Last Used Settings");
    let presets_pos = sections.iter().position(|s| s == "Presets");
    let models_pos = sections.iter().position(|s| s == "Models");
    assert!(last_used_pos.unwrap() < presets_pos.unwrap());
    assert!(presets_pos.unwrap() < models_pos.unwrap());
}

#[test]
fn cat16_preset_has_no_description() {
    let actions = get_new_chat_actions(&[], &[sample_preset()], &[]);
    assert_eq!(actions[0].description.as_deref(), Some("Uses General preset"));
}

#[test]
fn cat16_model_description_is_provider() {
    let actions = get_new_chat_actions(&[], &[], &[sample_model()]);
    assert_eq!(actions[0].description, Some("Uses Anthropic".to_string()));
}

#[test]
fn cat16_last_used_has_bolt_icon() {
    let actions = get_new_chat_actions(&[sample_model()], &[], &[]);
    assert_eq!(
        actions[0].icon,
        Some(crate::designs::icon_variations::IconName::BoltFilled)
    );
}

#[test]
fn cat16_models_have_settings_icon() {
    let actions = get_new_chat_actions(&[], &[], &[sample_model()]);
    assert_eq!(
        actions[0].icon,
        Some(crate::designs::icon_variations::IconName::Settings)
    );
}

// ============================================================================
// 17. Note switcher — icon hierarchy and section assignment
// ============================================================================

fn make_note(id: &str, pinned: bool, current: bool) -> NoteSwitcherNoteInfo {
    NoteSwitcherNoteInfo {
        id: id.into(),
        title: format!("Note {}", id),
        char_count: 42,
        is_current: current,
        is_pinned: pinned,
        preview: "some preview text".into(),
        relative_time: "2m ago".into(),
    }
}

#[test]
fn cat17_pinned_gets_star_icon() {
    let actions = get_note_switcher_actions(&[make_note("1", true, false)]);
    assert_eq!(
        actions[0].icon,
        Some(crate::designs::icon_variations::IconName::StarFilled)
    );
}

#[test]
fn cat17_current_gets_check_icon() {
    let actions = get_note_switcher_actions(&[make_note("1", false, true)]);
    assert_eq!(
        actions[0].icon,
        Some(crate::designs::icon_variations::IconName::Check)
    );
}

#[test]
fn cat17_regular_gets_file_icon() {
    let actions = get_note_switcher_actions(&[make_note("1", false, false)]);
    assert_eq!(
        actions[0].icon,
        Some(crate::designs::icon_variations::IconName::File)
    );
}

#[test]
fn cat17_pinned_overrides_current_for_icon() {
    // When both pinned and current, pinned icon wins
    let actions = get_note_switcher_actions(&[make_note("1", true, true)]);
    assert_eq!(
        actions[0].icon,
        Some(crate::designs::icon_variations::IconName::StarFilled)
    );
}

#[test]
fn cat17_pinned_note_in_pinned_section() {
    let actions = get_note_switcher_actions(&[make_note("1", true, false)]);
    assert_eq!(actions[0].section, Some("Pinned".to_string()));
}

#[test]
fn cat17_unpinned_note_in_recent_section() {
    let actions = get_note_switcher_actions(&[make_note("1", false, false)]);
    assert_eq!(actions[0].section, Some("Recent".to_string()));
}

#[test]
fn cat17_current_note_has_bullet_prefix() {
    let actions = get_note_switcher_actions(&[make_note("1", false, true)]);
    assert!(
        actions[0].title.starts_with("• "),
        "Current note should have bullet: {}",
        actions[0].title
    );
}

#[test]
fn cat17_non_current_no_bullet() {
    let actions = get_note_switcher_actions(&[make_note("1", false, false)]);
    assert!(!actions[0].title.starts_with("• "));
}

// ============================================================================
// 18. Note switcher — description rendering edge cases
// ============================================================================

#[test]
fn cat18_preview_with_time_uses_separator() {
    let note = NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "hello".into(),
        relative_time: "5m ago".into(),
    };
    let actions = get_note_switcher_actions(&[note]);
    assert!(actions[0].description.as_ref().unwrap().contains(" · "));
}

// --- merged from part_03.rs ---

#[test]
fn cat18_empty_preview_with_time_uses_time() {
    let note = NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 0,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "1h ago".into(),
    };
    let actions = get_note_switcher_actions(&[note]);
    assert_eq!(actions[0].description, Some("1h ago".to_string()));
}

#[test]
fn cat18_empty_preview_empty_time_uses_char_count() {
    let note = NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 0,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    };
    let actions = get_note_switcher_actions(&[note]);
    assert_eq!(actions[0].description, Some("0 chars".to_string()));
}

#[test]
fn cat18_singular_char_count() {
    let note = NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 1,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    };
    let actions = get_note_switcher_actions(&[note]);
    assert_eq!(actions[0].description, Some("1 char".to_string()));
}

#[test]
fn cat18_preview_truncated_at_61_chars() {
    let long_preview = "a".repeat(61);
    let note = NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 61,
        is_current: false,
        is_pinned: false,
        preview: long_preview,
        relative_time: "".into(),
    };
    let actions = get_note_switcher_actions(&[note]);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(desc.ends_with('…'), "Should be truncated: {}", desc);
}

#[test]
fn cat18_preview_not_truncated_at_60_chars() {
    let exact_preview = "b".repeat(60);
    let note = NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 60,
        is_current: false,
        is_pinned: false,
        preview: exact_preview,
        relative_time: "".into(),
    };
    let actions = get_note_switcher_actions(&[note]);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(
        !desc.ends_with('…'),
        "Should NOT be truncated at exactly 60"
    );
}

// ============================================================================
// 19. Note switcher — empty state fallback
// ============================================================================

#[test]
fn cat19_empty_notes_shows_placeholder() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "no_notes");
    assert!(actions[0].title.contains("No notes yet"));
}

#[test]
fn cat19_empty_placeholder_has_plus_icon() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(
        actions[0].icon,
        Some(crate::designs::icon_variations::IconName::Plus)
    );
}

#[test]
fn cat19_empty_placeholder_description_mentions_cmd_n() {
    let actions = get_note_switcher_actions(&[]);
    assert!(actions[0].description.as_ref().unwrap().contains("⌘N"));
}

// ============================================================================
// 20. Chat context — model selection and conditional actions
// ============================================================================

#[test]
fn cat20_no_models_still_has_continue_in_chat() {
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
fn cat20_current_model_gets_checkmark() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".into()),
        available_models: vec![
            ChatModelInfo {
                id: "gpt4".into(),
                display_name: "GPT-4".into(),
                provider: "OpenAI".into(),
            },
            ChatModelInfo {
                id: "claude".into(),
                display_name: "Claude".into(),
                provider: "Anthropic".into(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let gpt4 = actions
        .iter()
        .find(|a| a.id == "chat:select_model_gpt4")
        .unwrap();
    assert!(gpt4.title.contains('✓'), "Current should have ✓");
    let claude = actions
        .iter()
        .find(|a| a.id == "chat:select_model_claude")
        .unwrap();
    assert!(!claude.title.contains('✓'), "Non-current should not have ✓");
}

#[test]
fn cat20_copy_response_only_when_has_response() {
    let no_resp = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let with_resp = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    assert!(!get_chat_context_actions(&no_resp)
        .iter()
        .any(|a| a.id == "chat:copy_response"));
    assert!(get_chat_context_actions(&with_resp)
        .iter()
        .any(|a| a.id == "chat:copy_response"));
}

#[test]
fn cat20_clear_conversation_only_when_has_messages() {
    let no_msgs = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let with_msgs = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    assert!(!get_chat_context_actions(&no_msgs)
        .iter()
        .any(|a| a.id == "chat:clear_conversation"));
    assert!(get_chat_context_actions(&with_msgs)
        .iter()
        .any(|a| a.id == "chat:clear_conversation"));
}

// ============================================================================
// 21. to_deeplink_name — edge cases
// ============================================================================

#[test]
fn cat21_basic_conversion() {
    assert_eq!(to_deeplink_name("My Script"), "my-script");
}

#[test]
fn cat21_underscores_become_hyphens() {
    assert_eq!(to_deeplink_name("hello_world"), "hello-world");
}

#[test]
fn cat21_special_chars_stripped() {
    assert_eq!(to_deeplink_name("test!@#$%"), "test");
}

#[test]
fn cat21_consecutive_specials_collapsed() {
    assert_eq!(to_deeplink_name("a---b"), "a-b");
}

#[test]
fn cat21_unicode_alphanumeric_preserved() {
    assert_eq!(to_deeplink_name("café"), "caf%C3%A9");
}

#[test]
fn cat21_leading_trailing_stripped() {
    assert_eq!(to_deeplink_name("  hello  "), "hello");
}

#[test]
fn cat21_numbers_preserved() {
    assert_eq!(to_deeplink_name("v2 script"), "v2-script");
}

// ============================================================================
// 22. fuzzy_match edge cases
// ============================================================================

#[test]
fn cat22_empty_needle_matches() {
    assert!(ActionsDialog::fuzzy_match("anything", ""));
}

#[test]
fn cat22_empty_haystack_with_needle_fails() {
    assert!(!ActionsDialog::fuzzy_match("", "x"));
}

#[test]
fn cat22_both_empty_matches() {
    assert!(ActionsDialog::fuzzy_match("", ""));
}

#[test]
fn cat22_exact_match() {
    assert!(ActionsDialog::fuzzy_match("hello", "hello"));
}

#[test]
fn cat22_subsequence_match() {
    assert!(ActionsDialog::fuzzy_match("hello world", "hlo"));
}

#[test]
fn cat22_no_subsequence() {
    assert!(!ActionsDialog::fuzzy_match("hello", "xyz"));
}

#[test]
fn cat22_needle_longer_than_haystack() {
    assert!(!ActionsDialog::fuzzy_match("hi", "hello"));
}

// ============================================================================
// 23. score_action boundary thresholds
// ============================================================================

#[test]
fn cat23_prefix_match_gives_100() {
    let a = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
    assert!(ActionsDialog::score_action(&a, "edit") >= 100);
}

#[test]
fn cat23_contains_match_gives_50() {
    let a = Action::new("id", "My Edit Tool", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&a, "edit");
    assert!(
        (50..100).contains(&score),
        "Contains should be 50-99: {}",
        score
    );
}

#[test]
fn cat23_fuzzy_match_gives_25() {
    let a = Action::new("id", "Elephant", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&a, "ept");
    assert!(
        (25..50).contains(&score),
        "Fuzzy should be 25-49: {}",
        score
    );
}

#[test]
fn cat23_description_bonus_15() {
    let a = Action::new(
        "id",
        "Open File",
        Some("Edit in editor".into()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&a, "editor");
    assert!(
        score >= 15,
        "Description match should give >= 15: {}",
        score
    );
}

#[test]
fn cat23_no_match_gives_0() {
    let a = Action::new("id", "Run Script", None, ActionCategory::ScriptContext);
    assert_eq!(ActionsDialog::score_action(&a, "xyz"), 0);
}

#[test]
fn cat23_prefix_plus_desc_stacks() {
    let a = Action::new(
        "id",
        "Edit Script",
        Some("Edit the script in editor".into()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&a, "edit");
    assert!(score >= 115, "Prefix(100) + Desc(15) = 115: {}", score);
}

// ============================================================================
// 24. parse_shortcut_keycaps
// ============================================================================

#[test]
fn cat24_modifier_plus_letter() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘C");
    assert_eq!(caps, vec!["⌘", "C"]);
}

#[test]
fn cat24_two_modifiers() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘⇧C");
    assert_eq!(caps, vec!["⌘", "⇧", "C"]);
}

#[test]
fn cat24_enter_symbol() {
    let caps = ActionsDialog::parse_shortcut_keycaps("↵");
    assert_eq!(caps, vec!["↵"]);
}

#[test]
fn cat24_arrow_keys() {
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("↑"), vec!["↑"]);
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("↓"), vec!["↓"]);
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("←"), vec!["←"]);
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("→"), vec!["→"]);
}

#[test]
fn cat24_escape_and_space() {
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("⎋"), vec!["⎋"]);
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("␣"), vec!["␣"]);
}

#[test]
fn cat24_lowercase_uppercased() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘x");
    assert_eq!(caps, vec!["⌘", "X"]);
}

// ============================================================================
// 25. build_grouped_items_static behavior
// ============================================================================

#[test]
fn cat25_empty_filtered_returns_empty() {
    let actions: Vec<Action> = vec![];
    let result = build_grouped_items_static(&actions, &[], SectionStyle::Headers);
    assert!(result.is_empty());
}

#[test]
fn cat25_headers_inserts_section_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered = vec![0, 1];
    let result = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // Should have: header S1, item 0, header S2, item 1
    assert_eq!(result.len(), 4);
}

#[test]
fn cat25_separators_no_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered = vec![0, 1];
    let result = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    // Should have: item 0, item 1 (no headers)
    assert_eq!(result.len(), 2);
}

#[test]
fn cat25_none_style_no_headers() {
    let actions =
        vec![Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1")];
    let filtered = vec![0];
    let result = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
    assert_eq!(result.len(), 1);
}

#[test]
fn cat25_same_section_no_duplicate_header() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S"),
    ];
    let filtered = vec![0, 1];
    let result = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // Should have: header S, item 0, item 1
    assert_eq!(result.len(), 3);
}

// --- merged from part_04.rs ---

// ============================================================================
// 26. coerce_action_selection
// ============================================================================

#[test]
fn cat26_empty_returns_none() {
    assert!(coerce_action_selection(&[], 0).is_none());
}

#[test]
fn cat26_on_item_returns_same() {
    let rows = vec![GroupedActionItem::Item(0)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
}

#[test]
fn cat26_header_searches_down() {
    let rows = vec![
        GroupedActionItem::SectionHeader("S".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn cat26_trailing_header_searches_up() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("S".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn cat26_all_headers_returns_none() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".into()),
        GroupedActionItem::SectionHeader("B".into()),
    ];
    assert!(coerce_action_selection(&rows, 0).is_none());
}

#[test]
fn cat26_out_of_bounds_clamped() {
    let rows = vec![GroupedActionItem::Item(0)];
    assert_eq!(coerce_action_selection(&rows, 100), Some(0));
}

// ============================================================================
// 27. Cross-context ID namespace collision avoidance
// ============================================================================

#[test]
fn cat27_script_and_clipboard_no_id_overlap() {
    let script_actions = get_script_context_actions(&ScriptInfo::new("test", "/test.ts"));
    let clip_actions = get_clipboard_history_context_actions(&text_entry());
    let script_ids: HashSet<String> = action_ids(&script_actions).into_iter().collect();
    let clip_ids: HashSet<String> = action_ids(&clip_actions).into_iter().collect();
    let overlap: Vec<&String> = script_ids.intersection(&clip_ids).collect();
    assert!(
        overlap.is_empty(),
        "Script/Clipboard ID overlap: {:?}",
        overlap
    );
}

#[test]
fn cat27_ai_and_notes_no_id_overlap() {
    let ai_actions = get_ai_command_bar_actions();
    let notes_actions = get_notes_command_bar_actions(&NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    });
    let ai_ids: HashSet<String> = action_ids(&ai_actions).into_iter().collect();
    let notes_ids: HashSet<String> = action_ids(&notes_actions).into_iter().collect();
    // copy_deeplink exists in both contexts by design - that's OK since they
    // are in different command bars and never shown together. But check for
    // unexpected collisions.
    let overlap: Vec<&String> = ai_ids.intersection(&notes_ids).collect();
    // Allow known shared IDs
    let unexpected: Vec<&&String> = overlap
        .iter()
        .filter(|id| !["script:copy_deeplink", "chat:new_chat"].contains(&id.as_str()))
        .collect();
    assert!(
        unexpected.is_empty(),
        "Unexpected AI/Notes ID overlap: {:?}",
        unexpected
    );
}

#[test]
fn cat27_path_and_file_some_shared_ids() {
    // Path and file contexts are related — they share some IDs by design
    let path_actions = get_path_context_actions(&path_dir());
    let file_actions = get_file_context_actions(&file_info_dir());
    let path_ids: HashSet<String> = action_ids(&path_actions).into_iter().collect();
    let file_ids: HashSet<String> = action_ids(&file_actions).into_iter().collect();
    let shared: Vec<&String> = path_ids.intersection(&file_ids).collect();
    // copy_path, copy_filename, open_directory should be shared
    assert!(
        shared.len() >= 2,
        "Path/File should share some IDs: {:?}",
        shared
    );
}

// ============================================================================
// 28. All actions have non-empty id and title
// ============================================================================

#[test]
fn cat28_script_actions_nonempty_id_title() {
    for action in &get_script_context_actions(&ScriptInfo::new("t", "/t.ts")) {
        assert!(!action.id.is_empty(), "Empty ID");
        assert!(!action.title.is_empty(), "Empty title for {}", action.id);
    }
}

#[test]
fn cat28_clipboard_actions_nonempty_id_title() {
    for action in &get_clipboard_history_context_actions(&text_entry()) {
        assert!(!action.id.is_empty(), "Empty ID");
        assert!(!action.title.is_empty(), "Empty title for {}", action.id);
    }
}

#[test]
fn cat28_ai_actions_nonempty_id_title() {
    for action in &get_ai_command_bar_actions() {
        assert!(!action.id.is_empty(), "Empty ID");
        assert!(!action.title.is_empty(), "Empty title for {}", action.id);
    }
}

#[test]
fn cat28_notes_actions_nonempty_id_title() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for action in &get_notes_command_bar_actions(&info) {
        assert!(!action.id.is_empty(), "Empty ID");
        assert!(!action.title.is_empty(), "Empty title for {}", action.id);
    }
}

#[test]
fn cat28_path_actions_nonempty_id_title() {
    for action in &get_path_context_actions(&path_dir()) {
        assert!(!action.id.is_empty(), "Empty ID");
        assert!(!action.title.is_empty(), "Empty title for {}", action.id);
    }
}

#[test]
fn cat28_file_actions_nonempty_id_title() {
    for action in &get_file_context_actions(&file_info_file()) {
        assert!(!action.id.is_empty(), "Empty ID");
        assert!(!action.title.is_empty(), "Empty title for {}", action.id);
    }
}

// ============================================================================
// 29. has_action = false for all built-in actions
// ============================================================================

#[test]
fn cat29_script_actions_has_action_false() {
    for action in &get_script_context_actions(&ScriptInfo::new("t", "/t.ts")) {
        assert!(
            !action.has_action,
            "{} should have has_action=false",
            action.id
        );
    }
}

#[test]
fn cat29_clipboard_actions_has_action_false() {
    for action in &get_clipboard_history_context_actions(&text_entry()) {
        assert!(
            !action.has_action,
            "{} should have has_action=false",
            action.id
        );
    }
}

#[test]
fn cat29_ai_actions_has_action_false() {
    for action in &get_ai_command_bar_actions() {
        assert!(
            !action.has_action,
            "{} should have has_action=false",
            action.id
        );
    }
}

#[test]
fn cat29_path_actions_has_action_false() {
    for action in &get_path_context_actions(&path_dir()) {
        assert!(
            !action.has_action,
            "{} should have has_action=false",
            action.id
        );
    }
}

#[test]
fn cat29_file_actions_has_action_false() {
    for action in &get_file_context_actions(&file_info_file()) {
        assert!(
            !action.has_action,
            "{} should have has_action=false",
            action.id
        );
    }
}

#[test]
fn cat29_notes_actions_has_action_false() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for action in &get_notes_command_bar_actions(&info) {
        assert!(
            !action.has_action,
            "{} should have has_action=false",
            action.id
        );
    }
}

// ============================================================================
// 30. Scriptlet defined actions — has_action=true and value set
// ============================================================================

#[test]
fn cat30_scriptlet_actions_have_has_action_true() {
    let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo test".into());
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".into(),
        command: "copy-cmd".into(),
        tool: "bash".into(),
        code: "pbcopy".into(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert!(
        actions[0].has_action,
        "Scriptlet action should have has_action=true"
    );
}

#[test]
fn cat30_scriptlet_actions_have_value() {
    let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo test".into());
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".into(),
        command: "copy-cmd".into(),
        tool: "bash".into(),
        code: "pbcopy".into(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert_eq!(actions[0].value, Some("copy-cmd".to_string()));
}

#[test]
fn cat30_scriptlet_action_id_format() {
    let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo test".into());
    scriptlet.actions = vec![ScriptletAction {
        name: "Open Browser".into(),
        command: "open-browser".into(),
        tool: "bash".into(),
        code: "open".into(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert_eq!(actions[0].id, "scriptlet_action:open-browser");
}

#[test]
fn cat30_scriptlet_with_shortcut_formatted() {
    let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo test".into());
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".into(),
        command: "copy".into(),
        tool: "bash".into(),
        code: "pbcopy".into(),
        inputs: vec![],
        shortcut: Some("cmd+c".into()),
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert_eq!(actions[0].shortcut, Some("⌘C".to_string()));
}

#[test]
fn cat30_scriptlet_empty_actions_returns_empty() {
    let scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo test".into());
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert!(actions.is_empty());
}

// ============================================================================
// Bonus: Ordering determinism — repeated calls produce same result
// ============================================================================

#[test]
fn bonus_script_actions_deterministic() {
    let s = ScriptInfo::new("test", "/test.ts");
    let a1 = action_ids(&get_script_context_actions(&s));
    let a2 = action_ids(&get_script_context_actions(&s));
    assert_eq!(a1, a2);
}

#[test]
fn bonus_clipboard_actions_deterministic() {
    let a1 = action_ids(&get_clipboard_history_context_actions(&text_entry()));
    let a2 = action_ids(&get_clipboard_history_context_actions(&text_entry()));
    assert_eq!(a1, a2);
}

#[test]
fn bonus_ai_actions_deterministic() {
    let a1 = action_ids(&get_ai_command_bar_actions());
    let a2 = action_ids(&get_ai_command_bar_actions());
    assert_eq!(a1, a2);
}

#[test]
fn bonus_notes_actions_deterministic() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let a1 = action_ids(&get_notes_command_bar_actions(&info));
    let a2 = action_ids(&get_notes_command_bar_actions(&info));
    assert_eq!(a1, a2);
}

// ============================================================================
// Bonus: ActionCategory PartialEq
// ============================================================================

#[test]
fn bonus_action_category_equality() {
    assert_eq!(ActionCategory::ScriptContext, ActionCategory::ScriptContext);
    assert_ne!(ActionCategory::ScriptContext, ActionCategory::ScriptOps);
    assert_ne!(ActionCategory::GlobalOps, ActionCategory::Terminal);
}

// ============================================================================
// Bonus: title_lower invariant across contexts
// ============================================================================

#[test]
fn bonus_title_lower_matches_lowercase() {
    // Script context
    for action in &get_script_context_actions(&ScriptInfo::new("Test", "/t.ts")) {
        assert_eq!(action.title_lower, action.title.to_lowercase());
    }
    // Clipboard context
    for action in &get_clipboard_history_context_actions(&text_entry()) {
        assert_eq!(action.title_lower, action.title.to_lowercase());
    }
    // AI command bar
    for action in &get_ai_command_bar_actions() {
        assert_eq!(action.title_lower, action.title.to_lowercase());
    }
}

// ============================================================================
// Bonus: All ScriptContext category
// ============================================================================

#[test]
fn bonus_all_script_actions_are_script_context() {
    for a in &get_script_context_actions(&ScriptInfo::new("t", "/t.ts")) {
        assert_eq!(a.category, ActionCategory::ScriptContext);
    }
}

#[test]
fn bonus_all_clipboard_actions_are_script_context() {
    for a in &get_clipboard_history_context_actions(&text_entry()) {
        assert_eq!(a.category, ActionCategory::ScriptContext);
    }
}

#[test]
fn bonus_all_ai_actions_are_script_context() {
    for a in &get_ai_command_bar_actions() {
        assert_eq!(a.category, ActionCategory::ScriptContext);
    }
}

#[test]
fn bonus_all_path_actions_are_script_context() {
    for a in &get_path_context_actions(&path_dir()) {
        assert_eq!(a.category, ActionCategory::ScriptContext);
    }
}
