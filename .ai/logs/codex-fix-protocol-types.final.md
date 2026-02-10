Implemented protocol-scope fixes and removed protocol compile errors.

**Changed files**
- `src/protocol/types/ai.rs`
- `src/protocol/types/chat.rs`
- `src/protocol/types/elements_actions_scriptlets.rs`
- `src/protocol/types/grid_layout.rs`
- `src/protocol/types/input.rs`
- `src/protocol/types/menu_bar.rs`
- `src/protocol/types/primitives.rs`
- `src/protocol/types/system.rs`
- `src/protocol/message/constructors/final_sections.rs`

**What I changed**
- Added missing serde derive imports in split type files: `use serde::{Deserialize, Serialize};`
- Restored missing cross-type/helper imports introduced by split:
  - `ProtocolAction` in `chat.rs`
  - `ClipboardEntryType` in `system.rs`
  - `generate_semantic_id` / `generate_semantic_id_named` in `elements_actions_scriptlets.rs` and `primitives.rs`
- Fixed stale path in constructor:
  - `Vec<super::types::MenuBarItemData>` -> `Vec<MenuBarItemData>` in `final_sections.rs`

**How to test**
1. Protocol-focused error check:
   - `cargo check --message-format short 2>&1 | rg "src/protocol"`
   - Expected: no `error:` lines (only an existing warning about deprecated/unused `MouseEventData`).
2. Assignment-specified command:
   - `cargo check 2>&1 | grep src/protocol`
   - Expected: no protocol error lines.

**Verification run**
- `cargo check --message-format short 2>&1 | rg "src/protocol"`: protocol errors resolved.
- Full gate attempts:
  - `cargo check` failed due many non-protocol compile errors in other areas (`src/actions`, `src/components`, `src/prompts`, etc.).
  - `cargo clippy --all-targets -- -D warnings` failed for same reason.
  - `cargo test` failed for same reason.

**Risks / known gaps**
- Full repository is currently not buildable due unrelated parallel refactor breakages outside `src/protocol`.
- One protocol warning remains in `src/protocol/types/mod.rs` for deprecated/unused `MouseEventData` re-export (not a compile error).

**Commits**
- None.