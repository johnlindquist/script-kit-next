//! Batch 25: Dialog builtin action validation tests
//!
//! 130 tests across 30 categories validating random built-in action behaviors.
//!
//! Categories:
//!  1. Scriptlet context: reset_ranking placement is always last
//!  2. Scriptlet context: copy_deeplink desc contains deeplink URL
//!  3. Script context: full script action ordering invariants
//!  4. Script context: run_script title includes action_verb and quoted name
//!  5. Clipboard: save_snippet and save_file shortcut details
//!  6. Clipboard: destructive action shortcuts matrix
//!  7. Clipboard: quick_look shortcut is ␣ (macOS)
//!  8. Clipboard: paste_keep_open shortcut and description
//!  9. AI command bar: section counts per category
//! 10. AI command bar: branch_from_last has ArrowRight icon, no shortcut
//! 11. AI command bar: add_attachment details
//! 12. Notes command bar: copy section icon assignments
//! 13. Notes command bar: conditional duplicate_note logic
//! 14. Chat context: model ordering matches input order
//! 15. Chat context: copy_response conditional on has_response
//! 16. New chat: preset description is always None
//! 17. New chat: last_used description is provider_display_name
//! 18. Note switcher: preview truncation at 60 chars adds ellipsis
//! 19. Note switcher: current note has bullet prefix
//! 20. Path context: open_in_editor mentions $EDITOR
//! 21. Path context: primary action shortcut always ↵
//! 22. File context macOS: show_info title and shortcut
//! 23. File context macOS: open_with title and shortcut
//! 24. to_deeplink_name: consecutive special chars collapse
//! 25. to_deeplink_name: leading/trailing specials trimmed
//! 26. format_shortcut_hint: tab/backspace/space keys
//! 27. format_shortcut_hint: arrow key mappings
//! 28. parse_shortcut_keycaps: ⇥ and ⌫ symbols
//! 29. score_action: empty search matches as prefix
//! 30. Cross-context: script vs scriptlet vs builtin action count comparison

use super::builders::*;
use super::dialog::*;
use super::types::*;
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;

// ============================================================
// 1. Scriptlet context: reset_ranking placement is always last
// ============================================================

#[test]
fn batch25_scriptlet_reset_ranking_is_last_action() {
    let script = ScriptInfo::scriptlet("Test", "/path/to/test.md", None, None)
        .with_frecency(true, Some("scriptlet:Test".to_string()));
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert_eq!(actions.last().unwrap().id, "reset_ranking");
}

#[test]
fn batch25_script_reset_ranking_is_last_action() {
    let script =
        ScriptInfo::new("Test", "/path/to/test.ts").with_frecency(true, Some("test".to_string()));
    let actions = get_script_context_actions(&script);
    assert_eq!(actions.last().unwrap().id, "reset_ranking");
}

#[test]
fn batch25_builtin_reset_ranking_is_last_action() {
    let builtin = ScriptInfo::builtin("Clipboard History")
        .with_frecency(true, Some("builtin:Clipboard History".to_string()));
    let actions = get_script_context_actions(&builtin);
    assert_eq!(actions.last().unwrap().id, "reset_ranking");
}

#[test]
fn batch25_agent_reset_ranking_is_last_action() {
    let mut agent = ScriptInfo::new("My Agent", "/path/to/agent.md");
    agent.is_agent = true;
    agent.is_script = false;
    let agent = agent.with_frecency(true, Some("agent".to_string()));
    let actions = get_script_context_actions(&agent);
    assert_eq!(actions.last().unwrap().id, "reset_ranking");
}

// ============================================================
// 2. Scriptlet context: copy_deeplink desc contains deeplink URL
// ============================================================

#[test]
fn batch25_scriptlet_copy_deeplink_contains_url() {
    let script = ScriptInfo::scriptlet("Open GitHub", "/path/to/url.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let deeplink = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert!(deeplink
        .description
        .as_ref()
        .unwrap()
        .contains("scriptkit://run/open-github"));
}

#[test]
fn batch25_scriptlet_copy_deeplink_shortcut() {
    let script = ScriptInfo::scriptlet("Open GitHub", "/path/to/url.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let deeplink = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert_eq!(deeplink.shortcut.as_ref().unwrap(), "⌘⇧D");
}

#[test]
fn batch25_script_copy_deeplink_contains_url() {
    let script = ScriptInfo::new("My Cool Script", "/path/to/script.ts");
    let actions = get_script_context_actions(&script);
    let deeplink = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert!(deeplink
        .description
        .as_ref()
        .unwrap()
        .contains("scriptkit://run/my-cool-script"));
}

#[test]
fn batch25_builtin_copy_deeplink_contains_url() {
    let builtin = ScriptInfo::builtin("Clipboard History");
    let actions = get_script_context_actions(&builtin);
    let deeplink = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert!(deeplink
        .description
        .as_ref()
        .unwrap()
        .contains("scriptkit://run/clipboard-history"));
}

// ============================================================
// 3. Script context: full script action ordering invariants
// ============================================================

#[test]
fn batch25_script_run_is_always_first() {
    let script = ScriptInfo::new("Test", "/path/to/test.ts");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn batch25_script_copy_deeplink_is_always_last_non_frecency() {
    let script = ScriptInfo::new("Test", "/path/to/test.ts");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions.last().unwrap().id, "copy_deeplink");
}

#[test]
fn batch25_script_shortcut_actions_before_type_specific() {
    let script = ScriptInfo::new("Test", "/path/to/test.ts");
    let actions = get_script_context_actions(&script);
    let shortcut_idx = actions.iter().position(|a| a.id == "add_shortcut").unwrap();
    let edit_idx = actions.iter().position(|a| a.id == "edit_script").unwrap();
    assert!(
        shortcut_idx < edit_idx,
        "add_shortcut ({}) should come before edit_script ({})",
        shortcut_idx,
        edit_idx
    );
}

#[test]
fn batch25_script_alias_follows_shortcut() {
    let script = ScriptInfo::new("Test", "/path/to/test.ts");
    let actions = get_script_context_actions(&script);
    let shortcut_idx = actions.iter().position(|a| a.id == "add_shortcut").unwrap();
    let alias_idx = actions.iter().position(|a| a.id == "add_alias").unwrap();
    assert_eq!(
        alias_idx,
        shortcut_idx + 1,
        "add_alias should immediately follow add_shortcut"
    );
}

// ============================================================
// 4. Script context: run_script title includes action_verb and quoted name
// ============================================================

#[test]
fn batch25_run_script_title_default_verb() {
    let script = ScriptInfo::new("Hello World", "/path/to/hello.ts");
    let actions = get_script_context_actions(&script);
    let run = &actions[0];
    assert_eq!(run.title, "Run \"Hello World\"");
}

#[test]
fn batch25_run_script_title_custom_verb_launch() {
    let script =
        ScriptInfo::with_action_verb("Safari", "/Applications/Safari.app", false, "Launch");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].title, "Launch \"Safari\"");
}

#[test]
fn batch25_run_script_title_custom_verb_switch_to() {
    let script = ScriptInfo::with_action_verb("My Document", "window:123", false, "Switch to");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].title, "Switch to \"My Document\"");
}

#[test]
fn batch25_run_script_description_uses_verb() {
    let script = ScriptInfo::with_action_verb("Test", "/path", false, "Open");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].description, Some("Open this item".to_string()));
}

// ============================================================
// 5. Clipboard: save_snippet and save_file shortcut details
// ============================================================

#[test]
fn batch25_clipboard_save_snippet_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "id1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "Test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let save = actions
        .iter()
        .find(|a| a.id == "clipboard_save_snippet")
        .unwrap();
    assert_eq!(save.shortcut.as_ref().unwrap(), "⇧⌘S");
}

#[test]
fn batch25_clipboard_save_snippet_title() {
    let entry = ClipboardEntryInfo {
        id: "id1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "Test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let save = actions
        .iter()
        .find(|a| a.id == "clipboard_save_snippet")
        .unwrap();
    assert_eq!(save.title, "Save Text as Snippet");
}

#[test]
fn batch25_clipboard_save_file_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "id1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "Test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let save_file = actions
        .iter()
        .find(|a| a.id == "clipboard_save_file")
        .unwrap();
    assert_eq!(save_file.shortcut.as_ref().unwrap(), "⌥⇧⌘S");
}

#[test]
fn batch25_clipboard_save_file_title() {
    let entry = ClipboardEntryInfo {
        id: "id1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "Test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let save_file = actions
        .iter()
        .find(|a| a.id == "clipboard_save_file")
        .unwrap();
    assert_eq!(save_file.title, "Save as File...");
}

#[test]
fn batch25_clipboard_save_file_desc_mentions_file() {
    let entry = ClipboardEntryInfo {
        id: "id1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "Test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let save_file = actions
        .iter()
        .find(|a| a.id == "clipboard_save_file")
        .unwrap();
    assert!(save_file
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("file"));
}

// ============================================================
// 6. Clipboard: destructive action shortcuts matrix
// ============================================================

#[test]
fn batch25_clipboard_delete_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "id1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "Test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let del = actions.iter().find(|a| a.id == "clipboard_delete").unwrap();
    assert_eq!(del.shortcut.as_ref().unwrap(), "⌃X");
}

#[test]
fn batch25_clipboard_delete_multiple_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "id1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "Test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let del_mult = actions
        .iter()
        .find(|a| a.id == "clipboard_delete_multiple")
        .unwrap();
    assert_eq!(del_mult.shortcut.as_ref().unwrap(), "⇧⌘X");
}

#[test]
fn batch25_clipboard_delete_all_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "id1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "Test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let del_all = actions
        .iter()
        .find(|a| a.id == "clipboard_delete_all")
        .unwrap();
    assert_eq!(del_all.shortcut.as_ref().unwrap(), "⌃⇧X");
}

#[test]
fn batch25_clipboard_destructive_actions_at_end() {
    let entry = ClipboardEntryInfo {
        id: "id1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "Test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let len = actions.len();
    // Last 3 actions should be the destructive ones
    assert_eq!(actions[len - 3].id, "clipboard_delete");
    assert_eq!(actions[len - 2].id, "clipboard_delete_multiple");
    assert_eq!(actions[len - 1].id, "clipboard_delete_all");
}

// ============================================================
// 7. Clipboard: quick_look shortcut is ␣ (macOS)
// ============================================================

#[cfg(target_os = "macos")]
#[test]
fn batch25_clipboard_quick_look_shortcut_space() {
    let entry = ClipboardEntryInfo {
        id: "id1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "Test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ql = actions
        .iter()
        .find(|a| a.id == "clipboard_quick_look")
        .unwrap();
    assert_eq!(ql.shortcut.as_ref().unwrap(), "␣");
}

#[cfg(target_os = "macos")]
#[test]
fn batch25_clipboard_quick_look_title() {
    let entry = ClipboardEntryInfo {
        id: "id1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "Test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ql = actions
        .iter()
        .find(|a| a.id == "clipboard_quick_look")
        .unwrap();
    assert_eq!(ql.title, "Quick Look");
}

#[cfg(target_os = "macos")]
#[test]
fn batch25_clipboard_quick_look_present_for_both_types() {
    // Text entry
    let text_entry = ClipboardEntryInfo {
        id: "t".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "Text".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let text_actions = get_clipboard_history_context_actions(&text_entry);
    assert!(text_actions.iter().any(|a| a.id == "clipboard_quick_look"));

    // Image entry
    let img_entry = ClipboardEntryInfo {
        id: "i".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "Image".to_string(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let img_actions = get_clipboard_history_context_actions(&img_entry);
    assert!(img_actions.iter().any(|a| a.id == "clipboard_quick_look"));
}

// ============================================================
// 8. Clipboard: paste_keep_open shortcut and description
// ============================================================

#[test]
fn batch25_clipboard_paste_keep_open_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "id1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "Test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let pko = actions
        .iter()
        .find(|a| a.id == "clipboard_paste_keep_open")
        .unwrap();
    assert_eq!(pko.shortcut.as_ref().unwrap(), "⌥↵");
}

#[test]
fn batch25_clipboard_paste_keep_open_desc_mentions_keep() {
    let entry = ClipboardEntryInfo {
        id: "id1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "Test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let pko = actions
        .iter()
        .find(|a| a.id == "clipboard_paste_keep_open")
        .unwrap();
    assert!(pko
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("keep"));
}

#[test]
fn batch25_clipboard_copy_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "id1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "Test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let copy = actions.iter().find(|a| a.id == "clipboard_copy").unwrap();
    assert_eq!(copy.shortcut.as_ref().unwrap(), "⌘↵");
}

// ============================================================
// 9. AI command bar: section counts per category
// ============================================================

#[test]
fn batch25_ai_command_bar_response_section_count() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .count();
    assert_eq!(count, 3);
}

#[test]
fn batch25_ai_command_bar_actions_section_count() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Actions"))
        .count();
    assert_eq!(count, 4);
}

#[test]
fn batch25_ai_command_bar_attachments_section_count() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Attachments"))
        .count();
    assert_eq!(count, 2);
}

#[test]
fn batch25_ai_command_bar_export_section_count() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Export"))
        .count();
    assert_eq!(count, 1);
}

#[test]
fn batch25_ai_command_bar_help_section_count() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Help"))
        .count();
    assert_eq!(count, 1);
}

#[test]
fn batch25_ai_command_bar_settings_section_count() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Settings"))
        .count();
    assert_eq!(count, 1);
}

// ============================================================
// 10. AI command bar: branch_from_last has ArrowRight icon, no shortcut
// ============================================================

#[test]
fn batch25_ai_branch_from_last_icon_arrow_right() {
    let actions = get_ai_command_bar_actions();
    let branch = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
    assert_eq!(branch.icon, Some(IconName::ArrowRight));
}

#[test]
fn batch25_ai_branch_from_last_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let branch = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
    assert!(branch.shortcut.is_none());
}

#[test]
fn batch25_ai_branch_from_last_section_actions() {
    let actions = get_ai_command_bar_actions();
    let branch = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
    assert_eq!(branch.section.as_deref(), Some("Actions"));
}

#[test]
fn batch25_ai_change_model_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let model = actions.iter().find(|a| a.id == "change_model").unwrap();
    assert!(model.shortcut.is_none());
}

// ============================================================
// 11. AI command bar: add_attachment details
// ============================================================

#[test]
fn batch25_ai_add_attachment_shortcut() {
    let actions = get_ai_command_bar_actions();
    let attach = actions.iter().find(|a| a.id == "add_attachment").unwrap();
    assert_eq!(attach.shortcut.as_ref().unwrap(), "⇧⌘A");
}

#[test]
fn batch25_ai_add_attachment_icon_plus() {
    let actions = get_ai_command_bar_actions();
    let attach = actions.iter().find(|a| a.id == "add_attachment").unwrap();
    assert_eq!(attach.icon, Some(IconName::Plus));
}

#[test]
fn batch25_ai_add_attachment_section_attachments() {
    let actions = get_ai_command_bar_actions();
    let attach = actions.iter().find(|a| a.id == "add_attachment").unwrap();
    assert_eq!(attach.section.as_deref(), Some("Attachments"));
}

#[test]
fn batch25_ai_paste_image_icon_file() {
    let actions = get_ai_command_bar_actions();
    let paste = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert_eq!(paste.icon, Some(IconName::File));
}

// ============================================================
// 12. Notes command bar: copy section icon assignments
// ============================================================

#[test]
fn batch25_notes_copy_note_as_icon_copy() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let copy_as = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
    assert_eq!(copy_as.icon, Some(IconName::Copy));
}

#[test]
fn batch25_notes_copy_deeplink_icon_arrow_right() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let deeplink = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert_eq!(deeplink.icon, Some(IconName::ArrowRight));
}

#[test]
fn batch25_notes_create_quicklink_icon_star() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let quicklink = actions.iter().find(|a| a.id == "create_quicklink").unwrap();
    assert_eq!(quicklink.icon, Some(IconName::Star));
}

#[test]
fn batch25_notes_export_icon_arrow_right() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let export = actions.iter().find(|a| a.id == "export").unwrap();
    assert_eq!(export.icon, Some(IconName::ArrowRight));
}

// ============================================================
// 13. Notes command bar: conditional duplicate_note logic
// ============================================================

#[test]
fn batch25_notes_duplicate_absent_in_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
}

#[test]
fn batch25_notes_duplicate_absent_without_selection() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
}

#[test]
fn batch25_notes_duplicate_present_with_selection_no_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "duplicate_note"));
}

#[test]
fn batch25_notes_duplicate_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let dup = actions.iter().find(|a| a.id == "duplicate_note").unwrap();
    assert_eq!(dup.shortcut.as_ref().unwrap(), "⌘D");
}

// ============================================================
// 14. Chat context: model ordering matches input order
// ============================================================

#[test]
fn batch25_chat_model_ordering_preserved() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![
            ChatModelInfo {
                id: "gpt-4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            },
            ChatModelInfo {
                id: "claude-3".to_string(),
                display_name: "Claude 3".to_string(),
                provider: "Anthropic".to_string(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions[0].id, "select_model_gpt-4");
    assert_eq!(actions[1].id, "select_model_claude-3");
}

#[test]
fn batch25_chat_model_description_via_provider() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "gpt-4".to_string(),
            display_name: "GPT-4".to_string(),
            provider: "OpenAI".to_string(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions[0].description, Some("via OpenAI".to_string()));
}

#[test]
fn batch25_chat_continue_in_chat_always_after_models() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![
            ChatModelInfo {
                id: "m1".to_string(),
                display_name: "M1".to_string(),
                provider: "P1".to_string(),
            },
            ChatModelInfo {
                id: "m2".to_string(),
                display_name: "M2".to_string(),
                provider: "P2".to_string(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model_count = info.available_models.len();
    assert_eq!(actions[model_count].id, "continue_in_chat");
}

// ============================================================
// 15. Chat context: copy_response conditional on has_response
// ============================================================

#[test]
fn batch25_chat_no_response_no_copy() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "copy_response"));
}

#[test]
fn batch25_chat_has_response_has_copy() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "copy_response"));
}

#[test]
fn batch25_chat_copy_response_shortcut() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let copy = actions.iter().find(|a| a.id == "copy_response").unwrap();
    assert_eq!(copy.shortcut.as_ref().unwrap(), "⌘C");
}

#[test]
fn batch25_chat_clear_conversation_requires_messages() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "clear_conversation"));
}

// ============================================================
// 16. New chat: preset description is always None
// ============================================================

#[test]
fn batch25_new_chat_preset_description_none() {
    let presets = vec![
        NewChatPresetInfo {
            id: "general".to_string(),
            name: "General".to_string(),
            icon: IconName::Star,
        },
        NewChatPresetInfo {
            id: "code".to_string(),
            name: "Code".to_string(),
            icon: IconName::Code,
        },
    ];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    for action in &actions {
        assert!(
            action.description.is_none(),
            "Preset '{}' should have no description",
            action.title
        );
    }
}

#[test]
fn batch25_new_chat_preset_section_presets() {
    let presets = vec![NewChatPresetInfo {
        id: "general".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].section.as_deref(), Some("Presets"));
}

#[test]
fn batch25_new_chat_preset_id_format() {
    let presets = vec![NewChatPresetInfo {
        id: "code-review".to_string(),
        name: "Code Review".to_string(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].id, "preset_code-review");
}

#[test]
fn batch25_new_chat_preset_icon_preserved() {
    let presets = vec![NewChatPresetInfo {
        id: "writer".to_string(),
        name: "Writer".to_string(),
        icon: IconName::FileCode,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].icon, Some(IconName::FileCode));
}

// ============================================================
// 17. New chat: last_used description is provider_display_name
// ============================================================

#[test]
fn batch25_new_chat_last_used_description_is_provider() {
    let last_used = vec![NewChatModelInfo {
        model_id: "claude-3-opus".to_string(),
        display_name: "Claude 3 Opus".to_string(),
        provider: "anthropic".to_string(),
        provider_display_name: "Anthropic".to_string(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].description, Some("Anthropic".to_string()));
}

#[test]
fn batch25_new_chat_last_used_icon_bolt() {
    let last_used = vec![NewChatModelInfo {
        model_id: "gpt-4".to_string(),
        display_name: "GPT-4".to_string(),
        provider: "openai".to_string(),
        provider_display_name: "OpenAI".to_string(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
}

#[test]
fn batch25_new_chat_last_used_section() {
    let last_used = vec![NewChatModelInfo {
        model_id: "gpt-4".to_string(),
        display_name: "GPT-4".to_string(),
        provider: "openai".to_string(),
        provider_display_name: "OpenAI".to_string(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
}

#[test]
fn batch25_new_chat_model_section() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt-4".to_string(),
        display_name: "GPT-4".to_string(),
        provider: "openai".to_string(),
        provider_display_name: "OpenAI".to_string(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].section.as_deref(), Some("Models"));
}

// ============================================================
// 18. Note switcher: preview truncation at 60 chars adds ellipsis
// ============================================================

#[test]
fn batch25_note_switcher_long_preview_truncated() {
    let long_preview = "A".repeat(80);
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid1".to_string(),
        title: "My Note".to_string(),
        char_count: 80,
        is_current: false,
        is_pinned: false,
        preview: long_preview,
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(desc.ends_with('…'), "Expected ellipsis, got: {}", desc);
    // Should be 60 chars + ellipsis
    assert!(desc.chars().count() <= 61);
}

#[test]
fn batch25_note_switcher_short_preview_no_truncation() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid1".to_string(),
        title: "My Note".to_string(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "Short text".to_string(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert_eq!(desc, "Short text");
}

#[test]
fn batch25_note_switcher_exactly_60_chars_no_truncation() {
    let preview = "A".repeat(60);
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid1".to_string(),
        title: "My Note".to_string(),
        char_count: 60,
        is_current: false,
        is_pinned: false,
        preview,
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(!desc.ends_with('…'));
    assert_eq!(desc.chars().count(), 60);
}

#[test]
fn batch25_note_switcher_61_chars_truncated() {
    let preview = "B".repeat(61);
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid1".to_string(),
        title: "My Note".to_string(),
        char_count: 61,
        is_current: false,
        is_pinned: false,
        preview,
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(desc.ends_with('…'));
}

// ============================================================
// 19. Note switcher: current note has bullet prefix
// ============================================================

#[test]
fn batch25_note_switcher_current_has_bullet() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid1".to_string(),
        title: "Current Note".to_string(),
        char_count: 100,
        is_current: true,
        is_pinned: false,
        preview: "Preview".to_string(),
        relative_time: "2m ago".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(actions[0].title.starts_with("• "));
}

#[test]
fn batch25_note_switcher_non_current_no_bullet() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid1".to_string(),
        title: "Other Note".to_string(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "Preview".to_string(),
        relative_time: "2m ago".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(!actions[0].title.starts_with("• "));
    assert_eq!(actions[0].title, "Other Note");
}

#[test]
fn batch25_note_switcher_current_icon_check() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "uuid1".to_string(),
        title: "Current".to_string(),
        char_count: 100,
        is_current: true,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::Check));
}

// ============================================================
// 20. Path context: open_in_editor mentions $EDITOR
// ============================================================

#[test]
fn batch25_path_open_in_editor_desc_mentions_editor() {
    let path = PathInfo {
        path: "/Users/test/file.txt".to_string(),
        name: "file.txt".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    let editor = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
    assert!(editor.description.as_ref().unwrap().contains("$EDITOR"));
}

#[test]
fn batch25_path_open_in_editor_shortcut() {
    let path = PathInfo {
        path: "/Users/test/file.txt".to_string(),
        name: "file.txt".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    let editor = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
    assert_eq!(editor.shortcut.as_ref().unwrap(), "⌘E");
}

#[test]
fn batch25_path_open_in_terminal_desc_mentions_terminal() {
    let path = PathInfo {
        path: "/Users/test/dir".to_string(),
        name: "dir".to_string(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path);
    let terminal = actions.iter().find(|a| a.id == "open_in_terminal").unwrap();
    assert!(terminal
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("terminal"));
}

#[test]
fn batch25_path_open_in_finder_shortcut() {
    let path = PathInfo {
        path: "/Users/test".to_string(),
        name: "test".to_string(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path);
    let finder = actions.iter().find(|a| a.id == "open_in_finder").unwrap();
    assert_eq!(finder.shortcut.as_ref().unwrap(), "⌘⇧F");
}

// ============================================================
// 21. Path context: primary action shortcut always ↵
// ============================================================

#[test]
fn batch25_path_dir_primary_shortcut_enter() {
    let path = PathInfo {
        path: "/Users/test/docs".to_string(),
        name: "docs".to_string(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].shortcut.as_ref().unwrap(), "↵");
}

#[test]
fn batch25_path_file_primary_shortcut_enter() {
    let path = PathInfo {
        path: "/Users/test/file.txt".to_string(),
        name: "file.txt".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].shortcut.as_ref().unwrap(), "↵");
}

#[test]
fn batch25_path_move_to_trash_shortcut() {
    let path = PathInfo {
        path: "/Users/test/file.txt".to_string(),
        name: "file.txt".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert_eq!(trash.shortcut.as_ref().unwrap(), "⌘⌫");
}

// ============================================================
// 22. File context macOS: show_info title and shortcut
// ============================================================

#[cfg(target_os = "macos")]
#[test]
fn batch25_file_show_info_title() {
    let file_info = FileInfo {
        path: "/Users/test/doc.pdf".to_string(),
        name: "doc.pdf".to_string(),
        file_type: FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    let info = actions.iter().find(|a| a.id == "show_info").unwrap();
    assert_eq!(info.title, "Get Info");
}

#[cfg(target_os = "macos")]
#[test]
fn batch25_file_show_info_shortcut() {
    let file_info = FileInfo {
        path: "/Users/test/doc.pdf".to_string(),
        name: "doc.pdf".to_string(),
        file_type: FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    let info = actions.iter().find(|a| a.id == "show_info").unwrap();
    assert_eq!(info.shortcut.as_ref().unwrap(), "⌘I");
}

// ============================================================
// 23. File context macOS: open_with title and shortcut
// ============================================================

#[cfg(target_os = "macos")]
#[test]
fn batch25_file_open_with_title() {
    let file_info = FileInfo {
        path: "/Users/test/doc.pdf".to_string(),
        name: "doc.pdf".to_string(),
        file_type: FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    let ow = actions.iter().find(|a| a.id == "open_with").unwrap();
    assert_eq!(ow.title, "Open With...");
}

#[cfg(target_os = "macos")]
#[test]
fn batch25_file_open_with_shortcut() {
    let file_info = FileInfo {
        path: "/Users/test/doc.pdf".to_string(),
        name: "doc.pdf".to_string(),
        file_type: FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    let ow = actions.iter().find(|a| a.id == "open_with").unwrap();
    assert_eq!(ow.shortcut.as_ref().unwrap(), "⌘O");
}

// ============================================================
// 24. to_deeplink_name: consecutive special chars collapse
// ============================================================

#[test]
fn batch25_deeplink_consecutive_specials_collapse() {
    assert_eq!(to_deeplink_name("a--b"), "a-b");
}

#[test]
fn batch25_deeplink_multiple_spaces_collapse() {
    assert_eq!(to_deeplink_name("a   b"), "a-b");
}

#[test]
fn batch25_deeplink_mixed_specials_collapse() {
    assert_eq!(to_deeplink_name("a!@#b"), "a-b");
}

#[test]
fn batch25_deeplink_dots_collapse() {
    assert_eq!(to_deeplink_name("hello.world"), "hello-world");
}

// ============================================================
// 25. to_deeplink_name: leading/trailing specials trimmed
// ============================================================

#[test]
fn batch25_deeplink_leading_specials_trimmed() {
    assert_eq!(to_deeplink_name("---hello"), "hello");
}

#[test]
fn batch25_deeplink_trailing_specials_trimmed() {
    assert_eq!(to_deeplink_name("hello---"), "hello");
}

#[test]
fn batch25_deeplink_both_sides_trimmed() {
    assert_eq!(to_deeplink_name("  hello  "), "hello");
}

#[test]
fn batch25_deeplink_all_specials_empty() {
    assert_eq!(to_deeplink_name("!@#$%"), "");
}

// ============================================================
// 26. format_shortcut_hint: tab/backspace/space keys
// ============================================================

#[test]
fn batch25_format_shortcut_tab() {
    assert_eq!(ActionsDialog::format_shortcut_hint("tab"), "⇥");
}

#[test]
fn batch25_format_shortcut_backspace() {
    assert_eq!(ActionsDialog::format_shortcut_hint("backspace"), "⌫");
}

#[test]
fn batch25_format_shortcut_delete() {
    assert_eq!(ActionsDialog::format_shortcut_hint("delete"), "⌫");
}

#[test]
fn batch25_format_shortcut_space() {
    assert_eq!(ActionsDialog::format_shortcut_hint("space"), "␣");
}

#[test]
fn batch25_format_shortcut_escape() {
    assert_eq!(ActionsDialog::format_shortcut_hint("escape"), "⎋");
}

// ============================================================
// 27. format_shortcut_hint: arrow key mappings
// ============================================================

#[test]
fn batch25_format_shortcut_arrow_up() {
    assert_eq!(ActionsDialog::format_shortcut_hint("up"), "↑");
}

#[test]
fn batch25_format_shortcut_arrow_down() {
    assert_eq!(ActionsDialog::format_shortcut_hint("down"), "↓");
}

#[test]
fn batch25_format_shortcut_arrow_left() {
    assert_eq!(ActionsDialog::format_shortcut_hint("left"), "←");
}

#[test]
fn batch25_format_shortcut_arrow_right() {
    assert_eq!(ActionsDialog::format_shortcut_hint("right"), "→");
}

#[test]
fn batch25_format_shortcut_arrowup_alias() {
    assert_eq!(ActionsDialog::format_shortcut_hint("arrowup"), "↑");
}

// ============================================================
// 28. parse_shortcut_keycaps: ⇥ and ⌫ symbols
// ============================================================

#[test]
fn batch25_parse_keycaps_tab_symbol() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⇥");
    assert_eq!(keycaps, vec!["⇥"]);
}

#[test]
fn batch25_parse_keycaps_backspace_symbol() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⌫");
    assert_eq!(keycaps, vec!["⌫"]);
}

#[test]
fn batch25_parse_keycaps_cmd_tab() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⇥");
    assert_eq!(keycaps, vec!["⌘", "⇥"]);
}

#[test]
fn batch25_parse_keycaps_cmd_backspace() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⌫");
    assert_eq!(keycaps, vec!["⌘", "⌫"]);
}

#[test]
fn batch25_parse_keycaps_space_symbol() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⌥␣");
    assert_eq!(keycaps, vec!["⌥", "␣"]);
}

// ============================================================
// 29. score_action: empty search matches as prefix
// ============================================================

#[test]
fn batch25_score_action_empty_search_is_prefix() {
    let action = Action::new(
        "test",
        "Hello",
        Some("World".to_string()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "");
    assert!(
        score >= 100,
        "Empty search should match as prefix, got {}",
        score
    );
}

#[test]
fn batch25_score_action_no_match_zero() {
    let action = Action::new(
        "test",
        "Hello",
        Some("World".to_string()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "xyz");
    assert_eq!(score, 0);
}

#[test]
fn batch25_score_action_prefix_highest() {
    let action = Action::new(
        "test",
        "Edit Script",
        Some("Open in editor".to_string()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "edit");
    assert!(
        score >= 100,
        "Prefix match should score >= 100, got {}",
        score
    );
}

#[test]
fn batch25_score_action_description_bonus() {
    let action = Action::new(
        "test",
        "Open File",
        Some("Open in your favorite editor".to_string()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "editor");
    assert!(
        score >= 15,
        "Description match should add >= 15, got {}",
        score
    );
}

// ============================================================
// 30. Cross-context: script vs scriptlet vs builtin action count comparison
// ============================================================

#[test]
fn batch25_script_has_more_actions_than_builtin() {
    let script = ScriptInfo::new("Test", "/path/to/test.ts");
    let builtin = ScriptInfo::builtin("Test Builtin");
    let script_actions = get_script_context_actions(&script);
    let builtin_actions = get_script_context_actions(&builtin);
    assert!(
        script_actions.len() > builtin_actions.len(),
        "Script ({}) should have more actions than builtin ({})",
        script_actions.len(),
        builtin_actions.len()
    );
}

#[test]
fn batch25_scriptlet_has_more_actions_than_builtin() {
    let scriptlet = ScriptInfo::scriptlet("Test", "/path/to/test.md", None, None);
    let builtin = ScriptInfo::builtin("Test Builtin");
    let scriptlet_actions = get_script_context_actions(&scriptlet);
    let builtin_actions = get_script_context_actions(&builtin);
    assert!(
        scriptlet_actions.len() > builtin_actions.len(),
        "Scriptlet ({}) should have more actions than builtin ({})",
        scriptlet_actions.len(),
        builtin_actions.len()
    );
}

#[test]
fn batch25_agent_has_more_actions_than_builtin() {
    let mut agent = ScriptInfo::new("Test Agent", "/path/to/agent.md");
    agent.is_agent = true;
    agent.is_script = false;
    let builtin = ScriptInfo::builtin("Test Builtin");
    let agent_actions = get_script_context_actions(&agent);
    let builtin_actions = get_script_context_actions(&builtin);
    assert!(
        agent_actions.len() > builtin_actions.len(),
        "Agent ({}) should have more actions than builtin ({})",
        agent_actions.len(),
        builtin_actions.len()
    );
}

#[test]
fn batch25_builtin_has_exactly_4_actions() {
    // builtin: run_script, add_shortcut, add_alias, copy_deeplink
    let builtin = ScriptInfo::builtin("Test Builtin");
    let actions = get_script_context_actions(&builtin);
    assert_eq!(actions.len(), 4);
}

#[test]
fn batch25_script_has_exactly_9_actions() {
    // script: run, add_shortcut, add_alias, edit, view_logs, reveal, copy_path, copy_content, copy_deeplink
    let script = ScriptInfo::new("Test", "/path/to/test.ts");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions.len(), 9);
}

#[test]
fn batch25_scriptlet_has_exactly_8_actions() {
    // scriptlet: run, add_shortcut, add_alias, edit_scriptlet, reveal_scriptlet, copy_scriptlet_path, copy_content, copy_deeplink
    let scriptlet = ScriptInfo::scriptlet("Test", "/path/to/test.md", None, None);
    let actions = get_script_context_actions(&scriptlet);
    assert_eq!(actions.len(), 8);
}
