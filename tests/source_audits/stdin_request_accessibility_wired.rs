//! Source-audit tests verifying that the `requestAccessibility` stdin
//! protocol verb is wired end-to-end through the prompt-handler dispatcher
//! and produces an `AccessibilityStatus` response receipt.
//!
//! Background — Run 5 Pass #18 investigation found that several
//! `system_control` protocol verbs (`requestAccessibility`,
//! `checkAccessibility`, `getWindowBounds`, `frontmostWindow`,
//! `getSelectedText`, `setSelectedText`) were defined in
//! `src/protocol/message/variants/system_control.rs` but had no match arm in
//! `src/prompt_handler/mod.rs::ScriptListApp::handle_stdin_protocol_message`.
//! A `session.sh rpc requestAccessibility` call would parse successfully,
//! enter the dispatcher, fall through the catch-all `other =>`, and time out
//! on the caller side because no response was ever produced.
//! Note: `src/executor/selected_text.rs::handle_request_accessibility` was
//! shipped for a parallel executor-dispatcher path but returns an
//! `AccessibilityStatus` constructed against a different call site — the
//! stdin-RPC path needs its own dispatcher arm so that a direct
//! `session.sh rpc requestAccessibility --expect accessibilityStatus` round
//! trip produces the typed response.
//!
//! Run 7 Pass #5 `Prompt: Extend` wires `RequestAccessibility` by adding a
//! match arm modeled on Run 6 Pass #1 (`checkAccessibility`, commit
//! `730bd3c02`) — the two verbs share the same response shape
//! (`AccessibilityStatus { granted, requestId }`) and the same probe family
//! (`permissions_wizard::*_accessibility_permission()`). The arm (a) calls
//! `crate::permissions_wizard::request_accessibility_permission()` — the
//! shared probe at `src/permissions_wizard.rs:239` that internally invokes
//! `accessibility::application_is_trusted_with_prompt()` and triggers the
//! macOS system permission dialog when permission is not already granted,
//! (b) emits a `request_accessibility_result` tracing event with the
//! `request_id` and a `granted` field so the log is greppable per the Pass
//! #10/#13 `cid=stdin:req:<id>` correlation convention, and (c) routes a
//! `Message::accessibility_status(granted, request_id)` response through the
//! existing `response_sender` — reusing the `checkAccessibility` response
//! constructor because the request/check verbs return the same shape.
//!
//! These tests pin the structural invariants so a future refactor (e.g.
//! collapsing the match arms into a dispatcher table, or unifying the
//! request-vs-check probe into a different helper) cannot silently drop the
//! wire-up or swap the response constructor to a non-matching shape.

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
fn dispatcher_has_request_accessibility_arm() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);
    assert!(
        body.contains("Message::RequestAccessibility { request_id } =>"),
        "Expected `Message::RequestAccessibility {{ request_id }} =>` arm in \
         handle_stdin_protocol_message. Without it, stdin `requestAccessibility` \
         calls fall through to the `other =>` catch-all and are dropped as \
         `Unsupported protocol message received via stdin` — the exact \
         silent-drop symptom Run 5 Pass #18 diagnosed."
    );
}

#[test]
fn request_accessibility_arm_calls_permissions_wizard_probe() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);

    let arm_start = body
        .find("Message::RequestAccessibility { request_id } =>")
        .expect("RequestAccessibility arm must exist (see other test)");
    let after_arm = &body[arm_start..];
    let arm_end = after_arm
        .find("other =>")
        .unwrap_or(after_arm.len().min(2000));
    let arm_body = &after_arm[..arm_end];

    assert!(
        arm_body.contains("crate::permissions_wizard::request_accessibility_permission()"),
        "RequestAccessibility arm MUST call \
         `crate::permissions_wizard::request_accessibility_permission()` — the \
         same permissions_wizard probe family that `checkAccessibility` uses \
         (Run 6 Pass #1 template, commit `730bd3c02`). The probe internally \
         calls `accessibility::application_is_trusted_with_prompt()` which \
         triggers the macOS system dialog when permission is not already \
         granted. Switching to a different probe (e.g. the \
         `selected_text::request_accessibility_permission` wrapper, the \
         `keyword_manager::request_accessibility_permission` wrapper, or a \
         raw AX API call) would duplicate the permission-dialog plumbing and \
         diverge from the CheckAccessibility sibling arm. Arm body was:\n{arm_body}"
    );
}

#[test]
fn request_accessibility_arm_sends_accessibility_status_response() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);

    let arm_start = body
        .find("Message::RequestAccessibility { request_id } =>")
        .expect("RequestAccessibility arm must exist (see other test)");
    let after_arm = &body[arm_start..];
    let arm_end = after_arm
        .find("other =>")
        .unwrap_or(after_arm.len().min(2000));
    let arm_body = &after_arm[..arm_end];

    assert!(
        arm_body.contains("Message::accessibility_status(granted, request_id)"),
        "RequestAccessibility arm MUST construct its response via \
         `Message::accessibility_status(granted, request_id)` — the same \
         helper in `src/protocol/message/constructors/general.rs` that \
         `checkAccessibility` uses. The two verbs intentionally share a \
         response shape (`AccessibilityStatus {{ granted, requestId }}`) so a \
         caller awaiting `--expect accessibilityStatus` works uniformly for \
         both. Constructing the response by hand would duplicate the serde \
         rename contract; returning a different variant would break \
         `session.sh rpc requestAccessibility --expect accessibilityStatus`."
    );
    assert!(
        arm_body.contains("sender.try_send(response)"),
        "RequestAccessibility arm MUST route the response through \
         `self.response_sender.try_send(response)` — the same channel used by \
         `CheckAccessibility`, `GetWindowBounds`, `FrontmostWindow`, \
         `GetSelectedText`, and `ListAutomationWindows`."
    );
}

#[test]
fn request_accessibility_arm_emits_request_scoped_tracing_event() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);

    let arm_start = body
        .find("Message::RequestAccessibility { request_id } =>")
        .expect("RequestAccessibility arm must exist (see other test)");
    let after_arm = &body[arm_start..];
    let arm_end = after_arm
        .find("other =>")
        .unwrap_or(after_arm.len().min(2000));
    let arm_body = &after_arm[..arm_end];

    assert!(
        arm_body.contains(r#"event_type = "request_accessibility_result""#),
        "RequestAccessibility arm MUST emit a tracing event with \
         `event_type = \"request_accessibility_result\"` so ops can grep \
         `app.log` for the receipt the same way Pass #3 \
         (`config_fingerprint_result`), Run 6 Pass #1 \
         (`check_accessibility_result`), Run 7 Pass #1 \
         (`get_window_bounds_result`), Run 7 Pass #2 \
         (`frontmost_window_result`), and Run 7 Pass #3 \
         (`get_selected_text_result`) allow. The event_type name MUST be \
         distinct from `check_accessibility_result` so ops can distinguish \
         the two verbs in logs — `requestAccessibility` triggers the system \
         dialog whereas `checkAccessibility` is read-only."
    );
    assert!(
        arm_body.contains("request_id = %request_id"),
        "RequestAccessibility arm MUST include `request_id = %request_id` in \
         the tracing event fields so the correlation_id format \
         `cid=stdin:req:<request_id>` appears on the receipt line."
    );
    assert!(
        arm_body.contains("granted"),
        "RequestAccessibility arm MUST log the `granted` bool so ops can \
         tell at a glance whether the permission was granted (either \
         pre-existing or after the system dialog). Parity with the \
         `checkAccessibility` arm's field set."
    );
}
