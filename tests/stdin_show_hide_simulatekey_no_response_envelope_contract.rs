//! Source-level contract for the fire-and-forget response-envelope
//! contract on `ExternalCommand::Show`, `ExternalCommand::Hide`, and
//! `ExternalCommand::SimulateKey`.
//!
//! Run 9 Pass #5 correctively closes the Pass #4 anomaly
//! `[?] attacker-show-restores-last-view-not-main-menu` (filed Run 9
//! Pass #4, 2026-04-19T05:35Z). Pass #4 observed that a `rpc show`
//! from a "cold" session followed by `rpc getState` reported
//! `{promptType:"acpChat", choiceCount:0}` — it claimed `show` in
//! isolation restored a last-used view instead of the canonical
//! ScriptList. Pass #5 clean repro on a FRESH kill+restart session
//! (pid 75585) — `rpc getState` before show → `promptType:"none"`,
//! `rpc show` in isolation (no preceding `triggerBuiltin`), `rpc
//! getState` → `{promptType:"none", windowVisible:true, isFocused:true,
//! choiceCount:476}` (ScriptList). The Pass #4 observation was a
//! transient artifact — likely a stale view left by a prior pass in
//! that long-running pid 30450 session — NOT a show-restore bug. The
//! Show handler in every dispatcher is inspectable at source and does
//! not mutate `view.current_view`; the Hide handler calls
//! `view.reset_to_script_list(ctx)` (see
//! `src/main_entry/runtime_stdin_match_core.rs:138`), so Hide + Show
//! correctly lands back in ScriptList.
//!
//! The VALIDATED invariant Pass #5 pins is the no-echo half of Pass
//! #4's acceptance menu option (a): `show` / `hide` / `simulateKey`
//! emit NO matching-`requestId` response envelope. Callers that need
//! post-call state must follow up with `getState` — that is the
//! authoritative post-command receipt. This mirrors the Run 8 Pass
//! #23 `inputValue` pin and the Run 9 Pass #2 `requestId` pin on the
//! ingest side: once a contract is observable, it must be pinned at
//! source so the next refactor can't silently break it.
//!
//! Refactor threat: a well-meaning contributor "improves" the
//! automation DX by adding a `showResult` / `hideResult` /
//! `simulateKeyResult` response variant (analogous to `stateResult`)
//! and wiring `response_tx.send(Message::ShowResult { request_id,
//! .. })` at the tail of each dispatcher arm. Compilation passes,
//! existing tests pass, but every external automation script that
//! reads `responses.ndjson` and matches by `requestId` breaks — those
//! scripts currently treat the absence of a matching envelope as
//! "fire-and-forget; follow with getState", so a new echo would make
//! them incorrectly advance on the echoed receipt before any UI
//! mutation (show/hide) lands on screen.
//!
//! These asserts catch such a refactor before merge by forbidding
//! response-emission sinks inside the Show / Hide / SimulateKey arm
//! bodies of the three source-audit dispatcher files:
//!   - `src/main_entry/runtime_stdin_match_core.rs` (Show + Hide arms)
//!   - `src/main_entry/runtime_stdin_match_simulate_key.rs` (whole file;
//!     the file is a single match arm for `ExternalCommand::SimulateKey`)
//!   - `src/main_entry/app_run_setup.rs` (Show + Hide + SimulateKey arms,
//!     scoped via sibling-variant anchors)
//!
//! The live dispatcher at `src/main_entry/runtime_stdin.rs` is kept
//! in lock-step with `runtime_stdin_match_core.rs` by existing
//! source-audit tests in `src/app_impl/tests.rs`; a drift there would
//! already fail those tests, so pinning the audit snippets covers
//! the live path transitively.

const MATCH_CORE: &str = include_str!("../src/main_entry/runtime_stdin_match_core.rs");
const SIMULATE_KEY: &str = include_str!("../src/main_entry/runtime_stdin_match_simulate_key.rs");
const APP_RUN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");

const SHOW_ANCHOR: &str = "ExternalCommand::Show { ref request_id } => {";
const HIDE_ANCHOR: &str = "ExternalCommand::Hide { ref request_id } => {";
const SET_FILTER_ANCHOR: &str = "ExternalCommand::SetFilter {";

const SIMKEY_ANCHOR_APP: &str = "ExternalCommand::SimulateKey { ref key, ref modifiers, .. } => {";
const TRIGGER_ACTION_ANCHOR_APP: &str = "ExternalCommand::TriggerAction {";

/// Slice the `ExternalCommand::Show` arm body — start at the `Show`
/// anchor, end at the `Hide` anchor (the next sibling variant). Both
/// `runtime_stdin_match_core.rs` and `app_run_setup.rs` place Hide
/// immediately after Show; a contributor reordering sibling variants
/// must update this anchor.
fn show_arm<'a>(source: &'a str, file_name: &str) -> &'a str {
    let start = source.find(SHOW_ANCHOR).unwrap_or_else(|| {
        panic!(
            "{file_name} must declare `{SHOW_ANCHOR}` — if the arm's pattern \
             was reshaped (e.g. `{{ request_id, .. }}`), update this anchor in \
             the same commit"
        )
    });
    let end_offset = source[start..].find(HIDE_ANCHOR).unwrap_or_else(|| {
        panic!(
            "{file_name} must continue with `{HIDE_ANCHOR}` after Show — a \
             sibling-variant reorder must update this anchor"
        )
    });
    &source[start..start + end_offset]
}

/// Slice the `ExternalCommand::Hide` arm body — start at the `Hide`
/// anchor, end at the `SetFilter` anchor (the next sibling variant).
fn hide_arm<'a>(source: &'a str, file_name: &str) -> &'a str {
    let start = source.find(HIDE_ANCHOR).unwrap_or_else(|| {
        panic!(
            "{file_name} must declare `{HIDE_ANCHOR}` — if the arm's pattern \
             was reshaped, update this anchor in the same commit"
        )
    });
    let end_offset = source[start..].find(SET_FILTER_ANCHOR).unwrap_or_else(|| {
        panic!(
            "{file_name} must continue with `{SET_FILTER_ANCHOR}` after Hide \
             — a sibling-variant reorder must update this anchor"
        )
    });
    &source[start..start + end_offset]
}

/// Slice the `ExternalCommand::SimulateKey` arm body in
/// `app_run_setup.rs`. Sibling-variant anchor is `TriggerAction`,
/// matching the Pass #3 `stdin_simulatekey_printable_char_noop_contract`
/// slicing convention.
fn app_simulatekey_arm() -> &'static str {
    let start = APP_RUN_SETUP.find(SIMKEY_ANCHOR_APP).unwrap_or_else(|| {
        panic!(
            "app_run_setup.rs must declare `{SIMKEY_ANCHOR_APP}` — if the \
             arm's pattern was reshaped, update this anchor in the same commit"
        )
    });
    let end_offset = APP_RUN_SETUP[start..]
        .find(TRIGGER_ACTION_ANCHOR_APP)
        .unwrap_or_else(|| {
            panic!(
                "app_run_setup.rs must continue with \
                 `{TRIGGER_ACTION_ANCHOR_APP}` after SimulateKey — a \
                 sibling-variant reorder must update this anchor"
            )
        });
    &APP_RUN_SETUP[start..start + end_offset]
}

/// Forbidden response-emission sinks. These are the idioms a
/// contributor would reach for when adding an echo envelope:
///
/// * `response_tx.send(` / `response_sender.send(` /
///   `reader_response_tx.send(` — the three names under which the
///   stdout echo `mpsc::SyncSender<crate::protocol::Message>` is
///   passed through the code today (see
///   `src/stdin_commands/mod.rs::create_stdout_response_sender` and
///   `src/execute_script/mod.rs`). Any one of them appearing inside
///   the dispatch arm body means the dispatcher has been wired to
///   emit a response.
///
/// * `Message::StateResult` / `Message::ShowResult` /
///   `Message::HideResult` / `Message::SimulateKeyResult` — explicit
///   constructor references. `StateResult` is the only variant that
///   currently exists (emitted by `GetState`'s handler via the
///   prompt-message pipeline, NOT by these stdin dispatchers); the
///   other three are hypothetical future variants that would signal
///   a "give each stdin command an echo" refactor.
///
/// * `ShowResult {` / `HideResult {` / `SimulateKeyResult {` — the
///   bare-construct form (without the `Message::` prefix) that a
///   `use crate::protocol::message::...` could elide. Belt to the
///   `Message::`-prefixed suspenders above.
///
/// * `"showResult"` / `"hideResult"` / `"simulateKeyResult"` — the
///   wire-format string literals a `responseType` field would carry.
///   Covers a direct `serde_json::json!({"responseType":"showResult",...})`
///   escape hatch that bypasses the typed `Message` enum.
const FORBIDDEN_RESPONSE_SINKS: &[&str] = &[
    "response_tx.send(",
    "response_sender.send(",
    "reader_response_tx.send(",
    "Message::StateResult",
    "Message::ShowResult",
    "Message::HideResult",
    "Message::SimulateKeyResult",
    "ShowResult {",
    "HideResult {",
    "SimulateKeyResult {",
    r#""showResult""#,
    r#""hideResult""#,
    r#""simulateKeyResult""#,
];

fn assert_no_response_sink(arm: &str, label: &str) {
    for forbidden in FORBIDDEN_RESPONSE_SINKS {
        assert!(
            !arm.contains(forbidden),
            "{label} contains forbidden response-emission sink `{forbidden}`. \
             The Show/Hide/SimulateKey arms are fire-and-forget by contract \
             (Run 9 Pass #5 closing Pass #4's anomaly `attacker-show-\
             restores-last-view-not-main-menu`). Adding an echo envelope \
             would break every automation script that matches by \
             `requestId` and currently treats absence-of-envelope as \
             'fire-and-forget; follow with getState'. See \
             `removed-docs` §\"Prompt and control messages\" for the \
             rationale."
        );
    }
}

// doc-anchor-removed: [[removed-docs and control messages]]
#[test]
fn show_arm_emits_no_response_envelope() {
    // Both source-audit snippets AND the mega-dispatcher must keep
    // the Show arm body free of response-emission sinks.
    assert_no_response_sink(
        show_arm(MATCH_CORE, "runtime_stdin_match_core.rs"),
        "runtime_stdin_match_core.rs (Show arm)",
    );
    assert_no_response_sink(
        show_arm(APP_RUN_SETUP, "app_run_setup.rs"),
        "app_run_setup.rs (Show arm)",
    );
}

// doc-anchor-removed: [[removed-docs and control messages]]
#[test]
fn hide_arm_emits_no_response_envelope() {
    assert_no_response_sink(
        hide_arm(MATCH_CORE, "runtime_stdin_match_core.rs"),
        "runtime_stdin_match_core.rs (Hide arm)",
    );
    assert_no_response_sink(
        hide_arm(APP_RUN_SETUP, "app_run_setup.rs"),
        "app_run_setup.rs (Hide arm)",
    );
}

// doc-anchor-removed: [[removed-docs and control messages]]
#[test]
fn simulatekey_arm_emits_no_response_envelope() {
    // `runtime_stdin_match_simulate_key.rs` IS the SimulateKey arm
    // body — the whole file is a single match arm, so pass the full
    // `include_str!`ed contents.
    assert_no_response_sink(
        SIMULATE_KEY,
        "runtime_stdin_match_simulate_key.rs (whole file = SimulateKey arm)",
    );
    assert_no_response_sink(app_simulatekey_arm(), "app_run_setup.rs (SimulateKey arm)");
}

// doc-anchor-removed: [[removed-docs and control messages]]
#[test]
fn dispatcher_snippets_do_not_import_response_sender() {
    // A contributor adding an echo envelope would first have to bring
    // the response-sender (`mpsc::SyncSender<crate::protocol::Message>`)
    // into the dispatcher's scope. The three source-audit snippet
    // files currently do NOT reference any of the known binding
    // names, and this assertion pins that structural property so the
    // very first step of the refactor trips the test.
    //
    // `app_run_setup.rs` is NOT covered here — as the mega-dispatcher
    // it legitimately hosts other arms (e.g. GetConfigFingerprint)
    // that emit structured `config_fingerprint_result` tracing, and
    // transitively imports plenty of `mpsc`/`Message` names. The Show
    // / Hide / SimulateKey arms within it are pinned above at body
    // granularity.
    for (name, source) in [
        ("runtime_stdin_match_core.rs", MATCH_CORE),
        ("runtime_stdin_match_simulate_key.rs", SIMULATE_KEY),
    ] {
        for binding in ["response_tx", "response_sender", "reader_response_tx"] {
            assert!(
                !source.contains(binding),
                "{name} must not reference `{binding}` — the source-audit \
                 snippet dispatchers are response-channel-free by design. \
                 Introducing a binding is the first step of the echo-\
                 envelope refactor; Run 9 Pass #5 pins this structural \
                 property to force that conversation at review time."
            );
        }
    }
}
