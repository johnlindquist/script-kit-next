# Workflow Affordance Consistency Audit

## Summary
Scanned 6 workflow surfaces. 6 pass, 0 warning, 0 error. Keyboard-first affordances are consistent across the audited surfaces.

## What This Checks
- Keyboard-first consistency across command surfaces: universal three-key footer, mini-vs-expanded parity, explicit exceptions, and reportable runtime audits.

## Surface Status
| Surface | Status | Files |
| --- | --- | --- |
| actions_dialog | pass | `src/actions/dialog.rs` |
| clipboard_history | pass | `src/render_builtins/clipboard.rs`, `src/render_builtins/clipboard_history_layout.rs` |
| file_search | pass | `src/render_builtins/file_search.rs`, `src/render_builtins/file_search_layout.rs` |
| render_prompts::chat | pass | `src/render_prompts/other.rs`, `src/prompts/chat/render_core.rs` |
| render_prompts::term | pass | `src/render_prompts/term.rs` |
| prompts::path | pass | `src/prompts/path/render.rs` |

## Findings
### actions_dialog
- info — **command palette contract is audited**
  - Actions dialog already declares a machine-readable runtime contract for top search, footer hints, and chrome regressions. Treat it as the baseline command surface for every keyboard-first workflow.
  - Evidence: `src/actions/dialog.rs`

### clipboard_history
- info — **expanded clipboard workflow is reportable**
  - Clipboard History already routes through the shared expanded scaffold and emits footer hint audits, so its list-plus-preview workflow is visible to the audit system.
  - Evidence: `src/render_builtins/clipboard.rs`, `src/render_builtins/clipboard_history_layout.rs`

### file_search
- info — **mini and expanded file search are both auditable**
  - File Search already exposes both its compact and split-view workflows in source, emits distinct runtime chrome audits for each presentation, and keeps the mini footer on the canonical three-key hint strip.
  - Evidence: `src/render_builtins/file_search.rs`, `src/render_builtins/file_search_layout.rs`

### render_prompts::chat
- info — **chat teaches the same shortcuts in mini and full modes**
  - Chat already audits both its mini and full footers and carries status text as leading helper content instead of changing the shortcut vocabulary.
  - Evidence: `src/render_prompts/other.rs`, `src/prompts/chat/render_core.rs`

### render_prompts::term
- info — **terminal exception is explicit**
  - Term keeps a contextual footer on purpose, and the exception is already encoded in the chrome audit payload instead of hiding as silent drift.
  - Evidence: `src/render_prompts/term.rs`

### prompts::path
- info — **path prompt is fully auditable**
  - Path prompt now emits both chrome and hint audits while staying on the shared minimal scaffold, so it participates in the same keyboard-first consistency report as the rest of the mini surfaces.
  - Evidence: `src/prompts/path/render.rs`
