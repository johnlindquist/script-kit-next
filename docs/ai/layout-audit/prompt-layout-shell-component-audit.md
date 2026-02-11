# Prompt Layout Shell Component Audit

## Scope
- `src/components/prompt_layout_shell.rs`
- `src/render_prompts/other.rs`
- `src/render_prompts/div.rs`
- `src/components/prompt_container.rs` (for shared frame contract comparison)

## Current Shell Helper Behavior

`prompt_shell_container(radius, vibrancy_bg)` currently guarantees a root frame with:
- `flex + flex_col + w_full + h_full` (`src/components/prompt_layout_shell.rs:40`)
- `overflow_hidden` by default (`src/components/prompt_layout_shell.rs:48`)
- `relative` in shell config (`src/components/prompt_layout_shell.rs:34`)
- rounded corners from `radius` (`src/components/prompt_layout_shell.rs:56`)
- optional background only when caller passes `Some(Rgba)` (`src/components/prompt_layout_shell.rs:79`)

`prompt_shell_content(content)` currently guarantees:
- `flex_1 + w_full + min_h(0) + overflow_hidden` (`src/components/prompt_layout_shell.rs:63`)

## Consistency Evaluation

### Padding
- Not enforced by shell helpers.
- `prompt_shell_container` and `prompt_shell_content` do not apply horizontal or vertical insets.
- Each renderer decides its own spacing, so header/content padding is currently per-prompt policy.

### Overflow
- Partially enforced.
- Root and content wrappers clip overflow by default.
- No explicit contract for internal scroll ownership (content wrapper clips, but does not decide whether descendants should `overflow_y_scroll` or `overflow_hidden`).

### Rounded Corners
- Enforced when `prompt_shell_container` is used.
- Radius value is still caller-selected, so token consistency depends on each renderer choosing the same token source.

### Background and Vibrancy
- Partially enforced.
- The shell applies background only if caller passes `Some`; this supports vibrancy by allowing `None`.
- Vibrancy policy is not encoded in the API contract itself; call sites must remember to use `get_vibrancy_background(...)`.

## Adoption Reality in Prompt Renderers
- `render_select_prompt`, `render_env_prompt`, `render_drop_prompt`, `render_template_prompt`, and `render_chat_prompt` consistently use both helpers (`src/render_prompts/other.rs:107`, `src/render_prompts/other.rs:125`, `src/render_prompts/other.rs:143`, `src/render_prompts/other.rs:161`, `src/render_prompts/other.rs:179`).
- `render_webcam_prompt` reproduces shell behavior manually instead of using the shared helpers (`src/render_prompts/other.rs:232`).
- `render_div_prompt` uses `prompt_shell_container` and `prompt_shell_content`, but composes its own header/footer/overlay zones inline (`src/render_prompts/div.rs:134`).

## Gaps
1. No explicit shell vocabulary in code (surface/header/content/footer/overlay are implicit, not typed).
2. No helper-level padding contract.
3. No helper-level footer zone or overlay zone API; each prompt hand-builds them.
4. No first-class vibrancy mode enum; background handling is raw `Option<Rgba>`.
5. No conformance check that all renderers (including webcam/div variants) use the same shell primitives.

## Proposed Unified Shell Vocabulary
- `surface`: outer prompt chrome (radius, clipping, background/vibrancy behavior, positioning for overlays).
- `header_zone`: optional top section (title/input/meta rows, optional divider ownership).
- `content_zone`: required body section (fill behavior, minimum height, scroll policy).
- `footer_zone`: optional fixed bottom section (actions/hints/status).
- `overlay_zone`: optional absolute layer anchored to `surface` (dialogs, scrims, popovers).

## Proposed API Contract (Renderer-Facing)

```rust
pub enum PromptShellBackgroundMode {
    Opaque(Rgba),
    Vibrant, // do not paint background in shell
}

pub enum PromptShellOverflowPolicy {
    Clip,
    ScrollY,
    Visible,
}

pub struct PromptShellZones {
    pub header: Option<AnyElement>,
    pub content: AnyElement,
    pub footer: Option<AnyElement>,
    pub overlay: Option<AnyElement>,
}

pub struct PromptShellSpec {
    pub radius_px: f32,
    pub background_mode: PromptShellBackgroundMode,
    pub content_overflow: PromptShellOverflowPolicy,
    pub content_padding_px: Option<f32>,
    pub header_padding_px: Option<f32>,
    pub footer_padding_px: Option<f32>,
}

pub fn render_prompt_shell(spec: PromptShellSpec, zones: PromptShellZones) -> Div;
```

### Required Invariants
1. `surface` is always `relative + flex_col + w_full + h_full + rounded + overflow_hidden`.
2. `content_zone` is always `flex_1 + min_h(0)` and receives explicit overflow policy.
3. `overlay_zone` (when present) is always `absolute + inset_0` and rendered last.
4. Background behavior is explicit (`Opaque` vs `Vibrant`), not inferred from `Option<Rgba>`.
5. Header/footer are optional but zone order is fixed: header -> content -> footer -> overlay.

## Conformance Target for Prompt Renderers
All prompt renderers should call a single shell API that takes `PromptShellSpec + PromptShellZones`, instead of hand-assembling root/content/footer/overlay div trees. This removes per-renderer drift and makes shell behavior testable by contract.
