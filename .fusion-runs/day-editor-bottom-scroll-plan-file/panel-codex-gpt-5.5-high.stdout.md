## Role Findings

The candidate plan is the right minimal shape: keep the new capability as a shared `NotesEditor` primitive, but make the retry policy Day-owned.

Use `NotesEditor::focus_with_cursor_at_end` to call `InputState::set_selection(value.len(), value.len(), window, cx)`. That is the correct primitive because `set_selection` both focuses and asks the input to reveal the cursor.

Then make [DayPageView::focus_editor](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/day_page_view.rs:289) call it immediately and once again via `window.defer`. The immediate call covers already-laid-out reopen/popup cases. The deferred call covers first mount/rebind, where input layout and bounds may not exist yet.

Do not change Notes window focus behavior. The helper is safe because it is inert until invoked, and only Day Page should switch from `focus()` to `focus_with_cursor_at_end()`.

## Evidence And Assumptions

`InputState::set_selection` sets the byte selection, calls `scroll_to(end, None, cx)`, focuses, then notifies at [state.rs](/Users/johnlindquist/dev/script-kit-gpui/vendor/gpui-component/crates/ui/src/input/state.rs:1011).

`scroll_to` returns early without `last_layout` and `last_bounds` at [state.rs](/Users/johnlindquist/dev/script-kit-gpui/vendor/gpui-component/crates/ui/src/input/state.rs:1560), so a post-layout repeat is required for reliable first-open behavior.

Current Day Page load paths already place content cursor at end via [apply_loaded_content_to_editor](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/day_page_view.rs:88), and main Day open/reopen calls `focus_editor` at [day_page_view.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/day_page_view.rs:764). Popup close and Day switcher actions also return through `focus_editor` in [day_page_switcher.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/day_page_switcher.rs:169).

The best helper name is `focus_with_cursor_at_end`. I would not call it `scroll_to_bottom`, because the behavior is cursor-driven and relies on editor selection semantics.

## Failure Modes

Empty and short files should pass naturally: cursor `0` or content length reveals no scrollable overflow, but focus still lands on `input:day-page-editor`.

Long files are the real case: assert `focusedSemanticId === "input:day-page-editor"` and `editorScrollMetrics.maxScrollTop > 0`, then `scrollTop/liveScrollTop >= maxScrollTop - tolerance`.

Past day, note, and fragment rebinding should all be covered by callers that run `apply_loaded_content_to_editor` followed by `focus_editor`. If any bind path only loads and not focuses, fix that caller rather than broadening Notes behavior.

Popup Escape is important because user may manually page up, open actions, Escape, and expect Day Page to resume at the journal insertion point. The deferred focus should intentionally pull back to bottom.

Risk: external disk refresh currently uses `set_value`, not cursor-at-end. I would not auto-focus there unless the user-visible intent is “external refresh steals focus and bottom-scrolls.” That is different from open/reopen/rebind.

## Recommendation

Implement exactly this minimal patch:

1. Add `NotesEditor::focus_with_cursor_at_end`.
2. Change only `DayPageView::focus_editor` to call a private `focus_editor_at_end` immediately and again with `window.defer`.
3. Keep `load_value_with_cursor_at_end` unchanged.
4. Ensure every Day Page open/reopen/rebind/popup-close path calls `focus_editor` after content binding.

Smallest proof:

```bash
SCRIPT_KIT_AGENT_ARTIFACT_NAME=day-editor-bottom-scroll ./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui
PROBE_BINARY=target-agent/artifacts/day-editor-bottom-scroll/script-kit-gpui bun scripts/agentic/day-editor-bottom-focus-probe.ts
```

The probe should seed a long today file, open Day Page, assert focus and bottom scroll, page up, open/close actions, assert focus and bottom scroll again, then reopen Day Page and assert the same.

I would avoid a new source-audit test unless this behavior is expected to regress silently in CI without runtime probes. A compile check plus the DevTools probe is the right verification level for focus/render timing.

## Self Score

8/10. The design is minimal and matches GPUI timing constraints. Remaining uncertainty is whether one `window.defer` is always post-layout on every path; the proposed probe is the right way to prove that and should be the acceptance gate.


