Implemented the `src/platform.rs` split into a `src/platform/` module directory, with every extracted file under 500 lines.

**Changed files**
- Deleted: `src/platform.rs`
- Added: `src/platform/mod.rs`
- Added: `src/platform/app_window_management.rs`
- Added: `src/platform/visibility_focus.rs`
- Added: `src/platform/vibrancy_swizzle_materials.rs`
- Added: `src/platform/vibrancy_config.rs`
- Added: `src/platform/vibrancy_cycle.rs`
- Added: `src/platform/secondary_window_config.rs`
- Added: `src/platform/positioning.rs`
- Added: `src/platform/screenshots_window_open.rs`
- Added: `src/platform/ai_commands.rs`
- Added: `src/platform/tests.rs`

**How to test**
1. Run:
```bash
cargo check && cargo clippy --all-targets -- -D warnings && cargo test
```
2. Optional structure check:
```bash
wc -l src/platform/*.rs
```

**Verification run result**
- I ran all required commands.
- `cargo check` currently fails due unrelated in-progress parallel refactors outside `src/platform*` (examples: `src/notes/mod.rs:56`, `src/notes/window/render_editor_titlebar.rs:81`).
- `cargo clippy --all-targets -- -D warnings` fails on existing non-platform warnings/errors (examples: `src/app_shell/keymap.rs:176`, `src/actions/dialog.rs:308`).
- `cargo test` fails on unrelated non-platform breakages (examples: `src/main_sections/fonts.rs:10`, `src/app_impl/mod.rs:2`, `src/ai/window/tests.rs:256`).

**Risks / known gaps**
- Full repo verification gate is currently blocked by concurrent non-platform changes, so end-to-end green status is not attainable in this snapshot.
- The split preserves behavior via section extraction and ordered `include!` composition in `src/platform/mod.rs` (not deep API refactoring).

**Commits**
- None made.