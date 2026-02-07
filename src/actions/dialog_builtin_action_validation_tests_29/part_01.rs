//! Batch 29: Builtin action validation tests
//!
//! 115 tests across 30 categories validating built-in action window dialog behaviors.

use super::builders::*;
use super::command_bar::CommandBarConfig;
use super::dialog::{build_grouped_items_static, coerce_action_selection, GroupedActionItem};
use super::types::*;
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;

// =============================================================================
// Category 1: Note switcher — empty notes produces helpful placeholder action
// =============================================================================

#[test]
fn cat29_01_note_switcher_empty_produces_placeholder() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions.len(), 1);
}

#[test]
fn cat29_01_note_switcher_empty_placeholder_id() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions[0].id, "no_notes");
}

#[test]
fn cat29_01_note_switcher_empty_placeholder_title() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions[0].title, "No notes yet");
}

#[test]
fn cat29_01_note_switcher_empty_placeholder_desc_mentions_cmd_n() {
    let actions = get_note_switcher_actions(&[]);
    assert!(actions[0].description.as_ref().unwrap().contains("⌘N"));
}

#[test]
fn cat29_01_note_switcher_empty_placeholder_icon_plus() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions[0].icon, Some(IconName::Plus));
}

// =============================================================================
// Category 2: Note switcher — section is "Notes" for placeholder, else Pinned/Recent
// =============================================================================

#[test]
fn cat29_02_note_switcher_empty_section_notes() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions[0].section.as_deref(), Some("Notes"));
}

#[test]
fn cat29_02_note_switcher_pinned_section() {
    let note = NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Pinned Note".into(),
        char_count: 100,
        is_current: false,
        is_pinned: true,
        preview: "some text".into(),
        relative_time: "1h ago".into(),
    };
    let actions = get_note_switcher_actions(&[note]);
    assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
}

#[test]
fn cat29_02_note_switcher_unpinned_section() {
    let note = NoteSwitcherNoteInfo {
        id: "def".into(),
        title: "Regular Note".into(),
        char_count: 50,
        is_current: false,
        is_pinned: false,
        preview: "content".into(),
        relative_time: "2d ago".into(),
    };
    let actions = get_note_switcher_actions(&[note]);
    assert_eq!(actions[0].section.as_deref(), Some("Recent"));
}

// =============================================================================
// Category 3: Note switcher — note ID format is "note_{uuid}"
// =============================================================================

#[test]
fn cat29_03_note_switcher_id_format() {
    let note = NoteSwitcherNoteInfo {
        id: "550e8400-e29b-41d4-a716-446655440000".into(),
        title: "Test".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    };
    let actions = get_note_switcher_actions(&[note]);
    assert_eq!(actions[0].id, "note_550e8400-e29b-41d4-a716-446655440000");
}

#[test]
fn cat29_03_note_switcher_id_starts_with_note_prefix() {
    let note = NoteSwitcherNoteInfo {
        id: "xyz".into(),
        title: "T".into(),
        char_count: 1,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    };
    let actions = get_note_switcher_actions(&[note]);
    assert!(actions[0].id.starts_with("note_"));
}

// =============================================================================
// Category 4: Clipboard — text entry action count on macOS vs all platforms
// =============================================================================

#[test]
fn cat29_04_clipboard_text_action_count_cross_platform() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "Hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    // Cross-platform: paste, copy, keep_open, share, attach_to_ai, pin, save_snippet, save_file, delete, delete_multiple, delete_all = 11
    // macOS adds: quick_look = +1 = 12
    #[cfg(target_os = "macos")]
    assert_eq!(actions.len(), 12);
    #[cfg(not(target_os = "macos"))]
    assert_eq!(actions.len(), 11);
}

#[test]
fn cat29_04_clipboard_image_action_count_cross_platform() {
    let entry = ClipboardEntryInfo {
        id: "2".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "Image".into(),
        image_dimensions: Some((640, 480)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    // Image adds: ocr (+1 vs text)
    // macOS adds: quick_look, open_with, annotate_cleanshot, upload_cleanshot
    #[cfg(target_os = "macos")]
    assert_eq!(actions.len(), 16);
    #[cfg(not(target_os = "macos"))]
    assert_eq!(actions.len(), 12);
}

// =============================================================================
// Category 5: Clipboard — attach_to_ai shortcut and section
// =============================================================================

#[test]
fn cat29_05_clipboard_attach_to_ai_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ai = actions
        .iter()
        .find(|a| a.id == "clipboard_attach_to_ai")
        .unwrap();
    assert_eq!(ai.shortcut.as_deref(), Some("⌃⌘A"));
}

#[test]
fn cat29_05_clipboard_attach_to_ai_title() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ai = actions
        .iter()
        .find(|a| a.id == "clipboard_attach_to_ai")
        .unwrap();
    assert_eq!(ai.title, "Attach to AI Chat");
}

#[test]
fn cat29_05_clipboard_attach_to_ai_desc_mentions_ai() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ai = actions
        .iter()
        .find(|a| a.id == "clipboard_attach_to_ai")
        .unwrap();
    assert!(ai
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("ai"));
}

// =============================================================================
// Category 6: Clipboard — share action present for both text and image
// =============================================================================

#[test]
fn cat29_06_clipboard_share_present_for_text() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "abc".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_share"));
}

#[test]
fn cat29_06_clipboard_share_present_for_image() {
    let entry = ClipboardEntryInfo {
        id: "2".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_share"));
}

#[test]
fn cat29_06_clipboard_share_shortcut_same_for_both() {
    let text_entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let img_entry = ClipboardEntryInfo {
        id: "2".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "i".into(),
        image_dimensions: Some((10, 10)),
        frontmost_app_name: None,
    };
    let text_actions = get_clipboard_history_context_actions(&text_entry);
    let img_actions = get_clipboard_history_context_actions(&img_entry);
    let ts = text_actions
        .iter()
        .find(|a| a.id == "clipboard_share")
        .unwrap();
    let is = img_actions
        .iter()
        .find(|a| a.id == "clipboard_share")
        .unwrap();
    assert_eq!(ts.shortcut, is.shortcut);
}

// =============================================================================
// Category 7: Script context — with_all constructor preserves every field
// =============================================================================

#[test]
fn cat29_07_with_all_preserves_name() {
    let s = ScriptInfo::with_all(
        "Foo",
        "/p",
        true,
        "Launch",
        Some("cmd+f".into()),
        Some("f".into()),
    );
    assert_eq!(s.name, "Foo");
}

#[test]
fn cat29_07_with_all_preserves_action_verb() {
    let s = ScriptInfo::with_all("Foo", "/p", true, "Launch", None, None);
    assert_eq!(s.action_verb, "Launch");
}

#[test]
fn cat29_07_with_all_preserves_is_script() {
    let s = ScriptInfo::with_all("Foo", "/p", false, "Run", None, None);
    assert!(!s.is_script);
}

#[test]
fn cat29_07_with_all_run_title_uses_verb_and_name() {
    let s = ScriptInfo::with_all("My Tool", "/p", true, "Execute", None, None);
    let actions = get_script_context_actions(&s);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert!(run.title.contains("Execute"));
    assert!(run.title.contains("My Tool"));
}

// =============================================================================
// Category 8: Script context — agent has edit_script with "Agent" title but no view_logs
// =============================================================================

#[test]
fn cat29_08_agent_edit_title_says_agent() {
    let mut s = ScriptInfo::new("My Agent", "/p.md");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.title, "Edit Agent");
}

#[test]
fn cat29_08_agent_no_view_logs() {
    let mut s = ScriptInfo::new("My Agent", "/p.md");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

#[test]
fn cat29_08_agent_has_copy_content() {
    let mut s = ScriptInfo::new("My Agent", "/p.md");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "copy_content"));
}

#[test]
fn cat29_08_agent_has_reveal_in_finder() {
    let mut s = ScriptInfo::new("My Agent", "/p.md");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
}

// =============================================================================
// Category 9: Notes command bar — format action details
// =============================================================================

#[test]
fn cat29_09_notes_format_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let fmt = actions.iter().find(|a| a.id == "format").unwrap();
    assert_eq!(fmt.shortcut.as_deref(), Some("⇧⌘T"));
}

#[test]
fn cat29_09_notes_format_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let fmt = actions.iter().find(|a| a.id == "format").unwrap();
    assert_eq!(fmt.icon, Some(IconName::Code));
}

#[test]
fn cat29_09_notes_format_section() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let fmt = actions.iter().find(|a| a.id == "format").unwrap();
    assert_eq!(fmt.section.as_deref(), Some("Edit"));
}

#[test]
fn cat29_09_notes_format_absent_without_selection() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "format"));
}

// =============================================================================
// Category 10: Notes command bar — new_note always present with correct details
// =============================================================================

#[test]
fn cat29_10_notes_new_note_always_present_full() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "new_note"));
}

#[test]
fn cat29_10_notes_new_note_always_present_trash() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "new_note"));
}
