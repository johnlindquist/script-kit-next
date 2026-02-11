<!-- markdownlint-disable MD013 -->

# Prompt Wrapper Pattern

## When to use

- You are rendering a prompt entity (`Entity<...>`) that already owns its
  internal UI, focus, and key handling.
- You need a consistent outer shell (vibrancy/background/rounded frame) and
  app-level key interception (global shortcuts, actions dialogs).
- You are adding a new prompt type and need to wire it through view state,
  prompt handling, and render dispatch.

## Do not do

- Do not duplicate shell frame styling in every prompt entity; use shared
  shell helpers.
- Do not put app-level actions dialog routing directly inside inner prompt
  components unless that prompt truly owns the dialog.
- Do not create prompt entities inside `render_*` methods.
- Do not add a prompt type in only one place; missing `AppView` or render
  dispatch wiring will leave it unreachable.

## Canonical files

- `src/main.rs:295` includes all `render_prompts/*` wrapper files into
  `ScriptListApp`.
- `src/main_sections/render_impl.rs:159` dispatches `AppView` variants to
  `render_*_prompt` wrapper methods.
- `src/render_prompts/other.rs:107` and `src/render_prompts/other.rs:195`
  bracket the simple wrappers (`SelectPrompt`, `EnvPrompt`, `DropPrompt`,
  `TemplatePrompt`, `ChatPrompt`) that use shared shell helpers.
- `src/render_prompts/other.rs:117` and `src/render_prompts/other.rs:190`
  explicitly document wrapper interception before inner prompt key handlers.
- `src/components/prompt_layout_shell.rs:79` defines
  `prompt_shell_container(...)`.
- `src/components/prompt_layout_shell.rs:86` defines
  `prompt_shell_content(...)`.
- `src/prompts/env/render.rs:9` and `src/prompts/env/render.rs:89` show an
  inner prompt implementing `Render` and managing its own `track_focus` +
  `on_key_down`.
- `src/prompts/select/render.rs:96` and `src/prompts/select/render.rs:102`
  show the same inner-component pattern for `SelectPrompt`.
- `src/main_sections/app_view_state.rs:3` defines `AppView` variants for
  prompt entities and focus targets.
- `src/prompt_handler/part_001.rs:1464` and
  `src/prompt_handler/part_001.rs:1487` cover canonical prompt creation
  (`cx.new(...)`), `AppView` assignment, focus target, resize, and
  `cx.notify()`.
- `src/prompts/mod.rs:18` and `src/prompts/mod.rs:69` show module
  registration and re-export of prompt types.

## Adding a new prompt type

1. Add the prompt module and re-export in `src/prompts/mod.rs`.
2. Add an `AppView` variant (and focus target if needed) in
   `src/main_sections/app_view_state.rs`.
3. Create the prompt entity in `prompt_handler` (`cx.new(...)`), set
   `self.current_view`, set `pending_focus`, resize, and `cx.notify()`
   (pattern at `src/prompt_handler/part_001.rs:1464`).
4. Add a `render_*_prompt` wrapper in `src/render_prompts/*.rs`.
5. Route the new `AppView` variant in `src/main_sections/render_impl.rs` to
   that wrapper.
6. Ensure the wrapper file is included in `src/main.rs`.

## Minimal working snippet

```rust
impl ScriptListApp {
    fn render_my_prompt(
        &mut self,
        entity: gpui::Entity<prompts::MyPrompt>,
        cx: &mut gpui::Context<Self>,
    ) -> gpui::AnyElement {
        let shell_radius = crate::designs::get_tokens(self.current_design)
            .visual()
            .radius_lg;
        let vibrancy_bg =
            crate::ui_foundation::get_vibrancy_background(&self.theme);
        let handle_key = cx.listener(Self::other_prompt_shell_handle_key_default);

        crate::components::prompt_shell_container(shell_radius, vibrancy_bg)
            .on_key_down(handle_key) // app-level interception first
            .child(crate::components::prompt_shell_content(entity))
            .into_any_element()
    }
}

impl gpui::Render for prompts::MyPrompt {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let handle_key = cx.listener(
            |this: &mut Self, _event: &gpui::KeyDownEvent, _window, cx| {
                // Inner prompt state changes happen here.
                this.cursor_visible = true;
                cx.notify();
            },
        );

        gpui::div()
            .w_full()
            .h_full()
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child("My prompt")
    }
}
```
