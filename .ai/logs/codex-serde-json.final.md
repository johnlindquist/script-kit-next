Audit complete. I added `crates-audits/serde-ecosystem.md` with a full serde/serde_json/serde_yaml usage review across `src/**/*.rs` and `Cargo.toml`, including prioritized recommendations and file/line evidence.

Changed files:
- `crates-audits/serde-ecosystem.md`

Verification run:
- Attempted: `CARGO_TARGET_DIR=target-codex-serde-json-verify cargo check && ... cargo clippy ... && ... cargo test`
- Result: failed during build with environment error `No space left on device (os error 28)` while compiling/linking dependencies.
- I cleaned temporary build artifacts from `target-codex-serde-json-verify` using `find ... -delete` to recover space.

How to test:
1. Review the report at `crates-audits/serde-ecosystem.md`.
2. After freeing more disk space, rerun:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`

Risks / known gaps:
- Full lint/test gate could not complete due disk exhaustion, so repository-wide verification is still pending.
- This task produced an audit report only; no runtime/code behavior was changed.

Commits made:
- None.