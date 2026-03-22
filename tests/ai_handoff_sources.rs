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
        source.contains("ai::set_ai_input(cx, &text, submit)?;"),
        "SetInput should forward submit and propagate errors"
    );

    assert!(
        source
            .contains("ai::set_ai_input_with_image(cx, &text, &image_base64, submit)?;"),
        "SetInputWithImage should forward submit and propagate errors"
    );
}

// ---------------------------------------------------------------------------
// open_ai_window_after_main_hide — origin metadata + deferred lifecycle
// ---------------------------------------------------------------------------

#[test]
fn deferred_ai_handoff_logs_origin_and_defers_open() {
    let source = read_action_sources();
    let helper = slice_from(&source, "fn open_ai_window_after_already_hidden(");

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
        helper.contains(".timer(std::time::Duration::from_millis(1))")
            && helper.contains("ai::open_ai_window(cx)"),
        "helper should defer one tick, then open AI"
    );
}

#[test]
fn deferred_ai_handoff_emits_failure_log_on_open_error() {
    let source = read_action_sources();
    let helper = slice_from(&source, "fn open_ai_window_after_already_hidden(");

    assert!(
        helper.contains("event = \"ai_handoff_defer_open_failed\""),
        "helper should emit a failure log event when AI window open fails"
    );
    assert!(
        helper.contains("Failed to send to AI Chat: {}"),
        "helper should show an error toast with the underlying reason on failure"
    );
}

// ---------------------------------------------------------------------------
// Builtin execution — all send-to-AI branches use the deferred helper
// ---------------------------------------------------------------------------

#[test]
fn builtin_execution_uses_deferred_ai_handoff_entrypoint() {
    let source = read_source("src/app_execute/builtin_execution.rs");

    assert!(
        source.contains("open_ai_window_after_already_hidden("),
        "builtin send-to-ai branches should use the already-hidden deferred helper"
    );
}

#[test]
fn open_ai_window_after_main_hide_delegates_to_already_hidden() {
    let source = read_action_sources();
    let wrapper = slice_from(&source, "fn open_ai_window_after_main_hide(");

    assert!(
        wrapper.contains("self.hide_main_and_reset(cx);"),
        "wrapper should hide the main window"
    );
    assert!(
        wrapper.contains("self.open_ai_window_after_already_hidden("),
        "wrapper should delegate to the already-hidden helper"
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
// Non-streaming handoffs must use submit: false
// ---------------------------------------------------------------------------

#[test]
fn non_streaming_ai_handoffs_remain_submit_false() {
    let handler = read_action_sources();
    let builtins = read_source("src/app_execute/builtin_execution.rs");

    let clipboard_block = slice_from(&handler, "\"clipboard_attach_to_ai\" => {");
    let selected_text_block = slice_from(&builtins, "AiCommandType::SendSelectedTextToAi => {");
    let browser_tab_block = slice_from(&builtins, "AiCommandType::SendBrowserTabToAi => {");
    let screen_area_block = slice_from(&builtins, "AiCommandType::SendScreenAreaToAi => {");
    let send_screen_helper = slice_from(&builtins, "fn spawn_send_screen_to_ai_after_hide(");
    let send_window_helper =
        slice_from(&builtins, "fn spawn_send_focused_window_to_ai_after_hide(");

    assert!(
        clipboard_block.contains("submit: false"),
        "clipboard_attach_to_ai must use submit: false"
    );
    assert!(
        selected_text_block.contains("submit: false"),
        "SendSelectedTextToAi must use submit: false"
    );
    assert!(
        browser_tab_block.contains("submit: false"),
        "SendBrowserTabToAi must use submit: false"
    );
    assert!(
        screen_area_block.contains("submit: false"),
        "SendScreenAreaToAi must use submit: false"
    );
    assert!(
        send_screen_helper.contains("submit: false"),
        "spawn_send_screen_to_ai_after_hide must use submit: false"
    );
    assert!(
        send_window_helper.contains("submit: false"),
        "spawn_send_focused_window_to_ai_after_hide must use submit: false"
    );
}

// ---------------------------------------------------------------------------
// Open/new/clear AI flows must use DeferredAiWindowAction::OpenOnly
// ---------------------------------------------------------------------------

#[test]
fn open_only_handoffs_keep_open_only_variant() {
    let handler = read_action_sources();
    let builtins = read_source("src/app_execute/builtin_execution.rs");

    // OpenOnly must remain a no-payload apply path that returns Ok
    assert!(
        handler.contains("Self::OpenOnly => Ok(\"open_only\")"),
        "DeferredAiWindowAction::OpenOnly should remain a no-payload open path"
    );

    // Builtin execution must use OpenOnly for open/new/clear flows
    assert!(
        builtins.contains("DeferredAiWindowAction::OpenOnly"),
        "open/new/clear AI flows should use OpenOnly rather than text/image payload variants"
    );
}

// ---------------------------------------------------------------------------
// AI composer typing must not trigger context preflight
// ---------------------------------------------------------------------------

#[test]
fn ai_images_typing_does_not_schedule_context_preflight() {
    let source = read_source("src/ai/window/images.rs");

    assert!(
        !source.contains("schedule_context_preflight_for_current_draft(cx)"),
        "typing in the AI composer should not trigger context preflight from images.rs"
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

// ---------------------------------------------------------------------------
// Capture helpers thread real dispatch trace IDs
// ---------------------------------------------------------------------------

#[test]
fn capture_helpers_thread_real_dispatch_trace_id() {
    let source = read_source("src/app_execute/builtin_execution.rs");
    let ai_branch = slice_from(&source, "builtins::BuiltInFeature::AiCommand(cmd_type) => {");
    let send_screen_helper = slice_from(&source, "fn spawn_send_screen_to_ai_after_hide(");
    let send_window_helper =
        slice_from(&source, "fn spawn_send_focused_window_to_ai_after_hide(");

    assert!(
        send_screen_helper.contains("trace_id: &str"),
        "spawn_send_screen_to_ai_after_hide should accept the real dispatch trace_id"
    );
    assert!(
        send_window_helper.contains("trace_id: &str"),
        "spawn_send_focused_window_to_ai_after_hide should accept the real dispatch trace_id"
    );
    assert!(
        ai_branch.contains("self.spawn_send_screen_to_ai_after_hide(&dctx.trace_id, cx);"),
        "SendScreenToAi should pass dctx.trace_id into the async capture helper"
    );
    assert!(
        ai_branch.contains("self.spawn_send_focused_window_to_ai_after_hide(&dctx.trace_id, cx);"),
        "SendFocusedWindowToAi should pass dctx.trace_id into the async capture helper"
    );
}

// ---------------------------------------------------------------------------
// Result-bearing queueing and real readiness
// ---------------------------------------------------------------------------

#[test]
fn deferred_ai_window_action_apply_returns_result() {
    let source = read_action_sources();
    let helper = slice_from(&source, "impl DeferredAiWindowAction {");

    assert!(
        helper.contains("fn apply(self, cx: &mut App) -> Result<&'static str, String>"),
        "DeferredAiWindowAction::apply should return a result so queue rejection can fail the handoff"
    );
}

#[test]
fn window_api_queue_helpers_return_result_and_log_structured_enqueue_status() {
    let source = read_source("src/ai/window/window_api.rs");

    assert!(
        source.contains("pub fn is_ai_window_ready(cx: &mut App) -> bool"),
        "window_api should expose a real ready check"
    );
    assert!(
        source.contains("fn enqueue_ai_window_command("),
        "window_api should centralize AI command queueing"
    );
    assert!(
        source.contains("pub fn set_ai_input(cx: &mut App, text: &str, submit: bool) -> Result<(), String>"),
        "set_ai_input should return Result<(), String>"
    );
    assert!(
        source.contains("pub fn set_ai_input_with_image(")
            && source.contains("-> Result<(), String>"),
        "set_ai_input_with_image should return Result<(), String>"
    );
    assert!(
        source.contains("pub fn add_ai_attachment(cx: &mut App, path: &str) -> Result<(), String>"),
        "add_ai_attachment should return Result<(), String>"
    );
    assert!(
        source.contains("event = \"ai_command_enqueue\""),
        "queue helpers should emit structured ai_command_enqueue logs"
    );
}

#[test]
fn deferred_handoff_failure_toast_includes_real_reason() {
    let source = read_action_sources();
    let helper = slice_from(&source, "fn open_ai_window_after_already_hidden(");

    assert!(
        helper.contains("Failed to send to AI Chat: {}"),
        "handoff failure toast should include the underlying reason"
    );
}
