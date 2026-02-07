//! Batch 34: Dialog built-in action validation tests
//!
//! 120 tests across 30 categories validating random behaviors from
//! built-in action window dialogs.

use crate::actions::builders::{
    get_ai_command_bar_actions, get_chat_context_actions, get_clipboard_history_context_actions,
    get_file_context_actions, get_new_chat_actions, get_note_switcher_actions,
    get_notes_command_bar_actions, get_path_context_actions, get_script_context_actions,
    get_scriptlet_context_actions_with_custom, to_deeplink_name, ChatModelInfo, ChatPromptInfo,
    ClipboardEntryInfo, NewChatModelInfo, NewChatPresetInfo, NoteSwitcherNoteInfo, NotesInfo,
};
use crate::actions::command_bar::CommandBarConfig;
use crate::actions::dialog::ActionsDialog;
use crate::actions::types::{Action, ActionCategory, ScriptInfo, SearchPosition, SectionStyle};
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::FileInfo;
use crate::prompts::PathInfo;

// =====================================================================
// 1. Clipboard: pinned image entry has both unpin and image-specific actions
// =====================================================================

#[test]
fn clipboard_pinned_image_has_unpin_not_pin() {
    let entry = ClipboardEntryInfo {
        id: "pi-1".into(),
        content_type: ContentType::Image,
        pinned: true,
        preview: "screenshot".into(),
        image_dimensions: Some((1920, 1080)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_unpin"));
    assert!(!actions.iter().any(|a| a.id == "clipboard_pin"));
}

#[test]
fn clipboard_pinned_image_has_ocr() {
    let entry = ClipboardEntryInfo {
        id: "pi-2".into(),
        content_type: ContentType::Image,
        pinned: true,
        preview: "screenshot".into(),
        image_dimensions: Some((1920, 1080)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_ocr"));
}

#[test]
fn clipboard_unpinned_text_has_pin_not_unpin() {
    let entry = ClipboardEntryInfo {
        id: "ut-1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_pin"));
    assert!(!actions.iter().any(|a| a.id == "clipboard_unpin"));
}

#[test]
fn clipboard_pinned_text_has_unpin() {
    let entry = ClipboardEntryInfo {
        id: "pt-1".into(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_unpin"));
    assert!(!actions.iter().any(|a| a.id == "clipboard_pin"));
}

// =====================================================================
// 2. Clipboard: OCR shortcut and description details
// =====================================================================

#[test]
fn clipboard_ocr_shortcut_is_shift_cmd_c() {
    let entry = ClipboardEntryInfo {
        id: "ocr-1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ocr = actions.iter().find(|a| a.id == "clipboard_ocr").unwrap();
    assert_eq!(ocr.shortcut.as_deref(), Some("⇧⌘C"));
}

#[test]
fn clipboard_ocr_title_is_copy_text_from_image() {
    let entry = ClipboardEntryInfo {
        id: "ocr-2".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ocr = actions.iter().find(|a| a.id == "clipboard_ocr").unwrap();
    assert_eq!(ocr.title, "Copy Text from Image");
}

#[test]
fn clipboard_ocr_desc_mentions_ocr() {
    let entry = ClipboardEntryInfo {
        id: "ocr-3".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ocr = actions.iter().find(|a| a.id == "clipboard_ocr").unwrap();
    assert!(ocr.description.as_ref().unwrap().contains("OCR"));
}

#[test]
fn clipboard_ocr_absent_for_text_entry() {
    let entry = ClipboardEntryInfo {
        id: "ocr-4".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_ocr"));
}

// =====================================================================
// 3. Path context: move_to_trash description differs for file vs dir
// =====================================================================

#[test]
fn path_move_to_trash_file_desc_says_delete_file() {
    let path_info = PathInfo {
        path: "/tmp/foo.txt".into(),
        name: "foo.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert!(trash.description.as_ref().unwrap().contains("file"));
}

#[test]
fn path_move_to_trash_dir_desc_says_delete_folder() {
    let path_info = PathInfo {
        path: "/tmp/bar".into(),
        name: "bar".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert!(trash.description.as_ref().unwrap().contains("folder"));
}

#[test]
fn path_move_to_trash_shortcut_is_cmd_delete() {
    let path_info = PathInfo {
        path: "/tmp/foo.txt".into(),
        name: "foo.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert_eq!(trash.shortcut.as_deref(), Some("⌘⌫"));
}

#[test]
fn path_move_to_trash_is_last_action() {
    let path_info = PathInfo {
        path: "/tmp/foo.txt".into(),
        name: "foo.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    assert_eq!(actions.last().unwrap().id, "move_to_trash");
}

// =====================================================================
// 4. File context: description wording for specific actions
// =====================================================================

#[test]
fn file_open_file_desc_says_default_application() {
    let file_info = FileInfo {
        path: "/tmp/doc.pdf".into(),
        name: "doc.pdf".into(),
        file_type: crate::file_search::FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    let open = actions.iter().find(|a| a.id == "open_file").unwrap();
    assert!(open
        .description
        .as_ref()
        .unwrap()
        .contains("default application"));
}

#[test]
fn file_reveal_desc_says_reveal_in_finder() {
    let file_info = FileInfo {
        path: "/tmp/doc.pdf".into(),
        name: "doc.pdf".into(),
        file_type: crate::file_search::FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
    assert!(reveal
        .description
        .as_ref()
        .unwrap()
        .contains("Reveal in Finder"));
}

#[test]
fn file_copy_path_desc_says_full_path() {
    let file_info = FileInfo {
        path: "/tmp/doc.pdf".into(),
        name: "doc.pdf".into(),
        file_type: crate::file_search::FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert!(cp.description.as_ref().unwrap().contains("full path"));
}

#[test]
fn file_copy_filename_desc_says_just_the_filename() {
    let file_info = FileInfo {
        path: "/tmp/doc.pdf".into(),
        name: "doc.pdf".into(),
        file_type: crate::file_search::FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert!(cf.description.as_ref().unwrap().contains("filename"));
}

// =====================================================================
// 5. AI command bar: new_chat, delete_chat, toggle_shortcuts_help details
// =====================================================================

#[test]
fn ai_bar_new_chat_shortcut_cmd_n() {
    let actions = get_ai_command_bar_actions();
    let nc = actions.iter().find(|a| a.id == "new_chat").unwrap();
    assert_eq!(nc.shortcut.as_deref(), Some("⌘N"));
}

#[test]
fn ai_bar_new_chat_icon_plus() {
    let actions = get_ai_command_bar_actions();
    let nc = actions.iter().find(|a| a.id == "new_chat").unwrap();
    assert_eq!(nc.icon, Some(IconName::Plus));
}

#[test]
fn ai_bar_delete_chat_shortcut_cmd_delete() {
    let actions = get_ai_command_bar_actions();
    let dc = actions.iter().find(|a| a.id == "delete_chat").unwrap();
    assert_eq!(dc.shortcut.as_deref(), Some("⌘⌫"));
}

#[test]
fn ai_bar_delete_chat_icon_trash() {
    let actions = get_ai_command_bar_actions();
    let dc = actions.iter().find(|a| a.id == "delete_chat").unwrap();
    assert_eq!(dc.icon, Some(IconName::Trash));
}

// =====================================================================
// 6. AI command bar: toggle_shortcuts_help and section distribution
// =====================================================================

#[test]
fn ai_bar_toggle_shortcuts_help_shortcut_cmd_slash() {
    let actions = get_ai_command_bar_actions();
    let tsh = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(tsh.shortcut.as_deref(), Some("⌘/"));
}

#[test]
fn ai_bar_toggle_shortcuts_help_icon_star() {
    let actions = get_ai_command_bar_actions();
    let tsh = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(tsh.icon, Some(IconName::Star));
}

#[test]
fn ai_bar_toggle_shortcuts_help_section_help() {
    let actions = get_ai_command_bar_actions();
    let tsh = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(tsh.section.as_deref(), Some("Help"));
}

#[test]
fn ai_bar_section_help_has_one_action() {
    let actions = get_ai_command_bar_actions();
    let help_count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Help"))
        .count();
    assert_eq!(help_count, 1);
}

// =====================================================================
// 7. AI command bar: Settings section has exactly one action
// =====================================================================

#[test]
fn ai_bar_settings_section_has_one_action() {
    let actions = get_ai_command_bar_actions();
    let settings_count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Settings"))
        .count();
    assert_eq!(settings_count, 1);
}

#[test]
fn ai_bar_settings_action_is_change_model() {
    let actions = get_ai_command_bar_actions();
    let settings: Vec<_> = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Settings"))
        .collect();
    assert_eq!(settings[0].id, "change_model");
}

#[test]
fn ai_bar_change_model_has_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let cm = actions.iter().find(|a| a.id == "change_model").unwrap();
    assert!(cm.shortcut.is_none());
}

#[test]
fn ai_bar_total_section_count_is_six() {
    let actions = get_ai_command_bar_actions();
    let mut sections: Vec<&str> = actions
        .iter()
        .filter_map(|a| a.section.as_deref())
        .collect();
    sections.dedup();
    // Response, Actions, Attachments, Export, Actions(again), Help, Settings
    // Unique sections: Response, Actions, Attachments, Export, Help, Settings = 6
    let unique: std::collections::HashSet<&str> = actions
        .iter()
        .filter_map(|a| a.section.as_deref())
        .collect();
    assert_eq!(unique.len(), 6);
}

// =====================================================================
// 8. Notes command bar: browse_notes details
// =====================================================================

#[test]
fn notes_browse_notes_shortcut_cmd_p() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let bn = actions.iter().find(|a| a.id == "browse_notes").unwrap();
    assert_eq!(bn.shortcut.as_deref(), Some("⌘P"));
}

#[test]
fn notes_browse_notes_icon_folder_open() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let bn = actions.iter().find(|a| a.id == "browse_notes").unwrap();
    assert_eq!(bn.icon, Some(IconName::FolderOpen));
}

#[test]
fn notes_browse_notes_section_notes() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let bn = actions.iter().find(|a| a.id == "browse_notes").unwrap();
    assert_eq!(bn.section.as_deref(), Some("Notes"));
}

#[test]
fn notes_browse_notes_always_present_even_in_trash() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "browse_notes"));
}
