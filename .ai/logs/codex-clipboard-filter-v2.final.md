Changed files:
- `src/clipboard_history/types.rs`

What I changed:
- Extended `ContentType` to include `Link`, `File`, `Color` at `src/clipboard_history/types.rs:9`.
- Updated `as_str`/`from_str` mappings for new variants at `src/clipboard_history/types.rs:17`.
- Added `classify_content(text: &str, has_image: bool) -> ContentType` with requested precedence at `src/clipboard_history/types.rs:40`.
- Updated `ClipboardEntryMeta::display_preview` match to handle new non-image variants at `src/clipboard_history/types.rs:145`.
- Added classification unit tests (image/link/file/text cases) at `src/clipboard_history/types.rs:261`.
- Did not modify database, monitor, or UI files.

How to test:
1. Run `cargo test --lib clipboard_history::types::tests`

Verification result:
- Command run: `cargo test --lib clipboard_history::types::tests`
- Result: failed due existing repo-level compile/test issues outside this task, plus downstream exhaustive matches now needing follow-up for new `ContentType` variants (for example `src/clipboard_history/clipboard.rs`, `src/clipboard_history/database.rs`, `src/clipboard_history/quick_look.rs`, `src/clipboard_history/temp_file.rs`).

Risks / known gaps:
- Introducing new `ContentType` variants causes non-exhaustive `match` failures in other modules until they are updated (out of this task’s scope).
- Unit tests added in `types.rs` are present, but cannot be executed to completion until the broader compile issues are resolved.

Commits:
- None (not committed due failing verification).