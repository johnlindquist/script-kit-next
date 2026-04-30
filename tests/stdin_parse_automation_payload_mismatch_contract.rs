//! Source-level structural contract for Run 8 Pass #10
//! `stdin-parse-dispatch-structural-pin-against-if-let-ok-collapse`.
//!
//! Pass #9 (SHA `806db738e`) converted `parse_stdin_command`'s
//! ExternalCommand branch from `if let Ok(command) = from_value(...)` to
//! a `match` that captures the error (`Err(err) => err`) AND added a
//! short-circuit that surfaces an `automation_payload_mismatch` error
//! when the caller's `type` names a verb in `EXTERNAL_COMMAND_VERBS`.
//! Three behavioural unit tests (inside `src/stdin_commands/mod.rs`)
//! pin the *output* of that dispatch.
//!
//! This contract pins the *structure*. A contributor who "simplifies"
//! the dispatch back to `if let Ok(command) = from_value { return … }`
//! and drops the `EXTERNAL_COMMAND_VERBS` short-circuit would silently
//! reintroduce the Pass #8 bug: a known automation verb with the wrong
//! payload field would fall through to the SDK-Message fallback, whose
//! serde error mentions the wrong vocabulary
//! (`hello`, `arg`, `submit`, `setInput`) instead of the real
//! field-level diagnostic.
//!
//! Behavioural tests do NOT catch this: a refactor that uses
//! `if let Ok` AND also manually re-parses inside the Err branch
//! could pass the same assertions while diverging from the committed
//! structural contract. The grep here is the belt to the behavioural
//! suspenders.
//!
//! The three structural anchors pinned below MUST all appear inside
//! the body of `fn parse_stdin_command` in `src/stdin_commands/mod.rs`:
//!   (1) `let ext_err = match serde_json::from_value::<ExternalCommand>(`
//!       — captures the error with a named binding.
//!   (2) `Err(err) => err,`
//!       — the Err arm yields the error as a value (not `?`, not
//!       `return Err(_)`, not a panic).
//!   (3) `automation_payload_mismatch:` literal inside an
//!       `EXTERNAL_COMMAND_VERBS` short-circuit branch.
//!
//! If `parse_stdin_command` is renamed or moved, update the anchor
//! strings below — the test's purpose is to make the refactor
//! impossible to ship silently.

const SRC: &str = include_str!("../src/stdin_commands/mod.rs");

fn parse_stdin_command_body() -> &'static str {
    let header = "fn parse_stdin_command(";
    let header_pos = SRC.find(header).unwrap_or_else(|| {
        panic!(
            "src/stdin_commands/mod.rs: `{header}` not found — the function may have been renamed. \
             Update tests/stdin_parse_automation_payload_mismatch_contract.rs anchors to match."
        )
    });

    let open_rel = SRC[header_pos..]
        .find('{')
        .expect("no `{` after `fn parse_stdin_command(` header");
    let open_abs = header_pos + open_rel;

    let mut depth: i32 = 0;
    let mut close_abs: Option<usize> = None;
    for (offset, &b) in SRC.as_bytes()[open_abs..].iter().enumerate() {
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

    let close_abs = close_abs.expect("no matching `}` for parse_stdin_command body");
    &SRC[open_abs..=close_abs]
}

#[test]
fn parse_stdin_command_captures_ext_command_error_with_named_binding() {
    let body = parse_stdin_command_body();
    let anchor = "let ext_err = match serde_json::from_value::<ExternalCommand>(";
    assert!(
        body.contains(anchor),
        "parse_stdin_command must capture the ExternalCommand parse error with a named \
         binding via `match ... {{ Ok(...) => ..., Err(err) => err }}`, NOT drop it via \
         `if let Ok(command) = from_value(...)`. A refactor that reintroduces `if let Ok` \
         loses the field-level serde diagnostic and silently reintroduces the Pass #8 bug \
         (known automation verb + wrong payload surfaces the SDK vocabulary). Missing anchor: \
         `{anchor}`. See audits/afk/log.md Pass #9 / SHA 806db738e for the repair."
    );
}

#[test]
fn parse_stdin_command_err_arm_yields_error_as_value() {
    let body = parse_stdin_command_body();
    let anchor = "Err(err) => err,";
    assert!(
        body.contains(anchor),
        "parse_stdin_command's ExternalCommand match must bind the Err variant to a value \
         (so the short-circuit below can interpolate it into \
         `automation_payload_mismatch: ... : {{ext_err}}`). Refactors that replace the arm \
         with `Err(e) => return Err(e.into())`, `Err(_) => unreachable!()`, or a bare `?` \
         would drop the field-level diagnostic reaching the caller. Missing anchor: `{anchor}`."
    );
}

#[test]
fn parse_stdin_command_short_circuits_known_verbs_with_automation_payload_mismatch() {
    let body = parse_stdin_command_body();
    let verbs_anchor = "EXTERNAL_COMMAND_VERBS.contains(";
    let tag_anchor = "automation_payload_mismatch:";
    assert!(
        body.contains(verbs_anchor),
        "parse_stdin_command must short-circuit with an `EXTERNAL_COMMAND_VERBS`-backed \
         check BEFORE falling through to the SDK Message parser. Without this, a known \
         automation verb with the wrong payload field surfaces the SDK vocabulary in the \
         serde error — the Pass #8 bug. Missing anchor: `{verbs_anchor}`."
    );
    assert!(
        body.contains(tag_anchor),
        "parse_stdin_command's short-circuit error must carry the literal \
         `automation_payload_mismatch:` prefix. This prefix is the stable contract \
         documented in lat.md/automation.md §\"Session send parse receipts\" and is used \
         by agentic receipts to classify parse failures. Removing or renaming the prefix \
         is a silent contract break. Missing anchor: `{tag_anchor}`."
    );
}
