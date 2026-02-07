Completed the assignment by writing the analysis report to:

- `plans/actions-dialog-system.md`

**What I changed**
- Added `plans/actions-dialog-system.md` with prioritized findings and concrete recommendations across:
  - action routing
  - dialog state management
  - validation/test architecture
  - builder-pattern edge cases
  - missing/underused action types and simplification opportunities

**How to test / verify**
1. `cargo check`  
   - Result: passed.
2. Targeted dialog/action tests (attempted):  
   - `cargo test dialog_behavior_tests::no_duplicate_ids_in_script_context && cargo test dialog_cross_context_tests::agent_with_is_script_true_gets_script_actions_instead && cargo test dialog_validation_tests::test_script_info_agent_with_is_script_true_gets_script_actions && cargo test dialog_tests::action_config_default_values`
   - Result: blocked by existing unrelated test-compile error in `src/ai/window.rs:8549` (`AiApp::message_body_content` not found).

**Risks / known gaps**
- No production code fixes were applied in this task; this deliverable is analysis-only as requested.
- Full test verification is currently blocked by pre-existing compile issues outside actions/dialog scope (`src/ai/window.rs`).
- The working tree contains many parallel-agent modifications unrelated to this report.

**Commits**
- No commits were made.