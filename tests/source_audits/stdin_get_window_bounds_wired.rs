//! Source-audit tests verifying that the `getWindowBounds` stdin
//! protocol verb is wired end-to-end through the prompt-handler dispatcher
//! and produces a `WindowBounds` response receipt.
//!
//! Background — Run 5 Pass #18 investigation found that several
//! `system_control` protocol verbs (`checkAccessibility`, `frontmostWindow`,
//! `getWindowBounds`, …) were defined in
//! `src/protocol/message/variants/system_control.rs` but had no match arm in
//! `src/prompt_handler/mod.rs::ScriptListApp::handle_stdin_protocol_message`.
//! A `session.sh rpc getWindowBounds` call would parse successfully, enter
//! `handle_stdin_protocol_message`, fall through to the catch-all
//! `other => { tracing::warn!("Unsupported protocol message received via stdin"); }`,
//! and time out on the caller side because no response was ever produced.
//!
//! Run 7 Pass #1 `Prompt: Extend` wires `GetWindowBounds` by adding an
//! explicit match arm modeled on Run 6 Pass #1's `CheckAccessibility`
//! wiring (commit 730bd3c02). The arm (a) reads main-window bounds from
//! `crate::windows::list_automation_windows()` — the thread-safe Mutex-
//! protected registry used by `ListAutomationWindows` — matching on the
//! `"main"` id (the off-main-thread stdin dispatcher cannot call
//! `platform::get_main_window_bounds()` which is main-thread-only),
//! (b) emits a `get_window_bounds_result` tracing event with the
//! `request_id` and the four bounds fields so the log is greppable per
//! the Pass #10 / Pass #13 `cid=stdin:req:<id>` correlation convention,
//! and (c) routes a `Message::window_bounds(x, y, width, height, request_id)`
//! through the existing `response_sender` — the same channel the
//! `CheckAccessibility` arm uses.
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
fn dispatcher_has_get_window_bounds_arm() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);
    assert!(
        body.contains("Message::GetWindowBounds { request_id } =>"),
        "Expected `Message::GetWindowBounds {{ request_id }} =>` arm in \
         handle_stdin_protocol_message. Without it, stdin `getWindowBounds` \
         calls fall through to the `other =>` catch-all and are dropped as \
         `Unsupported protocol message received via stdin` — the exact \
         silent-drop symptom Run 5 Pass #18 diagnosed."
    );
}

#[test]
fn get_window_bounds_arm_reads_main_registry_bounds() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);

    let arm_start = body
        .find("Message::GetWindowBounds { request_id } =>")
        .expect("GetWindowBounds arm must exist (see other test)");
    let after_arm = &body[arm_start..];
    let arm_end = after_arm
        .find("other =>")
        .unwrap_or(after_arm.len().min(1200));
    let arm_body = &after_arm[..arm_end];

    assert!(
        arm_body.contains("crate::windows::list_automation_windows()"),
        "GetWindowBounds arm MUST source bounds from \
         `crate::windows::list_automation_windows()` — the thread-safe \
         Mutex-protected registry. A direct \
         `crate::platform::get_main_window_bounds()` call would panic at \
         runtime because the stdin dispatcher runs off the main thread and \
         the platform helper requires the main thread. Arm body was:\n{arm_body}"
    );
    assert!(
        arm_body.contains(".id == \"main\""),
        "GetWindowBounds arm MUST filter the registry on `.id == \"main\"` \
         to select the main prompt window. Returning bounds of the first \
         window in the Vec would silently report Agent Chat / notes window bounds \
         on sessions where those windows appear earlier in the registry."
    );
}

#[test]
fn get_window_bounds_arm_sends_window_bounds_response() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);

    let arm_start = body
        .find("Message::GetWindowBounds { request_id } =>")
        .expect("GetWindowBounds arm must exist (see other test)");
    let after_arm = &body[arm_start..];
    let arm_end = after_arm
        .find("other =>")
        .unwrap_or(after_arm.len().min(1200));
    let arm_body = &after_arm[..arm_end];

    assert!(
        arm_body.contains("Message::window_bounds(x, y, width, height, request_id)"),
        "GetWindowBounds arm MUST construct its response via \
         `Message::window_bounds(x, y, width, height, request_id)` — the \
         existing helper in src/protocol/message/constructors/history_window.rs. \
         Building the `WindowBounds` variant by hand would duplicate the \
         serde-rename contract and let it drift."
    );
    assert!(
        arm_body.contains("sender.try_send(response)"),
        "GetWindowBounds arm MUST route the response through \
         `self.response_sender.try_send(response)` — the same channel \
         `CheckAccessibility` and `ListAutomationWindows` use. Emitting \
         the response only via tracing::info would not reach the \
         session.sh `await-response.ts` listener and the caller would \
         still time out."
    );
}

#[test]
fn get_window_bounds_arm_emits_request_scoped_tracing_event() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);

    let arm_start = body
        .find("Message::GetWindowBounds { request_id } =>")
        .expect("GetWindowBounds arm must exist (see other test)");
    let after_arm = &body[arm_start..];
    let arm_end = after_arm
        .find("other =>")
        .unwrap_or(after_arm.len().min(1200));
    let arm_body = &after_arm[..arm_end];

    assert!(
        arm_body.contains(r#"event_type = "get_window_bounds_result""#),
        "GetWindowBounds arm MUST emit a tracing event with \
         `event_type = \"get_window_bounds_result\"` so ops can grep \
         `app.log` for the receipt the same way Pass #3 (getConfigFingerprint \
         / `config_fingerprint_result`) and Run 6 Pass #1 \
         (checkAccessibility / `check_accessibility_result`) allow. \
         Without a stable event_type, request-id-scoped grep falls back \
         to ambient prose matching."
    );
    assert!(
        arm_body.contains("request_id = %request_id"),
        "GetWindowBounds arm MUST include `request_id = %request_id` in \
         the tracing event fields so the correlation_id format \
         `cid=stdin:req:<request_id>` (Pass #10 shell-side grep convention) \
         appears on the receipt line. Emitting the event without the \
         request_id span field breaks concurrent-call correlation."
    );
}
