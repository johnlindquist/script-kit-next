Implemented the scoped render-wrapper optimization in `src/render_prompts/other.rs`.

**Changed files**
- `src/render_prompts/other.rs`
  - Added shared helpers:
    - `other_prompt_shell_radius_lg`
    - `other_prompt_shell_handle_key_default`
    - `other_prompt_shell_handle_key_chat`
    - `other_prompt_shell_handle_key_webcam`
  - Updated `render_select_prompt`, `render_env_prompt`, `render_drop_prompt`, `render_template_prompt`, and `render_chat_prompt` to:
    - stop computing unused `hex_to_rgba_with_opacity(...)`
    - stop allocating unused `create_box_shadows()`
    - use shared key-handler methods via `cx.listener(...)`
  - Updated webcam wrapper to use shared key handler and removed the needless-return style branch.
  - Added regression test module at the bottom:
    - `simple_prompt_wrappers_skip_unused_shell_allocations`

**How to test**
1. `cargo check`  
   - Passes.
2. (Optional targeted test) `cargo test --bin script-kit-gpui simple_prompt_wrappers_skip_unused_shell_allocations -- --nocapture`  
   - Currently blocked by unrelated existing test-compile errors outside scope (`src/render_prompts/arg.rs`, `src/ai/window.rs`).

**Risks / known gaps**
- Full `cargo test` / targeted test execution is currently blocked by pre-existing unrelated compile issues in other files.
- Key routing is centralized for these wrappers now; future prompt-specific key behavior should avoid accidentally reusing the default handler where semantics differ.

**Commits**
- None made.