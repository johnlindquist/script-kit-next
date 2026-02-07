Implemented the scoped AI chat UX fixes in `src/ai/window.rs` and `src/prompts/chat.rs`.

**Changed Files**
- `src/ai/window.rs:119`  
  Added stale-stream session tracking (`StreamingSessionKey`) and suppression set to prevent orphan/stale completion persistence after explicit stop/delete.
- `src/ai/window.rs:124`  
  Added `should_retry_existing_user_turn(...)` and updated retry flow to reuse the existing last user turn instead of duplicating a message.
- `src/ai/window.rs:1630`  
  Added helpers to mark/suppress current streaming session and gate orphan persistence.
- `src/ai/window.rs:1675`  
  Updated sidebar search to use `storage::search_chats(...)` (content + title), with title-filter fallback on storage error.
- `src/ai/window.rs:8469`  
  Added unit tests for retry decision logic and stale-persistence suppression behavior.
- `src/prompts/chat.rs:218`  
  Added `should_ignore_stream_reveal_update(...)` guard.
- `src/prompts/chat.rs:1604`  
  Updated streaming reveal loop to stop updating when stream ownership changes/stops (prevents stale reveal updates).
- `src/prompts/chat.rs:3255`  
  Added unit test for stream-reveal stale-update guard.

**Verification**
- `cargo check` passed.
- Targeted unit test runs are currently blocked by a pre-existing unrelated compile error:
  - `src/ai/window.rs:8729` references missing `AiApp::message_body_content`.
- Stdin JSON smoke run executed successfully (protocol parsed + chat prompt shown):
  - command parsed as `run`
  - `ShowChat prompt received`
  - `ChatPrompt with built-in AI` logged

**How To Test**
1. `cargo check`
2. `cargo test --lib` (currently expected to fail on the pre-existing `AiApp::message_body_content` error)
3. Stdin protocol smoke:
   - `TEST_PATH="$(pwd)/tests/smoke/test-chat-simple.ts"; timeout 20 bash -lc "printf '%s\n' '{\"type\":\"run\",\"path\":\"$TEST_PATH\"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui > /tmp/chat-smoke.log 2>&1"`
   - `rg -n "Parsed external command|ShowChat prompt received|ChatPrompt with built-in AI" /tmp/chat-smoke.log`

**Risks / Known Gaps**
- Full `cargo test` verification is blocked by existing unrelated test compile breakage in `src/ai/window.rs:8729`.
- End-to-end assertion of stale-completion persistence suppression in live streaming depends on broader AI/provider runtime conditions; core behavior is covered by the new helper tests.

**Commits**
- No commits were created.