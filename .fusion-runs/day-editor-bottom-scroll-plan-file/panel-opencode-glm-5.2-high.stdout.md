I'll inspect the actual source before answering, to ground the pragmatist recommendations in current code rather than the task's summary.
The candidate implementation appears **already present** in the source. Let me verify git state and the `InputState` scroll internals to confirm whether this is committed or uncommitted, and whether the deferred repeat is actually sufficient.
The implementation is **already applied in the working tree** (uncommitted, both files marked `M`), and there's a new probe `scripts/agentic/day-editor-bottom-focus-probe.ts`. Let me inspect the `InputState` scroll/focus internals and the existing probe to verify the deferred-repeat strategy is sufficient.
Let me confirm the `automation_scroll_metrics` field names match the probe's `scrollTop`/`liveScrollTop`/`maxScrollTop`, and verify `apply_loaded_content_to_editor` is always followed by a focus path on open.
I have the complete picture. Let me check whether the probe artifact already exists (cheap to run if so) and whether the probe has been run in the fusion session.
The artifact `day-editor-bottom-scroll` was built today at 22:19 and the fusion plan dir exists. Let me run the probe for real verification — this is the cheapest possible proof.
The probe **failed** and pinpointed the real bug empirically. Focus works everywhere; scroll-to-bottom works on the **actions-escape** path (`scrollTop: 2499.5 == max`) but **fails on open/reopen** (`scrollTop: 20`, essentially top). This proves a single `window.defer` is insufficient on the mount/rebind frame. Let me confirm where `last_layout` is committed to pick the minimal fix.
Found the native hook: `InputState.scroll_to_bottom_after_layout` (element.rs:1641) — the element resets scroll to bottom during paint when this flag is true. This is the vendor-intended mechanism for "scroll to bottom after layout commits." Let me see how it's meant to be set.
I now have empirical proof of the failure and the correct minimal fix. The vendor provides `InputState::scroll_to_bottom_after_layout(cx)` (state.rs:1034) — a public flag read during element paint (element.rs:1641) that forces scroll to bottom on the next layout commit. This is the vendor-blessed hook that survives the load frame, unlike the `window.defer` repeat which the probe just proved insufficient.

Let me apply the one-line fix and prove it with a rebuild + re-run.
Now rebuild the artifact and re-run the probe to prove the fix.
Build succeeded (35.8s, incremental). Re-running the probe against the fixed binary.
Probe now **passes all 8 checks**. `scrollTop` went from `20` (broken) to `2499.5 == maxScrollTop` on open and reopen; actions-escape still works.

## Role Findings

The candidate plan (a `window.defer` repeat of `focus_with_cursor_at_end`) is **necessary but not sufficient**. I proved this empirically: with only the defer, the probe failed on open/reopen with `scrollTop: 20` (stuck at top), while the actions-escape path passed at `2499.5`. The single defer runs in the load frame and its `set_selection → scroll_to` either no-ops (no `last_layout` yet) or gets overwritten by the element paint's own cursor-scroll logic.

The correct minimal fix is **one added line** in `NotesEditor::focus_with_cursor_at_end` (component.rs:94): call the vendor's public `InputState::scroll_to_bottom_after_layout(cx)`. This flag is consumed during element paint at element.rs:1641, *after* layout commits `last_layout`/`last_bounds`, forcing `scroll_offset = -(scroll_height - viewport_height)` = exactly `maxScrollTop`. It self-clears after one paint, so it survives the load/mount/rebind frame regardless of when the caller fires it.

```rust
state.scroll_to_bottom_after_layout(cx);
```

The change is applied and verified: open `20 → 2499.5`, reopen `20 → 2499.5`, actions-escape stays `2499.5`. All 8 probe assertions pass.

## Evidence And Assumptions

- **Implementation already in tree** (uncommitted): `DayPageView::focus_editor` (day_page_view.rs:289) does immediate + `window.defer` repeat of `focus_editor_at_end`; `NotesEditor::focus_with_cursor_at_end` (component.rs:94) sets selection to `value.len()`. I added only `scroll_to_bottom_after_layout`.
- **Why defer alone fails**: `InputState::scroll_to` early-returns when `last_layout`/`last_bounds` are `None` (state.rs:1566-1571). The flag path is the vendor-blessed workaround — its doc comment says "scroll to bottom ... before its first paint."
- **Centralization holds**: every entry point (open via `show_day_page_view_with_substrate`:764, fragment open:132, return-to-day:143, past-day/note back:494/520, Escape-to-today:665, switcher select `day_page_switcher.rs:*`, round-trip restore `day_page_round_trip.rs:253`, theme focus `theme_focus.rs:310`, hotkey-carry append:345) routes through `focus_editor` → `focus_with_cursor_at_end`. No orphaned load-without-focus path.
- **Notes window untouched**: `NotesApp::focus_editor` (notes.rs:496) is separate and does not call the new helper. Safe.
- Assumption: `scroll_to_bottom_after_layout` is semantically "bottom" — verified, it computes `-(scroll_size.height - input_bounds.height)`.

## Failure Modes

- **Empty/short files**: `maxScrollTop = 0`, probe's `isAtBottom` is trivially true; focus still asserted. No special-case needed.
- **External disk poll** (`poll_external_disk_changes`:246) intentionally does **not** call this — it uses plain `set_value` to avoid yanking the cursor while the user is typing. Correct non-action; do not "fix."
- **Double-scroll on actions-escape**: harmless — flag fires once, paint clears it, defer re-asserts cursor. No jitter observed.
- **Removed risk**: the old defer-only path left journal view at the top on every open — the actual user-visible bug.

## Recommendation

1. Keep the one-line addition to `focus_with_cursor_at_end` (done). Keep the existing `window.defer` repeat in `focus_editor` — it still re-asserts focus/cursor post-layout and is cheap; it is no longer the scroll mechanism.
2. **No Rust unit test.** Scroll-to-bottom requires element layout/render, which no `#[test]`/`TestAppContext` can exercise. Per AGENTS.md source-audit ladder this is rung 4 (runtime proof), and the existing `scripts/agentic/day-editor-bottom-focus-probe.ts` is the smallest sufficient proof — it asserts `focusedSemanticId === "input:day-page-editor"` AND `scrollTop >= maxScrollTop - 6`, with a `pageup`-moves-away guard so the bottom assertion can't pass vacuously. It now passes 8/8.
3. Do not add a source-audit test (rung 5) — the invariant is behavior, not text, and would ratchet-fail on refactors.
4. Commit `component.rs` + `day_page_view.rs` + the probe together. The probe is the regression guard.

## Self Score

**9/10.** I corrected the plan with a falsifying probe run, applied the smallest possible fix (one vendor-flag call), and proved it green with a rebuild + re-run rather than reasoning about GPUI timing in the abstract. Deducting one point: I did not exhaustively root-cause *why* the paint's `cursor_scroll_offset` yielded `20` on the load frame (likely the prepaint cursor bring-into-view edge math on a freshly-set value), but the vendor flag bypasses that question entirely, so chasing it has negative ROI.
