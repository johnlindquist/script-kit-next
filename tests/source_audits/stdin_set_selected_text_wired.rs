//! Source-audit tests verifying that the `setSelectedText` stdin
//! protocol verb is wired end-to-end through the prompt-handler dispatcher
//! and produces a `TextSet` response receipt.
//!
//! Background — Run 5 Pass #18 investigation found that several
//! `system_control` protocol verbs (`checkAccessibility`,
//! `requestAccessibility`, `frontmostWindow`, `getWindowBounds`,
//! `getSelectedText`, `setSelectedText`) were defined in
//! `src/protocol/message/variants/system_control.rs` but had no match arm
//! in `src/prompt_handler/mod.rs::ScriptListApp::handle_stdin_protocol_message`.
//! A `session.sh rpc setSelectedText` call would parse successfully, enter
//! the dispatcher, fall through the catch-all `other =>`, and time out on
//! the caller side because no response was ever produced.
//! Note: `src/executor/selected_text.rs::handle_set_selected_text` was
//! shipped for a parallel executor-dispatcher path but returns
//! `Message::Submit { id, value }` for SDK matching — it does NOT produce
//! the typed `TextSet` response that `session.sh rpc setSelectedText
//! --expect textSet` needs.
//!
//! `setSelectedText` is the **write-type** verb in the system_control
//! family — it injects text into the frontmost app via clipboard+Cmd+V
//! simulation (see `src/selected_text.rs:191` and the module doc at
//! `src/selected_text.rs:10`). It is the last unwired verb after Run 7
//! Passes #1 (`getWindowBounds`), #2 (`frontmostWindow`), #3
//! (`getSelectedText`), and #5 (`requestAccessibility`).
//!
//! Run 7 Pass #6 `Prompt: Extend` wires `SetSelectedText` by adding a
//! match arm modeled on Run 7 Pass #3 (`getSelectedText`). The arm (a)
//! calls `crate::selected_text::set_selected_text(&text)` — the shared
//! probe at `src/selected_text.rs:191` that `src/ai/tab_context.rs:4737`
//! already calls from an async executor context, proving off-main-thread
//! safety, (b) maps `Ok(())` to `Message::text_set_success(request_id)`
//! and `Err(e)` to `Message::text_set_error(e.to_string(), request_id)` —
//! the `TextSet` response shape has an explicit `success: bool` + optional
//! `error: String` so callers can distinguish the two outcomes without
//! string-sniffing, (c) emits a `set_selected_text_result` tracing event
//! with the `request_id`, a `text_len` field (instead of logging the
//! actual text — avoids leaking user-supplied content into app.log), and
//! a `success` boolean so the log is greppable per the Pass #10/#13
//! `cid=stdin:req:<id>` correlation convention, and (d) routes the
//! response through the existing `response_sender`.
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
fn dispatcher_has_set_selected_text_arm() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);
    assert!(
        body.contains("Message::SetSelectedText { text, request_id } =>"),
        "Expected `Message::SetSelectedText {{ text, request_id }} =>` arm \
         in handle_stdin_protocol_message. Without it, stdin \
         `setSelectedText` calls fall through to the `other =>` catch-all \
         and are dropped as `Unsupported protocol message received via \
         stdin` — the exact silent-drop symptom Run 5 Pass #18 diagnosed. \
         This is the *write-type* verb in the system_control family so a \
         silent drop is especially bad: callers believe text was inserted \
         but nothing happens."
    );
}

#[test]
fn set_selected_text_arm_calls_selected_text_probe() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);

    let arm_start = body
        .find("Message::SetSelectedText { text, request_id } =>")
        .expect("SetSelectedText arm must exist (see other test)");
    let after_arm = &body[arm_start..];
    let arm_end = after_arm
        .find("other =>")
        .unwrap_or(after_arm.len().min(2000));
    let arm_body = &after_arm[..arm_end];

    assert!(
        arm_body.contains("crate::selected_text::set_selected_text(&text)"),
        "SetSelectedText arm MUST call \
         `crate::selected_text::set_selected_text(&text)` — the shared \
         probe that `src/ai/tab_context.rs:4737` already uses. A direct \
         `arboard::Clipboard::set_text` + `enigo` call would duplicate the \
         accessibility-permission gate at src/selected_text.rs:192 and the \
         clipboard-restore ladder at src/selected_text.rs:196–254. Arm \
         body was:\n{arm_body}"
    );
}

#[test]
fn set_selected_text_arm_sends_text_set_response() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);

    let arm_start = body
        .find("Message::SetSelectedText { text, request_id } =>")
        .expect("SetSelectedText arm must exist (see other test)");
    let after_arm = &body[arm_start..];
    let arm_end = after_arm
        .find("other =>")
        .unwrap_or(after_arm.len().min(2000));
    let arm_body = &after_arm[..arm_end];

    assert!(
        arm_body.contains("Message::text_set_success(request_id)"),
        "SetSelectedText arm MUST construct its Ok-branch response via \
         `Message::text_set_success(request_id)` — the existing helper in \
         src/protocol/message/constructors/general.rs:413. Building the \
         `TextSet` variant by hand would duplicate the serde-rename \
         contract (`success`/`error`/`requestId`) and let it drift. \
         Returning `Message::Submit {{ id, value: None }}` (the shape the \
         executor path at src/executor/selected_text.rs:147 uses) would \
         break the `session.sh rpc setSelectedText --expect textSet` typed \
         match."
    );
    assert!(
        arm_body.contains("Message::text_set_error(e.to_string(), request_id)"),
        "SetSelectedText arm MUST construct its Err-branch response via \
         `Message::text_set_error(e.to_string(), request_id)` — the \
         existing helper in src/protocol/message/constructors/general.rs:422. \
         This populates the `success: false, error: Some(<msg>)` shape so \
         callers can distinguish success vs failure without string-sniffing \
         the `ERROR:` prefix the executor path prepends to a `Submit.value`. \
         Returning `Ok(())` on the Err branch or dropping the error would \
         make a failed paste look like success on the wire."
    );
    assert!(
        arm_body.contains("sender.try_send(response)"),
        "SetSelectedText arm MUST route the response through \
         `self.response_sender.try_send(response)` — the same channel \
         `CheckAccessibility`, `GetWindowBounds`, `FrontmostWindow`, \
         `GetSelectedText`, and `RequestAccessibility` use."
    );
}

#[test]
fn set_selected_text_arm_emits_request_scoped_tracing_event_without_leaking_text() {
    let content = read(HANDLER_PATH);
    let body = dispatcher_body(&content);

    let arm_start = body
        .find("Message::SetSelectedText { text, request_id } =>")
        .expect("SetSelectedText arm must exist (see other test)");
    let after_arm = &body[arm_start..];
    let arm_end = after_arm
        .find("other =>")
        .unwrap_or(after_arm.len().min(2000));
    let arm_body = &after_arm[..arm_end];

    assert!(
        arm_body.contains(r#"event_type = "set_selected_text_result""#),
        "SetSelectedText arm MUST emit a tracing event with \
         `event_type = \"set_selected_text_result\"` so ops can grep \
         `app.log` for the receipt the same way Pass #3 \
         (`config_fingerprint_result`), Run 6 Pass #1 \
         (`check_accessibility_result`), Run 7 Pass #1 \
         (`get_window_bounds_result`), Run 7 Pass #2 \
         (`frontmost_window_result`), Run 7 Pass #3 \
         (`get_selected_text_result`), and Run 7 Pass #5 \
         (`request_accessibility_result`) allow. The event_type name MUST \
         be distinct from the sibling verbs so ops can distinguish the \
         write-type verb in logs — `setSelectedText` has a side effect \
         (text injection) whereas the other verbs are read-only."
    );
    assert!(
        arm_body.contains("request_id = %request_id"),
        "SetSelectedText arm MUST include `request_id = %request_id` in \
         the tracing event fields so the correlation_id format \
         `cid=stdin:req:<request_id>` appears on the receipt line."
    );
    assert!(
        arm_body.contains("text_len"),
        "SetSelectedText arm MUST log `text_len` rather than the raw \
         `text` field. The raw text is user-supplied content — the SDK \
         caller may pass passwords from a password manager, generated \
         secrets, private notes, etc. Logging it leaks those to app.log. \
         A future refactor that 'helpfully' adds `text = %text` to the \
         tracing event would break this privacy invariant; this pin flags \
         that drift."
    );
    assert!(
        arm_body.contains("success"),
        "SetSelectedText arm MUST log a `success` boolean so ops can \
         distinguish Ok-paths from Err-paths at a glance without parsing \
         the response variant. Parity with Run 7 Pass #5's `granted` \
         field on `request_accessibility_result`."
    );
    assert!(
        !arm_body.contains("text = %text") && !arm_body.contains("text = ?text"),
        "SetSelectedText arm MUST NOT log the raw `text` field. User- \
         supplied content from the SDK caller may contain passwords, \
         credit-card numbers, or private notes — logging it creates a \
         data leak to app.log. Use `text_len` instead (number of bytes \
         pasted)."
    );
}
