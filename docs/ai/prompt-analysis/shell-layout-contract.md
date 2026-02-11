# Prompt Shell Layout Contract

## Scope
- `src/components/prompt_layout_shell.rs`
- `src/components/prompt_container.rs`
- `src/render_prompts/other.rs`
- `src/render_prompts/div.rs`
- `src/prompts/select/render.rs`

## Why This Exists
Prompt chrome is currently composed in multiple layers (wrapper shell, container components, and prompt-entity internals). The result is that header/content/footer/divider/overlay ownership is not predictable by prompt type.

## Current Composition Map

| Prompt path | Root shell owner | Header owner | Divider owner | Content slot owner | Footer owner | Overlay owner |
| --- | --- | --- | --- | --- | --- | --- |
| `render_select_prompt` in `other.rs` + `SelectPrompt::render` | Wrapper uses `prompt_shell_container` (`src/render_prompts/other.rs:119`) | `SelectPrompt` internal search row (`src/prompts/select/render.rs:150`) | `SelectPrompt` input border bottom (`src/prompts/select/render.rs:157`) | Wrapper uses `prompt_shell_content(entity)` (`src/render_prompts/other.rs:121`) and entity builds its own rounded container (`src/prompts/select/render.rs:335`) | None in wrapper/entity | None |
| `render_env_prompt`/`render_drop_prompt`/`render_template_prompt` in `other.rs` | Wrapper uses `prompt_shell_container` (`src/render_prompts/other.rs:137`, `src/render_prompts/other.rs:155`, `src/render_prompts/other.rs:173`) | Entity-defined (not shell-defined) | Entity-defined | Wrapper uses `prompt_shell_content(entity)` (`src/render_prompts/other.rs:139`, `src/render_prompts/other.rs:157`, `src/render_prompts/other.rs:175`) | Entity-defined or none | None |
| `render_chat_prompt` in `other.rs` | Wrapper uses `prompt_shell_container` (`src/render_prompts/other.rs:191`) | Entity-defined | Entity-defined | Wrapper uses `prompt_shell_content(entity)` (`src/render_prompts/other.rs:193`) | Entity-defined | Separate-window actions flow, no inline overlay here |
| `render_webcam_prompt` in `other.rs` | Wrapper builds shell manually (`src/render_prompts/other.rs:232`) | None | None | Manual inline content fill block (`src/render_prompts/other.rs:243`) | Wrapper uses `PromptFooter` (`src/render_prompts/other.rs:253`) | No inline overlay; actions handled separately |
| `render_div_prompt` in `div.rs` | Wrapper uses `prompt_shell_container` (`src/render_prompts/div.rs:134`) | Wrapper inline header strip (`src/render_prompts/div.rs:146`) | Wrapper inline divider (`src/render_prompts/div.rs:171`) | Wrapper uses `prompt_shell_content(entity.clone())` (`src/render_prompts/div.rs:177`) | Wrapper uses `PromptFooter` (`src/render_prompts/div.rs:181`) | Wrapper absolute overlay/backdrop (`src/render_prompts/div.rs:200`) |
| `PromptContainer` component | `PromptContainer` itself via `prompt_frame_root` (`src/components/prompt_container.rs:262`) | Slot API (`.header(...)`) (`src/components/prompt_container.rs:194`) | Built-in optional divider (`src/components/prompt_container.rs:273`) | Slot API + fill/intrinsic mode (`src/components/prompt_container.rs:278`) | Slot API or hint fallback (`src/components/prompt_container.rs:287`) | No overlay slot |

## Inconsistencies Causing Predictability Problems

1. Two shell abstractions with overlapping responsibilities.
- `prompt_shell_container` is wrapper-focused and always `relative` (overlay-capable) (`src/components/prompt_layout_shell.rs:34`, `src/components/prompt_layout_shell.rs:79`).
- `PromptContainer` is slot-focused but not overlay-capable by contract and owns its own background opacity policy (`src/components/prompt_container.rs:93`, `src/components/prompt_container.rs:262`).
- Both are valid for "chrome", so authors must guess which layer should own header/footer/divider.

2. Slot ownership moves between wrapper and entity.
- In simple wrappers, shell owns almost nothing beyond frame/content; entity owns chrome.
- In `div.rs`, wrapper owns header/divider/footer/overlay.
- In `select/render.rs`, entity owns its own rounded bordered container inside wrapper shell (`src/prompts/select/render.rs:343`).

3. Divider semantics are not uniform.
- `PromptContainer` divider is a dedicated slot rule (`src/components/prompt_container.rs:273`).
- `DivPrompt` divider is ad-hoc inline (`src/render_prompts/div.rs:171`).
- `SelectPrompt` uses a border on the input row as an implicit divider (`src/prompts/select/render.rs:157`).

4. Overlay location is implicit, not contractual.
- `prompt_shell_container` sets `relative` so overlays can anchor to root (`src/components/prompt_layout_shell.rs:36`, `src/components/prompt_layout_shell.rs:53`).
- Only `DivPrompt` uses this explicitly (`src/render_prompts/div.rs:200`).
- `PromptContainer` has no overlay slot, so prompts using it must build overlay outside it by convention.

5. Height policy differs per prompt family.
- `prompt_shell_container` defaults to `h_full` via frame root (`src/components/prompt_layout_shell.rs:45`).
- `render_div_prompt` and `render_webcam_prompt` override with fixed `STANDARD_HEIGHT` (`src/render_prompts/div.rs:118`, `src/render_prompts/other.rs:212`).
- There is no explicit shell sizing mode in the API.

6. Focus/key handling often correlates with chrome ownership, but no rule encodes that.
- `SelectPrompt` handles focus/keys in entity via `FocusablePrompt` (`src/prompts/select/render.rs:351`).
- `DivPrompt` and `WebcamPrompt` handle focus at wrapper root (`src/render_prompts/div.rs:136`, `src/render_prompts/other.rs:240`).
- This cross-couples behavior and structure and makes shell behavior hard to reason about.

## Single Mental Model (Proposed Contract)
Use one chrome vocabulary for every prompt wrapper, regardless of prompt type:

1. `surface`
- The outer frame.
- Required.
- Owns: radius, background/vibrancy behavior, clipping, sizing mode, relative positioning for overlays.

2. `header_slot`
- Optional top section for title/search/meta controls.
- Owns its own internal spacing/typography.

3. `header_divider_slot`
- Optional divider immediately after header.
- Explicitly opt-in. Never implicit via arbitrary borders.

4. `content_slot`
- Required main prompt body.
- Owns scroll behavior inside standardized fill wrapper (`flex_1 + min_h(0)`).

5. `footer_slot`
- Optional bottom actions/hints/status.
- Exactly one footer authority (either component footer or explicit hint renderer).

6. `overlay_slot`
- Optional absolute layer rendered last inside `surface`.
- Backdrop + popup placement lives here.

### Slot Order Invariant
`surface(header_slot -> header_divider_slot -> content_slot -> footer_slot -> overlay_slot)`

### Ownership Invariant
- Wrapper owns `surface` and `overlay_slot` always.
- Entity renders `content_slot` (and optionally an entity-local header subcomponent passed to wrapper header slot), but does not own a second outer frame.

## Naming Contract

Use these names consistently in code and docs:
- `PromptSurfaceSpec`: frame-level policy (radius, vibrancy/background mode, sizing mode, clipping).
- `PromptShellSlots`: `{ header, header_divider, content, footer, overlay }`.
- `PromptSurfaceSize`: `FillParent | Fixed(px)`.
- `PromptBackgroundMode`: `Vibrant | Opaque(Rgba)`.

Avoid mixed terms for the same concept (`container`, `shell`, `frame`, `chrome`) in API names. Keep `surface` + `slots` as the canonical pair.

## Folder Layout Recommendation

Target a single facade module for prompt chrome and move helpers behind it:

```text
src/components/prompt_shell/
  mod.rs                 # facade exports PromptSurfaceSpec + PromptShellSlots + render_prompt_shell
  surface.rs             # root frame/sizing/background/relative/clipping behavior
  slots.rs               # header/content/footer/divider slot wrappers
  overlay.rs             # absolute overlay/backdrop helpers
  compat_prompt_container.rs  # temporary adapter for PromptContainer callers during migration
```

Migration intent:
- `src/components/prompt_layout_shell.rs` becomes implementation detail or is folded into `prompt_shell/surface.rs`.
- `src/components/prompt_container.rs` becomes a compatibility adapter over `PromptShellSlots` instead of a parallel contract.

## Refactoring Recommendations (No Code in This Task)

1. Define contract first.
- Write this slot model into component docs and use it as review criteria.

2. Normalize wrapper assembly.
- Convert ad-hoc wrapper chrome (`div.rs`, `other.rs` webcam path) to slot-based composition at one entry point.
- Keep explicit policy for fixed-height prompts (`Fixed(STANDARD_HEIGHT)`) in `PromptSurfaceSize`.

3. Remove dual outer frames.
- For prompts like `SelectPrompt`, move outer rounded/border shell responsibilities out of entity-level `render` and into wrapper `surface` policy.
- Entity should return content-first layout, not another top-level window chrome container.

4. Unify divider responsibility.
- Route all header/content boundaries through `header_divider_slot`.
- Do not rely on input-row borders as cross-prompt divider semantics.

5. Standardize overlay path.
- Inline actions overlays (like `div.rs`) should always mount through `overlay_slot` helpers.
- Prompts without overlays pass `None`; structure remains predictable.

6. Add source-level conformance tests.
- One test set should assert each prompt wrapper maps to slot order consistently.
- One test set should assert that wrapper roots are the only owners of outer `surface` concerns.

## Practical Outcome
After this contract, authors should be able to answer these questions for any prompt in seconds:
- Where does header/footer live? In wrapper slots.
- Where is the divider defined? `header_divider_slot` only.
- Where does overlay mount? `overlay_slot`, rendered last.
- Who owns radius/background/sizing? `surface` policy only.
- Who owns prompt logic and list/input internals? `content_slot` entity.
