
#[test]
fn path_context_dir_has_open_directory() {
    let pi = PathInfo::new("src", "/project/src", true);
    let actions = get_path_context_actions(&pi);
    assert!(actions.iter().any(|a| a.id == "open_directory"));
}

// =========== 11. Script: run_script title includes verb and quoted name ===========

#[test]
fn script_run_title_default_verb() {
    let s = ScriptInfo::new("my-script", "/path/my-script.ts");
    let actions = get_script_context_actions(&s);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert_eq!(run.title, "Run \"my-script\"");
}

#[test]
fn script_run_title_custom_verb_launch() {
    let s = ScriptInfo::with_action_verb("Safari", "/app", false, "Launch");
    let actions = get_script_context_actions(&s);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert_eq!(run.title, "Launch \"Safari\"");
}

#[test]
fn script_run_title_custom_verb_switch_to() {
    let s = ScriptInfo::with_action_verb("Doc Window", "win:1", false, "Switch to");
    let actions = get_script_context_actions(&s);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert_eq!(run.title, "Switch to \"Doc Window\"");
}

#[test]
fn script_run_desc_includes_verb() {
    let s = ScriptInfo::with_action_verb("Safari", "/app", false, "Launch");
    let actions = get_script_context_actions(&s);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert!(run.description.as_ref().unwrap().contains("Launch"));
}

// =========== 12. Script: copy_deeplink URL format ===========

#[test]
fn script_copy_deeplink_url_contains_slugified_name() {
    let s = ScriptInfo::new("My Cool Script", "/path/script.ts");
    let actions = get_script_context_actions(&s);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert!(dl
        .description
        .as_ref()
        .unwrap()
        .contains("scriptkit://run/my-cool-script"));
}

#[test]
fn script_copy_deeplink_shortcut() {
    let s = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&s);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert_eq!(dl.shortcut.as_deref(), Some("⌘⇧D"));
}

#[test]
fn script_copy_deeplink_title() {
    let s = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&s);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert_eq!(dl.title, "Copy Deeplink");
}

#[test]
fn builtin_copy_deeplink_url_contains_slugified_name() {
    let s = ScriptInfo::builtin("Clipboard History");
    let actions = get_script_context_actions(&s);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert!(dl
        .description
        .as_ref()
        .unwrap()
        .contains("scriptkit://run/clipboard-history"));
}

// =========== 13. Script: reset_ranking has no shortcut ===========

#[test]
fn script_reset_ranking_no_shortcut() {
    let s = ScriptInfo::new("test", "/p").with_frecency(true, Some("/p".into()));
    let actions = get_script_context_actions(&s);
    let rr = actions.iter().find(|a| a.id == "reset_ranking").unwrap();
    assert!(rr.shortcut.is_none());
}

#[test]
fn script_reset_ranking_title() {
    let s = ScriptInfo::new("test", "/p").with_frecency(true, Some("/p".into()));
    let actions = get_script_context_actions(&s);
    let rr = actions.iter().find(|a| a.id == "reset_ranking").unwrap();
    assert_eq!(rr.title, "Reset Ranking");
}

#[test]
fn script_reset_ranking_desc_mentions_suggested() {
    let s = ScriptInfo::new("test", "/p").with_frecency(true, Some("/p".into()));
    let actions = get_script_context_actions(&s);
    let rr = actions.iter().find(|a| a.id == "reset_ranking").unwrap();
    assert!(rr.description.as_ref().unwrap().contains("Suggested"));
}

#[test]
fn script_reset_ranking_absent_when_not_suggested() {
    let s = ScriptInfo::new("test", "/p");
    let actions = get_script_context_actions(&s);
    assert!(!actions.iter().any(|a| a.id == "reset_ranking"));
}

// =========== 14. Script: add_shortcut vs update_shortcut descriptions ===========

#[test]
fn script_add_shortcut_desc_mentions_set() {
    let s = ScriptInfo::new("test", "/p");
    let actions = get_script_context_actions(&s);
    let a = actions.iter().find(|a| a.id == "add_shortcut").unwrap();
    assert!(a.description.as_ref().unwrap().contains("Set"));
}

#[test]
fn script_update_shortcut_desc_mentions_change() {
    let s = ScriptInfo::with_shortcut("test", "/p", Some("cmd+t".into()));
    let actions = get_script_context_actions(&s);
    let a = actions.iter().find(|a| a.id == "update_shortcut").unwrap();
    assert!(a.description.as_ref().unwrap().contains("Change"));
}

#[test]
fn script_remove_shortcut_desc_mentions_remove() {
    let s = ScriptInfo::with_shortcut("test", "/p", Some("cmd+t".into()));
    let actions = get_script_context_actions(&s);
    let a = actions.iter().find(|a| a.id == "remove_shortcut").unwrap();
    assert!(a.description.as_ref().unwrap().contains("Remove"));
}

#[test]
fn script_add_shortcut_shortcut_is_cmd_shift_k() {
    let s = ScriptInfo::new("test", "/p");
    let actions = get_script_context_actions(&s);
    let a = actions.iter().find(|a| a.id == "add_shortcut").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘⇧K"));
}

// =========== 15. Script: add_alias vs update_alias descriptions ===========

#[test]
fn script_add_alias_desc_mentions_alias() {
    let s = ScriptInfo::new("test", "/p");
    let actions = get_script_context_actions(&s);
    let a = actions.iter().find(|a| a.id == "add_alias").unwrap();
    assert!(a.description.as_ref().unwrap().contains("alias"));
}

#[test]
fn script_update_alias_desc_mentions_change() {
    let s = ScriptInfo::with_shortcut_and_alias("test", "/p", None, Some("t".into()));
    let actions = get_script_context_actions(&s);
    let a = actions.iter().find(|a| a.id == "update_alias").unwrap();
    assert!(a.description.as_ref().unwrap().contains("Change"));
}

#[test]
fn script_remove_alias_shortcut_is_cmd_opt_a() {
    let s = ScriptInfo::with_shortcut_and_alias("test", "/p", None, Some("t".into()));
    let actions = get_script_context_actions(&s);
    let a = actions.iter().find(|a| a.id == "remove_alias").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘⌥A"));
}

#[test]
fn script_add_alias_shortcut_is_cmd_shift_a() {
    let s = ScriptInfo::new("test", "/p");
    let actions = get_script_context_actions(&s);
    let a = actions.iter().find(|a| a.id == "add_alias").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘⇧A"));
}

// =========== 16. AI bar: paste_image details ===========

#[test]
fn ai_bar_paste_image_shortcut() {
    let actions = get_ai_command_bar_actions();
    let pi = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert_eq!(pi.shortcut.as_deref(), Some("⌘V"));
}

#[test]
fn ai_bar_paste_image_icon() {
    let actions = get_ai_command_bar_actions();
    let pi = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert_eq!(pi.icon, Some(IconName::File));
}

#[test]
fn ai_bar_paste_image_section() {
    let actions = get_ai_command_bar_actions();
    let pi = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert_eq!(pi.section.as_deref(), Some("Attachments"));
}

#[test]
fn ai_bar_paste_image_desc_mentions_clipboard() {
    let actions = get_ai_command_bar_actions();
    let pi = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert!(pi.description.as_ref().unwrap().contains("clipboard"));
}

// =========== 17. AI bar: toggle_shortcuts_help details ===========

#[test]
fn ai_bar_toggle_shortcuts_help_shortcut() {
    let actions = get_ai_command_bar_actions();
    let tsh = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(tsh.shortcut.as_deref(), Some("⌘/"));
}

#[test]
fn ai_bar_toggle_shortcuts_help_icon() {
    let actions = get_ai_command_bar_actions();
    let tsh = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(tsh.icon, Some(IconName::Star));
}

#[test]
fn ai_bar_toggle_shortcuts_help_section() {
    let actions = get_ai_command_bar_actions();
    let tsh = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(tsh.section.as_deref(), Some("Help"));
}

#[test]
fn ai_bar_toggle_shortcuts_help_title() {
    let actions = get_ai_command_bar_actions();
    let tsh = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(tsh.title, "Keyboard Shortcuts");
}

// =========== 18. AI bar: change_model details ===========

#[test]
fn ai_bar_change_model_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let cm = actions.iter().find(|a| a.id == "change_model").unwrap();
    assert!(cm.shortcut.is_none());
}

#[test]
fn ai_bar_change_model_icon() {
    let actions = get_ai_command_bar_actions();
    let cm = actions.iter().find(|a| a.id == "change_model").unwrap();
    assert_eq!(cm.icon, Some(IconName::Settings));
}

#[test]
fn ai_bar_change_model_section() {
    let actions = get_ai_command_bar_actions();
    let cm = actions.iter().find(|a| a.id == "change_model").unwrap();
    assert_eq!(cm.section.as_deref(), Some("Settings"));
}

#[test]
fn ai_bar_change_model_desc_mentions_model() {
    let actions = get_ai_command_bar_actions();
    let cm = actions.iter().find(|a| a.id == "change_model").unwrap();
    assert!(cm.description.as_ref().unwrap().contains("model"));
}

// =========== 19. AI bar: unique action IDs ===========

#[test]
fn ai_bar_all_ids_unique() {
    let actions = get_ai_command_bar_actions();
    let mut ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let total = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), total);
}

#[test]
fn ai_bar_no_empty_ids() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(!a.id.is_empty());
    }
}

#[test]
fn ai_bar_all_titles_non_empty() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(!a.title.is_empty());
    }
}

#[test]
fn ai_bar_all_have_sections() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(a.section.is_some(), "Action {} should have a section", a.id);
    }
}

// =========== 20. Notes: browse_notes always present ===========

#[test]
fn notes_browse_notes_always_present_no_selection() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "browse_notes"));
}

#[test]
fn notes_browse_notes_always_present_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "browse_notes"));
}

#[test]
fn notes_browse_notes_shortcut_cmd_p() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let bn = actions.iter().find(|a| a.id == "browse_notes").unwrap();
    assert_eq!(bn.shortcut.as_deref(), Some("⌘P"));
}

#[test]
fn notes_browse_notes_icon_folder_open() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let bn = actions.iter().find(|a| a.id == "browse_notes").unwrap();
    assert_eq!(bn.icon, Some(IconName::FolderOpen));
}

// =========== 21. Notes: export details ===========

#[test]
fn notes_export_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ex = actions.iter().find(|a| a.id == "export").unwrap();
    assert_eq!(ex.shortcut.as_deref(), Some("⇧⌘E"));
}

#[test]
fn notes_export_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ex = actions.iter().find(|a| a.id == "export").unwrap();
    assert_eq!(ex.icon, Some(IconName::ArrowRight));
}

#[test]
fn notes_export_section() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ex = actions.iter().find(|a| a.id == "export").unwrap();
    assert_eq!(ex.section.as_deref(), Some("Export"));
}

#[test]
fn notes_export_absent_in_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "export"));
}

// =========== 22. Notes: copy_note_as icon Copy ===========

#[test]
fn notes_copy_note_as_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let cna = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
    assert_eq!(cna.icon, Some(IconName::Copy));
}

#[test]
fn notes_copy_note_as_section() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let cna = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
    assert_eq!(cna.section.as_deref(), Some("Copy"));
}
