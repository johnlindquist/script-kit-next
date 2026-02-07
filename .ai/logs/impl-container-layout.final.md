Implemented the high-priority container/layout fixes in scope.

Changed files:
- `src/components/prompt_layout_shell.rs`
  - Added shared frame contract type `PromptFrameConfig` and helpers:
    - `prompt_frame_root(...)`
    - `prompt_frame_fill_content(...)`
    - `prompt_shell_frame_config(...)`
  - Updated `prompt_shell_container`/`prompt_shell_content` to use shared helpers.
  - Added behavior tests for default frame contract and shell config (`test_prompt_frame_defaults_apply_min_h_and_overflow_hidden`, `test_prompt_shell_frame_config_sets_relative_and_radius`).

- `src/components/prompt_container.rs`
  - Added explicit content policy enum `PromptContainerContentLayout` with default `Fill`.
  - Extended `PromptContainerConfig` with `content_layout` + builder.
  - Added `prompt_container_frame_config(...)` and switched root construction to shared `prompt_frame_root(...)` so `min_h(0)` + overflow policy match shell behavior.
  - Removed duplicated inline divider/hint rendering and routed through shared helper methods.
  - Content slot now applies fill wrapper only when policy is `Fill`; `Intrinsic` leaves content unwrapped.
  - Added tests:
    - `test_prompt_container_content_fill_mode_wraps_child_with_flex_1`
    - `test_prompt_container_intrinsic_mode_leaves_content_unwrapped`
    - `test_prompt_shell_and_prompt_container_share_same_root_contract`

How to test:
1. `cargo check`
2. `cargo test --lib test_prompt_frame_defaults_apply_min_h_and_overflow_hidden -- --exact`
3. `cargo test --lib test_prompt_shell_and_prompt_container_share_same_root_contract -- --exact`

Verification run:
- `cargo check` passed.
- `cargo test` is currently blocked by an unrelated pre-existing compile error in `src/ai/window.rs:8549` (`AiApp::message_body_content` not found), outside this scope.

Risks / known gaps:
- New tests are contract-level unit tests; they don’t assert rendered GPUI tree behavior yet.
- `PromptContainerContentLayout::Intrinsic` is implemented but not yet adopted by call sites.
- `src/components/prompt_layout_shell.rs` is currently untracked in this working tree; ensure that is intentional before final integration.

Commits made:
- None.