Implemented the close-callback fix in `src/app_impl/actions_dialog.rs`.

**Changed files**
- `src/app_impl/actions_dialog.rs:217`
- `src/app_impl/actions_dialog.rs:283`

**What changed**
- `close_actions_popup` now reads and invokes the dialog `on_close` callback **before** shared popup state is cleared (`self.actions_dialog = None`), fixing the stale per-dialog cleanup issue (e.g. `file_search_actions_path` callbacks).
- Added overlay-depth guard logic so if `on_close` already restored focus stack state, `close_actions_popup` does not double-pop the overlay.
- Added regression test `test_close_actions_popup_invokes_on_close_before_clearing_dialog_state` to lock the callback-before-clear ordering.

**Verification run**
- `cargo check` (failed due unrelated existing workspace errors in clipboard/types and other modules)
- `cargo clippy --all-targets -- -D warnings` (failed due unrelated pre-existing errors/warnings across multiple modules/tests)
- `cargo test close_actions_popup_invokes_on_close_before_clearing_dialog_state --lib` (blocked by same unrelated compile errors)

**How to test once workspace is green**
1. Run:
```bash
cargo check
cargo clippy --all-targets -- -D warnings
```
2. Run the regression test:
```bash
cargo test close_actions_popup_invokes_on_close_before_clearing_dialog_state --lib
```
3. Manual behavior check: open an actions popup with per-dialog cleanup (like file-search actions), close with `Escape`/`Enter`, and confirm cleanup state is cleared.

**Risks / known gaps**
- Full verification is currently blocked by unrelated compile/clippy failures already present in the working tree.
- Regression test is a source-order guard (ensures callback call appears before state clear), not an end-to-end runtime test.

Commits made: none.