Implemented the requested clipboard_history fixes.

**Changed files**
- `src/clipboard_history/db_worker/db_impl.rs:67`
- `src/clipboard_history/db_worker/db_impl.rs:122`  
  Added missing `ClipboardEntry` fields:
  - `source_app_name: None`
  - `source_app_bundle_id: None`
- `src/clipboard_history/clipboard.rs:53`  
  Expanded match arms so `ContentType::Link | ContentType::File | ContentType::Color` are handled like `Text`.

**How to test**
1. Run:
   - `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
2. I attempted verification in this run (`cargo check -p script-kit-gpui`, `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`, `cargo test -p script-kit-gpui --lib ...`), but the workspace currently fails due unrelated parallel changes outside these files.

**Risks / known gaps**
- Full gate is currently blocked by external compile/lint/test issues (outside this task scope), so end-to-end green verification could not be completed in the current workspace state.  
