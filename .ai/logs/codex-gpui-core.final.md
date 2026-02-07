Implemented the GPUI usage audit and wrote the report to `crates-audits/gpui.md`.

Changed files:
- `crates-audits/gpui.md`

Verification run:
- Ran: `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
- Result: stopped at `cargo check` due unrelated existing compile errors:
  - `src/watcher.rs:23` (`config::defaults` private module errors, repeated)
  - `src/hotkeys.rs:367` (type mismatch `&HotkeyConfig` vs `&Option<HotkeyConfig>`, repeated)
- Because `cargo check` failed, `clippy` and `test` were not reached.

How to test:
1. Review audit content in `crates-audits/gpui.md`.
2. After the branch compile issues are fixed, run:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`

Risks / known gaps:
- Audit recommendations are static-analysis based (code search + upstream GPUI pattern comparison), not validated by implementing migrations.
- Upstream GPUI moves quickly; recommendations tied to current upstream snapshot should be periodically refreshed.

Commits made:
- None.