// --- merged from part_01.rs ---
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
        .find(|a| a.id == "clip:clipboard_attach_to_ai")
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
        .find(|a| a.id == "clip:clipboard_attach_to_ai")
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
        .find(|a| a.id == "clip:clipboard_attach_to_ai")
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
    assert!(actions.iter().any(|a| a.id == "clip:clipboard_share"));
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
    assert!(actions.iter().any(|a| a.id == "clip:clipboard_share"));
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
        .find(|a| a.id == "clip:clipboard_share")
        .unwrap();
    let is = img_actions
        .iter()
        .find(|a| a.id == "clip:clipboard_share")
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
    assert_eq!(run.title, "Execute");
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

// --- merged from part_02.rs ---

#[test]
fn cat29_10_notes_new_note_shortcut() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let nn = actions.iter().find(|a| a.id == "new_note").unwrap();
    assert_eq!(nn.shortcut.as_deref(), Some("⌘N"));
}

#[test]
fn cat29_10_notes_new_note_icon() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let nn = actions.iter().find(|a| a.id == "new_note").unwrap();
    assert_eq!(nn.icon, Some(IconName::Plus));
}

// =============================================================================
// Category 11: AI command bar — copy_chat details
// =============================================================================

#[test]
fn cat29_11_ai_copy_chat_shortcut() {
    let actions = get_ai_command_bar_actions();
    let cc = actions.iter().find(|a| a.id == "chat:copy_chat").unwrap();
    assert_eq!(cc.shortcut.as_deref(), Some("⌥⇧⌘C"));
}

#[test]
fn cat29_11_ai_copy_chat_icon() {
    let actions = get_ai_command_bar_actions();
    let cc = actions.iter().find(|a| a.id == "chat:copy_chat").unwrap();
    assert_eq!(cc.icon, Some(IconName::Copy));
}

#[test]
fn cat29_11_ai_copy_chat_section() {
    let actions = get_ai_command_bar_actions();
    let cc = actions.iter().find(|a| a.id == "chat:copy_chat").unwrap();
    assert_eq!(cc.section.as_deref(), Some("Response"));
}

#[test]
fn cat29_11_ai_copy_chat_desc_mentions_conversation() {
    let actions = get_ai_command_bar_actions();
    let cc = actions.iter().find(|a| a.id == "chat:copy_chat").unwrap();
    assert!(cc
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("conversation"));
}

// =============================================================================
// Category 12: AI command bar — copy_last_code details
// =============================================================================

#[test]
fn cat29_12_ai_copy_last_code_shortcut() {
    let actions = get_ai_command_bar_actions();
    let clc = actions.iter().find(|a| a.id == "chat:copy_last_code").unwrap();
    assert_eq!(clc.shortcut.as_deref(), Some("⌥⌘C"));
}

#[test]
fn cat29_12_ai_copy_last_code_icon() {
    let actions = get_ai_command_bar_actions();
    let clc = actions.iter().find(|a| a.id == "chat:copy_last_code").unwrap();
    assert_eq!(clc.icon, Some(IconName::Code));
}

#[test]
fn cat29_12_ai_copy_last_code_section() {
    let actions = get_ai_command_bar_actions();
    let clc = actions.iter().find(|a| a.id == "chat:copy_last_code").unwrap();
    assert_eq!(clc.section.as_deref(), Some("Response"));
}

#[test]
fn cat29_12_ai_copy_last_code_desc_mentions_code() {
    let actions = get_ai_command_bar_actions();
    let clc = actions.iter().find(|a| a.id == "chat:copy_last_code").unwrap();
    assert!(clc
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("code"));
}

// =============================================================================
// Category 13: AI command bar — copy_response in command bar vs chat context
// =============================================================================

#[test]
fn cat29_13_ai_command_bar_copy_response_shortcut() {
    let actions = get_ai_command_bar_actions();
    let cr = actions.iter().find(|a| a.id == "chat:copy_response").unwrap();
    assert_eq!(cr.shortcut.as_deref(), Some("⇧⌘C"));
}

#[test]
fn cat29_13_chat_context_copy_response_shortcut() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let cr = actions.iter().find(|a| a.id == "chat:copy_response").unwrap();
    assert_eq!(cr.shortcut.as_deref(), Some("⌘C"));
}

#[test]
fn cat29_13_ai_vs_chat_copy_response_different_shortcuts() {
    let ai_actions = get_ai_command_bar_actions();
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let chat_actions = get_chat_context_actions(&info);
    let ai_cr = ai_actions.iter().find(|a| a.id == "chat:copy_response").unwrap();
    let chat_cr = chat_actions
        .iter()
        .find(|a| a.id == "chat:copy_response")
        .unwrap();
    assert_ne!(ai_cr.shortcut, chat_cr.shortcut);
}

// =============================================================================
// Category 14: Chat context — model ID format is "select_model_{id}"
// =============================================================================

#[test]
fn cat29_14_chat_model_id_format() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "claude-3-opus".into(),
            display_name: "Claude 3 Opus".into(),
            provider: "Anthropic".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "chat:select_model_claude-3-opus"));
}

#[test]
fn cat29_14_chat_model_title_is_display_name() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "gpt-4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let m = actions
        .iter()
        .find(|a| a.id == "chat:select_model_gpt-4")
        .unwrap();
    assert_eq!(m.title, "GPT-4");
}

#[test]
fn cat29_14_chat_model_description_via_provider() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "gpt-4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let m = actions
        .iter()
        .find(|a| a.id == "chat:select_model_gpt-4")
        .unwrap();
    assert_eq!(m.description.as_deref(), Some("Uses OpenAI"));
}

// =============================================================================
// Category 15: File context — open_file title format
// =============================================================================

#[test]
fn cat29_15_file_open_title_quotes_name() {
    let fi = FileInfo {
        path: "/test/doc.pdf".into(),
        name: "doc.pdf".into(),
        file_type: FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    let open = actions.iter().find(|a| a.id == "file:open_file").unwrap();
    assert!(open.title.contains("\"doc.pdf\""));
}

#[test]
fn cat29_15_file_dir_open_title_quotes_name() {
    let fi = FileInfo {
        path: "/test/Documents".into(),
        name: "Documents".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&fi);
    let open = actions.iter().find(|a| a.id == "file:open_directory").unwrap();
    assert!(open.title.contains("\"Documents\""));
}

#[test]
fn cat29_15_file_open_desc_says_default_application() {
    let fi = FileInfo {
        path: "/test/image.png".into(),
        name: "image.png".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    let open = actions.iter().find(|a| a.id == "file:open_file").unwrap();
    assert!(open
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("default app"));
}

#[test]
fn cat29_15_file_dir_open_desc_says_folder() {
    let fi = FileInfo {
        path: "/test/Docs".into(),
        name: "Docs".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&fi);
    let open = actions.iter().find(|a| a.id == "file:open_directory").unwrap();
    assert!(open
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("folder"));
}

// =============================================================================
// Category 16: Path context — select_file vs open_directory description wording
// =============================================================================

#[test]
fn cat29_16_path_select_file_desc_says_submit() {
    let pi = PathInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    let sel = actions.iter().find(|a| a.id == "file:select_file").unwrap();
    assert!(sel
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("selects this file"));
}

#[test]
fn cat29_16_path_open_directory_desc_says_navigate() {
    let pi = PathInfo {
        path: "/test/folder".into(),
        name: "folder".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    let od = actions.iter().find(|a| a.id == "file:open_directory").unwrap();
    assert!(od
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("opens this directory"));
}

#[test]
fn cat29_16_path_file_has_no_open_directory() {
    let pi = PathInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    assert!(!actions.iter().any(|a| a.id == "file:open_directory"));
}

#[test]
fn cat29_16_path_dir_has_no_select_file() {
    let pi = PathInfo {
        path: "/test/folder".into(),
        name: "folder".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    assert!(!actions.iter().any(|a| a.id == "file:select_file"));
}

// =============================================================================
// Category 17: to_deeplink_name — preserves numbers and lowercase letters
// =============================================================================

#[test]
fn cat29_17_deeplink_lowercase_preserved() {
    assert_eq!(to_deeplink_name("hello"), "hello");
}

#[test]
fn cat29_17_deeplink_numbers_preserved() {
    assert_eq!(to_deeplink_name("test123"), "test123");
}

#[test]
fn cat29_17_deeplink_mixed_case_lowered() {
    assert_eq!(to_deeplink_name("HelloWorld"), "helloworld");
}

#[test]
fn cat29_17_deeplink_spaces_to_hyphens() {
    assert_eq!(to_deeplink_name("my script name"), "my-script-name");
}

// =============================================================================
// Category 18: format_shortcut_hint (dialog.rs) — combined modifier+key combos
// =============================================================================

#[test]
fn cat29_18_format_hint_cmd_shift_k() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("cmd+shift+k"),
        "⌘⇧K"
    );
}

#[test]
fn cat29_18_format_hint_ctrl_alt_delete() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("ctrl+alt+delete"),
        "⌃⌥⌫"
    );
}

#[test]
fn cat29_18_format_hint_meta_alias() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("meta+c"),
        "⌘C"
    );
}

#[test]
fn cat29_18_format_hint_option_space() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("option+space"),
        "⌥␣"
    );
}

#[test]
fn cat29_18_format_hint_single_enter() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("enter"),
        "↵"
    );
}

// =============================================================================
// Category 19: parse_shortcut_keycaps — multi-symbol shortcut strings
// =============================================================================

#[test]
fn cat29_19_parse_keycaps_cmd_enter() {
    let keycaps = super::dialog::ActionsDialog::parse_shortcut_keycaps("⌘↵");
    assert_eq!(keycaps, vec!["⌘", "↵"]);
}

#[test]
fn cat29_19_parse_keycaps_all_modifiers_plus_key() {
    let keycaps = super::dialog::ActionsDialog::parse_shortcut_keycaps("⌃⌥⇧⌘K");
    assert_eq!(keycaps, vec!["⌃", "⌥", "⇧", "⌘", "K"]);
}

#[test]
fn cat29_19_parse_keycaps_space_symbol() {
    let keycaps = super::dialog::ActionsDialog::parse_shortcut_keycaps("␣");
    assert_eq!(keycaps, vec!["␣"]);
}

#[test]
fn cat29_19_parse_keycaps_arrows() {
    let keycaps = super::dialog::ActionsDialog::parse_shortcut_keycaps("↑↓←→");
    assert_eq!(keycaps, vec!["↑", "↓", "←", "→"]);
}

// =============================================================================
// Category 20: score_action — description bonus adds to prefix score
// =============================================================================

#[test]
fn cat29_20_score_prefix_plus_desc_bonus() {
    let action = Action::new(
        "edit_script",
        "Edit Script",
        Some("Edit the script in your editor".to_string()),
        ActionCategory::ScriptContext,
    );
    let score = super::dialog::ActionsDialog::score_action(&action, "edit");
    // prefix(100) + desc bonus(15) = 115
    assert!(score >= 115);
}

// --- merged from part_03.rs ---

#[test]
fn cat29_20_score_prefix_plus_shortcut_bonus() {
    let action = Action::new(
        "edit_script",
        "Edit Script",
        None,
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘E");
    // search for "e" — prefix match on title "edit script" (100) + shortcut "⌘e" contains "e" (10)
    let score = super::dialog::ActionsDialog::score_action(&action, "e");
    assert!(score >= 110);
}

#[test]
fn cat29_20_score_all_three_bonuses() {
    let action = Action::new(
        "edit_script",
        "Edit Script",
        Some("Edit the file".to_string()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘E");
    // "e" => title prefix(100) + desc contains(15) + shortcut contains(10)
    let score = super::dialog::ActionsDialog::score_action(&action, "e");
    assert!(score >= 125);
}

// =============================================================================
// Category 21: fuzzy_match — case sensitivity and edge cases
// =============================================================================

#[test]
fn cat29_21_fuzzy_match_exact() {
    assert!(super::dialog::ActionsDialog::fuzzy_match("hello", "hello"));
}

#[test]
fn cat29_21_fuzzy_match_subsequence() {
    assert!(super::dialog::ActionsDialog::fuzzy_match(
        "hello world",
        "hwd"
    ));
}

#[test]
fn cat29_21_fuzzy_match_no_match() {
    assert!(!super::dialog::ActionsDialog::fuzzy_match("hello", "xyz"));
}

#[test]
fn cat29_21_fuzzy_match_empty_needle_matches() {
    assert!(super::dialog::ActionsDialog::fuzzy_match("anything", ""));
}

#[test]
fn cat29_21_fuzzy_match_needle_longer_fails() {
    assert!(!super::dialog::ActionsDialog::fuzzy_match("hi", "hello"));
}

// =============================================================================
// Category 22: build_grouped_items_static — section headers with Headers style
// =============================================================================

#[test]
fn cat29_22_grouped_items_headers_adds_section_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Group1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Group2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // Should have: Header("Group1"), Item(0), Header("Group2"), Item(1)
    assert_eq!(grouped.len(), 4);
    assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(_)));
    assert!(matches!(grouped[1], GroupedActionItem::Item(0)));
    assert!(matches!(grouped[2], GroupedActionItem::SectionHeader(_)));
    assert!(matches!(grouped[3], GroupedActionItem::Item(1)));
}

#[test]
fn cat29_22_grouped_items_separators_no_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Group1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Group2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    // Should have just items, no headers
    assert_eq!(grouped.len(), 2);
    assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
    assert!(matches!(grouped[1], GroupedActionItem::Item(1)));
}

#[test]
fn cat29_22_grouped_items_same_section_one_header() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Same"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Same"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // Should have: Header("Same"), Item(0), Item(1) — only one header
    assert_eq!(grouped.len(), 3);
    assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(_)));
}

#[test]
fn cat29_22_grouped_items_empty_returns_empty() {
    let actions: Vec<Action> = vec![];
    let filtered: Vec<usize> = vec![];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    assert!(grouped.is_empty());
}

// =============================================================================
// Category 23: coerce_action_selection — header skipping behavior
// =============================================================================

#[test]
fn cat29_23_coerce_on_item_stays() {
    let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
}

#[test]
fn cat29_23_coerce_on_header_jumps_down() {
    let rows = vec![
        GroupedActionItem::SectionHeader("H".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn cat29_23_coerce_trailing_header_jumps_up() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("H".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn cat29_23_coerce_all_headers_returns_none() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".into()),
        GroupedActionItem::SectionHeader("B".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn cat29_23_coerce_empty_returns_none() {
    let rows: Vec<GroupedActionItem> = vec![];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

// =============================================================================
// Category 24: CommandBarConfig — all presets preserve close defaults
// =============================================================================

#[test]
fn cat29_24_ai_style_close_on_select() {
    let config = CommandBarConfig::ai_style();
    assert!(config.close_on_select);
}

#[test]
fn cat29_24_main_menu_close_on_escape() {
    let config = CommandBarConfig::main_menu_style();
    assert!(config.close_on_escape);
}

#[test]
fn cat29_24_no_search_close_on_click_outside() {
    let config = CommandBarConfig::no_search();
    assert!(config.close_on_click_outside);
}

#[test]
fn cat29_24_notes_style_close_defaults() {
    let config = CommandBarConfig::notes_style();
    assert!(config.close_on_select);
    assert!(config.close_on_escape);
    assert!(config.close_on_click_outside);
}

// =============================================================================
// Category 25: CommandBarConfig — show_icons and show_footer combinations
// =============================================================================

#[test]
fn cat29_25_ai_style_has_icons_and_footer() {
    let config = CommandBarConfig::ai_style();
    assert!(config.dialog_config.show_icons);
    assert!(config.dialog_config.show_footer);
}

#[test]
fn cat29_25_main_menu_no_icons_no_footer() {
    let config = CommandBarConfig::main_menu_style();
    assert!(!config.dialog_config.show_icons);
    assert!(!config.dialog_config.show_footer);
}

#[test]
fn cat29_25_notes_style_has_icons_and_footer() {
    let config = CommandBarConfig::notes_style();
    assert!(config.dialog_config.show_icons);
    assert!(config.dialog_config.show_footer);
}

#[test]
fn cat29_25_no_search_no_icons_no_footer() {
    let config = CommandBarConfig::no_search();
    assert!(!config.dialog_config.show_icons);
    assert!(!config.dialog_config.show_footer);
}

// =============================================================================
// Category 26: New chat — empty inputs produce empty actions
// =============================================================================

#[test]
fn cat29_26_new_chat_all_empty() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

#[test]
fn cat29_26_new_chat_only_last_used() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "P".into(),
        provider_display_name: "Provider".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
}

#[test]
fn cat29_26_new_chat_only_presets() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Presets"));
}

#[test]
fn cat29_26_new_chat_only_models() {
    let models = vec![NewChatModelInfo {
        model_id: "m2".into(),
        display_name: "Model 2".into(),
        provider: "P".into(),
        provider_display_name: "Provider".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Models"));
}

// =============================================================================
// Category 27: Scriptlet context with_custom — reset_ranking is always last
// =============================================================================

#[test]
fn cat29_27_scriptlet_frecency_reset_ranking_last() {
    let script =
        ScriptInfo::scriptlet("S", "/p.md", None, None).with_frecency(true, Some("s".into()));
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let last = actions.last().unwrap();
    assert_eq!(last.id, "reset_ranking");
}

#[test]
fn cat29_27_script_frecency_reset_ranking_last() {
    let script = ScriptInfo::new("S", "/p.ts").with_frecency(true, Some("s".into()));
    let actions = get_script_context_actions(&script);
    let last = actions.last().unwrap();
    assert_eq!(last.id, "reset_ranking");
}

#[test]
fn cat29_27_builtin_frecency_reset_ranking_last() {
    let script = ScriptInfo::builtin("CH").with_frecency(true, Some("ch".into()));
    let actions = get_script_context_actions(&script);
    let last = actions.last().unwrap();
    assert_eq!(last.id, "reset_ranking");
}

// =============================================================================
// Category 28: Cross-context — all built-in actions have ActionCategory::ScriptContext
// =============================================================================

#[test]
fn cat29_28_script_all_script_context() {
    let script = ScriptInfo::new("S", "/p.ts");
    let actions = get_script_context_actions(&script);
    for a in &actions {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "action {} has wrong category",
            a.id
        );
    }
}

#[test]
fn cat29_28_clipboard_all_script_context() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    for a in &actions {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "action {} has wrong category",
            a.id
        );
    }
}

#[test]
fn cat29_28_ai_all_script_context() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "action {} has wrong category",
            a.id
        );
    }
}

#[test]
fn cat29_28_path_all_script_context() {
    let pi = PathInfo {
        path: "/p".into(),
        name: "p".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    for a in &actions {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "action {} has wrong category",
            a.id
        );
    }
}

// =============================================================================
// Category 29: Cross-context — first action ID is always the primary action
// =============================================================================

#[test]
fn cat29_29_script_first_is_run_script() {
    let script = ScriptInfo::new("S", "/p.ts");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn cat29_29_clipboard_first_is_paste() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[0].id, "clip:clipboard_paste");
}

#[test]
fn cat29_29_path_file_first_is_select_file() {
    let pi = PathInfo {
        path: "/p/f.txt".into(),
        name: "f.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    assert_eq!(actions[0].id, "file:select_file");
}

#[test]
fn cat29_29_path_dir_first_is_open_directory() {
    let pi = PathInfo {
        path: "/p/dir".into(),
        name: "dir".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    assert_eq!(actions[0].id, "file:open_directory");
}

#[test]
fn cat29_29_file_first_is_open() {
    let fi = FileInfo {
        path: "/p/f.txt".into(),
        name: "f.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    assert_eq!(actions[0].id, "file:open_file");
}

// =============================================================================
// Category 30: Action builder — chaining preserves all fields correctly
// =============================================================================

#[test]
fn cat29_30_action_new_defaults() {
    let a = Action::new(
        "id",
        "Title",
        Some("Desc".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(a.id, "id");
    assert_eq!(a.title, "Title");
    assert_eq!(a.description.as_deref(), Some("Desc"));
    assert!(!a.has_action);
    assert!(a.shortcut.is_none());
    assert!(a.icon.is_none());
    assert!(a.section.is_none());
    assert!(a.value.is_none());
}

// --- merged from part_04.rs ---

#[test]
fn cat29_30_action_full_chain() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext)
        .with_shortcut("⌘X")
        .with_icon(IconName::Trash)
        .with_section("Danger");
    assert_eq!(a.shortcut.as_deref(), Some("⌘X"));
    assert_eq!(a.icon, Some(IconName::Trash));
    assert_eq!(a.section.as_deref(), Some("Danger"));
}

#[test]
fn cat29_30_action_title_lower_computed() {
    let a = Action::new("id", "Hello World", None, ActionCategory::ScriptContext);
    assert_eq!(a.title_lower, "hello world");
}

#[test]
fn cat29_30_action_description_lower_computed() {
    let a = Action::new(
        "id",
        "T",
        Some("FoO BaR".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(a.description_lower.as_deref(), Some("foo bar"));
}

#[test]
fn cat29_30_action_shortcut_lower_computed() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
    assert_eq!(a.shortcut_lower.as_deref(), Some("⌘e"));
}
