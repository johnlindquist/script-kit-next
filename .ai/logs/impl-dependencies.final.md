Implemented the dependency plan updates in `Cargo.toml`.

**Changed files**
- `Cargo.toml`

**What changed**
- Removed likely-unused direct deps:
  - `time`
  - `ropey`
  - `inventory`
  - `clap`
  - `rust-i18n`
- Enabled lighter feature configs:
  - `syntect` -> `default-fancy` backend (instead of `regex-onig`)
  - `rusqlite` -> `default-features = false`, `features = ["bundled"]`
  - `ureq` -> `default-features = false`, `features = ["json", "rustls"]`

**Verification run**
- `cargo check` (passed)

**How to test**
1. Run `cargo check`
2. Optional sanity checks:
   - `cargo tree -e features -i syntect`
   - `cargo tree -e features -i rusqlite`
   - `cargo tree -e features -i ureq`

**Risks / known gaps**
- `syntect` backend switch (`regex-onig` -> `default-fancy`) can cause subtle syntax-highlighting differences.
- I did not update `Cargo.lock` security pins (`bytes` / `time`) in this task since scope was `Cargo.toml`.
- Full `clippy`/`test` gate was not run here; only `cargo check` per assignment.

**Commits**
- No commits made.