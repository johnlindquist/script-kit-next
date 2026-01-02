# GPUI Component Recommendations

## Context
- Custom UI components live in `src/components/*` (Button, Toast, Scrollbar, Form*, PromptHeader/Container, TextInputState).
- `gpui-component` is already in use and themed in `src/theme.rs`, `src/main.rs`, `src/ai/window.rs`, `src/notes/window.rs`.
- The main header already uses `gpui-component` Input for filter (`src/render_script_list.rs`), while other prompts still use custom inputs/buttons.

## High-confidence swaps
### Button -> gpui-component Button
- Custom: `src/components/button.rs`
- Used in: `src/render_script_list.rs`, `src/render_prompts.rs`, `src/components/prompt_header.rs`
- Candidate: `gpui_component::button::Button` + `ButtonVariants`
- Why: already used in notes/browse panels; theme sync is in place; reduces duplicated hover/disabled styling.
- Caveat: custom shortcut labels ("Enter", "Cmd+K") may need a wrapper or a trailing element slot.

### Toasts -> gpui-component Notifications
- Custom: `src/components/toast.rs` + `src/toast_manager.rs`
- Used in: `src/prompt_handler.rs`, `src/app_execute.rs`, `src/render_builtins.rs`
- Candidate: `gpui_component::notification::Notification`
- Why: toasts already get converted in `ToastManager` before render; creating Notifications earlier removes indirection.
- Caveat: custom Toast supports details/actions; keep custom if those are still required.

### Form fields -> gpui-component inputs
- Custom: `src/components/form_fields.rs`
- Used in: `src/form_prompt.rs`
- Candidate: `gpui_component::input::Input` for single-line, plus checkbox/textarea equivalents if available.
- Why: eliminates custom selection/clipboard handling and focus management, matches other gpui-component input behavior.

## Medium-confidence / needs evaluation
### TextInputState in prompts
- Custom: `src/components/text_input.rs`
- Used in: `src/prompts/env.rs`, `src/render_prompts.rs`, arg prompt state in `src/main.rs`
- Candidate: `gpui_component::input::InputState` + `Input`
- Benefit: less custom editing logic to maintain.
- Risk: prompt UI relies on custom cursor placement and prefix rendering; may need styling work or a wrapper.

### Scrollbar
- Custom: `src/components/scrollbar.rs`
- Used in: `src/render_script_list.rs`, `src/actions.rs`
- Candidate: gpui-component scroll/scrollbar if exposed.
- Note: no usage found in the app; confirm gpui-component has a scrollbar before investing.

## Keep custom (for now)
- `PromptHeader` and `PromptContainer` in `src/components/prompt_header.rs` and `src/components/prompt_container.rs` encode layout-specific behavior (cursor alignment, CLS-free toggles, path prefix). There is no clear gpui-component equivalent yet.

## Suggested rollout
1. Swap button usage in main header and prompt buttons to gpui-component Button.
2. Replace FormPrompt inputs with gpui-component Input/Checkbox (if available).
3. Decide whether to keep custom Toast or fully adopt Notification.
4. Revisit TextInputState in Env/Arg prompts once styling requirements are clarified.
