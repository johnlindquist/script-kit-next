## Role Findings

P0 1 is functionally fixed by the current patch. The remaining work is to ratchet the geometry proof, not move more layout code.

Owners:
- `src/components/notes_editor/render.rs::NotesEditor::render_input`
- `src/components/notes_editor/render.rs::NotesEditor::render_input_state`
- `src/notes/window/render_editor_body.rs` call site using `self.notes_editor.read(cx).render_input(cx)`
- `src/notes/window/render_editor.rs` where the removed outer padding must stay removed

The runtime sample `NotesTitlebar y=0 h=36`, `NotesEditor y=36 h=216`, `NotesFooter y=252` proves the editor starts exactly below the titlebar. Add/keep a probe assertion that `editor.y >= titlebar.y + titlebar.h` and `footer.y >= editor.y + editor.h` for Notes, plus the existing shared render path checks.

P0 2 is not fixed yet. Day links are white because active markdown highlighting only proves the block markdown parser is running. `# heading` highlights because `src/notes/markdown_queries/markdown_highlights.scm` captures block grammar `(atx_heading (inline) @title)`. Inline links like `[Screenflow](scriptkit://...)` are parsed by `tree_sitter_md::INLINE_LANGUAGE`, and the current contract intentionally disables `markdown_inline` injection in `src/notes/markdown_highlighting.rs`. So `markdownInlineRegistered true` means the inline language exists, not that it is applied to editable text.

## Evidence And Assumptions

Evidence:
- `NotesEditor::new_markdown_pair` configures `InputState::code_editor("markdown")` in `src/components/notes_editor/component.rs`.
- `gpui-component` applies syntax in `vendor/gpui-component/crates/ui/src/input/element.rs::TextElement::highlight_lines` via `highlighter.styles(&range, &cx.theme().highlight_theme)`.
- The block query has `@title`, `@link_uri`, `@link_text`, but inline links require the inline grammar.
- `src/notes/markdown_queries/markdown_inline_highlights.scm` has the relevant captures for `(link_destination)`, `(uri_autolink)`, `(link_label)`, and `(link_text)`.
- `src/notes/markdown_highlighting.rs` explicitly keeps `markdown_inline` out of markdown injections for performance.

Assumption: the current screenshot text is in editable mode, not preview mode. Preview link styling is a separate path in `src/notes/markdown.rs` and is not the failing path.

## Failure Modes

- Re-enabling `markdown_inline` in `MARKDOWN_INJECTION_LANGUAGES` or `markdown_injections.scm` fixes colors but regresses scroll perf. Do not do this.
- Day-only styling fixes the screenshot but breaks the shared NotesEditor contract.
- Only asserting `markdownRegistered`, query fingerprints, or screenshot color is too weak. It misses exactly this bug: block highlighting active while inline captures are absent.
- A source-audit test for exact query text would be brittle. Prefer a behavior/highlighter test that proves ranges and styles.

## Recommendation

Fastest correct path:

1. Add a visible-range inline markdown overlay in the shared editable highlighting path.

Best owner:
- `vendor/gpui-component/crates/ui/src/highlighter/highlighter.rs::SyntaxHighlighter::styles`

Implementation shape:
- Keep the existing block markdown pass.
- If `self.language == "markdown"`, run a supplemental inline pass only for the requested `range`.
- Parse only that visible line/range with `markdown_inline` config from `LanguageRegistry`.
- Convert inline captures to the same `HighlightStyle` through `theme.style("link_text")` and `theme.style("link_uri")`.
- Combine with block styles using the existing highlight combination/merge behavior.
- Do not add `markdown_inline` to markdown injections.

If you want less vendor-specific surface area, add a small helper inside the same file, for example:
- `SyntaxHighlighter::markdown_inline_styles_for_visible_range`
- `SyntaxHighlighter::styles` calls it only for markdown

2. Normalize capture names in `src/notes/markdown_queries/markdown_inline_highlights.scm`.

Change:
- `@link_uri.markup` -> `@link_uri`
- `@link_text.markup` -> `@link_text`

Reason: vendored `SyntaxColors::style` maps exact names like `link_uri` and `link_text`; dotted suffixes only work if the resolver intentionally falls back by prefix. The block query already uses exact names.

3. Add semantic proof before screenshot proof.

Good test owners:
- `vendor/gpui-component/crates/ui/src/highlighter/highlighter.rs` tests, or a repo test that can instantiate `SyntaxHighlighter`
- `src/notes/markdown_highlighting.rs` tests for query compile plus runtime contract

Test assertion:
- Input: `# Heading\n[Screenflow](scriptkit://spine/file/screenflow)\n<https://eggo-brand.wzrrd.sh/>`
- Assert returned styles include non-default `color` or font style over exact byte ranges for:
  - `Screenflow` as `link_text`
  - `scriptkit://spine/file/screenflow` as `link_uri`
  - autolink URL as `link_uri`
- Assert `markdown_editor_runtime_info().inline_markdown_injection_disabled == true`.

4. Extend runtime probe evidence.

Owner:
- `scripts/agentic/day-notes-editor-runtime-parity-probe.ts`

Add a check that consumes new structured runtime metadata, not pixels. If needed, extend:
- `src/protocol/types/elements_actions_scriptlets.rs::ElementEditorRuntimeInfo`
- `src/components/notes_editor/component.rs::markdown_runtime_info_with_scroll`
- `src/notes/markdown_highlighting.rs`

Expose a compact `highlightSample` for the editor value/ranges, e.g. `{ role, text, range, styled }`, and assert Notes and Day both report styled `link_text` and `link_uri` while `inlineMarkdownInjectionDisabled` remains true.

Verification command set:
- `./scripts/agentic/agent-cargo.sh test markdown_highlighting`
- `./scripts/agentic/agent-cargo.sh test --lib notes`
- focused new highlighter test
- `SCRIPT_KIT_AGENT_ARTIFACT_NAME=day-notes-editor-fix ./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui`
- `bun scripts/agentic/day-notes-editor-runtime-parity-probe.ts --binary target-agent/artifacts/day-notes-editor-fix/script-kit-gpui`

## Self Score

8/10. The layout conclusion is strong. The link root cause is strong because it explains the exact heading-vs-link split and preserves the perf constraint. The only uncertainty is the cleanest implementation seam inside the vendored highlighter, which should be resolved by a tiny focused test before editing.
