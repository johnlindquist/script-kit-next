**Audit Findings (AI Window)**
1. High: Image attachments could be silently dropped on submit when early-return conditions triggered.
`src/ai/window/streaming_submit.rs:16` now gates submit without consuming the image, and `pending_image.take()` is delayed to `src/ai/window/streaming_submit.rs:40`.
2. Medium: Chat deletion left stale UI/resource state (drafts, collapse/expand state, rename/delete/edit IDs, image cache), which could cause stale behavior and memory growth.
Fixed in `src/ai/window/interactions.rs:201` and helper logic in `src/ai/window/types.rs:79`.
3. Medium: Commands were queued even when the AI window was closed, allowing stale queued actions to replay later.
Fixed in `src/ai/window/window_api.rs:313`, `src/ai/window/window_api.rs:348`, `src/ai/window/window_api.rs:386` via `src/ai/window/types.rs:90`.
4. Low: Duplicate delete paths could diverge in cleanup behavior.
`src/ai/window/chat.rs:215` now delegates to `delete_chat_by_id`.

**Changed Files**
- `src/ai/window/streaming_submit.rs`
- `src/ai/window/interactions.rs`
- `src/ai/window/chat.rs`
- `src/ai/window/window_api.rs`
- `src/ai/window/types.rs`
- `src/ai/window/tests.rs`

**Tests/Lint Run**
- `cargo check --lib`
- `cargo clippy --lib -- -D warnings`
- `cargo test --lib ai::window::tests`

Added tests:
- `src/ai/window/tests.rs:79`
- `src/ai/window/tests.rs:99`
- `src/ai/window/tests.rs:133`

**Risks / Known Gaps**
- Commit hygiene issue: commit `52a9783` unexpectedly includes non-scope files (`src/agents/parser.rs`, `src/terminal/**`) likely from hook auto-staging.
- In this parallel-agent run, I used scoped verification (per parallel-safe constraint), not full-workspace `cargo test`.

**Commits**
- `52a97830937f5ef56cdc7033f894b6b97926ab18` — `fix(ai-window): harden submit and cleanup state`

Need your direction on git history: keep `52a9783` as-is, or have me rewrite/split it once conflicting file owners release claims.