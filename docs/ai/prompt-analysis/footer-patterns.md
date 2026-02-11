# Prompt Footer Patterns

## Scope
- `src/components/prompt_footer.rs`
- `src/render_prompts/arg/render.rs`
- `src/render_prompts/form/render.rs`
- `src/render_prompts/div.rs`
- `src/render_prompts/other.rs`
- `src/prompts/env/render.rs`

## Current PromptFooter API Surface

### Core config shape
- `PromptFooterConfig` is string-driven for button labels/shortcuts, with optional `helper_text` and `info_label` (`src/components/prompt_footer.rs:184`).
- Secondary action is controlled by `show_secondary: bool`, not by action type (`src/components/prompt_footer.rs:196`).
- Footer helper and info are free-form text slots, with no semantic distinction for status vs hint vs error (`src/components/prompt_footer.rs:202`, `src/components/prompt_footer.rs:204`).

### Rendering behavior that affects UX consistency
- The component hides some UI based on sentinel strings (`"Built-in"`, `"Run Command"`) instead of explicit intent flags (`src/components/prompt_footer.rs:82`, `src/components/prompt_footer.rs:84`, `src/components/prompt_footer.rs:170`, `src/components/prompt_footer.rs:176`).
- Buttons are only interactive when callbacks are attached and the action is not disabled (`src/components/prompt_footer.rs:151`, `src/components/prompt_footer.rs:462`, `src/components/prompt_footer.rs:476`).
- If callback is missing but disabled is `false`, the button still renders with normal text styling but no click affordance.

## How Scoped Prompts Configure The Footer Today

| Prompt | Primary | Secondary | Helper text | Info label | Secondary behavior |
|---|---|---|---|---|---|
| Arg (`src/render_prompts/arg/render.rs:423`) | `Continue` + `↵` | `Actions` + `⌘K` only when `has_actions` | Dynamic status from input/choices (`src/render_prompts/arg/helpers.rs:100`) | `"{n} options"` when choices exist | toggles actions dialog (`src/render_prompts/arg/render.rs:446`) |
| Form (`src/render_prompts/form/render.rs:213`) | `Continue` + `↵` | `Actions` + `⌘K` only when `has_actions` | Submit hint varies by focused field (`src/render_prompts/form/render.rs:216`) | `"{n} fields"` | toggles actions dialog (`src/render_prompts/form/render.rs:222`) |
| Div (`src/render_prompts/div.rs:122`) | `Continue` + `↵` | `Actions` + `⌘K` only when `has_actions` | `review output and press Enter` | `Output` | toggles actions dialog (`src/render_prompts/div.rs:191`) |
| Webcam (`src/render_prompts/other.rs:219`) | `Capture Photo` + `↵` | `Actions` + `⌘K` always shown | `camera ready, press Enter to capture` | `Webcam` | toggles webcam actions (`src/render_prompts/other.rs:263`) |
| Env (`src/prompts/env/render.rs:285`) | `Save & Continue` / `Update & Continue` + `↵` | `Cancel` + `Esc` always shown | `Script running` | none | submits cancel (`src/prompts/env/render.rs:304`) |

## UX Inconsistencies

1. Secondary slot is overloaded with incompatible meanings.
- Most prompts use secondary for actions palette (`Actions`/`⌘K`), while `EnvPrompt` uses it for cancel (`Cancel`/`Esc`) (`src/prompts/env/render.rs:290`).
- There is no typed distinction in `PromptFooterConfig` to protect this behavior.

2. Primary click behavior is inconsistent.
- `FormPrompt` renders a primary action but only wires `on_secondary_click`; no `on_primary_click` is attached (`src/render_prompts/form/render.rs:222`).
- Because clickability depends on callback presence, form's primary looks present but is not clickable.

3. Secondary visibility policy is inconsistent.
- Arg/Form/Div gate secondary visibility on `has_actions` (`src/render_prompts/arg/render.rs:425`, `src/render_prompts/form/render.rs:215`, `src/render_prompts/div.rs:124`).
- Webcam forces secondary visible (`true`) regardless of local `has_actions` checks (`src/render_prompts/other.rs:221`).

4. Shortcut ownership is split across local and global handlers.
- Arg/Form/Div handle action shortcuts in their prompt-local key handlers.
- Webcam's local handler does not route `Cmd+K` (`src/render_prompts/other.rs:91`), yet footer advertises `⌘K`, relying on global interceptors outside this file.

5. Status/hint placement is semantically mixed.
- Footer helper currently carries running state + instructions in Arg/Form/Div/Webcam.
- `EnvPrompt` duplicates running status in body and footer (`src/prompts/env/render.rs:200`, `src/prompts/env/render.rs:288`), while validation errors stay in-body (`src/prompts/env/render.rs:190`).
- There is no documented rule for whether status, hint, or error belongs in helper/info/body/HUD.

6. Info label semantics are inconsistent.
- Arg/Form use quantitative metadata (`options`, `fields`), Div/Webcam use static mode labels (`Output`, `Webcam`), Env omits it entirely.
- Without slot semantics, info label meaning changes per prompt.

7. Footer behavior is coupled to string literals.
- Hiding primary/info via sentinel labels introduces copy-dependent behavior (`src/components/prompt_footer.rs:170`, `src/components/prompt_footer.rs:176`).

## Proposed Standardized Footer Contract

### What always appears
- `leading`: optional logo + one canonical `status_line` (single-line, ellipsized).
- `trailing`: optional `meta_label` (short, low-priority) + `primary_action` + optional `secondary_action`.
- `primary_action` is always present and must be either clickable or explicitly disabled.

### Slot semantics
- `status_line`: running state and immediate keyboard hint only.
  - Examples: `Script running - press Enter to continue`, `Script running - press Cmd+Enter to submit`.
- `meta_label`: static context/count only.
  - Examples: `12 options`, `3 fields`, `Webcam`.
- Errors do **not** go in footer helper/info slots.
  - Field errors remain inline in prompt body.
  - Global submit errors remain HUD/toast.

### Action naming and shortcuts
- `primary_action`: task verb for forward progress.
  - Default `Continue`.
  - Domain-specific allowed (`Capture Photo`, `Save & Continue`, `Update & Continue`).
- `secondary_action` must be one of:
  - `Actions` with shortcut `⌘K` (opens action palette).
  - `Cancel` with shortcut `Esc` (dismiss/cancel prompt).
  - `None`.
- Do not use arbitrary secondary labels for mixed behaviors.

### Typed contract (recommended)
```rust
pub struct PromptFooterContract {
    pub status_line: Option<String>,
    pub meta_label: Option<String>,
    pub primary: FooterPrimaryAction,
    pub secondary: FooterSecondaryAction,
    pub show_logo: bool,
}

pub struct FooterPrimaryAction {
    pub label: String,
    pub shortcut: FooterShortcut,
    pub enabled: bool,
}

pub enum FooterSecondaryAction {
    None,
    Actions { enabled: bool },
    Cancel { enabled: bool },
}

pub enum FooterShortcut {
    Enter,
    CmdEnter,
    CmdK,
    Esc,
    Custom(String),
}
```

## Refactoring Recommendations

1. Replace string-sentinel hiding with explicit flags.
- Remove `PROMPT_FOOTER_HIDDEN_INFO_LABEL` / `PROMPT_FOOTER_HIDDEN_PRIMARY_LABEL` checks and encode intent in config.

2. Introduce typed secondary action semantics.
- Replace `show_secondary + secondary_label + secondary_shortcut` with `FooterSecondaryAction`.
- Enforce label/shortcut mapping in `PromptFooter` itself for `Actions` and `Cancel`.

3. Make clickability contract explicit.
- Add debug assertion or constructor validation: if an enabled action is rendered, a callback must exist.
- At minimum, auto-disable actions with missing callbacks to avoid misleading visual affordance.

4. Add a small shared builder layer for prompt wrappers.
- Keep one constructor for `Actions` pattern and one for `Cancel` pattern.
- Migrate Arg/Form/Div/Webcam/Env to these builders to eliminate ad-hoc differences.

5. Align status/error placement rules.
- Keep footer status for concise run/hint text only.
- Keep validation errors in prompt body and do not duplicate in footer.
- For Env specifically, avoid duplicate running indicators in body + footer.

6. Fix form primary action parity.
- Wire `on_primary_click` in `render_form_prompt` to submit with the same validation path as Enter.

7. Add targeted tests for contract behavior.
- PromptFooter unit tests:
  - `test_footer_secondary_actions_variant_uses_actions_label_and_cmd_k`
  - `test_footer_secondary_cancel_variant_uses_cancel_label_and_esc`
  - `test_footer_enabled_primary_without_callback_is_disabled_or_rejected`
- Prompt render tests:
  - Form: primary click triggers submit path.
  - Env: secondary cancel remains mapped to Escape behavior.

## Migration Order (Low Risk)
1. Introduce new typed footer contract alongside existing config.
2. Convert Arg/Form/Div first (already using shared helper pattern).
3. Convert Env and Webcam with explicit `Cancel` vs `Actions` variants.
4. Remove legacy string-sentinel logic after all call sites are migrated.
