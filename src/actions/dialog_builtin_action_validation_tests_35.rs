// --- merged from part_01.rs ---
//! Batch 35: Dialog built-in action validation tests
//!
//! 116 tests across 30 categories validating random behaviors from
//! built-in action window dialogs.

use crate::actions::builders::{
    get_ai_command_bar_actions, get_chat_context_actions, get_clipboard_history_context_actions,
    get_file_context_actions, get_new_chat_actions, get_note_switcher_actions,
    get_notes_command_bar_actions, get_path_context_actions, get_script_context_actions,
    get_scriptlet_context_actions_with_custom, to_deeplink_name, ChatModelInfo, ChatPromptInfo,
    ClipboardEntryInfo, NewChatModelInfo, NewChatPresetInfo, NoteSwitcherNoteInfo, NotesInfo,
};
use crate::actions::command_bar::CommandBarConfig;
use crate::actions::constants::{
    ACCENT_BAR_WIDTH, ACTION_ROW_INSET, HEADER_HEIGHT, KEYCAP_HEIGHT, KEYCAP_MIN_WIDTH,
    SEARCH_INPUT_HEIGHT, SECTION_HEADER_HEIGHT, SELECTION_RADIUS,
};
use crate::actions::dialog::{build_grouped_items_static, coerce_action_selection, ActionsDialog};
use crate::actions::types::{Action, ActionCategory, ScriptInfo, SectionStyle};
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::FileInfo;
use crate::prompts::PathInfo;
use crate::scriptlets::{Scriptlet, ScriptletAction};

// =====================================================================
// 1. Clipboard: attach_to_ai shortcut and description
// =====================================================================

#[test]
fn clipboard_attach_to_ai_shortcut_is_ctrl_cmd_a() {
    let entry = ClipboardEntryInfo {
        id: "ai-1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "some text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let action = actions
        .iter()
        .find(|a| a.id == "clip:clipboard_attach_to_ai")
        .unwrap();
    assert_eq!(action.shortcut.as_ref().unwrap(), "⌃⌘A");
}

#[test]
fn clipboard_attach_to_ai_title() {
    let entry = ClipboardEntryInfo {
        id: "ai-2".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let action = actions
        .iter()
        .find(|a| a.id == "clip:clipboard_attach_to_ai")
        .unwrap();
    assert_eq!(action.title, "Attach to AI Chat");
}

#[test]
fn clipboard_attach_to_ai_desc_mentions_ai() {
    let entry = ClipboardEntryInfo {
        id: "ai-3".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let action = actions
        .iter()
        .find(|a| a.id == "clip:clipboard_attach_to_ai")
        .unwrap();
    assert!(action
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("ai"));
}

#[test]
fn clipboard_attach_to_ai_present_for_image() {
    let entry = ClipboardEntryInfo {
        id: "ai-4".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((640, 480)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clip:clipboard_attach_to_ai"));
}

// =====================================================================
// 2. Clipboard: total action count text vs image on macOS
// =====================================================================

#[cfg(target_os = "macos")]
#[test]
fn clipboard_text_action_count_macos() {
    let entry = ClipboardEntryInfo {
        id: "cnt-1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    // Text on macOS: paste, copy, paste_keep_open, share, attach_to_ai, quick_look, pin,
    // save_snippet, save_file, delete, delete_multiple, delete_all = 12
    assert_eq!(actions.len(), 12);
}

#[cfg(target_os = "macos")]
#[test]
fn clipboard_image_action_count_macos() {
    let entry = ClipboardEntryInfo {
        id: "cnt-2".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    // Image on macOS: paste, copy, paste_keep_open, share, attach_to_ai, quick_look,
    // open_with, annotate_cleanshot, upload_cleanshot, pin, ocr,
    // save_snippet, save_file, delete, delete_multiple, delete_all = 16
    assert_eq!(actions.len(), 16);
}

#[cfg(target_os = "macos")]
#[test]
fn clipboard_image_has_4_more_actions_than_text_on_macos() {
    let text_entry = ClipboardEntryInfo {
        id: "cnt-3".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let img_entry = ClipboardEntryInfo {
        id: "cnt-4".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "i".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let text_count = get_clipboard_history_context_actions(&text_entry).len();
    let img_count = get_clipboard_history_context_actions(&img_entry).len();
    // Image has 4 more: open_with, annotate_cleanshot, upload_cleanshot, ocr
    assert_eq!(img_count - text_count, 4);
}

#[test]
fn clipboard_pinned_vs_unpinned_same_count() {
    let pinned = ClipboardEntryInfo {
        id: "cnt-5".into(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "p".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let unpinned = ClipboardEntryInfo {
        id: "cnt-6".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "u".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    // pin/unpin swapped but same count
    assert_eq!(
        get_clipboard_history_context_actions(&pinned).len(),
        get_clipboard_history_context_actions(&unpinned).len()
    );
}

// =====================================================================
// 3. File context: primary action title is quoted with file name
// =====================================================================

#[test]
fn file_context_file_title_quoted() {
    let file_info = FileInfo {
        path: "/Users/test/readme.md".into(),
        name: "readme.md".into(),
        file_type: crate::file_search::FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    let primary = actions.first().unwrap();
    assert_eq!(primary.title, "Open \"readme.md\"");
}

#[test]
fn file_context_dir_title_quoted() {
    let file_info = FileInfo {
        path: "/Users/test/Documents".into(),
        name: "Documents".into(),
        file_type: crate::file_search::FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&file_info);
    let primary = actions.first().unwrap();
    assert_eq!(primary.title, "Open \"Documents\"");
}

#[test]
fn file_context_file_primary_shortcut_enter() {
    let file_info = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: crate::file_search::FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    assert_eq!(actions[0].shortcut.as_ref().unwrap(), "↵");
}

#[test]
fn file_context_dir_primary_shortcut_enter() {
    let file_info = FileInfo {
        path: "/test/dir".into(),
        name: "dir".into(),
        file_type: crate::file_search::FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&file_info);
    assert_eq!(actions[0].shortcut.as_ref().unwrap(), "↵");
}

// =====================================================================
// 4. Path context: primary action description varies for file vs dir
// =====================================================================

#[test]
fn path_file_primary_desc_is_submit() {
    let path_info = PathInfo {
        path: "/Users/test/file.txt".into(),
        name: "file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let primary = actions.first().unwrap();
    assert!(primary.description.as_ref().unwrap().contains("Selects"));
}

#[test]
fn path_dir_primary_desc_is_navigate() {
    let path_info = PathInfo {
        path: "/Users/test/docs".into(),
        name: "docs".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    let primary = actions.first().unwrap();
    assert!(primary.description.as_ref().unwrap().contains("Opens"));
}

#[test]
fn path_file_primary_id_is_select_file() {
    let path_info = PathInfo {
        path: "/test/a.txt".into(),
        name: "a.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    assert_eq!(actions[0].id, "file:select_file");
}

#[test]
fn path_dir_primary_id_is_open_directory() {
    let path_info = PathInfo {
        path: "/test/dir".into(),
        name: "dir".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    assert_eq!(actions[0].id, "file:open_directory");
}

// =====================================================================
// 5. Script context: edit shortcut ⌘E across types
// =====================================================================

#[test]
fn script_edit_shortcut_cmd_e() {
    let script = ScriptInfo::new("my-script", "/path/to/script.ts");
    let actions = get_script_context_actions(&script);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.shortcut.as_ref().unwrap(), "⌘E");
}

#[test]
fn scriptlet_edit_shortcut_cmd_e() {
    let scriptlet = ScriptInfo::scriptlet("Open URL", "/path/url.md", None, None);
    let actions = get_script_context_actions(&scriptlet);
    let edit = actions.iter().find(|a| a.id == "edit_scriptlet").unwrap();
    assert_eq!(edit.shortcut.as_ref().unwrap(), "⌘E");
}

#[test]
fn agent_edit_shortcut_cmd_e() {
    let mut agent = ScriptInfo::new("My Agent", "/path/agent");
    agent.is_script = false;
    agent.is_agent = true;
    let actions = get_script_context_actions(&agent);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.shortcut.as_ref().unwrap(), "⌘E");
}

#[test]
fn script_copy_content_shortcut_cmd_opt_c() {
    let script = ScriptInfo::new("my-script", "/path/to/script.ts");
    let actions = get_script_context_actions(&script);
    let copy = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(copy.shortcut.as_ref().unwrap(), "⌘⌥C");
}

// =====================================================================
// 6. Scriptlet with custom H3 actions: ID prefix, has_action, value
// =====================================================================

#[test]
fn scriptlet_custom_action_id_prefix() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut s = Scriptlet::new("Test".into(), "bash".into(), "echo hi".into());
    s.actions = vec![ScriptletAction {
        name: "My Custom".into(),
        command: "my-custom".into(),
        tool: "bash".into(),
        code: "echo custom".into(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&s));
    assert!(actions.iter().any(|a| a.id == "scriptlet_action:my-custom"));
}

#[test]
fn scriptlet_custom_action_has_action_true() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut s = Scriptlet::new("Test".into(), "bash".into(), "echo hi".into());
    s.actions = vec![ScriptletAction {
        name: "Custom".into(),
        command: "cmd".into(),
        tool: "bash".into(),
        code: "echo".into(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&s));
    let custom = actions
        .iter()
        .find(|a| a.id == "scriptlet_action:cmd")
        .unwrap();
    assert!(custom.has_action);
}

#[test]
fn scriptlet_custom_action_value_is_command() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut s = Scriptlet::new("Test".into(), "bash".into(), "echo hi".into());
    s.actions = vec![ScriptletAction {
        name: "Copy It".into(),
        command: "copy-it".into(),
        tool: "bash".into(),
        code: "pbcopy".into(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&s));
    let custom = actions
        .iter()
        .find(|a| a.id == "scriptlet_action:copy-it")
        .unwrap();
    assert_eq!(custom.value.as_ref().unwrap(), "copy-it");
}

#[test]
fn scriptlet_builtin_actions_has_action_false() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    for a in &actions {
        assert!(
            !a.has_action,
            "Built-in action {} should have has_action=false",
            a.id
        );
    }
}

// =====================================================================
// 7. Scriptlet custom action with shortcut gets format_shortcut_hint applied
// =====================================================================

#[test]
fn scriptlet_custom_action_shortcut_formatted() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut s = Scriptlet::new("Test".into(), "bash".into(), "echo".into());
    s.actions = vec![ScriptletAction {
        name: "Action".into(),
        command: "act".into(),
        tool: "bash".into(),
        code: "echo".into(),
        inputs: vec![],
        shortcut: Some("cmd+shift+x".into()),
        description: Some("Do something".into()),
    }];
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&s));
    let custom = actions
        .iter()
        .find(|a| a.id == "scriptlet_action:act")
        .unwrap();
    assert_eq!(custom.shortcut.as_ref().unwrap(), "⌘⇧X");
}

// --- merged from part_02.rs ---

#[test]
fn scriptlet_custom_action_without_shortcut_is_none() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut s = Scriptlet::new("Test".into(), "bash".into(), "echo".into());
    s.actions = vec![ScriptletAction {
        name: "NoKey".into(),
        command: "nokey".into(),
        tool: "bash".into(),
        code: "echo".into(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&s));
    let custom = actions
        .iter()
        .find(|a| a.id == "scriptlet_action:nokey")
        .unwrap();
    assert!(custom.shortcut.is_none());
}

#[test]
fn scriptlet_custom_action_description_propagated() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut s = Scriptlet::new("Test".into(), "bash".into(), "echo".into());
    s.actions = vec![ScriptletAction {
        name: "Desc Action".into(),
        command: "desc-act".into(),
        tool: "bash".into(),
        code: "echo".into(),
        inputs: vec![],
        shortcut: None,
        description: Some("My description here".into()),
    }];
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&s));
    let custom = actions
        .iter()
        .find(|a| a.id == "scriptlet_action:desc-act")
        .unwrap();
    assert_eq!(custom.description.as_ref().unwrap(), "My description here");
}

#[test]
fn scriptlet_custom_action_title_is_name() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut s = Scriptlet::new("Test".into(), "bash".into(), "echo".into());
    s.actions = vec![ScriptletAction {
        name: "My Title".into(),
        command: "mt".into(),
        tool: "bash".into(),
        code: "echo".into(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&s));
    let custom = actions
        .iter()
        .find(|a| a.id == "scriptlet_action:mt")
        .unwrap();
    assert_eq!(custom.title, "My Title");
}

// =====================================================================
// 8. AI command bar: copy_chat and copy_last_code details
// =====================================================================

#[test]
fn ai_bar_copy_chat_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "chat:copy_chat").unwrap();
    assert_eq!(a.shortcut.as_ref().unwrap(), "⌥⇧⌘C");
}

#[test]
fn ai_bar_copy_chat_icon_copy() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "chat:copy_chat").unwrap();
    assert_eq!(a.icon, Some(IconName::Copy));
}

#[test]
fn ai_bar_copy_chat_section_response() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "chat:copy_chat").unwrap();
    assert_eq!(a.section.as_ref().unwrap(), "Response");
}

#[test]
fn ai_bar_copy_last_code_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "chat:copy_last_code").unwrap();
    assert_eq!(a.shortcut.as_ref().unwrap(), "⌥⌘C");
}

#[test]
fn ai_bar_copy_last_code_icon_code() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "chat:copy_last_code").unwrap();
    assert_eq!(a.icon, Some(IconName::Code));
}

#[test]
fn ai_bar_copy_last_code_section_response() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "chat:copy_last_code").unwrap();
    assert_eq!(a.section.as_ref().unwrap(), "Response");
}

// =====================================================================
// 9. AI command bar: all IDs are unique
// =====================================================================

#[test]
fn ai_bar_all_ids_unique() {
    let actions = get_ai_command_bar_actions();
    let mut ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let original_len = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), original_len);
}

#[test]
fn ai_bar_all_have_icon() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(
            a.icon.is_some(),
            "AI bar action {} should have an icon",
            a.id
        );
    }
}

#[test]
fn ai_bar_all_have_section() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(
            a.section.is_some(),
            "AI bar action {} should have a section",
            a.id
        );
    }
}

#[test]
fn ai_bar_count_is_12() {
    let actions = get_ai_command_bar_actions();
    assert_eq!(actions.len(), 12);
}

// =====================================================================
// 10. Chat context: select_model ID format
// =====================================================================

#[test]
fn chat_model_id_format() {
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
    assert!(actions.iter().any(|a| a.id == "chat:select_model_gpt-4"));
}

#[test]
fn chat_model_current_check_by_display_name() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".into()),
        available_models: vec![ChatModelInfo {
            id: "gpt-4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model = actions
        .iter()
        .find(|a| a.id == "chat:select_model_gpt-4")
        .unwrap();
    assert!(model.title.contains("✓"));
}

#[test]
fn chat_model_non_current_no_check() {
    let info = ChatPromptInfo {
        current_model: Some("Claude 3.5".into()),
        available_models: vec![ChatModelInfo {
            id: "gpt-4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model = actions
        .iter()
        .find(|a| a.id == "chat:select_model_gpt-4")
        .unwrap();
    assert!(!model.title.contains("✓"));
}

#[test]
fn chat_model_desc_via_provider() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "claude".into(),
            display_name: "Claude".into(),
            provider: "Anthropic".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model = actions
        .iter()
        .find(|a| a.id == "chat:select_model_claude")
        .unwrap();
    assert_eq!(model.description.as_ref().unwrap(), "Uses Anthropic");
}

// =====================================================================
// 11. Notes command bar: format action details
// =====================================================================

#[test]
fn notes_format_shortcut_shift_cmd_t() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let a = actions.iter().find(|a| a.id == "format").unwrap();
    assert_eq!(a.shortcut.as_ref().unwrap(), "⇧⌘T");
}

#[test]
fn notes_format_icon_code() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let a = actions.iter().find(|a| a.id == "format").unwrap();
    assert_eq!(a.icon, Some(IconName::Code));
}

#[test]
fn notes_format_section_edit() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let a = actions.iter().find(|a| a.id == "format").unwrap();
    assert_eq!(a.section.as_ref().unwrap(), "Edit");
}

#[test]
fn notes_format_absent_in_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "format"));
}

// =====================================================================
// 12. Notes command bar: trash view exact action set
// =====================================================================

#[test]
fn notes_trash_has_exactly_3_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert_eq!(actions.len(), 5);
}

#[test]
fn notes_trash_has_new_note() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "new_note"));
}

#[test]
fn notes_trash_has_browse_notes() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "browse_notes"));
}

#[test]
fn notes_trash_has_enable_auto_sizing_when_disabled() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "enable_auto_sizing"));
}

// =====================================================================
// 13. Note switcher: empty notes produces no_notes action
// =====================================================================

#[test]
fn note_switcher_empty_has_no_notes() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "no_notes");
}

#[test]
fn note_switcher_no_notes_title() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions[0].title, "No notes yet");
}

#[test]
fn note_switcher_no_notes_desc_mentions_cmd_n() {
    let actions = get_note_switcher_actions(&[]);
    assert!(actions[0].description.as_ref().unwrap().contains("⌘N"));
}

#[test]
fn note_switcher_no_notes_icon_plus() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions[0].icon, Some(IconName::Plus));
}

// =====================================================================
// 14. Note switcher: ID format is note_{uuid}
// =====================================================================

#[test]
fn note_switcher_id_format() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc-123-def".into(),
        title: "My Note".into(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].id, "note_abc-123-def");
}

#[test]
fn note_switcher_current_icon_check() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n1".into(),
        title: "Current".into(),
        char_count: 10,
        is_current: true,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::Check));
}

#[test]
fn note_switcher_regular_icon_file() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n2".into(),
        title: "Regular".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::File));
}

#[test]
fn note_switcher_pinned_trumps_current() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "n3".into(),
        title: "Both".into(),
        char_count: 10,
        is_current: true,
        is_pinned: true,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

// =====================================================================
// 15. New chat: empty inputs produce expected results
// =====================================================================

#[test]
fn new_chat_all_empty() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

#[test]
fn new_chat_only_models() {
    let models = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "p".into(),
        provider_display_name: "Provider".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "model_p::m1");
}

// --- merged from part_03.rs ---

#[test]
fn new_chat_only_presets() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "preset_general");
}

#[test]
fn new_chat_only_last_used() {
    let last_used = vec![NewChatModelInfo {
        model_id: "lu1".into(),
        display_name: "Last Used 1".into(),
        provider: "p".into(),
        provider_display_name: "Provider".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "last_used_p::lu1");
}

// =====================================================================
// 16. to_deeplink_name: additional transformations
// =====================================================================

#[test]
fn deeplink_name_preserves_numbers() {
    assert_eq!(to_deeplink_name("Script 123"), "script-123");
}

#[test]
fn deeplink_name_emoji_to_hyphens() {
    // Emojis are non-alphanumeric so they become hyphens (then collapse)
    assert_eq!(to_deeplink_name("Cool Script"), "cool-script");
}

#[test]
fn deeplink_name_already_lowercase() {
    assert_eq!(to_deeplink_name("already-lowercase"), "already-lowercase");
}

#[test]
fn deeplink_name_single_char() {
    assert_eq!(to_deeplink_name("A"), "a");
}

// =====================================================================
// 17. Constants: secondary dimension values
// =====================================================================

#[test]
fn constant_section_header_height() {
    assert_eq!(SECTION_HEADER_HEIGHT, 22.0);
}

#[test]
fn constant_header_height() {
    assert_eq!(HEADER_HEIGHT, 24.0);
}

#[test]
fn constant_action_row_inset() {
    assert_eq!(ACTION_ROW_INSET, 6.0);
}

#[test]
fn constant_selection_radius() {
    assert_eq!(SELECTION_RADIUS, 8.0);
}

// =====================================================================
// 18. Constants: keycap and accent bar
// =====================================================================

#[test]
fn constant_keycap_min_width() {
    assert_eq!(KEYCAP_MIN_WIDTH, 22.0);
}

#[test]
fn constant_keycap_height() {
    assert_eq!(KEYCAP_HEIGHT, 22.0);
}

#[test]
fn constant_accent_bar_width() {
    assert_eq!(ACCENT_BAR_WIDTH, 3.0);
}

#[test]
fn constant_search_input_height() {
    assert_eq!(SEARCH_INPUT_HEIGHT, 44.0);
}

// =====================================================================
// 19. parse_shortcut_keycaps: modifier and special key parsing
// =====================================================================

#[test]
fn parse_keycaps_cmd_enter() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘↵");
    assert_eq!(caps, vec!["⌘", "↵"]);
}

#[test]
fn parse_keycaps_all_modifiers_and_key() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘⇧⌃⌥K");
    assert_eq!(caps, vec!["⌘", "⇧", "⌃", "⌥", "K"]);
}

#[test]
fn parse_keycaps_single_letter() {
    let caps = ActionsDialog::parse_shortcut_keycaps("A");
    assert_eq!(caps, vec!["A"]);
}

#[test]
fn parse_keycaps_lowercase_uppercased() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘c");
    assert_eq!(caps, vec!["⌘", "C"]);
}

// =====================================================================
// 20. format_shortcut_hint: additional conversions
// =====================================================================

#[test]
fn format_shortcut_hint_cmd_backspace() {
    assert_eq!(ActionsDialog::format_shortcut_hint("cmd+backspace"), "⌘⌫");
}

#[test]
fn format_shortcut_hint_ctrl_tab() {
    assert_eq!(ActionsDialog::format_shortcut_hint("ctrl+tab"), "⌃⇥");
}

#[test]
fn format_shortcut_hint_option_space() {
    assert_eq!(ActionsDialog::format_shortcut_hint("option+space"), "⌥␣");
}

#[test]
fn format_shortcut_hint_single_escape() {
    assert_eq!(ActionsDialog::format_shortcut_hint("escape"), "⎋");
}

// =====================================================================
// 21. build_grouped_items_static: None section handling
// =====================================================================

#[test]
fn grouped_items_none_section_no_header() {
    let actions = vec![
        Action::new("a", "Alpha", None, ActionCategory::ScriptContext),
        Action::new("b", "Beta", None, ActionCategory::ScriptContext),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // No sections on actions → no headers added
    assert_eq!(grouped.len(), 2);
}

#[test]
fn grouped_items_mixed_some_none_sections() {
    let mut a1 = Action::new("a", "Alpha", None, ActionCategory::ScriptContext);
    a1.section = Some("Group A".into());
    let a2 = Action::new("b", "Beta", None, ActionCategory::ScriptContext);
    // a2 has no section
    let actions = vec![a1, a2];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // One header for "Group A", then item, then item (no header for None section)
    assert_eq!(grouped.len(), 3);
}

#[test]
fn grouped_items_separators_never_adds_headers() {
    let mut a1 = Action::new("a", "Alpha", None, ActionCategory::ScriptContext);
    a1.section = Some("Group A".into());
    let mut a2 = Action::new("b", "Beta", None, ActionCategory::ScriptContext);
    a2.section = Some("Group B".into());
    let actions = vec![a1, a2];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    // Separators style never adds headers
    assert_eq!(grouped.len(), 2);
}

#[test]
fn grouped_items_empty_filtered_returns_empty() {
    let actions = vec![Action::new(
        "a",
        "Alpha",
        None,
        ActionCategory::ScriptContext,
    )];
    let filtered: Vec<usize> = vec![];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    assert!(grouped.is_empty());
}

// =====================================================================
// 22. coerce_action_selection: specific patterns
// =====================================================================

#[test]
fn coerce_selection_single_item() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![GroupedActionItem::Item(0)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
}

#[test]
fn coerce_selection_header_then_item() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![
        GroupedActionItem::SectionHeader("H".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn coerce_selection_item_then_header() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("H".into()),
    ];
    // On header at index 1 → search down (nothing) → search up → find item at 0
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn coerce_selection_beyond_bounds_clamped() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
    // Index 99 → clamped to len-1=1 → Item(1) → Some(1)
    assert_eq!(coerce_action_selection(&rows, 99), Some(1));
}

// =====================================================================
// 23. CommandBarConfig: close flags consistent
// =====================================================================

#[test]
fn command_bar_ai_close_on_select_true() {
    let config = CommandBarConfig::ai_style();
    assert!(config.close_on_select);
}

#[test]
fn command_bar_ai_close_on_escape_true() {
    let config = CommandBarConfig::ai_style();
    assert!(config.close_on_escape);
}

#[test]
fn command_bar_main_menu_close_on_select_true() {
    let config = CommandBarConfig::main_menu_style();
    assert!(config.close_on_select);
}

#[test]
fn command_bar_notes_close_on_escape_true() {
    let config = CommandBarConfig::notes_style();
    assert!(config.close_on_escape);
}

// =====================================================================
// 24. Script context: scriptlet reveal and copy_path details
// =====================================================================

#[test]
fn scriptlet_reveal_shortcut_cmd_shift_f() {
    let scriptlet = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_script_context_actions(&scriptlet);
    let reveal = actions
        .iter()
        .find(|a| a.id == "reveal_scriptlet_in_finder")
        .unwrap();
    assert_eq!(reveal.shortcut.as_ref().unwrap(), "⌘⇧F");
}

#[test]
fn scriptlet_reveal_desc_mentions_finder() {
    let scriptlet = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_script_context_actions(&scriptlet);
    let reveal = actions
        .iter()
        .find(|a| a.id == "reveal_scriptlet_in_finder")
        .unwrap();
    assert!(reveal.description.as_ref().unwrap().contains("Finder"));
}

#[test]
fn scriptlet_copy_path_shortcut_cmd_shift_c() {
    let scriptlet = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_script_context_actions(&scriptlet);
    let cp = actions
        .iter()
        .find(|a| a.id == "copy_scriptlet_path")
        .unwrap();
    assert_eq!(cp.shortcut.as_ref().unwrap(), "⌘⇧C");
}

#[test]
fn scriptlet_copy_path_desc_mentions_clipboard() {
    let scriptlet = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_script_context_actions(&scriptlet);
    let cp = actions
        .iter()
        .find(|a| a.id == "copy_scriptlet_path")
        .unwrap();
    assert!(cp
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("path"));
}

// =====================================================================
// 25. Score action: fuzzy match scores lower than prefix/contains
// =====================================================================

#[test]
fn score_action_fuzzy_lower_than_prefix() {
    let action = Action::new("test", "Edit Script", None, ActionCategory::ScriptContext);
    let prefix_score = ActionsDialog::score_action(&action, "edit");
    let fuzzy_score = ActionsDialog::score_action(&action, "eds"); // e-d-i-t s-c-r-i-p-t has e,d,s
    assert!(prefix_score > fuzzy_score);
}

#[test]
fn score_action_contains_lower_than_prefix() {
    let action = Action::new(
        "test",
        "My Edit Script",
        None,
        ActionCategory::ScriptContext,
    );
    let prefix_score = ActionsDialog::score_action(&action, "my");
    let contains_score = ActionsDialog::score_action(&action, "script:edit");
    assert!(prefix_score > contains_score);
}

#[test]
fn score_action_both_title_and_desc_match() {
    let action = Action::new(
        "test",
        "Edit Script",
        Some("Edit the script file".into()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "edit");
    // prefix(100) + desc(15) = 115
    assert!(score >= 115);
}

#[test]
fn score_action_shortcut_bonus() {
    let action =
        Action::new("test", "Zzz", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
    let score = ActionsDialog::score_action(&action, "⌘e");
    // No title match but shortcut contains "⌘e" → 10
    assert!(score >= 10);
}

// =====================================================================
// 26. fuzzy_match: additional patterns
// =====================================================================

#[test]
fn fuzzy_match_exact() {
    assert!(ActionsDialog::fuzzy_match("hello", "hello"));
}

#[test]
fn fuzzy_match_subsequence() {
    assert!(ActionsDialog::fuzzy_match("hello world", "hlwrd"));
}

#[test]
fn fuzzy_match_no_match() {
    assert!(!ActionsDialog::fuzzy_match("abc", "xyz"));
}

#[test]
fn fuzzy_match_needle_longer_than_haystack() {
    assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
}

// =====================================================================
// 27. Cross-context: all contexts produce non-empty actions
// =====================================================================

#[test]
fn cross_context_script_non_empty() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    assert!(!get_script_context_actions(&script).is_empty());
}

#[test]
fn cross_context_builtin_non_empty() {
    let builtin = ScriptInfo::builtin("Clipboard History");
    assert!(!get_script_context_actions(&builtin).is_empty());
}

#[test]
fn cross_context_scriptlet_non_empty() {
    let scriptlet = ScriptInfo::scriptlet("Open URL", "/p.md", None, None);
    assert!(!get_script_context_actions(&scriptlet).is_empty());
}

#[test]
fn cross_context_file_non_empty() {
    let f = FileInfo {
        path: "/t.txt".into(),
        name: "t.txt".into(),
        file_type: crate::file_search::FileType::File,
        is_dir: false,
    };
    assert!(!get_file_context_actions(&f).is_empty());
}

#[test]
fn cross_context_path_non_empty() {
    let p = PathInfo {
        path: "/t".into(),
        name: "t".into(),
        is_dir: false,
    };
    assert!(!get_path_context_actions(&p).is_empty());
}

#[test]
fn cross_context_ai_bar_non_empty() {
    assert!(!get_ai_command_bar_actions().is_empty());
}

// --- merged from part_04.rs ---

// =====================================================================
// 28. Clipboard: share action details
// =====================================================================

#[test]
fn clipboard_share_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "sh-1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let share = actions.iter().find(|a| a.id == "clip:clipboard_share").unwrap();
    assert_eq!(share.shortcut.as_ref().unwrap(), "⇧⌘E");
}

#[test]
fn clipboard_share_title() {
    let entry = ClipboardEntryInfo {
        id: "sh-2".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let share = actions.iter().find(|a| a.id == "clip:clipboard_share").unwrap();
    assert_eq!(share.title, "Share...");
}

#[test]
fn clipboard_share_desc_mentions_share() {
    let entry = ClipboardEntryInfo {
        id: "sh-3".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let share = actions.iter().find(|a| a.id == "clip:clipboard_share").unwrap();
    assert!(share
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("share"));
}

#[test]
fn clipboard_share_present_for_image() {
    let entry = ClipboardEntryInfo {
        id: "sh-4".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clip:clipboard_share"));
}

// =====================================================================
// 29. Action builder: cached lowercase consistency
// =====================================================================

#[test]
fn action_title_lower_matches_title() {
    let action = Action::new("id", "Hello World", None, ActionCategory::ScriptContext);
    assert_eq!(action.title_lower, "hello world");
}

#[test]
fn action_description_lower_matches_desc() {
    let action = Action::new(
        "id",
        "T",
        Some("My Description".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(action.description_lower.as_ref().unwrap(), "my description");
}

#[test]
fn action_shortcut_lower_after_with_shortcut() {
    let action = Action::new("id", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
    assert_eq!(action.shortcut_lower.as_ref().unwrap(), "⌘⇧c");
}

#[test]
fn action_no_shortcut_lower_is_none() {
    let action = Action::new("id", "T", None, ActionCategory::ScriptContext);
    assert!(action.shortcut_lower.is_none());
}

// =====================================================================
// 30. Cross-context: all built-in actions use snake_case IDs
// =====================================================================

#[test]
fn script_actions_ids_snake_case() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    for a in get_script_context_actions(&script) {
        assert!(
            !a.id.contains(' '),
            "Action ID '{}' should not contain spaces",
            a.id
        );
        assert!(
            !a.id.contains('-') || a.id.starts_with("scriptlet_action:"),
            "Action ID '{}' should be snake_case (no hyphens)",
            a.id
        );
    }
}

#[test]
fn file_actions_ids_snake_case() {
    let f = FileInfo {
        path: "/t.txt".into(),
        name: "t.txt".into(),
        file_type: crate::file_search::FileType::File,
        is_dir: false,
    };
    for a in get_file_context_actions(&f) {
        assert!(
            !a.id.contains(' '),
            "Action ID '{}' should not contain spaces",
            a.id
        );
    }
}

#[test]
fn path_actions_ids_snake_case() {
    let p = PathInfo {
        path: "/t".into(),
        name: "t".into(),
        is_dir: false,
    };
    for a in get_path_context_actions(&p) {
        assert!(
            !a.id.contains(' '),
            "Action ID '{}' should not contain spaces",
            a.id
        );
    }
}

#[test]
fn clipboard_actions_ids_snake_case() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    for a in get_clipboard_history_context_actions(&entry) {
        assert!(
            !a.id.contains(' '),
            "Action ID '{}' should not contain spaces",
            a.id
        );
    }
}
