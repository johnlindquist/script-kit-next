Implemented the type-safety changes for stdin key handling and request IDs, with compile-time typing across main + AI paths.

**Changed Files**
- `src/stdin_commands.rs:50`  
  Added `ExternalCommandRequestId` newtype (`serde(transparent)`) and moved `requestId` fields from `Option<String>` to `Option<ExternalCommandRequestId>`.
- `src/stdin_commands.rs:86`  
  Added `KeyModifier` enum (`Cmd|Shift|Alt|Ctrl`) with serde aliases (`meta/command`, `option`, `control`).
- `src/stdin_commands.rs:390`  
  Changed `ExternalCommand::SimulateKey.modifiers` to `Vec<KeyModifier>`.
- `src/stdin_commands.rs:407`  
  Changed `ExternalCommand::SimulateAiKey.modifiers` to `Vec<KeyModifier>`.
- `src/stdin_commands.rs:796`  
  Added/updated tests for typed modifiers, alias support, and rejection of unknown modifiers.
- `src/main.rs:242`  
  Imported `KeyModifier` and switched modifier parsing to typed checks (`contains(&KeyModifier::...)`) at `src/main.rs:3255`.
- `src/ai/window.rs:399`  
  Changed `AiCommand::SimulateKey.modifiers` to `Vec<KeyModifier>`.
- `src/ai/window.rs:1987`  
  Updated `handle_simulated_key(..., modifiers: &[KeyModifier], ...)` and typed checks.
- `src/ai/window.rs:8212`  
  Updated `simulate_ai_key` signature to `Vec<KeyModifier>`.
- `src/ai/window.rs:1067`  
  Added `message_body_content` helper used by existing unit test and message render path.

**Verification Run**
- `cargo test --lib test_external_command_simulate_key_ -- --nocapture`  
  Passed (4 tests).
- `cargo check`  
  Passed.
- `cargo clippy --all-targets -- -D warnings`  
  Fails due pre-existing repo-wide issues outside this scoped change (example files: `src/actions/dialog.rs`, `src/prompts/div.rs`, `src/actions/window.rs`, several `dialog_builtin_action_validation_tests_*` files).

**How To Test**
1. `cargo test --lib test_external_command_simulate_key_ -- --nocapture`
2. `cargo check`
3. Optional stdin check:
```bash
echo '{"type":"simulateKey","key":"k","modifiers":["meta","option","control"]}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui
```

**Risks / Known Gaps**
- `KeyModifier` parsing is now strict by design; unknown modifier strings are rejected at deserialize time.
- Full `cargo test` was not run in this handoff; verification was targeted + compile gate.
- Clippy is currently red due unrelated existing violations in other areas.

**Commits**
- No commits made in this handoff.