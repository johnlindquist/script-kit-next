# Components Module Organization Audit (`src/components/mod.rs`)

## Scope

- Reviewed only `src/components/mod.rs` as requested.
- Goal: identify organization patterns that make prompt layout duplication likely, then recommend a clearer hierarchy and token ownership model.

## Current Surface Area (from `mod.rs`)

`src/components/mod.rs` currently exports a broad, flat component namespace:

- Prompt chrome and wrappers:
  - `prompt_container`
  - `prompt_layout_shell`
  - `prompt_header`
  - `prompt_input`
  - `prompt_footer`
  - `focusable_prompt_wrapper`
  - `script_kit_input`
  - `input_tokens`
- List rendering:
  - `unified_list_item` (+ `Density`, `ListItemLayout`, `SectionHeader`, etc.)
- General primitives:
  - `button`
  - `text_input`
  - `scrollbar`
  - `toast`
  - `form_fields`
- Prompt-adjacent modal/input flows:
  - `alias_input`
  - `shortcut_recorder`

The root module also re-exports most items directly with many `#[allow(unused_imports)]` guards.

## Organizational Issues That Encourage Layout Duplication

1. Flat root exports hide ownership boundaries

- Prompt-shell composition, leaf controls, and modal flows all look equally “top-level canonical.”
- This makes it easy for feature code to assemble custom layouts from mixed parts rather than one standard prompt contract.

2. Multiple overlapping input entry points

- `PromptInput`, `ScriptKitInput`, `TextInputState`/`TextSelection`, `FormTextField`, and `AliasInput` are all visible from `components` root.
- Without a clear “prompt input of record,” teams can build similar-but-different header/input rows.

3. Wrapper responsibilities are fragmented

- `PromptContainer`, `prompt_shell_container/prompt_shell_content`, and `FocusablePrompt` suggest three shell/wrapper layers.
- When wrapper hierarchy is not explicit in module structure, implementations tend to rewrap or bypass layers differently.

4. Spacing/alignment tokens are not clearly centralized

- `input_tokens` are exposed, but prompt geometry spans header/input/list/footer/wrappers.
- Token ownership appears split between per-component configs and standalone constants, which encourages local spacing tweaks.

5. Public API breadth obscures canonical paths

- Broad re-exports + unused-import allowances are practical for migration, but they also reduce pressure to converge on one component stack.

## Recommended Hierarchy (for predictability)

Use a domain-first component tree and keep `src/components/mod.rs` as a thin facade:

```text
src/components/
  mod.rs                    # intentional public API only
  primitives/
    mod.rs
    button.rs
    text_input.rs
    scrollbar.rs
    toast.rs
    form_fields.rs
  prompt/
    mod.rs                  # canonical prompt composition API
    shell.rs                # container/content wrappers
    header.rs
    input.rs
    footer.rs
    focus.rs                # focus interception + key routing wrappers
    tokens.rs               # canonical spacing/alignment/size metrics
  list/
    mod.rs
    item.rs                 # unified list row + section header composition
    tokens.rs               # list density/row/slot alignment tokens
  modal/
    mod.rs
    alias_input.rs
    shortcut_recorder.rs
```

## Canonical Token Placement

Put prompt layout metrics in `src/components/prompt/tokens.rs` and treat that as the single source for prompt chrome geometry.

Suggested token groups:

- Vertical structure:
  - header height
  - input row height
  - list top/bottom insets
  - footer height
- Horizontal alignment:
  - shell left/right padding
  - header/input/list shared content inset
  - trailing action gutter
- Rhythm:
  - row gap
  - section gap
  - compact/default density scale

Implementation direction:

- Expose typed metrics (struct + constants) instead of many loose constants.
- Make `prompt_header`, `prompt_input`, `unified_list_item` integration, and `prompt_footer` read from the same token source.
- Keep `input_tokens` either:
  - merged into `prompt/tokens.rs`, or
  - clearly narrowed to text-editing semantics only (font/placeholder), not shell geometry.

## API Discipline Recommendation

In `src/components/mod.rs`, re-export only canonical assembly entry points for prompt UIs, and keep lower-level internals under submodule paths.

Example desired import behavior:

- Preferred:
  - `components::prompt::{PromptHeader, PromptInput, PromptFooter, PromptShell}`
- Discouraged for new prompt code:
  - root-level mixing of wrapper helpers + ad hoc input types.

This nudges feature implementations toward one predictable layout stack and reduces incidental duplication.
