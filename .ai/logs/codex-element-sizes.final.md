Implemented a size-system normalization pass for my scope and released all swarm claims.

**Changed files**
- `src/components/unified_list_item/types.rs`
- `src/components/unified_list_item_tests.rs`
- `src/components/button.rs`
- `src/components/alias_input.rs`
- `src/panel.rs`
- `src/prompts/select.rs`
- `src/prompts/template.rs`
- `src/prompts/env.rs`
- `tests/footer_button.rs`

**What changed**
- Unified list item heights to canonical tokens:
  - `Density::Comfortable` and `Density::Compact` now both use `crate::list_item::LIST_ITEM_HEIGHT`.
  - Unified section header height now uses `crate::list_item::SECTION_HEADER_HEIGHT`.
  - Added/updated tests to enforce those invariants.
- Introduced shared ghost-button size tokens in `Button`:
  - `BUTTON_GHOST_HEIGHT = 28.0`
  - `BUTTON_GHOST_PADDING_X = 8.0`
  - `BUTTON_GHOST_PADDING_Y = 4.0`
  - Wired render path to use these tokens.
- Added canonical prompt input height token:
  - `PROMPT_INPUT_FIELD_HEIGHT = 44.0` in `src/panel.rs`.
  - Applied it in alias/select/template/env prompt input containers.
- Standardized select list row height with `LIST_ITEM_HEIGHT`.
- Added integration tests for ghost button size tokens in `tests/footer_button.rs`.

**Validation run**
- `cargo check` ✅
- `cargo clippy --all-targets -- -D warnings` ❌ (fails from unrelated existing warnings/errors in action validation test files and a clippy lint in `src/ai/window.rs`)
- `cargo test unified_list_item_tests -- --nocapture` ✅
- `cargo test --test footer_button -- --nocapture` ✅
- `cargo test` ✅
- Runtime stdin-protocol smoke:
  - `{"type":"show"}` via stdin with `SCRIPT_KIT_AI_LOG=1` processed successfully (log shows `Processing external command: Show`).

**How to test**
1. `cargo check`
2. `cargo test unified_list_item_tests -- --nocapture`
3. `cargo test --test footer_button -- --nocapture`
4. `cargo test`
5. Runtime smoke:
   ```bash
   timeout 8 bash -lc 'cat <<'\''EOF'\'' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
   {"type":"show"}
   EOF'
   ```

**Risks / known gaps**
- `src/components/footer_button.rs` still uses local sizing (`.px(8)`, `.py(2)`) and is not yet token-aligned with `Button` ghost padding/height; this was cross-agent-owned during this pass.
- `cargo clippy --all-targets -- -D warnings` is currently red due unrelated repo-wide issues outside this task’s edited files.