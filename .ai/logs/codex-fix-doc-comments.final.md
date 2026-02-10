Changed doc-comment style in the scoped test files so they no longer use invalid inner doc comments in include/module contexts:

- `src/window_state/tests/window_state.rs:1`
- `src/window_state/tests/persistence.rs:1`
- `src/app_actions/tests/actions_button_visibility.rs:1`
- `src/app_actions/tests/clipboard_actions.rs:1`
- `src/app_actions/tests/clipboard_actions_menu_coverage.rs:1`

What I changed:
- Converted top-of-file `//!` blocks to regular `//` comments in those files.
- This removes both `E0753 expected outer doc comment` and `clippy::empty_line_after_doc_comments` for those specific files.

How to test:
1. Run clippy:
   - `cargo clippy --all-targets -- -D warnings`
2. Optional quick scope checks:
   - `rg -n "^//!" src/window_state/tests src/app_actions/tests`
   - `rg -nUP "(?m)^(///|//!).*\\n\\s*\\n\\s*(#\\[cfg\\(test\\)\\]|mod\\s+tests|use\\s)" src/window_state src/app_actions src/executor/tests src/executor/tests.rs`

Risks / known gaps:
- I could not complete a clean full clippy verification in this shared run because:
  - concurrent agents held cargo build locks in the shared `target/`,
  - and isolated-target clippy failed due low disk (`No space left on device`).
- Static checks for the edited files passed, but full workspace clippy status remains blocked by environment contention.