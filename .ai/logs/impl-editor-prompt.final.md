Implemented the highest-priority `editor.rs` fixes from the plan.

**Changed files**
1. `src/render_prompts/editor.rs`
- Added editor-specific footer helpers/config so editor prompts show `⌘↵` submit and helper text with `⌘↵`, `⌘S`, `⌘K` (`src/render_prompts/editor.rs:4`, `src/render_prompts/editor.rs:16`, `src/render_prompts/editor.rs:269`).
- Added reserved editor shortcut guardrails before SDK action-shortcut dispatch (`src/render_prompts/editor.rs:27`, `src/render_prompts/editor.rs:163`).
- Added editor key context and context-rich key logs including correlation id (`src/render_prompts/editor.rs:4`, `src/render_prompts/editor.rs:128`, `src/render_prompts/editor.rs:248`).
- Added in-file tests for footer shortcut hints and reserved shortcut behavior (`src/render_prompts/editor.rs:357`).

**How to test**
1. Verification gate:
```bash
cargo check && cargo clippy --all-targets -- -D warnings && cargo test
```
2. UI smoke run via stdin protocol:
```bash
cargo build
echo "{\"type\":\"run\",\"path\":\"$(pwd)/tests/smoke/test-editor-footer.ts\"}" | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```
3. Inspect screenshot:
- `.test-screenshots/editor-footer-1770440611867.png` (verified: footer shows `⌘↵/⌘S submit · ⌘K actions` and primary `Continue` with `⌘↵`).

**Verification status**
1. `cargo check` passes.
2. `cargo clippy --all-targets -- -D warnings` fails due pre-existing out-of-scope issues (examples: duplicate `mod tests` in `src/render_prompts/arg.rs`, many unused imports in `src/actions/dialog_builtin_action_validation_tests_*`, missing `AiApp::message_body_content` in `src/ai/window.rs`, clippy warning in `src/render_prompts/other.rs`).
3. `cargo test` also fails due those same pre-existing out-of-scope failures.

**Risks / known gaps**
1. Reserved shortcut policy is intentionally conservative but static; additional editor-owned combos may still need product-level confirmation.
2. Could not get a fully green clippy/test run because repository-wide failures outside `src/render_prompts/editor.rs` remain.

Commits made: none.