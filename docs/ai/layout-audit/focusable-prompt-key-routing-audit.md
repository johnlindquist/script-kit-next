# Focusable Prompt Key Routing Audit

## Scope
- Primary audit target: `src/components/focusable_prompt_wrapper.rs`
- Supporting evidence from prompt call sites and shell routing:
  - `src/prompts/select/render.rs`
  - `src/prompts/drop.rs`
  - `src/prompts/template/render.rs`
  - `src/prompts/env/render.rs`
  - `src/prompts/div/render.rs`
  - `src/prompts/path/render.rs`
  - `src/render_prompts/other.rs`
  - `src/app_impl/shortcuts_hud_grid.rs`

## Current Wrapper Contract
`src/components/focusable_prompt_wrapper.rs` currently implements this flow:
1. Match `Escape`, `Cmd+W`, `Cmd+K` in `match_focusable_prompt_intercepted_key` (`src/components/focusable_prompt_wrapper.rs:31`).
2. Call `app_key_handler` first for matched keys (`src/components/focusable_prompt_wrapper.rs:121`).
3. If `app_key_handler` returns `true`, stop propagation (`src/components/focusable_prompt_wrapper.rs:123`).
4. Otherwise call `entity_key_handler` for both non-intercepted keys and unconsumed intercepted keys (`src/components/focusable_prompt_wrapper.rs:128`).

This means wrapper-level routing is "pre-entity" but not terminal unless explicitly consumed.

## Inconsistent Interception Semantics

### 1) Escape meaning is prompt-dependent
- Select: Escape cancels (`submit_cancel`) and consumes (`src/prompts/select/render.rs:357`).
- Drop: Escape cancels and consumes (`src/prompts/drop.rs:214`).
- Template: Escape cancels and consumes (`src/prompts/template/render.rs:209`).
- Env: Escape cancels and consumes (`src/prompts/env/render.rs:319`).
- Path: Escape cancels only when actions are not showing; otherwise falls through (`src/prompts/path/render.rs:153`).
- Div: plain Escape submits (continue), not cancel; modified Escape falls through (`src/prompts/div/render.rs:139`).

User surprise risk: the same Escape key maps to cancel, submit, or shell fallback depending on prompt.

### 2) Cmd+K ownership is split between entity and shell
- Wrapper intercept list includes `CmdK` for all FocusablePrompt users (`src/components/focusable_prompt_wrapper.rs:27`).
- Path prompt consumes `CmdK` in the wrapper app handler (`src/prompts/path/render.rs:175`).
- Select/Drop/Template/Env app handlers return `false` for non-Escape, so `CmdK` falls through (`src/prompts/select/render.rs:362`, `src/prompts/drop.rs:219`, `src/prompts/template/render.rs:214`, `src/prompts/env/render.rs:324`).
- Shell then may toggle actions (`src/render_prompts/other.rs:30`) only if `sdk_actions` is present.
- Div uses shell-level Cmd+K interception and entity returns `false` for Cmd+K (`src/render_prompts/div.rs:60`, `src/prompts/div/render.rs:147`).

User surprise risk: Cmd+K action behavior differs by prompt and by data availability (`sdk_actions`).

### 3) Cmd+W is "intercepted" in wrapper but effectively shell-owned
- Wrapper matches `CmdW` (`src/components/focusable_prompt_wrapper.rs:39`).
- Most prompt app handlers return `false` for `CmdW` (example: `src/prompts/path/render.rs:179`, `src/prompts/div/render.rs:147`).
- Actual close behavior is shell global shortcut handling (`src/app_impl/shortcuts_hud_grid.rs:20`).

Maintenance risk: developers can assume wrapper owns Cmd+W because it is in the intercepted enum, but real ownership is outer shell.

### 4) "Interception order" comments conflict across layers
- Wrapper docs describe app-level interception first (`src/components/focusable_prompt_wrapper.rs:5`).
- Shell comments in `render_prompts/other.rs` say shell intercepts first for wrapped prompts (`src/render_prompts/other.rs:117`).
- In practice, both layers can participate, and consumption depends on local `stop_propagation` decisions.

Maintenance risk: contradictory mental models lead to accidental duplicate handling or missing stop conditions.

### 5) Dismissable policy is not centralized at the boundary
- `is_dismissable_view` marks `EnvPrompt` non-dismissable (`src/app_impl/shortcuts_hud_grid.rs:70`).
- `other_prompt_shell_handle_key_default` still calls global shortcuts with `is_dismissable=true` (`src/render_prompts/other.rs:37`).
- Current behavior is masked by Env prompt consuming Escape locally (`src/prompts/env/render.rs:320`).

Maintenance risk: changing Env prompt local Escape handling could unintentionally expose shell-level dismiss behavior.

## Recommended Contract (Shell vs Entity)

### Shell Responsibilities (outer prompt renderer / app)
- Own window-global shortcuts only:
  - `Cmd+W`: close window.
  - `Cmd+K`: toggle actions UI for the active host.
- Own modal/overlay routing first:
  - If actions dialog is open, route and consume Escape/navigation there.
- Apply dismissable policy in one place with a correct `is_dismissable` value per prompt.
- Always call `cx.stop_propagation()` when shell consumes a key.

### Entity Responsibilities (prompt root)
- Own prompt-local navigation/input/submit behavior.
- Handle Escape only for prompt-local cancel semantics when shell did not consume it.
- Do not own `Cmd+W`.
- Do not own `Cmd+K` unless there is an explicit prompt-specific exception documented in the shell layer.

### Wrapper Responsibilities (`FocusablePrompt`)
- Focus + key context plumbing (`track_focus`, `key_context`, entity listener).
- Prompt-local pre-routing hook only, not window-global routing.
- Explicit decision surface for routing outcomes (recommended future API):
  - `Consumed`
  - `ContinueEntity`
  - `BubbleToShell`

Using a tri-state routing decision is safer than `bool`, because it distinguishes "continue locally" from "intentionally bubble" and reduces accidental cross-layer coupling.

## Suggested Follow-up Work (Not implemented in this audit)
1. Normalize Escape semantics doc per prompt type (especially `DivPrompt` submit-on-Escape).
2. Move all Cmd+K/Cmd+W handling to shell wrappers, leaving FocusablePrompt for local prompt behavior.
3. Add focused routing tests that assert layer ownership for `Escape/Cmd+W/Cmd+K` on Select, Div, Path, Env.
