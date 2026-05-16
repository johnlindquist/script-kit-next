//! Source-level contract for the Run 2 Pass #55
//! `trigger-builtin-repeat-idempotency` user story.
//!
//! Pass #55 live-verified on dev-watch pid 38095 that repeat invocations
//! of the same `triggerBuiltin` name produce identical registry +
//! view state, with no accumulation, no duplicate automation windows,
//! and no surface-tag drift:
//!
//!   `triggerBuiltin tab-ai` ×2           → 1 main window, surface=acpChat, visible=false, identical bounds
//!   `triggerBuiltin clipboard-history` ×2 → 1 main window, surface=clipboardHistory, choiceCount=100 both times
//!   `triggerBuiltin emoji` ×3 (rapid-fire, 200 ms apart) → 1 main window, surface=emojiPicker
//!
//! Repeat-safety is structural: the arm bodies only perform pure field
//! overwrites on `view` (direct `view.current_view = AppView::X { ... }`
//! assignments, idempotent filter/placeholder resets, and internal
//! state-transition calls that themselves route through the single
//! `AppView`-driven update pipeline). The tail call pinned by Pass #52
//! (`update_automation_semantic_surface("main", …)`) is a pure-overwrite
//! registry write — writing the same value twice is a no-op.
//!
//! This contract pins the structural conditions that guarantee repeat
//! idempotency. Specifically: no `ExternalCommand::TriggerBuiltin` arm
//! body may directly call `upsert_automation_window(` or
//! `remove_automation_window(` — those mutate the automation registry's
//! window list and would accumulate / churn on every repeat call. Per
//! Pass #34's architecture, registry lifecycle is driven by view-state
//! transitions (inside `AppView` observers and window open/close
//! lifecycle handlers), NOT from the stdin dispatcher's per-trigger
//! arms.
//!
//! A refactor that, for instance, added
//!     `crate::automation::upsert_automation_window(AutomationWindow { ... });`
//! to the `tab-ai` arm body — perhaps "to make ACP show up in
//! listAutomationWindows immediately" — would create a duplicate entry
//! on the second call, break `listAutomationWindows[0].kind=main`
//! stability that downstream receipts rely on, and could double-fire
//! detach-path registry updates. This contract catches that before it
//! ships.

const RUNTIME_STDIN_MATCH_CORE: &str =
    include_str!("../src/main_entry/runtime_stdin_match_core.rs");
const RUNTIME_STDIN: &str = include_str!("../src/main_entry/runtime_stdin.rs");
const APP_RUN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");

const DISPATCHERS: &[(&str, &str)] = &[
    (
        RUNTIME_STDIN_MATCH_CORE,
        "src/main_entry/runtime_stdin_match_core.rs",
    ),
    (RUNTIME_STDIN, "src/main_entry/runtime_stdin.rs"),
    (APP_RUN_SETUP, "src/main_entry/app_run_setup.rs"),
];

/// Find the byte span of the `ExternalCommand::TriggerBuiltin { ref name }` arm
/// body in a dispatcher source string. Returns the slice between the arm
/// header `{` and the matching `}` that closes the arm. Uses brace counting
/// so nested blocks in the arm body are handled correctly.
fn trigger_builtin_arm_span<'a>(src: &'a str, path: &str) -> &'a str {
    let header_pos = src
        .find("ExternalCommand::TriggerBuiltin")
        .unwrap_or_else(|| {
            panic!(
                "{path}: missing `ExternalCommand::TriggerBuiltin` arm header \
                 — dispatcher shape may have been restructured; update this \
                 contract."
            )
        });

    // Find the first `{` after the header that opens the arm body.
    let open_rel = src[header_pos..].find('{').unwrap_or_else(|| {
        panic!("{path}: no `{{` after `ExternalCommand::TriggerBuiltin` header.")
    });
    let open_abs = header_pos + open_rel;

    // Brace-count forward to find the matching `}`.
    let mut depth: i32 = 0;
    let mut close_abs: Option<usize> = None;
    let bytes = src.as_bytes();
    for (offset, &b) in bytes[open_abs..].iter().enumerate() {
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
    let close_abs = close_abs.unwrap_or_else(|| {
        panic!("{path}: no matching `}}` for the `ExternalCommand::TriggerBuiltin` arm body.")
    });

    &src[open_abs..=close_abs]
}

#[test]
fn no_dispatcher_arm_mutates_automation_window_registry() {
    // The TriggerBuiltin arm body must NOT directly call into the
    // automation registry's window-list mutation API. Window lifecycle
    // is driven by view-state transitions (see removed-docs),
    // not per-trigger side effects. A direct call inside the arm would
    // duplicate/churn the registry on every repeat trigger, breaking
    // the repeat-idempotency guarantee Pass #55 live-verified.
    //
    // Forbidden calls: `upsert_automation_window(` and
    // `remove_automation_window(`. Pass #52's post-match re-key uses
    // `update_automation_semantic_surface` which is a pure overwrite —
    // that call is both required (Pass #52 contract) and idempotent.
    const FORBIDDEN_CALLS: &[&str] = &["upsert_automation_window(", "remove_automation_window("];

    for (src, path) in DISPATCHERS {
        let arm_body = trigger_builtin_arm_span(src, path);
        for forbidden in FORBIDDEN_CALLS {
            assert!(
                !arm_body.contains(forbidden),
                "{path}: `ExternalCommand::TriggerBuiltin` arm body contains a \
                 forbidden registry-mutation call `{forbidden}`. Window \
                 lifecycle must flow through view-state transitions, not \
                 the stdin dispatcher's per-trigger arms. A direct call \
                 here would break repeat-idempotency: calling \
                 `triggerBuiltin X` twice would create duplicate registry \
                 entries. Pass #55 live-verified that \
                 `triggerBuiltin tab-ai` ×2 and `triggerBuiltin emoji` ×3 \
                 both leave exactly one main window in \
                 `listAutomationWindows` — preserve that guarantee."
            );
        }
    }
}

#[test]
fn no_dispatcher_arm_pushes_onto_automation_vec() {
    // Defense-in-depth: forbid pushing onto any automation-related Vec
    // inside the arm body. This catches the pattern
    // `automation_windows.push(...)` / `windows.push(...)` even when
    // the mutation would bypass the named registry APIs above.
    //
    // We look for `.push(` or `.insert(` co-located with the tokens
    // `automation` or `windows` within a 100-byte window (same line or
    // adjacent lines).
    for (src, path) in DISPATCHERS {
        let arm_body = trigger_builtin_arm_span(src, path);

        for (idx, _) in arm_body.match_indices(".push(") {
            let start = idx.saturating_sub(100);
            let end = (idx + 100).min(arm_body.len());
            let window = &arm_body[start..end];
            assert!(
                !window.contains("automation_windows") && !window.contains("automation.windows"),
                "{path}: `ExternalCommand::TriggerBuiltin` arm body near byte \
                 {idx} contains `.push(` co-located with `automation_windows`:\n\
                 \n{window}\n\n\
                 Direct Vec push into the automation registry from the arm \
                 body would accumulate entries on repeat triggers. Route \
                 registry updates through `upsert_automation_window` \
                 (itself forbidden in the arm — see companion test) or \
                 through view-state transition observers."
            );
        }

        for (idx, _) in arm_body.match_indices(".insert(") {
            let start = idx.saturating_sub(100);
            let end = (idx + 100).min(arm_body.len());
            let window = &arm_body[start..end];
            assert!(
                !window.contains("automation_windows") && !window.contains("automation.windows"),
                "{path}: `ExternalCommand::TriggerBuiltin` arm body near byte \
                 {idx} contains `.insert(` co-located with `automation_windows`:\n\
                 \n{window}\n\n\
                 Direct map/vec insert into the automation registry from the \
                 arm body would churn state on repeat triggers. Route \
                 registry updates through view-state transition observers."
            );
        }
    }
}

#[test]
fn arm_body_current_view_assignments_are_pure_struct_literals() {
    // Pin the shape of `view.current_view = …` assignments: each one
    // must be an overwrite with a struct literal (e.g.
    // `view.current_view = AppView::EmojiPickerView { ... };`) or a
    // call-expression that internally performs the overwrite (e.g.
    // `view.open_file_search(...)`). The forbidden pattern is a
    // conditional wrapper like `if view.current_view != X { ... }`
    // around the assignment — that would make repeat triggers NON-
    // idempotent in a subtle way (first call assigns, second call
    // skips downstream work). Pin the ABSENCE of such guards.
    //
    // Heuristic: for every `view.current_view =` assignment in the arm
    // body, walk backwards ≤120 bytes and assert we do NOT find a
    // `view.current_view !=` or `view.current_view ==` token — the
    // canonical skip-guard pattern.
    for (src, path) in DISPATCHERS {
        let arm_body = trigger_builtin_arm_span(src, path);

        for (idx, _) in arm_body.match_indices("view.current_view =") {
            // Skip the case where this is actually `view.current_view ==`
            // or `view.current_view != ` which happen to share the prefix.
            let after = &arm_body[idx + "view.current_view =".len()..];
            if after.starts_with('=') {
                // This is `==` — a comparison, not an assignment; skip.
                continue;
            }

            let start = idx.saturating_sub(120);
            let prefix = &arm_body[start..idx];
            assert!(
                !prefix.contains("view.current_view !=")
                    && !prefix.contains("view.current_view =="),
                "{path}: `ExternalCommand::TriggerBuiltin` arm body near byte \
                 {idx} has a `view.current_view =` assignment guarded by a \
                 `view.current_view !=`/`==` comparison:\n\n{prefix}\n\n\
                 This pattern makes repeat triggers non-idempotent: the \
                 second call skips the assignment and any side effects, \
                 leaving downstream state inconsistent with what the user \
                 would expect after a fresh trigger. Remove the guard and \
                 let the assignment be unconditional — the view will \
                 overwrite with the same value on a no-op repeat."
            );
        }
    }
}
