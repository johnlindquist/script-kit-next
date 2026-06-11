# Spec: One-Key Gesture Grammar

Status: accepted direction, pre-implementation. Owning decision:
[ADR 0002](../adr/0002-one-key-gesture-grammar.md). Product framing:
`VISION.md` → "The Memory Layer".

## Summary

All memory-layer intent rides the existing main hotkey. The launcher opens on
key-down in 100% of cases; hold and double-tap escalate the already-open
window. Surfaces morph in place inside the main window. No new chords.

## Gesture Table

| Gesture | Window closed | Window open (launcher or Day Page) |
| --- | --- | --- |
| Tap | Launcher, visible on key-down | Toggle launcher ↔ Day Page |
| Hold (≥ hold threshold) | Day Page + dictation hot (push-to-talk) | — (window already open; see Open Questions) |
| Double-tap | Agent Chat | Agent Chat |
| Cmd+Enter | — | Agent Chat (established convention, unchanged) |
| Esc | — | Dismiss (the only dismiss) |

## State Machine

```text
CLOSED
  └─ key-down ──────────────► LAUNCHER_VISIBLE (render immediately, no waiting)
        │
        ├─ key-up < HOLD_MS ───► TAP_PENDING
        │     ├─ key-down < DOUBLE_MS ──► AGENT_CHAT (morph in place)
        │     └─ DOUBLE_MS elapses ─────► LAUNCHER (plain tap, steady state)
        │
        └─ key still down at HOLD_MS ──► DAY_PAGE + DICTATION_ACTIVE
              └─ key-up ──► dictation stops, transcript commits to today's
                            page, caret placed after committed text

LAUNCHER (open, steady)
  ├─ tap ──► DAY_PAGE  (in-flight query text carries over as capture start)
  ├─ double-tap ──► AGENT_CHAT
  ├─ Cmd+Enter ──► AGENT_CHAT
  └─ Esc ──► CLOSED

DAY_PAGE (open, steady)
  ├─ tap ──► LAUNCHER
  ├─ double-tap / Cmd+Enter ──► AGENT_CHAT
  └─ Esc ──► CLOSED
```

Proposed timings (tune against feel, not benchmarks):

- `HOLD_MS` ≈ 250ms — long enough that fast taps never trip it, short enough
  that push-to-talk feels immediate.
- `DOUBLE_MS` ≈ 300ms — second key-down window after a tap's key-up.

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
4. **Query carry-over.** Toggling launcher → Day Page with text in the search
   input moves that text into the Day Page as the start of a capture. Toggling
   back does not resurrect it in the launcher (it now belongs to the page).
5. **Push-to-talk dictation.** On hold, dictation starts at the hold threshold
   while the key is down; key-up stops capture and commits the transcript to
   today's page with a timestamp. Dictation readiness reuses the existing
   dictation pipeline and its setup NUX; if the model/mic is not ready, the
   hold gesture still opens the Day Page with a visible dictation-unavailable
   hint rather than failing silently.
6. **Tap-to-dismiss is retired.** Tap-while-open toggles surfaces; Esc is the
   only dismiss. This is a deliberate habit break and should be called out in
   release notes / first-run hints.
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
URLs, promoted clipboard entries, dictation commits, Agent Chat traces — in
time order. Long content renders truncated as excerpt cards (the underlying
file structure already enforces this: long captures are fragment files
referenced by excerpt + link, per
[ADR 0003](../adr/0003-markdown-files-as-memory-substrate.md)).

## Open Questions

- Hold while the window is already open: dead gesture, or push-to-talk into
  the focused surface? (Lean: push-to-talk everywhere, but unvalidated.)
- Exact `HOLD_MS`/`DOUBLE_MS` values need feel-testing with the real hotkey
  path latency.
- Whether double-tap morphs to the in-window Chat Prompt or focuses the
  dedicated Agent Chat surface when one is already open.
- Key-repeat suppression while held (OS key-repeat must not retrigger).

## Verification Expectations

Per repo policy, prefer behavior tests and runtime proof over source audits:

- Gesture classification: unit tests on the extracted state machine (timings
  injected, no sleeps).
- Key-down render: devtools probe proving launcher target identity exists
  before `HOLD_MS` elapses after key-down.
- Morph-not-swap: window identity receipt (same window id across
  launcher → Day Page → Agent Chat transitions).
- Carry-over: `getState` receipt showing query text present in the Day Page
  editor after toggle.
