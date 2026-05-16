//! Source-level contract for the `actions-cmdk-dispatcher-fallback-contract`
//! user story (Run 8 Pass #5). Defends the Pass #4 generic Cmd+K fallback
//! in the outer `_ =>` arm of both stdin `simulateKey` dispatchers against
//! a concrete refactor threat.
//!
//! Refactor threat: a contributor collapsing the triple-embedded dispatchers
//! (`src/main_entry/runtime_stdin_match_simulate_key.rs` and
//! `src/main_entry/app_run_setup.rs`) by extracting a shared helper —
//! the same shape the codebase already applies to other dispatcher work,
//! see recent `Prompt: Pin stdin triggerBuiltin …` commits — could drop
//! the outer-arm fallback in the dedup pass. The fallback is a pure
//! structural property: removing it does NOT break compilation, does NOT
//! fail any unit test, and does NOT panic at runtime — it silently
//! re-opens the recurring Cmd+K tool-gap that Run 7 Pass #17
//! (ClipboardHistory, commit `467dbd82d`) and Run 8 Pass #2 (EmojiPicker,
//! commit `5d5ac9f76`) each paid the cost of post-hoc.
//!
//! These tests assert the fallback block's presence and shape by reading
//! both dispatcher sources at compile time via `include_str!`. Any
//! mechanical removal, modifier-gate inversion, or call-target swap makes
//! at least one test red, which gives the refactoring contributor a clear
//! signal before the regression ships.

const CANONICAL_SIMULATEKEY: &str =
    include_str!("../src/main_entry/runtime_stdin_match_simulate_key.rs");
const APP_RUN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");
const ACTIONS_DIALOG: &str = include_str!("../src/app_impl/actions_dialog.rs");
const ACTIONS_TOGGLE: &str = include_str!("../src/app_impl/actions_toggle.rs");

const DISPATCHERS: &[(&str, &str)] = &[
    (
        "src/main_entry/runtime_stdin_match_simulate_key.rs",
        CANONICAL_SIMULATEKEY,
    ),
    ("src/main_entry/app_run_setup.rs", APP_RUN_SETUP),
];

const FALLBACK_LOG_LINE: &str = "SimulateKey: Cmd+K - generic actions toggle (fallback for view=";
const UNHANDLED_VIEW_EVENT: &str = "event = \"simulateKey_unhandled_view\"";
const FALLBACK_HELPER_CALL: &str = "view.simulate_key_requests_generic_actions_toggle(";

fn actions_toggle_helper_body() -> &'static str {
    let start = ACTIONS_TOGGLE
        .find("pub(crate) fn simulate_key_requests_generic_actions_toggle(")
        .expect("simulate_key_requests_generic_actions_toggle helper must exist");
    let section = &ACTIONS_TOGGLE[start..];
    let end = section
        .find("/// Single per-view actions-toggle dispatcher.")
        .unwrap_or(section.len());
    &section[..end]
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn both_dispatchers_contain_generic_cmdk_fallback_log_line() {
    // The distinguishing log string is structurally unique — it is the
    // one sentinel that separates the "fallback fired" from "per-view
    // arm fired" and "unhandled-view warn fired" cases in app.log. Any
    // shared-helper extraction that rewords or drops it breaks the
    // audit-receipt contract that Run 8 Pass #4 established.
    for (name, source) in DISPATCHERS {
        assert!(
            source.contains(FALLBACK_LOG_LINE),
            "{name} is missing the generic Cmd+K fallback log line \
             `{FALLBACK_LOG_LINE}…`. Pass #4 (Run 8) wired this in the \
             outer `_ =>` arm; a dedup refactor that collapses the two \
             dispatchers must preserve the log line verbatim so audit \
             tooling can still distinguish fallback-open from per-view-arm \
             open."
        );
    }
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn both_dispatchers_use_named_generic_cmdk_predicate() {
    for (name, source) in DISPATCHERS {
        assert!(
            source.contains(FALLBACK_HELPER_CALL),
            "{name} fallback block must call the named \
             `simulate_key_requests_generic_actions_toggle` predicate instead \
             of repeating the raw modifier and host checks inline."
        );
    }
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn both_dispatchers_call_toggle_actions_after_fallback_log() {
    // The fallback MUST call `view.toggle_actions(ctx, window)` — the
    // host-agnostic open path that routes through
    // `actions_dialog_host_for_current_view`. Swapping this for any
    // host-specific call (e.g., `toggle_clipboard_actions`) would defeat
    // the whole point of the generic fallback, since the fallback's job
    // is to cover views whose host is known only via `current_actions_host()`.
    for (name, source) in DISPATCHERS {
        let anchor_idx = source.find(FALLBACK_LOG_LINE).unwrap_or_else(|| {
            panic!(
                "{name} lost the fallback log anchor. Run the \
                 `both_dispatchers_contain_generic_cmdk_fallback_log_line` \
                 test first to see the underlying failure."
            )
        });
        // Scope: 400 bytes from the log anchor. The `toggle_actions` call
        // is the immediate next statement after the log emission; a 400-byte
        // window is generous enough for a reasonable reformat without
        // admitting arbitrary drift.
        let tail = &source[anchor_idx..anchor_idx.saturating_add(400).min(source.len())];
        assert!(
            tail.contains("view.toggle_actions(ctx, window);"),
            "{name} fallback block must call `view.toggle_actions(ctx, window)` \
             immediately after logging the fallback line. Substituting any \
             other call (or dropping the call entirely) reverts to the \
             pre-Pass-#4 behavior where Cmd+K on a host-registered view \
             silently fell through to the unhandled-view warn."
        );
    }
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn named_generic_cmdk_predicate_gates_on_modifiers_and_host() {
    // The modifier gate is load-bearing: relaxing it to just `has_cmd`
    // would capture Cmd+Shift+K, Cmd+Alt+K, Cmd+Ctrl+K etc., which the
    // per-view arms (or future per-view arms) might route to different
    // actions. Keeping the gate tight to the exact `has_cmd && !has_shift
    // && !_has_alt && !_has_ctrl && key_lower == "k"` combination mirrors
    // the live GPUI handler's `platform && !shift && !alt && !control`
    // pattern and lets per-view arms diverge for shifted / alted variants
    // without the fallback swallowing them first.
    let helper_body = actions_toggle_helper_body();
    for token in [
        "has_cmd",
        "!has_shift",
        "!has_alt",
        "!has_ctrl",
        "key_lower == \"k\"",
        "self.current_actions_host().is_some()",
    ] {
        assert!(
            helper_body.contains(token),
            "simulate_key_requests_generic_actions_toggle missing `{token}`. \
             The fallback must stay gated on plain Cmd+K plus a live actions host."
        );
    }
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn fallback_block_precedes_unhandled_view_warn() {
    // Ordering contract: the fallback `if` block must appear BEFORE the
    // `simulateKey_unhandled_view` tracing::warn inside the outer `_ =>`
    // arm. Reversing it would emit the `code=unhandled_view` warn first
    // and either fire the fallback after (double-log) or never reach it
    // (if the warn was the `if` branch's else-chained fallback). Either
    // way the audit-log invariant "fallback-intercepted keys do NOT
    // surface as unhandled_view" breaks.
    for (name, source) in DISPATCHERS {
        let fallback_idx = source
            .find(FALLBACK_LOG_LINE)
            .unwrap_or_else(|| panic!("{name} lost the fallback log anchor (see sibling test)."));
        let warn_idx = source.find(UNHANDLED_VIEW_EVENT).unwrap_or_else(|| {
            panic!(
                "{name} is missing the `{UNHANDLED_VIEW_EVENT}` tracing event. \
                 The outer `_ =>` arm's loud-fail block was removed. The \
                 fallback block depends on the warn being the `else` \
                 branch — without the warn the contract below is vacuously \
                 true but the dispatcher no longer loud-fails on genuinely \
                 unhandled views. Investigate."
            )
        });
        assert!(
            fallback_idx < warn_idx,
            "{name} fallback block must appear BEFORE the \
             `{UNHANDLED_VIEW_EVENT}` warn — got fallback@{fallback_idx}, \
             warn@{warn_idx}. Reversing the order would make \
             fallback-intercepted Cmd+K keys double-log as both \
             `unhandled_view` AND `generic actions toggle`, confusing \
             audit tooling and re-opening the regression Pass #4 closed."
        );
    }
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn current_actions_host_api_still_defined_on_app_impl_actions_dialog() {
    // The fallback predicate `view.current_actions_host().is_some()` is
    // a thin wrapper around `live_actions_host_for_view`. Renaming the
    // method without updating the fallback would break compilation at
    // the dispatcher sites (good), but the canonical definition site is
    // named here so the multi-file edit is forced to touch this test
    // too — that's the lever that makes renames a deliberate
    // multi-file change rather than a stdin-only silent breakage.
    assert!(
        ACTIONS_DIALOG
            .contains("pub(crate) fn current_actions_host(&self) -> Option<ActionsDialogHost>"),
        "`src/app_impl/actions_dialog.rs` must define \
         `pub(crate) fn current_actions_host(&self) -> Option<ActionsDialogHost>`. \
         This is the public API the Pass #4 generic Cmd+K fallback gates \
         on — renaming or privatizing it without touching the fallback \
         blocks in both stdin simulateKey dispatchers would silently \
         reduce Cmd+K coverage back to just the per-view arms."
    );
}
