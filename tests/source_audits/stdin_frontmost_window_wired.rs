//! Source-audit tests verifying that the `frontmostWindow` stdin
//! protocol verb is wired end-to-end through the prompt-handler dispatcher
//! and produces a `FrontmostWindowResult` response receipt.
//!
//! Background — Run 5 Pass #18 investigation found that several
//! `system_control` protocol verbs (`checkAccessibility`, `frontmostWindow`,
//! `getWindowBounds`, …) were defined in
//! `src/protocol/message/variants/system_control.rs` but had no match arm in
//! `src/prompt_handler/mod.rs::ScriptListApp::handle_stdin_protocol_message`.
//! A `session.sh rpc frontmostWindow` call would parse successfully, enter
//! `handle_stdin_protocol_message`, fall through to the catch-all
//! `other => { tracing::warn!("Unsupported protocol message received via stdin"); }`,
//! and time out on the caller side because no response was ever produced.
//! Note: `src/execute_script/mod.rs:1244` already handles `FrontmostWindow`
//! inside the execute-script reader loop (for a running script sending
//! messages back), but that path is distinct from the stdin RPC path —
//! external callers via `session.sh rpc` never reach it.
//!
//! Run 7 Pass #2 `Prompt: Extend` wires `FrontmostWindow` in the stdin
//! dispatcher by adding a match arm modeled on Run 6 Pass #1 (commit
//! `730bd3c02` — `checkAccessibility`) and Run 7 Pass #1 (commit
//! `ab93d91a1` — `getWindowBounds`). The arm (a) calls
//! `crate::window_control::get_frontmost_window_of_previous_app()` — the
//! already-shipped AX probe used by the execute-script path, which is
//! already proven safe to call off-main-thread (execute_script's reader
//! runs in `std::thread::spawn`), (b) builds a `SystemWindowInfo` from the
//! returned `WindowInfo` (mapping `id`→`window_id`, `app`→`app_name`,
//! `title`→`title`, `bounds` fields directly since `Bounds` and
//! `TargetWindowBounds` share field types), (c) emits a
//! `frontmost_window_result` tracing event with the `request_id` and
//! `window_present`/`error_present` boolean fields so the log is greppable
//! per the Pass #10/#13 `cid=stdin:req:<id>` correlation convention, and
//! (d) routes a `Message::frontmost_window_result(request_id, window_opt,
//! error_opt)` through the existing `response_sender` — the same channel
//! `CheckAccessibility`, `GetWindowBounds`, and `ListAutomationWindows` use.
//!
//! These tests pin the structural invariants so a future refactor (e.g.
//! collapsing the match arms into a dispatcher table) cannot silently drop
//! the wire-up and resurrect the Pass #18 silent-drop symptom.

use super::read_source as read;

const HANDLER_PATH: &str = "src/prompt_handler/mod.rs";

fn dispatcher_body<'a>(content: &'a str) -> &'a str {
    // Anchor on the fn signature — the dispatcher body ends at the next
    // top-level `}` pair. The following helper (`make_submit_callback`)
    // makes a useful backstop.
    let start = content
        .find("pub(crate) fn handle_stdin_protocol_message(")
        .expect("handle_stdin_protocol_message must exist in prompt_handler/mod.rs");
    let rest = &content[start..];
    let end = rest
        .find("\n    pub(crate) fn make_submit_callback(")
        .unwrap_or(rest.len());
    &rest[..end]
}

#[test]
fn dispatcher_has_frontmost_window_arm() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);
    assert!(
        body.contains("Message::FrontmostWindow { request_id } =>"),
        "Expected `Message::FrontmostWindow {{ request_id }} =>` arm in \
         handle_stdin_protocol_message. Without it, stdin `frontmostWindow` \
         calls fall through to the `other =>` catch-all and are dropped as \
         `Unsupported protocol message received via stdin` — the exact \
         silent-drop symptom Run 5 Pass #18 diagnosed."
    );
}

#[test]
fn frontmost_window_arm_calls_window_control_probe() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);

    let arm_start = body
        .find("Message::FrontmostWindow { request_id } =>")
        .expect("FrontmostWindow arm must exist (see other test)");
    let after_arm = &body[arm_start..];
    let arm_end = after_arm
        .find("other =>")
        .unwrap_or(after_arm.len().min(2000));
    let arm_body = &after_arm[..arm_end];

    assert!(
        arm_body.contains("crate::window_control::get_frontmost_window_of_previous_app()"),
        "FrontmostWindow arm MUST call \
         `crate::window_control::get_frontmost_window_of_previous_app()` — \
         the shared AX probe that `src/execute_script/mod.rs:1252` already \
         uses for the script-side handler. Using a different probe (e.g. \
         a raw NSWorkspace call) would duplicate the menu-bar-owner-pid + \
         AXFocusedWindow fallback ladder and let the stdin path drift from \
         the execute_script path. Arm body was:\n{arm_body}"
    );
}

#[test]
fn frontmost_window_arm_sends_frontmost_window_result_response() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);

    let arm_start = body
        .find("Message::FrontmostWindow { request_id } =>")
        .expect("FrontmostWindow arm must exist (see other test)");
    let after_arm = &body[arm_start..];
    let arm_end = after_arm
        .find("other =>")
        .unwrap_or(after_arm.len().min(2000));
    let arm_body = &after_arm[..arm_end];

    assert!(
        arm_body.contains("Message::frontmost_window_result(request_id, window_opt, error_opt)"),
        "FrontmostWindow arm MUST construct its response via \
         `Message::frontmost_window_result(request_id, window_opt, error_opt)` — \
         the existing helper in \
         src/protocol/message/constructors/history_window.rs. Building the \
         `FrontmostWindowResult` variant by hand would duplicate the \
         serde-rename contract and let it drift."
    );
    assert!(
        arm_body.contains("sender.try_send(response)"),
        "FrontmostWindow arm MUST route the response through \
         `self.response_sender.try_send(response)` — the same channel \
         `CheckAccessibility`, `GetWindowBounds`, and \
         `ListAutomationWindows` use. Emitting the response only via \
         tracing::info would not reach the session.sh `await-response.ts` \
         listener and the caller would still time out."
    );
}

#[test]
fn frontmost_window_arm_emits_request_scoped_tracing_event() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);

    let arm_start = body
        .find("Message::FrontmostWindow { request_id } =>")
        .expect("FrontmostWindow arm must exist (see other test)");
    let after_arm = &body[arm_start..];
    let arm_end = after_arm
        .find("other =>")
        .unwrap_or(after_arm.len().min(2000));
    let arm_body = &after_arm[..arm_end];

    assert!(
        arm_body.contains(r#"event_type = "frontmost_window_result""#),
        "FrontmostWindow arm MUST emit a tracing event with \
         `event_type = \"frontmost_window_result\"` so ops can grep \
         `app.log` for the receipt the same way Pass #3 \
         (`config_fingerprint_result`), Run 6 Pass #1 \
         (`check_accessibility_result`), and Run 7 Pass #1 \
         (`get_window_bounds_result`) allow. Without a stable event_type, \
         request-id-scoped grep falls back to ambient prose matching."
    );
    assert!(
        arm_body.contains("request_id = %request_id"),
        "FrontmostWindow arm MUST include `request_id = %request_id` in \
         the tracing event fields so the correlation_id format \
         `cid=stdin:req:<request_id>` (Pass #10 shell-side grep convention) \
         appears on the receipt line. Emitting the event without the \
         request_id span field breaks concurrent-call correlation."
    );
}
