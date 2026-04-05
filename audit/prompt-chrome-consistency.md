# Prompt Chrome Consistency Audit

## Summary
Scanned 7 prompt/builtin surfaces. 7 pass, 0 warning, 0 error. No current drift markers were detected.

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

## Verification

Use `scripts/agentic/session.sh` to verify runtime chrome emissions:

```bash
bash scripts/agentic/session.sh start default
bash scripts/agentic/session.sh send default '{"type":"show"}'
bash scripts/agentic/session.sh send default '{"type":"triggerBuiltin","name":"clipboard"}'
sleep 1
bash scripts/agentic/session.sh send default '{"type":"triggerBuiltin","name":"file-search"}'
sleep 1
bash scripts/agentic/session.sh status default
bash scripts/agentic/session.sh stop default
```

Expected runtime log output:
- `clipboard_history_chrome_checkpoint`
- `prompt_hint_audit` with `surface=clipboard_history hint_count=3 is_universal=true`
- `file_search_chrome_checkpoint`
- `prompt_hint_audit` with `surface=file_search hint_count=3 is_universal=true` (when mini mode is exercised)
- `file_search_state_rendered` with `state=loading_skeleton`, `state=results`, or `state=empty_no_results`
