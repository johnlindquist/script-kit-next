# Actions Button Visibility Research

Date: 2026-02-02

## 1) Files investigated (footer rendering, actions builders, prompts)

Footer rendering
- `src/render_script_list.rs` (footer construction uses `PromptFooterConfig::default()` without gating secondary visibility; actions click wired to `toggle_actions`) lines 957-1015. 
- `src/components/prompt_footer.rs` (default `show_secondary: true` and `PromptFooterConfig::show_secondary` setter) lines 91-167.

Actions builders
- `src/actions/builders.rs` (scriptlet-defined actions + script context actions; `get_global_actions` returns empty) lines 286-420, 780-804.
- `src/actions/dialog.rs` (`build_actions` composes script-specific actions + global actions) lines 752-776.

Prompts
- `src/render_prompts/div.rs` (`has_actions` from `sdk_actions`, gates Cmd+K + footer secondary) lines 11-117.
- `src/render_prompts/editor.rs` (`has_actions` from `sdk_actions`, gates Cmd+K + footer secondary) lines 10-213.
- `src/render_prompts/term.rs` (`has_actions` from `sdk_actions`, gates Cmd+K + footer secondary) lines 10-129.
- `src/prompts/path.rs` (PromptHeader config sets `show_actions_button(true)` unconditionally) lines 617-645.

Supporting behavior
- `src/app_render.rs` (`get_focused_script_info` returns `None` when selected item is a section header or no results) lines 1193-1208.
- `src/app_impl.rs` (`toggle_actions` opens actions dialog without checking for available actions) lines 3347-3408.
- `src/render_script_list.rs` (Cmd+K in main list calls `toggle_actions` unconditionally) lines 472-499.

## 2) Current behavior (how the Actions button is shown)

- Main script list footer always renders the Actions button because `PromptFooterConfig::default()` sets `show_secondary: true`, and `render_script_list` never overrides it for action availability. `PromptFooter` is created with this default config and always wires the secondary click to `toggle_actions`. (`src/render_script_list.rs` lines 983-1015; `src/components/prompt_footer.rs` lines 112-123)
- In the main list key handler, Cmd+K always toggles the actions dialog, regardless of whether any actions exist. (`src/render_script_list.rs` lines 472-499)
- ActionsDialog builds its action list from the focused script (optional) plus global actions; since `get_global_actions()` returns an empty vector, the dialog can be empty when no script is focused. (`src/actions/dialog.rs` lines 752-776; `src/actions/builders.rs` lines 800-804)
- In SDK prompts (Div/Editor/Term), `has_actions` is derived from `self.sdk_actions` and used to gate both Cmd+K handling and the footer secondary button. (`src/render_prompts/div.rs` lines 11-117; `src/render_prompts/editor.rs` lines 10-213; `src/render_prompts/term.rs` lines 10-129)
- Path prompt header always shows the Actions button because `PromptHeaderConfig::show_actions_button(true)` is hard-coded. (`src/prompts/path.rs` lines 617-645)

## 3) Root cause analysis (why it appears even without actions)

- The Actions button visibility is driven by UI defaults rather than actual action availability. `PromptFooterConfig::default()` enables the secondary button by default, and the main script list footer uses this default without checking action presence. (`src/components/prompt_footer.rs` lines 112-123; `src/render_script_list.rs` lines 983-1001)
- There is no centralized `has_actions` check for the main list. As a result:
  - `render_script_list` always shows Actions in the footer.
  - Cmd+K always calls `toggle_actions`, even when no actions are available. (`src/render_script_list.rs` lines 472-499)
- When the selected item is a section header or there are no results, `get_focused_script_info` returns `None`, and `ActionsDialog::build_actions` only adds `get_global_actions()` (currently empty), producing an empty actions list. (`src/app_render.rs` lines 1193-1208; `src/actions/dialog.rs` lines 752-776; `src/actions/builders.rs` lines 800-804)
- Path prompt bypasses the same gating concept by always enabling the header Actions button, even if no actions are available for the current selection. (`src/prompts/path.rs` lines 617-645)

## 4) Proposed solution (has_actions() function + integration)

### Add a centralized `has_actions()` helper
Implement a helper on `ScriptListApp` (or a shared module) that computes action availability consistently by view/context:

- Main list:
  - If `get_focused_script_info()` returns `Some`, compute actions the same way as `ActionsDialog::build_actions` (script context actions + global actions) and return `!actions.is_empty()`.
  - If `None`, only consider global actions (currently empty, so `false`).
  - This mirrors actual `ActionsDialog` content, preventing empty dialogs.

- SDK prompts (Div/Editor/Term/Other):
  - Return `self.sdk_actions.as_ref().map(|a| !a.is_empty()).unwrap_or(false)`.

- Path prompt (and similar context-specific prompts):
  - Use the appropriate builder (`get_path_context_actions` for PathInfo, file/clipboard builders as needed) and return whether the vector is non-empty.

### Integrate `has_actions()` at UI and handler call sites

- Footer visibility:
  - `src/render_script_list.rs`: set `.show_secondary(self.has_actions())` when building `PromptFooterConfig`.
  - Prompt footers already use `has_actions` in Div/Editor/Term; replace duplicated logic with `self.has_actions()` for consistency.

- Header visibility:
  - `src/prompts/path.rs`: set `.show_actions_button(self.has_actions())` instead of `true`.
  - If other header configs exist, use the same helper to keep behavior aligned with actions availability.

- Cmd+K handling / ActionsDialog:
  - `src/render_script_list.rs`: gate Cmd+K with `if self.has_actions() { ... }`.
  - `src/app_impl.rs` `toggle_actions`: early return (or no-op) when `!self.has_actions()` to avoid opening an empty dialog.

### Optional: expose `has_actions()` for ActionsDialog creation

If needed, a shared helper that returns the computed list (not just a boolean) can prevent duplicate action-building work and ensure that visibility and actual dialog contents are always in sync.

## Verification

1) What was changed
- Added `has_actions()` to `ScriptListApp`.
- Gated footer visibility on `has_actions()`.
- Gated Cmd+K handling on `has_actions()`.

2) Test results
- `actions_button_visibility_tests` (3 total) all passed.

3) Before/after comparison
- Before: Actions button always visible.
- After: Actions button only shows when actions are available.

4) Deviations
- `FooterButton` API was simplified during implementation.
