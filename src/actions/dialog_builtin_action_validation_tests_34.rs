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
use crate::actions::dialog::{build_grouped_items_static, coerce_action_selection, ActionsDialog};
use crate::actions::types::{
    Action, ActionCategory, ActionsDialogConfig, AnchorPosition, ScriptInfo, SearchPosition,
    SectionStyle,
};
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

// =====================================================================
// 9. Notes command bar: export details
// =====================================================================

#[test]
fn notes_export_shortcut_shift_cmd_e() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let exp = actions.iter().find(|a| a.id == "export").unwrap();
    assert_eq!(exp.shortcut.as_deref(), Some("⇧⌘E"));
}

#[test]
fn notes_export_icon_arrow_right() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let exp = actions.iter().find(|a| a.id == "export").unwrap();
    assert_eq!(exp.icon, Some(IconName::ArrowRight));
}

#[test]
fn notes_export_section_export() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let exp = actions.iter().find(|a| a.id == "export").unwrap();
    assert_eq!(exp.section.as_deref(), Some("Export"));
}

#[test]
fn notes_export_absent_without_selection() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "export"));
}

// =====================================================================
// 10. Chat context: all 4 flag combinations (has_messages x has_response)
// =====================================================================

#[test]
fn chat_no_messages_no_response_has_only_models_and_continue() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "m1".into(),
            display_name: "Model1".into(),
            provider: "P1".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    // 1 model + continue_in_chat = 2
    assert_eq!(actions.len(), 2);
    assert!(!actions.iter().any(|a| a.id == "copy_response"));
    assert!(!actions.iter().any(|a| a.id == "clear_conversation"));
}

#[test]
fn chat_has_messages_no_response_has_clear_no_copy() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "m1".into(),
            display_name: "Model1".into(),
            provider: "P1".into(),
        }],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "clear_conversation"));
    assert!(!actions.iter().any(|a| a.id == "copy_response"));
}

#[test]
fn chat_no_messages_has_response_has_copy_no_clear() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "m1".into(),
            display_name: "Model1".into(),
            provider: "P1".into(),
        }],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "copy_response"));
    assert!(!actions.iter().any(|a| a.id == "clear_conversation"));
}

#[test]
fn chat_has_both_flags_has_copy_and_clear() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "m1".into(),
            display_name: "Model1".into(),
            provider: "P1".into(),
        }],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "copy_response"));
    assert!(actions.iter().any(|a| a.id == "clear_conversation"));
    // 1 model + continue + copy + clear = 4
    assert_eq!(actions.len(), 4);
}

// =====================================================================
// 11. Chat context: continue_in_chat always present regardless of flags
// =====================================================================

#[test]
fn chat_continue_in_chat_always_present() {
    for (has_messages, has_response) in [(false, false), (true, false), (false, true), (true, true)]
    {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages,
            has_response,
        };
        let actions = get_chat_context_actions(&info);
        assert!(
            actions.iter().any(|a| a.id == "continue_in_chat"),
            "continue_in_chat missing for has_messages={has_messages}, has_response={has_response}"
        );
    }
}

#[test]
fn chat_continue_in_chat_shortcut_cmd_enter() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let cont = actions.iter().find(|a| a.id == "continue_in_chat").unwrap();
    assert_eq!(cont.shortcut.as_deref(), Some("⌘↵"));
}

#[test]
fn chat_continue_in_chat_desc_mentions_ai_chat() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let cont = actions.iter().find(|a| a.id == "continue_in_chat").unwrap();
    assert!(cont.description.as_ref().unwrap().contains("AI Chat"));
}

#[test]
fn chat_clear_conversation_shortcut_cmd_delete() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let clear = actions
        .iter()
        .find(|a| a.id == "clear_conversation")
        .unwrap();
    assert_eq!(clear.shortcut.as_deref(), Some("⌘⌫"));
}

// =====================================================================
// 12. Chat context: copy_response shortcut is ⌘C
// =====================================================================

#[test]
fn chat_copy_response_shortcut_cmd_c() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let cr = actions.iter().find(|a| a.id == "copy_response").unwrap();
    assert_eq!(cr.shortcut.as_deref(), Some("⌘C"));
}

#[test]
fn chat_copy_response_title() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let cr = actions.iter().find(|a| a.id == "copy_response").unwrap();
    assert_eq!(cr.title, "Copy Last Response");
}

#[test]
fn chat_copy_response_desc_mentions_assistant() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let cr = actions.iter().find(|a| a.id == "copy_response").unwrap();
    assert!(cr.description.as_ref().unwrap().contains("assistant"));
}

// =====================================================================
// 13. Script context: scriptlet with shortcut gets update/remove not add
// =====================================================================

#[test]
fn scriptlet_with_shortcut_has_update_shortcut() {
    let script = ScriptInfo::scriptlet(
        "My Scriptlet",
        "/path/bundle.md",
        Some("cmd+s".into()),
        None,
    );
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "update_shortcut"));
    assert!(!actions.iter().any(|a| a.id == "add_shortcut"));
}

#[test]
fn scriptlet_with_shortcut_has_remove_shortcut() {
    let script = ScriptInfo::scriptlet(
        "My Scriptlet",
        "/path/bundle.md",
        Some("cmd+s".into()),
        None,
    );
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
}

#[test]
fn scriptlet_without_shortcut_has_add_shortcut() {
    let script = ScriptInfo::scriptlet("My Scriptlet", "/path/bundle.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "add_shortcut"));
    assert!(!actions.iter().any(|a| a.id == "update_shortcut"));
}

#[test]
fn scriptlet_with_alias_has_update_alias() {
    let script = ScriptInfo::scriptlet("My Scriptlet", "/path/bundle.md", None, Some("ms".into()));
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "update_alias"));
    assert!(actions.iter().any(|a| a.id == "remove_alias"));
    assert!(!actions.iter().any(|a| a.id == "add_alias"));
}

// =====================================================================
// 14. Scriptlet context: copy_content desc mentions entire file
// =====================================================================

#[test]
fn scriptlet_copy_content_desc_mentions_entire_file() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert!(cc.description.as_ref().unwrap().contains("entire file"));
}

#[test]
fn scriptlet_copy_content_shortcut_opt_cmd_c() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(cc.shortcut.as_deref(), Some("⌘⌥C"));
}

#[test]
fn scriptlet_edit_scriptlet_shortcut_cmd_e() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let edit = actions.iter().find(|a| a.id == "edit_scriptlet").unwrap();
    assert_eq!(edit.shortcut.as_deref(), Some("⌘E"));
}

#[test]
fn scriptlet_edit_scriptlet_desc_mentions_editor() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let edit = actions.iter().find(|a| a.id == "edit_scriptlet").unwrap();
    assert!(edit.description.as_ref().unwrap().contains("$EDITOR"));
}

// =====================================================================
// 15. builders::format_shortcut_hint vs ActionsDialog::format_shortcut_hint
// =====================================================================

#[test]
fn builders_format_basic_cmd_c() {
    // builders::format_shortcut_hint is private, but we test via to_deeplink_name
    // and scriptlet-defined actions. The ActionsDialog version handles more aliases.
    let result = ActionsDialog::format_shortcut_hint("cmd+c");
    assert_eq!(result, "⌘C");
}

#[test]
fn dialog_format_handles_control_alias() {
    let result = ActionsDialog::format_shortcut_hint("control+x");
    assert_eq!(result, "⌃X");
}

#[test]
fn dialog_format_handles_option_alias() {
    let result = ActionsDialog::format_shortcut_hint("option+v");
    assert_eq!(result, "⌥V");
}

#[test]
fn dialog_format_handles_backspace_key() {
    let result = ActionsDialog::format_shortcut_hint("cmd+backspace");
    assert_eq!(result, "⌘⌫");
}

// =====================================================================
// 16. to_deeplink_name: various transformations
// =====================================================================

#[test]
fn deeplink_underscores_become_hyphens() {
    assert_eq!(to_deeplink_name("hello_world"), "hello-world");
}

#[test]
fn deeplink_multiple_underscores_collapse() {
    assert_eq!(to_deeplink_name("a___b"), "a-b");
}

#[test]
fn deeplink_mixed_punctuation() {
    assert_eq!(to_deeplink_name("Hello, World!"), "hello-world");
}

#[test]
fn deeplink_empty_string() {
    assert_eq!(to_deeplink_name(""), "");
}

// =====================================================================
// 17. Constants: UI dimensions validation
// =====================================================================

#[test]
fn constant_popup_width_320() {
    use crate::actions::constants::POPUP_WIDTH;
    assert_eq!(POPUP_WIDTH, 320.0);
}

#[test]
fn constant_popup_max_height_400() {
    use crate::actions::constants::POPUP_MAX_HEIGHT;
    assert_eq!(POPUP_MAX_HEIGHT, 400.0);
}

#[test]
fn constant_action_item_height_36() {
    use crate::actions::constants::ACTION_ITEM_HEIGHT;
    assert_eq!(ACTION_ITEM_HEIGHT, 36.0);
}

#[test]
fn constant_search_input_height_44() {
    use crate::actions::constants::SEARCH_INPUT_HEIGHT;
    assert_eq!(SEARCH_INPUT_HEIGHT, 44.0);
}

// =====================================================================
// 18. CommandBarConfig notes_style preset
// =====================================================================

#[test]
fn notes_style_search_position_top() {
    let config = CommandBarConfig::notes_style();
    assert!(matches!(
        config.dialog_config.search_position,
        SearchPosition::Top
    ));
}

#[test]
fn notes_style_section_style_separators() {
    let config = CommandBarConfig::notes_style();
    assert!(matches!(
        config.dialog_config.section_style,
        SectionStyle::Separators
    ));
}

#[test]
fn notes_style_show_icons_true() {
    let config = CommandBarConfig::notes_style();
    assert!(config.dialog_config.show_icons);
}

#[test]
fn notes_style_show_footer_true() {
    let config = CommandBarConfig::notes_style();
    assert!(config.dialog_config.show_footer);
}

// =====================================================================
// 19. Global actions: always empty
// =====================================================================

#[test]
fn global_actions_empty() {
    use crate::actions::builders::get_global_actions;
    let actions = get_global_actions();
    assert!(actions.is_empty());
}

#[test]
fn global_actions_returns_vec() {
    use crate::actions::builders::get_global_actions;
    let actions = get_global_actions();
    assert_eq!(actions.len(), 0);
}

// =====================================================================
// 20. Action::with_shortcut_opt with None leaves shortcut unset
// =====================================================================

#[test]
fn action_with_shortcut_opt_none_leaves_none() {
    let action =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
    assert!(action.shortcut.is_none());
    assert!(action.shortcut_lower.is_none());
}

#[test]
fn action_with_shortcut_opt_some_sets_both() {
    let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
        .with_shortcut_opt(Some("⌘X".into()));
    assert_eq!(action.shortcut.as_deref(), Some("⌘X"));
    assert_eq!(action.shortcut_lower.as_deref(), Some("⌘x"));
}

#[test]
fn action_with_shortcut_sets_shortcut_lower() {
    let action =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut("⇧⌘K");
    assert_eq!(action.shortcut.as_deref(), Some("⇧⌘K"));
    assert_eq!(action.shortcut_lower.as_deref(), Some("⇧⌘k"));
}

#[test]
fn action_new_has_no_shortcut_by_default() {
    let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
    assert!(action.shortcut.is_none());
    assert!(action.shortcut_lower.is_none());
}

// =====================================================================
// 21. Note switcher: section assignment based on pinned status
// =====================================================================

#[test]
fn note_switcher_pinned_section_is_pinned() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid-1".into(),
        title: "Pinned Note".into(),
        char_count: 100,
        is_current: false,
        is_pinned: true,
        preview: "Some text".into(),
        relative_time: "1m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
}

#[test]
fn note_switcher_unpinned_section_is_recent() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid-2".into(),
        title: "Regular Note".into(),
        char_count: 50,
        is_current: false,
        is_pinned: false,
        preview: "Some text".into(),
        relative_time: "2h ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Recent"));
}

#[test]
fn note_switcher_mixed_pinned_and_recent() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "a".into(),
            title: "A".into(),
            char_count: 10,
            is_current: false,
            is_pinned: true,
            preview: "".into(),
            relative_time: "".into(),
        },
        NoteSwitcherNoteInfo {
            id: "b".into(),
            title: "B".into(),
            char_count: 20,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
    assert_eq!(actions[1].section.as_deref(), Some("Recent"));
}

// =====================================================================
// 22. Clipboard: pin/unpin share the same shortcut ⇧⌘P
// =====================================================================

#[test]
fn clipboard_pin_shortcut_shift_cmd_p() {
    let entry = ClipboardEntryInfo {
        id: "pin-1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let pin = actions.iter().find(|a| a.id == "clipboard_pin").unwrap();
    assert_eq!(pin.shortcut.as_deref(), Some("⇧⌘P"));
}

#[test]
fn clipboard_unpin_shortcut_shift_cmd_p() {
    let entry = ClipboardEntryInfo {
        id: "pin-2".into(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "test".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let unpin = actions.iter().find(|a| a.id == "clipboard_unpin").unwrap();
    assert_eq!(unpin.shortcut.as_deref(), Some("⇧⌘P"));
}

#[test]
fn clipboard_pin_title_is_pin_entry() {
    let entry = ClipboardEntryInfo {
        id: "pin-3".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let pin = actions.iter().find(|a| a.id == "clipboard_pin").unwrap();
    assert_eq!(pin.title, "Pin Entry");
}

#[test]
fn clipboard_unpin_title_is_unpin_entry() {
    let entry = ClipboardEntryInfo {
        id: "pin-4".into(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "test".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let unpin = actions.iter().find(|a| a.id == "clipboard_unpin").unwrap();
    assert_eq!(unpin.title, "Unpin Entry");
}

// =====================================================================
// 23. Script context: agent action set details
// =====================================================================

#[test]
fn agent_edit_title_is_edit_agent() {
    let mut script = ScriptInfo::new("MyAgent", "/path/agent.md");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.title, "Edit Agent");
}

#[test]
fn agent_edit_desc_mentions_agent_file() {
    let mut script = ScriptInfo::new("MyAgent", "/path/agent.md");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert!(edit.description.as_ref().unwrap().contains("agent"));
}

#[test]
fn agent_has_reveal_in_finder() {
    let mut script = ScriptInfo::new("MyAgent", "/path/agent.md");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
}

#[test]
fn agent_has_no_view_logs() {
    let mut script = ScriptInfo::new("MyAgent", "/path/agent.md");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

// =====================================================================
// 24. New chat: preset icon is preserved
// =====================================================================

#[test]
fn new_chat_preset_icon_preserved() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].icon, Some(IconName::Star));
}

#[test]
fn new_chat_preset_section_is_presets() {
    let presets = vec![NewChatPresetInfo {
        id: "code".into(),
        name: "Code".into(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].section.as_deref(), Some("Presets"));
}

#[test]
fn new_chat_preset_desc_is_none() {
    let presets = vec![NewChatPresetInfo {
        id: "code".into(),
        name: "Code".into(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert!(actions[0].description.is_none());
}

#[test]
fn new_chat_model_desc_is_provider_display_name() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt-4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].description.as_deref(), Some("OpenAI"));
}

// =====================================================================
// 25. Note switcher: empty preview uses relative_time or char count
// =====================================================================

#[test]
fn note_switcher_empty_preview_with_time_shows_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n1".into(),
        title: "Note".into(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "5m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("5m ago"));
}

#[test]
fn note_switcher_empty_preview_empty_time_shows_chars() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n2".into(),
        title: "Note".into(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("42 chars"));
}

#[test]
fn note_switcher_preview_with_time_has_separator() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n3".into(),
        title: "Note".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "Hello world".into(),
        relative_time: "1h ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(desc.contains(" · "));
    assert!(desc.contains("Hello world"));
    assert!(desc.contains("1h ago"));
}

#[test]
fn note_switcher_preview_without_time_no_separator() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n4".into(),
        title: "Note".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "Hello world".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(!desc.contains(" · "));
    assert_eq!(desc, "Hello world");
}

// =====================================================================
// 26. Clipboard: upload_cleanshot shortcut is ⇧⌘U (macOS only)
// =====================================================================

#[cfg(target_os = "macos")]
#[test]
fn clipboard_upload_cleanshot_shortcut_shift_cmd_u() {
    let entry = ClipboardEntryInfo {
        id: "uc-1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let upload = actions
        .iter()
        .find(|a| a.id == "clipboard_upload_cleanshot")
        .unwrap();
    assert_eq!(upload.shortcut.as_deref(), Some("⇧⌘U"));
}

#[cfg(target_os = "macos")]
#[test]
fn clipboard_upload_cleanshot_title() {
    let entry = ClipboardEntryInfo {
        id: "uc-2".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let upload = actions
        .iter()
        .find(|a| a.id == "clipboard_upload_cleanshot")
        .unwrap();
    assert_eq!(upload.title, "Upload to CleanShot X");
}

#[cfg(target_os = "macos")]
#[test]
fn clipboard_upload_cleanshot_desc_mentions_cloud() {
    let entry = ClipboardEntryInfo {
        id: "uc-3".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let upload = actions
        .iter()
        .find(|a| a.id == "clipboard_upload_cleanshot")
        .unwrap();
    assert!(upload.description.as_ref().unwrap().contains("Cloud"));
}

#[cfg(target_os = "macos")]
#[test]
fn clipboard_upload_cleanshot_absent_for_text() {
    let entry = ClipboardEntryInfo {
        id: "uc-4".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_upload_cleanshot"));
}

// =====================================================================
// 27. Path context: open_in_editor and open_in_finder details
// =====================================================================

#[test]
fn path_open_in_editor_shortcut_cmd_e() {
    let path_info = PathInfo {
        path: "/tmp/code.rs".into(),
        name: "code.rs".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let editor = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
    assert_eq!(editor.shortcut.as_deref(), Some("⌘E"));
}

#[test]
fn path_open_in_editor_desc_mentions_editor() {
    let path_info = PathInfo {
        path: "/tmp/code.rs".into(),
        name: "code.rs".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let editor = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
    assert!(editor.description.as_ref().unwrap().contains("$EDITOR"));
}

#[test]
fn path_open_in_finder_shortcut_cmd_shift_f() {
    let path_info = PathInfo {
        path: "/tmp/code.rs".into(),
        name: "code.rs".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let finder = actions.iter().find(|a| a.id == "open_in_finder").unwrap();
    assert_eq!(finder.shortcut.as_deref(), Some("⌘⇧F"));
}

#[test]
fn path_open_in_finder_desc_mentions_finder() {
    let path_info = PathInfo {
        path: "/tmp/code.rs".into(),
        name: "code.rs".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let finder = actions.iter().find(|a| a.id == "open_in_finder").unwrap();
    assert!(finder.description.as_ref().unwrap().contains("Finder"));
}

// =====================================================================
// 28. Script context: copy_content shortcut ⌘⌥C for all types
// =====================================================================

#[test]
fn script_copy_content_shortcut_opt_cmd_c() {
    let script = ScriptInfo::new("Test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(cc.shortcut.as_deref(), Some("⌘⌥C"));
}

#[test]
fn agent_copy_content_shortcut_opt_cmd_c() {
    let mut script = ScriptInfo::new("Agent", "/path/agent.md");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(cc.shortcut.as_deref(), Some("⌘⌥C"));
}

#[test]
fn script_copy_content_desc_mentions_entire_file() {
    let script = ScriptInfo::new("Test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert!(cc.description.as_ref().unwrap().contains("entire file"));
}

#[test]
fn agent_copy_content_desc_mentions_entire_file() {
    let mut script = ScriptInfo::new("Agent", "/path/agent.md");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert!(cc.description.as_ref().unwrap().contains("entire file"));
}

// =====================================================================
// 29. score_action: title_lower and description_lower used for matching
// =====================================================================

#[test]
fn score_action_matches_case_insensitive() {
    let action = Action::new(
        "test",
        "Edit Script",
        Some("Open in editor".into()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "edit");
    assert!(score >= 100, "Prefix match should score >=100, got {score}");
}

#[test]
fn score_action_description_bonus_adds_points() {
    let action = Action::new(
        "test",
        "Open File",
        Some("Open in editor for editing".into()),
        ActionCategory::ScriptContext,
    );
    // "editor" is not in title but is in description
    let score = ActionsDialog::score_action(&action, "editor");
    assert!(
        score >= 15,
        "Description match should score >=15, got {score}"
    );
}

#[test]
fn score_action_no_match_returns_zero() {
    let action = Action::new(
        "test",
        "Edit Script",
        Some("Open in editor".into()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "zzzzz");
    assert_eq!(score, 0);
}

#[test]
fn score_action_shortcut_bonus() {
    let action =
        Action::new("test", "Something", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
    // "⌘e" matches shortcut_lower "⌘e"
    let score = ActionsDialog::score_action(&action, "⌘e");
    assert!(score >= 10, "Shortcut match should score >=10, got {score}");
}

// =====================================================================
// 30. Cross-context: all clipboard text actions have ScriptContext category
// =====================================================================

#[test]
fn all_clipboard_text_actions_have_script_context_category() {
    let entry = ClipboardEntryInfo {
        id: "cat-1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    for action in &actions {
        assert_eq!(
            action.category,
            ActionCategory::ScriptContext,
            "Action '{}' should have ScriptContext category",
            action.id
        );
    }
}

#[test]
fn all_path_actions_have_script_context_category() {
    let path_info = PathInfo {
        path: "/tmp/foo".into(),
        name: "foo".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    for action in &actions {
        assert_eq!(
            action.category,
            ActionCategory::ScriptContext,
            "Action '{}' should have ScriptContext category",
            action.id
        );
    }
}

#[test]
fn all_ai_bar_actions_have_script_context_category() {
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        assert_eq!(
            action.category,
            ActionCategory::ScriptContext,
            "Action '{}' should have ScriptContext category",
            action.id
        );
    }
}

#[test]
fn all_note_switcher_actions_have_script_context_category() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid".into(),
        title: "Note".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    for action in &actions {
        assert_eq!(
            action.category,
            ActionCategory::ScriptContext,
            "Action '{}' should have ScriptContext category",
            action.id
        );
    }
}
