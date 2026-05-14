---
lat:
  require-code-mention: true
---
# Notes Tests

These tests pin Notes-hosted ACP behavior, note-cart handoff semantics, and Notes window placement contracts.

## ACP transcript replay generation

These specs cover ACP thread replay so saved history cannot be contaminated by stale stream events.

### Replay resets transient stream state

Loading saved messages must bump transcript generation and clear stream, permission, tool, plan, mode, command, usage, context, and message-id state before inserting replayed messages.

### Stale stream events are discarded

The async stream pump must capture the thread generation before spawning and reject events whose generation no longer matches before calling `apply_event`.

## Notes ACP draft snapshot across agent switch

These specs cover draft preservation when Notes switches ACP agents by relaunching its embedded view.

### Draft snapshot preserves text and context

Thread and view snapshots must preserve composer text, caret, pending context parts, context consumption, and view-local mention or portal state.

### Agent switch restores snapshot after relaunch

Notes agent switching must capture the draft snapshot before clearing the cached view, relaunch without trimming or initial-input reduction, and restore the snapshot into the new view.

### Reused Notes ACP does not overwrite composer

Reusing an existing Notes ACP view must not call `set_input` as a generic restore path, because reuse and explicit replacement are different host intents.

## Notes context cart consume and dedupe

These specs cover note-cart handoff into the embedded ACP composer.

### Storage lists cart items once per dedupe key

Cart listing for ACP handoff must preserve note order while dropping repeated `NoteCartItem::dedup_key` values.

### Storage delete is note scoped

Batch cart deletion must run in a transaction and match both `note_id` and item id so consuming one note's cart cannot delete another note's item.

### Cart handoff stages replacement context and consumes items

Opening Notes ACP from a cart must list deduped items, stage them through replacement context, and consume all note-scoped persisted cart ids only after staging succeeds.

### Note switch detaches previous note context

Changing the selected note must clear host-owned embedded ACP context for the previous note so hidden Notes ACP state cannot leak stale note chips.

## Notes ACP Escape streaming cancellation

These specs cover Escape parity between Notes-hosted ACP and the main ACP surface.

### Escape cancels streaming before returning to editor

In Notes ACP mode, Escape must dismiss ACP-local popups first, cancel an active stream second, and return to the Notes editor only when no local ACP state consumed it.

## Notes ACP CmdW window close cleanup

These specs cover Notes window close behavior while embedded ACP is active.

### CmdW prepares ACP and closes dialogs before removing Notes

Cmd-W in Notes ACP mode must save the note, prepare the embedded ACP for host hide, persist Notes bounds, close dialogs, remove the window, and restore launcher state if needed.

## Notes ACP actions originating view

These specs cover delayed actions from Notes-hosted ACP actions popups.

### Actions popup refreshes models before snapshot

Opening the Notes ACP actions popup must refresh live models before building the shared actions dialog context.

### Actions dispatch rejects stale Notes ACP generation

Actions selected from a Notes ACP popup must operate on the originating ACP view and reject stale dispatches after the Notes host replaces that view.

## Notes multi-display snap session

These specs cover Notes bounds persistence and display recovery.

### Notes bounds use the Notes window role

Notes window placement must persist through `WindowRole::Notes` and never alias to the main launcher role.

### Restored Notes bounds are clamped to live displays

Restored Notes bounds must be accepted only when visible on a live display, otherwise they must route through the shared clamp helper before use.
