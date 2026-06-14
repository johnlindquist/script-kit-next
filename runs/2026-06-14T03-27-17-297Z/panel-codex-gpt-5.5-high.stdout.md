## Role Findings

Two PR-sized fixes are left:

1. **Notes clipping:** make the shared editable `NotesEditor` own its layout padding, then remove/simplify duplicate host padding. Current editable rendering ignores `NotesEditorLayout`, while preview uses it. Fix in `src/components/notes_editor/render.rs`, then update `src/notes/window/render_editor.rs`, `src/notes/window/render_editor_body.rs`, and `src/main_sections/day_page_view.rs` so there is one padding owner and the editor chain has `flex_1 + min_h(0) + h_full` consistently.

2. **Day markdown highlighting:** fix the active markdown highlight query, not Day-specific rendering. `markdown_inline_highlights.scm` has valid `@link_text` / `@link_uri` captures, but inline injection is intentionally disabled. The active `markdown_highlights.scm` uses `@text.uri` and `@text.reference`, which are not supported highlight names in `gpui-component`’s theme registry. Change those to supported names and add a unit test proving a markdown link produces non-default highlight styles.

## Evidence And Assumptions

Relevant files:

- `src/components/notes_editor/render.rs`: `render_input_state` returns only `div().h_full().child(editor)` and ignores `self.layout`.
- `src/components/notes_editor/component.rs`: shared constructor correctly calls `.code_editor("markdown")`, disables dynamic bottom margin, and registers the highlighter.
- `src/notes/window/render_editor.rs`: Notes adds editor padding outside the shared component.
- `src/main_sections/day_page_view.rs`: Day also constructs a shared `NotesEditorLayout`, but the editable render path does not consume it.
- `src/notes/markdown_queries/markdown_highlights.scm`: active editable markdown query uses unsupported `@text.uri` / `@text.reference`.
- `vendor/gpui-component/crates/ui/src/highlighter/registry.rs`: supported names include `link_text`, `link_uri`, `title`, `text.literal`, punctuation captures, etc.
- `scripts/agentic/day-notes-editor-runtime-parity-probe.ts`: already proves shared editor owner/runtime/scroll parity, but not pixel-visible highlighting or top clipping.

Assumption: the screenshots are from editable mode, not preview mode. If preview mode is involved, this changes the clipping owner but not the Day highlight diagnosis.

## Failure Modes

- Moving padding only in Notes will make Day diverge again. Fix belongs in `NotesEditor`, then hosts should become thinner.
- Re-enabling `markdown_inline` injection would likely restore inline coloring but violates the existing performance contract: current tests explicitly forbid inline markdown injection in editable markdown.
- Adding another source-audit test is the wrong default. This is visual/runtime behavior; prefer a behavior unit for highlight spans and a DevTools screenshot/layout proof.
- Runtime metadata can lie by omission here: `language: markdown` and `markdownRegistered: true` do not prove visible token color.

## Recommendation

Implement in this order:

1. In `src/components/notes_editor/render.rs`, add a layout-aware editable render path, likely by changing `render_input(&self, cx)` to wrap `render_input_state` with `.px(px(self.layout.padding_x)).py(px(self.layout.padding_y)).flex_1().min_h(px(0.)).h_full()`. Keep `render_input_state` as the raw input renderer for tests/legacy callers, or add `render_input_with_layout`.

2. In `src/notes/window/render_editor.rs`, remove the outer `.px(...).py(...)` once shared layout owns it, and ensure the editor body wrapper is `flex_1().min_h(px(0.)).h_full().flex().flex_col()`.

3. In `src/notes/window/render_editor_body.rs`, use `self.notes_editor.read(cx).render_input(cx)` instead of raw `NotesEditor::render_input_state(&self.editor_state, cx)` so Notes gets the same shared layout path as Day.

4. In `src/notes/markdown_queries/markdown_highlights.scm`, replace unsupported captures:
   - `@text.uri` -> `@link_uri`
   - `@text.reference` -> `@link_text` or another supported capture if visual design wants yellow/link styling.
   Do not add `markdown_inline` to injections.

5. Extend `src/notes/markdown_highlighting.rs` tests with a real highlighter-style test for `[Script Kit](https://scriptkit.com)` that asserts the URL/link range gets a non-default color under `HighlightTheme::default_dark()`.

Verification commands:

```bash
./scripts/agentic/agent-cargo.sh test markdown_highlighting
SCRIPT_KIT_AGENT_ARTIFACT_NAME=day-notes-editor-fix ./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui
PROBE_BINARY=target-agent/artifacts/day-notes-editor-fix/script-kit-gpui bun scripts/agentic/day-notes-editor-runtime-parity-probe.ts
PROBE_BINARY=target-agent/artifacts/day-notes-editor-fix/script-kit-gpui bun scripts/agentic/day-page-style-parity-probe.ts
```

Add or extend one DevTools probe to capture visible proof: seed Notes and Day with a markdown link, capture the editor region, and assert non-background/non-foreground colored pixels in the link/URL row plus first-line top bounds below the titlebar. That is the green proof the current metadata probes do not provide.

## Self Score

8/10. The owner paths and likely causes are concrete. The only remaining uncertainty is the exact GPUI pixel geometry for the clipping screenshot, which needs the runtime screenshot/layout proof after the layout patch.
