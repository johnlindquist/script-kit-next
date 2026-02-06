# Webcam Prompt Research Notes

## PromptBase Usage and Constructor Pattern
- Use `PromptBase` from `src/prompts/base.rs` as shared fields (`id`, `focus_handle`, `on_submit`, `theme`, `design_variant`) with constructor `PromptBase::new(id, focus_handle, on_submit, theme)` and builder `with_design` for variant selection. See lines `src/prompts/base.rs:36-92`.

## Focusable Implementation
- Common pattern: implement `Focusable` by cloning `focus_handle` (e.g., `DivPrompt` in `src/prompts/div.rs:855-858` and `ChatPrompt` in `src/prompts/chat.rs:2615-2618`).
- Macro helper `impl_focusable_via_base!` exists in `src/prompts/base.rs:198-241` for structs embedding `PromptBase`.

## Keyboard Event Handling and Key Matching
- `DivPrompt` renders key handler `handle_key` matching `"enter"` and `"escape"` in `src/prompts/div.rs:867-877`.
- `ChatPrompt` key handling via `handle_key` around `src/prompts/chat.rs:2670-2760` checks lowercase keys (`"escape"`, `"enter"`, `"k"` for `⌘K`, `"c"` for copy etc.) and setup mode using `resolve_setup_card_key` (`src/prompts/chat.rs:78-150` includes arrow key variants like `"up"`/`"arrowup"` and tab navigation).

## State Management (`cx.notify`)
- `ChatPrompt` uses `cx.notify()` after state mutations in key handling and provider setup (`src/prompts/chat.rs:2685-2690` for setup focus changes, `src/prompts/chat.rs:1132-1150` after submission and state updates, and `src/prompts/chat.rs:1139`/`116...` where state is changed and UI updates trigger notify).
- Use `cx.notify()` to trigger rerenders after updates to prompts and state changes.

## Theme Color Usage and DesignContext
- `DesignContext` in `src/prompts/base.rs:127-241` resolves colors based on theme or design tokens and provides helper methods (`bg_main`, `text_primary`, etc.)
- `DivPrompt` fetches design tokens/colors via `get_tokens(self.design_variant)` and `colors = tokens.colors()` in `src/prompts/div.rs:863-866`.
- `ChatPrompt` stores theme colors in `prompt_colors` and uses theme values via `self.prompt_colors` in rendering (`src/prompts/chat.rs` around `1132` and `2615` for usage).

## Submit Callback Pattern
- `PromptBase` exposes `submit(value: Option<String>)` and `cancel()` in `src/prompts/base.rs:81-90`.
- `DivPrompt` submits via internal `submit()` and `submit_with_value()` methods (`src/prompts/div.rs:244-252`).
- `ChatPrompt` handles submission through `handle_submit` calling `on_submit` when not in built-in AI mode (`src/prompts/chat.rs:1132-1151`).

## Verification
- Created `src/prompts/webcam.rs` with `WebcamState` enum (`Initializing`, `Live`, `Countdown`, `Captured`, `Error`) and `WebcamPrompt` struct (`base`, `state`, `mirror`, `countdown_seconds`, `captured_data`, `frame_data`, `frame_width`, `frame_height`, `error_message`).
- Constructor `WebcamPrompt::new` initializes PromptBase and default state/fields; methods `update_frame`, `set_captured`, and `set_error` update state and call `cx.notify()`.
- Implemented `Focusable` via cloned `PromptBase` handle and `Render` with key handling for `enter/space` (capture), `escape`, `m` (mirror), `r` (retake), `+`/`-` (timer) and state label rendering.
- Prompt rendering uses `PromptBase`/`DesignContext` theme colors (`dc.bg_main()`, `dc.bg_secondary()`, `dc.border()`, `colors.text_primary`, `colors.text_secondary`, `colors.text_tertiary`), and state/placeholder frame details. `cx.notify()` called in state changes.
- Known limitations: camera frame capture not yet wired; frame data and `set_error` are placeholders until backend feed is implemented.
- Compilation (`cargo check`) currently fails due to unresolved `render_webcam_prompt` method and non-exhaustive match arms in app views (`src/main.rs`, `src/app_impl.rs`, `src/app_render.rs`, `src/prompt_handler.rs`, `src/app_execute.rs`, `src/app_layout.rs`), plus unused import warning `src/prompts/mod.rs` (`pub use webcam::WebcamState` unused).
