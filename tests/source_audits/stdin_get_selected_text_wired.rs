//! Source-audit tests verifying that the `getSelectedText` stdin
//! protocol verb is wired end-to-end through the prompt-handler dispatcher
//! and produces a `SelectedText` response receipt.
//!
//! Background — Run 5 Pass #18 investigation found that several
//! `system_control` protocol verbs (`checkAccessibility`, `frontmostWindow`,
//! `getWindowBounds`, `getSelectedText`, …) were defined in
//! `src/protocol/message/variants/system_control.rs` but had no match arm in
//! `src/prompt_handler/mod.rs::ScriptListApp::handle_stdin_protocol_message`.
//! A `session.sh rpc getSelectedText` call would parse successfully, enter
//! `handle_stdin_protocol_message`, fall through to the catch-all
//! `other => { tracing::warn!("Unsupported protocol message received via stdin"); }`,
//! and time out on the caller side because no response was ever produced.
//! Note: `src/executor/selected_text.rs::handle_selected_text_message` was
//! shipped for a parallel executor dispatcher path but returns
//! `Message::Submit { id, value }` for SDK matching — it does NOT produce
//! the typed `SelectedText` response that `session.sh rpc getSelectedText
//! --expect selectedText` needs.
//!
//! Run 7 Pass #3 `Prompt: Extend` wires `GetSelectedText` by adding a match
//! arm modeled on Run 6 Pass #1 (`checkAccessibility`, commit `730bd3c02`),
//! Run 7 Pass #1 (`getWindowBounds`, commit `ab93d91a1`), and Run 7 Pass #2
//! (`frontmostWindow`, commit `ac2525d1c`). The arm (a) calls
//! `crate::selected_text::get_selected_text()` — the shared probe at
//! `src/selected_text.rs:141` that `app_execute/builtin_execution.rs` and
//! `context_snapshot/capture.rs` already call from async/background-executor
//! contexts, proving off-main-thread safety, (b) maps `Err(e)` to
//! `(String::new(), true)` with a warn log rather than dropping the verb —
//! the `SelectedText` response shape has no error field, so empty text with
//! a greppable warn line is the best we can do, (c) emits a
//! `get_selected_text_result` tracing event with the `request_id`, a
//! `text_len` field (instead of logging the actual text — avoids leaking
//! user-typed content into app.log), and an `error_present` boolean so the
//! log is greppable per the Pass #10/#13 `cid=stdin:req:<id>` correlation
//! convention, and (d) routes a `Message::selected_text_response(text,
//! request_id)` through the existing `response_sender`.
//!
//! These tests pin the structural invariants so a future refactor (e.g.
//! collapsing the match arms into a dispatcher table, or a privacy audit
//! that forgets `text_len` protects user content and adds raw `text`
//! logging) cannot silently drop the wire-up or introduce a data leak.

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
fn dispatcher_has_get_selected_text_arm() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);
    assert!(
        body.contains("Message::GetSelectedText { request_id } =>"),
        "Expected `Message::GetSelectedText {{ request_id }} =>` arm in \
         handle_stdin_protocol_message. Without it, stdin `getSelectedText` \
         calls fall through to the `other =>` catch-all and are dropped as \
         `Unsupported protocol message received via stdin` — the exact \
         silent-drop symptom Run 5 Pass #18 diagnosed."
    );
}

#[test]
fn get_selected_text_arm_calls_selected_text_probe() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);

    let arm_start = body
        .find("Message::GetSelectedText { request_id } =>")
        .expect("GetSelectedText arm must exist (see other test)");
    let after_arm = &body[arm_start..];
    let arm_end = after_arm
        .find("other =>")
        .unwrap_or(after_arm.len().min(2000));
    let arm_body = &after_arm[..arm_end];

    assert!(
        arm_body.contains("crate::selected_text::get_selected_text()"),
        "GetSelectedText arm MUST call \
         `crate::selected_text::get_selected_text()` — the shared probe \
         that `app_execute/builtin_execution.rs` and \
         `context_snapshot/capture.rs` already use. A direct AX API call \
         or a raw CGEvent clipboard simulation would duplicate the \
         3-strategy ladder (AX → clipboard → cache) and bypass the \
         accessibility permission check at src/selected_text.rs:143. \
         Arm body was:\n{arm_body}"
    );
}

#[test]
fn get_selected_text_arm_sends_selected_text_response() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);

    let arm_start = body
        .find("Message::GetSelectedText { request_id } =>")
        .expect("GetSelectedText arm must exist (see other test)");
    let after_arm = &body[arm_start..];
    let arm_end = after_arm
        .find("other =>")
        .unwrap_or(after_arm.len().min(2000));
    let arm_body = &after_arm[..arm_end];

    assert!(
        arm_body.contains("Message::selected_text_response(text, request_id)"),
        "GetSelectedText arm MUST construct its response via \
         `Message::selected_text_response(text, request_id)` — the \
         existing helper in src/protocol/message/constructors/general.rs. \
         Building the `SelectedText` variant by hand would duplicate the \
         serde-rename contract and let it drift. Returning \
         `Message::Submit {{ id, value }}` (the shape the executor path \
         at src/executor/selected_text.rs uses) would break the \
         `session.sh rpc getSelectedText --expect selectedText` typed \
         match."
    );
    assert!(
        arm_body.contains("sender.try_send(response)"),
        "GetSelectedText arm MUST route the response through \
         `self.response_sender.try_send(response)` — the same channel \
         `CheckAccessibility`, `GetWindowBounds`, `FrontmostWindow`, and \
         `ListAutomationWindows` use."
    );
}

#[test]
fn get_selected_text_arm_emits_request_scoped_tracing_event_without_leaking_text() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);

    let arm_start = body
        .find("Message::GetSelectedText { request_id } =>")
        .expect("GetSelectedText arm must exist (see other test)");
    let after_arm = &body[arm_start..];
    let arm_end = after_arm
        .find("other =>")
        .unwrap_or(after_arm.len().min(2000));
    let arm_body = &after_arm[..arm_end];

    assert!(
        arm_body.contains(r#"event_type = "get_selected_text_result""#),
        "GetSelectedText arm MUST emit a tracing event with \
         `event_type = \"get_selected_text_result\"` so ops can grep \
         `app.log` for the receipt the same way Pass #3 \
         (`config_fingerprint_result`), Run 6 Pass #1 \
         (`check_accessibility_result`), Run 7 Pass #1 \
         (`get_window_bounds_result`), and Run 7 Pass #2 \
         (`frontmost_window_result`) allow."
    );
    assert!(
        arm_body.contains("request_id = %request_id"),
        "GetSelectedText arm MUST include `request_id = %request_id` in \
         the tracing event fields so the correlation_id format \
         `cid=stdin:req:<request_id>` appears on the receipt line."
    );
    assert!(
        arm_body.contains("text_len = text.len()"),
        "GetSelectedText arm MUST log `text_len = text.len()` rather than \
         the raw `text` field. The raw text is user-typed content from \
         whatever app was frontmost when `getSelectedText` was called — \
         logging it leaks passwords, credit card numbers, private notes, \
         etc. into app.log. A future refactor that 'helpfully' adds \
         `text = %text` to the tracing event would break this privacy \
         invariant; this pin flags that drift."
    );
    assert!(
        !arm_body.contains("text = %text") && !arm_body.contains("text = ?text"),
        "GetSelectedText arm MUST NOT log the raw `text` field. User-typed \
         content from the frontmost app may contain passwords, credit \
         card numbers, or private notes — logging it creates a data leak \
         to app.log. Use `text_len = text.len()` instead."
    );
}
