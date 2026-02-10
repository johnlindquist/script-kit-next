Implemented the footer button audit in `src/components/prompt_footer.rs`.

**Changed files**
- `src/components/prompt_footer.rs`
  - Replaced footer button rendering to enforce clickable-state behavior directly in this file.
  - Added explicit `.cursor_pointer()` for clickable footer buttons (`Run Command` / `Actions`) in `render_button` (`src/components/prompt_footer.rs:361`).
  - Switched hover background to theme-token-based color (`accent.selected_subtle` via `PromptFooterColors.background`) with `footer_button_hover_rgba` (`src/components/prompt_footer.rs:156`, `src/components/prompt_footer.rs:336`).
  - Kept disabled state on default cursor + reduced opacity (`src/components/prompt_footer.rs:363`).
  - Added helper tests for clickable-state logic, activation key handling, and hover token usage (`src/components/prompt_footer.rs:620`).

**How to test**
1. `cargo check -p script-kit-gpui`
2. `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`
3. `cargo test -p script-kit-gpui --test prompt_footer`
4. `cargo test -p script-kit-gpui prompt_footer::tests::`

All commands passed.

**Risks / known gaps**
- I ran scoped checks/tests only (parallel-agent safe), not an unscoped workspace run.
- No manual screenshot-based visual validation was run in this pass.