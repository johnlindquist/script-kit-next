//! Batch 32: Builtin action validation tests
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
use crate::actions::dialog::{
    build_grouped_items_static, coerce_action_selection, GroupedActionItem,
};
use crate::actions::types::{Action, ActionCategory, SectionStyle};
use crate::actions::ActionsDialog;
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;

// ---------------------------------------------------------------------------
// 1. Script context: agent has no view_logs but has copy_content desc about file
// ---------------------------------------------------------------------------

#[test]
fn batch32_agent_copy_content_desc_mentions_entire_file() {
    let mut script = crate::actions::types::ScriptInfo::new("my-agent", "/p/my-agent.md");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert!(
        cc.description.as_ref().unwrap().contains("entire file"),
        "agent copy_content desc should mention 'entire file', got: {:?}",
        cc.description
    );
}

#[test]
fn batch32_agent_edit_script_desc_mentions_agent_file() {
    let mut script = crate::actions::types::ScriptInfo::new("my-agent", "/p/my-agent.md");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    let es = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert!(
        es.description.as_ref().unwrap().contains("agent"),
        "agent edit desc should mention 'agent', got: {:?}",
        es.description
    );
}

#[test]
fn batch32_agent_reveal_desc_mentions_agent_file() {
    let mut script = crate::actions::types::ScriptInfo::new("my-agent", "/p/my-agent.md");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    let r = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
    assert!(
        r.description.as_ref().unwrap().contains("agent"),
        "agent reveal desc should mention 'agent', got: {:?}",
        r.description
    );
}

#[test]
fn batch32_agent_copy_path_desc_mentions_agent() {
    let mut script = crate::actions::types::ScriptInfo::new("my-agent", "/p/my-agent.md");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert!(
        cp.description.as_ref().unwrap().contains("agent"),
        "agent copy_path desc should mention 'agent', got: {:?}",
        cp.description
    );
}

// ---------------------------------------------------------------------------
// 2. Scriptlet context with_custom: None scriptlet produces only built-in actions
// ---------------------------------------------------------------------------

#[test]
fn batch32_scriptlet_context_none_scriptlet_no_custom_actions() {
    let script = crate::actions::types::ScriptInfo::scriptlet("Test", "/p/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    // All actions should have has_action=false (built-in)
    for a in &actions {
        assert!(
            !a.has_action,
            "built-in action {} should have has_action=false",
            a.id
        );
    }
}

#[test]
fn batch32_scriptlet_context_none_scriptlet_has_edit_scriptlet() {
    let script = crate::actions::types::ScriptInfo::scriptlet("Test", "/p/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "edit_scriptlet"));
}

#[test]
fn batch32_scriptlet_context_none_scriptlet_has_reveal_scriptlet() {
    let script = crate::actions::types::ScriptInfo::scriptlet("Test", "/p/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "reveal_scriptlet_in_finder"));
}

#[test]
fn batch32_scriptlet_context_none_scriptlet_has_copy_scriptlet_path() {
    let script = crate::actions::types::ScriptInfo::scriptlet("Test", "/p/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "copy_scriptlet_path"));
}

// ---------------------------------------------------------------------------
// 3. Clipboard: frontmost_app_name empty string edge case
// ---------------------------------------------------------------------------

#[test]
fn batch32_clipboard_empty_app_name_paste_title() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: Some("".to_string()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
    // Empty string still goes through format!("Paste to {}", name) path
    assert_eq!(paste.title, "Paste to ");
}

#[test]
fn batch32_clipboard_long_app_name_paste_title() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: Some("Very Long Application Name Here".to_string()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
    assert_eq!(paste.title, "Paste to Very Long Application Name Here");
}

#[test]
fn batch32_clipboard_none_app_name_paste_active_app() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
    assert_eq!(paste.title, "Paste to Active App");
}

// ---------------------------------------------------------------------------
// 4. Clipboard: text entry has no image-specific actions
// ---------------------------------------------------------------------------

#[test]
fn batch32_clipboard_text_no_clipboard_open_with() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_open_with"));
}

#[test]
fn batch32_clipboard_text_no_clipboard_annotate_cleanshot() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions
        .iter()
        .any(|a| a.id == "clipboard_annotate_cleanshot"));
}

#[test]
fn batch32_clipboard_text_no_clipboard_upload_cleanshot() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_upload_cleanshot"));
}

#[test]
fn batch32_clipboard_text_no_clipboard_ocr() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_ocr"));
}

// ---------------------------------------------------------------------------
// 5. File context: reveal_in_finder always present for both file and dir
// ---------------------------------------------------------------------------

#[test]
fn batch32_file_reveal_in_finder_present_for_file() {
    let info = FileInfo {
        path: "/p/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
}

#[test]
fn batch32_file_reveal_in_finder_present_for_dir() {
    let info = FileInfo {
        path: "/p/mydir".into(),
        name: "mydir".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
}

#[test]
fn batch32_file_reveal_in_finder_shortcut_is_cmd_enter() {
    let info = FileInfo {
        path: "/p/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
    assert_eq!(reveal.shortcut.as_deref(), Some("⌘↵"));
}

#[test]
fn batch32_file_reveal_desc_says_reveal_in_finder() {
    let info = FileInfo {
        path: "/p/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
    assert!(reveal.description.as_ref().unwrap().contains("Finder"));
}

// ---------------------------------------------------------------------------
// 6. Path context: action ordering after primary action
// ---------------------------------------------------------------------------

#[test]
fn batch32_path_file_second_action_is_copy_path() {
    let info = PathInfo::new("test.txt", "/p/test.txt", false);
    let actions = get_path_context_actions(&info);
    assert_eq!(actions[1].id, "copy_path");
}

#[test]
fn batch32_path_dir_second_action_is_copy_path() {
    let info = PathInfo::new("mydir", "/p/mydir", true);
    let actions = get_path_context_actions(&info);
    assert_eq!(actions[1].id, "copy_path");
}

#[test]
fn batch32_path_file_third_action_is_open_in_finder() {
    let info = PathInfo::new("test.txt", "/p/test.txt", false);
    let actions = get_path_context_actions(&info);
    assert_eq!(actions[2].id, "open_in_finder");
}

#[test]
fn batch32_path_last_action_is_move_to_trash() {
    let info = PathInfo::new("test.txt", "/p/test.txt", false);
    let actions = get_path_context_actions(&info);
    assert_eq!(actions.last().unwrap().id, "move_to_trash");
}

// ---------------------------------------------------------------------------
// 7. Path context: name in primary action title is quoted
// ---------------------------------------------------------------------------

#[test]
fn batch32_path_file_primary_title_quotes_name() {
    let info = PathInfo::new("report.pdf", "/p/report.pdf", false);
    let actions = get_path_context_actions(&info);
    assert_eq!(actions[0].title, "Select \"report.pdf\"");
}

#[test]
fn batch32_path_dir_primary_title_quotes_name() {
    let info = PathInfo::new("Documents", "/p/Documents", true);
    let actions = get_path_context_actions(&info);
    assert_eq!(actions[0].title, "Open \"Documents\"");
}

#[test]
fn batch32_path_file_select_desc_submit() {
    let info = PathInfo::new("file.txt", "/p/file.txt", false);
    let actions = get_path_context_actions(&info);
    assert!(actions[0].description.as_ref().unwrap().contains("Submit"));
}

#[test]
fn batch32_path_dir_open_desc_navigate() {
    let info = PathInfo::new("dir", "/p/dir", true);
    let actions = get_path_context_actions(&info);
    assert!(actions[0]
        .description
        .as_ref()
        .unwrap()
        .contains("Navigate"));
}

// ---------------------------------------------------------------------------
// 8. AI command bar: export_markdown details
// ---------------------------------------------------------------------------

#[test]
fn batch32_ai_export_markdown_section_is_export() {
    let actions = get_ai_command_bar_actions();
    let em = actions.iter().find(|a| a.id == "export_markdown").unwrap();
    assert_eq!(em.section.as_deref(), Some("Export"));
}

#[test]
fn batch32_ai_export_markdown_icon_is_file_code() {
    let actions = get_ai_command_bar_actions();
    let em = actions.iter().find(|a| a.id == "export_markdown").unwrap();
    assert_eq!(em.icon, Some(IconName::FileCode));
}

#[test]
fn batch32_ai_export_markdown_shortcut() {
    let actions = get_ai_command_bar_actions();
    let em = actions.iter().find(|a| a.id == "export_markdown").unwrap();
    assert_eq!(em.shortcut.as_deref(), Some("⇧⌘E"));
}

#[test]
fn batch32_ai_export_markdown_desc_mentions_markdown() {
    let actions = get_ai_command_bar_actions();
    let em = actions.iter().find(|a| a.id == "export_markdown").unwrap();
    assert!(em.description.as_ref().unwrap().contains("Markdown"));
}

// ---------------------------------------------------------------------------
// 9. AI command bar: submit action details
// ---------------------------------------------------------------------------

#[test]
fn batch32_ai_submit_icon_is_arrow_up() {
    let actions = get_ai_command_bar_actions();
    let s = actions.iter().find(|a| a.id == "submit").unwrap();
    assert_eq!(s.icon, Some(IconName::ArrowUp));
}

#[test]
fn batch32_ai_submit_section_is_actions() {
    let actions = get_ai_command_bar_actions();
    let s = actions.iter().find(|a| a.id == "submit").unwrap();
    assert_eq!(s.section.as_deref(), Some("Actions"));
}

#[test]
fn batch32_ai_submit_shortcut_is_enter() {
    let actions = get_ai_command_bar_actions();
    let s = actions.iter().find(|a| a.id == "submit").unwrap();
    assert_eq!(s.shortcut.as_deref(), Some("↵"));
}

#[test]
fn batch32_ai_submit_desc_mentions_send() {
    let actions = get_ai_command_bar_actions();
    let s = actions.iter().find(|a| a.id == "submit").unwrap();
    assert!(
        s.description.as_ref().unwrap().contains("Send"),
        "submit desc should mention 'Send', got: {:?}",
        s.description
    );
}

// ---------------------------------------------------------------------------
// 10. Chat context: single model produces 2 actions minimum
// ---------------------------------------------------------------------------

#[test]
fn batch32_chat_single_model_no_flags_produces_2_actions() {
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
    assert_eq!(actions.len(), 2, "1 model + continue_in_chat = 2");
}
