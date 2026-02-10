**Findings (Markdown Audit)**
1. High: `href` values were not sanitized before click handling, so unsafe/malformed protocols were still rendered as interactive links and passed to click handlers.
- Affected: `src/prompts/div/prompt.rs:118`, `src/prompts/div/inline.rs:149`
- Fix: Added centralized `sanitize_div_href` validation/normalization (protocol allowlist, entity decoding, control-char/whitespace rejection where relevant) in `src/prompts/div/types.rs:141`, then enforced it in both render-time interactivity and click-time execution.

2. Medium: malformed HTML that parsed to zero elements produced a blank output state with no fallback.
- Affected: `src/prompts/div/render.rs:33`
- Fix: Added `should_show_raw_html_fallback` (`src/prompts/div/types.rs:172`) and raw-text fallback rendering in `src/prompts/div/render.rs:110` with explicit logging.

3. Medium: `<span>` nodes were rendered as block/column containers, causing layout drift in edge cases.
- Affected: `src/prompts/div/render_html.rs:224`
- Fix: Changed span rendering to inline segmentation (`render_inline_content`) instead of nested block-column rendering.

4. Review note: `src/render_prompts/div.rs` is mostly prompt shell wiring and did not expose additional XSS/sanitization issues in this pass.

**Changed Files**
- `src/prompts/div/types.rs`
- `src/prompts/div/prompt.rs`
- `src/prompts/div/inline.rs`
- `src/prompts/div/render.rs`
- `src/prompts/div/render_html.rs`
- `src/prompts/div/tests.rs`

**Tests / Linting Run**
- `cargo test --lib prompts::div::tests::`
- Result: pass (`13 passed`, `0 failed`)

**How To Test**
1. Run `cargo test --lib prompts::div::tests::`
2. Manually exercise a div prompt with links:
- valid: `submit:ok`, `https://example.com`, `file:///tmp/test.txt`
- blocked/non-clickable: `javascript:alert(1)`, `data:text/html,hi`, malformed/empty submit href
3. Render malformed HTML like `<div` and verify raw fallback text appears instead of blank content.

**Risks / Known Gaps**
- Parser remains best-effort and non-structured for partial malformed trees; fallback triggers only when parse result is fully empty.
- `file://` links are still allowed by design (click-gated), so untrusted content can still prompt local-file opens if clicked.

**Commits**
- No commits were created in this run.