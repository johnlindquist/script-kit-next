//! Source-level contract test for Run 9 Pass #1 — extension of the
//! `cmd_rpc` requestId validator in `scripts/agentic/session.sh`.
//!
//! Background (Run 8 Pass #24 attacker probe,
//! `[?] attacker-stdin-requestid-nul-newline-match-loss`): RPC payloads
//! carrying a `requestId` with JSON `\u0000`, `\n`, or literal control
//! characters produced a silent `timeout` envelope with no indication
//! that the send never round-tripped. Root cause: the sender-side
//! `sed -nE 's/.../([^"]+)/p'` extractor keeps the bytes literal
//! (backslash + n stays 2 chars) while the app's serde JSON parser
//! decodes the escapes, so the two never correlate in
//! `responses.ndjson`.
//!
//! Acceptance option (a) from the anomaly menu: reject at ingest with
//! a stable error code (`invalid_request_id_charset`) before writing to
//! the FIFO, so the caller gets an immediate actionable error instead
//! of a misleading timeout.
//!
//! This pin defends the specific refactor where a contributor "unifies"
//! the RPC validation by deduping the sed extractor and the gate into a
//! single helper — a plausible cleanup that would silently drop the
//! early reject branch. The gate lives inside `cmd_rpc`'s body,
//! BEFORE the session-alive checks and the FIFO write.

const SESSION_SH: &str = include_str!("../scripts/agentic/session.sh");

fn cmd_rpc_body() -> &'static str {
    // Anchor on the cmd_rpc signature — extract the function body up to
    // the next top-level `}` so tests don't pick up stray matches in
    // cmd_send or unrelated helpers. Shape mirrors
    // tests/session_rpc_parse_error_surface_contract.rs.
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
fn cmd_rpc_rejects_backslash_and_control_char_request_ids_before_fifo_write() {
    // The gate MUST test both conditions — backslash presence AND
    // [[:cntrl:]] — because the two cover disjoint failure classes:
    // backslash catches JSON-encoded escapes (\u0000, \n, \t) as
    // sed-extracted literals; [[:cntrl:]] catches raw unescaped control
    // bytes that survived extraction. Dropping either half leaves a
    // silent-timeout hole.
    let body = cmd_rpc_body();
    assert!(
        body.contains(r#"[[ "$request_id" == *'\'* ]]"#),
        "scripts/agentic/session.sh::cmd_rpc MUST test for a literal \
         backslash in $request_id. Missing anchor: \
         `[[ \"$request_id\" == *'\\\\'* ]]`. Without it, requestIds \
         like `\\u0000nul` or `line1\\nline2` pass the early gate and \
         silently time out."
    );
    assert!(
        body.contains(r#"[[ "$request_id" =~ [[:cntrl:]] ]]"#),
        "scripts/agentic/session.sh::cmd_rpc MUST test $request_id \
         against [[:cntrl:]]. Missing anchor: \
         `[[ \"$request_id\" =~ [[:cntrl:]] ]]`. Without it, raw \
         unescaped control bytes (tab, vertical-tab, etc.) slip past \
         the backslash check."
    );
}

#[test]
fn cmd_rpc_emits_invalid_request_id_charset_stable_code() {
    // The rejection envelope's error.code MUST be
    // `invalid_request_id_charset` — NOT `missing_request_id` (which
    // already covers the empty-string case and would confuse callers)
    // and NOT `parse_error` (which is the app-rejected code, not the
    // sender-side pre-validation code). Programmatic callers key on
    // `error.code` to decide whether to retry with a corrected id.
    let body = cmd_rpc_body();
    assert!(
        body.contains(r#"json_error "invalid_request_id_charset""#),
        "scripts/agentic/session.sh::cmd_rpc MUST emit \
         `invalid_request_id_charset` as the stable error code when the \
         requestId contains control chars or backslashes. Using any \
         other code confuses programmatic callers that distinguish \
         missing vs. invalid vs. app-rejected."
    );
}

#[test]
fn cmd_rpc_charset_gate_precedes_session_alive_checks() {
    // The gate MUST run BEFORE the FIFO existence / app-alive /
    // forwarder-alive checks. Putting it after those would mean a dead
    // session masks a caller bug with a `no_session` error instead of
    // the actionable `invalid_request_id_charset`. Anchor on the
    // relative position of the gate vs. `session_dir` resolution.
    let body = cmd_rpc_body();
    let gate_idx = body
        .find(r#"json_error "invalid_request_id_charset""#)
        .expect("invalid_request_id_charset gate must exist");
    let session_dir_idx = body
        .find(r#"sdir="$(session_dir "$name")""#)
        .expect("cmd_rpc must resolve session_dir");
    assert!(
        gate_idx < session_dir_idx,
        "scripts/agentic/session.sh::cmd_rpc charset gate (byte offset \
         {gate_idx}) MUST precede session_dir resolution (byte offset \
         {session_dir_idx}). A misordered gate would let a dead session \
         mask a caller bug."
    );
}
