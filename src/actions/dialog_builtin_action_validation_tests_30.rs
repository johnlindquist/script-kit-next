// --- merged from part_01.rs ---
//! Batch 30: Builtin action validation tests
//!
//! 30 categories validating random built-in action behaviors across
//! script, clipboard, file, path, AI, notes, chat, and new-chat contexts.

use crate::actions::builders::{
    get_ai_command_bar_actions, get_chat_context_actions, get_clipboard_history_context_actions,
    get_file_context_actions, get_new_chat_actions, get_note_switcher_actions,
    get_notes_command_bar_actions, get_path_context_actions, get_script_context_actions,
    get_scriptlet_context_actions_with_custom, to_deeplink_name, ChatModelInfo, ChatPromptInfo,
    ClipboardEntryInfo, NewChatModelInfo, NewChatPresetInfo, NoteSwitcherNoteInfo, NotesInfo,
};
use crate::actions::command_bar::CommandBarConfig;
use crate::actions::dialog::{build_grouped_items_static, coerce_action_selection, ActionsDialog};
use crate::actions::types::{Action, ActionCategory, AnchorPosition, SearchPosition, SectionStyle};
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;

// ---------------------------------------------------------------------------
// 1. Script context: copy_content description wording is consistent
// ---------------------------------------------------------------------------
#[test]
fn batch30_script_copy_content_desc_says_entire_file() {
    let script = crate::actions::types::ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert!(
        cc.description.as_ref().unwrap().contains("entire file"),
        "copy_content desc should mention 'entire file', got: {:?}",
        cc.description
    );
}

#[test]
fn batch30_scriptlet_copy_content_desc_says_entire_file() {
    let script = crate::actions::types::ScriptInfo::scriptlet("x", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert!(cc.description.as_ref().unwrap().contains("entire file"));
}

#[test]
fn batch30_agent_copy_content_desc_says_entire_file() {
    let mut script = crate::actions::types::ScriptInfo::new("agent", "/p/agent.ts");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert!(cc.description.as_ref().unwrap().contains("entire file"));
}

#[test]
fn batch30_builtin_has_no_copy_content() {
    let script = crate::actions::types::ScriptInfo::builtin("Settings");
    let actions = get_script_context_actions(&script);
    assert!(!actions.iter().any(|a| a.id == "copy_content"));
}

// ---------------------------------------------------------------------------
// 2. Clipboard: image-only actions absent for text entries
// ---------------------------------------------------------------------------
#[test]
fn batch30_clipboard_text_no_ocr() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hi".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clip:clipboard_ocr"));
}

#[test]
fn batch30_clipboard_text_no_open_with() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clip:clipboard_open_with"));
}

#[test]
fn batch30_clipboard_text_no_annotate_cleanshot() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions
        .iter()
        .any(|a| a.id == "clip:clipboard_annotate_cleanshot"));
}

#[test]
fn batch30_clipboard_text_no_upload_cleanshot() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clip:clipboard_upload_cleanshot"));
}

// ---------------------------------------------------------------------------
// 3. Clipboard: image entry has OCR and macOS image actions
// ---------------------------------------------------------------------------
#[test]
fn batch30_clipboard_image_has_ocr() {
    let entry = ClipboardEntryInfo {
        id: "i".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clip:clipboard_ocr"));
}

#[cfg(target_os = "macos")]
#[test]
fn batch30_clipboard_image_has_open_with_macos() {
    let entry = ClipboardEntryInfo {
        id: "i".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clip:clipboard_open_with"));
}

#[cfg(target_os = "macos")]
#[test]
fn batch30_clipboard_image_has_annotate_cleanshot_macos() {
    let entry = ClipboardEntryInfo {
        id: "i".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions
        .iter()
        .any(|a| a.id == "clip:clipboard_annotate_cleanshot"));
}

#[cfg(target_os = "macos")]
#[test]
fn batch30_clipboard_image_annotate_cleanshot_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "i".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let a = actions
        .iter()
        .find(|a| a.id == "clip:clipboard_annotate_cleanshot")
        .unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⇧⌘A"));
}

// ---------------------------------------------------------------------------
// 4. File context: directory has no quick_look on macOS
// ---------------------------------------------------------------------------
#[cfg(target_os = "macos")]
#[test]
fn batch30_file_dir_no_quick_look() {
    let info = FileInfo {
        path: "/tmp/dir".into(),
        name: "dir".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "file:quick_look"));
}

#[cfg(target_os = "macos")]
#[test]
fn batch30_file_regular_has_quick_look() {
    let info = FileInfo {
        path: "/tmp/f.txt".into(),
        name: "f.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "file:quick_look"));
}

#[cfg(target_os = "macos")]
#[test]
fn batch30_file_quick_look_shortcut_is_cmd_y() {
    let info = FileInfo {
        path: "/tmp/f.txt".into(),
        name: "f.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    let ql = actions.iter().find(|a| a.id == "file:quick_look").unwrap();
    assert_eq!(ql.shortcut.as_deref(), Some("⌘Y"));
}

#[test]
fn batch30_file_dir_has_open_directory() {
    let info = FileInfo {
        path: "/tmp/dir".into(),
        name: "dir".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "file:open_directory"));
    assert!(!actions.iter().any(|a| a.id == "file:open_file"));
}

// ---------------------------------------------------------------------------
// 5. Path context: total action count for file vs dir
// ---------------------------------------------------------------------------
#[test]
fn batch30_path_file_has_7_actions() {
    let info = PathInfo {
        path: "/tmp/f.txt".into(),
        name: "f.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    assert_eq!(
        actions.len(),
        7,
        "Path file should have 7 actions: select_file, copy_path, open_in_finder, open_in_editor, open_in_terminal, copy_filename, move_to_trash"
    );
}

#[test]
fn batch30_path_dir_has_7_actions() {
    let info = PathInfo {
        path: "/tmp/d".into(),
        name: "d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&info);
    assert_eq!(
        actions.len(),
        7,
        "Path dir should have 7 actions: open_directory, copy_path, open_in_finder, open_in_editor, open_in_terminal, copy_filename, move_to_trash"
    );
}

#[test]
fn batch30_path_file_first_action_is_select_file() {
    let info = PathInfo {
        path: "/tmp/f.txt".into(),
        name: "f.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    assert_eq!(actions[0].id, "file:select_file");
}

#[test]
fn batch30_path_dir_first_action_is_open_directory() {
    let info = PathInfo {
        path: "/tmp/d".into(),
        name: "d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&info);
    assert_eq!(actions[0].id, "file:open_directory");
}

// ---------------------------------------------------------------------------
// 6. Path context: open_in_terminal shortcut is ⌘T
// ---------------------------------------------------------------------------
#[test]
fn batch30_path_open_in_terminal_shortcut() {
    let info = PathInfo {
        path: "/tmp/f".into(),
        name: "f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    let t = actions.iter().find(|a| a.id == "file:open_in_terminal").unwrap();
    assert_eq!(t.shortcut.as_deref(), Some("⌘T"));
}

#[test]
fn batch30_path_open_in_terminal_desc_mentions_terminal() {
    let info = PathInfo {
        path: "/tmp/f".into(),
        name: "f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    let t = actions.iter().find(|a| a.id == "file:open_in_terminal").unwrap();
    assert!(t
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("terminal"));
}

#[test]
fn batch30_path_open_in_finder_shortcut() {
    let info = PathInfo {
        path: "/tmp/f".into(),
        name: "f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    let f = actions.iter().find(|a| a.id == "file:open_in_finder").unwrap();
    assert_eq!(f.shortcut.as_deref(), Some("⌘⇧F"));
}

#[test]
fn batch30_path_copy_path_shortcut() {
    let info = PathInfo {
        path: "/tmp/f".into(),
        name: "f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    let cp = actions.iter().find(|a| a.id == "file:copy_path").unwrap();
    assert_eq!(cp.shortcut.as_deref(), Some("⌘⇧C"));
}

// ---------------------------------------------------------------------------
// 7. AI command bar: all 12 actions have unique IDs
// ---------------------------------------------------------------------------
#[test]
fn batch30_ai_bar_12_actions() {
    let actions = get_ai_command_bar_actions();
    assert_eq!(actions.len(), 12);
}

#[test]
fn batch30_ai_bar_all_ids_unique() {
    let actions = get_ai_command_bar_actions();
    let mut ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 12, "All 12 AI bar action IDs must be unique");
}

#[test]
fn batch30_ai_bar_all_have_section() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(
            a.section.is_some(),
            "AI bar action '{}' should have a section",
            a.id
        );
    }
}

#[test]
fn batch30_ai_bar_all_have_icon() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(
            a.icon.is_some(),
            "AI bar action '{}' should have an icon",
            a.id
        );
    }
}

// ---------------------------------------------------------------------------
// 8. AI command bar: branch_from_last has no shortcut
// ---------------------------------------------------------------------------
#[test]
fn batch30_ai_bar_branch_from_last_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let b = actions.iter().find(|a| a.id == "chat:branch_from_last").unwrap();
    assert!(b.shortcut.is_none());
}

#[test]
fn batch30_ai_bar_change_model_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let cm = actions.iter().find(|a| a.id == "chat:change_model").unwrap();
    assert!(cm.shortcut.is_none());
}

#[test]
fn batch30_ai_bar_branch_from_last_icon_arrowright() {
    let actions = get_ai_command_bar_actions();
    let b = actions.iter().find(|a| a.id == "chat:branch_from_last").unwrap();
    assert_eq!(b.icon, Some(IconName::ArrowRight));
}

#[test]
fn batch30_ai_bar_change_model_icon_settings() {
    let actions = get_ai_command_bar_actions();
    let cm = actions.iter().find(|a| a.id == "chat:change_model").unwrap();
    assert_eq!(cm.icon, Some(IconName::Settings));
}

// --- merged from part_02.rs ---

// ---------------------------------------------------------------------------
// 9. Chat context: current model gets ✓ suffix
// ---------------------------------------------------------------------------
#[test]
fn batch30_chat_current_model_has_check() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".into()),
        available_models: vec![ChatModelInfo {
            id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let m = actions
        .iter()
        .find(|a| a.id == "chat:select_model_gpt4")
        .unwrap();
    assert!(m.title.contains("✓"), "Current model title should have ✓");
}

#[test]
fn batch30_chat_non_current_model_no_check() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".into()),
        available_models: vec![ChatModelInfo {
            id: "claude".into(),
            display_name: "Claude".into(),
            provider: "Anthropic".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let m = actions
        .iter()
        .find(|a| a.id == "chat:select_model_claude")
        .unwrap();
    assert!(!m.title.contains("✓"));
}

#[test]
fn batch30_chat_no_current_model_no_check() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let m = actions
        .iter()
        .find(|a| a.id == "chat:select_model_gpt4")
        .unwrap();
    assert!(!m.title.contains("✓"));
}

#[test]
fn batch30_chat_model_desc_says_via_provider() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "c3".into(),
            display_name: "Claude 3".into(),
            provider: "Anthropic".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let m = actions.iter().find(|a| a.id == "chat:select_model_c3").unwrap();
    assert_eq!(m.description.as_deref(), Some("Uses Anthropic"));
}

// ---------------------------------------------------------------------------
// 10. Notes command bar: new_note always present
// ---------------------------------------------------------------------------
#[test]
fn batch30_notes_new_note_present_full_mode() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "new_note"));
}

#[test]
fn batch30_notes_new_note_present_in_trash() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "new_note"));
}

#[test]
fn batch30_notes_new_note_shortcut_cmd_n() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let nn = actions.iter().find(|a| a.id == "new_note").unwrap();
    assert_eq!(nn.shortcut.as_deref(), Some("⌘N"));
}

#[test]
fn batch30_notes_new_note_icon_plus() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let nn = actions.iter().find(|a| a.id == "new_note").unwrap();
    assert_eq!(nn.icon, Some(IconName::Plus));
}

// ---------------------------------------------------------------------------
// 11. Notes command bar: full mode action count
// ---------------------------------------------------------------------------
#[test]
fn batch30_notes_full_mode_10_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note, duplicate_note, browse_notes, find_in_note, format,
    // copy_note_as, copy_deeplink, create_quicklink, export, enable_auto_sizing = 10
    assert_eq!(actions.len(), 10);
}

#[test]
fn batch30_notes_full_mode_auto_sizing_enabled_9_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    // same minus enable_auto_sizing = 9
    assert_eq!(actions.len(), 9);
}

#[test]
fn batch30_notes_no_selection_3_actions() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note, browse_notes, enable_auto_sizing = 3
    assert_eq!(actions.len(), 3);
}

#[test]
fn batch30_notes_trash_3_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note, restore_note, permanently_delete_note, browse_notes, enable_auto_sizing = 5
    assert_eq!(actions.len(), 5);
}

// ---------------------------------------------------------------------------
// 12. Note switcher: pinned note gets StarFilled icon
// ---------------------------------------------------------------------------
#[test]
fn batch30_note_switcher_pinned_icon_star_filled() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Pinned Note".into(),
        char_count: 50,
        is_current: false,
        is_pinned: true,
        preview: "Some preview".into(),
        relative_time: "1h ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn batch30_note_switcher_pinned_section() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "P".into(),
        char_count: 10,
        is_current: false,
        is_pinned: true,
        preview: "x".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
}

#[test]
fn batch30_note_switcher_pinned_and_current_icon_is_star() {
    // pinned trumps current for icon
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Both".into(),
        char_count: 10,
        is_current: true,
        is_pinned: true,
        preview: "x".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn batch30_note_switcher_regular_icon_file() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Regular".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "x".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::File));
}

// ---------------------------------------------------------------------------
// 13. Note switcher: preview truncation boundary at 60 chars
// ---------------------------------------------------------------------------
#[test]
fn batch30_note_switcher_60_chars_no_truncation() {
    let preview = "a".repeat(60);
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".into(),
        title: "T".into(),
        char_count: 60,
        is_current: false,
        is_pinned: false,
        preview,
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(!desc.contains("…"), "Exactly 60 chars should not truncate");
}

#[test]
fn batch30_note_switcher_61_chars_truncated() {
    let preview = "a".repeat(61);
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".into(),
        title: "T".into(),
        char_count: 61,
        is_current: false,
        is_pinned: false,
        preview,
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(desc.contains("…"), "61 chars should truncate with …");
}

#[test]
fn batch30_note_switcher_short_preview_no_truncation() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".into(),
        title: "T".into(),
        char_count: 5,
        is_current: false,
        is_pinned: false,
        preview: "hello".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(!desc.contains("…"));
    assert!(desc.contains("hello"));
}

// ---------------------------------------------------------------------------
// 14. Note switcher: empty preview falls back to char count
// ---------------------------------------------------------------------------
#[test]
fn batch30_note_switcher_empty_preview_empty_time_shows_chars() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".into(),
        title: "T".into(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert_eq!(desc, "42 chars");
}

#[test]
fn batch30_note_switcher_empty_preview_with_time_shows_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".into(),
        title: "T".into(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "3d ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert_eq!(desc, "3d ago");
}

#[test]
fn batch30_note_switcher_singular_char() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".into(),
        title: "T".into(),
        char_count: 1,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert_eq!(desc, "1 char");
}

#[test]
fn batch30_note_switcher_zero_chars() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".into(),
        title: "T".into(),
        char_count: 0,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert_eq!(desc, "0 chars");
}

// ---------------------------------------------------------------------------
// 15. New chat: empty inputs produce empty results
// ---------------------------------------------------------------------------
#[test]
fn batch30_new_chat_all_empty() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

#[test]
fn batch30_new_chat_only_models() {
    let models = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "p".into(),
        provider_display_name: "Provider".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Models"));
}

#[test]
fn batch30_new_chat_only_presets() {
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
fn batch30_new_chat_only_last_used() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "M1".into(),
        provider: "p".into(),
        provider_display_name: "Prov".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
}

// ---------------------------------------------------------------------------
// 16. New chat: section ordering is last_used → presets → models
// ---------------------------------------------------------------------------
#[test]
fn batch30_new_chat_section_ordering() {
    let last_used = vec![NewChatModelInfo {
        model_id: "lu".into(),
        display_name: "LU".into(),
        provider: "p".into(),
        provider_display_name: "P".into(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "gen".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "m".into(),
        display_name: "M".into(),
        provider: "p".into(),
        provider_display_name: "P".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &presets, &models);
    assert_eq!(actions.len(), 3);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
    assert_eq!(actions[1].section.as_deref(), Some("Presets"));
    assert_eq!(actions[2].section.as_deref(), Some("Models"));
}

// --- merged from part_03.rs ---

#[test]
fn batch30_new_chat_preset_desc_is_none() {
    let presets = vec![NewChatPresetInfo {
        id: "gen".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].description.as_deref(), Some("Uses General preset"));
}

#[test]
fn batch30_new_chat_last_used_desc_is_provider() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m".into(),
        display_name: "M".into(),
        provider: "p".into(),
        provider_display_name: "MyProvider".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].description.as_deref(), Some("Uses MyProvider"));
}

#[test]
fn batch30_new_chat_model_desc_is_provider() {
    let models = vec![NewChatModelInfo {
        model_id: "m".into(),
        display_name: "M".into(),
        provider: "p".into(),
        provider_display_name: "ProvDisplay".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].description.as_deref(), Some("Uses ProvDisplay"));
}

// ---------------------------------------------------------------------------
// 17. to_deeplink_name: various edge cases
// ---------------------------------------------------------------------------
#[test]
fn batch30_deeplink_name_unicode_preserved() {
    // non-ASCII characters are percent-encoded
    let result = to_deeplink_name("café");
    assert_eq!(result, "caf%C3%A9");
}

#[test]
fn batch30_deeplink_name_all_special_chars() {
    let result = to_deeplink_name("!@#$%^");
    assert_eq!(result, "_unnamed");
}

#[test]
fn batch30_deeplink_name_mixed_case_lowered() {
    let result = to_deeplink_name("MyScript");
    assert_eq!(result, "myscript");
}

#[test]
fn batch30_deeplink_name_numbers_preserved() {
    let result = to_deeplink_name("test123");
    assert_eq!(result, "test123");
}

// ---------------------------------------------------------------------------
// 18. Script context: action verb propagates to run_script title
// ---------------------------------------------------------------------------
#[test]
fn batch30_script_verb_run_default() {
    let script = crate::actions::types::ScriptInfo::new("foo", "/p/foo.ts");
    let actions = get_script_context_actions(&script);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert_eq!(run.title, "Run");
}

#[test]
fn batch30_script_verb_launch() {
    let script = crate::actions::types::ScriptInfo::with_action_verb(
        "Safari",
        "/Applications/Safari.app",
        false,
        "Launch",
    );
    let actions = get_script_context_actions(&script);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert_eq!(run.title, "Launch");
}

#[test]
fn batch30_script_verb_switch_to() {
    let script = crate::actions::types::ScriptInfo::with_action_verb(
        "Preview",
        "window:1",
        false,
        "Switch to",
    );
    let actions = get_script_context_actions(&script);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert_eq!(run.title, "Switch To");
}

#[test]
fn batch30_script_verb_desc_uses_verb() {
    let script = crate::actions::types::ScriptInfo::with_action_verb("X", "/p", false, "Open");
    let actions = get_script_context_actions(&script);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert_eq!(run.description.as_deref(), Some("Open this item"));
}

// ---------------------------------------------------------------------------
// 19. Script context: deeplink URL in copy_deeplink description
// ---------------------------------------------------------------------------
#[test]
fn batch30_deeplink_desc_contains_url() {
    let script = crate::actions::types::ScriptInfo::new("My Cool Script", "/p.ts");
    let actions = get_script_context_actions(&script);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert!(dl
        .description
        .as_ref()
        .unwrap()
        .contains("scriptkit://run/my-cool-script"));
}

#[test]
fn batch30_deeplink_shortcut_is_cmd_shift_d() {
    let script = crate::actions::types::ScriptInfo::new("X", "/p.ts");
    let actions = get_script_context_actions(&script);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert_eq!(dl.shortcut.as_deref(), Some("⌘⇧D"));
}

#[test]
fn batch30_deeplink_desc_for_builtin() {
    let script = crate::actions::types::ScriptInfo::builtin("Clipboard History");
    let actions = get_script_context_actions(&script);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert!(dl
        .description
        .as_ref()
        .unwrap()
        .contains("scriptkit://run/clipboard-history"));
}

// ---------------------------------------------------------------------------
// 20. CommandBarConfig: notes_style matches expected fields
// ---------------------------------------------------------------------------
#[test]
fn batch30_command_bar_notes_style_search_top() {
    let cfg = CommandBarConfig::notes_style();
    assert!(matches!(
        cfg.dialog_config.search_position,
        SearchPosition::Top
    ));
}

#[test]
fn batch30_command_bar_notes_style_section_separators() {
    let cfg = CommandBarConfig::notes_style();
    assert!(matches!(
        cfg.dialog_config.section_style,
        SectionStyle::Separators
    ));
}

#[test]
fn batch30_command_bar_notes_style_anchor_top() {
    let cfg = CommandBarConfig::notes_style();
    assert!(matches!(cfg.dialog_config.anchor, AnchorPosition::Top));
}

#[test]
fn batch30_command_bar_notes_style_show_icons_and_footer() {
    let cfg = CommandBarConfig::notes_style();
    assert!(cfg.dialog_config.show_icons);
    assert!(cfg.dialog_config.show_footer);
}

// ---------------------------------------------------------------------------
// 21. parse_shortcut_keycaps: modifier+letter combos
// ---------------------------------------------------------------------------
#[test]
fn batch30_parse_keycaps_cmd_c() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘C");
    assert_eq!(caps, vec!["⌘", "C"]);
}

#[test]
fn batch30_parse_keycaps_cmd_shift_a() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘⇧A");
    assert_eq!(caps, vec!["⌘", "⇧", "A"]);
}

#[test]
fn batch30_parse_keycaps_enter_alone() {
    let caps = ActionsDialog::parse_shortcut_keycaps("↵");
    assert_eq!(caps, vec!["↵"]);
}

#[test]
fn batch30_parse_keycaps_all_modifiers_plus_key() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌃⌥⇧⌘K");
    assert_eq!(caps, vec!["⌃", "⌥", "⇧", "⌘", "K"]);
}

// ---------------------------------------------------------------------------
// 22. score_action: various match scenarios
// ---------------------------------------------------------------------------
#[test]
fn batch30_score_prefix_match_gte_100() {
    let action = Action::new(
        "e",
        "Edit Script",
        Some("Open in $EDITOR".into()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "edit");
    assert!(score >= 100, "Prefix match should be ≥100, got {}", score);
}

#[test]
fn batch30_score_contains_match_50_to_99() {
    let action = Action::new("c", "Copy Edit Path", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "edit");
    assert!(
        (50..100).contains(&score),
        "Contains match should be 50..99, got {}",
        score
    );
}

#[test]
fn batch30_score_no_match_is_zero() {
    let action = Action::new("x", "Run Script", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "zzz");
    assert_eq!(score, 0);
}

#[test]
fn batch30_score_empty_search_is_prefix() {
    let action = Action::new("x", "Hello", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "");
    assert!(score >= 100, "Empty search is prefix match, got {}", score);
}

// ---------------------------------------------------------------------------
// 23. fuzzy_match: edge cases
// ---------------------------------------------------------------------------
#[test]
fn batch30_fuzzy_exact_match() {
    assert!(ActionsDialog::fuzzy_match("hello", "hello"));
}

#[test]
fn batch30_fuzzy_subsequence() {
    assert!(ActionsDialog::fuzzy_match("hello world", "hwo"));
}

#[test]
fn batch30_fuzzy_no_match() {
    assert!(!ActionsDialog::fuzzy_match("abc", "xyz"));
}

#[test]
fn batch30_fuzzy_needle_longer_than_haystack() {
    assert!(!ActionsDialog::fuzzy_match("ab", "abcdef"));
}

#[test]
fn batch30_fuzzy_empty_needle() {
    assert!(ActionsDialog::fuzzy_match("anything", ""));
}

// ---------------------------------------------------------------------------
// 24. build_grouped_items_static: Headers vs Separators behavior
// ---------------------------------------------------------------------------
#[test]
fn batch30_grouped_headers_adds_section_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered = vec![0, 1];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // Should be: Header("S1"), Item(0), Header("S2"), Item(1) = 4 items
    assert_eq!(grouped.len(), 4);
}

#[test]
fn batch30_grouped_separators_no_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered = vec![0, 1];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    // Should be: Item(0), Item(1) = 2 items (no headers)
    assert_eq!(grouped.len(), 2);
}

#[test]
fn batch30_grouped_empty_returns_empty() {
    let grouped = build_grouped_items_static(&[], &[], SectionStyle::Headers);
    assert!(grouped.is_empty());
}

#[test]
fn batch30_grouped_same_section_one_header() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S"),
    ];
    let filtered = vec![0, 1];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // Should be: Header("S"), Item(0), Item(1) = 3 items
    assert_eq!(grouped.len(), 3);
}

// ---------------------------------------------------------------------------
// 25. coerce_action_selection: header skipping
// ---------------------------------------------------------------------------
#[test]
fn batch30_coerce_on_item_stays() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
}

#[test]
fn batch30_coerce_on_header_jumps_down() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![
        GroupedActionItem::SectionHeader("H".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn batch30_coerce_trailing_header_jumps_up() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("H".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn batch30_coerce_all_headers_none() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![
        GroupedActionItem::SectionHeader("H1".into()),
        GroupedActionItem::SectionHeader("H2".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn batch30_coerce_empty_none() {
    assert_eq!(coerce_action_selection(&[], 0), None);
}

// ---------------------------------------------------------------------------
// 26. Clipboard: destructive actions ordering invariant
// ---------------------------------------------------------------------------
#[test]
fn batch30_clipboard_destructive_always_last_three() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let len = actions.len();
    assert!(len >= 3);
    assert_eq!(actions[len - 3].id, "clip:clipboard_delete");
    assert_eq!(actions[len - 2].id, "clip:clipboard_delete_multiple");
    assert_eq!(actions[len - 1].id, "clip:clipboard_delete_all");
}

#[test]
fn batch30_clipboard_image_destructive_also_last_three() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "i".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let len = actions.len();
    assert!(len >= 3);
    assert_eq!(actions[len - 3].id, "clip:clipboard_delete");
    assert_eq!(actions[len - 2].id, "clip:clipboard_delete_multiple");
    assert_eq!(actions[len - 1].id, "clip:clipboard_delete_all");
}

#[test]
fn batch30_clipboard_delete_all_desc_mentions_pinned() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let da = actions
        .iter()
        .find(|a| a.id == "clip:clipboard_delete_all")
        .unwrap();
    assert!(da
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("pinned"));
}

// ---------------------------------------------------------------------------
// 27. Script context: agent has specific action set
// ---------------------------------------------------------------------------
#[test]
fn batch30_agent_has_edit_script_title_edit_agent() {
    let mut script = crate::actions::types::ScriptInfo::new("a", "/p");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.title, "Edit Agent");
}

#[test]
fn batch30_agent_has_no_view_logs() {
    let mut script = crate::actions::types::ScriptInfo::new("a", "/p");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

// --- merged from part_04.rs ---

#[test]
fn batch30_agent_has_reveal_in_finder() {
    let mut script = crate::actions::types::ScriptInfo::new("a", "/p");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
}

#[test]
fn batch30_agent_has_copy_path() {
    let mut script = crate::actions::types::ScriptInfo::new("a", "/p");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    assert!(actions.iter().any(|a| a.id == "copy_path"));
}

// ---------------------------------------------------------------------------
// 28. Action builder: cached lowercase fields
// ---------------------------------------------------------------------------
#[test]
fn batch30_action_title_lower_precomputed() {
    let action = Action::new("x", "Hello World", None, ActionCategory::ScriptContext);
    assert_eq!(action.title_lower, "hello world");
}

#[test]
fn batch30_action_description_lower_precomputed() {
    let action = Action::new(
        "x",
        "T",
        Some("Open in $EDITOR".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(action.description_lower.as_deref(), Some("open in $editor"));
}

#[test]
fn batch30_action_shortcut_lower_after_with_shortcut() {
    let action = Action::new("x", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
    assert_eq!(action.shortcut_lower.as_deref(), Some("⌘⇧c"));
}

#[test]
fn batch30_action_no_shortcut_lower_is_none() {
    let action = Action::new("x", "T", None, ActionCategory::ScriptContext);
    assert!(action.shortcut_lower.is_none());
}

// ---------------------------------------------------------------------------
// 29. Action builder: with_icon and with_section
// ---------------------------------------------------------------------------
#[test]
fn batch30_action_with_icon_sets_field() {
    let action =
        Action::new("x", "T", None, ActionCategory::ScriptContext).with_icon(IconName::Star);
    assert_eq!(action.icon, Some(IconName::Star));
}

#[test]
fn batch30_action_new_no_icon() {
    let action = Action::new("x", "T", None, ActionCategory::ScriptContext);
    assert!(action.icon.is_none());
}

#[test]
fn batch30_action_with_section_sets_field() {
    let action =
        Action::new("x", "T", None, ActionCategory::ScriptContext).with_section("MySection");
    assert_eq!(action.section.as_deref(), Some("MySection"));
}

#[test]
fn batch30_action_new_no_section() {
    let action = Action::new("x", "T", None, ActionCategory::ScriptContext);
    assert!(action.section.is_none());
}

// ---------------------------------------------------------------------------
// 30. Cross-context: all built-in actions have has_action=false
// ---------------------------------------------------------------------------
#[test]
fn batch30_cross_context_script_actions_has_action_false() {
    let script = crate::actions::types::ScriptInfo::new("s", "/p.ts");
    let actions = get_script_context_actions(&script);
    for a in &actions {
        assert!(
            !a.has_action,
            "Script action '{}' should have has_action=false",
            a.id
        );
    }
}

#[test]
fn batch30_cross_context_clipboard_actions_has_action_false() {
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
        assert!(
            !a.has_action,
            "Clipboard action '{}' should have has_action=false",
            a.id
        );
    }
}

#[test]
fn batch30_cross_context_file_actions_has_action_false() {
    let info = FileInfo {
        path: "/f.txt".into(),
        name: "f.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    for a in &actions {
        assert!(
            !a.has_action,
            "File action '{}' should have has_action=false",
            a.id
        );
    }
}

#[test]
fn batch30_cross_context_path_actions_has_action_false() {
    let info = PathInfo {
        path: "/f".into(),
        name: "f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    for a in &actions {
        assert!(
            !a.has_action,
            "Path action '{}' should have has_action=false",
            a.id
        );
    }
}

#[test]
fn batch30_cross_context_ai_bar_actions_has_action_false() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(
            !a.has_action,
            "AI bar action '{}' should have has_action=false",
            a.id
        );
    }
}

#[test]
fn batch30_cross_context_notes_actions_has_action_false() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    for a in &actions {
        assert!(
            !a.has_action,
            "Notes action '{}' should have has_action=false",
            a.id
        );
    }
}
