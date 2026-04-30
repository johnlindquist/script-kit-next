//! Source-level contract test for Run 5 Pass #7 of the
//! `tool-session-send-parse-receipt` user story filed by Run 4 Pass #8's
//! attacker probe.
//!
//! Pass #8 Run 4 discovered that `scripts/agentic/session.sh send`
//! returned `{status:"ok",sent:true}` whether or not the running binary
//! successfully parsed the command. A probe sending `builtinId` instead
//! of `name` on `triggerBuiltin` saw `stdin_parse_failed` in app.log
//! but `sent:true` in the RPC envelope — every malformed command was
//! silently dropped from the automation surface.
//!
//! Pass #7 Run 5 closed this by adding a `--await-parse [--timeout MS]`
//! flag to `cmd_send`. When present, the flag tails app.log from the
//! pre-send offset for the next `stdin_command_parsed` or
//! `stdin_parse_failed` event (stdin is line-serialized in the app, so
//! exactly one parse event corresponds to one send) and returns a
//! structured envelope with `parseOutcome: parsed | parseError | timeout`.
//!
//! This contract pins the key invariants at source level. A refactor
//! that "simplifies" session.sh by removing the `--await-parse` branch,
//! renaming the outcome values, or collapsing the parsed+parseError
//! cases would silently re-introduce the silent-drop gap. Without this
//! pin, breakage would only surface when a human (or attacker probe)
//! noticed that automation claimed sent:true on a dropped payload.

const SESSION_SH: &str = include_str!("../scripts/agentic/session.sh");

#[test]
fn cmd_send_parses_await_parse_flag() {
    // The `--await-parse` flag must be recognized in cmd_send's argument
    // loop. Without this, callers passing the flag would see it silently
    // ignored (back to fire-and-forget behavior) — the worst possible
    // outcome because the envelope still says sent:true.
    assert!(
        SESSION_SH.contains("--await-parse) await_parse=\"1\"; shift ;;"),
        "scripts/agentic/session.sh: cmd_send must parse `--await-parse` \
         into a local `await_parse` flag. Removing this case falls the \
         flag through to the `*)` catch-all (silent ignore) and turns \
         every awaited send back into fire-and-forget — exactly the bug \
         Pass #8 Run 4 attacker probe exposed. See \
         tool-session-send-parse-receipt in audits/afk/stories.md."
    );
    assert!(
        SESSION_SH.contains("--timeout)     timeout_ms=\"${2:-2000}\"; shift 2 ;;"),
        "scripts/agentic/session.sh: cmd_send must accept `--timeout MS` \
         alongside `--await-parse` with a 2000ms default. Callers need \
         the escape hatch when the app is known slow to emit parse \
         events (startup, heavy filter computation). Dropping the flag \
         hard-codes 2s which is insufficient for startup races."
    );
}

#[test]
fn cmd_send_records_log_offset_before_fifo_write() {
    // The ordering invariant: read `wc -c < app.log` BEFORE writing the
    // payload to the input FIFO. If the offset is recorded AFTER the
    // send, a fast-emitting parse event could be missed (race window ~ms).
    // This test pins the ordering by requiring the offset-record block
    // to appear textually before the FIFO write block in cmd_send.
    //
    // Refactor threat: a contributor consolidating the pid/forwarder
    // alive checks might move the FIFO write earlier "for readability"
    // and introduce the race.
    let offset_marker = "start_offset=\"$(wc -c < \"$log_path\" | tr -d ' ')\"";
    let fifo_marker = "printf '%s\\n' \"$cmd\" > \"$input_fifo\"";
    let offset_pos = SESSION_SH.find(offset_marker).expect(
        "scripts/agentic/session.sh: cmd_send must record app.log byte \
         offset via `wc -c < \"$log_path\" | tr -d ' '` before sending. \
         Without a pre-send offset, the polling scan may find a stale \
         event from an earlier send. Pass #7 Run 5 verified this \
         receipt-correlation protocol against a running binary with 2 \
         parse outcomes.",
    );
    let fifo_pos = SESSION_SH.find(fifo_marker).expect(
        "scripts/agentic/session.sh: cmd_send must write the payload via \
         `printf '%s\\n' \"$cmd\" > \"$input_fifo\"`. This test pins the \
         payload-write shape so the ordering invariant below can be \
         checked at its position.",
    );
    assert!(
        offset_pos < fifo_pos,
        "scripts/agentic/session.sh: cmd_send records app.log offset \
         AFTER the FIFO write (offset_pos={offset_pos} >= \
         fifo_pos={fifo_pos}). This opens a race window where a fast \
         parse event could be missed by the poll loop. Move the \
         `start_offset=\"$(wc -c < ...)\"` block above the `printf ... > \
         $input_fifo` block."
    );
}

#[test]
fn cmd_send_await_parse_emits_three_parse_outcomes() {
    // When --await-parse is active, cmd_send MUST return one of exactly
    // three parseOutcome values: "parsed", "parseError", "timeout".
    // These names are the stable contract automation depends on; a
    // rename (e.g. "success" / "failed" / "noEvent") would break every
    // existing caller and the change would not be visible in diff until
    // callers start failing cryptically.
    //
    // Refactor threat: a contributor "normalizing" outcome names to
    // match the top-level status field convention (ok/error) would
    // break the domain-specific vocabulary that distinguishes
    // parsing-success vs command-execution-success.
    const REQUIRED_OUTCOMES: &[&str] = &[
        "parseOutcome:\\\"parsed\\\"",
        "parseOutcome:\\\"parseError\\\"",
        "parseOutcome:\\\"timeout\\\"",
    ];
    for outcome in REQUIRED_OUTCOMES {
        assert!(
            SESSION_SH.contains(outcome),
            "scripts/agentic/session.sh: cmd_send --await-parse must \
             emit `{outcome}` in its json_envelope calls. Removing or \
             renaming this outcome breaks callers that match on \
             parseOutcome. The three values are the stable surface \
             contract; add new ones alongside, don't rename existing \
             ones."
        );
    }
    // The parsed path must also surface commandType so callers can
    // confirm the parsed shape matches their intent (e.g. a send
    // intended as `getState` that parses as something else would still
    // pass the parseOutcome check but fail the commandType one).
    assert!(
        SESSION_SH.contains("commandType:\\\"${command_type:-unknown}\\\""),
        "scripts/agentic/session.sh: parsed-path envelope must include \
         `commandType:\\\"${{command_type:-unknown}}\\\"` so callers can \
         verify the parsed variant identity, not just that *some* \
         parse succeeded. The `:-unknown` fallback handles the edge \
         case where sed extraction fails on a log-format change."
    );
}

#[test]
fn cmd_send_scopes_happy_path_grep_on_request_id() {
    // Pass #8 Run 5 attacker probe showed that 5 concurrent
    // --await-parse sends with DISTINCT commandTypes all returned
    // `commandType:"show"` because each send's grep used
    // `grep -m1 'event_type=stdin_command_parsed'` against the same
    // pre-send offset — they latched onto whichever parse event the
    // Rust listener processed first.
    //
    // Pass #10 closes this by extracting the sent requestId and
    // scoping the happy-path grep on
    // `cid=stdin:req:<request_id> ` (Rust emits this format from
    // `src/stdin_commands/mod.rs::start_stdin_listener` via
    // `tracing::info!(correlation_id = %correlation_id)`). The
    // correlation_id format `stdin:req:<request_id>` comes from
    // mod.rs:786-790 where request_id is preferred over a synthetic
    // UUID when the command's serde-extracted requestId is present.
    //
    // The charset guard `^[A-Za-z0-9_.:/-]+$` defends the grep
    // invocation from attacker-controlled requestIds that could
    // contain whitespace, newlines, or grep/shell metachars —
    // anything outside the conservative charset falls back to the
    // legacy offset-first grep (single-caller precondition).
    //
    // Refactor threat: a contributor "simplifying" the poll loop by
    // removing the req_id branch would silently re-open the
    // cross-correlation gap Pass #8 filed. This test pins the
    // substrings so the simplification fails at build time.
    let extract_marker = "sed -nE 's/.*\"requestId\"[[:space:]]*:[[:space:]]*\"([^\"]*)\".*/\\1/p'";
    assert!(
        SESSION_SH.contains(extract_marker),
        "scripts/agentic/session.sh: cmd_send must extract requestId \
         from the sent payload via the exact sed pattern that \
         matches a JSON `\"requestId\":\"...\"` value. Removing this \
         extraction drops cmd_send back to offset-first grep, which \
         cross-correlates under concurrency (Pass #8 anomaly). \
         Expected:\n\n{extract_marker}\n"
    );
    let charset_marker = "[[ \"$req_id\" =~ ^[A-Za-z0-9_.:/-]+$ ]]";
    assert!(
        SESSION_SH.contains(charset_marker),
        "scripts/agentic/session.sh: cmd_send must validate the \
         extracted requestId against the conservative charset \
         `^[A-Za-z0-9_.:/-]+$` before using it in the grep \
         invocation. A requestId with whitespace, newlines, or \
         grep/shell metachars must fall back to legacy offset-grep. \
         Expected:\n\n{charset_marker}\n"
    );
    let scoped_grep_marker =
        "grep -F -- \"cid=stdin:req:${req_id} \" | grep -m1 'event_type=stdin_command_parsed'";
    assert!(
        SESSION_SH.contains(scoped_grep_marker),
        "scripts/agentic/session.sh: when req_id is set, cmd_send \
         must scope the happy-path grep on \
         `cid=stdin:req:<req_id> ` (with trailing space to avoid \
         prefix matches like `p10-F-1` colliding with `p10-F-10`). \
         Use `grep -F --` for fixed-string matching and arg-list \
         terminator so the req_id cannot be interpreted as a grep \
         flag. Expected:\n\n{scoped_grep_marker}\n"
    );
}

#[test]
fn cmd_send_scopes_sad_path_grep_on_request_id() {
    // Pass #12 Run 5 attacker probe (phase C) reproduced the sad-path
    // twin of Pass #8's happy-path cross-correlation anomaly: 5 parallel
    // `--await-parse` sends with DISTINCT malformed verbs (nonExistentVerb1
    // … nonExistentVerb5) and DISTINCT requestIds (p12-C-1 … p12-C-5)
    // all returned the SAME `parseError` text — whichever verb won the
    // log-tail race. Each envelope carried its correct requestId at the
    // shell level, but the `error` string was cross-attributed 4/5 to
    // the wrong caller.
    //
    // Pass #13 closes this symmetrically with the happy-path fix: the
    // Rust listener now emits
    //     correlation_id = "stdin:req:<request_id>"
    // on `stdin_parse_failed` spans too (see
    // src/stdin_commands/mod.rs::extract_request_id_lenient), and
    // cmd_send scopes the sad-path grep on
    // `cid=stdin:req:${req_id} ` just like the happy-path grep does.
    //
    // Refactor threat: a contributor "unifying" the happy and sad grep
    // blocks into a single loop that only scopes on the happy event
    // type would re-open the Pass #12 gap. This test pins the scoped
    // sad-path grep invocation so the simplification fails at build
    // time.
    let scoped_marker =
        "grep -F -- \"cid=stdin:req:${req_id} \" | grep -m1 'event_type=stdin_parse_failed'";
    assert!(
        SESSION_SH.contains(scoped_marker),
        "scripts/agentic/session.sh: when req_id is set, cmd_send must \
         scope the sad-path grep on `cid=stdin:req:<req_id> ` exactly \
         like the happy-path grep, using `grep -F --` for fixed-string \
         matching with trailing space to avoid prefix collisions \
         (e.g. `p12-C-1` vs `p12-C-10`). Without the scoped sad-path \
         grep, 5 concurrent malformed sends cross-attribute their \
         error text to the race winner. Expected:\n\n{scoped_marker}\n"
    );
    // The legacy unscoped grep must still appear as the req_id-empty
    // fallback — this preserves the single-caller precondition for
    // payloads that lack a structurally-valid requestId (e.g. totally
    // non-JSON input where even the `\"requestId\":\"...\"` pattern
    // doesn't appear). Removing this fallback would turn every
    // no-requestId parse failure into a timeout.
    let fallback_marker = "grep -m1 'event_type=stdin_parse_failed'";
    assert!(
        SESSION_SH.contains(fallback_marker),
        "scripts/agentic/session.sh: the legacy unscoped sad-path grep \
         `{fallback_marker}` must remain as the req_id-empty fallback \
         branch. Without it, a malformed payload with no extractable \
         requestId would never match and cmd_send would time out on \
         every parse failure — a regression the scoped-grep fix must \
         not introduce."
    );
}

#[test]
fn cmd_send_rejects_non_numeric_timeout() {
    // Pass #8 Run 5 attacker probe found `--timeout abc` silently
    // terminates cmd_send before json_envelope runs: `set -euo pipefail`
    // at session.sh:26 + `local deadline_ms=$(( now_ms + timeout_ms ))`
    // at the deadline computation below treats `abc` as an unbound
    // variable under `set -u`, `set -e` kills the function with no
    // JSON output. Violates: "session.sh send ALWAYS returns a JSON
    // envelope."
    //
    // Pass #9 closed this by validating `timeout_ms` before the
    // arithmetic path. This test pins the regex check so a future
    // "simplification" that drops the validation re-opens the silent-
    // drop gap that ANOTHER contributor already hit.
    //
    // Refactor threat: a contributor consolidating the flag-parse
    // block into a generic `parse_flags` helper might forget to carry
    // the validation over; without this test, the "abc crash" anomaly
    // filed in audits/afk/stories.md as
    // `session-send-await-parse-timeout-non-numeric` would re-surface
    // in a later attacker probe and cost another fix pass.
    let regex_marker = "if [ -n \"$await_parse\" ] && ! [[ \"$timeout_ms\" =~ ^[0-9]+$ ]]; then";
    assert!(
        SESSION_SH.contains(regex_marker),
        "scripts/agentic/session.sh: cmd_send must validate that \
         `timeout_ms` is a non-negative integer regex (`^[0-9]+$`) \
         before reaching the arithmetic `$(( now_ms + timeout_ms ))` \
         block. Without this guard, a non-numeric --timeout value \
         triggers `set -u` unbound-variable and silently kills the \
         function. Expected guard:\n\n{regex_marker}\n"
    );
    let error_envelope_marker =
        "json_error \"invalid_timeout\" \"--timeout must be a non-negative integer; got: ${timeout_ms}\"";
    assert!(
        SESSION_SH.contains(error_envelope_marker),
        "scripts/agentic/session.sh: on non-numeric --timeout, \
         cmd_send MUST emit a structured `invalid_timeout` error \
         envelope via `json_error` and `return 1`. The error code \
         `invalid_timeout` is stable contract — callers may match \
         on `.error.code == \"invalid_timeout\"` to distinguish \
         bad-input from other session failures. Renaming the code \
         breaks those callers. Expected line:\n\n{error_envelope_marker}\n"
    );
}

#[test]
fn cmd_send_rejects_flag_as_command() {
    // Pass #20 Run 8 attacker probe (2026-04-19T01:12Z, commit d3a15cefc)
    // reproduced a silent swallow: `send SESSION --await-parse JSON` (flag
    // BEFORE the JSON payload — an easy arg-order typo) bound
    // `cmd="--await-parse"` via `local cmd="${2:-}"`, wrote the literal
    // 13-byte string `--await-parse\n` to the input FIFO, and returned
    // `{sent:true}` with no warning. The stdin parser logged six
    // consecutive `stdin_parse_failed line_len=13 error="invalid number
    // at line 1 column 2"` entries — one per wrong-order send — but the
    // caller saw no indication of failure.
    //
    // Pass #21 Run 8 closed this by guarding cmd_send with a shape-check:
    // if the positional CMD starts with `--`, return
    // `{status:"error", code:"flag_as_command"}` before touching the FIFO.
    // This test pins the guard so a future "simplification" that drops
    // the check re-opens the silent-drop gap.
    //
    // Refactor threat: a contributor consolidating the `missing_command`
    // and `invalid_timeout` guards into a generic `validate_args` helper
    // might drop the `--*` shape-check as "premature validation";
    // without this test, the anomaly filed as
    // `attacker-session-send-argorder-swallow` would re-surface on the
    // next attacker pass that makes the same arg-order typo.
    let guard_marker = "if [[ \"$cmd\" == --* ]]; then";
    assert!(
        SESSION_SH.contains(guard_marker),
        "scripts/agentic/session.sh: cmd_send MUST reject CMD values that \
         start with `--` (flag-as-command arg-order typo). Without this \
         guard, `send SESSION --flag JSON` silently writes the flag \
         string to stdin, the parser logs `stdin_parse_failed`, and the \
         caller sees only `{{sent:true}}`. Expected guard line:\n\n{guard_marker}\n"
    );
    let error_envelope_marker = "json_error \"flag_as_command\"";
    assert!(
        SESSION_SH.contains(error_envelope_marker),
        "scripts/agentic/session.sh: on flag-as-command, cmd_send MUST \
         emit a structured `flag_as_command` error envelope via \
         `json_error` and `return 1`. The error code `flag_as_command` \
         is stable contract — callers may match on \
         `.error.code == \"flag_as_command\"` to distinguish arg-order \
         typos from other session failures. Renaming the code breaks \
         those callers. Expected substring:\n\n{error_envelope_marker}\n"
    );
    // Ordering invariant: the flag-as-command guard must appear AFTER
    // the missing_command check (so empty CMD still errors with the
    // missing_command code, not flag_as_command) and BEFORE the FIFO
    // write (so we never mangle the input pipe with a flag string).
    let missing_idx = SESSION_SH
        .find("json_error \"missing_command\"")
        .expect("missing_command guard must exist");
    let flag_idx = SESSION_SH
        .find(guard_marker)
        .expect("flag_as_command guard must exist");
    let fifo_idx = SESSION_SH
        .find("printf '%s\\n' \"$cmd\" > \"$input_fifo\"")
        .expect("FIFO write site must exist");
    assert!(
        missing_idx < flag_idx,
        "scripts/agentic/session.sh: flag_as_command guard must appear \
         AFTER the missing_command check. Otherwise an empty CMD (which \
         trivially satisfies `$cmd == --*` as false) would correctly \
         hit missing_command — but a refactor that inverts the order \
         and puts flag_as_command first would emit the wrong error code \
         for the empty-CMD case. missing_idx={missing_idx} flag_idx={flag_idx}"
    );
    assert!(
        flag_idx < fifo_idx,
        "scripts/agentic/session.sh: flag_as_command guard must appear \
         BEFORE the FIFO write (`printf '%s\\\\n' \"$cmd\" > \"$input_fifo\"`). \
         Otherwise the guard runs too late — the flag string has already \
         been written to stdin and the parser has already emitted \
         `stdin_parse_failed`. flag_idx={flag_idx} fifo_idx={fifo_idx}"
    );
}

#[test]
fn cmd_send_without_await_parse_stays_fire_and_forget() {
    // Backwards compatibility: existing callers that don't pass
    // --await-parse MUST get exactly the old envelope
    // `{status:"ok",session:...,sent:true}` with NO parseOutcome field.
    // This pins the return-on-no-flag branch so a well-meaning refactor
    // ("why not always await?") can't silently slow down every
    // fire-and-forget call from the thousands of places that use them.
    //
    // The exact shape `if [ -z "$await_parse" ]; then ... return 0; fi`
    // must appear after the FIFO write and before the poll loop.
    let fire_and_forget_marker = "if [ -z \"$await_parse\" ]; then\n    \
                                  json_envelope \"ok\" \"session:\\\"${name}\\\"\" \"sent:true\"\n    return 0\n  fi";
    assert!(
        SESSION_SH.contains(fire_and_forget_marker),
        "scripts/agentic/session.sh: cmd_send must short-circuit to the \
         legacy `sent:true` envelope (no parseOutcome) when \
         `--await-parse` is absent. Without this short-circuit, every \
         bare `session.sh send` would wait up to timeout_ms for a parse \
         event, slowing every existing caller from ~0ms to ~50–2000ms. \
         Expected block:\n\n{fire_and_forget_marker}\n"
    );
}
