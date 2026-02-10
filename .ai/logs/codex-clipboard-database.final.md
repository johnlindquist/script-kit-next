Changed file:
- `src/clipboard_history/database.rs`

What I changed:
- Updated `ContentType` matches to handle all 5 variants and treat `Link/File/Color` like `Text` (100-char text preview, no image dimensions):
  - `src/clipboard_history/database.rs:222`
  - `src/clipboard_history/database.rs:250`
- Added missing `ClipboardEntry` fields in both constructors:
  - `source_app_name: None`
  - `source_app_bundle_id: None`
  - `src/clipboard_history/database.rs:454`
  - `src/clipboard_history/database.rs:812`

Verification run:
- `cargo check -p script-kit-gpui`
- Result: still fails, but failures are outside this assigned file/scope:
  - `src/clipboard_history/db_worker/db_impl.rs`
  - `src/clipboard_history/clipboard.rs`
  - `src/clipboard_history/quick_look.rs`
  - `src/clipboard_history/temp_file.rs`

How to test:
1. Run `cargo check -p script-kit-gpui`.
2. Confirm `database.rs` no longer reports missing `ClipboardEntry` fields or non-exhaustive `ContentType` matches.
3. Resolve remaining out-of-scope compile errors above, then rerun `cargo check -p script-kit-gpui`.

Risks / known gaps:
- Workspace compilation is still blocked by related `ContentType`/`ClipboardEntry` updates needed in other files.
- No additional tests were run because compile currently fails before test execution due to those out-of-scope errors.

Commits:
- None.