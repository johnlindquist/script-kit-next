//! Source-level contract for Run 8 Pass #19's Fix of stdin `simulateKey`
//! routing alphanumeric keystrokes into an open ActionsDialog's filter.
//!
//! Before Pass #19, both stdin `SimulateKey` dispatchers
//! (`runtime_stdin_match_simulate_key.rs` and `app_run_setup.rs`) called
//! `route_key_to_actions_dialog(&key_lower, None, …)` with a hardcoded
//! `None` for the `key_char: Option<&str>` parameter. The shared router's
//! printable-char branch at `src/app_impl/actions_dialog.rs:479-486`:
//!
//!     if !modifiers.platform && !modifiers.control && !modifiers.alt {
//!         if let Some(ch) = printable_char(key_char) {
//!             dialog.update(cx, |d, cx| d.handle_char(ch, cx));
//!             ...
//!
//! could therefore never fire — `printable_char(None) -> None` (see
//! `src/ui_foundation/mod.rs:698`). Result: `simulateKey a` with the
//! ActionsDialog open on ScriptList did NOT refilter the dialog, even
//! though every other key class (arrows, Enter, Escape, Cmd+K) routed
//! correctly. Pass #18 observed this as an adjacent routing gap; Pass #19
//! fixes it by deriving `key_char` from `key` when the key is a single
//! character (mirroring live GPUI's `event.keystroke.key_char.as_deref()`
//! from `src/app_impl/startup_new_actions.rs:35`).
//!
//! This contract pins the two dispatchers' call shape so a future
//! "simplification" that drops the single-char guard or re-introduces
//! the `None` literal would fail loudly instead of silently regressing
//! the filter-routing path.

const SIMULATE_KEY_DISPATCHER_SRC: &str =
    include_str!("../src/main_entry/runtime_stdin_match_simulate_key.rs");
const APP_RUN_SETUP_SRC: &str = include_str!("../src/main_entry/app_run_setup.rs");

/// Locate the SimulateKey arm's `route_key_to_actions_dialog(` call and
/// return the 6 lines that follow (the call's argument list up to the
/// closing `)`). Panics with an actionable message if the call is absent.
fn route_call_args(src: &str, label: &str) -> String {
    let call_idx = src
        .find("view.route_key_to_actions_dialog(")
        .unwrap_or_else(|| {
            panic!(
                "{}: MUST retain a `view.route_key_to_actions_dialog(` call in the \
                 stdin SimulateKey dispatcher — the actions-popup pre-dispatch is \
                 the only path that lets simulateKey reach ActionsDialog.",
                label
            )
        });
    let after = &src[call_idx..];
    after.lines().take(8).collect::<Vec<_>>().join("\n")
}

#[test]
fn runtime_stdin_match_simulate_key_derives_key_char_from_single_char() {
    let body = route_call_args(
        SIMULATE_KEY_DISPATCHER_SRC,
        "src/main_entry/runtime_stdin_match_simulate_key.rs",
    );
    assert!(
        body.contains("key_char,"),
        "src/main_entry/runtime_stdin_match_simulate_key.rs MUST pass `key_char` \
         (NOT `None`) as the second argument to `route_key_to_actions_dialog(...)`. \
         Passing `None` prevents printable keystrokes from reaching the \
         ActionsDialog filter — see Pass #19 context in audits/afk/log.md. \
         Found call args:\n{body}"
    );
    assert!(
        !body.contains("None,"),
        "src/main_entry/runtime_stdin_match_simulate_key.rs MUST NOT pass a bare `None,` \
         literal as the second arg of `route_key_to_actions_dialog` — that regresses \
         Pass #19's fix. Use the `key_char: Option<&str>` local that's derived from \
         `key.chars().count() == 1`. Found:\n{body}"
    );
    assert!(
        SIMULATE_KEY_DISPATCHER_SRC
            .contains("let key_char: Option<&str> = if key.chars().count() == 1 {"),
        "src/main_entry/runtime_stdin_match_simulate_key.rs MUST derive the `key_char` \
         local with `if key.chars().count() == 1 {{ Some(key.as_str()) }} else {{ None }}`. \
         This matches live GPUI's semantic: key_char is Some only for printable \
         single-character input. Without the single-char guard, multi-char key names \
         (\"Escape\", \"ArrowUp\") would leak their first char into the dialog filter \
         via `printable_char`'s `.chars().next()` fallback."
    );
}

#[test]
fn app_run_setup_derives_key_char_from_single_char() {
    let body = route_call_args(APP_RUN_SETUP_SRC, "src/main_entry/app_run_setup.rs");
    assert!(
        body.contains("key_char,"),
        "src/main_entry/app_run_setup.rs MUST pass `key_char` (NOT `None`) as the \
         second argument to `route_key_to_actions_dialog(...)`. Same regression as \
         runtime_stdin_match_simulate_key.rs — see Pass #19. Found:\n{body}"
    );
    assert!(
        !body.contains("None,"),
        "src/main_entry/app_run_setup.rs MUST NOT pass a bare `None,` literal as the \
         second arg of `route_key_to_actions_dialog`. Use the `key_char` local. \
         Found:\n{body}"
    );
    assert!(
        APP_RUN_SETUP_SRC.contains("let key_char: Option<&str> = if key.chars().count() == 1 {"),
        "src/main_entry/app_run_setup.rs MUST derive `key_char` with the same \
         single-char guard as runtime_stdin_match_simulate_key.rs. Divergence between \
         the two stdin dispatchers silently breaks on whichever dispatcher the live \
         build is using for the current ExternalCommand routing path."
    );
}

#[test]
fn key_char_derivation_is_identical_across_both_dispatchers() {
    // Both files MUST contain the EXACT same derivation snippet. If one
    // drifts (e.g. one uses `key.len() == 1` which is wrong for multi-byte
    // UTF-8), the other silently out-diverges and the fix regresses on
    // whichever stdin path the build is routing through.
    let anchor = "let key_char: Option<&str> = if key.chars().count() == 1 {\n                                    Some(key.as_str())\n                                } else {\n                                    None\n                                };";
    assert!(
        SIMULATE_KEY_DISPATCHER_SRC.contains(anchor),
        "runtime_stdin_match_simulate_key.rs MUST contain the canonical key_char \
         derivation block verbatim. Drift is a regression vector."
    );
    assert!(
        APP_RUN_SETUP_SRC.contains(anchor),
        "app_run_setup.rs MUST contain the canonical key_char derivation block \
         verbatim. Drift is a regression vector."
    );
}
