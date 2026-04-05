# Prompt Chrome Consistency Audit

## Summary
Scanned 7 prompt/builtin surfaces. 5 pass, 2 warning, 0 error. Highest-leverage current drifts: clipboard_history, file_search.

## Surface Status
| Surface | Status | Files |
| --- | --- | --- |
| render_prompts::select | pass | `src/render_prompts/other.rs`, `src/prompts/select/render.rs` |
| render_prompts::arg | pass | `src/render_prompts/arg/render.rs` |
| render_prompts::form | pass | `src/render_prompts/form/render.rs` |
| render_prompts::chat | pass | `src/render_prompts/other.rs`, `src/prompts/chat/render_core.rs` |
| render_prompts::term | pass | `src/render_prompts/term.rs` |
| clipboard_history | warning | `src/render_builtins/clipboard_history_layout.rs` |
| file_search | warning | `src/render_builtins/file_search.rs`, `src/render_builtins/file_search_layout.rs` |

## Findings
### render_prompts::select
- pass — no drift markers detected in the audited source files.

### render_prompts::arg
- pass — no drift markers detected in the audited source files.

### render_prompts::form
- pass — no drift markers detected in the audited source files.

### render_prompts::chat
- pass — no drift markers detected in the audited source files.

### render_prompts::term
- info — **contextual footer exception**
  - Term intentionally owns a contextual footer. Keep it documented as an exception in the report instead of forcing universal hints onto the terminal surface.
  - Evidence: `src/render_prompts/term.rs`

### clipboard_history
- warning — **missing runtime chrome audit**
  - clipboard_history does not currently declare a runtime `emit_prompt_chrome_audit(...)` for its surface name.
  - Evidence: `src/render_builtins/clipboard_history_layout.rs`
- info — **manual expanded layout**
  - Clipboard History still hand-builds the 50/50 split layout instead of routing through `render_expanded_view_scaffold`, which makes future chrome drift easier to reintroduce.
  - Evidence: `src/render_builtins/clipboard_history_layout.rs`

### file_search
- warning — **missing prompt hint audit**
  - File Search does not emit `emit_prompt_hint_audit("file_search", ...)`, so its mini-mode footer drift bypasses the shared hint-contract warning path.
  - Evidence: `src/render_builtins/file_search_layout.rs`
- warning — **non-universal footer hints**
  - File Search mini mode still advertises `↵ Open`, `⌘↵ Ask AI`, and `⇥ Navigate` instead of the canonical `↵ Run`, `⌘K Actions`, `Tab AI` trio.
  - Evidence: `src/render_builtins/file_search_layout.rs`
