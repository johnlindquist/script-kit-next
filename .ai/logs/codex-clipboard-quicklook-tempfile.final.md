Changed files:

- `src/clipboard_history/quick_look.rs:17`
  - Expanded `match entry.content_type` so `ContentType::Link | ContentType::File | ContentType::Color` use `write_text_preview(...)` like `Text`.
- `src/clipboard_history/temp_file.rs:16`
  - Expanded `match entry.content_type` so `Link/File/Color` follow the text path and write `.txt` temp files.
- `src/clipboard_history/temp_file.rs:69`
  - Added `source_app_name: None` and `source_app_bundle_id: None` to `test_save_text_entry_to_temp_file` fixture.
- `src/clipboard_history/temp_file.rs:98`
  - Added `source_app_name: None` and `source_app_bundle_id: None` to `test_save_image_entry_to_temp_file` fixture.

How to test:

1. Run:
   - `cargo check -p script-kit-gpui`
2. Run scoped test:
   - `cargo test -p script-kit-gpui clipboard_history::temp_file::tests::test_save_text_entry_to_temp_file`

What I observed:

- `cargo check -p script-kit-gpui` still fails, but on unrelated files outside this task (notably `src/clipboard_history/db_worker/db_impl.rs` and `src/clipboard_history/clipboard.rs`, plus other pre-existing workspace issues).
- Scoped `cargo test` also fails due broader existing compile/test issues unrelated to the two edited files.

Risks / known gaps:

- I could not get a green check/test run because the branch currently has unrelated compile failures outside the assigned scope.
- No additional behavior changes were made beyond the requested match arms and test fixture fields.