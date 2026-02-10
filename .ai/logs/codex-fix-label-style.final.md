Implemented label-style normalization in `src/actions/builders/script_context.rs`.

**Changed file**
- `src/actions/builders/script_context.rs:9`
- `src/actions/builders/script_context.rs:58`
- `src/actions/builders/script_context.rs:70`
- `src/actions/builders/script_context.rs:80`
- `src/actions/builders/script_context.rs:104`
- `src/actions/builders/script_context.rs:114`
- `src/actions/builders/script_context.rs:149`
- `src/actions/builders/script_context.rs:160`
- `src/actions/builders/script_context.rs:206`
- `src/actions/builders/script_context.rs:252`
- `src/actions/builders/script_context.rs:287`
- `src/actions/builders/script_context.rs:302`
- `src/actions/builders/script_context.rs:351`

**What changed**
- Primary `run_script` label now uses a concise title-cased verb (`title_case_words(...)`) instead of `"<verb> \"<name>\""`.
- Updated labels to consistent verb-first Title Case:
  - `Edit Keyboard Shortcut`
  - `Delete Keyboard Shortcut`
  - `Edit Alias`
  - `Delete Alias`
  - `Show Logs`
  - `Open in Finder`
  - `Copy Deep Link`
  - `Delete Ranking Entry`
- Added/updated unit tests in the same file to validate:
  - verb title-casing for run label
  - label style consistency
  - no trailing `...`
  - label length `< 30`

**Verification run**
- `cargo check -p script-kit-gpui` (failed due unrelated workspace issues, e.g. `clipboard_history` missing fields and non-exhaustive matches).
- `cargo clippy -p script-kit-gpui --all-targets -- -D warnings` (failed due unrelated existing workspace issues, including duplicate test module files and many pre-existing warnings/errors under `-D warnings`).
- `cargo test -p script-kit-gpui test_get_script_context_actions_labels_use_consistent_verb_style --lib` (failed before reaching target test because of unrelated workspace compile errors).

**How to test (once workspace is green)**
1. `cargo check -p script-kit-gpui`
2. `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`
3. `cargo test -p script-kit-gpui test_get_script_context_actions_labels_use_consistent_verb_style --lib`
4. `cargo test -p script-kit-gpui test_get_script_context_actions_run_label_uses_title_case_verb --lib`

**Risks / known gaps**
- Workspace is currently broken by unrelated compile errors, so full verification could not complete to green.
- `run_script` label no longer includes the script name; any tests or UI expectations that depended on that exact title text may need updates.

Commits made: none.