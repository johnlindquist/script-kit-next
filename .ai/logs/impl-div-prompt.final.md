**Changed Files**
- `src/prompts/div.rs`
  - Implemented inline segment rendering to preserve nested inline formatting (`bold/italic/link/code`) instead of flattening text.
  - Updated block render paths (headers, paragraphs, lists, blockquote, inline wrappers) to use inline-preserving rendering.
  - Added optional language label rendering for code blocks.
  - Consolidated link handling through `handle_link_click(...)`.
  - Tightened Enter/Escape handling in `on_key_down` (ignore modified keys, stop propagation after submit).
  - Removed unused scroll offset state/method.
  - Added tests for nested inline style and nested link preservation.
- `src/render_prompts/div.rs`
  - Added `cx.stop_propagation()`/early return when parent/global handlers consume keys (global shortcut, Cmd+K, actions dialog handled/execute, SDK action shortcut).

**How To Test**
1. Run:
```bash
cargo check
```
2. Run focused tests:
```bash
cargo test test_collect_inline_segments -- --nocapture
```
3. For runtime behavior, launch with stdin protocol and exercise a div prompt containing nested inline markdown/html plus Cmd+K/actions shortcuts:
```bash
echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```

**Risks / Known Gaps**
- Focused test execution is currently blocked by an unrelated existing compile error in `src/ai/window.rs` (`AiApp::message_body_content` missing), so new tests were added but not fully runnable in this workspace state.
- Key propagation changes are intentional but could affect any flow that previously relied on bubbling after a handled parent shortcut.

**Commits**
- No commits were made.