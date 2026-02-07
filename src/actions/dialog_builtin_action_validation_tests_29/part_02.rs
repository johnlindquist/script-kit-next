
#[test]
fn cat29_10_notes_new_note_shortcut() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let nn = actions.iter().find(|a| a.id == "new_note").unwrap();
    assert_eq!(nn.shortcut.as_deref(), Some("⌘N"));
}

#[test]
fn cat29_10_notes_new_note_icon() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let nn = actions.iter().find(|a| a.id == "new_note").unwrap();
    assert_eq!(nn.icon, Some(IconName::Plus));
}

// =============================================================================
// Category 11: AI command bar — copy_chat details
// =============================================================================

#[test]
fn cat29_11_ai_copy_chat_shortcut() {
    let actions = get_ai_command_bar_actions();
    let cc = actions.iter().find(|a| a.id == "copy_chat").unwrap();
    assert_eq!(cc.shortcut.as_deref(), Some("⌥⇧⌘C"));
}

#[test]
fn cat29_11_ai_copy_chat_icon() {
    let actions = get_ai_command_bar_actions();
    let cc = actions.iter().find(|a| a.id == "copy_chat").unwrap();
    assert_eq!(cc.icon, Some(IconName::Copy));
}

#[test]
fn cat29_11_ai_copy_chat_section() {
    let actions = get_ai_command_bar_actions();
    let cc = actions.iter().find(|a| a.id == "copy_chat").unwrap();
    assert_eq!(cc.section.as_deref(), Some("Response"));
}

#[test]
fn cat29_11_ai_copy_chat_desc_mentions_conversation() {
    let actions = get_ai_command_bar_actions();
    let cc = actions.iter().find(|a| a.id == "copy_chat").unwrap();
    assert!(cc
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("conversation"));
}

// =============================================================================
// Category 12: AI command bar — copy_last_code details
// =============================================================================

#[test]
fn cat29_12_ai_copy_last_code_shortcut() {
    let actions = get_ai_command_bar_actions();
    let clc = actions.iter().find(|a| a.id == "copy_last_code").unwrap();
    assert_eq!(clc.shortcut.as_deref(), Some("⌥⌘C"));
}

#[test]
fn cat29_12_ai_copy_last_code_icon() {
    let actions = get_ai_command_bar_actions();
    let clc = actions.iter().find(|a| a.id == "copy_last_code").unwrap();
    assert_eq!(clc.icon, Some(IconName::Code));
}

#[test]
fn cat29_12_ai_copy_last_code_section() {
    let actions = get_ai_command_bar_actions();
    let clc = actions.iter().find(|a| a.id == "copy_last_code").unwrap();
    assert_eq!(clc.section.as_deref(), Some("Response"));
}

#[test]
fn cat29_12_ai_copy_last_code_desc_mentions_code() {
    let actions = get_ai_command_bar_actions();
    let clc = actions.iter().find(|a| a.id == "copy_last_code").unwrap();
    assert!(clc
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("code"));
}

// =============================================================================
// Category 13: AI command bar — copy_response in command bar vs chat context
// =============================================================================

#[test]
fn cat29_13_ai_command_bar_copy_response_shortcut() {
    let actions = get_ai_command_bar_actions();
    let cr = actions.iter().find(|a| a.id == "copy_response").unwrap();
    assert_eq!(cr.shortcut.as_deref(), Some("⇧⌘C"));
}

#[test]
fn cat29_13_chat_context_copy_response_shortcut() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let cr = actions.iter().find(|a| a.id == "copy_response").unwrap();
    assert_eq!(cr.shortcut.as_deref(), Some("⌘C"));
}

#[test]
fn cat29_13_ai_vs_chat_copy_response_different_shortcuts() {
    let ai_actions = get_ai_command_bar_actions();
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let chat_actions = get_chat_context_actions(&info);
    let ai_cr = ai_actions.iter().find(|a| a.id == "copy_response").unwrap();
    let chat_cr = chat_actions
        .iter()
        .find(|a| a.id == "copy_response")
        .unwrap();
    assert_ne!(ai_cr.shortcut, chat_cr.shortcut);
}

// =============================================================================
// Category 14: Chat context — model ID format is "select_model_{id}"
// =============================================================================

#[test]
fn cat29_14_chat_model_id_format() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "claude-3-opus".into(),
            display_name: "Claude 3 Opus".into(),
            provider: "Anthropic".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "select_model_claude-3-opus"));
}

#[test]
fn cat29_14_chat_model_title_is_display_name() {
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
    let m = actions
        .iter()
        .find(|a| a.id == "select_model_gpt-4")
        .unwrap();
    assert_eq!(m.title, "GPT-4");
}

#[test]
fn cat29_14_chat_model_description_via_provider() {
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
    let m = actions
        .iter()
        .find(|a| a.id == "select_model_gpt-4")
        .unwrap();
    assert_eq!(m.description.as_deref(), Some("via OpenAI"));
}

// =============================================================================
// Category 15: File context — open_file title format
// =============================================================================

#[test]
fn cat29_15_file_open_title_quotes_name() {
    let fi = FileInfo {
        path: "/test/doc.pdf".into(),
        name: "doc.pdf".into(),
        file_type: FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    let open = actions.iter().find(|a| a.id == "open_file").unwrap();
    assert!(open.title.contains("\"doc.pdf\""));
}

#[test]
fn cat29_15_file_dir_open_title_quotes_name() {
    let fi = FileInfo {
        path: "/test/Documents".into(),
        name: "Documents".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&fi);
    let open = actions.iter().find(|a| a.id == "open_directory").unwrap();
    assert!(open.title.contains("\"Documents\""));
}

#[test]
fn cat29_15_file_open_desc_says_default_application() {
    let fi = FileInfo {
        path: "/test/image.png".into(),
        name: "image.png".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    let open = actions.iter().find(|a| a.id == "open_file").unwrap();
    assert!(open
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("default application"));
}

#[test]
fn cat29_15_file_dir_open_desc_says_folder() {
    let fi = FileInfo {
        path: "/test/Docs".into(),
        name: "Docs".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&fi);
    let open = actions.iter().find(|a| a.id == "open_directory").unwrap();
    assert!(open
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("folder"));
}

// =============================================================================
// Category 16: Path context — select_file vs open_directory description wording
// =============================================================================

#[test]
fn cat29_16_path_select_file_desc_says_submit() {
    let pi = PathInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    let sel = actions.iter().find(|a| a.id == "select_file").unwrap();
    assert!(sel
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("submit"));
}

#[test]
fn cat29_16_path_open_directory_desc_says_navigate() {
    let pi = PathInfo {
        path: "/test/folder".into(),
        name: "folder".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    let od = actions.iter().find(|a| a.id == "open_directory").unwrap();
    assert!(od
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("navigate"));
}

#[test]
fn cat29_16_path_file_has_no_open_directory() {
    let pi = PathInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    assert!(!actions.iter().any(|a| a.id == "open_directory"));
}

#[test]
fn cat29_16_path_dir_has_no_select_file() {
    let pi = PathInfo {
        path: "/test/folder".into(),
        name: "folder".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    assert!(!actions.iter().any(|a| a.id == "select_file"));
}

// =============================================================================
// Category 17: to_deeplink_name — preserves numbers and lowercase letters
// =============================================================================

#[test]
fn cat29_17_deeplink_lowercase_preserved() {
    assert_eq!(to_deeplink_name("hello"), "hello");
}

#[test]
fn cat29_17_deeplink_numbers_preserved() {
    assert_eq!(to_deeplink_name("test123"), "test123");
}

#[test]
fn cat29_17_deeplink_mixed_case_lowered() {
    assert_eq!(to_deeplink_name("HelloWorld"), "helloworld");
}

#[test]
fn cat29_17_deeplink_spaces_to_hyphens() {
    assert_eq!(to_deeplink_name("my script name"), "my-script-name");
}

// =============================================================================
// Category 18: format_shortcut_hint (dialog.rs) — combined modifier+key combos
// =============================================================================

#[test]
fn cat29_18_format_hint_cmd_shift_k() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("cmd+shift+k"),
        "⌘⇧K"
    );
}

#[test]
fn cat29_18_format_hint_ctrl_alt_delete() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("ctrl+alt+delete"),
        "⌃⌥⌫"
    );
}

#[test]
fn cat29_18_format_hint_meta_alias() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("meta+c"),
        "⌘C"
    );
}

#[test]
fn cat29_18_format_hint_option_space() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("option+space"),
        "⌥␣"
    );
}

#[test]
fn cat29_18_format_hint_single_enter() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("enter"),
        "↵"
    );
}

// =============================================================================
// Category 19: parse_shortcut_keycaps — multi-symbol shortcut strings
// =============================================================================

#[test]
fn cat29_19_parse_keycaps_cmd_enter() {
    let keycaps = super::dialog::ActionsDialog::parse_shortcut_keycaps("⌘↵");
    assert_eq!(keycaps, vec!["⌘", "↵"]);
}

#[test]
fn cat29_19_parse_keycaps_all_modifiers_plus_key() {
    let keycaps = super::dialog::ActionsDialog::parse_shortcut_keycaps("⌃⌥⇧⌘K");
    assert_eq!(keycaps, vec!["⌃", "⌥", "⇧", "⌘", "K"]);
}

#[test]
fn cat29_19_parse_keycaps_space_symbol() {
    let keycaps = super::dialog::ActionsDialog::parse_shortcut_keycaps("␣");
    assert_eq!(keycaps, vec!["␣"]);
}

#[test]
fn cat29_19_parse_keycaps_arrows() {
    let keycaps = super::dialog::ActionsDialog::parse_shortcut_keycaps("↑↓←→");
    assert_eq!(keycaps, vec!["↑", "↓", "←", "→"]);
}

// =============================================================================
// Category 20: score_action — description bonus adds to prefix score
// =============================================================================

#[test]
fn cat29_20_score_prefix_plus_desc_bonus() {
    let action = Action::new(
        "edit_script",
        "Edit Script",
        Some("Edit the script in your editor".to_string()),
        ActionCategory::ScriptContext,
    );
    let score = super::dialog::ActionsDialog::score_action(&action, "edit");
    // prefix(100) + desc bonus(15) = 115
    assert!(score >= 115);
}
