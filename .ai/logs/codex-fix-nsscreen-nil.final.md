**Changed file**
- `src/execute_script/part_001_body/execute_interactive_merged.rs:983`

**What I changed**
- Added explicit nil guards before dereferencing `NSScreen` objects in the macOS `GetWindowBounds` path:
  - Guard `screens` before calling `firstObject`.
  - Guard `main_screen` before calling `frame`.
- Added fallback height when no screen is available:
  - `FALLBACK_PRIMARY_SCREEN_HEIGHT = 1080.0`
  - Logs when fallback is used.

**Verification run**
- `cargo check -p script-kit-gpui` ✅ passed
- `cargo clippy -p script-kit-gpui --all-targets -- -D warnings` ❌ failed due to unrelated pre-existing warnings in `src/prompts/div/render_html.rs` (`clippy::field_reassign_with_default`), not in the edited file.

**How to test**
1. Run:
   - `cargo check -p script-kit-gpui`
   - `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`
2. Exercise `GetWindowBounds` on macOS and confirm no crash when `NSScreen` list is unavailable/empty; fallback path should log:
   - `"GetWindowBounds: NSScreen firstObject nil, using fallback screen height=1080"`

**Risks / known gaps**
- Fallback `1080.0` is safe but may produce less accurate Y conversion on unusual display setups.
- Global clippy gate is currently blocked by unrelated file(s), so a full clean clippy run is not yet possible.

Commits made: none.