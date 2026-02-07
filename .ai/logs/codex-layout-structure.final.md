Implemented a layout-structure normalization for prompt wrappers by introducing a shared shell and moving the simpler prompt renderers onto it.

**Changed Files**
- `src/components/prompt_layout_shell.rs:1`
- `src/components/mod.rs:39`
- `src/render_prompts/other.rs:51`

**What Changed**
- Added shared layout helpers:
  - `prompt_shell_container(...)` for consistent root structure (`relative + flex_col + w_full/h_full + min_h(0) + overflow_hidden + rounded`).
  - `prompt_shell_content(...)` for consistent content slot (`flex_1 + min_h(0) + overflow_hidden`).
- Refactored wrappers in `src/render_prompts/other.rs` to use these helpers:
  - `render_select_prompt`
  - `render_env_prompt`
  - `render_drop_prompt`
  - `render_template_prompt`
  - `render_chat_prompt`
- Added a regression test in `src/components/prompt_layout_shell.rs:49` that enforces helper usage in those wrappers.

**How To Test**
1. `cargo check`
2. `cargo test --lib simple_prompt_wrappers_use_shared_layout_shell -- --nocapture`
3. UI smoke via stdin protocol:
   - `echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-select-actions-cmdk.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui`
   - `echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-env-prompt.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui`
   - `echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-chat-prompt.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui`

**Risks / Known Gaps**
- `cargo clippy --all-targets -- -D warnings` currently fails due pre-existing unrelated warnings/lints in action validation tests and `src/ai/window.rs`.
- Full `cargo test` currently fails due concurrent-agent WIP in `tests/footer_button.rs` (unresolved imports).
- This pass normalized wrapper skeletons in `render_prompts/other.rs`; prompt-specific internals still intentionally vary by prompt type.