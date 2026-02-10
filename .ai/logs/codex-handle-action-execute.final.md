Implemented the requested compile fixes in the two scoped files.

Changed files:
- `src/app_actions/handle_action.rs:1321`  
Added missing `ClipboardEntry` fields in the struct initializer:
`source_app_name: None` and `source_app_bundle_id: None`.
- `src/execute_script/part_001_body/execute_interactive_merged.rs:366`  
Expanded `match e.content_type` (preview content) to handle `Link | File | Color` the same as `Text`.
- `src/execute_script/part_001_body/execute_interactive_merged.rs:380`  
Expanded `match e.content_type` (protocol mapping) to map `Link | File | Color` to `protocol::ClipboardEntryType::Text`.

How to test:
1. Run `cargo check --lib`  
Expected: passes (this is what I ran successfully).
2. Optional full check in your environment: `cargo check`

Verification run:
- `cargo check --lib` passed.
- I also tried `cargo test --lib test_truncate_clipboard_history_preview`, but it fails due unrelated pre-existing test/module conflicts (e.g. duplicate `tests` module files in `src/protocol/message` and `src/designs/core`), not from these edits.

Risks / known gaps:
- No behavior risk expected from these edits; changes are strictly exhaustiveness/default-field fixes.
- Full test suite remains blocked by unrelated existing repository issues outside this task scope.