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
        source.contains("ai::set_ai_input_with_image(cx, &text, &image_base64, submit)?;"),
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

#[test]
fn script_list_tab_fallback_uses_canonical_ai_handoff() {
    let render_source = read_source("src/render_script_list/mod.rs");
    let handler_source = read_action_sources();
    let tab_block = slice_from(&render_source, "key if sk_is_key_tab(key) => {");
    let helper_block = slice_from(
        &handler_source,
        "pub(crate) fn open_ai_chat_from_main_window_query(",
    );

    assert!(
        tab_block.contains("this.open_ai_chat_from_main_window_query(query, cx);"),
        "ScriptList Tab fallback must delegate to a ScriptListApp handoff method"
    );
    assert!(
        !tab_block.contains("ai::open_ai_window(") && !tab_block.contains("ai::set_ai_input("),
        "ScriptList Tab fallback must not open AI inline before hiding the main window"
    );
    assert!(
        helper_block.contains("self.open_ai_window_after_main_hide(")
            && helper_block.contains("DeferredAiWindowAction::SetInput")
            && helper_block.contains("submit: false"),
        "main-window AI query helper must use the canonical hide-then-deferred-open flow"
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
    let send_screen_helper = slice_from(&builtins, "fn spawn_send_screen_to_ai_after_hide(");
    let send_window_helper =
        slice_from(&builtins, "fn spawn_send_focused_window_to_ai_after_hide(");
    let send_screen_area_helper =
        slice_from(&builtins, "fn spawn_send_screen_area_to_ai_after_hide(");
    let text_capture_helper = slice_from(
        &builtins,
        "fn spawn_capture_text_to_ai_after_already_hidden<",
    );

    assert!(
        clipboard_block.contains("submit: false"),
        "clipboard_attach_to_ai must use submit: false"
    );
    assert!(
        text_capture_helper.contains("submit: false"),
        "selected text/browser tab deferred capture must use submit: false"
    );
    assert!(
        send_screen_area_helper.contains("submit: false"),
        "spawn_send_screen_area_to_ai_after_hide must use submit: false"
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
    let ai_branch = slice_from(
        &source,
        "builtins::BuiltInFeature::AiCommand(cmd_type) => {",
    );
    let send_screen_helper = slice_from(&source, "fn spawn_send_screen_to_ai_after_hide(");
    let send_window_helper = slice_from(&source, "fn spawn_send_focused_window_to_ai_after_hide(");
    let send_screen_area_helper =
        slice_from(&source, "fn spawn_send_screen_area_to_ai_after_hide(");

    assert!(
        send_screen_helper.contains("trace_id: &str"),
        "spawn_send_screen_to_ai_after_hide should accept the real dispatch trace_id"
    );
    assert!(
        send_window_helper.contains("trace_id: &str"),
        "spawn_send_focused_window_to_ai_after_hide should accept the real dispatch trace_id"
    );
    assert!(
        send_screen_area_helper.contains("trace_id: &str"),
        "spawn_send_screen_area_to_ai_after_hide should accept the real dispatch trace_id"
    );
    assert!(
        ai_branch.contains("self.spawn_send_screen_to_ai_after_hide(&dctx.trace_id, cx);"),
        "SendScreenToAi should pass dctx.trace_id into the async capture helper"
    );
    assert!(
        ai_branch.contains("self.spawn_send_focused_window_to_ai_after_hide(&dctx.trace_id, cx);"),
        "SendFocusedWindowToAi should pass dctx.trace_id into the async capture helper"
    );
    assert!(
        ai_branch.contains("self.spawn_send_screen_area_to_ai_after_hide(&dctx.trace_id, cx);"),
        "SendScreenAreaToAi should pass dctx.trace_id into the async capture helper"
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
        source.contains(
            "pub fn set_ai_input(cx: &mut App, text: &str, submit: bool) -> Result<(), String>"
        ),
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

// ---------------------------------------------------------------------------
// Readiness gate — window must be ready before reporting success
// ---------------------------------------------------------------------------

#[test]
fn test_deferred_ai_handoff_checks_window_ready_before_success() {
    let handler = read_action_sources();
    let helper_body = slice_from(&handler, "fn open_ai_window_after_already_hidden(");

    assert!(
        helper_body.contains("ai::is_ai_window_ready"),
        "deferred AI handoff should verify the AI window is truly ready before showing success"
    );
    assert!(
        helper_body.contains("if !ai::is_ai_window_ready(cx)"),
        "deferred AI handoff should gate on is_ai_window_ready before applying the deferred action"
    );
}

// ---------------------------------------------------------------------------
// Failure toast — propagated error, not hardcoded message
// ---------------------------------------------------------------------------

#[test]
fn deferred_ai_handoff_emits_actionable_failure_toast_on_open_error() {
    let source = read_action_sources();
    let helper = slice_from(&source, "fn open_ai_window_after_already_hidden(");

    assert!(
        helper.contains("event = \"ai_handoff_defer_open_failed\""),
        "helper should emit a failure log event when AI handoff fails"
    );
    assert!(
        helper.contains("format!(\"Failed to send to AI Chat: {}\", error)"),
        "helper should surface the propagated handoff error in the toast"
    );
    // Ensure we do NOT use a hardcoded open-window message
    assert!(
        !helper.contains("show_error_toast(\"Failed to open AI window\")"),
        "helper must not use a hardcoded 'Failed to open AI window' toast"
    );
}

// ---------------------------------------------------------------------------
// SendScreenAreaToAi — uses the deferred hide-and-capture pattern
// ---------------------------------------------------------------------------

#[test]
fn send_screen_area_to_ai_uses_deferred_capture_flow() {
    let builtins = read_source("src/app_execute/builtin_execution.rs");

    // The SendScreenAreaToAi arm must call the deferred helper, not inline capture
    let screen_area_arm = slice_from(&builtins, "AiCommandType::SendScreenAreaToAi => {");
    assert!(
        screen_area_arm.contains("self.spawn_send_screen_area_to_ai_after_hide("),
        "SendScreenAreaToAi must delegate to the deferred capture helper"
    );
    assert!(
        !screen_area_arm.contains("platform::capture_screen_area()"),
        "SendScreenAreaToAi must not perform inline synchronous capture"
    );

    // The helper must exist and use the same deferred pattern
    let helper = slice_from(&builtins, "fn spawn_send_screen_area_to_ai_after_hide(");
    assert!(
        helper.contains("platform::defer_hide_main_window(cx);"),
        "screen area capture helper must defer main window hide"
    );
    assert!(
        helper.contains("ai_capture_hide_settle_duration()"),
        "screen area capture helper must wait for hide settle before capturing"
    );
    assert!(
        helper.contains("platform::capture_screen_area()"),
        "screen area capture helper must call capture_screen_area on background executor"
    );
    assert!(
        helper.contains("event = \"ai_capture_scheduled\""),
        "screen area capture helper must log ai_capture_scheduled"
    );
    assert!(
        helper.contains("event = \"ai_capture_completed\""),
        "screen area capture helper must log ai_capture_completed"
    );
    assert!(
        helper.contains("source_action = \"SendScreenAreaToAi\""),
        "screen area capture helper must use SendScreenAreaToAi as source_action"
    );
    assert!(
        helper.contains("open_ai_window_after_already_hidden("),
        "screen area capture helper must use deferred AI window open"
    );
}

// ---------------------------------------------------------------------------
// Continue in chat — uses set_ai_pending_chat API
// ---------------------------------------------------------------------------

#[test]
fn continue_in_chat_uses_pending_chat_api_after_open() {
    let source = read_source("src/prompts/chat/actions.rs");
    let block = slice_from(&source, "fn handle_continue_in_chat(");

    assert!(
        block.contains("ai::open_ai_window(cx)"),
        "continue-in-chat should open or focus the AI window first"
    );
    assert!(
        block.contains("ai::set_ai_pending_chat(cx, messages)"),
        "continue-in-chat should queue the transferred conversation through set_ai_pending_chat"
    );
}

// ---------------------------------------------------------------------------
// Selected text / browser tab — use deferred capture helpers
// ---------------------------------------------------------------------------

#[test]
fn selected_text_and_browser_tab_use_deferred_capture_helpers() {
    let source = read_source("src/app_execute/builtin_execution.rs");
    let ai_branch = slice_from(
        &source,
        "builtins::BuiltInFeature::AiCommand(cmd_type) => {",
    );
    let selected_arm = slice_from(&source, "AiCommandType::SendSelectedTextToAi => {");
    let browser_arm = slice_from(&source, "AiCommandType::SendBrowserTabToAi => {");
    let capture_helper = slice_from(&source, "fn spawn_capture_text_to_ai_after_already_hidden<");
    let selected_helper = slice_from(&source, "fn spawn_send_selected_text_to_ai_after_hide(");
    let browser_helper = slice_from(&source, "fn spawn_send_browser_tab_to_ai_after_hide(");

    assert!(
        ai_branch.contains("self.spawn_send_selected_text_to_ai_after_hide(&dctx.trace_id, cx);"),
        "SendSelectedTextToAi must delegate to the deferred capture helper"
    );
    assert!(
        ai_branch.contains("self.spawn_send_browser_tab_to_ai_after_hide(&dctx.trace_id, cx);"),
        "SendBrowserTabToAi must delegate to the deferred capture helper"
    );

    let selected_inline_block = &selected_arm[..selected_arm.len().min(500)];
    assert!(
        !selected_inline_block.contains("crate::selected_text::get_selected_text()"),
        "SendSelectedTextToAi must not capture selected text inline on the action path"
    );

    let browser_inline_block = &browser_arm[..browser_arm.len().min(500)];
    assert!(
        !browser_inline_block.contains("platform::get_focused_browser_tab_url()"),
        "SendBrowserTabToAi must not capture browser URL inline on the action path"
    );

    assert!(
        capture_helper.contains("event = \"ai_capture_scheduled\"")
            && capture_helper.contains("event = \"ai_capture_completed\"")
            && capture_helper.contains("DeferredAiWindowAction::SetInput")
            && capture_helper.contains("submit: false"),
        "Deferred text capture helper must emit capture logs and enqueue non-streaming SetInput"
    );

    assert!(
        selected_helper.contains("crate::selected_text::get_selected_text()"),
        "selected text helper must own the selected-text capture call"
    );
    assert!(
        browser_helper.contains("platform::get_focused_browser_tab_url()"),
        "browser tab helper must own the browser URL capture call"
    );
}

// ---------------------------------------------------------------------------
// Mini AI — render_main_panel branches on window_mode
// ---------------------------------------------------------------------------

#[test]
fn render_main_panel_branches_on_window_mode() {
    let source = read_source("src/ai/window/render_main_panel.rs");
    assert!(
        source.contains("match self.window_mode"),
        "render_main_panel must dispatch on window_mode"
    );
    assert!(
        source.contains("render_full_main_panel"),
        "render_main_panel must have a full branch"
    );
    assert!(
        source.contains("render_mini_main_panel"),
        "render_main_panel must have a mini branch"
    );
}

// ---------------------------------------------------------------------------
// Mini AI — overlay reuses sidebar_body
// ---------------------------------------------------------------------------

#[test]
fn mini_overlay_uses_dedicated_history_panel() {
    let source = read_source("src/ai/window/render_root.rs");
    assert!(
        source.contains("render_mini_history_panel(cx)"),
        "mini history overlay must use render_mini_history_panel"
    );
}

#[test]
fn sidebar_exposes_chat_list_body_and_mini_panel() {
    let source = read_source("src/ai/window/render_sidebar.rs");
    assert!(
        source.contains("fn render_chat_list_body("),
        "render_sidebar.rs must define render_chat_list_body"
    );
    assert!(
        source.contains("fn render_mini_history_panel("),
        "render_sidebar.rs must define render_mini_history_panel"
    );
    assert!(
        source.contains("fn render_sidebar_body("),
        "render_sidebar.rs must still define render_sidebar_body"
    );
    assert!(
        source.contains("fn render_sidebar("),
        "render_sidebar.rs must still define render_sidebar"
    );
}

// ---------------------------------------------------------------------------
// Mini AI — bounds persistence uses mode-aware role
// ---------------------------------------------------------------------------

#[test]
fn ai_bounds_persistence_is_mode_aware() {
    let interactions = read_source("src/ai/window/interactions.rs");
    assert!(
        interactions.contains("window_role_for_mode(self.window_mode)"),
        "maybe_persist_bounds must use mode-aware WindowRole"
    );

    let keydown = read_source("src/ai/window/render_keydown.rs");
    assert!(
        keydown.contains("window_role_for_mode(self.window_mode)"),
        "Cmd+W close handler must use mode-aware WindowRole"
    );
}

#[test]
fn window_role_has_ai_mini_variant() {
    let source = read_source("src/window_state/mod.rs");
    assert!(
        source.contains("AiMini"),
        "WindowRole must have an AiMini variant for separate bounds"
    );
}
