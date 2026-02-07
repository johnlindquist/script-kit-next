Implemented the list-item declutter pass for search results and committed it.

**Commit**
1. `7850f25ffe77a0b797638394bb52ad443a96bc21` `fix(list-item): declutter search result row accessories`

**Changed files**
- `src/list_item.rs`
- `src/designs/mod.rs`
- `src/list_item_tests.rs`

**What changed**
- Type badges in search are now subtle text labels (no loud pill styling).
- Search-mode source/category/action metadata is hidden (category text + action verb noise removed in search rows).
- Search shortcut/trigger text is quieter and only shown on selected/hovered rows during search.
- Match highlighting is toned down to subtle color tinting (removed aggressive bold emphasis).
- Description line is dimmer and only shown in search when focused (selected/hovered) or when description itself matched.
- Added tests for new search visibility logic and for accessory/badge behavior in search mode.

**Verification run**
- Passed: `cargo check --lib`
- Passed: `cargo clippy --lib -- -D warnings`
- Passed runtime protocol check: stdin `show` + `setFilter` with logs confirming `SetFilter` and `[RENDER_SCRIPT_LIST_START] filter='clip'`
- Full gate attempted but blocked by unrelated parallel changes:
  - `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
  - Fails in unrelated files (`src/render_prompts/arg.rs`, `src/ai/window.rs`, multiple `src/actions/dialog_builtin_action_validation_tests_*`)

**How to test**
1. Build:
   `cargo build`
2. Run with stdin protocol:
   ```bash
   cat <<'JSON' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
   {"type":"show"}
   {"type":"setFilter","text":"clip"}
   JSON
   ```
3. In filtered list rows, confirm:
   - No loud type/trigger/action/category pills.
   - Only subtle type text remains.
   - Shortcut text is subdued and appears only on selected/hovered rows.
   - Highlighting is quiet (no bold/yellow scatter look).
   - Description remains secondary/dim.

**Risks / known gaps**
- I could not complete full `cargo test` gate because unrelated in-progress agent changes currently break compile/test targets.
- External `captureWindow` command did not find the GPUI window title in this environment, so automated `captureWindow` visual snapshot via stdin command is currently unreliable.