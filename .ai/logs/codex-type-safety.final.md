Implemented a targeted type-safety upgrade and wrote the requested report.

Changed files:
- `src/prompts/commands.rs`
- `plans/type-safety-improvements.md`

What changed:
- In `src/prompts/commands.rs`:
  - Added typed keyword APIs on `SlashCommandType` (`all`, `from_keyword`, `keywords`, `matches_keyword_prefix`).
  - Refactored `CommandOption` to derive label/description/icon from `kind` instead of storing duplicated free-form strings.
  - Updated parsing to normalize alias raw commands to canonical forms (`/tests` -> `/test`, `/summary` -> `/summarize`).
  - Added tests:
    - `test_filter_by_prefix_matches_aliases`
    - `test_parse_command_aliases_use_canonical_raw_name`
- In `plans/type-safety-improvements.md`:
  - Added prioritized type-safety report across `src/**/*.rs` with concrete follow-ups and migration guidance.

How to test:
1. `cargo check --lib --no-default-features` (passes locally)
2. `cargo check`
3. `cargo clippy --all-targets -- -D warnings`
4. `cargo test`

Verification results / known gaps:
- `cargo check`, `cargo clippy`, and `cargo test` fail due pre-existing repository issues unrelated to this change, including:
  - `src/icons/types/icon_ref.rs:47` (`E0521`)
  - `src/ai/window.rs:8549` missing `AiApp::message_body_content` (`E0599`)
  - existing clippy failures in multiple pre-existing test files (unused imports, duplicate test module names)
- Because of those pre-existing failures, full gate verification could not be completed for this branch.

Risks:
- Alias canonicalization changes `SlashCommand.raw` for aliases; if any future code relies on alias-specific raw strings, behavior will differ (current codebase usage appears internal to this module tests).

Commits made:
- None.