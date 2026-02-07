Implemented the requested top-3 build improvements within scope (`Cargo.toml`, `.cargo/config.toml`) and did not touch CI/workflow files.

Changed files:
- `Cargo.toml:159` added `[profile.dev]` tuning for faster debug iteration:
  - `debug = 1`
  - `incremental = true`
  - `codegen-units = 256`
- `Cargo.toml:164` added matching `[profile.test]` tuning:
  - `debug = 1`
  - `incremental = true`
  - `codegen-units = 256`
- `.cargo/config.toml:9` added macOS linker/rustflags optimization:
  - `linker = "clang"`
  - `split-debuginfo=unpacked`
  - `link-arg=-Wl,-dead_strip`
- `.cargo/config.toml:18` added explicit incremental compilation env flags:
  - `CARGO_INCREMENTAL=1`
  - `CARGO_PROFILE_DEV_INCREMENTAL=true`
  - `CARGO_PROFILE_TEST_INCREMENTAL=true`

How to test:
1. Run `cargo check`

Verification run:
- `cargo check` passed.
- Existing warnings were reported in unrelated files (`src/theme/types.rs`, `src/watcher.rs`, `src/config/types.rs`).

Risks / known gaps:
- `-Wl,-dead_strip` can remove unused symbols, which may slightly affect some debug inspection workflows.
- I only ran `cargo check` (per assignment). I did not run `cargo clippy` or `cargo test` in this pass.
- `Cargo.toml` and `.cargo/config.toml` already had pre-existing unstaged edits from the timed-out prior worker; I layered only the requested build improvements on top and did not revert prior changes.

Commits made:
- None.