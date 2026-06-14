**Work Left**

P0 1 is basically fixed. Do not add more padding. Keep these owners as-is and add/keep geometry proof:

- `src/components/notes_editor/render.rs::NotesEditor::render_input`
- `src/notes/window/render_editor_body.rs::NotesApp::render_editor_body`
- `src/notes/window/render_editor.rs::NotesApp::render_editor`

Guard it by asserting actual runtime geometry: `NotesEditor.y >= NotesTitlebar.y + NotesTitlebar.h` and `NotesFooter.y >= NotesEditor.y + NotesEditor.h`, including small window sizes if possible.

P0 2 is still open. Day links are white because `markdown_inline` being registered is not the same as inline grammar captures being applied in editable rendering. The heading is yellow because block markdown highlighting is active. Normal inline links like `[Screenflow](scriptkit://...)` depend on inline markdown parsing, and inline injection is intentionally disabled for performance.

The exact render path to verify is:

- `vendor/gpui-component/crates/ui/src/input/element.rs::TextElement::highlight_lines`
- into `vendor/gpui-component/crates/ui/src/highlighter/highlighter.rs::SyntaxHighlighter::styles`

Fastest correct implementation path is likely a visible-range inline markdown supplement inside `SyntaxHighlighter::styles` or a helper it calls. Treat this as the implementation hypothesis to validate first with a focused highlighter test:

- Gate to markdown only.
- Parse only the visible/requested range or line slice with `markdown_inline`.
- Apply `src/notes/markdown_queries/markdown_inline_highlights.scm`.
- Map captures through the existing highlight theme as `link_text`, `link_uri`, and autolink URI styles.
- Merge with existing block styles.
- Do not re-enable `markdown_inline` injection globally.

Likely related owners:

- `vendor/gpui-component/crates/ui/src/highlighter/highlighter.rs::SyntaxHighlighter::styles`
- `src/notes/markdown_queries/markdown_inline_highlights.scm`
- `src/notes/markdown_highlighting.rs`
- `src/protocol/types/elements_actions_scriptlets.rs::ElementEditorRuntimeInfo`
- `src/components/notes_editor/component.rs::NotesEditor::markdown_runtime_info_with_scroll`
- `scripts/agentic/day-notes-editor-runtime-parity-probe.ts`

Do not treat changing dotted captures to bare captures as the fix. It may be cleanup, but the reported failure is better explained by inline captures not being applied through the editable highlighter path.

**Proof Plan**

Add a behavior test through the real highlighter path, not a source audit and not screenshot-only inspection. Use markdown like:

```md
# Heading
[Screenflow](scriptkit://spine/file/screenflow)
<https://eggo-brand.wzrrd.sh/>
```

Assert exact byte ranges receive link styles for:

- `Screenflow` as `link_text`
- `scriptkit://spine/file/screenflow` as `link_uri`
- `https://eggo-brand.wzrrd.sh/` as autolink/link URI

Then extend runtime metadata/probe only if the sample is derived from the same `InputState` / `TextElement::highlight_lines` / `SyntaxHighlighter::styles` path used for rendering. The probe should assert Notes and Day both expose styled `link_text` and `link_uri`, while `inlineMarkdownInjectionDisabled` remains `true`.

Run:

```bash
./scripts/agentic/agent-cargo.sh test markdown_highlighting
./scripts/agentic/agent-cargo.sh test --lib notes
SCRIPT_KIT_AGENT_ARTIFACT_NAME=day-notes-editor-fix ./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui
bun scripts/agentic/day-notes-editor-runtime-parity-probe.ts --binary target-agent/artifacts/day-notes-editor-fix/script-kit-gpui
```

If `ElementEditorRuntimeInfo` changes, update the protocol serialization tests too.
