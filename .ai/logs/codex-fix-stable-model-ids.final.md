**Changed files**
- `src/actions/builders/notes.rs`

**What I changed**
- Replaced index-based new-chat action IDs with stable IDs derived from identifiers:
  - `last_used_{provider}::{model_id}` instead of `last_used_{idx}`
  - `model_{provider}::{model_id}` instead of `model_{idx}`
- Added helper `new_chat_model_identifier()` to centralize ID key construction.
- Added tests in `notes.rs`:
  - `test_get_new_chat_actions_uses_provider_and_model_id_for_action_ids`
  - `test_get_new_chat_actions_model_ids_are_stable_when_order_changes`

**How to test**
1. Run:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
2. (Focused) run:
   - `cargo test --lib test_get_new_chat_actions_`

**Verification run results**
- `cargo check` failed due unrelated pre-existing workspace errors (not in `notes.rs`), e.g.:
  - missing fields in `ClipboardEntry` initializers in `src/clipboard_history/*`
  - non-exhaustive `ContentType` matches in `src/clipboard_history/*`
- `cargo clippy --all-targets -- -D warnings` failed due unrelated pre-existing issues, including:
  - duplicate test module declarations (`src/protocol/message/mod.rs`, `src/designs/core/part_04.rs`)
  - many existing unused-import / test-module issues outside scope
- `cargo test --lib test_get_new_chat_actions_` also failed from unrelated pre-existing workspace test/module issues.

**Risks / known gaps**
- Runtime handler logic in `src/ai/window/command_bar.rs` still parses `last_used_` / `model_` IDs as numeric indices. With stable IDs now emitted by `notes.rs`, that handler must be updated to resolve by identifier rather than index.