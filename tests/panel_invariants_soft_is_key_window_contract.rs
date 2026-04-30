//! Source-level contract for the `is_key_window` soft-invariant split.
//!
//! Background: at `PanelInvariantPhase::PostMakeKey`, the main panel's
//! invariant checker queried `[window isKeyWindow]` immediately after
//! calling `[window makeKeyWindow]`. AppKit dispatches the key-window
//! promotion asynchronously, so on cold-start show paths the observed
//! state is false even though the launcher did everything right. With
//! the original `record` wiring, this pushed into `mismatched`, flipped
//! `ok()` to false, and panicked in debug builds on the first two
//! cold-start `{"type":"show"}` events (observed before Run 8 Pass #22
//! at `src/platform/panel_invariants.rs:326:13`).
//!
//! Run 8 Pass #22 fix: split `is_key_window` out of the fail-loud
//! bucket into a new `soft_mismatched` bucket via `record_soft`. The
//! `mismatched` bucket keeps fail-loud semantics for invariants that
//! cannot race (level, style mask, activation policy, collection
//! behavior, animation behavior, restorable, autosave-name). Soft
//! failures are still logged through the `PANEL_INVARIANTS` tag (with
//! a `SOFT` discriminator so log triage can distinguish them) but do
//! NOT panic.
//!
//! These source-level asserts pin the structural guarantees that unit
//! tests on the AppKit-free predicates cannot reach:
//!   1. The `record_soft` method exists on `PanelInvariantReport`.
//!   2. The `soft_mismatched: Vec<Invariant>` field exists.
//!   3. The `log_soft_report` function exists.
//!   4. `finish()` calls `log_soft_report` when `soft_mismatched` is
//!      non-empty, so the log signal is preserved.
//!   5. The `is_key_window` check is the ONLY caller of `record_soft`
//!      — if a future contributor softens another invariant, this
//!      test turns red and they must defend the new softening in a
//!      follow-up contract update.
//!   6. `is_key_window` NO LONGER calls `record` — pin the literal
//!      `r.record_soft(` at that site instead.

const PANEL_INVARIANTS: &str = include_str!("../src/platform/panel_invariants.rs");

#[test]
fn record_soft_method_exists_on_report() {
    assert!(
        PANEL_INVARIANTS.contains("fn record_soft(\n        &mut self,\n        ok: bool,"),
        "PanelInvariantReport must expose a record_soft(&mut self, ok, ...) \
         helper — see Run 8 Pass #22. Softening the is_key_window check \
         without this helper reintroduces the cold-start panic."
    );
}

#[test]
fn soft_mismatched_field_exists_on_report() {
    assert!(
        PANEL_INVARIANTS.contains("pub soft_mismatched: Vec<Invariant>"),
        "PanelInvariantReport must carry a soft_mismatched: Vec<Invariant> \
         bucket distinct from `mismatched`. Without the second bucket, \
         `record_soft` would either be silently dropped or alias \
         `mismatched` and refire the cold-start panic."
    );
}

#[test]
fn log_soft_report_function_exists() {
    assert!(
        PANEL_INVARIANTS.contains("fn log_soft_report(context: &'static str,"),
        "panel_invariants.rs must expose a log_soft_report helper so soft \
         mismatches are still visible in structured logs; otherwise the \
         softening silently discards the signal."
    );
    assert!(
        PANEL_INVARIANTS.contains("\"SOFT context={}"),
        "log_soft_report must tag its structured log line with `SOFT` so \
         log triage can distinguish transient AppKit races from real \
         invariant breaks"
    );
}

#[test]
fn finish_routes_soft_mismatches_to_log_soft_report() {
    let finish_body = PANEL_INVARIANTS
        .split("fn finish(")
        .nth(1)
        .expect("panel_invariants.rs must define `fn finish(`");
    assert!(
        finish_body.contains("log_soft_report(context, phase, &r)"),
        "finish() must call log_soft_report when soft_mismatched is \
         non-empty; otherwise softening the is_key_window check drops the \
         log signal entirely"
    );
    assert!(
        finish_body.contains("!r.soft_mismatched.is_empty()"),
        "finish() must gate the log_soft_report call on \
         `!r.soft_mismatched.is_empty()` — calling it unconditionally \
         would spam the log on every healthy show"
    );
}

#[test]
fn is_key_window_uses_record_soft_and_only_it() {
    // The `is_key_window` block must use `record_soft`, not `record`.
    // Find the is_key_window literal and inspect the surrounding call.
    let is_key_idx = PANEL_INVARIANTS.find("\"is_key_window\"").expect(
        "panel_invariants.rs must keep the \"is_key_window\" \
                 invariant name",
    );

    // The `r.record_soft(` call must appear BEFORE the "is_key_window"
    // literal (the literal is passed as the `name` argument to
    // record_soft). Search backwards from the literal.
    let prefix = &PANEL_INVARIANTS[..is_key_idx];
    let record_soft_idx = prefix.rfind("r.record_soft(").expect(
        "the is_key_window block must call `r.record_soft(` — \
                 reverting to `r.record(` reintroduces the cold-start \
                 panic at panel_invariants.rs:326:13",
    );

    // No `r.record(` may appear between the nearest `r.record_soft(` and
    // the "is_key_window" literal — the call must be `record_soft`,
    // not `record`.
    let between = &PANEL_INVARIANTS[record_soft_idx..is_key_idx];
    assert!(
        !between.contains("r.record("),
        "the is_key_window block must call record_soft, not record; an \
         `r.record(` appears between the last record_soft call and the \
         \"is_key_window\" name literal"
    );
}

#[test]
fn record_soft_has_exactly_one_call_site() {
    // If a future contributor softens ANOTHER invariant, this test turns
    // red so they must justify the broader softening in a contract
    // update. The only legitimate `record_soft` caller today is
    // is_key_window.
    let call_count = PANEL_INVARIANTS.matches("r.record_soft(").count();
    assert_eq!(
        call_count, 1,
        "exactly one `r.record_soft(` call site is allowed (the \
         is_key_window check at PostMakeKey); found {call_count}. If you \
         need to soften another invariant, update this test and document \
         why in the panel_invariants.rs comment block."
    );
}

#[test]
fn non_racing_invariants_continue_to_use_record() {
    // The invariants that cannot race MUST stay on fail-loud `record`,
    // not `record_soft`. This prevents a drive-by "just soften them all"
    // refactor that would turn the whole panel-invariants system into a
    // best-effort log and silently accept real regressions.
    for name in [
        "\"main_thread\"",
        "\"main_window_registered\"",
        "\"window_class\"",
        "\"nonactivating_style\"",
        "\"can_become_key\"",
        "\"window_level\"",
        "\"collection_behavior\"",
        "\"activation_policy\"",
        "\"animation_behavior\"",
        "\"restorable\"",
        "\"frame_autosave_name\"",
    ] {
        let idx = PANEL_INVARIANTS
            .find(name)
            .unwrap_or_else(|| panic!("panel_invariants.rs must keep invariant name {name}"));
        let prefix = &PANEL_INVARIANTS[..idx];
        let record_idx = prefix
            .rfind("r.record(")
            .unwrap_or_else(|| panic!("{name} must be recorded via r.record("));
        let between = &PANEL_INVARIANTS[record_idx..idx];
        assert!(
            !between.contains("r.record_soft("),
            "invariant {name} must call record (fail-loud), not record_soft \
             — softening cannot-race invariants defeats the purpose of the \
             panel_invariants guard"
        );
    }
}
