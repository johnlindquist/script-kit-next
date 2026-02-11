# `cx.notify()` When And Where

## When to use

- Any time you mutate state that can change rendered UI.
- After selection/index/filter/view/focus changes in event handlers.
- After protocol-driven or async updates that should be visible now.

**Hard rule:** every render-affecting mutation must be followed by
`cx.notify()` (or end in a helper that guarantees it).

## Do not do

- Do not mutate UI state and return without notifying.
- Do not rely on unrelated events to eventually re-render your change.
- Do not move state mutation into `render()` to force updates.
- Do not spam duplicate notifies when no state actually changed.

## Canonical files

- `src/main_entry/app_run_setup.rs:1282`: stdin command dispatch
  mutates `view` state in command arms, then calls `ctx.notify()` once
  at `src/main_entry/app_run_setup.rs:1975`.
- `src/app_navigation/impl_movement.rs:10`: `set_selected_index`
  updates selection + scroll activity, then `cx.notify()` at
  `src/app_navigation/impl_movement.rs:18`.
- `src/prompts/select/prompt.rs:120`: `set_input` updates
  `filter_text`, refilters, scrolls, then `cx.notify()` at
  `src/prompts/select/prompt.rs:129`.
- `src/render_prompts/path.rs:161`: `handle_show_path_actions`
  toggles popup/dialog state, then `cx.notify()` at
  `src/render_prompts/path.rs:165`.
- `src/render_prompts/path.rs:169`: `handle_close_path_actions`
  clears popup/dialog/search state, then `cx.notify()` at
  `src/render_prompts/path.rs:175`.
- `src/app_impl/refresh_scriptlets.rs:196`: documents why notify is
  always called after data/cache refresh; concrete calls at
  `src/app_impl/refresh_scriptlets.rs:187` and
  `src/app_impl/refresh_scriptlets.rs:216`.

## Common mistakes

- State changed in one branch, but another branch `return`s before
  notify.
- Scroll/focus/cache updated, but notify omitted, so rows stay stale.
- Side effects moved into render path instead of mutation handlers.

Reference: `src/render_prompts/path.rs:199` explicitly warns against
side-effects in `render()`.

## Minimal snippet

```rust
pub fn move_down(&mut self, cx: &mut Context<Self>) {
    if self.focused_index < self.filtered_choices.len().saturating_sub(1) {
        self.focused_index += 1;
        self.list_scroll_handle
            .scroll_to_item(self.focused_index, ScrollStrategy::Nearest);
        cx.notify();
    }
}
```
