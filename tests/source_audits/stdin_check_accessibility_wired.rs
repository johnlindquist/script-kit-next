//! Source-audit tests verifying that the `checkAccessibility` stdin
//! protocol verb is wired end-to-end through the prompt-handler dispatcher
//! and produces an `AccessibilityStatus` response receipt.
//!
//! Background — Run 5 Pass #18 investigation found that several
//! `system_control` protocol verbs (`checkAccessibility`, `frontmostWindow`,
//! …) were defined in `src/protocol/message/variants/system_control.rs`
//! but had no match arm in
//! `src/prompt_handler/mod.rs::ScriptListApp::handle_stdin_protocol_message`.
//! A `session.sh rpc checkAccessibility` call would parse successfully,
//! enter `handle_stdin_protocol_message`, fall through to the catch-all
//! `other => { tracing::warn!("Unsupported protocol message received via stdin"); }`,
//! and time out on the caller side because no response was ever produced.
//!
//! Run 6 Pass #1 `Prompt: Extend` wires `CheckAccessibility` by adding an
//! explicit match arm that (a) invokes
//! `crate::permissions_wizard::check_accessibility_permission()` (the
//! already-shipped macOS `AXIsProcessTrusted` probe), (b) emits a
//! `check_accessibility_result` tracing event with the `request_id` and
//! `granted` boolean so the log is greppable per the Pass #10 / Pass #13
//! `cid=stdin:req:<id>` correlation convention, and (c) routes a
//! `Message::accessibility_status(granted, request_id)` through the
//! existing `response_sender` — the same path `ListAutomationWindows` uses.
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
fn dispatcher_has_check_accessibility_arm() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);
    assert!(
        body.contains("Message::CheckAccessibility { request_id } =>"),
        "Expected `Message::CheckAccessibility {{ request_id }} =>` arm in \
         handle_stdin_protocol_message. Without it, stdin `checkAccessibility` \
         calls fall through to the `other =>` catch-all and are dropped as \
         `Unsupported protocol message received via stdin` — the exact \
         silent-drop symptom Run 5 Pass #18 diagnosed."
    );
}

#[test]
fn check_accessibility_arm_calls_permissions_wizard_probe() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);

    let arm_start = body
        .find("Message::CheckAccessibility { request_id } =>")
        .expect("CheckAccessibility arm must exist (see other test)");
    let after_arm = &body[arm_start..];
    let arm_end = after_arm
        .find("other =>")
        .unwrap_or(after_arm.len().min(800));
    let arm_body = &after_arm[..arm_end];

    assert!(
        arm_body.contains("crate::permissions_wizard::check_accessibility_permission()"),
        "CheckAccessibility arm MUST call \
         `crate::permissions_wizard::check_accessibility_permission()` to \
         derive `granted`. A hardcoded `true`/`false` or a direct \
         `accessibility::application_is_trusted()` call would skip the \
         `#[instrument]`-wrapped probe and lose the debug tracing in \
         `permissions_wizard.rs`. Arm body was:\n{arm_body}"
    );
}

#[test]
fn check_accessibility_arm_sends_accessibility_status_response() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);

    let arm_start = body
        .find("Message::CheckAccessibility { request_id } =>")
        .expect("CheckAccessibility arm must exist (see other test)");
    let after_arm = &body[arm_start..];
    let arm_end = after_arm
        .find("other =>")
        .unwrap_or(after_arm.len().min(800));
    let arm_body = &after_arm[..arm_end];

    assert!(
        arm_body.contains("Message::accessibility_status(granted, request_id)"),
        "CheckAccessibility arm MUST construct its response via \
         `Message::accessibility_status(granted, request_id)` — the \
         existing helper in src/protocol/message/constructors/general.rs. \
         Building the `AccessibilityStatus` variant by hand would duplicate \
         the serde-rename contract and let it drift."
    );
    assert!(
        arm_body.contains("sender.try_send(response)"),
        "CheckAccessibility arm MUST route the response through \
         `self.response_sender.try_send(response)` — the same channel \
         `ListAutomationWindows` uses. Emitting the response only via \
         tracing::info would not reach the session.sh `await-response.ts` \
         listener and the caller would still time out."
    );
}

#[test]
fn check_accessibility_arm_emits_request_scoped_tracing_event() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);

    let arm_start = body
        .find("Message::CheckAccessibility { request_id } =>")
        .expect("CheckAccessibility arm must exist (see other test)");
    let after_arm = &body[arm_start..];
    let arm_end = after_arm
        .find("other =>")
        .unwrap_or(after_arm.len().min(800));
    let arm_body = &after_arm[..arm_end];

    assert!(
        arm_body.contains(r#"event_type = "check_accessibility_result""#),
        "CheckAccessibility arm MUST emit a tracing event with \
         `event_type = \"check_accessibility_result\"` so ops can grep \
         `app.log` for the receipt the same way Pass #3 (getConfigFingerprint \
         / `config_fingerprint_result`) and Pass #10's `cid=stdin:req:<id>` \
         correlation convention allow. Without a stable event_type, \
         request-id-scoped grep falls back to ambient prose matching."
    );
    assert!(
        arm_body.contains("request_id = %request_id"),
        "CheckAccessibility arm MUST include `request_id = %request_id` in \
         the tracing event fields so the correlation_id format \
         `cid=stdin:req:<request_id>` (Pass #10 shell-side grep convention) \
         appears on the receipt line. Emitting the event without the \
         request_id span field breaks concurrent-call correlation."
    );
}
