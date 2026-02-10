// Batch 11: Random builtin action/dialog validation tests
//
// 30 test categories covering fresh angles on action builders, edge cases,
// and behavioral invariants not thoroughly covered by batches 1-10.

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
    assert!(!text_actions.iter().any(|a| a.id == "clipboard_ocr"));
    assert!(image_actions.iter().any(|a| a.id == "clipboard_ocr"));
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
        assert_eq!(actions[len - 3].id, "clipboard_delete");
        assert_eq!(actions[len - 2].id, "clipboard_delete_multiple");
        assert_eq!(actions[len - 1].id, "clipboard_delete_all");
    }
}

#[test]
fn cat06_paste_is_always_first() {
    for entry in &[text_entry(), image_entry()] {
        let actions = get_clipboard_history_context_actions(entry);
        assert_eq!(actions[0].id, "clipboard_paste");
    }
}

// ============================================================================
// 7. Clipboard pin/unpin dynamic toggle
// ============================================================================

#[test]
fn cat07_unpinned_shows_pin() {
    let actions = get_clipboard_history_context_actions(&text_entry());
    assert!(actions.iter().any(|a| a.id == "clipboard_pin"));
    assert!(!actions.iter().any(|a| a.id == "clipboard_unpin"));
}

#[test]
fn cat07_pinned_shows_unpin() {
    let mut e = text_entry();
    e.pinned = true;
    let actions = get_clipboard_history_context_actions(&e);
    assert!(!actions.iter().any(|a| a.id == "clipboard_pin"));
    assert!(actions.iter().any(|a| a.id == "clipboard_unpin"));
}

#[test]
fn cat07_pin_unpin_share_same_shortcut() {
    let pin_actions = get_clipboard_history_context_actions(&text_entry());
    let mut pinned = text_entry();
    pinned.pinned = true;
    let unpin_actions = get_clipboard_history_context_actions(&pinned);
    let pin_sc = pin_actions
        .iter()
        .find(|a| a.id == "clipboard_pin")
        .unwrap()
        .shortcut
        .as_ref()
        .unwrap();
    let unpin_sc = unpin_actions
        .iter()
        .find(|a| a.id == "clipboard_unpin")
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
    let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
    assert_eq!(paste.title, "Paste to Active App");
}

#[test]
fn cat08_with_app_shows_name() {
    let mut e = text_entry();
    e.frontmost_app_name = Some("Firefox".into());
    let actions = get_clipboard_history_context_actions(&e);
    let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
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
    assert!(actions.iter().any(|a| a.id == "open_file"));
    assert!(!actions.iter().any(|a| a.id == "open_directory"));
}

#[test]
fn cat09_dir_has_open_directory_not_open_file() {
    let actions = get_file_context_actions(&file_info_dir());
    assert!(actions.iter().any(|a| a.id == "open_directory"));
    assert!(!actions.iter().any(|a| a.id == "open_file"));
}
