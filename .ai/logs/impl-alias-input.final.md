Implemented the highest-priority `alias_input` fixes in `src/components/alias_input.rs`.

**Changed files**
- `src/components/alias_input.rs`

**What changed**
- Aligned modal validation with save-path charset rules by adding `InvalidCharacters` and rejecting anything outside `[A-Za-z0-9_-]` in `validate_alias_input` (`src/components/alias_input.rs:109`, `src/components/alias_input.rs:117`).
- Kept Save disabled when validation fails and surfaced inline error text for invalid charset (`src/components/alias_input.rs:379`, `src/components/alias_input.rs:409`, `src/components/alias_input.rs:492`).
- Added platform-safe command modifier handling (`platform || control`) and used it for text input key handling (`src/components/alias_input.rs:143`, `src/components/alias_input.rs:512`, `src/components/alias_input.rs:544`).
- Added keyboard clear shortcut for existing aliases (`Cmd/Ctrl+Backspace` or `Cmd/Ctrl+Delete`) (`src/components/alias_input.rs:147`, `src/components/alias_input.rs:537`).
- Reduced per-key log noise by switching key logging to `log_debug` (`src/components/alias_input.rs:514`).
- Added tests for new behavior:
  - invalid chars rejected
  - command modifier mapping
  - clear shortcut conditions
  (`src/components/alias_input.rs:661`, `src/components/alias_input.rs:673`, `src/components/alias_input.rs:680`).

**How to test**
1. Run the required gate:
   - `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
2. In-app behavior check:
   - Open alias modal, enter `clip!` or `clip.history` and verify Save is disabled + inline charset error.
   - Edit an existing alias and press `Cmd+Backspace` (macOS) / `Ctrl+Backspace` (non-mac) and verify it triggers clear action.
   - Verify normal cmd/ctrl text shortcuts still work in the field (`a/c/v/x`).

**Verification results**
- `cargo check` passed.
- Full gate currently fails due unrelated pre-existing issues outside scope:
  - `src/render_prompts/other.rs:313` (`clippy::needless_return`)
  - test/clippy stack overflow/recursion-limit failures in other modules (e.g. `src/components/button.rs` during test expansion).

**Risks / known gaps**
- Conflict-aware alias validation (checking against existing registry) is still not implemented in this file.
- True validator unification with `save_alias_with_text` is partial because that function lives outside this scope.

**Commits**
- No commits were made.