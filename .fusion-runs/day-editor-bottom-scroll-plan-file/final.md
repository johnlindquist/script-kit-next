**Final Recommendation**

Use the shared editor helper, but make the scroll guarantee come from GPUI’s layout-aware hook, not from `window.defer` alone.

Minimal code shape:

1. Add or keep `NotesEditor::focus_with_cursor_at_end(window, cx)`.
2. Inside it:
   - read `state.value().len()`
   - call `state.set_selection(end, end, window, cx)` for cursor placement and focus
   - call `state.scroll_to_bottom_after_layout(cx)` so the bottom scroll is applied during the next input paint/layout pass
3. Change only `DayPageView::focus_editor` to call that helper immediately and once via `window.defer`.
4. Treat the deferred call as an idempotent focus/cursor reaffirmation, not the load-bearing scroll mechanism.
5. Keep Notes window behavior unchanged. Verify with grep that only Day Page calls the new helper; make the helper `pub(crate)` if that works with the module boundaries.

`focus_with_cursor_at_end` is still the best helper name. It describes the user intent and the editor primitive. Avoid naming it `scroll_to_bottom`, because the actual behavior is cursor/focus placement plus a layout-time bottom reveal.

**Why This Fix**

`InputState::set_selection` attempts to scroll to the cursor, but that path can no-op before `last_layout` / `last_bounds` exist. That makes first open/reopen and fresh content rebind timing fragile. The stronger mechanism is `InputState::scroll_to_bottom_after_layout(cx)`, which is consumed by the input element during paint after layout has usable bounds.

So the corrected implementation is:

```rust
pub(crate) fn focus_with_cursor_at_end(
    &self,
    window: &mut Window,
    cx: &mut Context<Self>,
) {
    self.input.update(cx, |state, cx| {
        let end = state.value().len();
        state.set_selection(end, end, window, cx);
        state.scroll_to_bottom_after_layout(cx);
    });
}
```

Keep the Day-owned deferred repeat in `DayPageView::focus_editor`, but do not rely on defer alone for bottom scroll.

**External Refresh**

Do not automatically change `poll_external_disk_changes` unless the product decision is explicit. External disk refresh is different from open/reopen/rebind after user navigation: stealing focus and forcing bottom while the user is editing or elsewhere could be disruptive. Treat this as an intentional exception, not accidental missing coverage.

**Verification**

Primary proof should be a DevTools runtime probe, not a Rust unit test or source audit.

Create or extend `scripts/agentic/day-editor-bottom-focus-probe.ts` to assert:

- after opening Day Page with a long today file:
  - `focusedSemanticId === "input:day-page-editor"`
  - `editor_scroll_metrics.maxScrollTop > 0`
  - `scrollTop` / `liveScrollTop` is within tolerance of `maxScrollTop`
- after `PageUp`, verify scroll moved away from bottom so the test is not vacuous
- open a Day Page popup/action, close with Escape, then assert focus and bottom scroll again
- reopen Day Page and assert focus and bottom scroll again
- for empty/short content, assert focus and allow `maxScrollTop === 0`; do not require `scrollTop > 0`

Run cargo through the repo wrapper:

```bash
SCRIPT_KIT_AGENT_ARTIFACT_NAME=day-editor-bottom-scroll \
  ./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui

PROBE_BINARY=target-agent/artifacts/day-editor-bottom-scroll/script-kit-gpui \
  bun scripts/agentic/day-editor-bottom-focus-probe.ts
```

Rust tests are optional and should only cover narrow cursor plumbing, such as “helper sets selection to `value.len()`.” They cannot prove the visual scroll timing. Avoid a source-audit test for this behavior.

**Risks**

The main risk is overclaiming coverage. The probe above proves open, popup return, and reopen for long content; it does not automatically prove every possible Day Page bind route. If day switching, fragment return, or past/today navigation are part of the acceptance bar, add targeted probe steps for those paths rather than broadening Notes behavior.
