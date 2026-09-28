# Spec: One-Key Gesture Grammar

Status: implemented (T2/T8/T9/T11 in flight). Owning decision:
[ADR 0002](../adr/0002-one-key-gesture-grammar.md). Product framing:
`VISION.md` → "The Memory Layer".

## Summary

The existing main hotkey keeps the launcher contract: tap opens or closes the
main window. The launcher opens on key-down in 100% of cases; hold and
double-tap can escalate the already-open window before the tap resolves.
Every surface opens in place under the same input. No new chords.

## Gesture Table

| Gesture | Window closed | Window open (launcher or Day Page) |
| --- | --- | --- |
| Tap | Launcher, visible on key-down | Close main window |
| Hold (≥ hold threshold) | Day Page | — (window already open; see Open Questions) |
| Double-tap | Agent Chat | Agent Chat; while the chat is showing, a double-tap hides the window |
| Cmd+Enter | — | Agent Chat on an empty input; "Do It" once words are typed |
| Esc | — | Back one step (let go of what's staged, back one view, clear the input), then hide |

## State Machine

```text
CLOSED
  └─ key-down ──────────────► LAUNCHER_VISIBLE (render immediately, no waiting)
        │
        ├─ key-up < HOLD_MS ───► TAP_PENDING
        │     ├─ key-down < DOUBLE_MS ──► AGENT_CHAT (in place)
        │     └─ DOUBLE_MS elapses ─────► LAUNCHER (plain tap, steady state)
        │
        └─ key still down at HOLD_MS ──► DAY_PAGE
              └─ key-up ──► no-op (Day Page stays; dictation is owned by the
                            dictation hotkey and the voice key)

LAUNCHER (open, steady)
  ├─ tap ──► CLOSED
  ├─ double-tap ──► AGENT_CHAT
  ├─ Cmd+Enter (empty input) ──► AGENT_CHAT
  └─ Esc (nothing staged, empty input) ──► CLOSED

DAY_PAGE (open, steady)
  ├─ tap ──► CLOSED
  ├─ double-tap / Cmd+Enter ──► AGENT_CHAT
  └─ Esc ──► back one view

AGENT_CHAT (open, steady)
  ├─ tap / double-tap ──► CLOSED
  └─ Esc ──► back one view (the sentence goes back to the input)
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
2. **No window swaps.** Launcher, Day Page, and Agent Chat are views of the
   main window under the same input (the one-window host, `src/one_window/**`;
   the chat is a work view and `AppView` stays `ScriptList`). Notes open as a
   view under the same input (see Notes below). There is no second window.
3. **The input never moves.** The input row keeps its position and size across
   launcher, Day Page, and chat; the view replaces the list beneath it, and the
   first header stays in place. It must read as the same window showing
   something else, not a screen change.
4. **No tap carry-over.** Tapping while the launcher is open closes the main
   window. It must not move search input into the Day Page; Day Page entry is
   owned by hold-from-closed and explicit in-app triggers.
5. **No dictation on the main hotkey's hold.** Hold opens the Day Page and
   nothing else. Dictation is invoked through the dictation hotkey or the voice
   key, which open the voice view under the same input; the Day Page does not
   start, render, or manage dictation.
6. **Tap-to-dismiss remains.** Tap-while-open closes the main window, matching
   the established launcher hotkey convention. Esc remains a dismiss path too.
7. **Shared note behavior.** The Day Page and the note view under the main
   input must not fork note behavior: day notes stay `brain/days/<date>.md`
   and other notes stay in the notes database, whichever surface writes them
   (per the shared component contract in `CLAUDE.md`).

## Notes

There is no Notes window. A note opens as a view under the main input: its
lines are rows under a `NOTE · <name>` header and the input never moves. Its role is unchanged: a note about something *other than*
today — pinned reference material, a document being drafted alongside other
work. The Day Page and a note must feel identical because they share the same
storage and the same input.

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
- ~~Where double-tap lands~~ — **answered:** a clean quick chat under the same
  input, never the launcher's selected row as context
  (`one_window_open_chat(ChatEntry::quick())` in
  `src/main_sections/gesture_routing.rs`).
- ~~Key-repeat suppression while held~~ — **answered:** repeated `KeyDown`
  while already down are ignored (`src/hotkeys/gesture.rs`, unit-tested).

## Verification Expectations

Per repo policy, prefer behavior tests and runtime proof over source audits:

- Gesture classification: unit tests on the extracted state machine (timings
  injected, no sleeps).
- Key-down render: devtools probe proving launcher target identity exists
  before `HOLD_MS` elapses after key-down.
- No swap: window identity receipt (same window id across
  launcher → Day Page → Agent Chat transitions), with the input's y unchanged.
- Tap-while-open: runtime receipt showing the main window hides instead of
  opening the Day Page.
