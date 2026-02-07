Implemented the idle-performance fix for the AI chat window in `src/ai/window.rs`.

**Changed files**
1. `src/ai/window.rs`

**What I changed**
1. Virtualized the sidebar list (date headers + chats) instead of rendering all sidebar items every frame.
   - Added row model and builder: `src/ai/window.rs:212`, `src/ai/window.rs:285`
   - Added sidebar `ListState`: `src/ai/window.rs:472`, `src/ai/window.rs:817`
   - Added count sync helper: `src/ai/window.rs:3580`
   - Switched `render_sidebar` to virtualized `list(...)` + `vertical_scrollbar(...)`: `src/ai/window.rs:3588`
2. Replaced full group rendering with lightweight header row rendering:
   - `src/ai/window.rs:3812`
3. Removed high-frequency hot-path debug logging that was running during idle/typing/render:
   - setup-card render logging removed around `src/ai/window.rs:4463`
   - main-panel render debug logging removed around `src/ai/window.rs:5816`
   - per-key capture logging removed around `src/ai/window.rs:6493`
4. Added a unit test for sidebar row construction correctness:
   - `src/ai/window.rs:8475`

**Validation run**
1. `cargo test test_build_sidebar_rows_inserts_headers_and_preserves_chat_order --lib`
   - Blocked by unrelated compile error outside this scope: `src/prompts/select.rs:376` (`E0282`, type annotation needed for `ranges`).
2. Earlier scoped checks and runtime log verification were performed, but current full compile/test gates are blocked by unrelated in-progress changes from other agents.

**How to test**
1. Build/check (once unrelated compile blocker is fixed):
```bash
cargo check
cargo clippy --all-targets -- -D warnings
cargo test
```
2. Runtime idle perf check (stdin protocol):
```bash
echo '{"type":"openAi"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```
3. In the AI window, verify:
   - typing latency is reduced,
   - sidebar scroll is smooth with many chats,
   - no repeated render/key debug spam in `~/.scriptkit/logs/script-kit-gpui.jsonl`.

**Risks / known gaps**
1. Repo-wide verification is currently blocked by unrelated `src/prompts/select.rs` compile errors.
2. Another agent is concurrently optimizing message-list rendering in `src/prompts/chat.rs`/`src/prompts/markdown.rs`; combined impact should be re-validated after their merge.