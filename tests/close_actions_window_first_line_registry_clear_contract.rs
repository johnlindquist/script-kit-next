//! Source-level contract pinning the Run 9 Pass #29 invariant that
//! `close_actions_window(cx: &mut App)` in `src/actions/window.rs`
//! clears the `actions-dialog` automation-registry entry as the
//! VERY FIRST statement of its body, BEFORE it touches the
//! `ACTIONS_WINDOW: Mutex<Option<WindowHandle<ActionsWindow>>>`
//! static. This ordering is load-bearing across four unrelated
//! files because of Pass #29's upgrade: all four hide dispatchers
//! (`src/main_sections/window_visibility.rs::hide_main_window_helper`,
//! `src/main_entry/runtime_stdin.rs`,
//! `src/main_entry/runtime_stdin_match_core.rs`,
//! `src/main_entry/app_run_setup.rs`) call
//! `crate::actions::close_actions_window(cx)` to tear down BOTH the
//! registry HashMap AND the static. If a future "simplification"
//! of this function drops the first-line
//! `crate::windows::remove_automation_window("actions-dialog")`
//! call (on the plausible-sounding grounds that "the static is the
//! source of truth, the registry entry is redundant"), the four
//! hide dispatchers will continue to compile cleanly — they still
//! call `close_actions_window(cx)` — but
//! `listAutomationWindows` post-hide will once again report
//! `{id:"actions-dialog", visible:true, parentWindowId:"main"}`
//! even though the NSWindow was torn down with main. That is the
//! exact Run 9 Pass #23 regression shape
//! (`attacker-hide-path-actions-dialog-registry-stale`), now
//! re-entered through a different code path.
//!
//! The existing contract coverage is weaker than this Pin by
//! design:
//!   - `tests/automation/actions_dialog_targeting.rs:700`
//!     (`close_actions_window_unregisters_from_automation_registry`)
//!     only checks that the string
//!     `remove_automation_window("actions-dialog")` appears
//!     SOMEWHERE in `src/actions/window.rs` — it doesn't scope to
//!     `close_actions_window`'s body, and it doesn't pin the
//!     ordering relative to the static clear. A refactor that
//!     moves the call into, say, an `ActionsWindow::Drop` impl
//!     (arguing "closure ownership should own its cleanup")
//!     would satisfy that existing test while breaking Pass #29's
//!     hide-path teardown. This Pin defends against that
//!     relocation.
//!   - `tests/source_audits/actions_popup_contract.rs:59-68`
//!     (`close_actions_window_emits_closed_receipt`) pins the
//!     `Closed` event emission but not the registry clear.
//!   - `tests/hide_path_actions_dialog_registry_teardown_contract.rs`
//!     (Pass #29's own hide-path pin) bans the legacy bare
//!     `crate::windows::remove_automation_window("actions-dialog")`
//!     call in the four hide dispatchers, forcing them through
//!     `close_actions_window(cx)`. That test chain becomes
//!     vacuously compliant if `close_actions_window`'s body
//!     quietly drops the registry clear — the four dispatchers
//!     would still pass their Pin ("they call the full teardown
//!     function"), but the full teardown function would no
//!     longer teardown. This file closes that loop.
//!
//! **Refactor threat**: a contributor reads the existing
//! `// Unregister from automation registry before destroying the
//! window` comment, notices the NSWindow is already gone by the
//! time hide completes, concludes "the comment is outdated because
//! there's nothing to destroy anymore", and deletes the
//! `remove_automation_window` call as dead code. Or a contributor
//! extracting the static-clearing logic into a shared
//! `WindowHandleOption::take_and_close(cx)` helper (plausible —
//! similar patterns exist for `NOTES_WINDOW` and `AI_WINDOW`)
//! moves the `ACTIONS_WINDOW.get()` + `guard.take()` block into
//! the helper but leaves the `remove_automation_window` call
//! orphaned in the old function body; a subsequent "cleanup" of
//! what now looks like a trivial one-line function inlines it
//! into callers, loses the registry clear in the process, and
//! breaks Pass #29's 4-dispatcher contract.
//!
//! The four assertions pin:
//!   1. `pub fn close_actions_window(cx: &mut App)` exists exactly
//!      once at source level (signature stability).
//!   2. Inside its body, the
//!      `crate::windows::remove_automation_window("actions-dialog")`
//!      call appears AT ALL and strictly BEFORE the first
//!      occurrence of `ACTIONS_WINDOW.get(` (first-line invariant).
//!   3. Inside its body, `ACTIONS_WINDOW.get(` AND `guard.take(`
//!      both appear (the `ACTIONS_WINDOW` static clear is still
//!      present — Pass #29's "clear the static too" half of the
//!      fix).
//!   4. The anchor comment
//!      `Unregister from automation registry before destroying`
//!      appears verbatim in the body, above the registry clear —
//!      if a "cleanup" deletes the rationale comment on its own
//!      (the classic "silent-cleanup" shape that preserves the
//!      code but loses the load-bearing explanation), a future
//!      contributor reading the bare
//!      `crate::windows::remove_automation_window(...)` call has
//!      no hint why it's first and is a single "simplification"
//!      away from moving or deleting it.

const SOURCE: &str = include_str!("../src/actions/window.rs");
const FN_SIGNATURE: &str = "pub fn close_actions_window(cx: &mut App)";
const ANCHOR_COMMENT: &str = "Unregister from automation registry before destroying";
const REGISTRY_CALL: &str = "crate::windows::remove_automation_window(\"actions-dialog\")";
const STATIC_READ: &str = "ACTIONS_WINDOW.get(";
const STATIC_TAKE: &str = "guard.take(";

/// Extract the body of `close_actions_window` — everything between the `{` that opens
/// the function and the matching `}`. Fails the test if either end isn't found.
fn extract_function_body(source: &str) -> &str {
    let fn_start = source
        .find(FN_SIGNATURE)
        .expect("close_actions_window function not found at source level");
    let body_open = source[fn_start..]
        .find('{')
        .expect("opening brace of close_actions_window not found")
        + fn_start;
    let body_bytes = source.as_bytes();
    let mut depth: i32 = 0;
    let mut i = body_open;
    while i < body_bytes.len() {
        match body_bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[body_open + 1..i];
                }
            }
            _ => {}
        }
        i += 1;
    }
    panic!("no matching closing brace found for close_actions_window body");
}

// @lat: [[lat.md/acp-chat#Detached window behavior]]
#[test]
fn close_actions_window_exists_with_exact_signature() {
    let hits: Vec<_> = SOURCE.match_indices(FN_SIGNATURE).collect();
    assert_eq!(
        hits.len(),
        1,
        "Expected exactly one `pub fn close_actions_window(cx: &mut App)` \
         definition in src/actions/window.rs (found {}). If the signature \
         changed (e.g. to take a WindowHandle or AsyncAppContext), the Pass \
         #29 upgrade may need a corresponding review — four hide dispatchers \
         call this function and assume the `&mut App` signature lets them \
         pass `cx` (window_visibility.rs::hide_main_window_helper) or `ctx` \
         (the three runtime_stdin hide arms, which rely on \
         `Context<ScriptListApp>`'s `DerefMut<Target=App>` coercion).",
        hits.len()
    );
}

// @lat: [[lat.md/acp-chat#Detached window behavior]]
#[test]
fn close_actions_window_first_statement_clears_automation_registry() {
    let body = extract_function_body(SOURCE);
    let registry_idx = body.find(REGISTRY_CALL).unwrap_or_else(|| {
        panic!(
            "`close_actions_window` body MUST contain \
             `{REGISTRY_CALL}` as its first real statement. Without \
             this call, the four Pass #29 hide dispatchers \
             (`src/main_sections/window_visibility.rs`, \
             `src/main_entry/runtime_stdin.rs`, \
             `src/main_entry/runtime_stdin_match_core.rs`, \
             `src/main_entry/app_run_setup.rs`) will silently stop \
             cleaning the `actions-dialog` entry from the \
             `AutomationWindowState` HashMap on hide — they all call \
             `crate::actions::close_actions_window(cx)` and rely on \
             this function's first line to do the registry clean-up. \
             `listAutomationWindows` post-hide will then revert to \
             reporting `{{id:\"actions-dialog\", visible:true}}` (the \
             Run 9 Pass #23 regression, Pass #29 closure shape). \
             Body follows:\n{body}"
        )
    });
    let static_idx = body.find(STATIC_READ).unwrap_or_else(|| {
        panic!(
            "`close_actions_window` body MUST read `{STATIC_READ}` to \
             clear the `ACTIONS_WINDOW` static — this is the other half \
             of the Pass #29 fix. Body follows:\n{body}"
        )
    });
    assert!(
        registry_idx < static_idx,
        "Inside `close_actions_window`, the \
         `{REGISTRY_CALL}` call (offset {registry_idx}) MUST appear \
         BEFORE the first `{STATIC_READ}` read (offset {static_idx}). \
         Any refactor that reorders these — e.g. moving the registry \
         clear inside the `if let Some(handle) = guard.take() {{ ... }}` \
         arm so it only runs when the static had a handle — would \
         break Pass #29's hide-path contract: the four hide \
         dispatchers call this function on EVERY hide regardless of \
         whether a popup was open, and the unconditional \
         first-line registry clear is what makes that idempotent. \
         Gating the clear on the static's presence re-introduces the \
         Pass #23 regression on any code path that leaves the \
         registry entry without a matching static (e.g. a panic \
         between `register_attached_popup` and the static-set, or a \
         test harness that touches the registry directly)."
    );
}

// @lat: [[lat.md/acp-chat#Detached window behavior]]
#[test]
fn close_actions_window_also_clears_actions_window_static() {
    let body = extract_function_body(SOURCE);
    assert!(
        body.contains(STATIC_READ),
        "`close_actions_window` body MUST read `{STATIC_READ}` — \
         without this, the Pass #29 stale-static bug returns: the \
         `ACTIONS_WINDOW: Mutex<Option<WindowHandle>>` static keeps \
         holding `Some(handle)` across hide, and a subsequent \
         unfocused `simulateKey cmd+k` routes `is_actions_window_open()=true` \
         into the CLOSE branch of `toggle_clipboard_actions`, popping \
         whichever overlay was on top instead of opening the actions \
         dialog. Body follows:\n{body}"
    );
    assert!(
        body.contains(STATIC_TAKE),
        "`close_actions_window` body MUST call `{STATIC_TAKE}` to \
         pull the handle out of the `ACTIONS_WINDOW` Mutex<Option<>> \
         and clear it. Without this, the static is read but never \
         reset, re-introducing the Pass #29 regression shape. Body \
         follows:\n{body}"
    );
}

// @lat: [[lat.md/acp-chat#Detached window behavior]]
#[test]
fn close_actions_window_anchor_comment_above_registry_clear() {
    let body = extract_function_body(SOURCE);
    let comment_idx = body.find(ANCHOR_COMMENT).unwrap_or_else(|| {
        panic!(
            "`close_actions_window` body MUST contain the anchor \
             comment `{ANCHOR_COMMENT}...` — this is the \
             load-bearing rationale for why \
             `{REGISTRY_CALL}` is the FIRST statement of the \
             function. Without the comment, a future contributor \
             reading the bare call has no signal that it is \
             (a) the first-line invariant, (b) relied on by four \
             hide dispatchers, or (c) the Pass #29 fix for a \
             known-live regression. The classic silent-cleanup \
             refactor deletes the comment first (it looks like \
             dead weight) and then deletes the call (it looks \
             redundant with the `ACTIONS_WINDOW.get()` clear \
             below). Either half alone is catastrophic; the \
             comment-first variant is harder to catch in code \
             review because the code still works at compile-time. \
             Body follows:\n{body}"
        )
    });
    let registry_idx = body
        .find(REGISTRY_CALL)
        .expect("covered by close_actions_window_first_statement_clears_automation_registry");
    assert!(
        comment_idx < registry_idx,
        "The anchor comment `{ANCHOR_COMMENT}...` (offset \
         {comment_idx}) MUST precede the `{REGISTRY_CALL}` call \
         (offset {registry_idx}). A refactor that moved the call up \
         but left the comment below it — a typical \
         clippy-needless-continue-style reorder — breaks the \
         reader's ability to associate the rationale with the \
         call."
    );
}
