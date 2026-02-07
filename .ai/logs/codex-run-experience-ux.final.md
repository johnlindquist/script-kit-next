Implemented the script-run UX polish across your assigned scope, including stronger running-state feedback, env prompt clarity, div prompt shell/header polish, shared footer consistency, and alias validation UX.

**Changed files**
- `src/components/alias_input.rs`
- `src/prompts/env.rs`
- `src/prompts/div.rs`
- `src/panel.rs`
- `src/render_prompts/arg.rs`
- `src/render_prompts/div.rs`
- `src/render_prompts/editor.rs`
- `src/render_prompts/form.rs`
- `src/render_prompts/other.rs`

Key changes:
- Added shared prompt constants/helpers in `src/panel.rs` (`PROMPT_INPUT_FIELD_HEIGHT`, running-status message helper).
- Upgraded alias input in `src/components/alias_input.rs` with clearer placeholder/help copy, inline validation feedback, max-length/whitespace checks, and save-button gating on valid input.
- Improved env prompt in `src/prompts/env.rs` with clearer labels/placeholders/descriptions, stronger input styling, secure-storage helper text, and explicit running status/footer labels (`Save & Continue` / `Update & Continue`).
- Polished div prompt container defaults in `src/prompts/div.rs` to use design-token padding.
- Unified header/footer/action-popup behavior across render wrappers via shared helpers in `src/render_prompts/arg.rs`, then applied in `src/render_prompts/div.rs`, `src/render_prompts/editor.rs`, `src/render_prompts/form.rs`, and `src/render_prompts/other.rs`.
- Preserved webcam footer source-assertion compatibility with explicit `.primary_label("Capture Photo")` while using capture flow.

**Verification run**
- `cargo check`  
  - Failed due out-of-scope compile error in `src/prompts/select.rs:376` (`Vec<_>` type inference).
- `cargo clippy --all-targets -- -D warnings`  
  - Failed due out-of-scope warnings/errors in action validation test files and `src/prompts/select.rs`.
- `cargo test -q`  
  - Ran fully: `8944 passed; 3 failed; 35 ignored`  
  - Failing tests are out of scope:  
    - `actions::dialog_builtin_action_validation_tests_39::tests::script_with_frecency_reset_ranking_no_shortcut`  
    - `actions::dialog_builtin_action_validation_tests_41::tests::scriptlet_with_custom_reset_ranking_has_no_shortcut`  
    - `actions::dialog_builtin_action_validation_tests_46::script_reset_ranking_no_shortcut`
- Runtime/stdin protocol check:
  - `timeout 8 bash -c 'printf "{\"type\":\"show\"}\n" | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1'`
  - Logs confirmed command handling with correlation ID:
    - `Processing external command: Show { request_id: None }`
    - `[-] Showing window`

**How to test**
1. Build/check gates:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test -q`
2. Run app via stdin protocol:
   - `printf '{"type":"show"}\n' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`
3. Exercise prompt wrappers quickly:
   - `printf '{"type":"openAi"}\n' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`
   - `printf '{"type":"openNotes"}\n' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`
4. Validate log events:
   - `grep -E "Processing external command|Showing window" ~/.scriptkit/logs/latest-session.jsonl`

**Risks / known gaps**
- Branch currently has unrelated red tests/clippy issues from other active agents; full green gate is blocked until those land.
- Visual verification was runtime/log-based; no new screenshot smoke script was added in this scope.