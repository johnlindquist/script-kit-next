
#[test]
fn cat28_09_file_copy_path_desc_mentions_clipboard() {
    let file_info = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert!(cp
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("clipboard"));
}

#[test]
fn cat28_09_file_copy_filename_desc_mentions_filename() {
    let file_info = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert!(cf
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("filename"));
}

// =============================================================================
// Category 10: Path context — move_to_trash description dynamic for dir vs file
// =============================================================================

#[test]
fn cat28_10_path_trash_desc_dir_says_folder() {
    let path_info = PathInfo {
        name: "MyDir".into(),
        path: "/test/MyDir".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert!(trash
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("folder"));
}

#[test]
fn cat28_10_path_trash_desc_file_says_file() {
    let path_info = PathInfo {
        name: "doc.txt".into(),
        path: "/test/doc.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert!(trash
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("file"));
}

#[test]
fn cat28_10_path_trash_shortcut() {
    let path_info = PathInfo {
        name: "doc.txt".into(),
        path: "/test/doc.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert_eq!(trash.shortcut.as_deref(), Some("⌘⌫"));
}

#[test]
fn cat28_10_path_trash_title() {
    let path_info = PathInfo {
        name: "doc.txt".into(),
        path: "/test/doc.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert_eq!(trash.title, "Move to Trash");
}

// =============================================================================
// Category 11: Path context — select_file / open_directory title includes quoted name
// =============================================================================

#[test]
fn cat28_11_path_select_file_title_quotes_name() {
    let path_info = PathInfo {
        name: "report.pdf".into(),
        path: "/test/report.pdf".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let sel = actions.iter().find(|a| a.id == "select_file").unwrap();
    assert!(sel.title.contains("\"report.pdf\""));
}

#[test]
fn cat28_11_path_open_dir_title_quotes_name() {
    let path_info = PathInfo {
        name: "Projects".into(),
        path: "/test/Projects".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    let od = actions.iter().find(|a| a.id == "open_directory").unwrap();
    assert!(od.title.contains("\"Projects\""));
}

#[test]
fn cat28_11_path_select_desc_says_submit() {
    let path_info = PathInfo {
        name: "file.txt".into(),
        path: "/test/file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let sel = actions.iter().find(|a| a.id == "select_file").unwrap();
    assert!(sel
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("submit"));
}

#[test]
fn cat28_11_path_open_dir_desc_says_navigate() {
    let path_info = PathInfo {
        name: "Projects".into(),
        path: "/test/Projects".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    let od = actions.iter().find(|a| a.id == "open_directory").unwrap();
    assert!(od
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("navigate"));
}

// =============================================================================
// Category 12: AI command bar — toggle_shortcuts_help details
// =============================================================================

#[test]
fn cat28_12_ai_toggle_shortcuts_shortcut() {
    let actions = get_ai_command_bar_actions();
    let tsh = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(tsh.shortcut.as_deref(), Some("⌘/"));
}

#[test]
fn cat28_12_ai_toggle_shortcuts_icon() {
    let actions = get_ai_command_bar_actions();
    let tsh = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(tsh.icon, Some(IconName::Star));
}

#[test]
fn cat28_12_ai_toggle_shortcuts_section() {
    let actions = get_ai_command_bar_actions();
    let tsh = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(tsh.section.as_deref(), Some("Help"));
}

#[test]
fn cat28_12_ai_toggle_shortcuts_title() {
    let actions = get_ai_command_bar_actions();
    let tsh = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(tsh.title, "Keyboard Shortcuts");
}

// =============================================================================
// Category 13: AI command bar — new_chat details
// =============================================================================

#[test]
fn cat28_13_ai_new_chat_shortcut() {
    let actions = get_ai_command_bar_actions();
    let nc = actions.iter().find(|a| a.id == "new_chat").unwrap();
    assert_eq!(nc.shortcut.as_deref(), Some("⌘N"));
}

#[test]
fn cat28_13_ai_new_chat_icon() {
    let actions = get_ai_command_bar_actions();
    let nc = actions.iter().find(|a| a.id == "new_chat").unwrap();
    assert_eq!(nc.icon, Some(IconName::Plus));
}

#[test]
fn cat28_13_ai_new_chat_section() {
    let actions = get_ai_command_bar_actions();
    let nc = actions.iter().find(|a| a.id == "new_chat").unwrap();
    assert_eq!(nc.section.as_deref(), Some("Actions"));
}

#[test]
fn cat28_13_ai_new_chat_desc_mentions_new() {
    let actions = get_ai_command_bar_actions();
    let nc = actions.iter().find(|a| a.id == "new_chat").unwrap();
    assert!(nc
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("new"));
}

// =============================================================================
// Category 14: AI command bar — delete_chat details
// =============================================================================

#[test]
fn cat28_14_ai_delete_chat_shortcut() {
    let actions = get_ai_command_bar_actions();
    let dc = actions.iter().find(|a| a.id == "delete_chat").unwrap();
    assert_eq!(dc.shortcut.as_deref(), Some("⌘⌫"));
}

#[test]
fn cat28_14_ai_delete_chat_icon() {
    let actions = get_ai_command_bar_actions();
    let dc = actions.iter().find(|a| a.id == "delete_chat").unwrap();
    assert_eq!(dc.icon, Some(IconName::Trash));
}

#[test]
fn cat28_14_ai_delete_chat_section() {
    let actions = get_ai_command_bar_actions();
    let dc = actions.iter().find(|a| a.id == "delete_chat").unwrap();
    assert_eq!(dc.section.as_deref(), Some("Actions"));
}

#[test]
fn cat28_14_ai_delete_chat_desc_mentions_delete() {
    let actions = get_ai_command_bar_actions();
    let dc = actions.iter().find(|a| a.id == "delete_chat").unwrap();
    assert!(dc
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("delete"));
}

// =============================================================================
// Category 15: Chat context — continue_in_chat shortcut
// =============================================================================

#[test]
fn cat28_15_chat_continue_in_chat_shortcut() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let cic = actions.iter().find(|a| a.id == "continue_in_chat").unwrap();
    assert_eq!(cic.shortcut.as_deref(), Some("⌘↵"));
}

#[test]
fn cat28_15_chat_continue_in_chat_desc_mentions_chat() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let cic = actions.iter().find(|a| a.id == "continue_in_chat").unwrap();
    assert!(cic
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("chat"));
}

#[test]
fn cat28_15_chat_continue_always_present() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".into()),
        available_models: vec![ChatModelInfo {
            id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "continue_in_chat"));
}

// =============================================================================
// Category 16: Chat context — clear_conversation details
// =============================================================================

#[test]
fn cat28_16_chat_clear_shortcut() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let clr = actions
        .iter()
        .find(|a| a.id == "clear_conversation")
        .unwrap();
    assert_eq!(clr.shortcut.as_deref(), Some("⌘⌫"));
}

#[test]
fn cat28_16_chat_clear_absent_when_no_messages() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "clear_conversation"));
}

#[test]
fn cat28_16_chat_clear_desc_mentions_clear() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let clr = actions
        .iter()
        .find(|a| a.id == "clear_conversation")
        .unwrap();
    assert!(clr
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("clear"));
}

#[test]
fn cat28_16_chat_copy_response_absent_when_no_response() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "copy_response"));
}

// =============================================================================
// Category 17: Notes — export section icon
// =============================================================================

#[test]
fn cat28_17_notes_export_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let exp = actions.iter().find(|a| a.id == "export").unwrap();
    assert_eq!(exp.icon, Some(IconName::ArrowRight));
}

#[test]
fn cat28_17_notes_export_section() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let exp = actions.iter().find(|a| a.id == "export").unwrap();
    assert_eq!(exp.section.as_deref(), Some("Export"));
}

#[test]
fn cat28_17_notes_export_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let exp = actions.iter().find(|a| a.id == "export").unwrap();
    assert_eq!(exp.shortcut.as_deref(), Some("⇧⌘E"));
}

#[test]
fn cat28_17_notes_export_absent_without_selection() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "export"));
}
