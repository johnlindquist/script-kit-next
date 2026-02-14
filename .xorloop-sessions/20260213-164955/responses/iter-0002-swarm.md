All 3 tasks completed and `cargo check` passes. Here's the summary:

## Swarm Results

| Task | Commit | Status |
|------|--------|--------|
| **confirm-window-safety** | `a999b613` fix(confirm): replace expect/swallowed errors with proper error handling | Done |
| **hotkeys-init-safety** | `39e44c15` fix(hotkeys): guard MAIN_MANAGER access before lock | Done |
| **logging-fallback-safety** | `3192dfb7` fix(logging): replace /dev/null fallback with io::sink | Done |

All 3 commits are on the branch, `cargo check` passes. The fixes address:

1. **`src/confirm/window.rs`** — Replaced `.expect()` with proper error propagation; added `tracing::error!` logging for poisoned mutex locks instead of silently swallowing
2. **`src/hotkeys/mod.rs`** — Replaced `.unwrap()` on `MAIN_MANAGER.get()` with a match that logs and returns early if uninitialized
3. **`src/logging/mod.rs`** — Replaced `/dev/null` `.expect()` fallbacks with `std::io::sink()` which is cross-platform and cannot fail
