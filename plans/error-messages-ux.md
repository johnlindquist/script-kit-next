# Error Messages UX Audit (src/**/*.rs)

## Scope and method
- Audited user-facing error/status/HUD/toast text in `src/**/*.rs`.
- Focused on clarity, context, consistency, and recovery guidance.
- Prioritized issues that block users or hide next steps.

## Changes implemented in this pass

### 1) Shortcut parse errors now provide recovery guidance
- File: `src/shortcuts/types.rs`
- Improved `ShortcutParseError` text from terse/technical to actionable language.
- Examples:
  - Before: `shortcut string is empty`
  - After: `Shortcut is empty. Enter one key, for example 'cmd+k' or 'ctrl+k'.`

### 2) Shortcut persistence errors now include clearer context
- File: `src/shortcuts/persistence.rs`
- `PersistenceError` display strings now clarify operation and user recovery:
  - `Io`: now says shortcut overrides could not be read/written.
  - `Json`: now points to `~/.scriptkit/shortcuts.json` and valid JSON syntax.
  - `InvalidShortcut`: now ties error to specific binding and override value.

### 3) Tests added for user-facing message quality
- Files:
  - `src/shortcuts/types_tests.rs`
  - `src/shortcuts/persistence.rs` (unit test in module)
  - `tests/shortcut_error_messages.rs` (integration test)
- New tests assert message copy includes recovery/context.

## High-priority findings (not yet changed)

### P1: Generic `Error: ...` messaging lacks task context
- `src/main.rs:700`
- `src/app_impl.rs:2683`
- Current UX: shows `Error: <details>` for calculator fallback failures.
- Problem: users do not know what failed or what to do.
- Recommendation: `Could not evaluate expression "{expr}". Check syntax and try again.`

### P1: Repeated `No item selected` is ambiguous
- Many call sites in `src/app_actions.rs` (e.g., `src/app_actions.rs:798`, `src/app_actions.rs:891`, `src/app_actions.rs:954`)
- Current UX: same message across different actions.
- Problem: no action context; user must infer what selection is required.
- Recommendation: action-specific copy, e.g. `Select an item to copy its path.`

### P1: Unhandled protocol message warning is non-actionable
- `src/prompt_handler.rs:653`
- Current UX: `'<type>' is not yet implemented`
- Problem: no recovery guidance.
- Recommendation: add next step: `Update the script to a supported message type or update Script Kit GPUI.`

### P2: Conflict warning in shortcut recorder lacks next step
- `src/components/shortcut_recorder.rs:607`
- Current UX: `Already used by "..."`
- Problem: warns but does not tell user what to do.
- Recommendation: append guidance: `Choose a different shortcut or clear the existing binding first.`

### P2: Messaging tone is inconsistent across HUD/toast surface
- Examples:
  - `Failed to ...` (`src/app_actions.rs` many)
  - `Error: ...` (`src/main.rs:700`, `src/app_impl.rs:2683`)
  - `Could not ...` (new shortcut persistence)
- Problem: inconsistent tone feels fragmented and reduces scanability.
- Recommendation: standardize on one style guide:
  - Summary: sentence case, plain language.
  - Recovery: include one concrete next step when possible.
  - Details: technical error text only in expandable details/logs.

## Positive patterns worth reusing

### Protocol parse issues already include structured detail
- `src/execute_script.rs:317-357`
- `src/prompt_handler.rs:612-644`
- Strengths:
  - message summary + detail body
  - severity mapping
  - copy-details action

### Interpreter-not-found messaging includes platform guidance
- `src/scriptlets.rs:1552-1569`
- Strength: clear installation path + next step.

## Recommended rollout plan

1. Centralize user-facing message helpers by domain
- Example modules: `shortcuts`, `protocol`, `clipboard`, `fallback`.
- Avoid ad-hoc string drift across files.

2. Add action-specific empty-selection helpers
- Replace raw `No item selected` with intent-specific text.

3. Split summary vs technical details consistently
- HUD/toast title: user action and recovery.
- Details pane/logs: OS/stack/system error.

4. Add copy quality tests for critical flows
- Shortcut parse/persistence (done).
- Protocol unhandled message.
- Clipboard/action selection failures.

## Verification performed for this pass
- `cargo test --test shortcut_error_messages`
  - result: pass (2 tests)

## Known validation limits during this pass
- Full `cargo test` for all unit tests is currently blocked by unrelated existing failures in the tree (e.g., `src/ai/window.rs` test compile error unrelated to these changes).
- The required full gate (`cargo check && cargo clippy --all-targets -- -D warnings && cargo test`) should be re-run after cross-agent baseline is green.
