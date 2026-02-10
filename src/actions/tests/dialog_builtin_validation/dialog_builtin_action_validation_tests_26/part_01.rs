// Batch 26 – Builtin action validation tests
//
// 30 categories · ~120 tests
// Focus areas:
//   - Scriptlet context with_custom: action ordering with multiple custom actions
//   - Clipboard share and attach_to_ai details
//   - Notes full mode section assignments
//   - Chat context edge: two models same provider
//   - AI command bar: Submit action details
//   - Path vs file context: description wording differences
//   - ScriptInfo constructor: is_agent mutually exclusive with is_script/is_scriptlet
//   - format_shortcut_hint: multi-key combos with numbers
//   - to_deeplink_name: Unicode edge cases
//   - Cross-context: description is always Some for built-in actions

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
