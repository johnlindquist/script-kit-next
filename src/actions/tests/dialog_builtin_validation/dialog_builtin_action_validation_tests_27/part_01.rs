// Batch 27 – Builtin action validation tests
//
// 30 categories · ~119 tests
// Focus areas:
//   - Clipboard: frontmost_app_name dynamic paste title
//   - Scriptlet context: edit_scriptlet shortcut and desc vs script edit_script
//   - Script context: agent action count and ordering invariants
//   - Notes command bar: conditional auto_sizing action presence
//   - AI command bar: paste_image shortcut and section
//   - Chat context: zero models, response+messages combo action counts
//   - New chat: model_idx ID pattern and section assignments
//   - Note switcher: empty preview+empty time falls back to char count
//   - File context: copy_filename vs copy_path shortcut difference
//   - Path context: copy_filename has no shortcut (unlike file context)
//   - format_shortcut_hint (dialog.rs): intermediate modifier in non-last position
//   - to_deeplink_name: repeated hyphens from mixed punctuation
//   - score_action: fuzzy bonus value is 25 (not 50)
//   - build_grouped_items_static: None section skips header even in Headers mode
//   - CommandBarConfig: notes_style uses Separators not Headers
//   - Cross-context: every context's first action has shortcut ↵

use super::builders::*;
use super::command_bar::CommandBarConfig;
use super::dialog::{build_grouped_items_static, coerce_action_selection, GroupedActionItem};
use super::types::*;
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;

// ─────────────────────────────────────────────
// 1. Clipboard: frontmost_app_name dynamic paste title
// ─────────────────────────────────────────────

#[test]
fn cat27_01_clipboard_paste_title_with_app_name() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: Some("Safari".into()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[0].title, "Paste to Safari");
}

#[test]
fn cat27_01_clipboard_paste_title_without_app_name() {
    let entry = ClipboardEntryInfo {
        id: "2".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[0].title, "Paste to Active App");
}

#[test]
fn cat27_01_clipboard_paste_title_with_long_app_name() {
    let entry = ClipboardEntryInfo {
        id: "3".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: Some("Visual Studio Code".into()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions[0].title.contains("Visual Studio Code"));
}

#[test]
fn cat27_01_clipboard_paste_id_is_clipboard_paste() {
    let entry = ClipboardEntryInfo {
        id: "4".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: Some("Finder".into()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[0].id, "clipboard_paste");
}

// ─────────────────────────────────────────────
// 2. Scriptlet context: edit_scriptlet desc mentions markdown
// ─────────────────────────────────────────────

#[test]
fn cat27_02_scriptlet_edit_desc_mentions_markdown() {
    let script = ScriptInfo::scriptlet("My Snippet", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let edit = actions.iter().find(|a| a.id == "edit_scriptlet").unwrap();
    assert!(edit
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("markdown"));
}

#[test]
fn cat27_02_scriptlet_reveal_desc_mentions_bundle() {
    let script = ScriptInfo::scriptlet("My Snippet", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let reveal = actions
        .iter()
        .find(|a| a.id == "reveal_scriptlet_in_finder")
        .unwrap();
    assert!(reveal
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("bundle"));
}

#[test]
fn cat27_02_scriptlet_copy_path_id_is_copy_scriptlet_path() {
    let script = ScriptInfo::scriptlet("My Snippet", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"copy_scriptlet_path"));
}

#[test]
fn cat27_02_scriptlet_has_copy_content() {
    let script = ScriptInfo::scriptlet("My Snippet", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"copy_content"));
}

// ─────────────────────────────────────────────
// 3. Script context: agent action count and ordering
// ─────────────────────────────────────────────

#[test]
fn cat27_03_agent_has_exactly_8_actions_no_shortcut_no_alias() {
    let mut script = ScriptInfo::builtin("my-agent");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    // run_script, add_shortcut, add_alias, edit_script(agent), reveal, copy_path, copy_content, copy_deeplink
    assert_eq!(actions.len(), 8);
}

#[test]
fn cat27_03_agent_first_action_is_run_script() {
    let mut script = ScriptInfo::builtin("my-agent");
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn cat27_03_agent_last_action_is_copy_deeplink() {
    let mut script = ScriptInfo::builtin("my-agent");
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    assert_eq!(actions.last().unwrap().id, "copy_deeplink");
}

#[test]
fn cat27_03_agent_with_suggested_last_is_reset_ranking() {
    let mut script = ScriptInfo::builtin("my-agent");
    script.is_agent = true;
    script.is_suggested = true;
    let actions = get_script_context_actions(&script);
    assert_eq!(actions.last().unwrap().id, "reset_ranking");
}

// ─────────────────────────────────────────────
// 4. Notes command bar: auto_sizing action is conditional
// ─────────────────────────────────────────────

#[test]
fn cat27_04_notes_auto_sizing_absent_when_enabled() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(!ids.contains(&"enable_auto_sizing"));
}

#[test]
fn cat27_04_notes_auto_sizing_present_when_disabled() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"enable_auto_sizing"));
}

#[test]
fn cat27_04_notes_auto_sizing_shortcut() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let auto = actions
        .iter()
        .find(|a| a.id == "enable_auto_sizing")
        .unwrap();
    assert_eq!(auto.shortcut.as_deref(), Some("⌘A"));
}

#[test]
fn cat27_04_notes_auto_sizing_icon_is_settings() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let auto = actions
        .iter()
        .find(|a| a.id == "enable_auto_sizing")
        .unwrap();
    assert_eq!(auto.icon, Some(IconName::Settings));
}

// ─────────────────────────────────────────────
// 5. AI command bar: paste_image details
// ─────────────────────────────────────────────

#[test]
fn cat27_05_ai_paste_image_shortcut() {
    let actions = get_ai_command_bar_actions();
    let paste = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert_eq!(paste.shortcut.as_deref(), Some("⌘V"));
}

#[test]
fn cat27_05_ai_paste_image_section_is_attachments() {
    let actions = get_ai_command_bar_actions();
    let paste = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert_eq!(paste.section.as_deref(), Some("Attachments"));
}

#[test]
fn cat27_05_ai_paste_image_icon_is_file() {
    let actions = get_ai_command_bar_actions();
    let paste = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert_eq!(paste.icon, Some(IconName::File));
}

#[test]
fn cat27_05_ai_paste_image_desc_mentions_clipboard() {
    let actions = get_ai_command_bar_actions();
    let paste = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert!(paste
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("clipboard"));
}

// ─────────────────────────────────────────────
// 6. Chat context: zero models, combo action counts
// ─────────────────────────────────────────────

#[test]
fn cat27_06_chat_zero_models_no_flags_one_action() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "continue_in_chat");
}

#[test]
fn cat27_06_chat_one_model_both_flags_four_actions() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".into()),
        available_models: vec![ChatModelInfo {
            id: "gpt-4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    // 1 model + continue_in_chat + copy_response + clear_conversation = 4
    assert_eq!(actions.len(), 4);
}

#[test]
fn cat27_06_chat_three_models_no_flags() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![
            ChatModelInfo {
                id: "a".into(),
                display_name: "A".into(),
                provider: "P".into(),
            },
            ChatModelInfo {
                id: "b".into(),
                display_name: "B".into(),
                provider: "P".into(),
            },
            ChatModelInfo {
                id: "c".into(),
                display_name: "C".into(),
                provider: "P".into(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    // 3 models + continue_in_chat = 4
    assert_eq!(actions.len(), 4);
}

#[test]
fn cat27_06_chat_response_without_messages() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    // continue_in_chat + copy_response = 2
    assert_eq!(actions.len(), 2);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"copy_response"));
    assert!(!ids.contains(&"clear_conversation"));
}

// ─────────────────────────────────────────────
// 7. New chat: model_idx ID pattern and section
// ─────────────────────────────────────────────

#[test]
fn cat27_07_new_chat_model_ids_use_model_prefix() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt-4o".into(),
        display_name: "GPT-4o".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].id, "model_0");
}

#[test]
fn cat27_07_new_chat_model_section_is_models() {
    let models = vec![NewChatModelInfo {
        model_id: "claude".into(),
        display_name: "Claude".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].section.as_deref(), Some("Models"));
}

#[test]
fn cat27_07_new_chat_preset_ids_use_preset_prefix() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].id, "preset_general");
}

#[test]
fn cat27_07_new_chat_last_used_ids_use_last_used_prefix() {
    let last = vec![NewChatModelInfo {
        model_id: "x".into(),
        display_name: "X Model".into(),
        provider: "p".into(),
        provider_display_name: "Provider".into(),
    }];
    let actions = get_new_chat_actions(&last, &[], &[]);
    assert_eq!(actions[0].id, "last_used_0");
}

// ─────────────────────────────────────────────
// 8. Note switcher: empty preview+empty time→char count
// ─────────────────────────────────────────────

#[test]
fn cat27_08_note_switcher_empty_preview_empty_time_shows_chars() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Test".into(),
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
fn cat27_08_note_switcher_empty_preview_with_time_shows_time_only() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Test".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "5m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("5m ago"));
}
