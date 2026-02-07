Implemented a scoped robustness fix for the chat auto-follow logic and added regression tests in `src/prompts/chat.rs`.

**Changed files**
- `src/prompts/chat.rs:249`  
  Switched near-bottom rejoin math to overflow-safe arithmetic:
  `scroll_top_item_ix.saturating_add(CHAT_SCROLL_BOTTOM_REJOIN_BUFFER_ITEMS) >= total_items`
- `src/prompts/chat.rs:3544`  
  Added `chat_scroll_follow_state_keeps_manual_mode_when_not_near_bottom`
- `src/prompts/chat.rs:3552`  
  Added `chat_scroll_follow_state_handles_large_scroll_indices_without_panicking` (guards overflow/panic edge case)

`src/ai/session.rs` and `src/ai/window.rs` were not edited by me in this pass.

**Verification run**
- `cargo check` -> fails due unrelated existing errors in `src/watcher.rs` and `src/hotkeys.rs`
- `cargo clippy --all-targets -- -D warnings` -> same unrelated blockers (+ existing unused imports)
- `cargo test` -> same unrelated blockers

No new failures from `src/prompts/chat.rs` appeared before compilation stopped.

**How to test (after external blockers are fixed)**
1. `cargo check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`
4. `cargo test chat_scroll_follow_state_`
5. Runtime behavior check:
   - build app, open AI/chat window
   - scroll up while streaming: auto-follow should stop and show “Jump to latest”
   - scroll back near bottom or click “Jump to latest”: auto-follow should resume

**Risks / known gaps**
- Full project verification is currently blocked by out-of-scope compile errors in `src/watcher.rs` and `src/hotkeys.rs`.
- I could not run the new tests to completion until those blockers are resolved.

Commits made: none.