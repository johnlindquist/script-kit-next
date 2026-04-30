//! Source-level contract test for the stdin `requestId` verbatim-echo
//! contract.
//!
//! Background: Run 8 Pass #24 (attacker probe, anomaly slug
//! `attacker-stdin-requestid-unbounded`) observed that `rpc` payloads
//! carrying a 10 000-char `requestId` are accepted verbatim by the stdin
//! parser and echoed back on the response envelope without any cap,
//! truncation, or transformation. The sole bound is the stdin line cap
//! `MAX_STDIN_COMMAND_BYTES = 16 KiB`. Run 9 Pass #2 adopts acceptance
//! option (a) from the anomaly menu: document the verbatim contract in
//! `lat.md/protocol.md` AND pin it at source level here, mirroring the
//! Run 8 Pass #23 pin for `stateResult.inputValue`.
//!
//! Refactor threat: a well-meaning contributor "hardens"
//! `ExternalCommandRequestId` by (a) replacing the transparent
//! newtype over `String` with a length-bounded wrapper (e.g.
//! `Bounded<256>`, `SmallString<128>`, `ArrayString<N>`), (b) adding a
//! `TryFrom<String>` impl that rejects values past a cap, or (c)
//! sneaking a `.truncate(N)` / `.chars().take(N).collect()` step into
//! any `From`/`Deserialize` path. All three would silently break
//! correlation for callers whose ids legitimately exceed the chosen
//! cap. These asserts catch such a refactor before merge by pinning
//! the exact transparent-over-`String` shape and forbidding any cap
//! literal wrapper inside the newtype declaration and its standard
//! impls.

const STDIN_COMMANDS: &str = include_str!("../src/stdin_commands/mod.rs");
const QUERY_OPS: &str = include_str!("../src/protocol/message/variants/query_ops.rs");

fn external_command_request_id_block() -> &'static str {
    // Anchor on the newtype declaration's `#[derive(...)]` line and slice
    // through the standard impls (as_str, From, Display, AsRef, Deref)
    // up to the next top-level enum/struct/fn boundary. The KeyModifier
    // `#[derive(...` right after is a reliable terminator.
    let start = STDIN_COMMANDS
        .find("pub struct ExternalCommandRequestId(String);")
        .expect("src/stdin_commands/mod.rs must declare ExternalCommandRequestId");
    // Step backwards to include the `#[derive(...)]` + `#[serde(transparent)]`
    // attribute lines so the transparent-repr check covers the declaration.
    let derive_idx = STDIN_COMMANDS[..start]
        .rfind("#[derive(")
        .expect("ExternalCommandRequestId must carry a #[derive(...)] attr");
    let after = &STDIN_COMMANDS[derive_idx..];
    let terminator = after
        .find("pub enum KeyModifier")
        .expect("KeyModifier enum must follow ExternalCommandRequestId impls");
    &after[..terminator]
}

// @lat: [[lat.md/protocol#Protocol#Query and introspection]]
#[test]
fn external_command_request_id_is_transparent_newtype_over_bare_string() {
    // The declaration MUST be a transparent newtype over `String`.
    // ArrayString<N>, SmallString<N>, Bounded<N>, or any other
    // length-bounded wrapper changes deserialization semantics for
    // long payloads and silently defeats the verbatim-echo contract.
    let block = external_command_request_id_block();
    assert!(
        block.contains("#[serde(transparent)]"),
        "src/stdin_commands/mod.rs ExternalCommandRequestId MUST keep \
         #[serde(transparent)] so the JSON wire shape is a bare string \
         (no object wrapper). A refactor that drops this attr changes \
         the stdin protocol in a non-backward-compatible way."
    );
    assert!(
        block.contains("pub struct ExternalCommandRequestId(String);"),
        "src/stdin_commands/mod.rs ExternalCommandRequestId MUST be a \
         transparent newtype over a bare `String` tuple field. Any \
         swap to `Bounded<N>`, `SmallString<N>`, `ArrayString<N>`, or \
         a custom bounded type silently caps requestIds and breaks \
         correlation for callers whose ids exceed the chosen cap."
    );
}

// @lat: [[lat.md/protocol#Protocol#Query and introspection]]
#[test]
fn external_command_request_id_has_no_length_bounded_wrapper() {
    // Forbid specific length-cap construct names in the declaration +
    // impls block. Even if a contributor keeps the outer `String`
    // tuple, a `.truncate(N)` or `.chars().take(N).collect()` hidden
    // inside `From<String>` would silently cap on ingest.
    let block = external_command_request_id_block();
    for forbidden in [
        ".truncate(",
        ".chars().take(",
        ".char_indices().take(",
        "&s[..",
        "[..",
        "ArrayString",
        "SmallString",
        "Bounded<",
    ] {
        assert!(
            !block.contains(forbidden),
            "src/stdin_commands/mod.rs ExternalCommandRequestId block \
             must not contain `{forbidden}` — that would silently cap \
             the verbatim-echo path the stdin RPC contract guarantees"
        );
    }
}

// @lat: [[lat.md/protocol#Protocol#Query and introspection]]
#[test]
fn external_command_request_id_from_string_is_pass_through() {
    // `impl From<String>` MUST be a bare `Self(value)` pass-through
    // with no validation, no trimming, no case folding. A refactor
    // that swaps this for `TryFrom<String>` + a 256-char cap would
    // silently reject long correlation ids at deserialization — the
    // serde path calls `From<String>` transparently.
    let block = external_command_request_id_block();
    assert!(
        block.contains("impl From<String> for ExternalCommandRequestId {"),
        "ExternalCommandRequestId MUST keep `impl From<String>` — the \
         transparent-serde path relies on it for bare-string ingest"
    );
    assert!(
        block.contains("Self(value)"),
        "ExternalCommandRequestId::from(value) MUST be a bare \
         `Self(value)` construction. Any intermediary step (validation, \
         truncation, case-folding) silently breaks verbatim echo."
    );
    // The From<String> body runs from its `impl` line to the next `}`;
    // anchor the forbidden-substr check to that window only.
    let from_start = block
        .find("impl From<String> for ExternalCommandRequestId {")
        .expect("anchor for From<String> impl already asserted above");
    let from_end = block[from_start..]
        .find("\n}")
        .map(|off| from_start + off)
        .expect("From<String> impl must close with `\\n}`");
    let from_body = &block[from_start..from_end];
    for forbidden in [".truncate(", ".chars().take(", "if value.len()"] {
        assert!(
            !from_body.contains(forbidden),
            "ExternalCommandRequestId::from(String) body must not \
             contain `{forbidden}` — verbatim-echo requires a \
             pass-through conversion"
        );
    }
}

// @lat: [[lat.md/protocol#Protocol#Query and introspection]]
#[test]
fn max_stdin_command_bytes_remains_sole_bound() {
    // The verbatim-requestId contract names `MAX_STDIN_COMMAND_BYTES =
    // 16 * 1024` as the sole bound. Pinning the literal here forces a
    // cross-file consistency check: any change to the cap must ripple
    // into the doc comment at `ExternalCommandRequestId` AND the
    // lat.md/protocol.md paragraph in the same commit.
    assert!(
        STDIN_COMMANDS.contains("const MAX_STDIN_COMMAND_BYTES: usize = 16 * 1024;"),
        "src/stdin_commands/mod.rs must keep MAX_STDIN_COMMAND_BYTES = \
         16 * 1024 — the requestId verbatim-echo contract at \
         ExternalCommandRequestId references this literal cap"
    );
}

// @lat: [[lat.md/protocol#Protocol#Query and introspection]]
#[test]
fn state_result_request_id_is_bare_string_with_verbatim_doc() {
    // The response side (`StateResult.request_id`) MUST remain a bare
    // `String` so the sender-echoed value round-trips byte-for-byte.
    // A swap to `ExternalCommandRequestId` here is harmless in JSON
    // shape (both serialize as bare strings) but a swap to a
    // length-bounded wrapper silently breaks long-id callers.
    let start = QUERY_OPS
        .find("#[serde(rename = \"stateResult\")]")
        .expect("query_ops.rs must declare the stateResult serde rename");
    let block = &QUERY_OPS[start..start + 600];
    assert!(
        block.contains("request_id: String,"),
        "StateResult.request_id MUST be a bare `String` — the stdin \
         verbatim-echo contract requires byte-for-byte round-trip on \
         the response envelope"
    );
    // The doc comment that names the contract must sit on the variant
    // so a future refactor can't silently swap the type without also
    // rewriting the rationale.
    let doc_window_start = QUERY_OPS[..start].rfind("/// Response with current UI state");
    assert!(
        doc_window_start.is_some(),
        "StateResult must keep its leading doc comment block (the \
         `/// Response with current UI state` header is the anchor \
         for the verbatim-echo rationale)"
    );
    let doc_window = &QUERY_OPS[doc_window_start.unwrap()..start];
    assert!(
        doc_window.contains("attacker-stdin-requestid-unbounded"),
        "StateResult's doc comment must cite the anomaly slug \
         `attacker-stdin-requestid-unbounded` so future contributors \
         find the historical rationale before changing the type"
    );
    assert!(
        doc_window.contains("tests/stdin_requestid_verbatim_contract.rs"),
        "StateResult's doc comment must reference this contract test \
         so grep-for-the-test-file reaches the declaration"
    );
}
