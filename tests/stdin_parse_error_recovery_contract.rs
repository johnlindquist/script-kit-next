//! Source-level contract for the Run 2 Pass #56
//! `stdin-protocol-parse-error-recovery-contract` user story.
//!
//! Pass #56 live-verified on dev-watch pid 38095 that the stdin listener
//! loop survives parse errors of every shape and continues to dispatch
//! subsequent valid commands:
//!
//!   Send 1: `{"malformed":"nope-no-type-field"}`     → `stdin_parse_failed` (missing field `type`), loop continues
//!   Send 2: `this is not even json at all`             → `stdin_parse_failed` (expected ident), loop continues
//!   Send 3: `{"type":"unknownCommandType","requestId":"p56-u"}` → `stdin_parse_failed` (unknown variant), loop continues
//!   Send 4: `{"type":"listAutomationWindows","requestId":"p56-v1"}` → parsed + dispatched + valid response received ✓
//!
//! The listener at `start_stdin_listener` in `src/stdin_commands/mod.rs`
//! wraps `parse_stdin_command(trimmed)` in a `match`; the `Err(e) => { … }`
//! arm MUST:
//! 1. log `stdin_parse_failed` via `tracing::warn!` (so agents can detect it);
//! 2. NOT `break` — that would exit the listener thread and lose all
//!    subsequent stdin commands permanently until the app restarts;
//! 3. NOT `return` — same effect as `break` at the outer-loop level;
//! 4. NOT `panic!` / `unreachable!` — would crash the thread, same worst
//!    outcome as `break` plus an uglier crash.
//!
//! The oversized-line arm (`Ok(StdinLineRead::TooLong { … })`) shares the
//! same invariants for the same reason: a 20kB poorly-behaved script
//! output must not kill the listener.
//!
//! The read-error arm (`Err(e) => { ... break; }`) at the outer match IS
//! allowed to break — that's reached only on underlying IO failure (EOF,
//! broken pipe), where continuing to loop would busy-spin on the same
//! error forever.
//!
//! A refactor that "simplifies" the parse-error arm to
//!     `Err(e) => return Err(e),`
//! or
//!     `Err(_) => break,`
//! — perhaps to propagate errors "cleanly" — would silently break this
//! resilience. The app would appear healthy (no crash, no log spam) but
//! the automation surface would go dark after the first malformed line.
//! This contract catches that refactor before it ships.

const LISTENER_SRC: &str = include_str!("../src/stdin_commands/mod.rs");

/// Find the byte span of `pub fn start_stdin_listener` and return its
/// body as a slice, using brace-counting to locate the matching `}`.
fn start_stdin_listener_body<'a>() -> &'a str {
    let src: &str = LISTENER_SRC;
    let header_pos = src
        .find("pub fn start_stdin_listener(")
        .expect("src/stdin_commands/mod.rs: missing `pub fn start_stdin_listener(` — function may have been renamed; update this contract.");

    let open_rel = src[header_pos..]
        .find('{')
        .expect("no `{` after `pub fn start_stdin_listener(`");
    let open_abs = header_pos + open_rel;

    let mut depth: i32 = 0;
    let mut close_abs: Option<usize> = None;
    for (offset, &b) in src.as_bytes()[open_abs..].iter().enumerate() {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    close_abs = Some(open_abs + offset);
                    break;
                }
            }
            _ => {}
        }
    }
    let close_abs = close_abs.expect("no matching `}` for start_stdin_listener body");
    &src[open_abs..=close_abs]
}

/// Given a slice that starts at a match-arm header like
/// `Err(e) => {` (with a leading `{`), return the brace-balanced body
/// of the arm (the slice between the opening `{` and matching `}`).
fn balanced_block_body<'a>(src: &'a str, arm_header_start: usize) -> &'a str {
    let open_rel = src[arm_header_start..]
        .find('{')
        .unwrap_or_else(|| panic!("no `{{` after arm header at byte {arm_header_start}"));
    let open_abs = arm_header_start + open_rel;

    let mut depth: i32 = 0;
    let mut close_abs: Option<usize> = None;
    for (offset, &b) in src.as_bytes()[open_abs..].iter().enumerate() {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    close_abs = Some(open_abs + offset);
                    break;
                }
            }
            _ => {}
        }
    }
    let close_abs = close_abs
        .unwrap_or_else(|| panic!("no matching `}}` for arm block starting at byte {open_abs}"));
    &src[open_abs..=close_abs]
}

/// Extract the body of the parse-error arm: the `Err(e) => { … }` arm
/// inside the `match parse_stdin_command(trimmed) { … }` expression.
/// Returns the brace-balanced block between `Err(e) => {` and its
/// matching `}`.
fn parse_error_arm_body<'a>(listener_body: &'a str) -> &'a str {
    let parse_match_pos = listener_body
        .find("match parse_stdin_command(trimmed) {")
        .expect(
            "listener body missing `match parse_stdin_command(trimmed) {` — dispatcher shape may \
             have been restructured; update this contract.",
        );

    // Inside the match, find the first `Err(e) => {` — that's the parse
    // error arm (the outer `Err(e) => {` for read errors is outside this
    // inner match).
    let err_arm_rel = listener_body[parse_match_pos..]
        .find("Err(e) => {")
        .expect("parse match missing `Err(e) => {` arm");
    let err_arm_abs = parse_match_pos + err_arm_rel;

    balanced_block_body(listener_body, err_arm_abs)
}

/// Extract the body of the oversized-line arm:
/// `Ok(StdinLineRead::TooLong { … }) => { … }`.
///
/// The tricky bit: `balanced_block_body` starts searching for `{` from
/// the given position, but the `Ok(StdinLineRead::TooLong { raw, raw_len })`
/// pattern *itself* contains an opening `{` for the destructure. We must
/// skip past the destructure pattern's `{ … }` first and then locate the
/// `=> {` that opens the arm body proper.
fn too_long_arm_body<'a>(listener_body: &'a str) -> &'a str {
    let anchor = listener_body
        .find("Ok(StdinLineRead::TooLong {")
        .expect("listener body missing `Ok(StdinLineRead::TooLong {` arm");
    // Search for `=> {` after the anchor — this is the arm-body opener,
    // which lives *after* the destructure pattern's closing `}`.
    let arrow_rel = listener_body[anchor..]
        .find("=> {")
        .expect("TooLong arm missing `=> {` — arm header shape unexpected");
    let arrow_abs = anchor + arrow_rel;
    balanced_block_body(listener_body, arrow_abs)
}

#[test]
fn parse_error_arm_logs_and_does_not_break() {
    let body = start_stdin_listener_body();
    let arm = parse_error_arm_body(body);

    assert!(
        arm.contains("stdin_parse_failed"),
        "start_stdin_listener's parse-error arm does NOT log `stdin_parse_failed`:\n\n{arm}\n\n\
         Agents rely on this event name (via `grep stdin_parse_failed app.log` or structured \
         tracing subscribers) to detect when a malformed command hit the listener. Removing or \
         renaming the event breaks the observability surface. Pass #56 live-verified that 3 \
         distinct malformed payloads each produced one `stdin_parse_failed` line before the \
         loop continued to dispatch a valid `listAutomationWindows` command."
    );

    // The arm must NOT contain `break`, `return`, or any panic form.
    // Each of these would kill the listener thread on the first
    // malformed line, turning every typo in a script's protocol output
    // into a silent automation blackout.
    const FORBIDDEN: &[&str] = &["break", "return", "panic!(", "unreachable!(", "todo!("];
    for forbidden in FORBIDDEN {
        assert!(
            !arm.contains(forbidden),
            "start_stdin_listener's parse-error arm contains forbidden control-flow `{forbidden}`:\n\n\
             {arm}\n\n\
             The arm must fall through (log + continue) so the listener survives malformed \
             input. A `break` exits the listener thread permanently; `return` is equivalent at \
             this scope; `panic!` / `unreachable!` / `todo!` crash the thread. Any of these \
             turns every malformed line into a silent automation blackout — Pass #56 \
             live-verified that the loop recovers from 3 distinct parse-error kinds (missing \
             field, malformed JSON, unknown variant) and continues to dispatch subsequent \
             valid commands. Preserve that guarantee."
        );
    }
}

#[test]
fn too_long_arm_logs_and_does_not_break() {
    let body = start_stdin_listener_body();
    let arm = too_long_arm_body(body);

    assert!(
        arm.contains("stdin_command_too_large"),
        "start_stdin_listener's TooLong arm does NOT log `stdin_command_too_large`:\n\n{arm}\n\n\
         This event lets agents detect that a script sent a line exceeding \
         MAX_STDIN_COMMAND_BYTES (16 KiB). Silent skipping with no log would hide real bugs \
         where a script accidentally blasts huge payloads."
    );

    const FORBIDDEN: &[&str] = &["break", "return", "panic!(", "unreachable!(", "todo!("];
    for forbidden in FORBIDDEN {
        assert!(
            !arm.contains(forbidden),
            "start_stdin_listener's TooLong arm contains forbidden control-flow `{forbidden}`:\n\n\
             {arm}\n\n\
             Oversized input must be logged and skipped, not fatal. An existing unit test \
             (`test_read_stdin_line_bounded_skips_oversized_line_and_recovers`) exercises the \
             reader's own recovery; this contract guards the listener's handling of that \
             already-recovered state."
        );
    }
}

#[test]
fn parse_error_arm_scopes_correlation_id_on_request_id() {
    // Pass #13 Run 5 fix for the concurrent sad-path cross-correlation
    // anomaly filed by Pass #12. When --await-parse callers fire 5
    // concurrent malformed payloads, their parseError envelopes
    // previously all reported whichever verb won the log-tail race
    // because `stdin_parse_failed` spans carried a synthetic UUID
    // correlation_id (`stdin:parse:<uuid>`) instead of the requestId
    // scope the happy-path uses. The shell-side grep had no per-send
    // selector to latch onto.
    //
    // The fix: lenient pre-deserialization extract of `requestId`
    // from the raw line (`extract_request_id_lenient`, same sed +
    // charset pattern as session.sh), used to build
    // `correlation_id = "stdin:req:<id>"` when extractable. This must
    // appear in the Err(e) arm for the shell-side scoped sad-path grep
    // (pinned by session_send_await_parse_contract::cmd_send_scopes_sad_path_grep_on_request_id)
    // to match anything under concurrency.
    //
    // Refactor threat: a contributor "simplifying" the Err arm back to
    // `format!("stdin:parse:{}", Uuid::new_v4())` unconditionally —
    // perhaps because "correlation_id should always be unique" —
    // would silently re-open the Pass #12 gap. This test pins the
    // lenient-extract invocation so that simplification fails at
    // build time before landing.
    let body = start_stdin_listener_body();
    let arm = parse_error_arm_body(body);

    assert!(
        arm.contains("extract_request_id_lenient(trimmed)"),
        "start_stdin_listener's parse-error arm does NOT call \
         `extract_request_id_lenient(trimmed)`:\n\n{arm}\n\n\
         The correlation_id must be built by trying the lenient \
         requestId extract first so sad-path spans carry `stdin:req:<id>` \
         when the malformed payload still contains a valid requestId. \
         Without this, the shell-side sad-path scoped grep \
         (`grep -F -- \"cid=stdin:req:${{req_id}} \" | grep -m1 \
         'event_type=stdin_parse_failed'`) matches zero lines under \
         concurrency and every parse failure cross-correlates or times \
         out. Pass #12 reproduced this with 5 parallel distinct-verb \
         distinct-requestId sad-path sends; Pass #13 fixed it here."
    );
    assert!(
        arm.contains("stdin:req:{}"),
        "start_stdin_listener's parse-error arm does NOT format the \
         extracted id into the canonical `stdin:req:<id>` \
         correlation_id shape:\n\n{arm}\n\n\
         The shell-side scoped grep matches on `cid=stdin:req:<id> ` \
         (with trailing space) — if the listener emits a different \
         prefix (e.g. `stdin:parse-req:<id>` or `req:<id>`), the grep \
         misses the event and sad-path correlation breaks. Keep the \
         format `stdin:req:{{}}` exactly symmetric with the happy-path \
         correlation_id built a few lines above."
    );
    assert!(
        arm.contains("stdin:parse:{}"),
        "start_stdin_listener's parse-error arm does NOT preserve the \
         `stdin:parse:<uuid>` fallback for payloads lacking an \
         extractable requestId:\n\n{arm}\n\n\
         The fallback is required so totally non-JSON input (which \
         can't match the `\"requestId\":\"...\"` structure) still \
         carries a non-empty correlation_id for existing offset-first \
         callers. Removing the fallback would produce empty \
         correlation_id strings for no-requestId payloads, which in \
         turn would make the happy-path grep's trailing-space guard \
         match bogus prefixes."
    );
}

#[test]
fn extract_request_id_lenient_module_helper_exists() {
    // The lenient requestId extractor is the Rust mirror of
    // scripts/agentic/session.sh cmd_send's sed+charset pattern. The
    // two sides MUST agree on which requestIds survive the charset
    // guard — if Rust accepts a requestId that the shell rejects (or
    // vice-versa), one side emits `stdin:req:<id>` while the other
    // scopes on legacy offset-first grep, and the receipt either
    // misses the event or cross-correlates under concurrency.
    //
    // This test pins the function's existence and the charset at
    // source level. A refactor that renames the function, changes
    // the charset (e.g. adding `+`, `=`, or `@` to the allowed set
    // without mirroring on the shell side) would silently desync
    // the two sides. The test failing points the next contributor
    // at both files.
    assert!(
        LISTENER_SRC.contains("fn extract_request_id_lenient(line: &str) -> Option<String>"),
        "src/stdin_commands/mod.rs: missing `fn extract_request_id_lenient(line: &str) -> Option<String>`. \
         This helper scopes parse-failed correlation IDs on requestId \
         so the shell-side sad-path scoped grep can correlate \
         concurrent malformed sends. Renamed? Update this contract \
         AND scripts/agentic/session.sh's sed+charset pattern — the \
         two sides must stay symmetric."
    );
    // Charset mirror: the Rust helper must accept exactly the same
    // character class as the shell validator `[A-Za-z0-9_.:/-]`. The
    // Rust expression is
    //     c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ':' | '/')
    // — pin both halves so a well-meaning "extend allowed chars"
    // refactor desyncs loudly.
    assert!(
        LISTENER_SRC.contains("c.is_ascii_alphanumeric()"),
        "src/stdin_commands/mod.rs: extract_request_id_lenient must \
         gate on `c.is_ascii_alphanumeric()` to mirror the `[A-Za-z0-9]` \
         half of the shell charset regex. Removing this would let the \
         Rust side accept non-alphanumeric payloads the shell would \
         reject."
    );
    assert!(
        LISTENER_SRC.contains("matches!(c, '_' | '-' | '.' | ':' | '/')"),
        "src/stdin_commands/mod.rs: extract_request_id_lenient must \
         allow exactly `_` `-` `.` `:` `/` as the special-char set to \
         mirror the shell charset regex `[_.:/-]`. Adding or removing \
         characters here without mirroring the change in \
         scripts/agentic/session.sh's `[[ \"$req_id\" =~ \
         ^[A-Za-z0-9_.:/-]+$ ]]` desyncs the two sides and opens \
         either grep-injection (Rust accepts metachars shell would \
         reject) or silent undercorrelation (shell accepts chars Rust \
         rejects, so Rust emits the UUID fallback and shell's scoped \
         grep finds nothing)."
    );
}

#[test]
fn read_error_arm_is_allowed_to_break() {
    // Defense-in-depth: confirm the OUTER read-error arm (the one after
    // the TooLong arm) DOES contain `break` — if a future refactor
    // "harmonized" all three arms to never break, the listener would
    // busy-spin on a broken stdin forever (each iteration re-reads the
    // same error). This positive-assertion contract is the inverse of
    // the two above.
    let body = start_stdin_listener_body();

    // The read-error arm lives AFTER the `Ok(StdinLineRead::TooLong {`
    // arm and immediately before the outer loop's closing `}`. Use
    // `stdin_read_error` as the anchor since it's unique to this arm.
    let anchor_pos = body.find("stdin_read_error").expect(
        "listener body missing `stdin_read_error` event — read-error arm may have been \
         restructured; update this contract.",
    );

    // The `break;` must appear within 400 bytes after the anchor (the
    // arm body is short: log + break). A looser cap would still catch
    // deletion, but 400 is tight enough to catch refactors that move
    // the break out of the arm.
    let tail = &body[anchor_pos..(anchor_pos + 400).min(body.len())];
    assert!(
        tail.contains("break;"),
        "listener's read-error arm does NOT contain `break;` within 400 bytes of the \
         `stdin_read_error` anchor:\n\n{tail}\n\n\
         This arm handles IO-level read failures (EOF, broken pipe) from stdin. Continuing to \
         loop on these errors would busy-spin forever, since the same error recurs on the next \
         `read_stdin_line_bounded` call. `break` is the ONLY correct response. A refactor that \
         removed the `break` to match the parse/too-long arms' recovery behavior would be \
         incorrect: those arms recover because the input was merely malformed; this arm must \
         exit because the input stream itself is broken."
    );
}
