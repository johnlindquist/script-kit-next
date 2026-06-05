//! Source-level contract pinning the lock-release-before-update discipline
//! at `src/notes/window/window_ops.rs::close_notes_window` — the single
//! `WindowCommand::CloseNotesWindow` dispatcher in the notes-window teardown
//! path. Parity pin with the actions-dialog teardown contracts
//! (`tests/close_actions_window_first_line_registry_clear_contract.rs`,
//! Run 9 Pass #30, commit `6b18cb8f1`) — same shape of concern, different
//! surface, different invariant.
//!
//! The shape:
//!
//! ```ignore
//! /// Close the notes window
//! pub fn close_notes_window(cx: &mut App) {
//!     // SAFETY: Release lock BEFORE calling handle.update() to prevent deadlock
//!     // If handle.update() causes Drop to fire synchronously and tries to acquire
//!     // the same lock, we would deadlock. Taking the handle out first avoids this.
//!     let handle = {
//!         let slot = NOTES_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
//!         slot.lock().ok().and_then(|mut g| g.take())
//!     };
//!     crate::windows::remove_automation_window("notes");
//!     crate::windows::remove_runtime_window_handle("notes");
//!
//!     if let Some(handle) = handle {
//!         /* handle.update(...) → save bounds, close dialogs, remove_window */
//!     }
//! }
//! ```
//!
//! Why this Pin exists. `close_notes_window` is the SOLE dispatcher of
//! `WindowCommand::CloseNotesWindow` at
//! `src/window_orchestrator/executor.rs:99`. Other notes-close paths
//! (traffic light, Cmd+W, toggle — see the comment at
//! `src/notes/window.rs:337`) DO NOT route through this function; they
//! have their own close logic. So the invariants here apply to the
//! orchestrator-dispatched close path specifically, which is the path
//! invoked from the keyboard shortcut handler and programmatic close.
//!
//! Five load-bearing details:
//!
//!   1. **Signature stability** — `pub fn close_notes_window(cx: &mut App)`
//!      with `&mut App` (not `&mut Context<T>`, not `&mut cx`, not
//!      `(handle: WindowHandle<NotesWindow>, cx: &mut App)`). The single
//!      caller at `src/window_orchestrator/executor.rs:99` passes `cx`
//!      through `WindowCommand::CloseNotesWindow` dispatch. A signature
//!      change that requires the caller to fetch the handle itself
//!      defeats the lock-release-before-update safety property (the
//!      caller would hold the lock across dispatch).
//!
//!   2. **Lock-release-before-update SAFETY discipline** — the
//!      `let handle = { let slot = NOTES_WINDOW.get_or_init(...); ... .take() };`
//!      block MUST complete (i.e. the scope-introduced `slot` guard must
//!      drop) BEFORE `handle.update(cx, ...)` is called. The SAFETY
//!      comment above the block explicitly calls out the deadlock risk:
//!      `handle.update()` can cause `Drop` to fire synchronously, and
//!      if the Drop impl re-acquires `NOTES_WINDOW`, two concurrent
//!      paths deadlock. A "cleaner" refactor that inlines the
//!      `.lock().ok().and_then(...)` chain into the caller of
//!      `handle.update()` would collapse the scope and hold the lock
//!      across the update — silently reintroducing the deadlock.
//!
//!   3. **Pair of registry clears** — BOTH
//!      `crate::windows::remove_automation_window("notes")` AND
//!      `crate::windows::remove_runtime_window_handle("notes")` fire,
//!      unconditionally, between the static `.take()` and the
//!      `handle.update()`. Dropping either half leaves one registry
//!      shard stale (same failure shape as the Run 9 Pass #28
//!      actions-dialog regression — `listAutomationWindows` reports a
//!      ghost entry while `inspectAutomationWindow` fails with
//!      "No OS window matched"). The two calls are unconditional,
//!      NOT gated on `handle.is_some()` — the registry can drift
//!      out of sync with the static, and the fix is to clear both
//!      shards eagerly even if the static held None.
//!
//!   4. **String literal `"notes"` on both calls** — the registry
//!      key is the bare string `"notes"`. A refactor that introduces
//!      a typed enum (e.g. `WindowKind::Notes`) for
//!      `remove_automation_window` callsites but forgets to update
//!      the registration site's key, or that flips one caller to
//!      `"notes-window"` or `"notesWindow"`, breaks the pairing
//!      silently — the registry entry registered under the old key
//!      is never cleared.
//!
//!   5. **SAFETY anchor comment** — the multi-line comment
//!      containing the phrase `Release lock BEFORE` (literal, with
//!      that exact capitalization) and `deadlock` explains WHY the
//!      block is structured the way it is. A classic silent-cleanup
//!      refactor that deletes the comment ("looks like obvious doc
//!      noise") then "simplifies" the scoped block's `{ ... }` into
//!      a single expression would collapse the lock scope and
//!      reintroduce the deadlock — the comment is the load-bearing
//!      rationale that stops this.
//!
//! **Refactor threat** (named per `looper/rules/discipline.md`):
//! "A contributor consolidates `close_notes_window`,
//! `close_actions_window`, and `close_ai_window` into a generic
//! `close_registered_window<T: RegisteredWindow>(cx: &mut App)`
//! helper parameterized by the window kind. The generic helper
//! accepts a handle-source closure and a registry-key constant; to
//! simplify the caller site, it inlines the `slot.lock().ok()...take()`
//! pattern directly into the call to `handle.update()` via a
//! single-expression `if let Some(h) = slot.lock().ok().and_then(...take()).as_ref()
//! { h.update(cx, ...) }`. The inline form collapses the intermediate
//! scope that was releasing the `NOTES_WINDOW` lock; under the right
//! Drop timing, re-entrance deadlocks the main thread. The deletion
//! of the SAFETY comment is the first step because it no longer
//! matches the consolidated helper's generic shape." This pin
//! defends against BOTH the structural collapse (items 1-4) AND the
//! comment deletion (item 5).

const SOURCE: &str = include_str!("../src/notes/window/window_ops.rs");
const FN_SIGNATURE: &str = "pub fn close_notes_window(cx: &mut App)";
const STATIC_TAKE_MARKER: &str = "NOTES_WINDOW.get_or_init";
const HANDLE_TAKE_MARKER: &str = ".take()";
const REMOVE_AUTO: &str = "crate::windows::remove_automation_window(\"notes\")";
const REMOVE_RUNTIME: &str = "crate::windows::remove_runtime_window_handle(\"notes\")";
const SAFETY_RELEASE: &str = "Release lock BEFORE";
const SAFETY_DEADLOCK: &str = "deadlock";

/// Extract the body of `close_notes_window` — everything between the
/// `{` opening the function and the matching `}`. Panics on malformed source.
fn extract_function_body(source: &str) -> &str {
    let fn_start = source
        .find(FN_SIGNATURE)
        .expect("close_notes_window function not found at source level");
    let body_open = source[fn_start..]
        .find('{')
        .expect("opening brace of close_notes_window not found")
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
    panic!("no matching closing brace found for close_notes_window body");
}

#[test]
fn close_notes_window_exists_with_exact_signature() {
    let hits: Vec<_> = SOURCE.match_indices(FN_SIGNATURE).collect();
    assert_eq!(
        hits.len(),
        1,
        "Expected exactly one `{FN_SIGNATURE}` definition in \
         src/notes/window/window_ops.rs (found {}). If the signature \
         changed — e.g. to take `(handle: WindowHandle<NotesWindow>, cx: &mut App)` \
         (caller now holds the handle and thus the lock across dispatch) \
         or to return `Result<(), String>` (caller may conditionally \
         skip calls based on result) — the single call site at \
         src/window_orchestrator/executor.rs:99 \
         (`WindowCommand::CloseNotesWindow => crate::notes::close_notes_window(cx)`) \
         fails to compile OR silently changes the lock-scope discipline. \
         The `(cx: &mut App)` signature is load-bearing: it forces the \
         callee to own the lock acquisition and release.",
        hits.len()
    );
}

#[test]
fn close_notes_window_takes_handle_via_scoped_lock_block() {
    let body = extract_function_body(SOURCE);
    assert!(
        body.contains(STATIC_TAKE_MARKER),
        "`close_notes_window` body MUST reference `{STATIC_TAKE_MARKER}` — \
         the static-slot access that starts the lock-release-before-update \
         block. If a refactor renames `NOTES_WINDOW` to `NOTES_HANDLE_SLOT` \
         or wraps it in a typed `Slot<NotesWindow>` without updating this \
         read, the pin catches the drift. Body follows:\n{body}"
    );
    assert!(
        body.contains(HANDLE_TAKE_MARKER),
        "`close_notes_window` body MUST contain `{HANDLE_TAKE_MARKER}` — \
         the handle MUST be TAKEN (moved out) from the static slot, not \
         cloned or borrowed. A refactor that changes `.take()` to \
         `.clone()` would leave a dangling reference in the static after \
         the window is removed, and a subsequent `close_notes_window` \
         call would see Some(invalid_handle) and attempt to update a \
         destroyed window. Body follows:\n{body}"
    );
    // The SAFETY block's scope introducer (`let handle = {`) must appear
    // BEFORE the registry clears — verifies the lock-release ordering.
    let handle_block_pos = body
        .find("let handle = {")
        .expect("expected `let handle = {` as the opening of the lock-release-before-update block");
    let remove_auto_pos = body
        .find(REMOVE_AUTO)
        .expect("expected the automation-registry clear to appear in the body");
    assert!(
        handle_block_pos < remove_auto_pos,
        "`let handle = {{` (opening the lock-release scope) MUST appear \
         BEFORE `{REMOVE_AUTO}` — the lock must be released as the block \
         exits, BEFORE any external state mutation. A refactor that \
         moves the registry clears inside the lock-scoped block, or that \
         reorders them before the `.take()`, breaks the discipline. \
         handle_block_pos={handle_block_pos} remove_auto_pos={remove_auto_pos}"
    );
}

#[test]
fn close_notes_window_clears_both_registry_shards() {
    let body = extract_function_body(SOURCE);
    assert!(
        body.contains(REMOVE_AUTO),
        "`close_notes_window` body MUST contain `{REMOVE_AUTO}` — one \
         half of the two-shard registry clear. Dropping this half \
         leaves `listAutomationWindows` reporting a stale \
         `{{id:\"notes\"}}` entry even though the NSWindow is gone \
         (same failure shape as Run 9 Pass #28 for actions-dialog). \
         Body follows:\n{body}"
    );
    assert!(
        body.contains(REMOVE_RUNTIME),
        "`close_notes_window` body MUST contain `{REMOVE_RUNTIME}` — \
         the second half of the two-shard registry clear. Dropping \
         this half leaves the runtime window-handle registry with a \
         dead reference; a subsequent window-bookkeeping query \
         (runtime introspection, Cmd+W dispatch, focus restore) \
         sees a ghost entry. Body follows:\n{body}"
    );
    // Both clears should appear BEFORE the `if let Some(handle) = handle`
    // branch — the registry state must be consistent before the async
    // `handle.update()` fires, because the Drop inside the update might
    // itself query the registry.
    let remove_auto_pos = body
        .find(REMOVE_AUTO)
        .expect("automation registry clear missing");
    let remove_runtime_pos = body
        .find(REMOVE_RUNTIME)
        .expect("runtime registry clear missing");
    let update_branch_pos = body
        .find("if let Some(handle) = handle")
        .expect("expected `if let Some(handle) = handle` update branch in body");
    assert!(
        remove_auto_pos < update_branch_pos && remove_runtime_pos < update_branch_pos,
        "Both registry clears MUST appear BEFORE the `if let Some(handle) = handle` \
         update branch. Moving either clear inside (or after) the update branch \
         would gate the registry clear on `handle.is_some()` and skip it when \
         the static held None — leaving stale registry entries from a prior \
         open that didn't populate the static cleanly. \
         remove_auto_pos={remove_auto_pos} remove_runtime_pos={remove_runtime_pos} \
         update_branch_pos={update_branch_pos}"
    );
}

#[test]
fn close_notes_window_safety_comment_carries_deadlock_rationale() {
    // Look at the bytes immediately preceding the function body — the
    // SAFETY comment should live in the first ~400 bytes of the body.
    let body = extract_function_body(SOURCE);
    let head = &body[..body.len().min(400)];
    assert!(
        head.contains(SAFETY_RELEASE),
        "The SAFETY comment at the top of `close_notes_window` MUST \
         contain the phrase `{SAFETY_RELEASE}` (literal, including \
         the uppercase BEFORE) — the load-bearing rationale that \
         stops a future contributor from 'simplifying' the scoped \
         `let handle = {{ ... }};` block into a single expression \
         that holds the lock across `handle.update()`. The classic \
         silent-cleanup refactor deletes the comment first (looks \
         like doc noise), then the structural collapse follows. \
         Body head (first 400 bytes) follows:\n{head}"
    );
    assert!(
        head.contains(SAFETY_DEADLOCK),
        "The SAFETY comment at the top of `close_notes_window` MUST \
         contain the word `{SAFETY_DEADLOCK}` — spells out the \
         concrete failure mode (re-entrant lock acquisition via Drop \
         → hang). Without the word `deadlock`, the comment reads as \
         a vague ordering preference rather than a safety-critical \
         constraint. Body head follows:\n{head}"
    );
}
