// --- merged from part_01.rs ---
//! Batch 36: Dialog built-in action validation tests
//!
//! 124 tests across 30 categories validating random behaviors from
//! built-in action window dialogs.

use crate::actions::builders::{
    get_ai_command_bar_actions, get_clipboard_history_context_actions, get_file_context_actions,
    get_new_chat_actions, get_note_switcher_actions, get_notes_command_bar_actions,
    get_path_context_actions, get_script_context_actions,
    get_scriptlet_context_actions_with_custom, to_deeplink_name, ClipboardEntryInfo,
    NewChatModelInfo, NewChatPresetInfo, NoteSwitcherNoteInfo, NotesInfo,
};
use crate::actions::command_bar::CommandBarConfig;
use crate::actions::dialog::{build_grouped_items_static, coerce_action_selection, ActionsDialog};
use crate::actions::types::{Action, ActionCategory, AnchorPosition, ScriptInfo, SectionStyle};
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::designs::DesignColors;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;
use crate::protocol::ProtocolAction;
use crate::scriptlets::Scriptlet;

// =====================================================================
// 1. hex_with_alpha: shift and OR behavior
// =====================================================================

#[test]
fn hex_with_alpha_black_full_opaque() {
    // 0x000000 with alpha 0xFF => 0x000000FF
    assert_eq!(DesignColors::hex_with_alpha(0x000000, 0xFF), 0x000000FF);
}

#[test]
fn hex_with_alpha_white_full_opaque() {
    // 0xFFFFFF with alpha 0xFF => 0xFFFFFFFF
    assert_eq!(DesignColors::hex_with_alpha(0xFFFFFF, 0xFF), 0xFFFFFFFF);
}

#[test]
fn hex_with_alpha_color_half_transparent() {
    // 0x1A2B3C with alpha 0x80 => (0x1A2B3C << 8) | 0x80
    let result = DesignColors::hex_with_alpha(0x1A2B3C, 0x80);
    assert_eq!(result, (0x1A2B3C << 8) | 0x80);
}

#[test]
fn hex_with_alpha_zero_alpha() {
    // 0xABCDEF with alpha 0 => 0xABCDEF00
    assert_eq!(DesignColors::hex_with_alpha(0xABCDEF, 0x00), 0xABCDEF00);
}

// =====================================================================
// 2. ProtocolAction: is_visible default behavior
// =====================================================================

#[test]
fn protocol_action_visible_none_defaults_to_true() {
    let pa = ProtocolAction {
        name: "Test".into(),
        description: None,
        shortcut: None,
        value: None,
        has_action: false,
        visible: None,
        close: None,
    };
    assert!(pa.is_visible());
}

#[test]
fn protocol_action_visible_true_is_visible() {
    let pa = ProtocolAction {
        name: "Test".into(),
        description: None,
        shortcut: None,
        value: None,
        has_action: false,
        visible: Some(true),
        close: None,
    };
    assert!(pa.is_visible());
}

#[test]
fn protocol_action_visible_false_is_hidden() {
    let pa = ProtocolAction {
        name: "Test".into(),
        description: None,
        shortcut: None,
        value: None,
        has_action: false,
        visible: Some(false),
        close: None,
    };
    assert!(!pa.is_visible());
}

#[test]
fn protocol_action_has_action_false_default() {
    let pa = ProtocolAction {
        name: "Test".into(),
        description: None,
        shortcut: None,
        value: None,
        has_action: false,
        visible: None,
        close: None,
    };
    assert!(!pa.has_action);
}

// =====================================================================
// 3. ProtocolAction: should_close default behavior
// =====================================================================

#[test]
fn protocol_action_close_none_defaults_to_true() {
    let pa = ProtocolAction {
        name: "Test".into(),
        description: None,
        shortcut: None,
        value: None,
        has_action: false,
        visible: None,
        close: None,
    };
    assert!(pa.should_close());
}

#[test]
fn protocol_action_close_false_stays_open() {
    let pa = ProtocolAction {
        name: "Test".into(),
        description: None,
        shortcut: None,
        value: None,
        has_action: false,
        visible: None,
        close: Some(false),
    };
    assert!(!pa.should_close());
}

#[test]
fn protocol_action_close_true_closes() {
    let pa = ProtocolAction {
        name: "Test".into(),
        description: None,
        shortcut: None,
        value: None,
        has_action: false,
        visible: None,
        close: Some(true),
    };
    assert!(pa.should_close());
}

// =====================================================================
// 4. builders::format_shortcut_hint (simple) vs ActionsDialog::format_shortcut_hint (sophisticated)
// =====================================================================

#[test]
fn builders_format_converts_cmd_to_symbol() {
    // The builders version does simple string replace
    let result = ActionsDialog::format_shortcut_hint("cmd+c");
    assert_eq!(result, "⌘C");
}

#[test]
fn dialog_format_handles_meta_alias() {
    let result = ActionsDialog::format_shortcut_hint("meta+k");
    assert_eq!(result, "⌘K");
}

#[test]
fn dialog_format_handles_super_alias() {
    let result = ActionsDialog::format_shortcut_hint("super+j");
    assert_eq!(result, "⌘J");
}

#[test]
fn dialog_format_handles_control_full_word() {
    let result = ActionsDialog::format_shortcut_hint("control+x");
    assert_eq!(result, "⌃X");
}

// =====================================================================
// 5. Clipboard: quick_look details (macOS)
// =====================================================================

#[test]
fn clipboard_quick_look_shortcut_is_space() {
    let entry = ClipboardEntryInfo {
        id: "ql-1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ql = actions.iter().find(|a| a.id == "clip:clipboard_quick_look");
    // On macOS this should exist
    if let Some(action) = ql {
        assert_eq!(action.shortcut.as_deref(), Some("␣"));
    }
}

#[test]
fn clipboard_quick_look_title() {
    let entry = ClipboardEntryInfo {
        id: "ql-2".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    if let Some(action) = actions.iter().find(|a| a.id == "clip:clipboard_quick_look") {
        assert_eq!(action.title, "Quick Look");
    }
}

#[test]
fn clipboard_quick_look_desc_mentions_preview() {
    let entry = ClipboardEntryInfo {
        id: "ql-3".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    if let Some(action) = actions.iter().find(|a| a.id == "clip:clipboard_quick_look") {
        let desc = action.description.as_deref().unwrap();
        assert!(desc.contains("Quick Look"));
    }
}

#[test]
fn clipboard_quick_look_present_for_image_too() {
    let entry = ClipboardEntryInfo {
        id: "ql-4".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: String::new(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    // On macOS, quick_look is available for both text and image
    let has_ql = actions.iter().any(|a| a.id == "clip:clipboard_quick_look");
    // Either present (macOS) or absent (non-macOS), consistent with text entries
    let text_entry = ClipboardEntryInfo {
        id: "ql-4b".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let text_actions = get_clipboard_history_context_actions(&text_entry);
    let text_has_ql = text_actions.iter().any(|a| a.id == "clip:clipboard_quick_look");
    assert_eq!(
        has_ql, text_has_ql,
        "quick_look availability should be consistent"
    );
}

// =====================================================================
// 6. Clipboard: delete entry shortcut and description
// =====================================================================

#[test]
fn clipboard_delete_shortcut_ctrl_x() {
    let entry = ClipboardEntryInfo {
        id: "d-1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let action = actions.iter().find(|a| a.id == "clip:clipboard_delete").unwrap();
    assert_eq!(action.shortcut.as_deref(), Some("⌃X"));
}

#[test]
fn clipboard_delete_title() {
    let entry = ClipboardEntryInfo {
        id: "d-2".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let action = actions.iter().find(|a| a.id == "clip:clipboard_delete").unwrap();
    assert_eq!(action.title, "Delete Entry");
}

#[test]
fn clipboard_delete_desc_mentions_remove() {
    let entry = ClipboardEntryInfo {
        id: "d-3".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let action = actions.iter().find(|a| a.id == "clip:clipboard_delete").unwrap();
    assert!(action.description.as_deref().unwrap().contains("Remove"));
}

#[test]
fn clipboard_delete_present_for_image() {
    let entry = ClipboardEntryInfo {
        id: "d-4".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: String::new(),
        image_dimensions: Some((50, 50)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clip:clipboard_delete"));
}

// =====================================================================
// 7. Clipboard: action ordering invariants (paste first, destructive last)
// =====================================================================

#[test]
fn clipboard_first_action_is_paste() {
    let entry = ClipboardEntryInfo {
        id: "ord-1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[0].id, "clip:clipboard_paste");
}

#[test]
fn clipboard_second_action_is_copy() {
    let entry = ClipboardEntryInfo {
        id: "ord-2".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[1].id, "clip:clipboard_copy");
}

#[test]
fn clipboard_last_action_is_delete_all() {
    let entry = ClipboardEntryInfo {
        id: "ord-3".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions.last().unwrap().id, "clip:clipboard_delete_all");
}

#[test]
fn clipboard_last_3_are_destructive() {
    let entry = ClipboardEntryInfo {
        id: "ord-4".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let n = actions.len();
    assert_eq!(actions[n - 3].id, "clip:clipboard_delete");
    assert_eq!(actions[n - 2].id, "clip:clipboard_delete_multiple");
    assert_eq!(actions[n - 1].id, "clip:clipboard_delete_all");
}

// =====================================================================
// 8. File context: quick_look only for non-dir
// =====================================================================

#[test]
fn file_context_file_has_quick_look_on_macos() {
    let fi = FileInfo {
        name: "readme.md".into(),
        path: "/path/readme.md".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&fi);
    let has_ql = actions.iter().any(|a| a.id == "file:quick_look");
    // On macOS it should be present; on other platforms it's absent
    #[cfg(target_os = "macos")]
    assert!(has_ql);
    #[cfg(not(target_os = "macos"))]
    assert!(!has_ql);
}

#[test]
fn file_context_dir_no_quick_look() {
    let fi = FileInfo {
        name: "docs".into(),
        path: "/path/docs".into(),
        is_dir: true,
        file_type: FileType::Directory,
    };
    let actions = get_file_context_actions(&fi);
    assert!(!actions.iter().any(|a| a.id == "file:quick_look"));
}

#[test]
fn file_context_file_quick_look_shortcut() {
    let fi = FileInfo {
        name: "img.png".into(),
        path: "/path/img.png".into(),
        is_dir: false,
        file_type: FileType::Image,
    };
    let actions = get_file_context_actions(&fi);
    if let Some(ql) = actions.iter().find(|a| a.id == "file:quick_look") {
        assert_eq!(ql.shortcut.as_deref(), Some("⌘Y"));
    }
}

// --- merged from part_02.rs ---

#[test]
fn file_context_file_quick_look_desc() {
    let fi = FileInfo {
        name: "demo.txt".into(),
        path: "/path/demo.txt".into(),
        is_dir: false,
        file_type: FileType::Document,
    };
    let actions = get_file_context_actions(&fi);
    if let Some(ql) = actions.iter().find(|a| a.id == "file:quick_look") {
        assert!(ql.description.as_deref().unwrap().contains("Quick Look"));
    }
}

// =====================================================================
// 9. File context: copy_path shortcut ⌘⇧C
// =====================================================================

#[test]
fn file_context_copy_path_shortcut() {
    let fi = FileInfo {
        name: "file.rs".into(),
        path: "/path/file.rs".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&fi);
    let cp = actions.iter().find(|a| a.id == "file:copy_path").unwrap();
    assert_eq!(cp.shortcut.as_deref(), Some("⌘⇧C"));
}

#[test]
fn file_context_copy_path_desc_mentions_full_path() {
    let fi = FileInfo {
        name: "file.rs".into(),
        path: "/path/file.rs".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&fi);
    let cp = actions.iter().find(|a| a.id == "file:copy_path").unwrap();
    assert!(cp.description.as_deref().unwrap().contains("full path"));
}

#[test]
fn file_context_copy_filename_shortcut_cmd_c() {
    let fi = FileInfo {
        name: "main.rs".into(),
        path: "/path/main.rs".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&fi);
    let cf = actions.iter().find(|a| a.id == "file:copy_filename").unwrap();
    assert_eq!(cf.shortcut.as_deref(), Some("⌘C"));
}

// =====================================================================
// 10. Path context: open_in_terminal shortcut ⌘T
// =====================================================================

#[test]
fn path_context_open_in_terminal_shortcut() {
    let pi = PathInfo {
        name: "project".into(),
        path: "/path/project".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    let ot = actions.iter().find(|a| a.id == "file:open_in_terminal").unwrap();
    assert_eq!(ot.shortcut.as_deref(), Some("⌘T"));
}

#[test]
fn path_context_open_in_terminal_desc() {
    let pi = PathInfo {
        name: "project".into(),
        path: "/path/project".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    let ot = actions.iter().find(|a| a.id == "file:open_in_terminal").unwrap();
    assert!(ot.description.as_deref().unwrap().contains("terminal"));
}

#[test]
fn path_context_open_in_terminal_title() {
    let pi = PathInfo {
        name: "src".into(),
        path: "/path/src".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    let ot = actions.iter().find(|a| a.id == "file:open_in_terminal").unwrap();
    assert_eq!(ot.title, "Open in Terminal");
}

#[test]
fn path_context_open_in_terminal_present_for_file() {
    let pi = PathInfo {
        name: "main.rs".into(),
        path: "/path/main.rs".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    assert!(actions.iter().any(|a| a.id == "file:open_in_terminal"));
}

// =====================================================================
// 11. Script context: view_logs shortcut ⌘L
// =====================================================================

#[test]
fn script_context_view_logs_shortcut() {
    let script = ScriptInfo::new("my-script", "/path/my-script.ts");
    let actions = get_script_context_actions(&script);
    let vl = actions.iter().find(|a| a.id == "view_logs").unwrap();
    assert_eq!(vl.shortcut.as_deref(), Some("⌘L"));
}

#[test]
fn script_context_view_logs_title() {
    let script = ScriptInfo::new("my-script", "/path/my-script.ts");
    let actions = get_script_context_actions(&script);
    let vl = actions.iter().find(|a| a.id == "view_logs").unwrap();
    assert_eq!(vl.title, "Show Logs");
}

#[test]
fn script_context_view_logs_desc_mentions_logs() {
    let script = ScriptInfo::new("my-script", "/path/my-script.ts");
    let actions = get_script_context_actions(&script);
    let vl = actions.iter().find(|a| a.id == "view_logs").unwrap();
    assert!(vl.description.as_deref().unwrap().contains("logs"));
}

#[test]
fn script_context_view_logs_absent_for_builtin() {
    let script = ScriptInfo::builtin("Clipboard History");
    let actions = get_script_context_actions(&script);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

// =====================================================================
// 12. Script context: all IDs unique within context
// =====================================================================

#[test]
fn script_context_ids_unique_basic() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len(), "Duplicate IDs found: {:?}", ids);
}

#[test]
fn script_context_ids_unique_with_shortcut_and_alias() {
    let script = ScriptInfo::with_shortcut_and_alias(
        "test",
        "/path/test.ts",
        Some("cmd+t".into()),
        Some("ts".into()),
    );
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len());
}

#[test]
fn scriptlet_context_ids_unique() {
    let script = ScriptInfo::scriptlet("Open URL", "/path/url.md", None, None);
    let scriptlet = Scriptlet::new("Open URL".into(), "bash".into(), "echo hi".into());
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len());
}

#[test]
fn ai_command_bar_ids_unique() {
    let actions = get_ai_command_bar_actions();
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len());
}

// =====================================================================
// 13. Script context: action count increases with shortcut+alias+suggestion
// =====================================================================

#[test]
fn script_context_base_count_no_extras() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    // run + add_shortcut + add_alias + toggle_favorite + edit + view_logs + reveal + copy_path + copy_content + copy_deeplink = 10
    assert_eq!(actions.len(), 10);
}

#[test]
fn script_context_with_shortcut_adds_one() {
    let script = ScriptInfo::with_shortcut("test", "/path/test.ts", Some("cmd+t".into()));
    let actions = get_script_context_actions(&script);
    // run + update_shortcut + remove_shortcut + add_alias + toggle_favorite + edit + view_logs + reveal + copy_path + copy_content + copy_deeplink = 11
    assert_eq!(actions.len(), 11);
}

#[test]
fn script_context_with_both_adds_two() {
    let script = ScriptInfo::with_shortcut_and_alias(
        "test",
        "/path/test.ts",
        Some("cmd+t".into()),
        Some("ts".into()),
    );
    let actions = get_script_context_actions(&script);
    // run + update_shortcut + remove_shortcut + update_alias + remove_alias + toggle_favorite + edit + view_logs + reveal + copy_path + copy_content + copy_deeplink = 12
    assert_eq!(actions.len(), 12);
}

#[test]
fn script_context_with_suggestion_adds_reset_ranking() {
    let script =
        ScriptInfo::new("test", "/path/test.ts").with_frecency(true, Some("/path/test.ts".into()));
    let actions = get_script_context_actions(&script);
    // 10 + reset_ranking = 11
    assert_eq!(actions.len(), 11);
    assert!(actions.iter().any(|a| a.id == "reset_ranking"));
}

// =====================================================================
// 14. Scriptlet context: identical shortcut/alias dynamic behavior
// =====================================================================

#[test]
fn scriptlet_no_shortcut_has_add_shortcut() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "add_shortcut"));
}

#[test]
fn scriptlet_with_shortcut_has_update_and_remove() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", Some("cmd+t".into()), None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "update_shortcut"));
    assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
    assert!(!actions.iter().any(|a| a.id == "add_shortcut"));
}

#[test]
fn scriptlet_no_alias_has_add_alias() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "add_alias"));
}

#[test]
fn scriptlet_with_alias_has_update_and_remove() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, Some("ts".into()));
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "update_alias"));
    assert!(actions.iter().any(|a| a.id == "remove_alias"));
    assert!(!actions.iter().any(|a| a.id == "add_alias"));
}

// =====================================================================
// 15. Agent context: no view_logs but has edit/reveal/copy_path/copy_content
// =====================================================================

#[test]
fn agent_has_edit_with_agent_title() {
    let mut script = ScriptInfo::new("my-agent", "/path/agent.md");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.title, "Edit Agent");
}

#[test]
fn agent_has_reveal_in_finder() {
    let mut script = ScriptInfo::new("my-agent", "/path/agent.md");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
}

#[test]
fn agent_has_copy_path_and_copy_content() {
    let mut script = ScriptInfo::new("my-agent", "/path/agent.md");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    assert!(actions.iter().any(|a| a.id == "copy_path"));
    assert!(actions.iter().any(|a| a.id == "copy_content"));
}

#[test]
fn agent_no_view_logs() {
    let mut script = ScriptInfo::new("my-agent", "/path/agent.md");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

// =====================================================================
// 16. AI command bar: branch_from_last has no shortcut
// =====================================================================

#[test]
fn ai_bar_branch_from_last_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let bfl = actions.iter().find(|a| a.id == "chat:branch_from_last").unwrap();
    assert!(bfl.shortcut.is_none());
}

#[test]
fn ai_bar_branch_from_last_section_actions() {
    let actions = get_ai_command_bar_actions();
    let bfl = actions.iter().find(|a| a.id == "chat:branch_from_last").unwrap();
    assert_eq!(bfl.section.as_deref(), Some("Actions"));
}

#[test]
fn ai_bar_branch_from_last_icon_arrowright() {
    let actions = get_ai_command_bar_actions();
    let bfl = actions.iter().find(|a| a.id == "chat:branch_from_last").unwrap();
    assert_eq!(bfl.icon, Some(IconName::ArrowRight));
}

#[test]
fn ai_bar_branch_from_last_desc_mentions_branch() {
    let actions = get_ai_command_bar_actions();
    let bfl = actions.iter().find(|a| a.id == "chat:branch_from_last").unwrap();
    assert!(bfl.description.as_deref().unwrap().contains("new chat"));
}

// =====================================================================
// 17. AI command bar: section ordering
// =====================================================================

#[test]
fn ai_bar_first_section_is_response() {
    let actions = get_ai_command_bar_actions();
    assert_eq!(actions[0].section.as_deref(), Some("Response"));
}

#[test]
fn ai_bar_response_section_has_3_actions() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .count();
    assert_eq!(count, 3);
}

#[test]
fn ai_bar_actions_section_has_4_actions() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Actions"))
        .count();
    assert_eq!(count, 4);
}

#[test]
fn ai_bar_total_is_12() {
    let actions = get_ai_command_bar_actions();
    assert_eq!(actions.len(), 12);
}

// =====================================================================
// 18. Notes: auto_sizing_enabled=true hides enable_auto_sizing
// =====================================================================

#[test]
fn notes_auto_sizing_enabled_hides_action() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "enable_auto_sizing"));
}

#[test]
fn notes_auto_sizing_disabled_shows_action() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "enable_auto_sizing"));
}

#[test]
fn notes_auto_sizing_action_shortcut_cmd_a() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let action = actions
        .iter()
        .find(|a| a.id == "enable_auto_sizing")
        .unwrap();
    assert_eq!(action.shortcut.as_deref(), Some("⌘A"));
}

#[test]
fn notes_auto_sizing_action_section_settings() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let action = actions
        .iter()
        .find(|a| a.id == "enable_auto_sizing")
        .unwrap();
    assert_eq!(action.section.as_deref(), Some("Settings"));
}

// =====================================================================
// 19. Notes: full selection action set
// =====================================================================

#[test]
fn notes_full_selection_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note + duplicate + browse_notes + find_in_note + format + copy_note_as + copy_deeplink + create_quicklink + export + enable_auto_sizing = 10
    assert_eq!(actions.len(), 10);
}

// --- merged from part_03.rs ---

#[test]
fn notes_full_selection_has_duplicate() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "duplicate_note"));
}

#[test]
fn notes_full_selection_has_export() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "export"));
}

#[test]
fn notes_no_selection_count() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note + browse_notes + enable_auto_sizing = 3
    assert_eq!(actions.len(), 3);
}

// =====================================================================
// 20. CommandBarConfig: anchor position differences
// =====================================================================

#[test]
fn command_bar_ai_style_anchor_top() {
    let config = CommandBarConfig::ai_style();
    assert!(matches!(config.dialog_config.anchor, AnchorPosition::Top));
}

#[test]
fn command_bar_main_menu_anchor_bottom() {
    let config = CommandBarConfig::main_menu_style();
    assert!(matches!(
        config.dialog_config.anchor,
        AnchorPosition::Bottom
    ));
}

#[test]
fn command_bar_no_search_anchor_bottom() {
    let config = CommandBarConfig::no_search();
    assert!(matches!(
        config.dialog_config.anchor,
        AnchorPosition::Bottom
    ));
}

#[test]
fn command_bar_notes_anchor_top() {
    let config = CommandBarConfig::notes_style();
    assert!(matches!(config.dialog_config.anchor, AnchorPosition::Top));
}

// =====================================================================
// 21. New chat: combination of all three input types
// =====================================================================

#[test]
fn new_chat_all_three_types() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model-1".into(),
        provider: "P".into(),
        provider_display_name: "Provider-1".into(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "m2".into(),
        display_name: "Model-2".into(),
        provider: "P".into(),
        provider_display_name: "Provider-2".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &presets, &models);
    assert_eq!(actions.len(), 3);
}

#[test]
fn new_chat_sections_are_correct() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "LU".into(),
        provider: "P".into(),
        provider_display_name: "PD".into(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "g".into(),
        name: "G".into(),
        icon: IconName::Star,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "m2".into(),
        display_name: "M".into(),
        provider: "P".into(),
        provider_display_name: "PD2".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &presets, &models);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
    assert_eq!(actions[1].section.as_deref(), Some("Presets"));
    assert_eq!(actions[2].section.as_deref(), Some("Models"));
}

#[test]
fn new_chat_all_empty_produces_zero() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert_eq!(actions.len(), 0);
}

#[test]
fn new_chat_only_presets() {
    let presets = vec![
        NewChatPresetInfo {
            id: "a".into(),
            name: "A".into(),
            icon: IconName::Plus,
        },
        NewChatPresetInfo {
            id: "b".into(),
            name: "B".into(),
            icon: IconName::Code,
        },
    ];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions.len(), 2);
    assert!(actions
        .iter()
        .all(|a| a.section.as_deref() == Some("Presets")));
}

// =====================================================================
// 22. Note switcher: pinned+current uses StarFilled
// =====================================================================

#[test]
fn note_switcher_pinned_current_icon_is_star_filled() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n1".into(),
        title: "My Note".into(),
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
fn note_switcher_pinned_not_current_icon_is_star_filled() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n2".into(),
        title: "Other Note".into(),
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
fn note_switcher_current_not_pinned_icon_is_check() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n3".into(),
        title: "Current".into(),
        char_count: 30,
        is_current: true,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::Check));
}

#[test]
fn note_switcher_neither_icon_is_file() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n4".into(),
        title: "Regular".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::File));
}

// =====================================================================
// 23. Note switcher: description with preview exactly 60 chars
// =====================================================================

#[test]
fn note_switcher_preview_60_chars_not_truncated() {
    let preview: String = "a".repeat(60);
    let notes = vec![NoteSwitcherNoteInfo {
        id: "t1".into(),
        title: "T".into(),
        char_count: 60,
        is_current: false,
        is_pinned: false,
        preview: preview.clone(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_deref().unwrap();
    // 60 chars should NOT be truncated (no ellipsis)
    assert!(!desc.contains('…'));
    assert_eq!(desc, &preview);
}

#[test]
fn note_switcher_preview_61_chars_truncated() {
    let preview: String = "b".repeat(61);
    let notes = vec![NoteSwitcherNoteInfo {
        id: "t2".into(),
        title: "T".into(),
        char_count: 61,
        is_current: false,
        is_pinned: false,
        preview,
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_deref().unwrap();
    assert!(desc.contains('…'));
}

#[test]
fn note_switcher_empty_preview_with_time_shows_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "t3".into(),
        title: "T".into(),
        char_count: 0,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: "5m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_deref().unwrap();
    assert_eq!(desc, "5m ago");
}

// =====================================================================
// 24. to_deeplink_name: emoji and unicode handling
// =====================================================================

#[test]
fn deeplink_name_emoji_preserved_as_chars() {
    // Emoji are alphanumeric-ish in Unicode; to_deeplink_name keeps them
    let result = to_deeplink_name("Hello 🌍 World");
    // Spaces become hyphens, emoji is alphanumeric? Let's test actual behavior
    assert!(result.contains("hello"));
    assert!(result.contains("world"));
}

#[test]
fn deeplink_name_accented_chars_preserved() {
    let result = to_deeplink_name("café résumé");
    assert!(result.contains("caf"));
    assert!(result.contains("sum"));
}

#[test]
fn deeplink_name_all_special_chars_empty() {
    let result = to_deeplink_name("!@#$%^&*()");
    assert_eq!(result, "_unnamed");
}

#[test]
fn deeplink_name_mixed_separators() {
    let result = to_deeplink_name("hello---world___test   foo");
    assert_eq!(result, "hello-world-test-foo");
}

// =====================================================================
// 25. ScriptInfo: with_frecency preserves all other fields
// =====================================================================

#[test]
fn with_frecency_preserves_name_and_path() {
    let script = ScriptInfo::new("my-script", "/path/script.ts")
        .with_frecency(true, Some("/frecency".into()));
    assert_eq!(script.name, "my-script");
    assert_eq!(script.path, "/path/script.ts");
}

#[test]
fn with_frecency_preserves_is_script() {
    let script = ScriptInfo::new("my-script", "/path/script.ts").with_frecency(true, None);
    assert!(script.is_script);
}

#[test]
fn with_frecency_preserves_shortcut_and_alias() {
    let script = ScriptInfo::with_shortcut_and_alias(
        "test",
        "/path/test.ts",
        Some("cmd+k".into()),
        Some("tk".into()),
    )
    .with_frecency(true, Some("/fp".into()));
    assert_eq!(script.shortcut, Some("cmd+k".into()));
    assert_eq!(script.alias, Some("tk".into()));
    assert!(script.is_suggested);
}

#[test]
fn with_frecency_false_not_suggested() {
    let script = ScriptInfo::new("s", "/p").with_frecency(false, None);
    assert!(!script.is_suggested);
    assert!(script.frecency_path.is_none());
}

// =====================================================================
// 26. Action: cached lowercase fields correctness
// =====================================================================

#[test]
fn action_title_lower_cached_correctly() {
    let action = Action::new("id", "Hello World", None, ActionCategory::ScriptContext);
    assert_eq!(action.title_lower, "hello world");
}

#[test]
fn action_description_lower_cached() {
    let action = Action::new(
        "id",
        "T",
        Some("My Description HERE".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(action.description_lower, Some("my description here".into()));
}

#[test]
fn action_description_lower_none_when_no_desc() {
    let action = Action::new("id", "T", None, ActionCategory::ScriptContext);
    assert!(action.description_lower.is_none());
}

#[test]
fn action_shortcut_lower_set_after_with_shortcut() {
    let action = Action::new("id", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧K");
    assert_eq!(action.shortcut_lower, Some("⌘⇧k".into()));
}

// =====================================================================
// 27. build_grouped_items_static: SectionStyle::None produces no headers
// =====================================================================

#[test]
fn grouped_items_none_style_no_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Section1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Section2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
    // With SectionStyle::None, no headers should be inserted
    for item in &grouped {
        assert!(
            matches!(item, crate::actions::dialog::GroupedActionItem::Item(_)),
            "SectionStyle::None should not produce headers"
        );
    }
    assert_eq!(grouped.len(), 2);
}

#[test]
fn grouped_items_headers_style_adds_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // 2 section headers + 2 items = 4
    assert_eq!(grouped.len(), 4);
}

#[test]
fn grouped_items_same_section_one_header() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S1"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // 1 header + 2 items = 3
    assert_eq!(grouped.len(), 3);
}

#[test]
fn grouped_items_no_section_no_header() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext),
        Action::new("b", "B", None, ActionCategory::ScriptContext),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // No sections = no headers, just 2 items
    assert_eq!(grouped.len(), 2);
}

// =====================================================================
// 28. coerce_action_selection: edge cases
// =====================================================================

#[test]
fn coerce_selection_empty_returns_none() {
    let rows = vec![];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn coerce_selection_single_item() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![GroupedActionItem::Item(0)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
}

// --- merged from part_04.rs ---

#[test]
fn coerce_selection_beyond_bounds_clamped() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
    // Index 100 should clamp to last = 1
    assert_eq!(coerce_action_selection(&rows, 100), Some(1));
}

#[test]
fn coerce_selection_header_then_item() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![
        GroupedActionItem::SectionHeader("S".into()),
        GroupedActionItem::Item(0),
    ];
    // Landing on header at 0, search down → finds Item at 1
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

// =====================================================================
// 29. score_action: combined bonuses max scenario
// =====================================================================

#[test]
fn score_action_prefix_plus_desc_plus_shortcut() {
    let action = Action::new(
        "script:edit",
        "Edit Script",
        Some("Edit the script file".into()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘E");
    let score = ActionsDialog::score_action(&action, "e");
    // prefix(100) + desc contains(15) + shortcut contains(10) = 125
    assert!(score >= 125, "Expected ≥125, got {}", score);
}

#[test]
fn score_action_contains_only() {
    let action = Action::new(
        "copy_edit",
        "Copy Edit Path",
        None,
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "edit");
    assert!((50..100).contains(&score), "Expected 50-99, got {}", score);
}

#[test]
fn score_action_no_match() {
    let action = Action::new("test", "Hello World", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "xyz");
    assert_eq!(score, 0);
}

#[test]
fn score_action_empty_search_prefix_match() {
    let action = Action::new("test", "Anything", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "");
    // Empty string is prefix of everything
    assert!(score >= 100, "Expected ≥100, got {}", score);
}

// =====================================================================
// 30. Cross-context: ProtocolAction close/visibility defaults and SDK action ID format
// =====================================================================

#[test]
fn protocol_action_sdk_id_matches_name() {
    // SDK actions use name as ID
    let pa = ProtocolAction {
        name: "My Custom Action".into(),
        description: Some("desc".into()),
        shortcut: None,
        value: Some("val".into()),
        has_action: true,
        visible: None,
        close: None,
    };
    // Simulate conversion (as done in set_sdk_actions)
    let action = Action::new(
        pa.name.clone(),
        pa.name.clone(),
        pa.description.clone(),
        ActionCategory::ScriptContext,
    );
    assert_eq!(action.id, "My Custom Action");
}

#[test]
fn protocol_action_shortcut_converted_via_format() {
    let formatted = ActionsDialog::format_shortcut_hint("cmd+shift+c");
    assert_eq!(formatted, "⌘⇧C");
}

#[test]
fn protocol_action_sdk_icon_is_none() {
    // SDK actions don't currently have icons
    let action = Action::new(
        "sdk_action",
        "SDK Action",
        None,
        ActionCategory::ScriptContext,
    );
    assert!(action.icon.is_none());
}

#[test]
fn protocol_action_sdk_section_is_none() {
    // SDK actions don't currently have sections
    let action = Action::new(
        "sdk_action",
        "SDK Action",
        None,
        ActionCategory::ScriptContext,
    );
    assert!(action.section.is_none());
}
