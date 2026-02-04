//! Batch 27 – Builtin action validation tests
//!
//! 30 categories · ~119 tests
//! Focus areas:
//!   - Clipboard: frontmost_app_name dynamic paste title
//!   - Scriptlet context: edit_scriptlet shortcut and desc vs script edit_script
//!   - Script context: agent action count and ordering invariants
//!   - Notes command bar: conditional auto_sizing action presence
//!   - AI command bar: paste_image shortcut and section
//!   - Chat context: zero models, response+messages combo action counts
//!   - New chat: model_idx ID pattern and section assignments
//!   - Note switcher: empty preview+empty time falls back to char count
//!   - File context: copy_filename vs copy_path shortcut difference
//!   - Path context: copy_filename has no shortcut (unlike file context)
//!   - format_shortcut_hint (dialog.rs): intermediate modifier in non-last position
//!   - to_deeplink_name: repeated hyphens from mixed punctuation
//!   - score_action: fuzzy bonus value is 25 (not 50)
//!   - build_grouped_items_static: None section skips header even in Headers mode
//!   - CommandBarConfig: notes_style uses Separators not Headers
//!   - Cross-context: every context's first action has shortcut ↵

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

#[test]
fn cat27_08_note_switcher_singular_char_count() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "T".into(),
        char_count: 1,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("1 char"));
}

#[test]
fn cat27_08_note_switcher_zero_chars() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Empty".into(),
        char_count: 0,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("0 chars"));
}

// ─────────────────────────────────────────────
// 9. File context: copy_filename vs copy_path shortcut
// ─────────────────────────────────────────────

#[test]
fn cat27_09_file_copy_filename_shortcut_is_cmd_c() {
    let file = FileInfo {
        name: "test.txt".into(),
        path: "/tmp/test.txt".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&file);
    let copy_fn = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert_eq!(copy_fn.shortcut.as_deref(), Some("⌘C"));
}

#[test]
fn cat27_09_file_copy_path_shortcut_is_cmd_shift_c() {
    let file = FileInfo {
        name: "test.txt".into(),
        path: "/tmp/test.txt".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&file);
    let copy_p = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert_eq!(copy_p.shortcut.as_deref(), Some("⌘⇧C"));
}

#[test]
fn cat27_09_file_dir_copy_filename_also_cmd_c() {
    let dir = FileInfo {
        name: "mydir".into(),
        path: "/tmp/mydir".into(),
        is_dir: true,
        file_type: FileType::Directory,
    };
    let actions = get_file_context_actions(&dir);
    let copy_fn = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert_eq!(copy_fn.shortcut.as_deref(), Some("⌘C"));
}

// ─────────────────────────────────────────────
// 10. Path context: copy_filename has no shortcut
// ─────────────────────────────────────────────

#[test]
fn cat27_10_path_copy_filename_no_shortcut() {
    let path = PathInfo {
        name: "file.rs".into(),
        path: "/tmp/file.rs".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    let copy_fn = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert!(copy_fn.shortcut.is_none());
}

#[test]
fn cat27_10_path_copy_path_has_shortcut() {
    let path = PathInfo {
        name: "file.rs".into(),
        path: "/tmp/file.rs".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    let copy_p = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert_eq!(copy_p.shortcut.as_deref(), Some("⌘⇧C"));
}

#[test]
fn cat27_10_path_dir_copy_filename_still_no_shortcut() {
    let path = PathInfo {
        name: "src".into(),
        path: "/tmp/src".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path);
    let copy_fn = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert!(copy_fn.shortcut.is_none());
}

// ─────────────────────────────────────────────
// 11. format_shortcut_hint: intermediate modifier handling
// ─────────────────────────────────────────────

#[test]
fn cat27_11_format_hint_cmd_shift_c() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("cmd+shift+c"),
        "⌘⇧C"
    );
}

#[test]
fn cat27_11_format_hint_ctrl_alt_delete() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("ctrl+alt+delete"),
        "⌃⌥⌫"
    );
}

#[test]
fn cat27_11_format_hint_single_key_enter() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("enter"),
        "↵"
    );
}

#[test]
fn cat27_11_format_hint_super_alias() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("super+k"),
        "⌘K"
    );
}

#[test]
fn cat27_11_format_hint_option_space() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("option+space"),
        "⌥␣"
    );
}

// ─────────────────────────────────────────────
// 12. to_deeplink_name: mixed punctuation collapses
// ─────────────────────────────────────────────

#[test]
fn cat27_12_deeplink_mixed_punctuation_collapses() {
    assert_eq!(to_deeplink_name("a...b"), "a-b");
}

#[test]
fn cat27_12_deeplink_parens_and_brackets() {
    assert_eq!(to_deeplink_name("foo (bar) [baz]"), "foo-bar-baz");
}

#[test]
fn cat27_12_deeplink_ampersand_and_at() {
    assert_eq!(to_deeplink_name("copy & paste @ home"), "copy-paste-home");
}

#[test]
fn cat27_12_deeplink_slash_and_backslash() {
    assert_eq!(to_deeplink_name("path/to\\file"), "path-to-file");
}

// ─────────────────────────────────────────────
// 13. score_action: fuzzy bonus value
// ─────────────────────────────────────────────

#[test]
fn cat27_13_score_fuzzy_match_gives_25() {
    // "rn" is a subsequence of "run script" but not prefix or contains
    let action = Action::new(
        "run_script",
        "Run Script",
        None,
        ActionCategory::ScriptContext,
    );
    // "rp" → r...(u)(n)(space)(s)(c)(r)(i)(p) - subsequence r,p
    let score = super::dialog::ActionsDialog::score_action(&action, "rp");
    // fuzzy match gives 25
    assert_eq!(score, 25);
}

#[test]
fn cat27_13_score_prefix_gives_at_least_100() {
    let action = Action::new("edit", "Edit Script", None, ActionCategory::ScriptContext);
    let score = super::dialog::ActionsDialog::score_action(&action, "edit");
    assert!(score >= 100);
}

#[test]
fn cat27_13_score_contains_gives_50() {
    let action = Action::new("test", "Open Editor", None, ActionCategory::ScriptContext);
    let score = super::dialog::ActionsDialog::score_action(&action, "editor");
    // "editor" is contained in "open editor" but not a prefix
    assert!(score >= 50);
}

#[test]
fn cat27_13_score_no_match_gives_0() {
    let action = Action::new("test", "Run Script", None, ActionCategory::ScriptContext);
    let score = super::dialog::ActionsDialog::score_action(&action, "xyz");
    assert_eq!(score, 0);
}

// ─────────────────────────────────────────────
// 14. build_grouped_items_static: None section in Headers mode
// ─────────────────────────────────────────────

#[test]
fn cat27_14_headers_mode_no_section_skips_header() {
    let actions = vec![
        Action::new("a", "Alpha", None, ActionCategory::ScriptContext),
        Action::new("b", "Beta", None, ActionCategory::ScriptContext),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // No sections → no headers, just items
    assert_eq!(grouped.len(), 2);
    assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
    assert!(matches!(grouped[1], GroupedActionItem::Item(1)));
}

#[test]
fn cat27_14_headers_mode_with_section_adds_header() {
    let actions = vec![
        Action::new("a", "Alpha", None, ActionCategory::ScriptContext).with_section("Group A"),
        Action::new("b", "Beta", None, ActionCategory::ScriptContext).with_section("Group A"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // 1 header + 2 items
    assert_eq!(grouped.len(), 3);
    assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(_)));
}

#[test]
fn cat27_14_headers_mode_two_sections_two_headers() {
    let actions = vec![
        Action::new("a", "Alpha", None, ActionCategory::ScriptContext).with_section("X"),
        Action::new("b", "Beta", None, ActionCategory::ScriptContext).with_section("Y"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // 2 headers + 2 items
    assert_eq!(grouped.len(), 4);
    let header_count = grouped
        .iter()
        .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(header_count, 2);
}

#[test]
fn cat27_14_separators_mode_never_adds_headers() {
    let actions = vec![
        Action::new("a", "Alpha", None, ActionCategory::ScriptContext).with_section("X"),
        Action::new("b", "Beta", None, ActionCategory::ScriptContext).with_section("Y"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    // No headers in Separators mode
    assert_eq!(grouped.len(), 2);
}

// ─────────────────────────────────────────────
// 15. CommandBarConfig: notes_style uses Separators
// ─────────────────────────────────────────────

#[test]
fn cat27_15_notes_style_uses_separators() {
    let cfg = CommandBarConfig::notes_style();
    assert_eq!(cfg.dialog_config.section_style, SectionStyle::Separators);
}

#[test]
fn cat27_15_ai_style_uses_headers() {
    let cfg = CommandBarConfig::ai_style();
    assert_eq!(cfg.dialog_config.section_style, SectionStyle::Headers);
}

#[test]
fn cat27_15_main_menu_uses_separators() {
    let cfg = CommandBarConfig::main_menu_style();
    assert_eq!(cfg.dialog_config.section_style, SectionStyle::Separators);
}

#[test]
fn cat27_15_no_search_uses_hidden() {
    let cfg = CommandBarConfig::no_search();
    assert_eq!(cfg.dialog_config.search_position, SearchPosition::Hidden);
}

// ─────────────────────────────────────────────
// 16. Cross-context: first action shortcut is ↵
// ─────────────────────────────────────────────

#[test]
fn cat27_16_script_first_shortcut_is_enter() {
    let script = ScriptInfo::new("test", "/p.ts");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn cat27_16_clipboard_first_shortcut_is_enter() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn cat27_16_path_file_first_shortcut_is_enter() {
    let path = PathInfo {
        name: "f.txt".into(),
        path: "/f.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn cat27_16_file_context_first_shortcut_is_enter() {
    let file = FileInfo {
        name: "a.txt".into(),
        path: "/a.txt".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&file);
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

// ─────────────────────────────────────────────
// 17. Clipboard: image text action difference (image has OCR)
// ─────────────────────────────────────────────

#[test]
fn cat27_17_clipboard_image_has_ocr() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"clipboard_ocr"));
}

#[test]
fn cat27_17_clipboard_text_no_ocr() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(!ids.contains(&"clipboard_ocr"));
}

#[test]
fn cat27_17_clipboard_image_more_actions_than_text() {
    let text_entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let img_entry = ClipboardEntryInfo {
        id: "2".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "".into(),
        image_dimensions: Some((10, 10)),
        frontmost_app_name: None,
    };
    let text_actions = get_clipboard_history_context_actions(&text_entry);
    let img_actions = get_clipboard_history_context_actions(&img_entry);
    assert!(img_actions.len() > text_actions.len());
}

// ─────────────────────────────────────────────
// 18. coerce_action_selection: all items selectable
// ─────────────────────────────────────────────

#[test]
fn cat27_18_coerce_all_items_stays_at_index() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::Item(1),
        GroupedActionItem::Item(2),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(1));
}

#[test]
fn cat27_18_coerce_index_beyond_len_clamps() {
    let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
    // ix=10 → clamped to len-1 = 1
    assert_eq!(coerce_action_selection(&rows, 10), Some(1));
}

#[test]
fn cat27_18_coerce_header_at_0_jumps_to_1() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn cat27_18_coerce_only_headers_returns_none() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".into()),
        GroupedActionItem::SectionHeader("B".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

// ─────────────────────────────────────────────
// 19. Action: with_shortcut sets shortcut_lower
// ─────────────────────────────────────────────

#[test]
fn cat27_19_with_shortcut_sets_shortcut_lower() {
    let action = Action::new("t", "Test", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
    assert_eq!(action.shortcut_lower, Some("⌘e".into()));
}

#[test]
fn cat27_19_no_shortcut_shortcut_lower_is_none() {
    let action = Action::new("t", "Test", None, ActionCategory::ScriptContext);
    assert!(action.shortcut_lower.is_none());
}

#[test]
fn cat27_19_title_lower_is_precomputed() {
    let action = Action::new("t", "Edit Script", None, ActionCategory::ScriptContext);
    assert_eq!(action.title_lower, "edit script");
}

#[test]
fn cat27_19_description_lower_is_precomputed() {
    let action = Action::new(
        "t",
        "Test",
        Some("Open in $EDITOR".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(action.description_lower, Some("open in $editor".into()));
}

// ─────────────────────────────────────────────
// 20. Script context: scriptlet vs script edit action IDs differ
// ─────────────────────────────────────────────

#[test]
fn cat27_20_script_edit_id_is_edit_script() {
    let script = ScriptInfo::new("test", "/p.ts");
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"edit_script"));
    assert!(!ids.contains(&"edit_scriptlet"));
}

#[test]
fn cat27_20_scriptlet_edit_id_is_edit_scriptlet() {
    let script = ScriptInfo::scriptlet("s", "/p.md", None, None);
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"edit_scriptlet"));
    assert!(!ids.contains(&"edit_script"));
}

#[test]
fn cat27_20_agent_edit_id_is_edit_script() {
    let mut script = ScriptInfo::builtin("agent");
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"edit_script"));
}

#[test]
fn cat27_20_agent_edit_title_says_agent() {
    let mut script = ScriptInfo::builtin("agent");
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert!(edit.title.contains("Agent"));
}

// ─────────────────────────────────────────────
// 21. Notes command bar: copy section requires selection+not trash
// ─────────────────────────────────────────────

#[test]
fn cat27_21_notes_copy_section_present_with_selection() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"copy_note_as"));
    assert!(ids.contains(&"copy_deeplink"));
    assert!(ids.contains(&"create_quicklink"));
}

#[test]
fn cat27_21_notes_copy_section_absent_without_selection() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(!ids.contains(&"copy_note_as"));
    assert!(!ids.contains(&"copy_deeplink"));
}

#[test]
fn cat27_21_notes_copy_section_absent_in_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(!ids.contains(&"copy_note_as"));
}

#[test]
fn cat27_21_notes_create_quicklink_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ql = actions.iter().find(|a| a.id == "create_quicklink").unwrap();
    assert_eq!(ql.shortcut.as_deref(), Some("⇧⌘L"));
}

// ─────────────────────────────────────────────
// 22. AI command bar: section counts
// ─────────────────────────────────────────────

#[test]
fn cat27_22_ai_response_section_has_3_actions() {
    let actions = get_ai_command_bar_actions();
    let response_count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .count();
    assert_eq!(response_count, 3);
}

#[test]
fn cat27_22_ai_actions_section_has_4_actions() {
    let actions = get_ai_command_bar_actions();
    let action_count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Actions"))
        .count();
    assert_eq!(action_count, 4);
}

#[test]
fn cat27_22_ai_attachments_section_has_2_actions() {
    let actions = get_ai_command_bar_actions();
    let attach_count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Attachments"))
        .count();
    assert_eq!(attach_count, 2);
}

#[test]
fn cat27_22_ai_total_is_12() {
    let actions = get_ai_command_bar_actions();
    assert_eq!(actions.len(), 12);
}

// ─────────────────────────────────────────────
// 23. parse_shortcut_keycaps: various combos
// ─────────────────────────────────────────────

#[test]
fn cat27_23_parse_keycaps_cmd_e() {
    let caps = super::dialog::ActionsDialog::parse_shortcut_keycaps("⌘E");
    assert_eq!(caps, vec!["⌘", "E"]);
}

#[test]
fn cat27_23_parse_keycaps_all_modifiers_and_key() {
    let caps = super::dialog::ActionsDialog::parse_shortcut_keycaps("⌘⌃⌥⇧A");
    assert_eq!(caps, vec!["⌘", "⌃", "⌥", "⇧", "A"]);
}

#[test]
fn cat27_23_parse_keycaps_enter_alone() {
    let caps = super::dialog::ActionsDialog::parse_shortcut_keycaps("↵");
    assert_eq!(caps, vec!["↵"]);
}

#[test]
fn cat27_23_parse_keycaps_lowercase_uppercased() {
    let caps = super::dialog::ActionsDialog::parse_shortcut_keycaps("⌘c");
    assert_eq!(caps, vec!["⌘", "C"]);
}

// ─────────────────────────────────────────────
// 24. fuzzy_match: various patterns
// ─────────────────────────────────────────────

#[test]
fn cat27_24_fuzzy_match_exact() {
    assert!(super::dialog::ActionsDialog::fuzzy_match("edit", "edit"));
}

#[test]
fn cat27_24_fuzzy_match_subsequence() {
    assert!(super::dialog::ActionsDialog::fuzzy_match(
        "edit script",
        "es"
    ));
}

#[test]
fn cat27_24_fuzzy_match_no_match() {
    assert!(!super::dialog::ActionsDialog::fuzzy_match("edit", "z"));
}

#[test]
fn cat27_24_fuzzy_match_empty_needle() {
    assert!(super::dialog::ActionsDialog::fuzzy_match("anything", ""));
}

#[test]
fn cat27_24_fuzzy_match_needle_longer_fails() {
    assert!(!super::dialog::ActionsDialog::fuzzy_match("ab", "abc"));
}

// ─────────────────────────────────────────────
// 25. Script context: view_logs exclusive to is_script
// ─────────────────────────────────────────────

#[test]
fn cat27_25_script_has_view_logs() {
    let script = ScriptInfo::new("test", "/p.ts");
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"view_logs"));
}

#[test]
fn cat27_25_scriptlet_no_view_logs() {
    let script = ScriptInfo::scriptlet("s", "/p.md", None, None);
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(!ids.contains(&"view_logs"));
}

#[test]
fn cat27_25_builtin_no_view_logs() {
    let script = ScriptInfo::builtin("Clipboard");
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(!ids.contains(&"view_logs"));
}

#[test]
fn cat27_25_agent_no_view_logs() {
    let mut script = ScriptInfo::builtin("agent");
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(!ids.contains(&"view_logs"));
}

// ─────────────────────────────────────────────
// 26. Clipboard: delete_all desc mentions pinned exception
// ─────────────────────────────────────────────

#[test]
fn cat27_26_clipboard_delete_all_desc_mentions_pinned() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let del_all = actions
        .iter()
        .find(|a| a.id == "clipboard_delete_all")
        .unwrap();
    assert!(del_all
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("pinned"));
}

#[test]
fn cat27_26_clipboard_delete_multiple_desc_mentions_filter() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let del_multi = actions
        .iter()
        .find(|a| a.id == "clipboard_delete_multiple")
        .unwrap();
    assert!(del_multi
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("filter"));
}

#[test]
fn cat27_26_clipboard_delete_shortcut_is_ctrl_x() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let del = actions.iter().find(|a| a.id == "clipboard_delete").unwrap();
    assert_eq!(del.shortcut.as_deref(), Some("⌃X"));
}

// ─────────────────────────────────────────────
// 27. Note switcher: preview with time uses separator
// ─────────────────────────────────────────────

#[test]
fn cat27_27_note_switcher_preview_with_time_has_separator() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".into(),
        title: "T".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "Hello world".into(),
        relative_time: "3m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(desc.contains(" · "));
    assert!(desc.contains("Hello world"));
    assert!(desc.contains("3m ago"));
}

#[test]
fn cat27_27_note_switcher_long_preview_truncated() {
    let long_preview = "a".repeat(80);
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".into(),
        title: "T".into(),
        char_count: 80,
        is_current: false,
        is_pinned: false,
        preview: long_preview,
        relative_time: "1h ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(desc.contains("…"));
}

#[test]
fn cat27_27_note_switcher_exactly_60_chars_no_truncation() {
    let exact = "b".repeat(60);
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".into(),
        title: "T".into(),
        char_count: 60,
        is_current: false,
        is_pinned: false,
        preview: exact,
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(!desc.contains("…"));
}

// ─────────────────────────────────────────────
// 28. ScriptInfo: with_frecency builder chain
// ─────────────────────────────────────────────

#[test]
fn cat27_28_with_frecency_sets_is_suggested() {
    let script = ScriptInfo::new("test", "/p.ts").with_frecency(true, Some("/p".into()));
    assert!(script.is_suggested);
    assert_eq!(script.frecency_path, Some("/p".into()));
}

#[test]
fn cat27_28_with_frecency_false_not_suggested() {
    let script = ScriptInfo::new("test", "/p.ts").with_frecency(false, None);
    assert!(!script.is_suggested);
    assert!(script.frecency_path.is_none());
}

#[test]
fn cat27_28_with_frecency_preserves_other_fields() {
    let script = ScriptInfo::new("test", "/p.ts").with_frecency(true, None);
    assert!(script.is_script);
    assert_eq!(script.action_verb, "Run");
    assert_eq!(script.name, "test");
}

// ─────────────────────────────────────────────
// 29. CommandBarConfig: anchor positions
// ─────────────────────────────────────────────

#[test]
fn cat27_29_default_config_anchor_bottom() {
    let cfg = CommandBarConfig::default();
    assert_eq!(cfg.dialog_config.anchor, AnchorPosition::Bottom);
}

#[test]
fn cat27_29_ai_style_anchor_top() {
    let cfg = CommandBarConfig::ai_style();
    assert_eq!(cfg.dialog_config.anchor, AnchorPosition::Top);
}

#[test]
fn cat27_29_main_menu_anchor_bottom() {
    let cfg = CommandBarConfig::main_menu_style();
    assert_eq!(cfg.dialog_config.anchor, AnchorPosition::Bottom);
}

#[test]
fn cat27_29_notes_style_anchor_top() {
    let cfg = CommandBarConfig::notes_style();
    assert_eq!(cfg.dialog_config.anchor, AnchorPosition::Top);
}

// ─────────────────────────────────────────────
// 30. Cross-context: all action IDs are non-empty
// ─────────────────────────────────────────────

#[test]
fn cat27_30_script_action_ids_non_empty() {
    let script = ScriptInfo::new("test", "/p.ts");
    let actions = get_script_context_actions(&script);
    for a in &actions {
        assert!(!a.id.is_empty(), "action ID should not be empty");
    }
}

#[test]
fn cat27_30_clipboard_action_ids_non_empty() {
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
        assert!(!a.id.is_empty());
    }
}

#[test]
fn cat27_30_ai_action_ids_non_empty() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(!a.id.is_empty());
    }
}

#[test]
fn cat27_30_path_action_ids_non_empty() {
    let path = PathInfo {
        name: "f".into(),
        path: "/f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    for a in &actions {
        assert!(!a.id.is_empty());
    }
}

#[test]
fn cat27_30_file_action_ids_non_empty() {
    let file = FileInfo {
        name: "f.txt".into(),
        path: "/f.txt".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&file);
    for a in &actions {
        assert!(!a.id.is_empty());
    }
}

#[test]
fn cat27_30_notes_action_ids_non_empty() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    for a in &actions {
        assert!(!a.id.is_empty());
    }
}
