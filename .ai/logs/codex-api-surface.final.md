Implemented API-surface tightening and wrote the requested report at `plans/api-surface-cleanup.md`.

**Changed Files**
- `plans/api-surface-cleanup.md`
  - New report with implemented changes, verification status, and prioritized follow-up cleanup plan.
- `src/window_control_enhanced/mod.rs:18`
  - Removed public re-exports, added `compile_fail` API-visibility contract doctest, kept internals crate-only.
- `src/lib.rs:69`
  - `window_control_enhanced` reduced from `pub mod` to `pub(crate) mod`.
- `src/lib.rs:271`
  - `MAIN_WINDOW_VISIBLE` changed from `pub static` to private `static`.
- `src/lib.rs:276`
  - `SCRIPT_REQUESTED_HIDE` changed from `pub static` to private `static`.
- `src/theme/color_resolver.rs:28`
  - Added `compile_fail` doctest for field privacy contract.
- `src/theme/color_resolver.rs:54`
  - Made resolver fields private (`ColorResolver`, `TypographyResolver`, `SpacingResolver`), added needed accessor methods, updated tests.
- `src/render_script_list.rs:856`
  - Migrated resolver usage from direct field access to semantic accessor methods.

**How To Test**
1. `cargo check`
2. `cargo test --doc window_control_enhanced -- --nocapture`
3. `cargo test --doc color_resolver -- --nocapture`
4. `cargo build`
5. `timeout 8 bash -c 'echo "{\"type\":\"show\"}" | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1'`
6. Optional full gate (currently blocked by unrelated workspace issues): `cargo clippy --all-targets -- -D warnings && cargo test`

**Verification Results**
- Passed: `cargo check`
- Passed: both doctest targets above
- Passed runtime sanity: stdin JSON `Show` command parsed and handled in logs
- Fails (unrelated pre-existing errors outside this task): `cargo clippy --all-targets -- -D warnings`, `cargo test`  
  - Includes `src/ai/window.rs` missing `AiApp::message_body_content`, duplicate test module conflict, and many unused imports in action validation test files.

**Risks / Known Gaps**
- External consumers (if any) that relied on `script_kit_gpui::window_control_enhanced` internals will break due tightened visibility.
- Full repository clippy/test gate is not green due unrelated in-flight errors.
- `#![allow(dead_code)]` remains on internalized modules to avoid noisy warnings during incremental cleanup.

**Commits**
- None made.