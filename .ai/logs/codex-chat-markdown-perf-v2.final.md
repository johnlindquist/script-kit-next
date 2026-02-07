**Status**
Blocked by external in-progress changes; I could not complete verification of the markdown/chat performance patch yet.

**What I ran**
1. `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
2. Re-ran `cargo check` multiple times to confirm stability of failures.

**Current blockers**
- `src/watcher.rs:23`, `src/watcher.rs:25`, `src/watcher.rs:27`, `src/watcher.rs:29`, `src/watcher.rs:31`  
  Uses `config::defaults::*`, but `src/config/mod.rs:14` declares `mod defaults;` (private).
- `src/hotkeys.rs:367`, `src/hotkeys.rs:368`, `src/hotkeys.rs:374`, `src/hotkeys.rs:375`, `src/hotkeys.rs:1169`, `src/hotkeys.rs:1173`  
  `Option<HotkeyConfig>` is passed where `&HotkeyConfig` is required.

These files are currently claimed by `impl-config`, so I could not edit them without violating swarm locking rules.

**Changed files**
- None (no edits made).

**How to test once unblocked**
1. Wait for `impl-config` to release/fix `src/config/mod.rs`, `src/watcher.rs`, `src/hotkeys.rs`.
2. Run:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`

**Risks / known gaps**
- The markdown relayout/scroll fix in `src/prompts/markdown.rs` and chat integration in `src/prompts/chat.rs` could not be fully validated because compilation fails before reaching clippy/tests.

**Commits**
- None.