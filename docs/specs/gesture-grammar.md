# Spec: One-Key Gesture Grammar

Status: implemented (T2/T8/T9/T11 in flight). Owning decision:
[ADR 0002](../adr/0002-one-key-gesture-grammar.md). Product framing:
`VISION.md` → "The Memory Layer".

## Summary

The existing main hotkey keeps the launcher contract: tap opens or closes the
main window. The launcher opens on key-down in 100% of cases; hold and
double-tap can escalate the already-open window before the tap resolves.
Surfaces morph in place inside the main window. No new chords.

## Gesture Table

| Gesture | Window closed | Window open (launcher or Day Page) |
| --- | --- | --- |
| Tap | Launcher, visible on key-down | Close main window |
| Hold (≥ hold threshold) | Day Page | — (window already open; see Open Questions) |
| Double-tap | Agent Chat | Agent Chat |
| Cmd+Enter | — | Agent Chat (established convention, unchanged) |
| Esc | — | Dismiss |

## State Machine

```text
CLOSED
  └─ key-down ──────────────► LAUNCHER_VISIBLE (render immediately, no waiting)
        │
        ├─ key-up < HOLD_MS ───► TAP_PENDING
        │     ├─ key-down < DOUBLE_MS ──► AGENT_CHAT (morph in place)
        │     └─ DOUBLE_MS elapses ─────► LAUNCHER (plain tap, steady state)
        │
        └─ key still down at HOLD_MS ──► DAY_PAGE
              └─ key-up ──► no-op (Day Page stays; dictation is owned by the
                            dedicated dictation shortcut/window)

LAUNCHER (open, steady)
  ├─ tap ──► CLOSED
  ├─ double-tap ──► AGENT_CHAT
  ├─ Cmd+Enter ──► AGENT_CHAT
  └─ Esc ──► CLOSED

DAY_PAGE (open, steady)
  ├─ tap ──► CLOSED
  ├─ double-tap / Cmd+Enter ──► AGENT_CHAT
  └─ Esc ──► CLOSED
```

Confirmed timings (`src/hotkeys/gesture.rs`, module-level constants):

- `HOLD_MS` = 250ms — long enough that fast taps never trip it, short enough
  that hold-to-open-Day-Page feels immediate.
- `DOUBLE_MS` = 300ms — second key-down window after a tap's key-up.

## Hard Requirements

1. **Key-down instant.** The launcher renders on key-down, before any
   tap/hold/double-tap classification. Classification only ever *deepens* an
   already-visible window. Any design that delays first paint to disambiguate
   is rejected.
2. **No window swaps.** Launcher, Day Page, and Agent Chat are surfaces of the
   main window (the architecture already morphs content in place:
   `MainWindowMode` in `src/main_sections/app_view_state.rs`, Chat Prompt in
   `src/render_prompts/other.rs`). The Notes window remains separate (see
   Notes Window below) but the Day Page never opens as a second window.
3. **Geometrically calm morphs.** The input row keeps its position and size
   across launcher ↔ Day Page; the results list folds away and editor space
   expands beneath. The transformation must read as "the window grew a body,"
   not a screen change. Same window frame, same size as the main input window.
4. **No tap carry-over.** Tapping while the launcher is open closes the main
   window. It must not move search input into the Day Page; Day Page entry is
   owned by hold-from-closed and explicit in-app triggers.
5. **No dictation on hold.** Hold opens the Day Page and nothing else.
   Dictation is invoked exclusively through the dedicated dictation
   shortcut/window; the Day Page does not start, render, or manage dictation.
   (An earlier push-to-talk-on-hold design was implemented and deliberately
   removed.)
6. **Tap-to-dismiss remains.** Tap-while-open closes the main window, matching
   the established launcher hotkey convention. Esc remains a dismiss path too.
7. **Shared editor entity.** The Day Page hosts the same editor entity as the
   Notes window. The editor body must be extracted from `NotesApp`
   (`src/notes/window/`) into a shared component (per the shared component
   contract in `CLAUDE.md`) so note behavior does not fork. No note feature may
   change behavior as part of the extraction.

## Notes Window

The floating Notes window survives with its current behavior. Its role: a note
about something *other than* today — pinned reference material, a document
being drafted alongside other work. The Day Page (in the main window) and a
note (in the Notes window) must feel identical because they are the same
editor entity in different shells.

## Day Page Content Rendering

The Day Page shows everything captured today — deliberate captures, auto-kept
URLs, promoted clipboard entries, Agent Chat traces — in
time order. Long content renders truncated as excerpt cards (the underlying
file structure already enforces this: long captures are fragment files
referenced by excerpt + link, per
[ADR 0003](../adr/0003-markdown-files-as-memory-substrate.md)).

## Open Questions

- **Hold while the window is already open:** still unanswered. Current behavior
  (T8): `HoldStart` while open is not wired — hold only opens the Day Page from
  closed; tap-while-open closes the main window.
- ~~Exact `HOLD_MS`/`DOUBLE_MS` values~~ — **answered:** `HOLD_MS = 250`,
  `DOUBLE_MS = 300` in `src/hotkeys/gesture.rs`, overridable for tests.
- ~~Whether double-tap morphs to the in-window Chat Prompt or focuses the
  dedicated Agent Chat surface~~ — **answered:** double-tap routes to the
  dedicated Agent Chat surface via `open_tab_ai_agent_chat_with_entry_intent`
  (`src/main_sections/gesture_routing.rs`).
- ~~Key-repeat suppression while held~~ — **answered:** repeated `KeyDown`
  while already down are ignored (`src/hotkeys/gesture.rs`, unit-tested).

## Verification Expectations

Per repo policy, prefer behavior tests and runtime proof over source audits:

- Gesture classification: unit tests on the extracted state machine (timings
  injected, no sleeps).
- Key-down render: devtools probe proving launcher target identity exists
  before `HOLD_MS` elapses after key-down.
- Morph-not-swap: window identity receipt (same window id across
  launcher → Day Page → Agent Chat transitions).
- Tap-while-open: runtime receipt showing the main window hides instead of
  morphing to the Day Page.
