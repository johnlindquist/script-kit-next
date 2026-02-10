
#[test]
fn clipboard_text_no_open_with() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_open_with"));
}

#[test]
fn clipboard_text_no_annotate_cleanshot() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions
        .iter()
        .any(|a| a.id == "clipboard_annotate_cleanshot"));
}

#[test]
fn clipboard_text_no_upload_cleanshot() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_upload_cleanshot"));
}

// =========== 28. Script: scriptlet vs with_custom share common actions ===========

#[test]
fn scriptlet_both_contexts_have_run_script() {
    let s = ScriptInfo::scriptlet("My Script", "/s.md", None, None);
    let script_actions = get_script_context_actions(&s);
    let custom_actions = get_scriptlet_context_actions_with_custom(&s, None);
    assert!(script_actions.iter().any(|a| a.id == "run_script"));
    assert!(custom_actions.iter().any(|a| a.id == "run_script"));
}

#[test]
fn scriptlet_both_contexts_have_copy_content() {
    let s = ScriptInfo::scriptlet("My Script", "/s.md", None, None);
    let script_actions = get_script_context_actions(&s);
    let custom_actions = get_scriptlet_context_actions_with_custom(&s, None);
    assert!(script_actions.iter().any(|a| a.id == "copy_content"));
    assert!(custom_actions.iter().any(|a| a.id == "copy_content"));
}

#[test]
fn scriptlet_both_contexts_have_edit_scriptlet() {
    let s = ScriptInfo::scriptlet("My Script", "/s.md", None, None);
    let script_actions = get_script_context_actions(&s);
    let custom_actions = get_scriptlet_context_actions_with_custom(&s, None);
    assert!(script_actions.iter().any(|a| a.id == "edit_scriptlet"));
    assert!(custom_actions.iter().any(|a| a.id == "edit_scriptlet"));
}

#[test]
fn scriptlet_both_contexts_have_copy_deeplink() {
    let s = ScriptInfo::scriptlet("My Script", "/s.md", None, None);
    let script_actions = get_script_context_actions(&s);
    let custom_actions = get_scriptlet_context_actions_with_custom(&s, None);
    assert!(script_actions.iter().any(|a| a.id == "copy_deeplink"));
    assert!(custom_actions.iter().any(|a| a.id == "copy_deeplink"));
}

// =========== 29. Dialog format_shortcut_hint: arrow key variants ===========

#[test]
fn dialog_format_hint_up() {
    assert_eq!(ActionsDialog::format_shortcut_hint("up"), "↑");
}

#[test]
fn dialog_format_hint_arrowup() {
    assert_eq!(ActionsDialog::format_shortcut_hint("arrowup"), "↑");
}

#[test]
fn dialog_format_hint_down() {
    assert_eq!(ActionsDialog::format_shortcut_hint("down"), "↓");
}

#[test]
fn dialog_format_hint_arrowdown() {
    assert_eq!(ActionsDialog::format_shortcut_hint("arrowdown"), "↓");
}

// =========== 30. Dialog format_shortcut_hint: control and opt aliases ===========

#[test]
fn dialog_format_hint_control() {
    assert_eq!(ActionsDialog::format_shortcut_hint("control+k"), "⌃K");
}

#[test]
fn dialog_format_hint_opt() {
    assert_eq!(ActionsDialog::format_shortcut_hint("opt+k"), "⌥K");
}

#[test]
fn dialog_format_hint_command() {
    assert_eq!(ActionsDialog::format_shortcut_hint("command+k"), "⌘K");
}

#[test]
fn dialog_format_hint_arrowleft() {
    assert_eq!(ActionsDialog::format_shortcut_hint("arrowleft"), "←");
}
