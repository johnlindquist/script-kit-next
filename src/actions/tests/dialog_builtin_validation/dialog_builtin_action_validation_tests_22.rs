// --- merged from part_01.rs ---
// Batch 22: Built-in action validation tests
//
// ~140 tests across 30 categories validating built-in dialog actions.

use super::builders::{
    get_ai_command_bar_actions, get_chat_context_actions, get_clipboard_history_context_actions,
    get_file_context_actions, get_new_chat_actions, get_note_switcher_actions,
    get_notes_command_bar_actions, get_path_context_actions, get_script_context_actions,
    get_scriptlet_context_actions_with_custom, ChatModelInfo, ChatPromptInfo, ClipboardEntryInfo,
    NewChatModelInfo, NewChatPresetInfo, NoteSwitcherNoteInfo, NotesInfo,
};
use super::command_bar::CommandBarConfig;
use super::dialog::{
    build_grouped_items_static, coerce_action_selection, ActionsDialog, GroupedActionItem,
};
use super::types::{
    Action, ActionCategory, ActionsDialogConfig, AnchorPosition, ScriptInfo, SearchPosition,
    SectionStyle,
};
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;

// ============================================================
// 1. format_shortcut_hint edge cases: empty, single char, trailing +
// ============================================================

#[test]
fn batch22_format_shortcut_hint_empty_string() {
    let result = ActionsDialog::format_shortcut_hint("");
    assert_eq!(result, "");
}

#[test]
fn batch22_format_shortcut_hint_single_letter() {
    // Single letter without '+' means it is both first and last part
    let result = ActionsDialog::format_shortcut_hint("c");
    assert_eq!(result, "C");
}

#[test]
fn batch22_format_shortcut_hint_return_key() {
    let result = ActionsDialog::format_shortcut_hint("return");
    assert_eq!(result, "↵");
}

#[test]
fn batch22_format_shortcut_hint_mixed_case_modifiers() {
    let result = ActionsDialog::format_shortcut_hint("Cmd+Shift+c");
    assert_eq!(result, "⌘⇧C");
}

#[test]
fn batch22_format_shortcut_hint_opt_alias() {
    let result = ActionsDialog::format_shortcut_hint("opt+s");
    assert_eq!(result, "⌥S");
}

// ============================================================
// 2. parse_shortcut_keycaps: single modifier, all modifiers, mixed
// ============================================================

#[test]
fn batch22_parse_keycaps_single_modifier() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘");
    assert_eq!(caps, vec!["⌘"]);
}

#[test]
fn batch22_parse_keycaps_all_four_modifiers() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘⌃⌥⇧");
    assert_eq!(caps, vec!["⌘", "⌃", "⌥", "⇧"]);
}

#[test]
fn batch22_parse_keycaps_lowercase_letter_uppercased() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘a");
    assert_eq!(caps, vec!["⌘", "A"]);
}

#[test]
fn batch22_parse_keycaps_number_stays() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘1");
    assert_eq!(caps, vec!["⌘", "1"]);
}

#[test]
fn batch22_parse_keycaps_empty_string() {
    let caps = ActionsDialog::parse_shortcut_keycaps("");
    assert!(caps.is_empty());
}

// ============================================================
// 3. score_action: combined bonus from all fields
// ============================================================

#[test]
fn batch22_score_prefix_plus_desc_plus_shortcut() {
    let action = Action::new(
        "id",
        "Edit Script",
        Some("Edit in editor".to_string()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘E");
    let score = ActionsDialog::score_action(&action, "edit");
    // prefix=100 + desc(edit)=15 = 115 (shortcut ⌘e doesn't contain "edit")
    assert!(score >= 115, "Expected >=115, got {}", score);
}

#[test]
fn batch22_score_no_match_zero() {
    let action = Action::new("id", "Copy Path", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "zzzzz");
    assert_eq!(score, 0);
}

#[test]
fn batch22_score_desc_only_match() {
    let action = Action::new(
        "id",
        "Open File",
        Some("Launch the editor".to_string()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "editor");
    // No title match, desc match=15
    assert_eq!(score, 15);
}

#[test]
fn batch22_score_shortcut_only_match() {
    let action =
        Action::new("id", "Run Script", None, ActionCategory::ScriptContext).with_shortcut("↵");
    let score = ActionsDialog::score_action(&action, "↵");
    // shortcut match=10
    assert!(score >= 10, "Expected >=10, got {}", score);
}

#[test]
fn batch22_score_empty_search_is_prefix() {
    let action = Action::new("id", "Anything", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "");
    // Empty string is prefix of everything
    assert!(score >= 100, "Expected >=100, got {}", score);
}

// ============================================================
// 4. fuzzy_match: Unicode, emoji, repeated chars
// ============================================================

#[test]
fn batch22_fuzzy_match_unicode_subsequence() {
    assert!(ActionsDialog::fuzzy_match("café latte", "cfl"));
}

#[test]
fn batch22_fuzzy_match_exact() {
    assert!(ActionsDialog::fuzzy_match("hello", "hello"));
}

#[test]
fn batch22_fuzzy_match_both_empty() {
    assert!(ActionsDialog::fuzzy_match("", ""));
}

#[test]
fn batch22_fuzzy_match_needle_longer() {
    assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
}

#[test]
fn batch22_fuzzy_match_repeated_chars() {
    assert!(ActionsDialog::fuzzy_match("banana", "aaa"));
}

// ============================================================
// 5. to_deeplink_name: emoji stripped, single char, numeric
// ============================================================

#[test]
fn batch22_deeplink_single_char() {
    assert_eq!(super::builders::to_deeplink_name("a"), "a");
}

#[test]
fn batch22_deeplink_all_special_chars_returns_empty() {
    assert_eq!(super::builders::to_deeplink_name("@#$%^&*"), "");
}

#[test]
fn batch22_deeplink_numeric_only() {
    assert_eq!(super::builders::to_deeplink_name("42"), "42");
}

#[test]
fn batch22_deeplink_underscores_to_hyphens() {
    assert_eq!(
        super::builders::to_deeplink_name("hello_world"),
        "hello-world"
    );
}

#[test]
fn batch22_deeplink_empty_string() {
    assert_eq!(super::builders::to_deeplink_name(""), "");
}

// ============================================================
// 6. coerce_action_selection: item at end, headers at beginning
// ============================================================

#[test]
fn batch22_coerce_item_at_end_headers_at_start() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".into()),
        GroupedActionItem::SectionHeader("B".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(2));
}

#[test]
fn batch22_coerce_single_item() {
    let rows = vec![GroupedActionItem::Item(0)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
}

#[test]
fn batch22_coerce_single_header() {
    let rows = vec![GroupedActionItem::SectionHeader("H".into())];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn batch22_coerce_ix_beyond_bounds_clamped() {
    let rows = vec![
        GroupedActionItem::SectionHeader("H".into()),
        GroupedActionItem::Item(0),
    ];
    // ix=999 should be clamped to len-1=1 which is an Item
    assert_eq!(coerce_action_selection(&rows, 999), Some(1));
}

#[test]
fn batch22_coerce_empty_returns_none() {
    let rows: Vec<GroupedActionItem> = vec![];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

// ============================================================
// 7. build_grouped_items_static: section transitions from None to Some
// ============================================================

#[test]
fn batch22_grouped_none_to_some_section() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Sec"),
    ];
    let filtered = vec![0, 1];
    let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // First action has no section → no header; second has section → header + item
    assert_eq!(items.len(), 3); // Item(0), SectionHeader("Sec"), Item(1)
}

#[test]
fn batch22_grouped_rapid_alternation() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("X"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Y"),
        Action::new("c", "C", None, ActionCategory::ScriptContext).with_section("X"),
    ];
    let filtered = vec![0, 1, 2];
    let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // Each section change adds a header: X, item, Y, item, X, item = 6
    assert_eq!(items.len(), 6);
}

#[test]
fn batch22_grouped_separators_no_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("X"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Y"),
    ];
    let filtered = vec![0, 1];
    let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    assert_eq!(items.len(), 2); // No headers, just items
}

#[test]
fn batch22_grouped_none_style_no_headers() {
    let actions =
        vec![Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("X")];
    let filtered = vec![0];
    let items = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
    assert_eq!(items.len(), 1);
}

// ============================================================
// 8. Script context: agent description mentions "agent"
// ============================================================

#[test]
fn batch22_agent_edit_desc_mentions_agent() {
    let mut s = ScriptInfo::new("my-agent", "/p");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert!(edit
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("agent"));
}

#[test]
fn batch22_agent_reveal_desc_mentions_agent() {
    let mut s = ScriptInfo::new("my-agent", "/p");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
    assert!(reveal
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("agent"));
}

#[test]
fn batch22_agent_copy_path_desc_mentions_agent() {
    let mut s = ScriptInfo::new("my-agent", "/p");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert!(cp
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("agent"));
}

#[test]
fn batch22_script_edit_desc_mentions_editor() {
    let s = ScriptInfo::new("my-script", "/p");
    let actions = get_script_context_actions(&s);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert!(edit.description.as_ref().unwrap().contains("$EDITOR"));
}

// ============================================================
// 9. Script context: add/update/remove shortcut shortcuts are consistent
// ============================================================

#[test]
fn batch22_add_shortcut_has_cmd_shift_k() {
    let s = ScriptInfo::new("s", "/p");
    let actions = get_script_context_actions(&s);
    let add = actions.iter().find(|a| a.id == "add_shortcut").unwrap();
    assert_eq!(add.shortcut.as_deref(), Some("⌘⇧K"));
}

#[test]
fn batch22_update_shortcut_has_cmd_shift_k() {
    let s = ScriptInfo::with_shortcut("s", "/p", Some("cmd+x".into()));
    let actions = get_script_context_actions(&s);
    let upd = actions.iter().find(|a| a.id == "update_shortcut").unwrap();
    assert_eq!(upd.shortcut.as_deref(), Some("⌘⇧K"));
}

#[test]
fn batch22_remove_shortcut_has_cmd_opt_k() {
    let s = ScriptInfo::with_shortcut("s", "/p", Some("cmd+x".into()));
    let actions = get_script_context_actions(&s);
    let rem = actions.iter().find(|a| a.id == "remove_shortcut").unwrap();
    assert_eq!(rem.shortcut.as_deref(), Some("⌘⌥K"));
}

#[test]
fn batch22_add_alias_has_cmd_shift_a() {
    let s = ScriptInfo::new("s", "/p");
    let actions = get_script_context_actions(&s);
    let add = actions.iter().find(|a| a.id == "add_alias").unwrap();
    assert_eq!(add.shortcut.as_deref(), Some("⌘⇧A"));
}

#[test]
fn batch22_remove_alias_has_cmd_opt_a() {
    let s = ScriptInfo::with_shortcut_and_alias("s", "/p", None, Some("a".into()));
    let actions = get_script_context_actions(&s);
    let rem = actions.iter().find(|a| a.id == "remove_alias").unwrap();
    assert_eq!(rem.shortcut.as_deref(), Some("⌘⌥A"));
}

// ============================================================
// 10. Clipboard context: text vs image action count difference
// ============================================================

#[test]
fn batch22_clipboard_text_action_count() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    // Text has no OCR, no open_with, no annotate/upload cleanshot
    let text_count = actions.len();
    assert!(
        text_count >= 10,
        "Text should have >=10 actions, got {}",
        text_count
    );
}

#[test]
fn batch22_clipboard_image_has_ocr() {
    let entry = ClipboardEntryInfo {
        id: "2".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: String::new(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_ocr"));
}

// --- merged from part_02.rs ---

#[test]
fn batch22_clipboard_text_no_ocr() {
    let entry = ClipboardEntryInfo {
        id: "3".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_ocr"));
}

#[test]
fn batch22_clipboard_image_more_actions_than_text() {
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
        preview: String::new(),
        image_dimensions: Some((50, 50)),
        frontmost_app_name: None,
    };
    let t = get_clipboard_history_context_actions(&text_entry).len();
    let i = get_clipboard_history_context_actions(&img_entry).len();
    assert!(
        i > t,
        "Image {} should have more actions than text {}",
        i,
        t
    );
}

// ============================================================
// 11. Clipboard context: pin/unpin toggle based on pinned state
// ============================================================

#[test]
fn batch22_clipboard_unpinned_shows_pin() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_pin"));
    assert!(!actions.iter().any(|a| a.id == "clipboard_unpin"));
}

#[test]
fn batch22_clipboard_pinned_shows_unpin() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_unpin"));
    assert!(!actions.iter().any(|a| a.id == "clipboard_pin"));
}

#[test]
fn batch22_clipboard_pin_unpin_same_shortcut() {
    let pinned_entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let unpinned_entry = ClipboardEntryInfo {
        id: "2".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let pa = get_clipboard_history_context_actions(&pinned_entry);
    let ua = get_clipboard_history_context_actions(&unpinned_entry);
    let unpin_shortcut = pa
        .iter()
        .find(|a| a.id == "clipboard_unpin")
        .unwrap()
        .shortcut
        .as_deref();
    let pin_shortcut = ua
        .iter()
        .find(|a| a.id == "clipboard_pin")
        .unwrap()
        .shortcut
        .as_deref();
    assert_eq!(unpin_shortcut, pin_shortcut);
    assert_eq!(pin_shortcut, Some("⇧⌘P"));
}

// ============================================================
// 12. Chat context: model count affects total action count
// ============================================================

#[test]
fn batch22_chat_zero_models_no_flags() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    // Just continue_in_chat
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "continue_in_chat");
}

#[test]
fn batch22_chat_two_models_both_flags() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".into()),
        available_models: vec![
            ChatModelInfo {
                id: "gpt4".into(),
                display_name: "GPT-4".into(),
                provider: "OpenAI".into(),
            },
            ChatModelInfo {
                id: "claude".into(),
                display_name: "Claude".into(),
                provider: "Anthropic".into(),
            },
        ],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    // 2 models + continue + copy_response + clear_conversation = 5
    assert_eq!(actions.len(), 5);
}

#[test]
fn batch22_chat_current_model_checkmark() {
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
    let model_action = actions
        .iter()
        .find(|a| a.id == "select_model_gpt4")
        .unwrap();
    assert!(model_action.title.contains('✓'));
}

#[test]
fn batch22_chat_non_current_model_no_checkmark() {
    let info = ChatPromptInfo {
        current_model: Some("Other".into()),
        available_models: vec![ChatModelInfo {
            id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model_action = actions
        .iter()
        .find(|a| a.id == "select_model_gpt4")
        .unwrap();
    assert!(!model_action.title.contains('✓'));
}

// ============================================================
// 13. AI command bar: every action has an icon
// ============================================================

#[test]
fn batch22_ai_command_bar_all_have_icons() {
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        assert!(
            action.icon.is_some(),
            "AI action '{}' should have an icon",
            action.id
        );
    }
}

#[test]
fn batch22_ai_command_bar_all_have_sections() {
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        assert!(
            action.section.is_some(),
            "AI action '{}' should have a section",
            action.id
        );
    }
}

#[test]
fn batch22_ai_command_bar_total_is_12() {
    let actions = get_ai_command_bar_actions();
    assert_eq!(actions.len(), 12);
}

#[test]
fn batch22_ai_export_markdown_icon_is_filecode() {
    let actions = get_ai_command_bar_actions();
    let export = actions.iter().find(|a| a.id == "export_markdown").unwrap();
    assert_eq!(export.icon, Some(IconName::FileCode));
}

// ============================================================
// 14. Notes command bar: trash mode removes most actions
// ============================================================

#[test]
fn batch22_notes_trash_mode_minimal() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // Trash mode: new_note, browse_notes, enable_auto_sizing = 3
    assert_eq!(actions.len(), 3);
}

#[test]
fn batch22_notes_full_mode_max_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // Full: new+dup+browse+find+format+copy_note_as+copy_deeplink+create_quicklink+export+auto_sizing = 10
    assert_eq!(actions.len(), 10);
}

#[test]
fn batch22_notes_auto_sizing_enabled_removes_one() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    // Same as full minus enable_auto_sizing = 9
    assert_eq!(actions.len(), 9);
}

#[test]
fn batch22_notes_no_selection_no_trash() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note + browse_notes + enable_auto_sizing = 3
    assert_eq!(actions.len(), 3);
}

// ============================================================
// 15. New chat actions: section assignment and ID patterns
// ============================================================

#[test]
fn batch22_new_chat_last_used_section() {
    let last_used = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "OpenAI".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
}

#[test]
fn batch22_new_chat_preset_section() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].section.as_deref(), Some("Presets"));
}

#[test]
fn batch22_new_chat_model_section() {
    let models = vec![NewChatModelInfo {
        model_id: "claude".into(),
        display_name: "Claude".into(),
        provider: "Anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].section.as_deref(), Some("Models"));
}

#[test]
fn batch22_new_chat_id_patterns() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "M1".into(),
        provider: "P".into(),
        provider_display_name: "P".into(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "code".into(),
        name: "Code".into(),
        icon: IconName::Code,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "m2".into(),
        display_name: "M2".into(),
        provider: "P".into(),
        provider_display_name: "P".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &presets, &models);
    assert_eq!(actions[0].id, "last_used_0");
    assert_eq!(actions[1].id, "preset_code");
    assert_eq!(actions[2].id, "model_0");
}

#[test]
fn batch22_new_chat_empty_all_returns_empty() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

// ============================================================
// 16. Note switcher: icon priority hierarchy
// ============================================================

#[test]
fn batch22_note_switcher_pinned_icon_starfilled() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 10,
        is_current: false,
        is_pinned: true,
        preview: "p".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn batch22_note_switcher_current_icon_check() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 10,
        is_current: true,
        is_pinned: false,
        preview: "p".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::Check));
}

#[test]
fn batch22_note_switcher_regular_icon_file() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "p".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::File));
}

#[test]
fn batch22_note_switcher_pinned_trumps_current() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 10,
        is_current: true,
        is_pinned: true,
        preview: "p".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

// ============================================================
// 17. Note switcher: description format (preview+time, char count)
// ============================================================

#[test]
fn batch22_note_switcher_preview_plus_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "Hello world".into(),
        relative_time: "2m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(desc.contains("Hello world"));
    assert!(desc.contains("2m ago"));
    assert!(desc.contains(" · "));
}

// --- merged from part_03.rs ---

#[test]
fn batch22_note_switcher_no_preview_uses_char_count() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
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
fn batch22_note_switcher_one_char_singular() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
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
fn batch22_note_switcher_preview_truncation_at_60() {
    let long_preview = "a".repeat(70);
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 70,
        is_current: false,
        is_pinned: false,
        preview: long_preview,
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(desc.ends_with('…'));
}

// ============================================================
// 18. Note switcher: current bullet prefix
// ============================================================

#[test]
fn batch22_note_switcher_current_has_bullet() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "My Note".into(),
        char_count: 10,
        is_current: true,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(actions[0].title.starts_with("• "));
}

#[test]
fn batch22_note_switcher_non_current_no_bullet() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "My Note".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(!actions[0].title.starts_with("• "));
}

// ============================================================
// 19. Note switcher: section assignment by pin state
// ============================================================

#[test]
fn batch22_note_switcher_pinned_section() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 10,
        is_current: false,
        is_pinned: true,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
}

#[test]
fn batch22_note_switcher_unpinned_section() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Recent"));
}

// ============================================================
// 20. File context: file vs dir primary action IDs
// ============================================================

#[test]
fn batch22_file_context_file_primary_is_open_file() {
    let fi = FileInfo {
        name: "test.txt".into(),
        path: "/p/test.txt".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&fi);
    assert_eq!(actions[0].id, "open_file");
}

#[test]
fn batch22_file_context_dir_primary_is_open_directory() {
    let fi = FileInfo {
        name: "docs".into(),
        path: "/p/docs".into(),
        is_dir: true,
        file_type: FileType::Directory,
    };
    let actions = get_file_context_actions(&fi);
    assert_eq!(actions[0].id, "open_directory");
}

#[test]
fn batch22_file_context_always_has_reveal_in_finder() {
    let fi = FileInfo {
        name: "f".into(),
        path: "/p/f".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&fi);
    assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
}

#[test]
fn batch22_file_context_always_has_copy_path() {
    let fi = FileInfo {
        name: "f".into(),
        path: "/p/f".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&fi);
    assert!(actions.iter().any(|a| a.id == "copy_path"));
}

#[test]
fn batch22_file_context_copy_filename_has_shortcut() {
    let fi = FileInfo {
        name: "f".into(),
        path: "/p/f".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&fi);
    let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert_eq!(cf.shortcut.as_deref(), Some("⌘C"));
}

// ============================================================
// 21. Path context: dir vs file primary action, trash always last
// ============================================================

#[test]
fn batch22_path_dir_primary_is_open_directory() {
    let pi = PathInfo {
        name: "src".into(),
        path: "/src".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    assert_eq!(actions[0].id, "open_directory");
}

#[test]
fn batch22_path_file_primary_is_select_file() {
    let pi = PathInfo {
        name: "f.rs".into(),
        path: "/f.rs".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    assert_eq!(actions[0].id, "select_file");
}

#[test]
fn batch22_path_trash_is_always_last() {
    let pi_dir = PathInfo {
        name: "d".into(),
        path: "/d".into(),
        is_dir: true,
    };
    let pi_file = PathInfo {
        name: "f".into(),
        path: "/f".into(),
        is_dir: false,
    };
    let da = get_path_context_actions(&pi_dir);
    let fa = get_path_context_actions(&pi_file);
    assert_eq!(da.last().unwrap().id, "move_to_trash");
    assert_eq!(fa.last().unwrap().id, "move_to_trash");
}

#[test]
fn batch22_path_trash_desc_dir_says_folder() {
    let pi = PathInfo {
        name: "d".into(),
        path: "/d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert!(trash.description.as_ref().unwrap().contains("folder"));
}

#[test]
fn batch22_path_trash_desc_file_says_file() {
    let pi = PathInfo {
        name: "f".into(),
        path: "/f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert!(trash.description.as_ref().unwrap().contains("file"));
}

// ============================================================
// 22. Path context: copy_filename has no shortcut
// ============================================================

#[test]
fn batch22_path_copy_filename_no_shortcut() {
    let pi = PathInfo {
        name: "f".into(),
        path: "/f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert!(cf.shortcut.is_none());
}

#[test]
fn batch22_path_open_in_terminal_shortcut() {
    let pi = PathInfo {
        name: "f".into(),
        path: "/f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    let ot = actions.iter().find(|a| a.id == "open_in_terminal").unwrap();
    assert_eq!(ot.shortcut.as_deref(), Some("⌘T"));
}

// ============================================================
// 23. CommandBarConfig preset values
// ============================================================

#[test]
fn batch22_command_bar_default_bottom_search() {
    let cfg = CommandBarConfig::default();
    assert_eq!(cfg.dialog_config.search_position, SearchPosition::Bottom);
}

#[test]
fn batch22_command_bar_ai_top_search() {
    let cfg = CommandBarConfig::ai_style();
    assert_eq!(cfg.dialog_config.search_position, SearchPosition::Top);
    assert!(cfg.dialog_config.show_icons);
    assert!(cfg.dialog_config.show_footer);
}

#[test]
fn batch22_command_bar_no_search_hidden() {
    let cfg = CommandBarConfig::no_search();
    assert_eq!(cfg.dialog_config.search_position, SearchPosition::Hidden);
}

#[test]
fn batch22_command_bar_notes_separators() {
    let cfg = CommandBarConfig::notes_style();
    assert_eq!(cfg.dialog_config.section_style, SectionStyle::Separators);
    assert!(cfg.dialog_config.show_icons);
}

#[test]
fn batch22_command_bar_main_menu_no_icons() {
    let cfg = CommandBarConfig::main_menu_style();
    assert!(!cfg.dialog_config.show_icons);
    assert!(!cfg.dialog_config.show_footer);
}

// ============================================================
// 24. Action builder chaining preserves all fields
// ============================================================

#[test]
fn batch22_action_chain_shortcut_icon_section() {
    let action = Action::new(
        "id",
        "Title",
        Some("Desc".into()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘T")
    .with_icon(IconName::Star)
    .with_section("Sec");
    assert_eq!(action.shortcut.as_deref(), Some("⌘T"));
    assert_eq!(action.icon, Some(IconName::Star));
    assert_eq!(action.section.as_deref(), Some("Sec"));
    assert_eq!(action.title, "Title");
    assert_eq!(action.description.as_deref(), Some("Desc"));
}

#[test]
fn batch22_action_with_shortcut_opt_none_preserves() {
    let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
        .with_shortcut("⌘A")
        .with_shortcut_opt(None);
    // with_shortcut_opt(None) should NOT clear existing shortcut
    assert_eq!(action.shortcut.as_deref(), Some("⌘A"));
}

#[test]
fn batch22_action_with_shortcut_opt_some_sets() {
    let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
        .with_shortcut_opt(Some("⌘B".into()));
    assert_eq!(action.shortcut.as_deref(), Some("⌘B"));
}

#[test]
fn batch22_action_defaults_no_icon_no_section() {
    let action = Action::new("id", "T", None, ActionCategory::ScriptContext);
    assert!(action.icon.is_none());
    assert!(action.section.is_none());
    assert!(action.shortcut.is_none());
    assert!(!action.has_action);
    assert!(action.value.is_none());
}

// ============================================================
// 25. Action lowercase caching correctness
// ============================================================

#[test]
fn batch22_action_title_lower_precomputed() {
    let action = Action::new("id", "Hello World", None, ActionCategory::ScriptContext);
    assert_eq!(action.title_lower, "hello world");
}

#[test]
fn batch22_action_description_lower_precomputed() {
    let action = Action::new(
        "id",
        "T",
        Some("Open In Editor".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(action.description_lower.as_deref(), Some("open in editor"));
}

#[test]
fn batch22_action_shortcut_lower_after_with_shortcut() {
    let action = Action::new("id", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
    assert_eq!(action.shortcut_lower.as_deref(), Some("⌘e"));
}

#[test]
fn batch22_action_no_shortcut_lower_is_none() {
    let action = Action::new("id", "T", None, ActionCategory::ScriptContext);
    assert!(action.shortcut_lower.is_none());
}

// ============================================================
// 26. Scriptlet context with custom actions via get_scriptlet_context_actions_with_custom
// ============================================================

#[test]
fn batch22_scriptlet_custom_run_is_first() {
    let script = ScriptInfo::scriptlet("My Script", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn batch22_scriptlet_custom_has_edit_scriptlet() {
    let script = ScriptInfo::scriptlet("My Script", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "edit_scriptlet"));
}

#[test]
fn batch22_scriptlet_custom_has_copy_content() {
    let script = ScriptInfo::scriptlet("My Script", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "copy_content"));
}

#[test]
fn batch22_scriptlet_custom_frecency_adds_reset() {
    let script = ScriptInfo::scriptlet("My Script", "/p.md", None, None)
        .with_frecency(true, Some("/frec".into()));
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    // Reset ranking should be last
    assert_eq!(actions.last().unwrap().id, "reset_ranking");
}

// ============================================================
// 27. Cross-context: all actions have non-empty ID and title
// ============================================================

#[test]
fn batch22_cross_script_non_empty_ids_titles() {
    let s = ScriptInfo::new("s", "/p");
    for a in get_script_context_actions(&s) {
        assert!(!a.id.is_empty(), "Action ID should not be empty");
        assert!(!a.title.is_empty(), "Action title should not be empty");
    }
}

// --- merged from part_04.rs ---

#[test]
fn batch22_cross_clipboard_non_empty_ids_titles() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    for a in get_clipboard_history_context_actions(&entry) {
        assert!(!a.id.is_empty());
        assert!(!a.title.is_empty());
    }
}

#[test]
fn batch22_cross_ai_non_empty_ids_titles() {
    for a in get_ai_command_bar_actions() {
        assert!(!a.id.is_empty());
        assert!(!a.title.is_empty());
    }
}

#[test]
fn batch22_cross_notes_non_empty_ids_titles() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for a in get_notes_command_bar_actions(&info) {
        assert!(!a.id.is_empty());
        assert!(!a.title.is_empty());
    }
}

// ============================================================
// 28. Cross-context: all built-in action IDs are snake_case
// ============================================================

fn is_snake_case(s: &str) -> bool {
    !s.contains(' ') && !s.contains('-') && s == s.to_lowercase()
        || s.starts_with("select_model_") // model IDs may contain mixed case
        || s.starts_with("note_") // note IDs contain UUIDs
        || s.starts_with("last_used_")
        || s.starts_with("preset_")
        || s.starts_with("model_")
        || s.starts_with("scriptlet_action:")
}

#[test]
fn batch22_cross_script_ids_snake_case() {
    let s = ScriptInfo::new("s", "/p");
    for a in get_script_context_actions(&s) {
        assert!(is_snake_case(&a.id), "ID '{}' should be snake_case", a.id);
    }
}

#[test]
fn batch22_cross_ai_ids_snake_case() {
    for a in get_ai_command_bar_actions() {
        assert!(is_snake_case(&a.id), "ID '{}' should be snake_case", a.id);
    }
}

#[test]
fn batch22_cross_clipboard_ids_snake_case() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    for a in get_clipboard_history_context_actions(&entry) {
        assert!(is_snake_case(&a.id), "ID '{}' should be snake_case", a.id);
    }
}

#[test]
fn batch22_cross_path_ids_snake_case() {
    let pi = PathInfo {
        name: "f".into(),
        path: "/f".into(),
        is_dir: false,
    };
    for a in get_path_context_actions(&pi) {
        assert!(is_snake_case(&a.id), "ID '{}' should be snake_case", a.id);
    }
}

// ============================================================
// 29. format_shortcut_hint: function keys and special aliases
// ============================================================

#[test]
fn batch22_format_shortcut_control_alias() {
    let result = ActionsDialog::format_shortcut_hint("control+c");
    assert_eq!(result, "⌃C");
}

#[test]
fn batch22_format_shortcut_meta_alias() {
    let result = ActionsDialog::format_shortcut_hint("meta+v");
    assert_eq!(result, "⌘V");
}

#[test]
fn batch22_format_shortcut_super_alias() {
    let result = ActionsDialog::format_shortcut_hint("super+v");
    assert_eq!(result, "⌘V");
}

#[test]
fn batch22_format_shortcut_option_alias() {
    let result = ActionsDialog::format_shortcut_hint("option+space");
    assert_eq!(result, "⌥␣");
}

#[test]
fn batch22_format_shortcut_esc_alias() {
    let result = ActionsDialog::format_shortcut_hint("esc");
    assert_eq!(result, "⎋");
}

// ============================================================
// 30. ActionsDialogConfig default values
// ============================================================

#[test]
fn batch22_actions_dialog_config_default_search_bottom() {
    let cfg = ActionsDialogConfig::default();
    assert_eq!(cfg.search_position, SearchPosition::Bottom);
}

#[test]
fn batch22_actions_dialog_config_default_section_separators() {
    let cfg = ActionsDialogConfig::default();
    assert_eq!(cfg.section_style, SectionStyle::Separators);
}

#[test]
fn batch22_actions_dialog_config_default_anchor_bottom() {
    let cfg = ActionsDialogConfig::default();
    assert_eq!(cfg.anchor, AnchorPosition::Bottom);
}

#[test]
fn batch22_actions_dialog_config_default_no_icons() {
    let cfg = ActionsDialogConfig::default();
    assert!(!cfg.show_icons);
}

#[test]
fn batch22_actions_dialog_config_default_no_footer() {
    let cfg = ActionsDialogConfig::default();
    assert!(!cfg.show_footer);
}
