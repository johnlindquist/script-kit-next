Verified and committed.

- The uncommitted theme chooser changes in `src/render_builtins/theme_chooser.rs` compile and lint clean.
- I confirmed interactive row affordances are present (`.cursor_pointer()` and hover backgrounds) in the theme list rendering, e.g. `src/render_builtins/theme_chooser.rs:607`, `src/render_builtins/theme_chooser.rs:613`, `src/render_builtins/theme_chooser.rs:615`.

Changed files:
- `src/render_builtins/theme_chooser.rs`

Commits made:
- `b77f71a` `feat(theme-chooser): preserve pointer and hover row affordances`

How to test:
1. `CARGO_BUILD_JOBS=2 cargo check --lib`
2. `CARGO_TARGET_DIR=target-codex-verify-theme-chooser-v2 CARGO_BUILD_JOBS=1 cargo clippy --lib -- -D warnings`

Risks / known gaps:
- I did not run `cargo test` or full-workspace checks due the task’s scoped verification requirement and parallel-agent lock contention.
