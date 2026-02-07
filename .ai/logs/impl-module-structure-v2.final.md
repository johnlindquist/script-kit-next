Implemented the top-3 module-structure cleanup in a safe, non-overlapping subset of `mod.rs` files.

**Changed files**
- `src/app_shell/mod.rs:40`
  - Changed internal submodules from `pub mod` to `pub(crate) mod`.
  - Normalized re-exports to `pub use self::...` for consistent import style.
- `src/ai/mod.rs:35`
  - Changed internal submodules from `pub mod` to `pub(crate) mod`.
  - Consolidated/ordered re-export blocks with `self::...` paths.
  - Kept `#![allow(unused_imports)]` to avoid warning churn from broad API re-exports.
- `src/scripts/mod.rs:23`
  - Restricted `input_detection` and `search` modules to `pub(crate) mod`.
  - Normalized top-level re-exports to `self::...`.
  - Tightened test-only re-exports from `pub use` to `pub(crate) use`.

**How to test**
1. Run:
```bash
cargo check
```
2. Result: passes in current tree.

**Risks / known gaps**
- Visibility tightening may affect any external consumers using submodule paths like `script_kit_gpui::ai::window` or `script_kit_gpui::scripts::search`.
- I intentionally did not touch files currently claimed by other active agents, so this is a targeted subset of the full module-structure cleanup.
- `cargo check` still reports existing unrelated warnings in other modules.

**Commits**
- No commits made.