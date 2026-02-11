<!-- markdownlint-disable MD013 -->
# Env + Template Prompt Spacing Audit

## Scope

Audit target:

- Wrapper path for env/template prompts in `src/render_prompts/other.rs`
- Internal layout and spacing in `src/prompts/env/render.rs`
- Internal layout and spacing in `src/prompts/template/render.rs`

No Rust code changes were made. This is a documentation-only spacing audit.

## Render Call Chains

### EnvPrompt

1. `ScriptListApp::render_env_prompt(...)` computes shell radius from tokens (`other_prompt_shell_radius_lg`) and wraps the entity in shared shell helpers:
   - `prompt_shell_container(shell_radius, vibrancy_bg)`
   - `prompt_shell_content(entity)`
   (`src/render_prompts/other.rs:125`, `src/render_prompts/other.rs:130`, `src/render_prompts/other.rs:137`, `src/render_prompts/other.rs:139`).
2. `prompt_shell_container(...)` contributes frame behavior (`w_full`, `h_full`, `overflow_hidden`, `rounded(radius)`, `relative`) but no padding/gap/margin (`src/components/prompt_layout_shell.rs:79`, `src/components/prompt_layout_shell.rs:40`).
3. `prompt_shell_content(...)` contributes `flex_1`, `min_h(0)`, `overflow_hidden`, but no spacing inset (`src/components/prompt_layout_shell.rs:86`, `src/components/prompt_layout_shell.rs:63`).
4. `EnvPrompt::render(...)` defines all internal spacing and appends `PromptFooter` as a sibling of the main content area (`src/prompts/env/render.rs:39`, `src/prompts/env/render.rs:49`, `src/prompts/env/render.rs:278`).
5. `FocusablePrompt::build(...)` adds focus/key wrappers, not spacing (`src/prompts/env/render.rs:313`).

### TemplatePrompt

1. `ScriptListApp::render_template_prompt(...)` uses the same shell wrapper pattern as env:
   - `prompt_shell_container(shell_radius, vibrancy_bg)`
   - `prompt_shell_content(entity)`
   (`src/render_prompts/other.rs:161`, `src/render_prompts/other.rs:166`, `src/render_prompts/other.rs:173`, `src/render_prompts/other.rs:175`).
2. Shell helpers are spacing-neutral (layout frame only) as above (`src/components/prompt_layout_shell.rs:79`, `src/components/prompt_layout_shell.rs:86`).
3. `TemplatePrompt::render(...)` owns all prompt body spacing via design spacing tokens plus a few literals (`src/prompts/template/render.rs:39`, `src/prompts/template/render.rs:47`).
4. `TemplatePrompt` does not append `PromptFooter`; submit/cancel is keyboard-driven and inline help text is used at the bottom (`src/prompts/template/render.rs:195`, `src/prompts/template/render.rs:227`).
5. `FocusablePrompt::build(...)` wraps focus/key behavior only (`src/prompts/template/render.rs:203`).

## Spacing Inventory (Token-Derived vs Hardcoded/Shared Metrics)

### A) Wrapper Layer (`render_prompts/other.rs` + shell helper)

| Surface | Rule | Source Type | Evidence |
| --- | --- | --- | --- |
| Prompt shell corner radius | `radius_lg` from design visual tokens | Token-derived | `other_prompt_shell_radius_lg -> tokens.visual().radius_lg` (`src/render_prompts/other.rs:13`) |
| Outer shell sizing | `w_full`, `h_full`, `min_h(0)` | Hardcoded layout behavior | `prompt_frame_root` and `prompt_frame_fill_content` (`src/components/prompt_layout_shell.rs:44`, `src/components/prompt_layout_shell.rs:46`, `src/components/prompt_layout_shell.rs:67`) |
| Wrapper insets/gaps | None | N/A | No `.p/.px/.py/.gap/.mt/.mb/.ml/.mr` in `render_env_prompt`/`render_template_prompt` wrappers (`src/render_prompts/other.rs:125`, `src/render_prompts/other.rs:161`) |

Observation: the wrapper standardizes frame mechanics but delegates all spacing rhythm to inner prompt renderers.

### B) EnvPrompt Internal Spacing (`src/prompts/env/render.rs`)

| Surface | Rule | Source Type | Evidence |
| --- | --- | --- | --- |
| Main vertical lane inset | `.px(32)` | Hardcoded literal | `src/prompts/env/render.rs:55` |
| Main section stacking | `.gap(24)` | Hardcoded literal | `src/prompts/env/render.rs:56` |
| Icon shell size/radius | `.size(64)`, `.rounded(16)` | Hardcoded literal | `src/prompts/env/render.rs:60`, `src/prompts/env/render.rs:64` |
| Title/description spacing | `.gap(8)` | Hardcoded literal | `src/prompts/env/render.rs:83` |
| Input group width cap | `.max_w(400)` | Hardcoded literal | `src/prompts/env/render.rs:104` |
| Input group label-field-hint gap | `.gap(8)` | Hardcoded literal | `src/prompts/env/render.rs:107` |
| Input field min height | `PROMPT_INPUT_FIELD_HEIGHT` | Shared metric constant | `src/prompts/env/render.rs:117`; constant `44.0` (`src/panel.rs:76`) |
| Input field inset | `.px(16)`, `.py(12)` | Hardcoded literal | `src/prompts/env/render.rs:118`, `src/prompts/env/render.rs:119` |
| Input row internal gap | `.gap(12)` | Hardcoded literal | `src/prompts/env/render.rs:127` |
| Placeholder cursor metrics | `CURSOR_WIDTH`, `CURSOR_HEIGHT_LG`, `.ml(4)` | Shared metric + literal | `src/prompts/env/render.rs:160`, `src/prompts/env/render.rs:161`, `src/prompts/env/render.rs:166`; constants `2.0`, `18.0` (`src/panel.rs:169`, `src/panel.rs:187`) |
| Running indicator spacing | `.gap(8)` | Hardcoded literal | `src/prompts/env/render.rs:208` |
| Existing-key status stack | outer `.gap(8)`, inner `.gap(6)` | Hardcoded literal | `src/prompts/env/render.rs:232`, `src/prompts/env/render.rs:239` |
| Footer placement | Footer appended directly after content region, no explicit spacer/margin | Structural (no token) | `src/prompts/env/render.rs:277`, `src/prompts/env/render.rs:278` |

Notes:

- EnvPrompt uses design tokens for colors/typography, but spacing is mostly literal px plus shared panel constants.
- No clamped spacing behavior exists in this file.

### C) TemplatePrompt Internal Spacing (`src/prompts/template/render.rs`)

| Surface | Rule | Source Type | Evidence |
| --- | --- | --- | --- |
| Root content inset | `.p(spacing.padding_lg)` | Token-derived | `src/prompts/template/render.rs:47` |
| Preview block top margin | `.mt(spacing.padding_sm)` | Token-derived | `src/prompts/template/render.rs:54` |
| Preview block x/y inset | `.px(spacing.item_padding_x)`, `.py(spacing.padding_md)` | Token-derived | `src/prompts/template/render.rs:55`, `src/prompts/template/render.rs:56` |
| Section spacing before intro text | `.mt(spacing.padding_lg)` | Token-derived | `src/prompts/template/render.rs:69`, `src/prompts/template/render.rs:76` |
| Group header spacing | `.mt(spacing.padding_md)` | Token-derived | `src/prompts/template/render.rs:91` |
| Per-row top spacing | `.mt(spacing.padding_sm)` | Token-derived | `src/prompts/template/render.rs:134` |
| Row/inner utility gaps | `.gap_1()`, `.gap_2()` | Hardcoded utility scale | `src/prompts/template/render.rs:137`, `src/prompts/template/render.rs:143` |
| Label column width | `.w(140)` | Hardcoded literal | `src/prompts/template/render.rs:146` |
| Field min height | `PROMPT_INPUT_FIELD_HEIGHT` | Shared metric constant | `src/prompts/template/render.rs:154`; constant `44.0` (`src/panel.rs:76`) |
| Field x/y inset | `.px(spacing.item_padding_x)`, `.py(spacing.padding_sm)` | Token-derived | `src/prompts/template/render.rs:155`, `src/prompts/template/render.rs:156` |
| Field corner radius | `.rounded(4)` | Hardcoded literal | `src/prompts/template/render.rs:160` |
| Validation message indent | `.pl(144)` | Hardcoded literal | `src/prompts/template/render.rs:169` |
| Naming-tip/help spacing | `.mt(spacing.padding_md)` and `.mt(spacing.padding_lg)` | Token-derived | `src/prompts/template/render.rs:187`, `src/prompts/template/render.rs:197` |

Default token values (baseline `DesignSpacing::default`):

- `padding_sm = 8.0`
- `padding_md = 12.0`
- `padding_lg = 16.0`
- `item_padding_x = 16.0`
  (`src/designs/traits/parts.rs:151`, `src/designs/traits/parts.rs:152`, `src/designs/traits/parts.rs:153`, `src/designs/traits/parts.rs:164`).

Notes:

- TemplatePrompt has stronger token usage than EnvPrompt, but still mixes in hardcoded geometry (`140`, `144`, `4`, utility gaps).
- No clamped spacing behavior exists in this file.

## Field Grouping and Footer Integration

### Field grouping

- `EnvPrompt`: single field block (label + field + storage hint) with optional status/error blocks; no dynamic group headers (`src/prompts/env/render.rs:100`, `src/prompts/env/render.rs:190`, `src/prompts/env/render.rs:217`).
- `TemplatePrompt`: explicit grouping by `input.group`; new group header inserted when group name changes (`src/prompts/template/render.rs:85`, `src/prompts/template/render.rs:87`, `src/prompts/template/render.rs:90`).

### Footer integration

- `EnvPrompt` embeds `PromptFooter` directly in the render tree and wires primary/secondary actions (`src/prompts/env/render.rs:278`, `src/prompts/env/render.rs:296`, `src/prompts/env/render.rs:304`).
- `TemplatePrompt` has no `PromptFooter`; keyboard affordance text is rendered inline (`src/prompts/template/render.rs:195`).
- Wrapper (`render_env_prompt` / `render_template_prompt`) does not normalize footer behavior; it only hosts the entity (`src/render_prompts/other.rs:137`, `src/render_prompts/other.rs:173`).

## Token vs Hardcoded Summary

- **EnvPrompt:** mostly hardcoded spacing literals and shared panel constants; no design spacing tokens used for layout spacing.
- **TemplatePrompt:** mostly token-based spacing for section rhythm/insets, with hardcoded row geometry and utility-gap shortcuts.
- **Wrapper path:** tokenized radius only; otherwise spacing-neutral frame behavior.
- **Clamped rules:** none found in scoped env/template/wrapper files.

## Normalization Suggestions (No Code Changes Applied)

1. Define one spacing ownership model for full-screen prompts: either token-first or explicit local constants, then apply consistently to both env and template.
2. For EnvPrompt, migrate major literals (`32`, `24`, `16`, `12`, `8`, `6`) to `tokens.spacing()` aliases to align with template rhythm.
3. For TemplatePrompt, replace mixed utility/literals (`gap_1`, `gap_2`, `w(140)`, `pl(144)`, `rounded(4)`) with named metrics (token or shared constants) to make row alignment grepable and variant-safe.
4. Standardize footer affordance strategy for env/template: either both use `PromptFooter` or both provide a consistent non-footer bottom action lane; current split creates uneven bottom spacing behavior.
5. Add a small shared "prompt field row metrics" helper (label width, error indent, row gap) so template grouping rows and env single-field rows can converge on one predictable spacing contract.

## Known Boundaries

- This audit focuses on env/template prompt internals and their wrapper path.
- `PromptFooter` internals (its own fixed paddings/gaps) are referenced by behavior but audited in shared-components workstream.
