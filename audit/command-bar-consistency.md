# Command Bar Consistency Audit

## Summary
Scanned 4 command bar presets. 4 pass, 0 warning, 0 error. Every audited preset is visible in source, validated by runtime chrome rules, and persisted as markdown.

## What This Checks
- CommandBar preset parity: constructor presence, runtime `emit_command_bar_chrome_audit(...)`, and validated search/section/anchor contract for `main_menu`, `no_search`, `notes`, and `ai`.

## Surface Status
| Surface | Status | Files |
| --- | --- | --- |
| command_bar::main_menu | pass | `src/actions/command_bar.rs` |
| command_bar::no_search | pass | `src/actions/command_bar.rs` |
| command_bar::notes | pass | `src/actions/command_bar.rs` |
| command_bar::ai | pass | `src/actions/command_bar.rs` |

## Findings
### command_bar::main_menu
- info — **command bar preset is reportable**
  - command_bar::main_menu is configured by `main_menu_style` with the audited search/section/anchor contract and emits `emit_command_bar_chrome_audit("main_menu", ...)`, so the preset can be persisted into `./audit/command-bar-consistency.md`.
  - Evidence: `src/actions/command_bar.rs`

### command_bar::no_search
- info — **command bar preset is reportable**
  - command_bar::no_search is configured by `no_search` with the audited search/section/anchor contract and emits `emit_command_bar_chrome_audit("no_search", ...)`, so the preset can be persisted into `./audit/command-bar-consistency.md`.
  - Evidence: `src/actions/command_bar.rs`

### command_bar::notes
- info — **command bar preset is reportable**
  - command_bar::notes is configured by `notes_style` with the audited search/section/anchor contract and emits `emit_command_bar_chrome_audit("notes", ...)`, so the preset can be persisted into `./audit/command-bar-consistency.md`.
  - Evidence: `src/actions/command_bar.rs`

### command_bar::ai
- info — **command bar preset is reportable**
  - command_bar::ai is configured by `ai_style` with the audited search/section/anchor contract and emits `emit_command_bar_chrome_audit("ai", ...)`, so the preset can be persisted into `./audit/command-bar-consistency.md`.
  - Evidence: `src/actions/command_bar.rs`
