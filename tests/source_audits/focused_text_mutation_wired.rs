const MUTATION: &str = include_str!("../../src/platform/accessibility/mutation.rs");
const CLIPBOARD: &str = include_str!("../../src/platform/accessibility/clipboard.rs");
const AX: &str = include_str!("../../src/platform/accessibility/ax.rs");
const FOCUSED_TEXT: &str = include_str!("../../src/platform/accessibility/focused_text.rs");

#[test]
fn mutation_api_exposes_replace_append_copy_boundaries() {
    assert!(MUTATION.contains("replace_focused_text"));
    assert!(MUTATION.contains("append_focused_text"));
    assert!(MUTATION.contains("copy_text_output"));
    assert!(MUTATION.contains("TextMutationOptions"));
}

#[test]
fn clipboard_helper_exposes_snapshot_and_change_count() {
    assert!(CLIPBOARD.contains("PasteboardSnapshot"));
    assert!(CLIPBOARD.contains("PasteboardItemSnapshot"));
    assert!(CLIPBOARD.contains("PasteboardRepresentation"));
    assert!(CLIPBOARD.contains("pasteboardItems"));
    assert!(CLIPBOARD.contains("dataForType"));
    assert!(CLIPBOARD.contains("write_plain_text_to_pasteboard"));
    assert!(CLIPBOARD.contains("general_pasteboard_change_count"));
}

#[test]
fn clipboard_paste_fallback_restores_only_when_change_count_still_matches() {
    assert!(CLIPBOARD.contains("paste_plain_text_preserving_clipboard"));
    assert!(CLIPBOARD.contains("capture_general_pasteboard_snapshot()"));
    assert!(CLIPBOARD.contains("write_plain_text_to_pasteboard(text)"));
    assert!(CLIPBOARD.contains("simulate_paste_with_cg()"));
    assert!(CLIPBOARD.contains("current_change_count == temporary_change_count"));
    assert!(CLIPBOARD.contains("restore_general_pasteboard_snapshot(&snapshot)"));
    assert!(CLIPBOARD.contains("Clipboard changed during focused-text Agent Chat paste fallback"));
}

#[test]
fn capture_registers_short_lived_ax_session_for_later_mutation() {
    assert!(AX.contains("register_focused_text_session"));
    assert!(AX.contains("FOCUSED_TEXT_SESSION_TTL_MS"));
    assert!(AX.contains("CFRetain"));
    assert!(AX.contains("prune_stale_sessions_locked"));
    assert!(AX.contains("app_process_id"));
    assert!(FOCUSED_TEXT.contains("register_focused_text_session("));
    assert!(FOCUSED_TEXT.contains("app.process_id"));
    assert!(
        !FOCUSED_TEXT.contains("AXUIElementRef"),
        "focused_text DTO layer must not expose raw AX handles"
    );
}

#[test]
fn replace_and_append_use_registered_ax_value_mutation_with_verification() {
    assert!(MUTATION.contains("replace_registered_focused_text"));
    assert!(MUTATION.contains("append_registered_focused_text"));
    assert!(AX.contains("AXUIElementSetAttributeValue"));
    assert!(AX.contains("set_whole_text_direct"));
    assert!(AX.contains("set_selected_text_range"));
    assert!(AX.contains("verify_whole_text"));
    assert!(AX.contains("whole_text(element)"));
}

#[test]
fn replace_and_append_have_clipboard_safe_fallback_after_direct_ax_fails() {
    assert!(AX.contains("paste_replace_fallback"));
    assert!(AX.contains("paste_append_fallback"));
    assert!(AX.contains("refocus_registered_target_for_paste(target)?"));
    assert!(AX.contains("activate_application_for_pid"));
    assert!(AX.contains("set_focused_ui_element_for_app"));
    assert!(AX.contains("verify_registered_target_is_focused_for_paste"));
    assert!(AX.contains("set_whole_text_direct(target.element, text).is_err()"));
    assert!(AX.contains("set_whole_text_direct(target.element, &appended).is_err()"));
    assert!(AX.contains("paste_plain_text_preserving_clipboard(text)"));
    assert!(AX.contains("paste_plain_text_preserving_clipboard(&output)"));
    assert!(AX.contains("paste_plain_text_preserving_clipboard(appended)"));
    assert!(
        AX.contains("location: 0") && AX.contains("current.encode_utf16().count()"),
        "fallback replace/select-all append must select the full focused field"
    );
}

#[test]
fn mutation_paths_reject_missing_or_stale_sessions_before_writing() {
    let registry_lookup = AX
        .find("fn registered_target")
        .expect("registered_target helper must exist");
    let write_call = AX
        .find("fn set_whole_text_direct")
        .expect("set_whole_text_direct helper must exist");
    let registry_body = &AX[registry_lookup..write_call];

    assert!(registry_body.contains("FocusedTextError::StaleSession"));
    assert!(registry_body.contains("validate_mutation_session"));
    assert!(
        registry_lookup < write_call,
        "session validation helper must be declared before direct write helper"
    );
}
