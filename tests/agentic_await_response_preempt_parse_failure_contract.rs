//! Source-level contract for Run 8 Pass #17's Add of the preemptive
//! `stdin_parse_failed` scan in `scripts/agentic/await-response.ts`.
//!
//! Before this pass, `cmd_rpc`'s parse-failure surfacing was post-hoc —
//! after the full `--timeout` elapsed, `session.sh` scanned the app.log
//! tail for a correlated `stdin_parse_failed` event and replaced the
//! generic timeout envelope with `parse_error`. Correct, but slow: a
//! malformed payload round-trip took the full `--timeout` (typically
//! 3000-5000ms).
//!
//! The preemptive variant runs in parallel with the typed-response poll
//! inside `await-response.ts`, short-circuiting with a `parse_error`
//! envelope + exit code 3 within one poll cycle (~50ms) of the parse
//! failure landing in app.log. Gated on the same charset
//! `[A-Za-z0-9_.:/-]` as Rust's `extract_request_id_lenient` — charset-
//! unsafe requestIds fall through to the shell post-hoc unscoped-grep
//! fallback (session.sh lines ~600-602).
//!
//! This contract pins the structural invariants so a future
//! "simplification" of `await-response.ts` that drops the preemptive
//! scan, unscopes the cid grep, or collapses exit code 3 back into 1
//! would fail loudly instead of silently regressing DX latency.

const AWAIT_RESPONSE_SRC: &str = include_str!("../scripts/agentic/await-response.ts");

#[test]
fn await_response_declares_parse_error_exit_code_three() {
    // Exit code 3 is stable contract — cmd_rpc callers (and future
    // programmatic consumers) MAY key on it to distinguish "parse error
    // surfaced preemptively" from "generic timeout" (exit 1) or
    // "infrastructure error" (exit 2). Collapsing back to exit 1 would
    // work behaviourally (the envelope still carries code:"parse_error")
    // but breaks the exit-code contract that distinguishes fast-fail
    // from slow-timeout for shell consumers.
    assert!(
        AWAIT_RESPONSE_SRC.contains("const PARSE_ERROR_EXIT_CODE = 3;"),
        "scripts/agentic/await-response.ts MUST declare \
         `const PARSE_ERROR_EXIT_CODE = 3;` as a top-level constant. \
         This is the stable exit code for preemptively-detected parse \
         failures. See audits/afk/log.md Run 8 Pass #17 for context."
    );
    assert!(
        AWAIT_RESPONSE_SRC.contains("process.exit(PARSE_ERROR_EXIT_CODE);"),
        "scripts/agentic/await-response.ts MUST exit with \
         `process.exit(PARSE_ERROR_EXIT_CODE)` after emitting the \
         preemptive parse_error envelope — NOT `process.exit(1)` (which \
         collides with the generic-timeout exit) or `process.exit(3)` \
         (which duplicates the constant's intent). The named constant \
         is what the contract pins."
    );
}

#[test]
fn await_response_charset_gates_preemptive_scan_like_session_sh() {
    // The preemptive scan MUST be gated on the same
    // `^[A-Za-z0-9_.:/-]+$` charset as Rust's extract_request_id_lenient
    // (src/stdin_commands/mod.rs) and session.sh's cmd_rpc charset gate
    // at line ~598. Rust routes charset-unsafe requestIds to
    // `cid=stdin:parse:<uuid>` instead of `cid=stdin:req:<id>` — the
    // scoped grep would never match, so preemptive detection would miss
    // and the await would time out. The shell post-hoc scan then
    // catches these via its unscoped-grep fallback. Without the charset
    // gate, we'd either (a) miss charset-unsafe parse failures silently
    // in the preemptive path, or (b) need an unscoped grep here that
    // cross-matches concurrent parse failures from other in-flight
    // sends — the exact cross-correlation race Pass #13 (Run 5) closed
    // on the shell side.
    assert!(
        AWAIT_RESPONSE_SRC.contains("const REQUEST_ID_CHARSET = /^[A-Za-z0-9_.:/-]+$/;"),
        "scripts/agentic/await-response.ts MUST declare \
         `const REQUEST_ID_CHARSET = /^[A-Za-z0-9_.:/-]+$/;` — mirrors \
         Rust's extract_request_id_lenient charset (src/stdin_commands/mod.rs) \
         AND session.sh cmd_rpc's charset gate. Desyncing any one of the \
         three sides reintroduces Run 5 Pass #16's charset-boundary bug."
    );
    assert!(
        AWAIT_RESPONSE_SRC.contains("REQUEST_ID_CHARSET.test(requestId)"),
        "scripts/agentic/await-response.ts MUST gate the preemptive scan \
         on `REQUEST_ID_CHARSET.test(requestId)`. Unscoped scanning for \
         charset-unsafe ids would race concurrent parse failures — the \
         shell post-hoc fallback handles those correctly via unscoped \
         grep with byte-window scoping (session.sh lines ~600-602)."
    );
}

#[test]
fn await_response_scoped_cid_marker_has_trailing_space() {
    // The scoped cid marker `cid=stdin:req:${requestId} ` MUST include
    // the trailing space to prevent prefix matches — e.g. requestId
    // `p17-get` must not match a line with `cid=stdin:req:p17-get-foo`.
    // This is the exact same scoping session.sh cmd_rpc uses at line
    // ~599. Dropping the trailing space silently reintroduces the
    // prefix-match anomaly Run 5 Pass #10 documented for `cmd_send`'s
    // happy-path grep.
    assert!(
        AWAIT_RESPONSE_SRC.contains("const cidMarker = `cid=stdin:req:${requestId} `;"),
        "scripts/agentic/await-response.ts MUST build its cid marker \
         as `cid=stdin:req:${{requestId}} ` with a trailing space. \
         Prefix matches without the space produce cross-request \
         cross-correlation — the same bug class Run 5 Pass #10 closed \
         on session.sh. Missing anchor: \
         `const cidMarker = \\`cid=stdin:req:${{requestId}} \\`;`."
    );
}

#[test]
fn await_response_matches_stdin_parse_failed_event_type_literal() {
    // The scan MUST match `event_type=stdin_parse_failed` specifically.
    // A looser match on `Failed to parse` prose would pick up unrelated
    // tracing lines (e.g. responses.ndjson parse failures, config.ts
    // parse failures) and falsely attribute them to this request. The
    // Rust-side tracing field is `event_type=stdin_parse_failed` emitted
    // from src/stdin_commands/mod.rs's Err-arm; this is the documented
    // stable contract name (see removed-docs §"Session send
    // parse receipts").
    assert!(
        AWAIT_RESPONSE_SRC.contains("\"event_type=stdin_parse_failed\""),
        "scripts/agentic/await-response.ts MUST match the exact literal \
         `event_type=stdin_parse_failed` in its preemptive scan. Looser \
         matches (e.g. `Failed to parse`) falsely correlate unrelated \
         log lines. Missing anchor: `line.includes(\"event_type=stdin_parse_failed\")`."
    );
}

#[test]
fn await_response_truncates_error_message_to_bounded_length() {
    // Serde error messages can embed attacker-controlled input (e.g. a
    // 500-char variant name). Unbounded error text blows out caller
    // stdin-pipe budgets and session.sh's `cut -c1-200` on the shell
    // side. Preemptive scan MUST mirror that 200-char bound so the
    // behaviour is symmetric across the two detection paths — a caller
    // seeing `parse_error` from preemptive-path vs post-hoc-path should
    // get the same truncation class.
    assert!(
        AWAIT_RESPONSE_SRC.contains("const ERROR_MSG_MAX_CHARS = 200;"),
        "scripts/agentic/await-response.ts MUST declare \
         `const ERROR_MSG_MAX_CHARS = 200;` — mirrors session.sh's \
         `cut -c1-200` cap on the post-hoc path. Different caps on the \
         two paths is a DX gotcha that silently changes error-text \
         length depending on which detection fired first."
    );
    assert!(
        AWAIT_RESPONSE_SRC.contains("errMsg.length > ERROR_MSG_MAX_CHARS"),
        "scripts/agentic/await-response.ts MUST truncate via \
         `errMsg.length > ERROR_MSG_MAX_CHARS` + `errMsg.substring(0, ERROR_MSG_MAX_CHARS)` \
         (NOT a hardcoded literal like `> 200` that diverges from the \
         constant). The named constant lets the cap evolve in one place."
    );
    assert!(
        AWAIT_RESPONSE_SRC.contains(r" at line \d+ column \d+.*$"),
        "scripts/agentic/await-response.ts MUST strip serde's \
         ` at line N column M` suffix via regex replace, symmetric with \
         session.sh's `sed -E 's/ at line [0-9]+ column [0-9]+.*$//'` \
         on the post-hoc path. Without stripping, the 200-char budget \
         gets consumed by diagnostic noise instead of the root error."
    );
}

#[test]
fn await_response_preempt_scan_runs_inside_poll_loop_before_typed_scan() {
    let loop_start = AWAIT_RESPONSE_SRC
        .find("while (Date.now() < deadline) {")
        .expect(
            "await-response.ts MUST retain `while (Date.now() < deadline) {` \
             as the poll-loop header.",
        );
    let loop_body = &AWAIT_RESPONSE_SRC[loop_start..];

    let preempt_pos = loop_body.find("if (charsetSafeRequestId").expect(
        "await-response.ts MUST guard the preemptive scan with \
         `if (charsetSafeRequestId` INSIDE the poll loop.",
    );
    let typed_scan_pos = loop_body
        .find("scanProtocolBus")
        .or_else(|| loop_body.find("scanLog"))
        .expect(
            "await-response.ts MUST retain the scan call inside \
         the poll loop for typed-response detection.",
        );

    assert!(
        preempt_pos < typed_scan_pos,
        "scripts/agentic/await-response.ts MUST run the preemptive \
         parse-failure scan BEFORE the typed-response scan within each \
         poll iteration. Reordering wastes up to POLL_INTERVAL (50ms) \
         of fast-fail latency. Found preempt at offset {preempt_pos} \
         but typed-scan at {typed_scan_pos} — the preemptive block was \
         moved after the typed scan.",
    );
}

#[test]
fn await_response_preempt_emits_parse_error_code_not_timeout() {
    // The envelope emitted by the preemptive path MUST use
    // `code: "parse_error"` — stable contract matching session.sh's
    // post-hoc envelope. Emitting `code: "parseError"` (camelCase,
    // which is the cmd_send --await-parse *value* on a different
    // field) would confuse programmatic callers that key on
    // `error.code`. Emitting `code: "timeout"` (the pre-preempt
    // fallback code on this same exit path) would silently kill the
    // distinction this pass introduced.
    assert!(
        AWAIT_RESPONSE_SRC.contains(
            r#"errorResult(
      sessionName,
      requestId,
      "parse_error","#
        ) || AWAIT_RESPONSE_SRC.contains(r#"errorResult(sessionName, requestId, "parse_error","#)
            || AWAIT_RESPONSE_SRC.contains(
                r#"errorResult(
        sessionName,
        requestId,
        "parse_error","#
            ),
        "scripts/agentic/await-response.ts MUST emit \
         `errorResult(sessionName, requestId, \"parse_error\", errMsg)` \
         in the preemptive path — NOT `\"parseError\"` (camelCase, \
         wrong field) and NOT `\"timeout\"` (generic fallback). The \
         `parse_error` code is stable contract shared with session.sh \
         cmd_rpc's post-hoc envelope."
    );
}
