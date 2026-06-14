**What’s left**

The PR-sized fix is two narrow changes plus proof.

1. **Notes clipping**
The likely root cause is layout ownership/bypass: editable `NotesEditor::render_input` ignores `NotesEditorLayout`, while preview uses it, and Notes still calls the raw/static path. The exact titlebar clipping mechanism still needs runtime bounds proof.

Implement:

- [src/components/notes_editor/render.rs](/Users/johnlindquist/dev/script-kit-gpui/src/components/notes_editor/render.rs): make `NotesEditor::render_input(&self, cx)` own editable layout:
  - wrap `render_input_state(...)`
  - apply `flex_1()`, `min_h(px(0.))`, `h_full()`
  - apply `px(px(self.layout.padding_x))` / `py(px(self.layout.padding_y))`

- [src/notes/window/render_editor_body.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window/render_editor_body.rs): stop calling `NotesEditor::render_input_state(&self.editor_state, cx)` directly. Use the entity path:
  - `self.notes_editor.read(cx).render_input(cx)`

- [src/notes/window/render_editor.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window/render_editor.rs): remove the outer `.px(metrics.editor_padding_x).py(metrics.editor_padding_y)` once `NotesEditor` owns padding, or Notes will double-pad. Keep the height chain explicit with `flex_1`, `min_h(px(0.))`, and `h_full` where needed.

- [src/main_sections/day_page_view.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/day_page_view.rs): keep using `self.notes_editor.read(cx).render_input(cx)`. Do not add a Day-local editor path or revive the old inline popup.

2. **Day markdown highlighting**
This is not proven by `language: markdown` or highlighter registration metadata. The concrete source issue is the active markdown query using unsupported capture names.

Implement:

- [src/notes/markdown_queries/markdown_highlights.scm](/Users/johnlindquist/dev/script-kit-gpui/src/notes/markdown_queries/markdown_highlights.scm):
  - change `@text.uri` to `@link_uri`
  - change `@text.reference` to `@link_text`

Do **not** enable `markdown_inline` injection. Existing tests intentionally forbid that for editable markdown performance.

3. **Tests/probes to update**
- Update the existing source-audit assertion that expects Notes to call `render_input_state(&self.editor_state, ...)`; it should now assert the Notes body uses the entity-owned `render_input(cx)` path.
- Add or extend a markdown highlighting behavior test so `[Script Kit](https://scriptkit.com)` produces supported captures or non-default style spans for link ranges.
- Extend DevTools/runtime coverage. Existing metadata/style parity probes are not enough unless they check real geometry and visible pixels/spans.

**Verification commands**

```bash
./scripts/agentic/agent-cargo.sh test markdown_highlighting
./scripts/agentic/agent-cargo.sh test --lib notes
SCRIPT_KIT_AGENT_ARTIFACT_NAME=day-notes-editor-fix ./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui
PROBE_BINARY=target-agent/artifacts/day-notes-editor-fix/script-kit-gpui bun scripts/agentic/day-notes-editor-host-contract-probe.ts
```

The DevTools probe must prove both P0 symptoms directly:

- Notes: seeded first line has bounds below the titlebar/editor body top, with screenshot/JSON receipt.
- Day: seeded markdown link has visible non-default link styling, verified by style spans or pixel sampling, not just `markdownRegistered: true`.
