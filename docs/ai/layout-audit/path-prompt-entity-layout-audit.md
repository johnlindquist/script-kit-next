# Path Prompt Entity Layout Audit

## Scope
- Audited file: `src/prompts/path/render.rs`
- Cross-checked layout contracts:
  - `src/components/prompt_container.rs`
  - `src/components/prompt_header/component.rs`
  - `src/components/prompt_layout_shell.rs`

## Evidence Anchors
- `src/prompts/path/render.rs:30` list virtualization starts with `uniform_list("path-list", ...)`.
- `src/prompts/path/render.rs:93` header state derives from `PromptHeaderConfig` with actions-mode toggles.
- `src/prompts/path/render.rs:131` container uses `PromptContainerConfig::new().show_divider(true).hint(...)`.
- `src/components/prompt_container.rs:226` footer hint is centered by `render_hint` (`justify_center`).
- `src/components/prompt_header/component.rs:405` actions buttons/search are overlaid for CLS-free toggling.
- `src/components/prompt_header/component.rs:449` right-side header actions slot reserves fixed width via `actions_density`.
- `src/components/prompt_layout_shell.rs:63` fill slot contract enforces `flex_1 + min_h(0) + overflow_hidden`.

## Current Composition (Observed)
1. Path prompt uses a `PromptContainer` with `header + divider + flex list + hint footer`.
2. The list is virtualized via `uniform_list("path-list", filtered_count, ...)` and rendered in a `flex_1` slot.
3. Header actions state is driven by `actions_showing` and mapped to `PromptHeaderConfig.actions_mode(show_actions)`.
4. In actions mode, header focus is intentionally disabled with `.focused(!show_actions)`.

## Spacing/Alignment Findings
1. Vertical shell rhythm is internally consistent.
- `PromptContainer` defaults produce a stable column frame with clipped overflow and fill content behavior.
- `render.rs` correctly attaches list content to that fill slot (`.flex_1().w_full()`), so list height expansion/collapse is predictable.

2. Actions toggle is layout-stable (no horizontal shift).
- `PromptHeader` overlays button/search layers in a fixed-width right slot (`min_w(actions_density.reserved_min_width_px())`) and toggles by opacity/visibility.
- Path prompt relies on this correctly by only flipping `actions_mode` and not changing header structure.

3. Footer hint placement is consistent but can look visually "floating" on long text.
- Hint is centered (`justify_center`) with horizontal padding in `PromptContainer::render_hint`.
- Path prompt default hint string is long and wraps sooner than list rows on narrow widths, which can make bottom spacing feel uneven.

4. Empty-result positioning is under-specified.
- With `filtered_count == 0`, the virtualized list renders no rows and no explicit empty-state element.
- The center footer still shows guidance, but the content region becomes a blank block; this can be interpreted as alignment failure rather than zero results.

## Potential Unexpected Element Positioning
1. Long path prefix + query text share one inline flow in header input.
- The prefix is truncated with `max_w`, but there is no explicit spacer token between prefix and query text.
- Depending on truncation point/font fallback, query text can appear attached to the truncated path segment.

2. Actions mode intentionally hides cursor in the header.
- `.focused(!show_actions)` prevents dual-cursor visuals while actions dialog owns input.
- This is correct behavior, but should be documented as canonical so it is not "fixed" by adding a second active cursor later.

## Canonical Navigator Layout Rules (Recommended)
1. Use a four-zone shell for navigator prompts:
- Zone A: Header (`PromptHeader`), fixed-height feel, no per-prompt ad-hoc padding.
- Zone B: Divider (1px, tokenized margin from container config).
- Zone C: List region (`flex_1 + min_h(0) + overflow_hidden`) with virtualized rows.
- Zone D: Hint/footer, single rhythm band at bottom.

2. Header contract:
- Always pass `path_prefix` through shared header config for path-like prompts.
- Reserve a fixed right-side action slot (`actions_density`) for both normal and actions mode.
- Toggle actions state by `actions_mode` only; do not mount/unmount different header trees.

3. Actions-mode focus contract:
- When external actions dialog owns keyboard input, set header focus false (`focused(false)`) to avoid duplicate cursor affordances.
- Keep mirrored actions search text read-only in header; authoritative input remains the dialog.

4. List virtualization contract:
- Keep virtualized list in a fill slot and track scroll with a stable handle.
- Provide an explicit empty-state element for `0` items so the content zone communicates state rather than appearing misaligned.

5. Footer/hint rhythm contract:
- Treat footer as one line of assistive text by default; if text may wrap, constrain length or ellipsize.
- Keep footer padding tokenized and consistent across navigator prompts.

## Suggested Follow-up (Not implemented in this task)
1. Add explicit path-prompt empty-state content for `filtered_count == 0` in `src/prompts/path/render.rs`.
2. Standardize path-prefix/query separator in shared header input rendering to reduce visual ambiguity when prefix truncates.
3. Add layout tests (or screenshot tests) that cover:
- actions mode on/off with no horizontal shift,
- zero-result content state,
- long path prefix truncation with active query text.
