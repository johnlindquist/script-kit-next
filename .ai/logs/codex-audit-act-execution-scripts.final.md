Completed the script-context execution audit and wrote findings to:

- `.ai/logs/codex-audit-act-execution-scripts.final.md`

What I verified:
1. Every action ID from `get_script_context_actions()` has a handler in `src/app_actions/handle_action.rs`.
2. Missing script/permission handling is mixed: `run_script`, `copy_*`, and `copy_content` have user-visible failures, but editor/reveal flows are inconsistent.
3. Completion feedback is mostly present, with two gaps documented below.

Key risks/known gaps found:
1. Reveal actions can show success HUD before async reveal actually succeeds.
2. `edit_script` / `edit_scriptlet` launch failures are log-only (no user-facing error).

Verification run:
- `sed -n '1,260p' .ai/logs/codex-audit-act-execution-scripts.final.md`
- Manual call-chain cross-check against:
  - `src/actions/builders/script_context.rs`
  - `src/actions/dialog/part_02/part_02.rs`
  - `src/actions/dialog/part_02/part_03.rs`
  - `src/app_impl/actions_dialog.rs`
  - `src/app_impl/startup.rs`
  - `src/app_actions/handle_action.rs`
  - `src/app_impl/selection_fallback.rs`
  - `src/execute_script/part_001_body/execute_interactive_merged.rs`

Commits:
- `e42cc78` `docs(audit): record builtin execution findings` (this commit includes `.ai/logs/codex-audit-act-execution-scripts.final.md`)