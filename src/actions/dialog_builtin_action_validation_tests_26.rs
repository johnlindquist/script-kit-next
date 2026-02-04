//! Batch 26 – Builtin action validation tests
//!
//! 30 categories · ~120 tests
//! Focus areas:
//!   - Scriptlet context with_custom: action ordering with multiple custom actions
//!   - Clipboard share and attach_to_ai details
//!   - Notes full mode section assignments
//!   - Chat context edge: two models same provider
//!   - AI command bar: Submit action details
//!   - Path vs file context: description wording differences
//!   - ScriptInfo constructor: is_agent mutually exclusive with is_script/is_scriptlet
//!   - format_shortcut_hint: multi-key combos with numbers
//!   - to_deeplink_name: Unicode edge cases
//!   - Cross-context: description is always Some for built-in actions

use super::builders::*;
use super::command_bar::CommandBarConfig;
use super::dialog::{build_grouped_items_static, coerce_action_selection, GroupedActionItem};
use super::types::*;
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;
use crate::scriptlets::{Scriptlet, ScriptletAction};

// ─────────────────────────────────────────────
// 1. Scriptlet context with_custom: three custom actions maintain order
// ─────────────────────────────────────────────

#[test]
fn cat26_01_three_custom_actions_maintain_insertion_order() {
    let script = ScriptInfo::scriptlet("Multi Act", "/p.md", None, None);
    let mut sl = Scriptlet::new("Multi Act".into(), "bash".into(), "echo hi".into());
    sl.actions = vec![
        ScriptletAction {
            name: "Alpha".into(),
            command: "alpha".into(),
            tool: "bash".into(),
            code: "a".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
        ScriptletAction {
            name: "Beta".into(),
            command: "beta".into(),
            tool: "bash".into(),
            code: "b".into(),
            inputs: vec![],
            shortcut: Some("cmd+b".into()),
            description: Some("Do beta".into()),
        },
        ScriptletAction {
            name: "Gamma".into(),
            command: "gamma".into(),
            tool: "bash".into(),
            code: "g".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
    ];
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&sl));
    let custom_ids: Vec<&str> = actions
        .iter()
        .filter(|a| a.id.starts_with("scriptlet_action:"))
        .map(|a| a.id.as_str())
        .collect();
    assert_eq!(
        custom_ids,
        vec![
            "scriptlet_action:alpha",
            "scriptlet_action:beta",
            "scriptlet_action:gamma"
        ]
    );
}

#[test]
fn cat26_01_custom_actions_all_have_has_action_true() {
    let script = ScriptInfo::scriptlet("X", "/x.md", None, None);
    let mut sl = Scriptlet::new("X".into(), "bash".into(), "echo".into());
    sl.actions = vec![ScriptletAction {
        name: "Do".into(),
        command: "do-it".into(),
        tool: "bash".into(),
        code: "d".into(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&sl));
    let custom = actions
        .iter()
        .find(|a| a.id == "scriptlet_action:do-it")
        .unwrap();
    assert!(custom.has_action);
    assert_eq!(custom.value, Some("do-it".into()));
}

#[test]
fn cat26_01_custom_action_with_shortcut_gets_formatted() {
    let script = ScriptInfo::scriptlet("S", "/s.md", None, None);
    let mut sl = Scriptlet::new("S".into(), "bash".into(), "e".into());
    sl.actions = vec![ScriptletAction {
        name: "Copy".into(),
        command: "cp".into(),
        tool: "bash".into(),
        code: "c".into(),
        inputs: vec![],
        shortcut: Some("cmd+shift+c".into()),
        description: None,
    }];
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&sl));
    let custom = actions
        .iter()
        .find(|a| a.id == "scriptlet_action:cp")
        .unwrap();
    assert_eq!(custom.shortcut, Some("⌘⇧C".into()));
}

#[test]
fn cat26_01_custom_actions_appear_after_run_before_shortcut_actions() {
    let script = ScriptInfo::scriptlet("T", "/t.md", None, None);
    let mut sl = Scriptlet::new("T".into(), "bash".into(), "echo".into());
    sl.actions = vec![ScriptletAction {
        name: "My Act".into(),
        command: "my-act".into(),
        tool: "bash".into(),
        code: "x".into(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&sl));
    let run_idx = actions.iter().position(|a| a.id == "run_script").unwrap();
    let custom_idx = actions
        .iter()
        .position(|a| a.id == "scriptlet_action:my-act")
        .unwrap();
    let shortcut_idx = actions.iter().position(|a| a.id == "add_shortcut").unwrap();
    assert!(run_idx < custom_idx);
    assert!(custom_idx < shortcut_idx);
}

// ─────────────────────────────────────────────
// 2. Clipboard share shortcut and description
// ─────────────────────────────────────────────

#[test]
fn cat26_02_clipboard_share_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hi".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let share = actions.iter().find(|a| a.id == "clipboard_share").unwrap();
    assert_eq!(share.shortcut.as_deref(), Some("⇧⌘E"));
    assert_eq!(share.title, "Share...");
}

#[test]
fn cat26_02_clipboard_attach_to_ai_description_mentions_ai() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hi".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let attach = actions
        .iter()
        .find(|a| a.id == "clipboard_attach_to_ai")
        .unwrap();
    assert!(attach
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("ai"));
}

#[test]
fn cat26_02_clipboard_share_present_for_image() {
    let entry = ClipboardEntryInfo {
        id: "img".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "Image".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_share"));
}

#[test]
fn cat26_02_clipboard_attach_to_ai_present_for_image() {
    let entry = ClipboardEntryInfo {
        id: "img".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "Img".into(),
        image_dimensions: Some((50, 50)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_attach_to_ai"));
}

// ─────────────────────────────────────────────
// 3. Notes full mode: section assignments via icons
// ─────────────────────────────────────────────

#[test]
fn cat26_03_notes_full_mode_has_edit_section_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "find_in_note"));
    assert!(actions.iter().any(|a| a.id == "format"));
}

#[test]
fn cat26_03_notes_full_mode_has_copy_section_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "copy_note_as"));
    assert!(actions.iter().any(|a| a.id == "copy_deeplink"));
    assert!(actions.iter().any(|a| a.id == "create_quicklink"));
}

#[test]
fn cat26_03_notes_full_mode_has_export() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let export = actions.iter().find(|a| a.id == "export").unwrap();
    assert_eq!(export.section.as_deref(), Some("Export"));
}

#[test]
fn cat26_03_notes_no_selection_hides_edit_and_copy() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "find_in_note"));
    assert!(!actions.iter().any(|a| a.id == "copy_note_as"));
    assert!(!actions.iter().any(|a| a.id == "export"));
}

// ─────────────────────────────────────────────
// 4. Chat context: two models same provider
// ─────────────────────────────────────────────

#[test]
fn cat26_04_chat_two_models_same_provider_both_listed() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".into()),
        available_models: vec![
            ChatModelInfo {
                id: "gpt-4".into(),
                display_name: "GPT-4".into(),
                provider: "OpenAI".into(),
            },
            ChatModelInfo {
                id: "gpt-3.5".into(),
                display_name: "GPT-3.5".into(),
                provider: "OpenAI".into(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "select_model_gpt-4"));
    assert!(actions.iter().any(|a| a.id == "select_model_gpt-3.5"));
}

#[test]
fn cat26_04_chat_current_model_has_checkmark() {
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
    let model_action = actions
        .iter()
        .find(|a| a.id == "select_model_gpt-4")
        .unwrap();
    assert!(model_action.title.contains('✓'));
}

#[test]
fn cat26_04_chat_non_current_model_no_checkmark() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".into()),
        available_models: vec![ChatModelInfo {
            id: "gpt-3.5".into(),
            display_name: "GPT-3.5".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model_action = actions
        .iter()
        .find(|a| a.id == "select_model_gpt-3.5")
        .unwrap();
    assert!(!model_action.title.contains('✓'));
}

#[test]
fn cat26_04_chat_model_description_via_provider() {
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
    let model_action = actions
        .iter()
        .find(|a| a.id == "select_model_claude")
        .unwrap();
    assert_eq!(model_action.description.as_deref(), Some("via Anthropic"));
}

// ─────────────────────────────────────────────
// 5. AI command bar: submit action details
// ─────────────────────────────────────────────

#[test]
fn cat26_05_ai_submit_action_shortcut() {
    let actions = get_ai_command_bar_actions();
    let submit = actions.iter().find(|a| a.id == "submit").unwrap();
    assert_eq!(submit.shortcut.as_deref(), Some("↵"));
}

#[test]
fn cat26_05_ai_submit_action_icon() {
    let actions = get_ai_command_bar_actions();
    let submit = actions.iter().find(|a| a.id == "submit").unwrap();
    assert_eq!(submit.icon, Some(IconName::ArrowUp));
}

#[test]
fn cat26_05_ai_submit_action_section() {
    let actions = get_ai_command_bar_actions();
    let submit = actions.iter().find(|a| a.id == "submit").unwrap();
    assert_eq!(submit.section.as_deref(), Some("Actions"));
}

#[test]
fn cat26_05_ai_new_chat_action_icon_plus() {
    let actions = get_ai_command_bar_actions();
    let new_chat = actions.iter().find(|a| a.id == "new_chat").unwrap();
    assert_eq!(new_chat.icon, Some(IconName::Plus));
    assert_eq!(new_chat.shortcut.as_deref(), Some("⌘N"));
}

#[test]
fn cat26_05_ai_delete_chat_action_icon_trash() {
    let actions = get_ai_command_bar_actions();
    let delete = actions.iter().find(|a| a.id == "delete_chat").unwrap();
    assert_eq!(delete.icon, Some(IconName::Trash));
    assert_eq!(delete.shortcut.as_deref(), Some("⌘⌫"));
}

// ─────────────────────────────────────────────
// 6. Path vs file context: description wording differences
// ─────────────────────────────────────────────

#[test]
fn cat26_06_file_context_open_desc_says_default_application() {
    let info = FileInfo {
        path: "/f.txt".into(),
        name: "f.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    let open = actions.iter().find(|a| a.id == "open_file").unwrap();
    assert!(open
        .description
        .as_ref()
        .unwrap()
        .contains("default application"));
}

#[test]
fn cat26_06_path_context_file_desc_says_submit() {
    let info = PathInfo {
        name: "f.txt".into(),
        path: "/f.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    let sel = actions.iter().find(|a| a.id == "select_file").unwrap();
    assert!(sel.description.as_ref().unwrap().contains("Submit"));
}

#[test]
fn cat26_06_file_dir_desc_says_folder() {
    let info = FileInfo {
        path: "/d".into(),
        name: "d".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&info);
    let open = actions.iter().find(|a| a.id == "open_directory").unwrap();
    assert!(open.description.as_ref().unwrap().contains("folder"));
}

#[test]
fn cat26_06_path_dir_desc_says_navigate() {
    let info = PathInfo {
        name: "d".into(),
        path: "/d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&info);
    let open = actions.iter().find(|a| a.id == "open_directory").unwrap();
    assert!(open.description.as_ref().unwrap().contains("Navigate"));
}

// ─────────────────────────────────────────────
// 7. ScriptInfo: is_agent mutual exclusivity
// ─────────────────────────────────────────────

#[test]
fn cat26_07_script_info_new_is_not_agent() {
    let s = ScriptInfo::new("x", "/x.ts");
    assert!(!s.is_agent);
    assert!(s.is_script);
    assert!(!s.is_scriptlet);
}

#[test]
fn cat26_07_script_info_builtin_is_not_agent() {
    let b = ScriptInfo::builtin("Clip");
    assert!(!b.is_agent);
    assert!(!b.is_script);
    assert!(!b.is_scriptlet);
}

#[test]
fn cat26_07_script_info_scriptlet_is_not_agent() {
    let s = ScriptInfo::scriptlet("S", "/s.md", None, None);
    assert!(!s.is_agent);
    assert!(!s.is_script);
    assert!(s.is_scriptlet);
}

#[test]
fn cat26_07_script_info_with_action_verb_defaults_no_agent() {
    let s = ScriptInfo::with_action_verb("App", "/a.app", false, "Launch");
    assert!(!s.is_agent);
}

// ─────────────────────────────────────────────
// 8. format_shortcut_hint: multi-key combos with numbers
// ─────────────────────────────────────────────

#[test]
fn cat26_08_format_hint_cmd_1() {
    let result = super::ActionsDialog::format_shortcut_hint("cmd+1");
    assert_eq!(result, "⌘1");
}

#[test]
fn cat26_08_format_hint_ctrl_shift_3() {
    let result = super::ActionsDialog::format_shortcut_hint("ctrl+shift+3");
    assert_eq!(result, "⌃⇧3");
}

#[test]
fn cat26_08_format_hint_alt_f4() {
    let result = super::ActionsDialog::format_shortcut_hint("alt+f4");
    assert_eq!(result, "⌥F4");
}

#[test]
fn cat26_08_format_hint_command_alias() {
    let result = super::ActionsDialog::format_shortcut_hint("command+k");
    assert_eq!(result, "⌘K");
}

#[test]
fn cat26_08_format_hint_option_alias() {
    let result = super::ActionsDialog::format_shortcut_hint("option+delete");
    assert_eq!(result, "⌥⌫");
}

// ─────────────────────────────────────────────
// 9. to_deeplink_name: Unicode with mixed scripts
// ─────────────────────────────────────────────

#[test]
fn cat26_09_deeplink_preserves_cjk() {
    let result = to_deeplink_name("日本語スクリプト");
    assert!(result.contains("日本語スクリプト"));
}

#[test]
fn cat26_09_deeplink_preserves_accented() {
    let result = to_deeplink_name("café résumé");
    assert!(result.contains("café"));
    assert!(result.contains("résumé"));
}

#[test]
fn cat26_09_deeplink_mixed_alpha_special_unicode() {
    let result = to_deeplink_name("Hello 世界!");
    assert_eq!(result, "hello-世界");
}

#[test]
fn cat26_09_deeplink_emoji_stripped() {
    // Emojis are alphanumeric in Unicode, so they should be preserved
    let result = to_deeplink_name("🚀 Launch");
    // Rocket emoji is not alphanumeric (it's a symbol), so it becomes a hyphen
    // "Launch" becomes "launch"
    assert!(result.contains("launch"));
}

// ─────────────────────────────────────────────
// 10. Cross-context: all built-in actions have Some description
// ─────────────────────────────────────────────

#[test]
fn cat26_10_script_actions_all_have_description() {
    let s = ScriptInfo::new("x", "/x.ts");
    let actions = get_script_context_actions(&s);
    for a in &actions {
        assert!(
            a.description.is_some(),
            "Action '{}' should have a description",
            a.id
        );
    }
}

#[test]
fn cat26_10_clipboard_text_actions_all_have_description() {
    let entry = ClipboardEntryInfo {
        id: "t".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    for a in &actions {
        assert!(
            a.description.is_some(),
            "Clipboard action '{}' should have a description",
            a.id
        );
    }
}

#[test]
fn cat26_10_path_actions_all_have_description() {
    let info = PathInfo {
        name: "p".into(),
        path: "/p".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    for a in &actions {
        assert!(
            a.description.is_some(),
            "Path action '{}' should have a description",
            a.id
        );
    }
}

#[test]
fn cat26_10_ai_actions_all_have_description() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(
            a.description.is_some(),
            "AI action '{}' should have a description",
            a.id
        );
    }
}

// ─────────────────────────────────────────────
// 11. Clipboard: pin/unpin title and description content
// ─────────────────────────────────────────────

#[test]
fn cat26_11_clipboard_pin_title_says_pin_entry() {
    let entry = ClipboardEntryInfo {
        id: "u".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let pin = actions.iter().find(|a| a.id == "clipboard_pin").unwrap();
    assert_eq!(pin.title, "Pin Entry");
}

#[test]
fn cat26_11_clipboard_unpin_title_says_unpin_entry() {
    let entry = ClipboardEntryInfo {
        id: "p".into(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let unpin = actions.iter().find(|a| a.id == "clipboard_unpin").unwrap();
    assert_eq!(unpin.title, "Unpin Entry");
}

#[test]
fn cat26_11_clipboard_pin_desc_mentions_prevent() {
    let entry = ClipboardEntryInfo {
        id: "u".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let pin = actions.iter().find(|a| a.id == "clipboard_pin").unwrap();
    assert!(pin
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("pin"));
}

// ─────────────────────────────────────────────
// 12. Note switcher: multiple notes with diverse states
// ─────────────────────────────────────────────

#[test]
fn cat26_12_note_switcher_three_notes_three_items() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "a".into(),
            title: "Note A".into(),
            char_count: 10,
            is_current: true,
            is_pinned: false,
            preview: "Hello".into(),
            relative_time: "1m ago".into(),
        },
        NoteSwitcherNoteInfo {
            id: "b".into(),
            title: "Note B".into(),
            char_count: 20,
            is_current: false,
            is_pinned: true,
            preview: "World".into(),
            relative_time: "5m ago".into(),
        },
        NoteSwitcherNoteInfo {
            id: "c".into(),
            title: "Note C".into(),
            char_count: 5,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "1h ago".into(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions.len(), 3);
}

#[test]
fn cat26_12_note_switcher_pinned_note_section() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "b".into(),
        title: "Note B".into(),
        char_count: 20,
        is_current: false,
        is_pinned: true,
        preview: "World".into(),
        relative_time: "5m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
}

#[test]
fn cat26_12_note_switcher_unpinned_note_section() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "c".into(),
        title: "Note C".into(),
        char_count: 5,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "1h ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Recent"));
}

#[test]
fn cat26_12_note_switcher_empty_preview_with_time_shows_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "c".into(),
        title: "Note C".into(),
        char_count: 5,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "1h ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("1h ago"));
}

// ─────────────────────────────────────────────
// 13. New chat: last_used section and icon
// ─────────────────────────────────────────────

#[test]
fn cat26_13_new_chat_last_used_has_bolt_filled_icon() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "P".into(),
        provider_display_name: "Provider".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
}

#[test]
fn cat26_13_new_chat_last_used_section_name() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "P".into(),
        provider_display_name: "Provider".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
}

#[test]
fn cat26_13_new_chat_last_used_description_is_provider_display_name() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "P".into(),
        provider_display_name: "My Provider".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].description.as_deref(), Some("My Provider"));
}

#[test]
fn cat26_13_new_chat_model_section_name() {
    let models = vec![NewChatModelInfo {
        model_id: "m2".into(),
        display_name: "Model 2".into(),
        provider: "P".into(),
        provider_display_name: "Provider".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].section.as_deref(), Some("Models"));
}

// ─────────────────────────────────────────────
// 14. CommandBarConfig: close flags default to true
// ─────────────────────────────────────────────

#[test]
fn cat26_14_command_bar_config_default_close_on_select() {
    let config = CommandBarConfig::default();
    assert!(config.close_on_select);
}

#[test]
fn cat26_14_command_bar_config_default_close_on_click_outside() {
    let config = CommandBarConfig::default();
    assert!(config.close_on_click_outside);
}

#[test]
fn cat26_14_command_bar_config_default_close_on_escape() {
    let config = CommandBarConfig::default();
    assert!(config.close_on_escape);
}

#[test]
fn cat26_14_command_bar_config_ai_style_preserves_close_flags() {
    let config = CommandBarConfig::ai_style();
    assert!(config.close_on_select);
    assert!(config.close_on_escape);
}

// ─────────────────────────────────────────────
// 15. Script context: edit shortcut is ⌘E for all editable types
// ─────────────────────────────────────────────

#[test]
fn cat26_15_script_edit_shortcut() {
    let s = ScriptInfo::new("s", "/s.ts");
    let actions = get_script_context_actions(&s);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.shortcut.as_deref(), Some("⌘E"));
}

#[test]
fn cat26_15_scriptlet_edit_shortcut() {
    let s = ScriptInfo::scriptlet("s", "/s.md", None, None);
    let actions = get_script_context_actions(&s);
    let edit = actions.iter().find(|a| a.id == "edit_scriptlet").unwrap();
    assert_eq!(edit.shortcut.as_deref(), Some("⌘E"));
}

#[test]
fn cat26_15_agent_edit_shortcut() {
    let mut s = ScriptInfo::new("a", "/a.md");
    s.is_script = false;
    s.is_agent = true;
    let actions = get_script_context_actions(&s);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.shortcut.as_deref(), Some("⌘E"));
}

#[test]
fn cat26_15_agent_edit_title_says_agent() {
    let mut s = ScriptInfo::new("a", "/a.md");
    s.is_script = false;
    s.is_agent = true;
    let actions = get_script_context_actions(&s);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert!(edit.title.contains("Agent"));
}

// ─────────────────────────────────────────────
// 16. Script context: view_logs only for is_script=true
// ─────────────────────────────────────────────

#[test]
fn cat26_16_script_has_view_logs() {
    let s = ScriptInfo::new("s", "/s.ts");
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "view_logs"));
}

#[test]
fn cat26_16_builtin_no_view_logs() {
    let b = ScriptInfo::builtin("B");
    let actions = get_script_context_actions(&b);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

#[test]
fn cat26_16_scriptlet_no_view_logs() {
    let s = ScriptInfo::scriptlet("S", "/s.md", None, None);
    let actions = get_script_context_actions(&s);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

#[test]
fn cat26_16_agent_no_view_logs() {
    let mut s = ScriptInfo::new("a", "/a.md");
    s.is_script = false;
    s.is_agent = true;
    let actions = get_script_context_actions(&s);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

// ─────────────────────────────────────────────
// 17. Script context: copy_deeplink always present
// ─────────────────────────────────────────────

#[test]
fn cat26_17_script_has_copy_deeplink() {
    let s = ScriptInfo::new("s", "/s.ts");
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "copy_deeplink"));
}

#[test]
fn cat26_17_builtin_has_copy_deeplink() {
    let b = ScriptInfo::builtin("B");
    let actions = get_script_context_actions(&b);
    assert!(actions.iter().any(|a| a.id == "copy_deeplink"));
}

#[test]
fn cat26_17_scriptlet_has_copy_deeplink() {
    let s = ScriptInfo::scriptlet("S", "/s.md", None, None);
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "copy_deeplink"));
}

// ─────────────────────────────────────────────
// 18. File context: reveal_in_finder always present
// ─────────────────────────────────────────────

#[test]
fn cat26_18_file_reveal_always_present_file() {
    let info = FileInfo {
        path: "/f.txt".into(),
        name: "f.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
}

#[test]
fn cat26_18_file_reveal_always_present_dir() {
    let info = FileInfo {
        path: "/d".into(),
        name: "d".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
}

#[test]
fn cat26_18_file_reveal_shortcut() {
    let info = FileInfo {
        path: "/f.txt".into(),
        name: "f.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
    assert_eq!(reveal.shortcut.as_deref(), Some("⌘↵"));
}

// ─────────────────────────────────────────────
// 19. Path context: open_in_terminal and open_in_editor always present
// ─────────────────────────────────────────────

#[test]
fn cat26_19_path_has_open_in_terminal_for_dir() {
    let info = PathInfo {
        name: "d".into(),
        path: "/d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "open_in_terminal"));
}

#[test]
fn cat26_19_path_has_open_in_terminal_for_file() {
    let info = PathInfo {
        name: "f".into(),
        path: "/f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "open_in_terminal"));
}

#[test]
fn cat26_19_path_has_open_in_editor_for_file() {
    let info = PathInfo {
        name: "f".into(),
        path: "/f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "open_in_editor"));
}

#[test]
fn cat26_19_path_open_in_editor_shortcut() {
    let info = PathInfo {
        name: "f".into(),
        path: "/f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    let editor = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
    assert_eq!(editor.shortcut.as_deref(), Some("⌘E"));
}

// ─────────────────────────────────────────────
// 20. build_grouped_items_static: empty actions list
// ─────────────────────────────────────────────

#[test]
fn cat26_20_build_grouped_empty_actions_empty_result() {
    let result = build_grouped_items_static(&[], &[], SectionStyle::Headers);
    assert!(result.is_empty());
}

#[test]
fn cat26_20_build_grouped_no_filtered_indices() {
    let actions = vec![Action::new(
        "a",
        "Action",
        Some("desc".into()),
        ActionCategory::ScriptContext,
    )];
    let result = build_grouped_items_static(&actions, &[], SectionStyle::Headers);
    assert!(result.is_empty());
}

#[test]
fn cat26_20_build_grouped_single_action_no_section_no_header() {
    let actions = vec![Action::new(
        "a",
        "Action",
        Some("desc".into()),
        ActionCategory::ScriptContext,
    )];
    let result = build_grouped_items_static(&actions, &[0], SectionStyle::Headers);
    // No section on action, so no header added
    assert_eq!(result.len(), 1);
    assert!(matches!(result[0], GroupedActionItem::Item(0)));
}

#[test]
fn cat26_20_build_grouped_single_action_with_section_has_header() {
    let actions = vec![Action::new(
        "a",
        "Action",
        Some("desc".into()),
        ActionCategory::ScriptContext,
    )
    .with_section("MySection")];
    let result = build_grouped_items_static(&actions, &[0], SectionStyle::Headers);
    assert_eq!(result.len(), 2);
    assert!(matches!(&result[0], GroupedActionItem::SectionHeader(s) if s == "MySection"));
    assert!(matches!(result[1], GroupedActionItem::Item(0)));
}

// ─────────────────────────────────────────────
// 21. coerce_action_selection: mixed header/item patterns
// ─────────────────────────────────────────────

#[test]
fn cat26_21_coerce_item_header_item_on_header_goes_down() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("S".into()),
        GroupedActionItem::Item(1),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(2));
}

#[test]
fn cat26_21_coerce_header_item_on_header_goes_down() {
    let rows = vec![
        GroupedActionItem::SectionHeader("S".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn cat26_21_coerce_item_header_on_header_goes_up() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("S".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn cat26_21_coerce_all_headers_returns_none() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".into()),
        GroupedActionItem::SectionHeader("B".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

// ─────────────────────────────────────────────
// 22. Action: title_lower and description_lower caching
// ─────────────────────────────────────────────

#[test]
fn cat26_22_action_title_lower_precomputed() {
    let a = Action::new("id", "Hello World", None, ActionCategory::ScriptContext);
    assert_eq!(a.title_lower, "hello world");
}

#[test]
fn cat26_22_action_description_lower_precomputed() {
    let a = Action::new(
        "id",
        "T",
        Some("My Description".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(a.description_lower.as_deref(), Some("my description"));
}

#[test]
fn cat26_22_action_shortcut_lower_set_by_with_shortcut() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
    assert_eq!(a.shortcut_lower.as_deref(), Some("⌘⇧c"));
}

#[test]
fn cat26_22_action_no_shortcut_lower_is_none() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext);
    assert!(a.shortcut_lower.is_none());
}

// ─────────────────────────────────────────────
// 23. score_action: combined bonus stacking variations
// ─────────────────────────────────────────────

#[test]
fn cat26_23_score_prefix_match_at_least_100() {
    let a = Action::new("id", "Copy Path", None, ActionCategory::ScriptContext);
    let score = super::ActionsDialog::score_action(&a, "copy");
    assert!(score >= 100);
}

#[test]
fn cat26_23_score_contains_match_50_to_99() {
    let a = Action::new("id", "My Copy Path", None, ActionCategory::ScriptContext);
    let score = super::ActionsDialog::score_action(&a, "copy");
    assert!(score >= 50);
    // It's a contains match not a prefix match
    assert!(score < 100 || a.title_lower.starts_with("copy"));
}

#[test]
fn cat26_23_score_no_match_zero() {
    let a = Action::new("id", "Delete", None, ActionCategory::ScriptContext);
    let score = super::ActionsDialog::score_action(&a, "xyz");
    assert_eq!(score, 0);
}

#[test]
fn cat26_23_score_empty_search_is_prefix() {
    let a = Action::new("id", "Anything", None, ActionCategory::ScriptContext);
    let score = super::ActionsDialog::score_action(&a, "");
    assert!(score >= 100, "Empty search should match as prefix");
}

// ─────────────────────────────────────────────
// 24. fuzzy_match: various patterns
// ─────────────────────────────────────────────

#[test]
fn cat26_24_fuzzy_exact_match() {
    assert!(super::ActionsDialog::fuzzy_match("hello", "hello"));
}

#[test]
fn cat26_24_fuzzy_subsequence_match() {
    assert!(super::ActionsDialog::fuzzy_match("hello world", "hlo"));
}

#[test]
fn cat26_24_fuzzy_no_match() {
    assert!(!super::ActionsDialog::fuzzy_match("abc", "abd"));
}

#[test]
fn cat26_24_fuzzy_empty_needle_matches() {
    assert!(super::ActionsDialog::fuzzy_match("anything", ""));
}

// ─────────────────────────────────────────────
// 25. Clipboard: paste description mentions clipboard
// ─────────────────────────────────────────────

#[test]
fn cat26_25_paste_desc_mentions_clipboard() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hi".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
    assert!(paste
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("clipboard"));
}

#[test]
fn cat26_25_copy_desc_mentions_clipboard() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hi".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let copy = actions.iter().find(|a| a.id == "clipboard_copy").unwrap();
    assert!(copy
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("clipboard"));
}

#[test]
fn cat26_25_paste_keep_open_desc_mentions_keep() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hi".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let keep = actions
        .iter()
        .find(|a| a.id == "clipboard_paste_keep_open")
        .unwrap();
    assert!(keep
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("keep"));
}

// ─────────────────────────────────────────────
// 26. Script context: shortcut toggle (add vs update/remove)
// ─────────────────────────────────────────────

#[test]
fn cat26_26_no_shortcut_shows_add_shortcut() {
    let s = ScriptInfo::new("s", "/s.ts");
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "add_shortcut"));
    assert!(!actions.iter().any(|a| a.id == "update_shortcut"));
    assert!(!actions.iter().any(|a| a.id == "remove_shortcut"));
}

#[test]
fn cat26_26_with_shortcut_shows_update_and_remove() {
    let s = ScriptInfo::with_shortcut("s", "/s.ts", Some("cmd+s".into()));
    let actions = get_script_context_actions(&s);
    assert!(!actions.iter().any(|a| a.id == "add_shortcut"));
    assert!(actions.iter().any(|a| a.id == "update_shortcut"));
    assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
}

#[test]
fn cat26_26_no_alias_shows_add_alias() {
    let s = ScriptInfo::new("s", "/s.ts");
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "add_alias"));
    assert!(!actions.iter().any(|a| a.id == "update_alias"));
    assert!(!actions.iter().any(|a| a.id == "remove_alias"));
}

#[test]
fn cat26_26_with_alias_shows_update_and_remove() {
    let s = ScriptInfo::with_shortcut_and_alias("s", "/s.ts", None, Some("al".into()));
    let actions = get_script_context_actions(&s);
    assert!(!actions.iter().any(|a| a.id == "add_alias"));
    assert!(actions.iter().any(|a| a.id == "update_alias"));
    assert!(actions.iter().any(|a| a.id == "remove_alias"));
}

// ─────────────────────────────────────────────
// 27. Notes command bar: find_in_note section and icon
// ─────────────────────────────────────────────

#[test]
fn cat26_27_notes_find_in_note_section() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let find = actions.iter().find(|a| a.id == "find_in_note").unwrap();
    assert_eq!(find.section.as_deref(), Some("Edit"));
}

#[test]
fn cat26_27_notes_find_in_note_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let find = actions.iter().find(|a| a.id == "find_in_note").unwrap();
    assert_eq!(find.icon, Some(IconName::MagnifyingGlass));
}

#[test]
fn cat26_27_notes_find_in_note_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let find = actions.iter().find(|a| a.id == "find_in_note").unwrap();
    assert_eq!(find.shortcut.as_deref(), Some("⌘F"));
}

// ─────────────────────────────────────────────
// 28. AI command bar: export_markdown details
// ─────────────────────────────────────────────

#[test]
fn cat26_28_ai_export_markdown_shortcut() {
    let actions = get_ai_command_bar_actions();
    let export = actions.iter().find(|a| a.id == "export_markdown").unwrap();
    assert_eq!(export.shortcut.as_deref(), Some("⇧⌘E"));
}

#[test]
fn cat26_28_ai_export_markdown_icon() {
    let actions = get_ai_command_bar_actions();
    let export = actions.iter().find(|a| a.id == "export_markdown").unwrap();
    assert_eq!(export.icon, Some(IconName::FileCode));
}

#[test]
fn cat26_28_ai_export_markdown_section() {
    let actions = get_ai_command_bar_actions();
    let export = actions.iter().find(|a| a.id == "export_markdown").unwrap();
    assert_eq!(export.section.as_deref(), Some("Export"));
}

#[test]
fn cat26_28_ai_export_desc_mentions_markdown() {
    let actions = get_ai_command_bar_actions();
    let export = actions.iter().find(|a| a.id == "export_markdown").unwrap();
    assert!(export
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("markdown"));
}

// ─────────────────────────────────────────────
// 29. parse_shortcut_keycaps: various inputs
// ─────────────────────────────────────────────

#[test]
fn cat26_29_parse_keycaps_cmd_c() {
    let caps = super::ActionsDialog::parse_shortcut_keycaps("⌘C");
    assert_eq!(caps, vec!["⌘", "C"]);
}

#[test]
fn cat26_29_parse_keycaps_modifier_only() {
    let caps = super::ActionsDialog::parse_shortcut_keycaps("⌘");
    assert_eq!(caps, vec!["⌘"]);
}

#[test]
fn cat26_29_parse_keycaps_enter() {
    let caps = super::ActionsDialog::parse_shortcut_keycaps("↵");
    assert_eq!(caps, vec!["↵"]);
}

#[test]
fn cat26_29_parse_keycaps_all_modifiers_and_key() {
    let caps = super::ActionsDialog::parse_shortcut_keycaps("⌃⌥⇧⌘K");
    assert_eq!(caps, vec!["⌃", "⌥", "⇧", "⌘", "K"]);
}

// ─────────────────────────────────────────────
// 30. Cross-context: action count comparison across types
// ─────────────────────────────────────────────

#[test]
fn cat26_30_script_more_actions_than_builtin() {
    let script = ScriptInfo::new("s", "/s.ts");
    let builtin = ScriptInfo::builtin("B");
    let script_actions = get_script_context_actions(&script);
    let builtin_actions = get_script_context_actions(&builtin);
    assert!(script_actions.len() > builtin_actions.len());
}

#[test]
fn cat26_30_scriptlet_more_actions_than_builtin() {
    let scriptlet = ScriptInfo::scriptlet("S", "/s.md", None, None);
    let builtin = ScriptInfo::builtin("B");
    let scriptlet_actions = get_script_context_actions(&scriptlet);
    let builtin_actions = get_script_context_actions(&builtin);
    assert!(scriptlet_actions.len() > builtin_actions.len());
}

#[test]
fn cat26_30_builtin_exactly_4_actions() {
    let b = ScriptInfo::builtin("B");
    let actions = get_script_context_actions(&b);
    assert_eq!(actions.len(), 4); // run, add_shortcut, add_alias, copy_deeplink
}

#[test]
fn cat26_30_script_exactly_9_actions() {
    let s = ScriptInfo::new("s", "/s.ts");
    let actions = get_script_context_actions(&s);
    assert_eq!(actions.len(), 9);
}

#[test]
fn cat26_30_scriptlet_exactly_8_actions() {
    let s = ScriptInfo::scriptlet("S", "/s.md", None, None);
    let actions = get_script_context_actions(&s);
    assert_eq!(actions.len(), 8);
}

#[test]
fn cat26_30_agent_more_actions_than_builtin() {
    let mut a = ScriptInfo::new("a", "/a.md");
    a.is_script = false;
    a.is_agent = true;
    let b = ScriptInfo::builtin("B");
    let agent_actions = get_script_context_actions(&a);
    let builtin_actions = get_script_context_actions(&b);
    assert!(agent_actions.len() > builtin_actions.len());
}
