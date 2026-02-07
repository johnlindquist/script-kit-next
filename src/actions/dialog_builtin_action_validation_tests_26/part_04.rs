
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
