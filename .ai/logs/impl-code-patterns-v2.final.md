Implemented the top 5 consistency fixes from `plans/code-pattern-consistency.md` in the requested scope.

**Changed files**
- `src/window_control.rs:240`  
  - Replaced CFString creation `unwrap` path with fallible `try_create_cf_string(...) -> anyhow::Result<_>` and propagated with `?`.
  - Updated AX minimize path to use fallible string creation (`AXMinimized`).
  - Added test: `src/window_control.rs:1636`.
- `src/menu_executor.rs:164`  
  - Replaced CFString `unwrap` with fallible helper + `?`.
  - Removed `menu_item.unwrap()` by switching to `let Some(menu_item) = ... else { ... }` (`src/menu_executor.rs:555`).
- `src/menu_bar.rs:275`  
  - Replaced CFString `unwrap` with fallible helper + `?`.
- `src/window_control_enhanced/capabilities.rs:137`  
  - Replaced CFString `unwrap` with non-panicking helper returning `Option<_>` and early-return handling.
  - Added test: `src/window_control_enhanced/capabilities.rs:329`.
- `src/shortcuts/types.rs:204`  
  - Normalized arrow-key aliases (`up|arrowup`, etc.) in display/canonical paths (`key_display`, `key_display_text`, `to_canonical_string`).
  - Added tests in `src/shortcuts/types_tests.rs:249` and `src/shortcuts/types_tests.rs:283`.
- Additional tests for CFString interior-NUL handling:
  - `src/menu_executor_tests.rs:109`
  - `src/menu_bar_tests.rs:13`

**How to test**
1. `cargo check`
2. `cargo test try_create_cf_string_rejects_interior_nul -- --nocapture`
3. `cargo test normalizes_arrow_alias_key_names -- --nocapture`

I also ran:
- `cargo clippy --all-targets -- -D warnings` (fails on pre-existing unrelated warnings/errors in other files, not introduced by this change set).

**Risks / known gaps**
- AX helper logic is still duplicated across modules; this pass removed panic-style construction and normalized error propagation, but did not fully centralize helper implementation.
- `clippy -D warnings` is currently red repo-wide due unrelated existing issues, so full lint gate cannot pass until those are addressed.

**Swarm coordination**
- Status updated to `done`.
- All claimed files released.

**Commits**
- No commits made by me in this task.