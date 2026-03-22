//! Source-level contract tests for the deferred AI handoff launcher.
//!
//! These tests lock the `DeferredAiWindowAction` enum shape, the
//! `open_ai_window_after_main_hide` helper lifecycle, and the rule that all
//! user-facing send-to-AI entrypoints go through the deferred helper instead of
//! calling `ai::open_ai_window` / `ai::set_ai_input*` inline.

use script_kit_gpui::test_utils::{read_all_handle_action_sources, read_source};

fn read_action_sources() -> String {
    read_all_handle_action_sources()
}

fn slice_from<'a>(source: &'a str, needle: &str) -> &'a str {
    let idx = source
        .find(needle)
        .unwrap_or_else(|| panic!("expected to find '{needle}'"));
    &source[idx..]
}

// ---------------------------------------------------------------------------
// DeferredAiWindowAction enum — submit flag on text/image variants
// ---------------------------------------------------------------------------

#[test]
fn deferred_ai_handoff_supports_prefill_and_submit_variants() {
    let source = read_action_sources();

    assert!(
        source.contains("SetInput {") && source.contains("submit: bool"),
        "SetInput should carry submit so one helper can handle prefill and send flows"
    );

    assert!(
        source.contains("SetInputWithImage {")
            && source.contains("image_base64: String")
            && source.contains("submit: bool"),
        "SetInputWithImage should carry submit so one helper can handle prefill and send flows"
    );

    assert!(
        source.contains("Self::SetInput { text, submit } => ai::set_ai_input(cx, &text, submit)"),
        "SetInput should forward submit"
    );

    assert!(
        source
            .contains("ai::set_ai_input_with_image(cx, &text, &image_base64, submit);"),
        "SetInputWithImage should forward submit"
    );
}

// ---------------------------------------------------------------------------
// open_ai_window_after_main_hide — origin metadata + deferred lifecycle
// ---------------------------------------------------------------------------

#[test]
fn deferred_ai_handoff_logs_origin_and_defers_open() {
    let source = read_action_sources();
    let helper = slice_from(&source, "fn open_ai_window_after_main_hide(");

    assert!(
        helper.contains("source_action: &str") && helper.contains("trace_id: &str"),
        "helper should capture origin metadata"
    );

    assert!(
        helper.contains("event = \"ai_handoff_defer_open_start\"")
            && helper.contains("event = \"ai_handoff_defer_open_success\"")
            && helper.contains("source_action = %source_action")
            && helper.contains("trace_id = %trace_id"),
        "helper should emit machine-readable start/success logs"
    );

    assert!(
        helper.contains("self.hide_main_and_reset(cx);")
            && helper.contains(".timer(std::time::Duration::from_millis(1))")
            && helper.contains("ai::open_ai_window(cx)"),
        "helper should hide main, defer one tick, then open AI"
    );
}

#[test]
fn deferred_ai_handoff_emits_failure_log_on_open_error() {
    let source = read_action_sources();
    let helper = slice_from(&source, "fn open_ai_window_after_main_hide(");

    assert!(
        helper.contains("event = \"ai_handoff_defer_open_failed\""),
        "helper should emit a failure log event when AI window open fails"
    );
    assert!(
        helper.contains("show_error_toast(\"Failed to open AI window\""),
        "helper should show an error toast on failure"
    );
}

// ---------------------------------------------------------------------------
// Builtin execution — all send-to-AI branches use the deferred helper
// ---------------------------------------------------------------------------

#[test]
fn builtin_execution_uses_deferred_ai_handoff_entrypoint() {
    let source = read_source("src/app_execute/builtin_execution.rs");

    assert!(
        source.contains("open_ai_window_after_main_hide("),
        "builtin send-to-ai branches should use the deferred helper"
    );
}

#[test]
fn builtin_execution_does_not_inline_ai_open_in_chat_arm() {
    let source = read_source("src/app_execute/builtin_execution.rs");

    // The AiChat arm should not directly call ai::open_ai_window
    let ai_chat_section = source
        .find("BuiltInFeature::AiChat")
        .expect("Expected AiChat arm");
    let ai_chat_block = &source[ai_chat_section..ai_chat_section + 600];

    assert!(
        !ai_chat_block.contains("ai::open_ai_window("),
        "AiChat should not directly call ai::open_ai_window — use deferred helper"
    );
}

// ---------------------------------------------------------------------------
// Clipboard action — uses the deferred helper, not inline
// ---------------------------------------------------------------------------

#[test]
fn clipboard_attach_to_ai_uses_deferred_helper() {
    let source = read_action_sources();

    // The clipboard AI attach path should use the deferred helper
    assert!(
        source.contains("clipboard_attach_to_ai")
            || source.contains("clipboard_send_to_ai")
            || source.contains("DeferredAiWindowAction::SetInput"),
        "clipboard AI action should build a DeferredAiWindowAction"
    );
}

// ---------------------------------------------------------------------------
// File action — uses the deferred helper for attach_to_ai
// ---------------------------------------------------------------------------

#[test]
fn file_attach_to_ai_uses_deferred_helper() {
    let source = read_action_sources();
    let attach_section = slice_from(&source, "\"attach_to_ai\"");

    assert!(
        attach_section.contains("open_ai_window_after_main_hide("),
        "attach_to_ai file action should use the deferred AI window helper"
    );
    assert!(
        attach_section.contains("DeferredAiWindowAction::AddAttachment"),
        "attach_to_ai file action should use AddAttachment variant"
    );
}
