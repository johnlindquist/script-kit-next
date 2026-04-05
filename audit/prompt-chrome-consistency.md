# Prompt Chrome Consistency Audit

## Summary
Scanned 7 prompt/builtin surfaces. 7 pass, 0 warning, 0 error. 1 intentional exception documented: render_prompts::term.

## Scope Notes
- Scope: prompt and builtin chrome surfaces only. Excluded this pass: ACP compact-chat popup surfaces (for example src/ai/acp/model_selector_popup.rs).
- Verification precondition: keep only one visible target window per GPUI window kind when using `simulateGpuiEvent`; ambiguous same-kind routing now fails closed.
- Intentional exception: render_prompts::term.

## Surface Status
| Surface | Status | Files |
| --- | --- | --- |
| render_prompts::select | pass | `src/render_prompts/other.rs`, `src/prompts/select/render.rs` |
| render_prompts::arg | pass | `src/render_prompts/arg/render.rs` |
| render_prompts::form | pass | `src/render_prompts/form/render.rs` |
| render_prompts::chat | pass | `src/render_prompts/other.rs`, `src/prompts/chat/render_core.rs` |
| render_prompts::term | pass | `src/render_prompts/term.rs` |
| clipboard_history | pass | `src/render_builtins/clipboard.rs`, `src/render_builtins/clipboard_history_layout.rs` |
| file_search | pass | `src/render_builtins/file_search.rs`, `src/render_builtins/file_search_layout.rs` |

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
- pass — no drift markers detected in the audited source files.

### file_search
- pass — no drift markers detected in the audited source files.
