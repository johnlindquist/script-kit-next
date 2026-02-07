Implemented the memory-safety fixes from `plans/memory-safety-audit.md` for the high-risk areas.

**Changed Files**
- `src/menu_executor.rs`
- `src/menu_executor_tests.rs`
- `src/window_control.rs`
- `src/window_manager.rs`

**What Changed**
- `src/menu_executor.rs`
  - Added `CFRetain` + helper.
  - Added RAII wrapper `OwnedAxElement` with `Drop`-based `CFRelease`.
  - Fixed menu traversal to retain AX children that outlive parent `CFArray`.
  - Updated submenu navigation to keep owned submenu refs alive during traversal.
- `src/menu_executor_tests.rs`
  - Added retain/release regression test:
  - `test_owned_ax_element_retain_release_when_created_from_borrowed`.
- `src/window_control.rs`
  - Added RAII wrapper `OwnedCachedWindowRef` with `Drop` release.
  - `cache_window` now releases replaced cached pointer (fixes overwrite leak).
  - `get_cached_window` now retains under lock and returns owned wrapper (fixes cross-lock lifetime risk).
  - Updated window operations to consume owned cached refs safely.
  - Added cache ownership tests:
  - `test_window_cache_releases_previous_pointer_on_overwrite`
  - `test_window_cache_get_returns_owned_reference_and_releases_on_drop`
- `src/window_manager.rs`
  - Replaced raw pointer wrapper + unsafe `Send/Sync` with `RegisteredWindowHandle` metadata.
  - Added liveness-checked resolution (`NSApp` windows scan) before returning a window handle.
  - Added main-thread guardrails for register/get/unregister/find.
  - Prunes stale registry entries when resolution fails.
  - Updated tests for new handle model.

**Verification Run**
- `cargo check` ✅
- `cargo check --lib` ✅
- `cargo test test_owned_ax_element_retain_release_when_created_from_borrowed -- --nocapture` ✅
- `cargo test test_window_cache_releases_previous_pointer_on_overwrite -- --nocapture` ✅
- `cargo test test_window_cache_get_returns_owned_reference_and_releases_on_drop -- --nocapture` ✅
- `cargo test test_registered_window_handle_wrapper -- --nocapture` ✅
- `cargo test test_window_manager_struct -- --nocapture` ✅
- `cargo clippy --lib -- -D warnings` ❌ (blocked by pre-existing unrelated warnings in `src/actions/dialog.rs` and `src/prompts/div.rs`)

**Risks / Known Gaps**
- `window_manager` now enforces main-thread usage for register/get/unregister/find; off-main callers now log and return `None`/no-op.
- Full clippy cleanliness is still blocked by unrelated existing warnings outside this change set.

**How to Test**
1. `cargo check`
2. `cargo test test_owned_ax_element_retain_release_when_created_from_borrowed -- --nocapture`
3. `cargo test test_window_cache_releases_previous_pointer_on_overwrite -- --nocapture`
4. `cargo test test_window_cache_get_returns_owned_reference_and_releases_on_drop -- --nocapture`
5. `cargo test test_registered_window_handle_wrapper -- --nocapture`

Commits made: none.