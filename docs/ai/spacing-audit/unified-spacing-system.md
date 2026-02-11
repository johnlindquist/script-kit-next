# Unified Spacing System Proposal

Snapshot date: 2026-02-11.

## Scope

Primary synthesis targets:
- `src/render_prompts/other.rs`
- `src/prompts/select/render.rs`
- `src/render_prompts/div.rs`
- `src/prompts/div/render.rs`
- `src/render_prompts/form/render.rs`
- `src/render_prompts/arg/render.rs`
- `src/prompts/template/render.rs`
- `src/components/prompt_input.rs`
- `src/components/prompt_footer.rs`

No Rust code changes were made. This is a documentation-only proposal.

## Goal

Define one spacing contract for prompt chrome and content lanes that can be applied consistently to:
- header
- body
- list
- footer

and make spacing ownership explicit across wrapper layers vs entity layers.

## Current Call-Chain Sketches

### 1) Select

```text
render_select_prompt (other.rs)
  -> prompt_shell_container(radius_lg) + prompt_shell_content(entity)
  -> SelectPrompt::render
      -> input lane
      -> choices lane (uniform_list)
      -> FocusablePrompt wrapper
```

Key references:
- `src/render_prompts/other.rs:119`
- `src/render_prompts/other.rs:121`
- `src/prompts/select/render.rs:97`
- `src/prompts/select/render.rs:150`
- `src/prompts/select/render.rs:326`

### 2) Div

```text
render_div_prompt (wrapper)
  -> prompt_shell_container(radius_lg)
  -> header lane (panel constants)
  -> divider lane
  -> prompt_shell_content(entity)
      -> DivPrompt::render
          -> container padding + scroll content
  -> PromptFooter
  -> actions overlay (absolute)
```

Key references:
- `src/render_prompts/div.rs:134`
- `src/render_prompts/div.rs:148`
- `src/render_prompts/div.rs:173`
- `src/render_prompts/div.rs:177`
- `src/prompts/div/render.rs:82`
- `src/prompts/div/render.rs:129`
- `src/render_prompts/div.rs:181`

### 3) Form

```text
render_form_prompt (wrapper)
  -> root shell (custom, not prompt_shell_container)
  -> scrollable body lane (token padding)
      -> FormPromptState entity content
  -> PromptFooter
  -> actions overlay (absolute)
```

Key references:
- `src/render_prompts/form/render.rs:185`
- `src/render_prompts/form/render.rs:206`
- `src/render_prompts/form/render.rs:222`
- `src/render_prompts/form/render.rs:268`

### 4) Arg

```text
render_arg_prompt (wrapper)
  -> root shell (custom, not prompt_shell_container)
  -> header lane (panel constants + cursor math)
  -> optional divider + list lane
  -> PromptFooter
  -> actions overlay (absolute)
```

Key references:
- `src/render_prompts/arg/render.rs:320`
- `src/render_prompts/arg/render.rs:337`
- `src/render_prompts/arg/render.rs:397`
- `src/render_prompts/arg/render.rs:408`
- `src/render_prompts/arg/render.rs:435`
- `src/render_prompts/arg/render.rs:494`

## Where Spacing Rules Are Scattered

### A) Header spacing has one constant set, but prompts mix in other sources

Shared panel header constants exist:
- `HEADER_PADDING_X = 16` (`src/panel.rs:53`)
- `HEADER_PADDING_Y = 8` (`src/panel.rs:58`)
- `HEADER_GAP = 12` (`src/panel.rs:61`)
- `HEADER_TOTAL_HEIGHT` (`src/panel.rs:72`)

But usage diverges:
- Div/Arg header lanes use panel constants (`src/render_prompts/div.rs:148`, `src/render_prompts/arg/render.rs:337`).
- Form has no explicit header lane but still uses header-derived overlay offset helper (`src/render_prompts/form/render.rs:17`).
- Select has no panel-header lane; input spacing is local/token mixed (`src/prompts/select/render.rs:154`).

### B) Body horizontal inset is inconsistent across representative prompts

- Select input uses token inset `spacing.item_padding_x`, but list lane is hardcoded `px(8)` (`src/prompts/select/render.rs:154`, `src/prompts/select/render.rs:332`).
- Div wrapper header uses `16`, while entity body default padding resolves to `padding_md` (typically denser than header lane) (`src/render_prompts/div.rs:148`, `src/prompts/div/types.rs:120`, `src/prompts/div/render.rs:129`).
- Form body uses `padding_xl` (`src/render_prompts/form/render.rs:206`), much wider than footer inset.
- Template body uses token-driven insets (`src/prompts/template/render.rs:47`, `src/prompts/template/render.rs:55`).

### C) List lane metrics are partly shared, partly local

Shared metrics exist:
- `LIST_ITEM_HEIGHT` used by Select and Arg rows (`src/prompts/select/render.rs:284`, `src/render_prompts/arg/render.rs:297`).
- `PROMPT_INPUT_FIELD_HEIGHT` exists (`src/panel.rs:76`) and is used by Select/Template fields (`src/prompts/select/render.rs:153`, `src/prompts/template/render.rs:154`).

But list container/radius rules diverge:
- Select row radius hardcoded `8` and root radius hardcoded `12` (`src/prompts/select/render.rs:285`, `src/prompts/select/render.rs:343`).
- Select list lane inset hardcoded `8` (`src/prompts/select/render.rs:332`).
- Arg list lane vertical padding tokenized (`src/render_prompts/arg/render.rs:408`), divider inset tokenized (`src/render_prompts/arg/render.rs:397`).

### D) Footer has a strong internal contract, but external alignment is inconsistent

Footer contract is centralized in component constants:
- fixed height via `FOOTER_HEIGHT` (`src/components/prompt_footer.rs:493`, `src/window_resize/mod.rs:229`)
- container insets/gaps (`src/components/prompt_footer.rs:54`, `src/components/prompt_footer.rs:42`, `src/components/prompt_footer.rs:44`)
- button internals (`src/components/prompt_footer.rs:366`)

But prompt body/header lanes do not align to footer inset consistently:
- Footer horizontal inset is fixed `12` (`src/components/prompt_footer.rs:54`, `src/components/prompt_footer.rs:498`).
- Arg/Div header lanes use `16` (`src/render_prompts/arg/render.rs:337`, `src/render_prompts/div.rs:148`).
- Form body lane is `padding_xl` (`src/render_prompts/form/render.rs:206`).

### E) Shared input component is not the layout authority for these prompts

`PromptInput` has a clear cursor/spacing model (`src/components/prompt_input.rs:454`, `src/components/prompt_input.rs:463`), but Arg/Select/Form/Div render key input/list geometry directly in prompt-specific code.

## Unified Spacing Contract (Proposal)

Define one semantic metrics struct for prompt spacing ownership.

```text
PromptSpacingContract {
  radius_shell_lg
  lane_inset_x
  lane_gap_header
  lane_inset_y_header
  lane_inset_y_body
  lane_inset_y_list
  lane_inset_y_empty_state
  row_height_list
  row_radius_list
  footer_height
  footer_inset_x
  footer_section_gap
  footer_button_gap
}
```

### Proposed source mapping

- `radius_shell_lg`: `tokens.visual().radius_lg`
- `lane_inset_x`: panel header lane inset (`HEADER_PADDING_X`) as canonical prompt chrome inset
- `lane_gap_header`: `HEADER_GAP`
- `lane_inset_y_header`: `HEADER_PADDING_Y`
- `lane_inset_y_body`: token `spacing.padding_lg` for general body sections
- `lane_inset_y_list`: token `spacing.padding_xs` for list wrappers
- `lane_inset_y_empty_state`: token `spacing.padding_xl`
- `row_height_list`: `LIST_ITEM_HEIGHT`
- `row_radius_list`: tokenized radius (target: `visual.radius_md`; remove local literals)
- `footer_height`: `FOOTER_HEIGHT`
- `footer_inset_x`: same as `lane_inset_x` (instead of separate fixed 12)
- `footer_section_gap`: keep current `8` initially
- `footer_button_gap`: keep current `4` initially

## Header/Body/List/Footer Pattern (Target)

### Header

- Wrapper-owned lane.
- Always uses `lane_inset_x`, `lane_inset_y_header`, `lane_gap_header`.
- Prompts without visible header must expose `header_height = 0` to overlay-offset logic.

### Body

- Prompt wrapper defines outer body lane width with `lane_inset_x`.
- Prompt entities may define inner section rhythm, but should not redefine outer lane inset.
- Body section vertical rhythm defaults to `lane_inset_y_body`.

### List

- List container uses the same `lane_inset_x` as header/body.
- List wrapper vertical breathing uses `lane_inset_y_list`.
- Empty-state vertical breathing uses `lane_inset_y_empty_state`.
- Rows use `row_height_list` and tokenized `row_radius_list`.

### Footer

- Footer frame remains fixed-height (`footer_height`).
- Footer outer horizontal inset aligns to `lane_inset_x`.
- Internal footer sections keep existing `section_gap`/`button_gap` in phase 1 to limit visual churn.

## Prompt-Specific Alignment Notes

- Select: convert list lane inset from hardcoded `8` to shared `lane_inset_x`; replace root/row hardcoded radii with contract radii (`src/prompts/select/render.rs:332`, `src/prompts/select/render.rs:343`, `src/prompts/select/render.rs:285`).
- Div: align entity content padding default and footer inset with wrapper lane inset (`src/render_prompts/div.rs:148`, `src/prompts/div/types.rs:120`, `src/components/prompt_footer.rs:54`).
- Form: align body inset and footer inset; decouple actions-overlay top from header assumptions when no header is rendered (`src/render_prompts/form/render.rs:206`, `src/render_prompts/form/render.rs:17`).
- Arg: align footer inset with header/list lane and keep tokenized list breathing; optionally converge cursor layout on `PromptInput` behavior later (`src/render_prompts/arg/render.rs:337`, `src/render_prompts/arg/render.rs:408`, `src/components/prompt_input.rs:463`).

## Overlay Offset Rule (Target)

Current helper:
- `top = HEADER_TOTAL_HEIGHT + padding_sm - border_thin`
- `right = padding_sm`

Reference:
- `src/render_prompts/arg/helpers.rs:2`

Proposed:
- Resolve top anchor from prompt-specific `header_height` in the spacing contract, not globally from `HEADER_TOTAL_HEIGHT`.
- Keep right anchor tied to lane inset scale (`padding_sm` or a contract alias).

## Migration Order

1. Introduce `PromptSpacingContract` in shared prompt layout plumbing (no visual change initially).
2. Route Arg/Div/Form wrappers to contract-derived lane inset values.
3. Align `PromptFooter` outer inset to contract lane inset; keep internal footer gaps unchanged in first pass.
4. Align Select list lane and radii to contract.
5. Update overlay offset helper to consume per-prompt header-height contract.

## Risks / Known Gaps

- Changing footer outer inset from `12` to lane-aligned inset will visibly shift footer content.
- Form currently has no explicit header lane but uses header-based overlay top math.
- Select currently has no footer; adopting a footer lane policy may require product/UX decisions beyond spacing normalization.
- `PromptInput` is not yet the active source of truth for Arg/Select input rendering paths.
