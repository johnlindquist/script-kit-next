//! Source-level structural contract: `ActionsDialog`'s drill-down
//! route stack MUST preserve parent UI state across push/pop.
//!
//! # Why this matters (UX)
//!
//! The shared `ActionsDialog` at `src/actions/dialog.rs` supports
//! drill-down sub-menus via `route_stack: Vec<ActionsDialogRouteState>`.
//! The canonical live consumer today is the Agent Chat agent-switch menu
//! at `src/ai/agent_chat/ui/view.rs` — users open Cmd+K on the Agent Chat chat,
//! select "Switch agent", and drill into a child route listing
//! available agents. Escape from the child MUST pop back to the
//! parent with ITS search text, selection, and scroll position
//! intact. That round-trip is powered by four carefully-ordered
//! calls inside `push_route` / `pop_route` / `apply_route_state`:
//!
//!   1. `push_route` calls `snapshot_current_route_state()` BEFORE
//!      pushing the child, so the parent's `search_text` +
//!      `selected_action_id` are captured into the parent's
//!      `ActionsDialogRouteState` on the stack.
//!   2. `push_route` then calls `apply_route_state_from_route(&route)`
//!      to build the child's UI (clear search, load child's actions).
//!   3. `pop_route` guards on `self.route_stack.len() <= 1` so the
//!      root route is NEVER popped — otherwise the stack goes empty
//!      and the next `push_route` silently corrupts via `last_mut()`
//!      returning `None` inside `snapshot_current_route_state`.
//!   4. `pop_route` calls `apply_route_state(&state, cx)` with the
//!      new top-of-stack, which MUST assign the parent's saved
//!      `search_text` back to `self.search_text` — this is THE
//!      parent-state-restore. Without it, Escape from the child
//!      lands the user back at the root with an empty filter,
//!      losing whatever they had typed.
//!
//! Pass #11 (`tests/actions_dialog_escape_filter_agnostic_contract.rs`)
//! pinned the `handle_escape` delegation chain — it calls
//! `self.pop_route(cx)` and chooses between `PoppedRoute` and
//! `CloseDialog`. But the escape-filter contract doesn't pin the
//! BODY of `pop_route` itself: a contributor could "simplify"
//! `pop_route` to `self.route_stack.pop(); true` and flip zero
//! Pass #11 tests red while destroying the parent-state-restore
//! invariant on every drill-down return.
//!
//! Pass #13 / #14 pinned section-header skip at the
//! arrow-key and selection-clamp level. This pass pins the
//! drill-down depth-preservation invariants at the route-stack
//! mutation level — the three angles together cover "the cursor
//! never lands on a header" AND "drill-down restore doesn't lose
//! parent state".
//!
//! # Anchors
//!
//!   (1) `push_route` body MUST call `snapshot_current_route_state(`
//!       — without it the parent's `search_text` / selection is
//!       never captured and pop loses state.
//!   (2) `push_route` body MUST call `apply_route_state_from_route(`
//!       — without it the child's UI is never built (search stays
//!       on parent's text, actions stay parent's list).
//!   (3) `pop_route` body MUST contain the `self.route_stack.len() <= 1`
//!       guard (any equivalent: `< 2` / `== 1` / `<= 1`). Without
//!       the guard, popping the root leaves the stack empty and the
//!       next `push_route` silently corrupts via the `last_mut()`
//!       no-op branch in `snapshot_current_route_state`.
//!   (4) `pop_route` body MUST call `apply_route_state(` —
//!       without it the parent's saved state is never reapplied
//!       even if the pop succeeds.
//!   (5) `apply_route_state` body MUST assign
//!       `self.search_text = state.search_text` — the
//!       parent-search-text-restore line. A "simplification" to
//!       `self.search_text.clear()` (mirroring
//!       `apply_route_state_from_route`) would silently drop the
//!       parent's saved filter on every drill-down return.

const DIALOG_SOURCE: &str = include_str!("../src/actions/dialog.rs");

fn extract_fn_body<'a>(src: &'a str, header: &str) -> &'a str {
    let header_pos = src.find(header).unwrap_or_else(|| {
        panic!(
            "source: `{header}` not found — the method may have been renamed. \
             Update the contract anchors in \
             tests/actions_dialog_route_stack_contract.rs."
        )
    });
    let open_rel = src[header_pos..]
        .find('{')
        .unwrap_or_else(|| panic!("no `{{` after `{header}` header"));
    let open_abs = header_pos + open_rel;

    let mut depth: i32 = 0;
    let mut close_abs: Option<usize> = None;
    for (offset, &b) in src.as_bytes()[open_abs..].iter().enumerate() {
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
    let close_abs = close_abs.unwrap_or_else(|| panic!("no matching `}}` for `{header}` body"));
    &src[open_abs..=close_abs]
}

#[test]
fn push_route_snapshots_parent_state_before_pushing() {
    // Snapshot MUST fire BEFORE the push so the parent's search_text
    // and selected_action_id are captured into the parent's
    // `ActionsDialogRouteState` (which becomes the `last_mut()` the
    // snapshot writes into). If the order were reversed — push then
    // snapshot — the snapshot would write to the CHILD's freshly-
    // allocated state and the parent's state would be lost forever.
    let body = extract_fn_body(DIALOG_SOURCE, "pub fn push_route(");
    assert!(
        body.contains("snapshot_current_route_state("),
        "`push_route` MUST call `snapshot_current_route_state()` \
         before pushing the child. Without it the parent's \
         `search_text` / `selected_action_id` are never captured \
         and popping back loses the parent's filter on every \
         drill-down return (Agent Chat agent-switch being the canonical \
         consumer). Body was:\n{body}"
    );

    // Structural ordering check: the snapshot call must appear
    // BEFORE the first `route_stack.push(` call in the body. The
    // byte offsets are linear in source order.
    let snap_idx = body.find("snapshot_current_route_state(").unwrap();
    let push_idx = body
        .find("route_stack")
        .and_then(|start| body[start..].find(".push(").map(|rel| start + rel));
    if let Some(push_idx) = push_idx {
        assert!(
            snap_idx < push_idx,
            "`push_route`: snapshot_current_route_state() MUST appear \
             BEFORE `route_stack.push(` — if the push happens first, \
             `last_mut()` inside snapshot writes to the CHILD's \
             state (not the parent's), and the parent's filter / \
             selection is lost. Body was:\n{body}"
        );
    }
}

#[test]
fn push_route_builds_child_ui_via_apply_from_route() {
    let body = extract_fn_body(DIALOG_SOURCE, "pub fn push_route(");
    assert!(
        body.contains("apply_route_state_from_route("),
        "`push_route` MUST call `apply_route_state_from_route(&route)` \
         so the child's actions / context_title / search_placeholder \
         replace the parent's. Skipping this would leave the parent's \
         actions visible under the child's route banner — users \
         would see \"Switch Agent\" in the breadcrumb but the old \
         Agent Chat-host action list underneath. Body was:\n{body}"
    );
}

#[test]
fn pop_route_guards_against_popping_the_root() {
    // Without a depth guard, `pop_route` could pop the root route,
    // leaving `route_stack` empty. The subsequent
    // `self.route_stack.last().cloned()` at the current
    // implementation's line 1142 would return `None` and early-
    // return `false` — but the stack is already corrupted. Next
    // `push_route` would call `snapshot_current_route_state()` on
    // an empty stack (no-op via `if let Some(last_mut)`), then push
    // a single child with no parent underneath, so `pop_route`
    // would always close the dialog instead of restoring a parent.
    let body = extract_fn_body(DIALOG_SOURCE, "pub fn pop_route(");
    let has_depth_guard = body.contains("self.route_stack.len() <= 1")
        || body.contains("self.route_stack.len() < 2")
        || body.contains("self.route_stack.len() == 1")
        || body.contains("self.route_stack.len() > 1");
    assert!(
        has_depth_guard,
        "`pop_route` MUST contain a depth guard on \
         `self.route_stack.len()` (`<= 1`, `< 2`, `== 1`, or `> 1`) \
         so the root route is never popped. Without it, the stack \
         goes empty and subsequent `push_route` silently corrupts. \
         Body was:\n{body}"
    );
}

#[test]
fn pop_route_restores_parent_state_via_apply_route_state() {
    // Note: `apply_route_state(` — NOT `apply_route_state_from_route(`
    // which is the push variant that *clears* search_text rather than
    // restoring the caller-provided one. Getting these mixed up would
    // clear the parent's filter on pop — the same regression class as
    // the Pass #11 escape-filter-agnostic contract, just one layer
    // deeper in the call chain.
    let body = extract_fn_body(DIALOG_SOURCE, "pub fn pop_route(");
    assert!(
        body.contains("apply_route_state(") && !body.contains("apply_route_state_from_route("),
        "`pop_route` MUST call `apply_route_state(&state, cx)` — the \
         stateful variant that restores the parent's saved \
         `search_text` / selection / scroll. It MUST NOT call \
         `apply_route_state_from_route(&route)`, which is the *push* \
         variant that clears search_text to build a fresh child UI. \
         Mixing these would lose the parent's filter on every pop. \
         Body was:\n{body}"
    );
}

#[test]
fn apply_route_state_restores_parent_search_text() {
    // The parent-state-restore line. Without this assignment,
    // `self.search_text` would stay on whatever the child had (or
    // empty if the child was freshly-pushed and never typed in),
    // and popping back would land the user at the root with the
    // wrong filter content.
    let body = extract_fn_body(DIALOG_SOURCE, "fn apply_route_state(");
    assert!(
        body.contains("self.search_text = state.search_text"),
        "`apply_route_state` MUST assign \
         `self.search_text = state.search_text.clone()` (or similar) \
         to restore the parent's saved filter on pop. A \
         `self.search_text.clear()` substitution (mirroring \
         `apply_route_state_from_route`) would silently drop the \
         parent's filter on every drill-down return. Body was:\n{body}"
    );

    // Symmetric negative pin: apply_route_state (the stateful
    // variant) must NOT call `self.search_text.clear()` — that's
    // the push-variant's job. Confusing the two is the shape the
    // regression takes as an "unify both apply_* fns" refactor.
    assert!(
        !body.contains("self.search_text.clear()"),
        "`apply_route_state` MUST NOT call `self.search_text.clear()` \
         — that's `apply_route_state_from_route`'s job (which clears \
         to build a fresh child UI on push). The stateful variant \
         restores the parent's saved filter; clearing it here would \
         be the same regression class as the Pass #11 \
         escape-filter-agnostic contract, one layer deeper. Body was:\n{body}"
    );
}
