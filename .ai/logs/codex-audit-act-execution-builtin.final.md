Added the audit report and traced the full built-in execution path from selection to handler dispatch.

Changed files:
- `.ai/logs/codex-audit-act-execution-builtin.final.md`
- `.ai/logs/codex-audit-act-execution-scripts.final.md` (was already staged in the tree and got included in the commit)

How to test:
1. `cargo test test_get_builtin_entries_hides_preview_ai_commands -- --nocapture`
2. `cargo test builtin_confirmation_modal_failure_does_not_auto_confirm -- --nocapture`

Verification run:
- `cargo test test_get_builtin_entries_hides_preview_ai_commands -- --nocapture` passed.
- `cargo test builtin_confirmation_modal_failure_does_not_auto_confirm -- --nocapture` ran with 0 matched tests.
- A stricter path-specific run (`cargo test app_actions::tests::clipboard_actions::builtin_confirmation_modal_failure_does_not_auto_confirm -- --nocapture`) failed due unrelated concurrent clipboard-history compile breakages outside this task’s scope.

Risks / known gaps (from the audit):
- `builtin-open-ai` / `builtin-open-notes` are missing from `NO_MAIN_WINDOW_BUILTINS` in `src/app_impl/execution_scripts.rs:317`, so hotkey/deeplink command-ID execution can re-show the main window.
- Confirmation modal open failure in `src/app_execute/builtin_execution.rs:87` is log-only (no user-facing toast/HUD).
- Several AI command failure paths log `Failed to open AI` without user feedback (`src/app_execute/builtin_execution.rs:634`, `src/app_execute/builtin_execution.rs:677`, `src/app_execute/builtin_execution.rs:709`, `src/app_execute/builtin_execution.rs:752`, `src/app_execute/builtin_execution.rs:796`).
- Orphaned built-in variants remain defined/handled but not constructed by entry builders.

Commits made:
- `e42cc78`