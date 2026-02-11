# Focus And Key Routing Map

## Scope

- `src/components/focusable_prompt_wrapper.rs`
- `src/render_prompts/arg/render.rs`
- `src/prompts/select/render.rs`
- `src/prompts/env/render.rs`
- `src/prompts/div/render.rs`

Supporting context for shell-level global shortcut ownership:

- `src/render_prompts/other.rs`
- `src/render_prompts/div.rs`
- `src/app_impl/shortcuts_hud_grid.rs`

## Executive Summary

- `ArgPrompt` is mostly single-owner.
  One `on_key_down` closure in
  `ScriptListApp::render_arg_prompt` owns global shortcuts,
  actions routing, and local navigation/input.
- `SelectPrompt`, `EnvPrompt`, and `DivPrompt` are split-owner.
  Prompt entity render code handles local routing, while parent shell render
  code handles shared global shortcuts and actions popup flow.
- `FocusablePrompt` advertises interception for `Escape`, `Cmd+W`, and
  `Cmd+K`, but current prompt usage mainly consumes `Escape` there.
  `Cmd+W` and `Cmd+K` are mostly handled at shell/app level.

## Per-Prompt Routing Map

### ArgPrompt (`src/render_prompts/arg/render.rs`)

- Focus handle wiring:
  root element calls `.track_focus(&self.focus_handle)`.
- `key_context` owner:
  root element calls `.key_context("arg_prompt")`.
- Global shortcuts owner:
  same `handle_key` closure calls
  `handle_global_shortcut_with_options(...)`, plus `Cmd+K` handling.
- Entity navigation/input owner:
  same `handle_key` closure handles arrows, enter, tab,
  and text input delegation.

### SelectPrompt (`src/prompts/select/render.rs`)

- Focus handle wiring:
  entity uses
  `FocusablePrompt::focus_handle(self.focus_handle.clone())`.
- `key_context` owner:
  entity uses `FocusablePrompt::key_context("select_prompt")`.
- Global shortcuts owner:
  split between entity and shell.
  Entity `app_key_handler` consumes `Escape`.
  Shell (`src/render_prompts/other.rs`) handles `Cmd+K` and
  `handle_global_shortcut_with_options(...)`.
- Entity navigation/input owner:
  `entity_key_handler` in `FocusablePrompt::build(...)`
  handles list movement, filtering, and submit.

### EnvPrompt (`src/prompts/env/render.rs`)

- Focus handle wiring:
  entity uses
  `FocusablePrompt::focus_handle(self.focus_handle.clone())`.
- `key_context` owner:
  entity uses `FocusablePrompt::key_context("env_prompt")`.
- Global shortcuts owner:
  split between entity and shell.
  Entity `app_key_handler` consumes `Escape`.
  Shell (`src/render_prompts/other.rs`) handles `Cmd+K` and
  `handle_global_shortcut_with_options(...)`.
- Entity navigation/input owner:
  `entity_key_handler` in `FocusablePrompt::build(...)`
  handles submit and text editing.

### DivPrompt (`src/prompts/div/render.rs`)

- Focus handle wiring:
  entity uses
  `FocusablePrompt::focus_handle(self.focus_handle.clone())`.
  Parent shell in `src/render_prompts/div.rs` also owns a key surface.
- `key_context` owner:
  entity uses `FocusablePrompt::key_context("div_prompt")`.
- Global shortcuts owner:
  split between entity and shell.
  Entity `app_key_handler` consumes unmodified `Escape`.
  Shell handles `Cmd+W`, `Cmd+K`, actions routing, and popup-state gates.
- Entity navigation/input owner:
  `entity_key_handler` submits on Enter and applies modifier policy.

## What `FocusablePrompt` Actually Does

- `match_focusable_prompt_intercepted_key(...)` checks only
  `Escape`, `Cmd+W`, and `Cmd+K`.
- `build(...)` calls `app_key_handler` first for intercepted keys.
- If `app_key_handler` returns `true`, it calls `cx.stop_propagation()`.
- Otherwise it calls `entity_key_handler`.
- It applies optional `key_context` and always applies `track_focus`.

So today it is both a focus/context wrapper and a partial key router,
not a complete global-shortcut authority.

## Cognitive Load And Bug Surface

1. One key can have multiple owners.

- `Escape` appears in entity `app_key_handler` callbacks and in
  app-global shortcut policy.
- `Cmd+K` appears in shell handlers and in wrapper interception enums.

1. Behavior depends on propagation order.

- Shell global handling depends on inner handlers not consuming first.
- A local `cx.stop_propagation()` change can break global shortcuts
  without compiler errors.

1. Abstraction intent and usage diverge.

- Wrapper docs imply app-level interception for
  `Escape/Cmd+W/Cmd+K`.
- Actual usage spreads those keys across wrapper and shell layers.

1. Prompt families require different lookup paths.

- Arg: mostly one file.
- Select/Env: entity render plus `src/render_prompts/other.rs`.
- Div: entity render plus `src/render_prompts/div.rs`.

1. Popup policy can drift by layer.

- Shell code gates escape behavior on popup state.
- Entity-level escape handlers can bypass that policy if they consume first.

## Recommended Single Lookup Contract

Use this sequence for debugging and code review.

1. Focus handle source of truth.

- First check prompt construction where `FocusHandle` is injected.
- Then check render root wiring (`track_focus` or
  `FocusablePrompt::focus_handle`).

1. `key_context` source of truth.

- Check only the prompt render root chain.
- Ignore shell wrappers unless they explicitly set their own context.

1. Global shortcut source of truth.

- Treat app-shell handlers as canonical global owners:
  `handle_global_shortcut_with_options(...)` plus
  per-prompt shell handlers.
- Treat entity handlers as local navigation/input owners.

## Recommended Ownership Rule (Future Consistency)

- Global keys (`Cmd+W`, `Cmd+K`, shared `Escape` policy):
  one app-shell layer.
- Prompt-local keys (typing, list navigation, submit semantics):
  one entity layer.
- `FocusablePrompt` should remain the focus/context wrapper,
  not a second global-policy source.

This keeps key ownership deterministic: global in shell,
local in entity.

## Practical Grep Entry Points

- Focus wiring:
  `rg -n "focus_handle\(|track_focus\(" src/prompts src/render_prompts src/prompt_handler`
- Key contexts:
  `rg -n "key_context\(" src/prompts src/render_prompts`
- Global shortcuts:
  `rg -n "handle_global_shortcut_with_options" src`
  `rg -n "other_prompt_shell_handle_key_default|render_div_prompt" src`
- Wrapper interception:
  `rg -n "match_focusable_prompt_intercepted_key" src/components src/prompts`
  `rg -n "FocusablePromptInterceptedKey" src/components src/prompts`
