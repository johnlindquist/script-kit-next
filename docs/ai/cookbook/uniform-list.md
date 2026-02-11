<!-- markdownlint-disable MD013 -->

# `uniform_list` Pattern

## When to use

- You need a virtualized, scrollable list where every rendered row has the
  same height.
- You need keyboard selection that auto-scrolls to keep the selected row
  visible.
- You need predictable redraws after selection/filter changes with `cx.notify()`.

> Legacy note: older docs referenced a 52px convention. In current source,
> the canonical fixed row height is `LIST_ITEM_HEIGHT = 40.0` and should be
> treated as the single source of truth.

## Do not do

- Do not use `uniform_list` for variable-height rows.
- Do not render rows at a different height than your list constant.
- Do not mutate selection/filter state without calling `cx.notify()`.
- Do not call `scroll_to_item` without wiring `.track_scroll(&handle)` on the
  list.

## Canonical files

- `src/list_item/part_000.rs:49` documents fixed-height requirements for
  `uniform_list` rows.
- `src/list_item/part_000.rs:54` defines `LIST_ITEM_HEIGHT`.
- `src/prompts/select/prompt.rs:37` stores `UniformListScrollHandle` in prompt
  state.
- `src/prompts/select/prompt.rs:80` initializes the handle with
  `UniformListScrollHandle::new()`.
- `src/prompts/select/prompt.rs:173` and `src/prompts/select/prompt.rs:184`
  update selection, call `scroll_to_item(...)`, then `cx.notify()`.
- `src/prompts/select/prompt.rs:133` toggles selection and calls `cx.notify()`.
- `src/prompts/select/render.rs:248` creates the `uniform_list`.
- `src/prompts/select/render.rs:333` enforces fixed row height with
  `.h(px(LIST_ITEM_HEIGHT))`.
- `src/prompts/select/render.rs:371` binds scrolling with
  `.track_scroll(&self.list_scroll_handle)`.
- `src/prompts/path/prompt.rs:291` and `src/prompts/path/prompt.rs:301` show
  the same selection + scroll + notify loop in another prompt.
- `src/prompts/path/render.rs:98` and `src/prompts/path/render.rs:129` show
  `uniform_list` + `track_scroll` in `PathPrompt`.

## Minimal working snippet

```rust
use gpui::{
    div, px, uniform_list, Context, ScrollStrategy, UniformListScrollHandle,
    Window,
};

const ITEM_HEIGHT: f32 = crate::list_item::LIST_ITEM_HEIGHT;

struct ExampleList {
    rows: Vec<String>,
    selected_index: usize,
    scroll: UniformListScrollHandle,
}

impl ExampleList {
    fn move_down(&mut self, cx: &mut Context<Self>) {
        if self.selected_index < self.rows.len().saturating_sub(1) {
            self.selected_index += 1;
            self.scroll
                .scroll_to_item(self.selected_index, ScrollStrategy::Nearest);
            cx.notify();
        }
    }

    fn render_rows(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl gpui::IntoElement {
        uniform_list(
            "example-rows",
            self.rows.len(),
            cx.processor(move |this: &mut Self, visible, _window, _cx| {
                visible
                    .map(|ix| {
                        let is_selected = ix == this.selected_index;
                        div()
                            .id(ix)
                            .h(px(ITEM_HEIGHT))
                            .child(format!(
                                "{}{}",
                                if is_selected { "> " } else { "" },
                                this.rows[ix]
                            ))
                    })
                    .collect::<Vec<_>>()
            }),
        )
        .track_scroll(&self.scroll)
        .h_full()
    }
}
```
