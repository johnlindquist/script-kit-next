//! Source-level contract test for Run 5 Pass #15 of Run 5 — fix of the
//! silent-drop gap in `scripts/agentic/session.sh` `cmd_rpc` where a
//! malformed payload caused a generic `timeout` error envelope while
//! the app.log had a perfectly-correlated `stdin_parse_failed` event
//! with `cid=stdin:req:<request_id>` (emitted by the lenient extract
//! landed in Pass #13).
//!
//! Pass #15 discovered this while generating receipt baselines for
//! `getElements` on emoji-picker. The Verify payload had a malformed
//! `target:{kind:"main"}` (missing the required `type` discriminator
//! for `AutomationWindowTarget`). `session.sh rpc` waited the full
//! 4000ms timeout and returned:
//!   {"status":"error","error":{"code":"timeout",
//!    "message":"No response matching requestId 'p15-get' and type
//!    'elementsResult' within 4000ms"}}
//! …while app.log already had:
//!   cid=stdin:req:p15-get Failed to parse external command
//!   event_type=stdin_parse_failed line_len=69
//!   payload_summary={type:getElements, len:69}
//!   error=missing field `type`
//! The DX cost is 4 seconds of wasted wall-clock plus a misleading
//! error code that doesn't tell the caller the app rejected the payload.
//!
//! The fix mirrors `cmd_send --await-parse`'s sad-path receipt scan
//! (Pass #8 Run 5 happy path + Pass #13 Run 5 sad path) onto the
//! `cmd_rpc` post-hoc error path: after `await-response.ts` exits
//! non-zero, scan the app.log tail since `start_offset` for a
//! `stdin_parse_failed` event scoped to this request's
//! `cid=stdin:req:<request_id>`. If found, replace the generic timeout
//! envelope with `{status:"error", error:{code:"parse_error",
//! message:"<extracted error>"}}`.
//!
//! This contract pins the key invariants so a future refactor that
//! "simplifies" `cmd_rpc` by dropping the post-hoc scan, renaming the
//! code value, or unscoping the grep would flip the tests red. Without
//! these pins, breakage would only surface when a human (or attacker
//! probe) noticed that `session.sh rpc` returned `timeout` on a payload
//! the app had already rejected at parse time.

const SESSION_SH: &str = include_str!("../scripts/agentic/session.sh");

fn cmd_rpc_body() -> &'static str {
    // Anchor on the cmd_rpc signature — extract the function body up to
    // the next top-level `}` so tests don't pick up stray matches in
    // cmd_send or unrelated helpers.
    let start = SESSION_SH
        .find("cmd_rpc() {")
        .expect("cmd_rpc function must exist in session.sh");
    let after = &SESSION_SH[start..];
    let end = after
        .find("\n}\n")
        .expect("cmd_rpc must be terminated by `\\n}\\n`");
    &after[..end + 3]
}

#[test]
fn cmd_rpc_scopes_post_hoc_parse_failure_check_to_request_id() {
    // The post-hoc scan MUST be scoped to this request's correlation_id.
    // Unscoped grep would cross-correlate with concurrent parse failures
    // on other requests — the exact class of bug Pass #12 attacker probe
    // found on the `cmd_send --await-parse` sad path, closed by Pass #13.
    // Reintroducing that bug on `cmd_rpc` would be a regression of the
    // same shape. Trailing space prevents prefix matches
    // (e.g. `p15-get` must not match `cid=stdin:req:p15-get-notarget`).
    let body = cmd_rpc_body();
    assert!(
        body.contains(r#"grep -F -- "cid=stdin:req:${request_id} ""#),
        "scripts/agentic/session.sh::cmd_rpc MUST scope its post-hoc \
         `stdin_parse_failed` check to `cid=stdin:req:${{request_id}} ` \
         (with trailing space). Unscoped grep cross-correlates with \
         concurrent parse failures and resurrects Pass #12's race. \
         See tests/session_send_await_parse_contract.rs for the \
         symmetric cmd_send pin."
    );
    assert!(
        body.contains("grep -m1 'event_type=stdin_parse_failed'"),
        "scripts/agentic/session.sh::cmd_rpc MUST grep for \
         `event_type=stdin_parse_failed` specifically. Matching on \
         generic `Failed to parse` prose would pick up unrelated error \
         lines (e.g. `Failed to parse responses.ndjson`) and falsely \
         attribute them to this request."
    );
}

#[test]
fn cmd_rpc_gates_parse_failure_check_on_nonzero_exit() {
    // The post-hoc scan runs only when await-response.ts exited non-zero.
    // Running it on every call (including success) would waste I/O and,
    // worse, could pick up a *prior* parse failure that predates this
    // send but post-dates start_offset in a pathological race. Gating
    // on exit_code keeps the scan a pure fallback.
    let body = cmd_rpc_body();
    assert!(
        body.contains(r#"if [ "$exit_code" -ne 0 ] && [ -f "$log_path" ]; then"#),
        "scripts/agentic/session.sh::cmd_rpc MUST gate its post-hoc \
         parse-failure scan on `$exit_code -ne 0`. Running on every call \
         is wasteful and could attribute a prior parse failure to this \
         request."
    );
}

#[test]
fn cmd_rpc_emits_parse_error_code_not_timeout() {
    // On a detected parse failure, the envelope's error.code MUST be
    // `parse_error` — NOT `timeout` (the generic fallback) and NOT
    // `parseError` (the camelCase cmd_send --await-parse outcome used
    // as a *field value*, not an error code). This three-way distinction
    // matters for programmatic callers that key on `error.code`.
    let body = cmd_rpc_body();
    assert!(
        body.contains(r#""code":"parse_error""#),
        "scripts/agentic/session.sh::cmd_rpc MUST emit \
         `\"code\":\"parse_error\"` in the parse-failure recovery envelope. \
         Using `timeout` (the pre-fix code) or `parseError` (the \
         cmd_send outcome value) breaks programmatic callers that \
         distinguish these three categories."
    );
}

#[test]
fn cmd_rpc_truncates_and_escapes_extracted_error_text() {
    // The parse-failure error text comes from app.log and is
    // attacker-influenced (serde error messages can embed the input
    // verbatim). The shell MUST truncate and JSON-escape before
    // interpolating into the envelope or we risk emitting invalid JSON
    // for messages containing `\` or `"`. Symmetric with the sad-path
    // error handling in cmd_send --await-parse (line ~446 of
    // session.sh).
    let body = cmd_rpc_body();
    assert!(
        body.contains(r#"cut -c1-200"#),
        "scripts/agentic/session.sh::cmd_rpc MUST truncate the \
         extracted error text to 200 chars (matches the cmd_send \
         --await-parse bound). Unbounded error text can blow out the \
         caller's 16KB stdin-pipe budget when probed with \
         attacker-crafted long errors."
    );
    assert!(
        body.contains(r#"sed 's/\\/\\\\/g; s/"/\\"/g'"#),
        "scripts/agentic/session.sh::cmd_rpc MUST JSON-escape the \
         extracted error text (backslashes then double quotes) before \
         interpolation. Without escaping, a serde error containing a \
         backslash or quote produces invalid JSON in the envelope."
    );
}

#[test]
fn cmd_rpc_falls_back_to_unscoped_grep_on_charset_boundary() {
    // Rust's `extract_request_id_lenient` (src/stdin_commands/mod.rs)
    // accepts only `[A-Za-z0-9_.:/-]` as a correlation id — payloads
    // with requestIds outside that charset (e.g. `a+b`, `a\b`) get an
    // auto-generated `stdin:parse:<uuid>` cid in app.log, NOT
    // `stdin:req:${request_id}`. The scoped grep would miss those
    // parse-failure lines, silently degrading the post-hoc scan to a
    // generic `timeout` envelope even though app.log carries a
    // correlated failure (just under a different cid prefix).
    //
    // Run 5 Pass #16 attacker probe documented this as
    // `[?] cmd-rpc-scan-misses-requestid-charset-boundary`. The fix
    // (Run 8 Pass #12) mirrors `cmd_send`'s charset gate at line ~390
    // of session.sh: when `request_id` fails the conservative charset
    // check, fall back to unscoped `grep -m1 'event_type=stdin_parse_failed'`
    // on the log tail. This pin asserts both branches — the scoped
    // primary path AND the unscoped fallback. A "simplification" that
    // drops the fallback would resurrect the Pass #16 anomaly.
    let body = cmd_rpc_body();
    assert!(
        body.contains(r#"if [[ "$request_id" =~ ^[A-Za-z0-9_.:/-]+$ ]]; then"#),
        "scripts/agentic/session.sh::cmd_rpc MUST gate its scoped \
         `cid=stdin:req:${{request_id}}` grep on a charset check matching \
         cmd_send's line ~390 pattern. Without the gate, requestIds \
         outside `[A-Za-z0-9_.:/-]` land on a Rust-side `stdin:parse:<uuid>` \
         cid that the scoped grep never matches — the Pass #16 anomaly. \
         Missing anchor: `if [[ \"$request_id\" =~ ^[A-Za-z0-9_.:/-]+$ ]]; then`."
    );
    assert!(
        body.contains(
            r#"else
        failed_line="$(tail -c "$tail_bytes" "$log_path" 2>/dev/null | grep -m1 'event_type=stdin_parse_failed' || true)""#
        ),
        "scripts/agentic/session.sh::cmd_rpc MUST fall back to an \
         UNSCOPED `grep -m1 'event_type=stdin_parse_failed'` when the \
         charset gate fails. The fallback trades cross-call \
         de-interleaving for restored parse-error surfacing — the \
         correct trade for the attacker-probe case where agents test \
         with exotic requestIds. Missing anchor: the else-branch \
         unscoped grep pattern."
    );
}

#[test]
fn cmd_rpc_preserves_start_offset_for_parse_failure_window() {
    // The scan boundary is `start_offset` — the byte offset of app.log
    // recorded BEFORE the FIFO write. Using a later offset (e.g.
    // post-await) would miss the parse failure event; using 0 would
    // pick up unrelated prior failures. The cmd_rpc function already
    // records start_offset for the happy-path response wait; this test
    // pins that the parse-failure scan uses that same offset rather
    // than recording a second one.
    let body = cmd_rpc_body();
    assert!(
        body.contains(r#"new_size > start_offset"#)
            || body.contains(r#""$new_size" -gt "$start_offset""#),
        "scripts/agentic/session.sh::cmd_rpc MUST bound the tail read \
         from `start_offset` (the pre-FIFO-write byte position) to \
         avoid both (a) missing the parse failure event (offset too \
         late) and (b) attributing prior unrelated failures to this \
         request (offset = 0)."
    );
}
