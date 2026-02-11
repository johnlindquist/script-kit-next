# Tall Prompt Wrappers Spacing Audit

Snapshot date: 2026-02-11.

## Scope

Primary wrapper files audited:

- `src/render_prompts/editor.rs`
- `src/render_prompts/term.rs`
- `src/render_prompts/path.rs`

Supporting call-chain and spacing-source references:

- `src/main_sections/render_impl.rs`
- `src/components/prompt_layout_shell.rs`
- `src/render_prompts/arg/helpers.rs`
- `src/window_resize/mod.rs`
- `src/components/prompt_footer.rs`
- `src/prompts/path/render.rs`

No Rust code changes were made. This is a documentation-only audit.

## Render Call Chain

### Editor wrapper

- Dispatch path: `AppView::EditorPrompt` and `AppView::ScratchPadView`
  call `render_editor_prompt(...)`
  (`src/main_sections/render_impl.rs:179`,
  `src/main_sections/render_impl.rs:230`).
- `editor_prompt_shell_layout(self)` computes radius, explicit height,
  vibrancy fallback bg, and overlay offsets
  (`src/render_prompts/editor.rs:162`).
- Height contract comes from `window_resize::layout::MAX_HEIGHT`
  (`src/render_prompts/editor.rs:34`).
- Root shell uses shared `prompt_shell_container(...)` and then
  `.h(content_height)`
  (`src/render_prompts/editor.rs:329`,
  `src/render_prompts/editor.rs:330`).
- Content lane uses `prompt_shell_content(entity)`
  (`src/render_prompts/editor.rs:334`,
  `src/components/prompt_layout_shell.rs:86`).
- Footer is wrapper-owned via `PromptFooter::new(...)` as a sibling
  under content (`src/render_prompts/editor.rs:336`,
  `src/render_prompts/editor.rs:349`).
- Actions overlay is gated by `editor_prompt_actions_dialog(...)` and
  rendered via `editor_prompt_actions_overlay(...)`
  (`src/render_prompts/editor.rs:383`,
  `src/render_prompts/editor.rs:386`).

### Term wrapper

- Dispatch path: `AppView::TermPrompt` and
  `AppView::QuickTerminalView` call `render_term_prompt(...)`
  (`src/main_sections/render_impl.rs:176`,
  `src/main_sections/render_impl.rs:233`).
- `render_term_prompt` resolves overlay offsets and sets explicit
  `content_height = window_resize::layout::MAX_HEIGHT`
  (`src/render_prompts/term.rs:114`,
  `src/render_prompts/term.rs:132`).
- Root shell is manual, not `prompt_shell_container(...)`:
  `relative + flex_col + w_full + h(content_height) + overflow_hidden`
  (`src/render_prompts/term.rs:282`,
  `src/render_prompts/term.rs:289`, `src/render_prompts/term.rs:290`).
- Content lane is manual `flex_1 + min_h(0) + overflow_hidden`
  (`src/render_prompts/term.rs:294`).
- Footer is wrapper-owned via `PromptFooter::new(...)`
  (`src/render_prompts/term.rs:296`, `src/render_prompts/term.rs:297`).
- Actions overlay is inline `when_some(...)` with absolute full-surface
  layer and top-right anchored dialog
  (`src/render_prompts/term.rs:314`, `src/render_prompts/term.rs:335`,
  `src/render_prompts/term.rs:348`, `src/render_prompts/term.rs:349`).

### Path wrapper

- Dispatch path: `AppView::PathPrompt` calls `render_path_prompt(...)`
  (`src/main_sections/render_impl.rs:185`).
- `render_path_prompt` resolves overlay offsets with
  `prompt_actions_dialog_offsets(...)` (`src/render_prompts/path.rs:188`).
- Root shell is manual and fill-based:
  `relative + flex_col + w_full + h_full + overflow_hidden + rounded`
  (`src/render_prompts/path.rs:387`, `src/render_prompts/path.rs:394`,
  `src/render_prompts/path.rs:396`).
- Wrapper content child is `div().size_full().child(entity)` with no
  wrapper-level footer child (`src/render_prompts/path.rs:399`).
- `PathPrompt::render` owns header/content/hint grouping via
  `PromptContainer` (`src/prompts/path/render.rs:140`,
  `src/prompts/path/render.rs:143`, `src/prompts/path/render.rs:123`).
- Actions overlay uses full-surface absolute flex container with
  `pt(actions_dialog_top)` and `pr(actions_dialog_right)`
  (`src/render_prompts/path.rs:403`, `src/render_prompts/path.rs:408`,
  `src/render_prompts/path.rs:409`).

## Shell and Height Contracts

- Editor shell contract:
  shared shell helper + explicit `MAX_HEIGHT`
  (`src/render_prompts/editor.rs:329`, `src/render_prompts/editor.rs:330`,
  `src/render_prompts/editor.rs:34`).
- Term shell contract:
  manual shell + explicit `MAX_HEIGHT`
  (`src/render_prompts/term.rs:282`, `src/render_prompts/term.rs:289`,
  `src/render_prompts/term.rs:132`).
- Path shell contract:
  manual shell + `h_full` (no explicit pixel-height contract)
  (`src/render_prompts/path.rs:387`, `src/render_prompts/path.rs:394`).
- Shared max-height source used by editor/term:
  `window_resize::layout::MAX_HEIGHT`
  (`src/window_resize/mod.rs:242`).

## Content vs Footer Grouping

- Editor keeps content and footer at wrapper level:
  `prompt_shell_content(entity)` above `PromptFooter`
  (`src/render_prompts/editor.rs:334`, `src/render_prompts/editor.rs:336`).
- Term mirrors that grouping pattern, but with manual content container
  instead of `prompt_shell_content`
  (`src/render_prompts/term.rs:294`, `src/render_prompts/term.rs:296`).
- Path wrapper does not own a footer sibling. It delegates header/list/
  hint grouping to `PathPrompt` entity internals
  (`src/render_prompts/path.rs:399`, `src/prompts/path/render.rs:140`).

`PromptFooter` spacing rules that apply to editor/term wrappers:

- Fixed height `FOOTER_HEIGHT = 30`
  (`src/components/prompt_footer.rs:35`,
  `src/components/prompt_footer.rs:493`).
- Horizontal inset `12`, bottom inset `2`, section gap `8`, button gap `4`
  (`src/components/prompt_footer.rs:54`,
  `src/components/prompt_footer.rs:56`,
  `src/components/prompt_footer.rs:42`,
  `src/components/prompt_footer.rs:44`).

## Actions Dialog Overlay Insets

All three wrappers use the same inset helper:

- `prompt_actions_dialog_offsets(padding_sm, border_thin)`
  (`src/render_prompts/editor.rs:30`, `src/render_prompts/term.rs:115`,
  `src/render_prompts/path.rs:189`).

Shared inset formula:

```text
top   = HEADER_TOTAL_HEIGHT + padding_sm - border_thin
right = padding_sm
```

Formula source: `src/render_prompts/arg/helpers.rs:4` and
`src/render_prompts/arg/helpers.rs:5`.

Overlay composition differs by wrapper:

- Editor:
  full-surface backdrop includes modal scrim color and pointer cursor,
  plus outside-click close
  (`src/render_prompts/editor.rs:95`, `src/render_prompts/editor.rs:97`).
- Term:
  full-surface click-catcher exists, but no explicit backdrop `.bg(...)`
  on that catcher (`src/render_prompts/term.rs:339`,
  `src/render_prompts/term.rs:343`).
- Path:
  no backdrop click-catcher in overlay structure; overlay is a positioner
  for the dialog (`src/render_prompts/path.rs:403`,
  `src/render_prompts/path.rs:406`).

## Scattered Spacing Rule Map

- Tall shell frame primitives are centralized in
  `src/components/prompt_layout_shell.rs:79` and
  `src/components/prompt_layout_shell.rs:86`,
  but term/path re-implement equivalent shell/content rules locally.
- Explicit tall height is centralized (`MAX_HEIGHT`) for editor/term,
  but path still uses `h_full`.
- Footer spacing is centralized inside `PromptFooter` constants, but only
  editor/term wrappers consume that contract directly.
- Overlay inset math is centralized, while overlay backdrop behavior is
  wrapper-specific.

## Findings

- Editor is the closest to the canonical tall wrapper pattern:
  shared shell helper, explicit tall height, wrapper-level
  content/footer split.
- Term has near-identical grouping behavior, but spacing contract
  ownership is duplicated in local shell code.
- Path diverges in two places:
  no explicit tall height and no wrapper-level footer grouping.
- Overlay insets are shared across all three wrappers, while overlay
  interaction surfaces (scrim/click-catcher) are inconsistent.
