// Batch 24: Dialog builtin action validation tests
//
// 131 tests across 30 categories validating random built-in action behaviors.

use super::builders::*;
use super::command_bar::CommandBarConfig;
use super::dialog::*;
use super::types::*;
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;

// ============================================================
// 1. Agent context: is_agent flag enables agent-specific actions
// ============================================================

#[test]
fn batch24_agent_has_edit_agent_title() {
    let mut script = ScriptInfo::new("My Agent", "/path/to/agent.md");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.title, "Edit Agent");
}

#[test]
fn batch24_agent_has_copy_content() {
    let mut script = ScriptInfo::new("My Agent", "/path/to/agent.md");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    assert!(actions.iter().any(|a| a.id == "copy_content"));
}

#[test]
fn batch24_agent_lacks_view_logs() {
    let mut script = ScriptInfo::new("My Agent", "/path/to/agent.md");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

#[test]
fn batch24_agent_has_reveal_in_finder() {
    let mut script = ScriptInfo::new("My Agent", "/path/to/agent.md");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
}

// ============================================================
// 2. Agent edit description mentions agent file
// ============================================================

#[test]
fn batch24_agent_edit_desc_mentions_agent_file() {
    let mut script = ScriptInfo::new("Agent X", "/path/to/agent");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert!(edit.description.as_ref().unwrap().contains("agent"));
}

#[test]
fn batch24_agent_reveal_desc_mentions_agent() {
    let mut script = ScriptInfo::new("Agent X", "/path/to/agent");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
    assert!(reveal.description.as_ref().unwrap().contains("agent"));
}

#[test]
fn batch24_agent_copy_path_desc_mentions_agent() {
    let mut script = ScriptInfo::new("Agent X", "/path/to/agent");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert!(cp.description.as_ref().unwrap().contains("agent"));
}

#[test]
fn batch24_script_edit_desc_mentions_editor() {
    let script = ScriptInfo::new("My Script", "/path/to/script.ts");
    let actions = get_script_context_actions(&script);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert!(edit.description.as_ref().unwrap().contains("$EDITOR"));
}

// ============================================================
// 3. ScriptInfo constructors: is_agent defaults to false
// ============================================================

#[test]
fn batch24_new_is_agent_false() {
    let s = ScriptInfo::new("test", "/path");
    assert!(!s.is_agent);
}

#[test]
fn batch24_builtin_is_agent_false() {
    let s = ScriptInfo::builtin("Clipboard");
    assert!(!s.is_agent);
}

#[test]
fn batch24_scriptlet_is_agent_false() {
    let s = ScriptInfo::scriptlet("Open URL", "/path.md", None, None);
    assert!(!s.is_agent);
}

#[test]
fn batch24_with_shortcut_is_agent_false() {
    let s = ScriptInfo::with_shortcut("test", "/path", Some("cmd+t".to_string()));
    assert!(!s.is_agent);
}

#[test]
fn batch24_with_all_is_agent_false() {
    let s = ScriptInfo::with_all("test", "/path", true, "Run", None, None);
    assert!(!s.is_agent);
}

// ============================================================
// 4. Chat context: has_response/has_messages flag combinations
// ============================================================

#[test]
fn batch24_chat_no_response_no_messages() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    // Only continue_in_chat
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "continue_in_chat");
}

#[test]
fn batch24_chat_response_only() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions.len(), 2);
    assert!(actions.iter().any(|a| a.id == "copy_response"));
    assert!(!actions.iter().any(|a| a.id == "clear_conversation"));
}

#[test]
fn batch24_chat_messages_only() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions.len(), 2);
    assert!(!actions.iter().any(|a| a.id == "copy_response"));
    assert!(actions.iter().any(|a| a.id == "clear_conversation"));
}

#[test]
fn batch24_chat_both_flags() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions.len(), 3);
    assert!(actions.iter().any(|a| a.id == "copy_response"));
    assert!(actions.iter().any(|a| a.id == "clear_conversation"));
}

// ============================================================
// 5. Chat context: model checkmark only for current model
// ============================================================

#[test]
fn batch24_chat_current_model_checkmark() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".to_string()),
        available_models: vec![
            ChatModelInfo {
                id: "gpt4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            },
            ChatModelInfo {
                id: "claude".to_string(),
                display_name: "Claude".to_string(),
                provider: "Anthropic".to_string(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let gpt4 = actions
        .iter()
        .find(|a| a.id == "select_model_gpt4")
        .unwrap();
    assert!(gpt4.title.contains("✓"));
    let claude = actions
        .iter()
        .find(|a| a.id == "select_model_claude")
        .unwrap();
    assert!(!claude.title.contains("✓"));
}

#[test]
fn batch24_chat_no_current_model_no_checkmark() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "gpt4".to_string(),
            display_name: "GPT-4".to_string(),
            provider: "OpenAI".to_string(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let gpt4 = actions
        .iter()
        .find(|a| a.id == "select_model_gpt4")
        .unwrap();
    assert!(!gpt4.title.contains("✓"));
}

#[test]
fn batch24_chat_model_description_via_provider() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "m1".to_string(),
            display_name: "Model One".to_string(),
            provider: "TestProvider".to_string(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let m1 = actions.iter().find(|a| a.id == "select_model_m1").unwrap();
    assert_eq!(m1.description.as_ref().unwrap(), "via TestProvider");
}

// ============================================================
// 6. Clipboard macOS-specific image actions (cfg(target_os = "macos"))
// ============================================================

#[cfg(target_os = "macos")]
#[test]
fn batch24_clipboard_image_has_open_with() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".to_string(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_open_with"));
}

#[cfg(target_os = "macos")]
#[test]
fn batch24_clipboard_image_has_annotate_cleanshot() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".to_string(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions
        .iter()
        .any(|a| a.id == "clipboard_annotate_cleanshot"));
}

#[cfg(target_os = "macos")]
#[test]
fn batch24_clipboard_image_has_upload_cleanshot() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".to_string(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_upload_cleanshot"));
}

#[cfg(target_os = "macos")]
#[test]
fn batch24_clipboard_text_no_open_with() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_open_with"));
}

// ============================================================
// 7. Clipboard: OCR only for image, not text
// ============================================================

#[test]
fn batch24_clipboard_image_has_ocr() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".to_string(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_ocr"));
}

#[test]
fn batch24_clipboard_text_no_ocr() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_ocr"));
}

#[test]
fn batch24_clipboard_ocr_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".to_string(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ocr = actions.iter().find(|a| a.id == "clipboard_ocr").unwrap();
    assert_eq!(ocr.shortcut.as_ref().unwrap(), "⇧⌘C");
}

// ============================================================
// 8. Clipboard: image with None dimensions still gets image actions
// ============================================================

#[test]
fn batch24_clipboard_image_no_dimensions_still_has_ocr() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_ocr"));
}

#[test]
fn batch24_clipboard_image_no_dimensions_has_paste() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_paste"));
}

// ============================================================
// 9. Notes: trash mode minimal actions
// ============================================================

#[test]
fn batch24_notes_trash_minimal_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // Trash: new_note, browse_notes, enable_auto_sizing (3)
    assert_eq!(actions.len(), 3);
}

#[test]
fn batch24_notes_trash_has_new_note() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "new_note"));
}

#[test]
fn batch24_notes_trash_has_browse() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "browse_notes"));
}
