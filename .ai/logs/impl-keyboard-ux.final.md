Implemented the keyboard UX plan items for alias consistency, focus handling, and missing navigation shortcuts.

**Changed files**
- `src/confirm/window.rs`
- `src/prompts/div.rs`
- `src/prompts/env.rs`
- `src/actions/command_bar.rs`
- `src/actions/window.rs`
- `src/app_impl.rs`
- `src/keyboard_routing_tests.rs`

**What changed**
- Normalized key matching to handle aliases consistently (`enter`/`return`, `escape`/`esc`, arrows).
- Added intent/helper-based key routing in actions UIs.
- Added missing shortcuts in actions UIs: `home`, `end`, `pageup`, `pagedown`.
- Improved main-input focus close path to use shared actions-popup close logic.
- Added/updated regression tests for keyboard routing and alias coverage.

**Verification run**
1. `cargo check` ✅ pass.
2. UI stdin protocol sanity check ✅ pass:  
   `printf '{"type":"show"}\n' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui`  
   Logs show parsed and processed `show` command.
3. `cargo test --lib` ❌ blocked by pre-existing unrelated test compile error:  
   `src/ai/window.rs:8729` (`AiApp::message_body_content` not found).

**How to test**
1. `cargo check`
2. `cargo build`
3. `timeout 8 bash -lc 'printf "%s\n" '\''{"type":"show"}'\'' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1'`
4. (When unrelated test breakage is fixed) `cargo test --lib`

**Risks / known gaps**
- Full lib tests currently fail due unrelated pre-existing test code in `src/ai/window.rs`, so new keyboard tests could not be executed end-to-end yet.
- `src/app_impl.rs` is actively edited by another parallel agent, so merge/cherry-pick conflict risk is high in that file.

**Commits**
- None.