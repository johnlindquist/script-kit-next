# PathPrompt Wrapper Analysis: Focus and Actions Ownership

## Scope
- `src/render_prompts/path.rs`
- `src/components/prompt_layout_shell.rs`
- `src/components/focusable_prompt_wrapper.rs`

## Current Ownership Map

### Key handling layers
1. `PathPrompt` has an inner `FocusablePrompt` key router that intercepts `Escape/Cmd+W/Cmd+K` (`src/prompts/path/render.rs:146`, `src/components/focusable_prompt_wrapper.rs:31`).
2. The Path wrapper has a second outer `.on_key_down(...)` router with its own `Cmd+K` toggle and full actions-dialog keyboard handling (`src/render_prompts/path.rs:213`).
3. The outer router also handles global shortcuts before modal routing (`src/render_prompts/path.rs:227`), unlike most prompt wrappers that gate global shortcuts while overlay is open.

### Actions/search state layers
- App-wide overlay flag: `show_actions_popup` (`src/render_prompts/path.rs:204`).
- Path-specific shared bool: `path_actions_showing` mirrored into prompt `actions_showing` (`src/render_prompts/path.rs:56`, `src/prompts/path/types.rs:86`).
- Path-specific shared search text: `path_actions_search_text` mirrored into prompt `actions_search_text` (`src/render_prompts/path.rs:79`, `src/prompts/path/types.rs:88`).
- Dialog-internal search text: `ActionsDialog.search_text` read/synced manually (`src/render_prompts/path.rs:104`).

### Focus ownership layers
- `AppView::PathPrompt` stores a `focus_handle` used for restore-after-close (`src/render_prompts/path.rs:113`).
- `PathPrompt` stores its own focus handle and tracks focus in `FocusablePrompt` (`src/prompts/path/render.rs:148`).
- Actions close has two paths with different focus behavior:
  - `path_prompt_close_actions_popup(...)` closes and restores focus (`src/render_prompts/path.rs:119`).
  - `handle_close_path_actions(...)` closes without focus restore (`src/render_prompts/path.rs:169`).

## Why This Is Hard To Reason About

1. `Cmd+K` ownership is duplicated.
- Inner `PathPrompt` intercepts `Cmd+K` and toggles actions (`src/prompts/path/render.rs:175`).
- Outer wrapper also intercepts `Cmd+K` and toggles actions (`src/render_prompts/path.rs:246`).
- You must inspect both layers to know which handler wins in a given focus/propagation case.

2. “Is actions open?” has multiple names and multiple stores.
- `show_actions_popup`, `path_actions_showing`, and `PathPrompt.actions_showing` all represent similar state.
- The prompt uses shared mutex state for UI mode decisions (`src/prompts/path/render.rs:70`, `src/prompts/path/render.rs:154`) while the wrapper uses app state.

3. Search query has no single source of truth.
- Real editing happens in `ActionsDialog`.
- Header display uses mirrored `actions_search_text` via mutex.
- Sync is imperative and point-in-time (`src/render_prompts/path.rs:104`, `src/render_prompts/path.rs:322`, `src/render_prompts/path.rs:332`), which increases drift risk and cognitive load.

4. Focus contract is split across close call sites.
- Some close paths restore focus explicitly, others do not.
- This forces readers to reason about close source instead of a single close policy.

5. Path wrapper does not follow the shared shell/key conventions used elsewhere.
- It does not use `prompt_shell_container/prompt_shell_content` (`src/components/prompt_layout_shell.rs:79`).
- It bypasses shared actions routing used by other prompts and inlines modal routing logic.

## Comparison To Other Prompt Wrappers
- Most wrappers keep overlay routing in one app-level handler and use shared `route_key_to_actions_dialog(...)`.
- PathPrompt uniquely splits ownership across:
  - prompt-level `FocusablePrompt` app-key interceptor,
  - wrapper-level modal router,
  - shared mutex mirrors for header mode/search text.

That split is the main reason focus ownership and Cmd+K behavior are harder to reason about here than in Arg/Form/Chat wrappers.

## Proposed Clearer Organization

### 1) Single owner for path actions overlay state (app layer)
Use one path-specific state object in `ScriptListApp`:

```rust
struct PathActionsOverlayState {
    is_open: bool,
    dialog: Option<Entity<ActionsDialog>>,
    query: String, // header mirror for path prompt chrome
}
```

- `is_open` replaces `path_actions_showing` + prompt `actions_showing`.
- `query` replaces `path_actions_search_text` + prompt `actions_search_text`.
- `show_actions_popup` remains for non-path prompts or is renamed to clarify global-vs-path ownership.

### 2) Make PathPrompt entity navigation-only
`PathPrompt` should own only directory/filter/list navigation state.
- Keep: `current_path`, `filter_text`, `selected_index`, entries.
- Remove ownership of overlay-open/search mirror mutexes.
- Consume a read-only overlay snapshot for rendering header mode.

Suggested render input type:

```rust
struct PathPromptOverlaySnapshot {
    is_open: bool,
    query: String,
}
```

### 3) Unify key responsibility boundaries
- `render_prompts/path.rs` (wrapper/app layer):
  - owns `Cmd+K`,
  - owns modal keys while overlay open,
  - owns global-shortcut gating when overlay is open.
- `prompts/path/render.rs` (entity layer):
  - owns browse keys only (up/down/left/right/tab/enter/backspace/char),
  - never toggles overlays directly.

This removes double interception and aligns with `FocusablePrompt`’s intended two-level model.

### 4) One close path with one focus policy
Create one close API (e.g. `path_actions_overlay_close(window, cx, reason)`):
- clears dialog/state,
- restores focus,
- logs reason.

All call sites use this one path (keyboard, events, callbacks).

## Naming Proposal (Greppable + Ownership-Explicit)

Current name | Proposed name | Owner
--- | --- | ---
`show_actions_popup` (path usage) | `path_actions_overlay.is_open` | `ScriptListApp`
`actions_dialog` (path usage) | `path_actions_overlay.dialog` | `ScriptListApp`
`path_actions_showing` / `actions_showing` | removed | n/a
`path_actions_search_text` / `actions_search_text` | `path_actions_overlay.query` | `ScriptListApp`
`handle_show_path_actions` | `path_actions_overlay_open_for(path_info, cx)` | `ScriptListApp`
`handle_close_path_actions` + `path_prompt_close_actions_popup` | `path_actions_overlay_close(window, cx, reason)` | `ScriptListApp`

## Suggested Incremental Migration Order
1. Introduce `PathActionsOverlayState` and keep old fields temporarily.
2. Route all close paths through one close function with focus restore.
3. Move `Cmd+K` handling to wrapper only; remove entity `Cmd+K` interception.
4. Replace shared mutex mirrors with `PathPromptOverlaySnapshot`.
5. Switch wrapper container to `prompt_shell_container/prompt_shell_content` so layout and overlay rules match other prompt wrappers.
