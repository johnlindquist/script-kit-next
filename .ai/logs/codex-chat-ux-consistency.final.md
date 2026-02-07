Implemented the chat UX consistency pass in the claimed scope and released all swarm claims.

**Changed files**
1. `src/prompts/chat.rs:182`
2. `src/components/prompt_footer.rs:63`

**What changed**
1. `src/prompts/chat.rs`
2. Added explicit key-action routing (`Enter` send, `Shift+Enter` newline, `Escape` close/blur path) with tests.
3. Added one-shot auto-focus on open (`pending_auto_focus`).
4. Normalized multiline paste behavior for chat input.
5. Polished input/footer UX details (focus-aware styling, spacing, helper hint `Shift+Enter newline`), using theme tokens.
6. Fixed Rust compile issue in clipboard paste path (removed let-chain pattern that broke on current edition/toolchain).

1. `src/components/prompt_footer.rs`
2. Moved footer visuals to theme token backgrounds and theme-derived UI font sizing for consistency/contrast.

**Validation run**
1. `cargo test resolve_chat_input_key_action_routes_enter_variants --lib` passed.
2. `cargo test resolve_chat_input_key_action_routes_shortcuts_and_fallback --lib` passed.
3. `cargo check` passed.
4. Stdin JSON smoke flow verified:
   - `{"type":"run","path":".../tests/smoke/test-chat-input.ts"}`
   - Logs showed `ShowChat`, `Applying pending focus: ChatPrompt`, and successful screenshot capture.
5. Screenshot was captured and read: `test-screenshots/chat-input-1770423758435.png`.

**How to test**
1. `cargo check`
2. `cargo test resolve_chat_input_key_action_routes_enter_variants --lib`
3. `cargo test resolve_chat_input_key_action_routes_shortcuts_and_fallback --lib`
4. `printf '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-chat-input.ts"}\n' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`
5. Optional log check: `rg -i "ShowChat|Applying pending focus|CaptureScreenshot" ~/.scriptkit/logs/latest-session.jsonl`

**Risks / known gaps**
1. Repo-wide `cargo clippy --all-targets -- -D warnings` and full `cargo test` are currently red due unrelated pre-existing failures in `actions::*` validation test modules.
2. `src/prompts/chat.rs` had parallel-agent activity; functional overlap risk exists until final integration pass is run on the merged tree.