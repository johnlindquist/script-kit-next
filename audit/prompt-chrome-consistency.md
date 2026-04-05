# Prompt Chrome Consistency Audit

## Summary
Scanned 8 prompt/builtin surfaces. 8 pass, 0 warning, 0 error. 2 intentional exceptions documented: render_prompts::term, file_search.

## Scope Notes
- Scope: prompt and builtin chrome surfaces only. Excluded this pass: ACP compact-chat popup surfaces (for example src/ai/acp/model_selector_popup.rs).
- Verification precondition: keep only one visible target window per GPUI window kind when using `simulateGpuiEvent`; ambiguous same-kind routing now fails closed.
- Intentional exceptions: render_prompts::term, file_search.

## Surface Status
| Surface | Status | Files |
| --- | --- | --- |
| render_prompts::select | pass | `src/render_prompts/other.rs`, `src/prompts/select/render.rs` |
| render_prompts::arg | pass | `src/render_prompts/arg/render.rs` |
| render_prompts::form | pass | `src/render_prompts/form/render.rs` |
| render_prompts::chat | pass | `src/render_prompts/other.rs`, `src/prompts/chat/render_core.rs` |
| render_prompts::term | pass | `src/render_prompts/term.rs` |
| prompts::path | pass | `src/prompts/path/render.rs` |
| clipboard_history | pass | `src/render_builtins/clipboard.rs`, `src/render_builtins/clipboard_history_layout.rs` |
| file_search | pass | `src/render_builtins/file_search.rs` |

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

### prompts::path
- pass — no drift markers detected in the audited source files.

### clipboard_history
- pass — no drift markers detected in the audited source files.

### file_search
- info — **contextual primary label follows three-key pattern**
  - File Search uses `↵ Open` / `↵ Browse` as the primary action label instead of `↵ Run`, paired with canonical `⌘K Actions` and `Tab AI`. This is an accepted contextual variant of the three-key footer pattern.
  - Evidence: `src/render_builtins/file_search.rs`
- info — **intentional loading and empty states**
  - File Search keeps a crafted loading state with skeleton rows plus helpful empty-state copy for both the blank-query and no-results cases.
  - Evidence: `src/render_builtins/file_search.rs`
