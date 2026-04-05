# Prompt Chrome Consistency Audit

## Summary
Scanned 7 prompt/builtin surfaces. 7 pass, 0 warning, 0 error. The only non-standard footer remains the documented terminal exception, and it is explicitly declared rather than drifting silently.

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
- info — contextual footer exception remains explicit.
  - Term intentionally owns a contextual footer. Keep it documented as an exception instead of forcing the universal footer onto the terminal surface.
  - Evidence: `src/render_prompts/term.rs`

### clipboard_history
- pass — layout now routes through the shared expanded-view scaffold instead of hand-building split chrome.
- pass — footer hints emit `emit_prompt_hint_audit("clipboard_history", ...)` and resolve to the canonical three-key contract.
- info — list-pane vertical padding remains content-local inside the scaffold, not as outer layout chrome.
- Evidence: `src/render_builtins/clipboard.rs`, `src/render_builtins/clipboard_history_layout.rs`

### file_search
- pass — expanded presentation still routes through `render_expanded_view_scaffold(...)`.
- pass — mini presentation now emits `emit_prompt_hint_audit("file_search", ...)` and uses the canonical `↵ Run`, `⌘K Actions`, `Tab AI` footer.
- info — runtime `file_search_chrome_checkpoint` continues to expose `layout_mode=mini|expanded` for inspection without changing footer semantics.
- Evidence: `src/render_builtins/file_search.rs`, `src/render_builtins/file_search_layout.rs`

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
