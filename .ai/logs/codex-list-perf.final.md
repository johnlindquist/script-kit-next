Changed files:
- `plans/list-performance.md`

What I delivered:
- Added a severity-ranked performance audit for the unified list item/select list system in `plans/list-performance.md`, covering:
  - virtualization gaps
  - allocation hotspots in filtering/rendering
  - search/filter optimization opportunities
  - list item type-system issues
  - missing tests and a phased refactor plan

How to test:
1. `cargo check`
2. `cargo test` (currently fails due pre-existing unrelated compile error)

Verification run:
- `cargo check` ✅ passed
- `cargo test metadata_parses_shortcut_type_and_last_run -- --nocapture` ❌ failed due existing issue in `src/ai/window.rs:8549`:
  - `E0599`: `AiApp::message_body_content` not found

Risks / known gaps:
- This task produced an analysis report only; recommendations are not yet implemented in code.
- Full test suite is currently blocked by the unrelated `src/ai/window.rs` compile failure above.

Commits made:
- None