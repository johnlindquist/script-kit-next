# Shared Components Spacing Audit

Snapshot date: 2026-02-11.

## Scope

Primary audit targets:
- `src/components/prompt_input.rs`
- `src/components/prompt_footer.rs`
- `src/components/focusable_prompt_wrapper.rs`
- `src/prompts/base.rs`
- `docs/ai/spacing-audit/shared-components.md`

Supporting references used to identify spacing drift:
- `src/components/prompt_header/component.rs`
- `src/render_prompts/arg/render.rs`
- `src/prompts/env/render.rs`
- `src/prompts/select/render.rs`
- `src/prompts/chat/render_core.rs`
- `src/prompts/path/render.rs`
- `src/panel.rs`
- `src/config/types.rs`

No Rust code changes were made. This is a documentation-only spacing audit.

## Shared Component Ownership

### PromptInput (`src/components/prompt_input.rs`)

Spacing/layout metrics currently owned by `PromptInput`:
- Default input padding is hardcoded to `top=8`, `bottom=8`, `left=16`, `right=16` (`src/components/prompt_input.rs:53`, `src/components/prompt_input.rs:54`, `src/components/prompt_input.rs:55`, `src/components/prompt_input.rs:56`).
- Config-derived padding copies `padding.top` to both top and bottom (`src/components/prompt_input.rs:63`, `src/components/prompt_input.rs:66`, `src/components/prompt_input.rs:67`).
- Effective config currently supports only `top/left/right` (no `bottom`) in `ContentPadding` (`src/config/types.rs:157`, `src/config/types.rs:159`, `src/config/types.rs:161`, `src/config/types.rs:163`).
- Cursor alignment behavior depends on `CURSOR_WIDTH + CURSOR_GAP_X` compensation when placeholder is shown (`src/components/prompt_input.rs:37`, `src/components/prompt_input.rs:454`, `src/components/prompt_input.rs:463`, `src/components/prompt_input.rs:474`).
- Rendered input row itself contributes no outer insets; it is a `flex_row/items_center/flex_1` content primitive (`src/components/prompt_input.rs:423`, `src/components/prompt_input.rs:425`, `src/components/prompt_input.rs:426`).

### PromptFooter (`src/components/prompt_footer.rs`)

Spacing/layout metrics currently owned by `PromptFooter`:
- Footer frame height is centralized through `FOOTER_HEIGHT` (`src/components/prompt_footer.rs:35`, `src/components/prompt_footer.rs:493`).
- Footer container insets are fixed constants: `padding_x=12`, `padding_bottom=2` (`src/components/prompt_footer.rs:54`, `src/components/prompt_footer.rs:56`, `src/components/prompt_footer.rs:498`, `src/components/prompt_footer.rs:500`).
- Section and button-cluster gaps are fixed constants: `section_gap=8`, `button_gap=4` (`src/components/prompt_footer.rs:42`, `src/components/prompt_footer.rs:44`, `src/components/prompt_footer.rs:436`, `src/components/prompt_footer.rs:457`, `src/components/prompt_footer.rs:522`).
- Button interior geometry is local and fixed: `gap=6`, `px=8`, `py=6`, `rounded=4` (`src/components/prompt_footer.rs:366`, `src/components/prompt_footer.rs:367`, `src/components/prompt_footer.rs:368`, `src/components/prompt_footer.rs:369`).
- Divider geometry is fixed: `1x16` with `mx=4` (`src/components/prompt_footer.rs:62`, `src/components/prompt_footer.rs:64`, `src/components/prompt_footer.rs:66`, `src/components/prompt_footer.rs:417`, `src/components/prompt_footer.rs:418`, `src/components/prompt_footer.rs:419`).

### FocusablePrompt Wrapper (`src/components/focusable_prompt_wrapper.rs`)

`FocusablePrompt` owns keyboard routing/focus only:
- Applies `key_context`, `track_focus`, and `on_key_down` (`src/components/focusable_prompt_wrapper.rs:133`, `src/components/focusable_prompt_wrapper.rs:137`, `src/components/focusable_prompt_wrapper.rs:138`).
- Defines no shared spacing/radius/inset contract for prompt roots.

### PromptBase / DesignContext (`src/prompts/base.rs`)

`PromptBase`/`DesignContext` currently own identity/callback/theme/color, but not spacing:
- `ResolvedColors` contains only color fields (`src/prompts/base.rs:102`, `src/prompts/base.rs:104`, `src/prompts/base.rs:112`).
- `DesignContext` exposes color helpers (`bg_main`, `text_primary`, etc.) with no inset/gap/radius fields (`src/prompts/base.rs:131`, `src/prompts/base.rs:188`, `src/prompts/base.rs:212`).

## Where Shared Defaults Conflict With Per-Prompt Hardcodes

### 1) PromptInput defaults are largely bypassed in production prompt paths

- `PromptInput::new(...)` is only used by `PromptHeader` in runtime prompt code (`src/components/prompt_header/component.rs:92`, `src/prompts/path/render.rs:105`).
- `PromptHeader` explicitly zeroes `PromptInput` padding via `.padding(Some(InputPadding::uniform(0.0)))` (`src/components/prompt_header/component.rs:71`).
- `PromptHeader` then imposes its own nested frame insets/radius (`pt 8`, `px 8`, `pb 6`, inner `px 10`, `py 6`, `rounded 10`) (`src/components/prompt_header/component.rs:97`, `src/components/prompt_header/component.rs:99`, `src/components/prompt_header/component.rs:100`, `src/components/prompt_header/component.rs:101`).
- Arg prompt bypasses `PromptInput` entirely and renders custom cursor math (`HEADER_PADDING_X/Y/GAP`, `ml(-CURSOR_WIDTH)`) (`src/render_prompts/arg/render.rs:337`, `src/render_prompts/arg/render.rs:338`, `src/render_prompts/arg/render.rs:342`, `src/render_prompts/arg/render.rs:381`).
- Env prompt also bypasses `PromptInput` and hardcodes field geometry (`px 16`, `py 12`, `gap 12`) (`src/prompts/env/render.rs:118`, `src/prompts/env/render.rs:119`, `src/prompts/env/render.rs:127`).

Consequence: `PromptInput` is not currently the effective source of truth for header/input spacing outside the path prompt header internals.

### 2) PromptFooter has one shared implementation, but chat re-implements the same geometry locally

- Shared footer constants define canonical frame/button/divider spacing (`src/components/prompt_footer.rs:42`, `src/components/prompt_footer.rs:44`, `src/components/prompt_footer.rs:54`, `src/components/prompt_footer.rs:56`, `src/components/prompt_footer.rs:366`, `src/components/prompt_footer.rs:368`).
- Most prompt wrappers consume `PromptFooter::new(...)` directly (`src/render_prompts/arg/render.rs:435`, `src/render_prompts/div.rs:181`, `src/render_prompts/editor.rs:349`, `src/render_prompts/form/render.rs:222`, `src/render_prompts/other.rs:253`, `src/render_prompts/term.rs:297`, `src/prompts/env/render.rs:296`).
- Chat prompt re-implements footer internals with duplicated literals (`gap 6`, `px 8`, divider `1x16` + `mx 4`, container `px 12`, `pb 2`) (`src/prompts/chat/render_core.rs:17`, `src/prompts/chat/render_core.rs:18`, `src/prompts/chat/render_core.rs:83`, `src/prompts/chat/render_core.rs:84`, `src/prompts/chat/render_core.rs:85`, `src/prompts/chat/render_core.rs:168`, `src/prompts/chat/render_core.rs:170`).
- Chat button vertical padding differs from shared footer (`py 2` in chat vs `py 6` in `PromptFooter`) (`src/prompts/chat/render_core.rs:19`, `src/prompts/chat/render_core.rs:49`, `src/components/prompt_footer.rs:368`).

Consequence: chat footer can drift visually/interaction-wise from all other prompt footers as shared constants evolve.

### 3) FocusablePrompt does not standardize shell spacing, so prompt roots diverge

- `FocusablePrompt` adds key/focus behavior only; it does not impose shell frame geometry (`src/components/focusable_prompt_wrapper.rs:132`, `src/components/focusable_prompt_wrapper.rs:137`).
- Select prompt root hardcodes `rounded(12)` and list lane `px(8)` (`src/prompts/select/render.rs:343`, `src/prompts/select/render.rs:332`).
- Env prompt uses custom centered layout spacing (`px(32)`, `gap(24)`) (`src/prompts/env/render.rs:55`, `src/prompts/env/render.rs:56`).
- Path prompt composes higher-level shared components (`PromptHeader` + `PromptContainer`) but still routes through the same geometry-neutral `FocusablePrompt` wrapper (`src/prompts/path/render.rs:105`, `src/prompts/path/render.rs:146`).

Consequence: root inset/radius/gap rhythms are prompt-specific and not governed by a shared contract.

### 4) PromptBase/DesignContext cannot enforce spacing consistency yet

- `DesignContext` only resolves colors, not spacing/radius metrics (`src/prompts/base.rs:102`, `src/prompts/base.rs:131`, `src/prompts/base.rs:188`).
- Prompt spacing therefore comes from a mix of `panel.rs` constants, design token lookups, and local literals.

Consequence: there is no single variant-aware API equivalent to `dc.c.*` for spacing/inset decisions.

## Consolidation-Oriented Token/Metrics Candidates

Recommended shared metrics to formalize next (documentation recommendation only):

1. Shared shell metrics:
- `shell_radius_lg`
- `shell_content_inset_x`
- `shell_section_gap_md`
- `shell_divider_inset_x`

2. Shared input-frame metrics:
- `input_frame_padding_x`
- `input_frame_padding_y`
- `input_frame_radius`
- `input_cursor_gap_x`
- `input_placeholder_compensation_x`

3. Shared footer metrics:
- `footer_padding_x`
- `footer_padding_bottom`
- `footer_section_gap`
- `footer_button_gap`
- `footer_button_padding_x`
- `footer_button_padding_y`
- `footer_divider_width`
- `footer_divider_height`
- `footer_divider_margin_x`

4. Shared content/list metrics:
- `content_lane_inset_x`
- `list_lane_padding_y`
- `empty_state_padding_y`
- `row_radius_md`

5. Design-context integration metrics:
- Add spacing/radius accessors to prompt design context (parallel to color accessors) so prompts can resolve layout rhythm through one API instead of mixed literals.

## Suggested Consolidation Order

1. Make `PromptInput` the true source of input-frame spacing:
- Remove header-level zero-padding override and nested hardcoded frame duplication where possible.

2. Eliminate footer forks:
- Route chat footer through `PromptFooter` or extract shared footer metrics into a single exported token map consumed by both.

3. Standardize root-shell geometry for `FocusablePrompt` call sites:
- Adopt one shared shell helper or a shared shell metrics contract for container radius/insets.

4. Extend prompt base/design context with spacing accessors:
- Keep color and spacing resolution together so per-variant prompt chrome can remain consistent.

## Audit Boundary

- This report inventories spacing ownership and drift points only.
- No behavior or rendering code was modified.
