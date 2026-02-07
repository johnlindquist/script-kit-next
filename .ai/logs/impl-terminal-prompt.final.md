Implemented highest-priority terminal prompt fixes in `src/render_prompts/term.rs`.

Changed behavior:
1. Added explicit wrapper ESC policy for SDK terminal prompts so non-dismissable ESC is swallowed at the app layer, while Quick Terminal ESC behavior is preserved (`src/render_prompts/term.rs:22`, `src/render_prompts/term.rs:148`, `src/render_prompts/term.rs:165`).
2. Added Cmd+K action mode routing:
   - SDK actions when present
   - terminal command fallback when SDK actions are absent (`src/render_prompts/term.rs:6`, `src/render_prompts/term.rs:64`, `src/render_prompts/term.rs:190`, `src/render_prompts/term.rs:302`).
3. Added terminal action-id fallback execution for actions dialog selections (`src/render_prompts/term.rs:31`, `src/render_prompts/term.rs:77`, `src/render_prompts/term.rs:213`).
4. Switched wrapper key handling to `.key_context("term_prompt")` + `.capture_key_down(...)` so policy is enforced before terminal internals (`src/render_prompts/term.rs:285`).
5. Added focused unit tests for action-mode selection, ESC policy, and action-id mapping (`src/render_prompts/term.rs:353`).

Changed files:
1. `src/render_prompts/term.rs`

How to test:
1. Full gate (requested):
   - `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
2. Terminal prompt stdin smoke:
   - `timeout 15 bash -lc 'printf "%s\n" "{\"type\":\"run\",\"path\":\"$PWD/tests/smoke/test-term-footer.ts\"}" | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1'`
3. Verify logs include:
   - `Received: {"type":"run",...test-term-footer.ts}`
   - `Prompt message received: ShowTerm {...}`
   - `Resize: 500 -> 700`
4. Visual verify screenshot:
   - `test-screenshots/term-footer-1770441076708.png` (opened and checked; footer shows `Close` + `Actions`).

Verification results in this workspace:
1. `cargo check` passes.
2. `cargo clippy --all-targets -- -D warnings` fails outside scope due rustc/clippy stack overflow while compiling lib tests.
3. `cargo test` fails outside scope with rustc SIGBUS/SIGBUS backtrace in macro parsing during lib test compilation.
4. Stdin JSON terminal smoke test passes and logs confirm term prompt flow.

Risks / known gaps:
1. Full repo verification is currently blocked by unrelated workspace-wide test/clippy compiler failures.
2. Terminal fallback action mapping is string-id based; if terminal command IDs change elsewhere, mapping in `src/render_prompts/term.rs` must be updated.

Commits made:
1. None (did not commit because full verification gate is not green in current workspace).