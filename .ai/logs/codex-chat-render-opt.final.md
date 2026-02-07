Implemented AI chat render-path optimizations in:

- `src/prompts/chat.rs`
- `src/prompts/markdown.rs`

What changed:

- `src/prompts/chat.rs:221` added `build_conversation_turns(...)` and moved turn construction out of render.
- `src/prompts/chat.rs:422` added cached turn state (`conversation_turns_cache`, dirty flag, `ListState`) and synchronized list sizing/count.
- `src/prompts/chat.rs:810` added cache invalidation/rebuild helpers and wired them into message mutation paths.
- `src/prompts/chat.rs:2914` replaced non-virtualized full message rendering with virtualized `list(...)` + `ListSizingBehavior::Infer`, so only visible turns render.
- `src/prompts/chat.rs:2735` ensures turn cache is refreshed before render; input stays separate from expensive per-message recompute.
- `src/prompts/markdown.rs:99` changed code-block raw text from `String` to `Arc<str>` to reduce per-render cloning.
- `src/prompts/markdown.rs:112` changed markdown cache value type to `Arc<Vec<ParsedBlock>>` to avoid deep clones on cache hits.
- `src/prompts/markdown.rs:490` copy-button path now clones `Arc<str>` and allocates `String` only on click.
- `src/prompts/markdown.rs:708` markdown render now reuses shared parsed blocks (`Arc`) and builds elements from slice.

Validation run:

- `cargo check --lib` ✅
- `cargo clippy --all-targets -- -D warnings` ❌ blocked by unrelated current-branch issues (action test files + `src/prompts/select.rs`).
- `cargo test` ❌ blocked by unrelated current-branch failures in:
  - `src/actions/dialog_builtin_action_validation_tests_39.rs`
  - `src/actions/dialog_builtin_action_validation_tests_41.rs`
  - `src/actions/dialog_builtin_action_validation_tests_46.rs`
- Runtime verification via stdin JSON protocol ✅
  - Parsed `Run` command
  - `ShowChat received` with `message_count=240`
  - screenshot captured successfully
  - verified image: `.test-screenshots/chat-render-perf-1770422800924.png` (750x500 PNG)

How to test:

1. `cargo check --lib`
2. Run stress scenario via stdin JSON (not CLI args), e.g. your chat stress script:
   - `echo '{"type":"run","path":"/tmp/test-chat-render-perf.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`
3. Confirm logs include `ShowChat received` and `Screenshot captured successfully` in `~/.scriptkit/logs/script-kit-gpui.jsonl`.
4. Open `.test-screenshots/chat-render-perf-1770422800924.png` and verify chat renders correctly with long history.

Risks / known gaps:

- Full repo clippy/test gate currently failing from unrelated parallel-agent changes, so only scoped behavior/build/runtime was validated.
- Virtualized list uses inferred sizing; very large variable-height markdown blocks can still incur measurement cost, but only for visible rows.