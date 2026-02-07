Implemented removal of action subtitle/description rendering in the actions popup UI.

**Changed files**
- `src/actions/dialog.rs:52`  
  Added `action_subtitle_for_display(_action: &Action) -> Option<&str>` that returns `None` (centralized “do not render subtitle” policy).
- `src/actions/dialog.rs:1578`  
  Updated action-row rendering to use the helper; title/shortcut/icon still render, subtitle line is now suppressed.
- `src/actions/dialog_tests.rs:260`  
  Added regression test `action_subtitle_is_hidden_even_when_description_exists`.

**Validation run**
- `cargo test action_subtitle_is_hidden_even_when_description_exists` ✅ passed
- `cargo check` ✅ passed
- Stdin JSON protocol smoke run (opened actions popup via `simulateKey` and captured screenshot) ✅
  - Logs show popup opened + screenshot captured (`/tmp/arg-actions-delayed-smoke.log`)
  - Screenshot: `test-screenshots/arg-actions-delayed-1770422137685.png`
  - OCR output confirms only action titles (no subtitle lines)
- `cargo clippy --all-targets -- -D warnings` ❌ fails due pre-existing unrelated lint errors in many existing test files and `src/ai/window.rs:5724`
- `cargo test` ❌ currently blocked by unrelated shared-tree compile error: `src/prompts/chat.rs:3093` (`build_conversation_turns` missing)

**How to test**
1. `cargo test action_subtitle_is_hidden_even_when_description_exists`
2. Run the UI smoke via stdin protocol:
```bash
timeout 40 bash -lc '{ \
echo "{\"type\":\"run\",\"path\":\"'"$(pwd)"'/tests/smoke/test-arg-actions-delayed-capture.ts\"}"; \
sleep 1.0; \
echo "{\"type\":\"simulateKey\",\"key\":\"k\",\"modifiers\":[\"cmd\"]}"; \
sleep 2.8; \
echo "{\"type\":\"simulateKey\",\"key\":\"escape\",\"modifiers\":[]}"; \
sleep 0.2; \
echo "{\"type\":\"simulateKey\",\"key\":\"enter\",\"modifiers\":[]}"; \
} | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 | tee /tmp/arg-actions-delayed-smoke.log'
```
3. Open `test-screenshots/arg-actions-delayed-1770422137685.png` and verify each action row is single-line title + shortcut badges only.

**Risks / known gaps**
- Shared working tree has unrelated in-progress breakages (clippy/test failures listed above), so full repo gate is not green independently of this change.
- Change is applied to the main actions dialog render path; description data remains in structs/builders by design.