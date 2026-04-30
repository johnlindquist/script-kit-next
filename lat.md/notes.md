# Notes

The Notes window is a separate floating host, not just another launcher panel. It owns its own editor state, actions UI, list or trash modes, and an embedded ACP surface that can be opened and reused inside the same window.

## Key Facts

These facts describe the stable Notes host behavior and its ACP integration.

- Notes lives in a dedicated `NotesApp` window with its own global window handle and entity cache.
- The Notes host switches between two persistent surface modes: the editor (`Notes`) and embedded ACP (`Acp`).
- Embedded ACP is reused when possible instead of respawned on every open, and Notes wires host-specific callbacks for ACP actions, close, history, and portal behavior.
- Notes-hosted ACP only allows the portal kinds that Notes can host locally. `AcpHistory` is supported; broader main-panel portals are intentionally rejected in the Notes host.
- Notes keyboard handling has its own dialog, command-bar, actions-panel, and note-switcher routing, with explicit Enter, Escape, and Tab handling to avoid GPUI focus timing issues.
- In Notes ACP mode, `Escape` dismisses ACP-local popup state first, including the attach menu, and only exits back to the editor when ACP has nothing local left to close.
- Embedded ACP close and embedded ACP-actions close both route through Notes-owned host helpers, so dismissing ACP returns to the editor while dismissing ACP actions returns focus to the ACP composer.
- The Notes ACP actions popup closes through the shared Notes host helper even when the popup window owns the close event, so Escape and backdrop close restore the embedded ACP composer there, and the async cancel branch only consumes the already-closed popup without a second focus bounce.
- The Notes ACP actions popup reuses the shared ACP actions dialog chrome, including the top search field and compact headerless layout, while still filtering out actions the Notes host cannot support.
- Notes-hosted ACP uses the same staged `AcpHistory` replacement contract as detached ACP and the main window. Reopened inline `@history` mentions attach through the pending portal session, and cancelling the local history popup restores the staged composer text and caret instead of leaving the original token behind.

## Key Files

These files define the Notes host, keyboard handling, and persistence behavior.

- [src/notes/window.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window.rs) - Main Notes window host, layout constants, surface modes, and persistent window state.
- [src/notes/window/window_ops.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window/window_ops.rs) - Notes window open/toggle-close plumbing, including the Root-leased close-all-dialogs path.
- [src/notes/window/acp_host.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window/acp_host.rs) - Embedded ACP lifecycle, host callbacks, history popup wiring, and Notes-specific portal restrictions.
- [src/notes/window/keyboard.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window/keyboard.rs) - Notes keyboard routing for dialogs, command bars, actions panels, note switching, and editor/ACP focus behavior.
- [src/notes/model.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/model.rs) - Core note types and IDs.
- [src/notes/storage.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/storage.rs) - Note persistence and storage behavior.

## Source Documents

These source files justify the Notes contract summarized here.

- [src/notes/window.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window.rs)
- [src/notes/window/window_ops.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window/window_ops.rs)
- [src/notes/window/acp_host.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window/acp_host.rs)
- [src/notes/window/keyboard.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window/keyboard.rs)
- [src/notes/model.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/model.rs)
- [src/notes/storage.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/storage.rs)

## Related Pages

These pages cover the broader architecture and ACP surfaces Notes plugs into.

- [architecture](./architecture.md)
- [acp-chat](./acp-chat.md)
- [surfaces](./surfaces.md)

## Host Contract

These rules describe what makes the Notes host distinct from the launcher host.

- The Notes window is independent from the main launcher window and should be documented that way.
- ACP inside Notes is host-owned and callback-driven, not a clone of the launcher ACP surface.
- Notes keyboard routing explicitly compensates for dialog focus timing, so those Enter or Escape paths should be treated as part of the behavior contract rather than incidental glue.
- The Notes toggle-close branch in `window_ops.rs` must clear open dialogs via the already-leased `Root` (`root.close_all_dialogs(window, cx)`), never via `window.close_all_dialogs(cx)`. The `WindowExt::close_all_dialogs` helper wraps its body in `Root::update(...)`, and `handle.update(cx, |root, window, cx| { ... })` on a `WindowHandle<Root>` already holds that lease, so the helper re-enters `EntityMap::lease()` and panics with a double-lease abort on rapid `openNotes → hide → openNotes` toggles.
- `src/notes/window/window_ops.rs::close_notes_window` is the single `WindowCommand::CloseNotesWindow` dispatcher (call site at `src/window_orchestrator/executor.rs:99`). Its body takes the handle out of `NOTES_WINDOW` via a scoped `let handle = { ... .take() };` block whose sole purpose is to release the `NOTES_WINDOW` lock BEFORE `handle.update(cx, ...)` runs — otherwise a re-entrant Drop that touches the same static would deadlock. Both `remove_automation_window("notes")` and `remove_runtime_window_handle("notes")` fire unconditionally between the static take and the update branch, so the registry pair stays consistent even when the static held None. `tests/close_notes_window_lock_release_before_update_contract.rs` pins the `pub fn close_notes_window(cx: &mut App)` signature, the `NOTES_WINDOW.get_or_init` + `.take()` scoped lock-release discipline, the two-shard registry pair with both `"notes"` string keys firing before the `if let Some(handle) = handle` update branch, and the SAFETY comment's `Release lock BEFORE` + `deadlock` rationale — against a consolidation refactor that collapses `close_notes_window`, `close_actions_window`, and `close_ai_window` into a generic `close_registered_window<T>` helper that inlines the lock-scoped block into the caller of `handle.update()` and thereby holds the lock across the update.

## ACP staging replacement

Reusing the Notes-hosted ACP surface must replace prior host-owned pending context instead of appending onto stale chips from the previous note or note cart.

- `stage_note_target_in_embedded_acp(...)` stages a single `FocusedTarget` through `stage_inline_context_parts_from_host(...)`, so note-target switches follow the same replacement path as note-cart handoff.
- This keeps the ACP composer from preserving stale note text or older host-owned context parts when Notes reuses the embedded chat session.
