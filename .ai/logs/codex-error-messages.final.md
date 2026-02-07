Implemented a focused error-message UX pass and wrote the full audit report.

**Changed files**
- `src/shortcuts/types.rs`
- `src/shortcuts/persistence.rs`
- `src/shortcuts/types_tests.rs`
- `tests/shortcut_error_messages.rs`
- `plans/error-messages-ux.md`

**What changed**
- Improved `ShortcutParseError` copy to be actionable (clear cause + recovery examples).
- Improved `PersistenceError` copy to include operation context and recovery guidance.
- Added tests that validate user-facing message quality:
  - unit coverage in `src/shortcuts/types_tests.rs`
  - unit coverage in `src/shortcuts/persistence.rs`
  - integration coverage in `tests/shortcut_error_messages.rs`
- Added audit report at `plans/error-messages-ux.md` with prioritized findings across `src/**/*.rs`.

**How to test**
1. `cargo check`
2. `cargo test --test shortcut_error_messages`
3. Full gate (currently baseline-blocked by unrelated issues):  
   `cargo clippy --all-targets -- -D warnings && cargo test`

**Verification run results**
- `cargo check`: passed.
- `cargo test --test shortcut_error_messages`: passed (2/2).
- `cargo clippy --all-targets -- -D warnings`: failed due unrelated pre-existing workspace issues (e.g. duplicate `tests` module, many unused imports, missing `AiApp::message_body_content` in `src/ai/window.rs` tests).
- `cargo test`: failed for the same unrelated baseline reasons.

**Risks / known gaps**
- Full workspace quality gate is red from unrelated files, so only targeted verification for this change set is green.
- Report includes additional high-priority UX fixes not implemented in this pass (e.g. generic `Error: ...` fallback copy, repeated `No item selected` ambiguity).

**Commits**
- `9bc8923` `fix(shortcuts): clarify parse and persistence errors`
- `c06d098` `docs(plans): add error-messaging UX audit`