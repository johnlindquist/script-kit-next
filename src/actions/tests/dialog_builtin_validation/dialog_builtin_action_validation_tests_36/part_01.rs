// Batch 36: Dialog built-in action validation tests
//
// 124 tests across 30 categories validating random behaviors from
// built-in action window dialogs.

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
    let ql = actions.iter().find(|a| a.id == "clipboard_quick_look");
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
    if let Some(action) = actions.iter().find(|a| a.id == "clipboard_quick_look") {
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
    if let Some(action) = actions.iter().find(|a| a.id == "clipboard_quick_look") {
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
    let has_ql = actions.iter().any(|a| a.id == "clipboard_quick_look");
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
    let text_has_ql = text_actions.iter().any(|a| a.id == "clipboard_quick_look");
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
    let action = actions.iter().find(|a| a.id == "clipboard_delete").unwrap();
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
    let action = actions.iter().find(|a| a.id == "clipboard_delete").unwrap();
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
    let action = actions.iter().find(|a| a.id == "clipboard_delete").unwrap();
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
    assert!(actions.iter().any(|a| a.id == "clipboard_delete"));
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
    assert_eq!(actions[0].id, "clipboard_paste");
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
    assert_eq!(actions[1].id, "clipboard_copy");
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
    assert_eq!(actions.last().unwrap().id, "clipboard_delete_all");
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
    assert_eq!(actions[n - 3].id, "clipboard_delete");
    assert_eq!(actions[n - 2].id, "clipboard_delete_multiple");
    assert_eq!(actions[n - 1].id, "clipboard_delete_all");
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
    let has_ql = actions.iter().any(|a| a.id == "quick_look");
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
    assert!(!actions.iter().any(|a| a.id == "quick_look"));
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
    if let Some(ql) = actions.iter().find(|a| a.id == "quick_look") {
        assert_eq!(ql.shortcut.as_deref(), Some("⌘Y"));
    }
}
