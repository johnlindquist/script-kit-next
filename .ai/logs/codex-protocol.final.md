**Status**
Completed the `src/protocol/*` split and fixed one protocol regression I found afterward (`src/protocol/io/parsing.rs` was truncated at `serialize_message` and had an unclosed delimiter).

All protocol files are now under 500 lines (`max = 465` via `find src/protocol -name '*.rs' | xargs wc -l`).

**Changed Files**
- Deleted: `src/protocol/message.rs`
- Deleted: `src/protocol/types.rs`
- Deleted: `src/protocol/io.rs`
- Added/updated: `src/protocol/message/mod.rs`
- Added/updated: `src/protocol/message/tests.rs`
- Added/updated: `src/protocol/message/variants/ai.rs`
- Added/updated: `src/protocol/message/variants/prompts_media.rs`
- Added/updated: `src/protocol/message/variants/query_ops.rs`
- Added/updated: `src/protocol/message/variants/system_control.rs`
- Added/updated: `src/protocol/message/constructors/final_sections.rs`
- Added/updated: `src/protocol/message/constructors/general.rs`
- Added/updated: `src/protocol/message/constructors/handshake.rs`
- Added/updated: `src/protocol/message/constructors/history_window.rs`
- Added/updated: `src/protocol/message/constructors/prompts.rs`
- Added/updated: `src/protocol/message/constructors/query_ops.rs`
- Added/updated: `src/protocol/types/mod.rs`
- Added/updated: `src/protocol/types/ai.rs`
- Added/updated: `src/protocol/types/chat.rs`
- Added/updated: `src/protocol/types/elements_actions_scriptlets.rs`
- Added/updated: `src/protocol/types/grid_layout.rs`
- Added/updated: `src/protocol/types/input.rs`
- Added/updated: `src/protocol/types/menu_bar.rs`
- Added/updated: `src/protocol/types/primitives.rs`
- Added/updated: `src/protocol/types/system.rs`
- Added/updated: `src/protocol/types/tests/mod.rs`
- Added/updated: `src/protocol/types/tests/exec_submit.rs`
- Added/updated: `src/protocol/types/tests/mouse_choice.rs`
- Added/updated: `src/protocol/io/mod.rs`
- Added/updated: `src/protocol/io/parsing.rs`
- Added/updated: `src/protocol/io/reader.rs`
- Added/updated: `src/protocol/io/tests/mod.rs`
- Added/updated: `src/protocol/io/tests/grid.rs`
- Added/updated: `src/protocol/io/tests/hello.rs`
- Added/updated: `src/protocol/io/tests/parsing.rs`
- Added/updated: `src/protocol/io/tests/reader.rs`

**Verification**
Ran:
- `cargo check && cargo clippy --all-targets -- -D warnings && cargo test` (fails before protocol due unrelated active-agent files)
- `cargo check` (current blocker)
- `cargo check --keep-going` (same blocker)

Current blocker:
- `src/stories/arg_prompt_stories/split.rs:15` unclosed delimiter (outside `src/protocol/*` scope)

After fixing protocol `parsing.rs`, the prior protocol delimiter error is gone.

**How To Test (once branch compiles again)**
- `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`

**Risks / Known Gaps**
- Full mandatory verification gate cannot complete until unrelated syntax errors in other agent scopes are fixed.
- Because gate is blocked externally, no commit/push was made.

**Commits**
- None (blocked by external compile failures).