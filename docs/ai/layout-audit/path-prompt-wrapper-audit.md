# Path Prompt Wrapper Audit

## Scope
- Reviewed `src/render_prompts/path.rs` outer wrapper behavior for modal actions overlay positioning, clipping, key routing, and focus restoration.

## Current Wrapper Snapshot
- Uses a bespoke shell (`div().relative().flex().flex_col().w_full().h_full().overflow_hidden().rounded(...)`) instead of `prompt_shell_container` (`src/render_prompts/path.rs:387`).
- Computes overlay offsets via `prompt_actions_dialog_offsets(...)` (`src/render_prompts/path.rs:188`).
- Renders actions overlay with an absolute full-frame host + flex right alignment + top/right padding (`src/render_prompts/path.rs:401`).
- Implements custom modal key routing inline (`src/render_prompts/path.rs:210`).
- Closes path actions via local helpers (`src/render_prompts/path.rs:119`, `src/render_prompts/path.rs:169`) instead of shared `close_actions_popup(...)` (`src/app_impl/actions_dialog.rs:216`).

## Wrapper-Layer Surprises

### 1) Overlay anchoring is indirect (flex + padding), not explicit absolute top/right
- Path uses `.absolute().inset_0().flex().justify_end().pt(...).pr(...)` for dialog positioning (`src/render_prompts/path.rs:403`).
- Most modal wrappers anchor explicitly with `.absolute().top(px(...)).right(px(...))` (`src/render_prompts/arg/render.rs:492`, `src/render_prompts/div.rs:233`, `src/render_prompts/editor.rs:346`).
- Surprise: the path wrapper’s position depends on flex behavior plus padding, which is harder to reason about than a direct top/right anchor when header heights or alignment rules change.

### 2) No backdrop click affordance for modal overlay
- Path overlay does not include a backdrop click target (`src/render_prompts/path.rs:401`).
- Canonical overlays include a full-frame backdrop and close on click (`src/render_prompts/arg/render.rs:483`, `src/render_prompts/div.rs:225`, `src/render_prompts/editor.rs:338`).
- Surprise: pointer users lose the standard “click outside to dismiss” affordance and there is no explicit cursor hint for dismissability.

### 3) Modal key routing diverges from shared router behavior
- Path manually handles keys while actions are open (`src/render_prompts/path.rs:273`).
- Shared router `route_key_to_actions_dialog(...)` handles additional keys (`Home`/`End`/`PageUp`/`PageDown`) and enforces one modal behavior path (`src/app_impl/actions_dialog.rs:38`).
- Path always runs global shortcut handling before modal routing (`src/render_prompts/path.rs:227`), while other prompts gate global shortcuts when actions are open (`src/render_prompts/div.rs:48`, `src/render_prompts/editor.rs:122`, `src/render_prompts/term.rs:187`).
- Surprise: `Cmd+W` can close the window while the path actions dialog is open, which differs from other prompt wrappers.

### 4) Focus restoration is split and asymmetric
- Keyboard-close helper restores path prompt focus (`src/render_prompts/path.rs:113`, `src/render_prompts/path.rs:133`).
- Event-driven close from `PathPromptEvent::CloseActions` only calls `handle_close_path_actions(cx)` with no `window` focus restore (`src/prompt_handler/mod.rs:1224`, `src/render_prompts/path.rs:169`).
- Shared close flow (`close_actions_popup`) centralizes overlay-stack pop + focus fallback (`src/app_impl/actions_dialog.rs:258`).
- Surprise: close behavior depends on entry path (key-driven vs event-driven), so focus recovery can drift.

## Canonical Wrapper Pattern For Prompts With Modal Overlays

1. Use a shared shell frame contract.
- `prompt_shell_container(radius, vibrancy_bg)` + `prompt_shell_content(entity)` for consistent radius/overflow/inset behavior.

2. Use one modal key-routing contract.
- Key handlers should call `route_key_to_actions_dialog(...)` for modal keys and only run global dismiss handlers when `!show_actions_popup`.

3. Use one close/focus contract.
- Close overlays through `close_actions_popup(host, window, cx)` so focus restoration and overlay-stack handling stay centralized.

4. Use a two-layer absolute overlay host.
- Layer A (backdrop): `.absolute().inset_0()` click target with `.cursor_pointer()` + close handler.
- Layer B (dialog): `.absolute().top(px(dialog_top)).right(px(dialog_right)).child(dialog)`.

5. Keep offsets header-relative via shared helper.
- Continue using `prompt_actions_dialog_offsets(padding_sm, border_thin)`.
- Treat the top offset as “header+divider baseline” and avoid additional per-prompt magic numbers.

6. Keep inset rules consistent.
- Overlay host always `absolute + inset_0` inside the rounded shell.
- Dialog anchor always top/right offsets from shared tokens.
- Backdrop covers the full prompt frame, not just content area.

## Suggested Path-Prompt Normalization Targets
- Add `ActionsDialogHost::PathPrompt` and route path actions through shared modal close/routing APIs.
- Replace path overlay flex anchor with explicit `absolute().top().right()` anchor.
- Add a `path-actions-backdrop` click target with pointer affordance.
- Gate `handle_global_shortcut_with_options(...)` behind `!show_actions_popup` for path wrapper parity.
