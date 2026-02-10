Implemented the scoped kit-store git ops task and only changed the requested files.

**Changed Files**
- `src/kit_store/git_ops.rs`
  - Added:
    - `install_kit(repo_url: &str) -> Result<String, String>`
      - Extracts repo name (e.g. `https://github.com/user/my-kit` -> `my-kit`)
      - Runs `git clone <repo_url> ~/.scriptkit/kits/<kit_name>` (resolved via `$HOME/.scriptkit/kits/<kit_name>`)
      - Returns kit name on success
    - `update_kit(kit_path: &str) -> Result<(), String>`
      - Runs `git -C <kit_path> pull --ff-only`
    - `remove_kit(kit_path: &str) -> Result<(), String>`
      - Runs `std::fs::remove_dir_all`
  - Added unit tests for URL-to-name extraction.
- `src/kit_store/mod.rs`
  - Added `pub mod git_ops;`

**How To Test**
1. Run isolated tests for this file:
   - `rustc --edition=2021 --test src/kit_store/git_ops.rs -o /tmp/git_ops_tests && /tmp/git_ops_tests`
2. Optional isolated warning check:
   - `rustc --edition=2021 --crate-type lib -D warnings src/kit_store/git_ops.rs -o /tmp/libgit_ops.rlib`

I also attempted:
- `cargo test --lib kit_store::git_ops::tests`
- `cargo check --lib`

Both were blocked by unrelated in-progress workspace errors from other files.

**Risks / Known Gaps**
- Workspace-wide `cargo check/clippy/test` is currently failing due unrelated concurrent changes, so full-gate verification could not be completed in this branch state.
- `remove_kit` returns an error if the path does not exist (direct `remove_dir_all` behavior).

**Commit**
- `c3219e8` `feat(kit-store): add git ops helpers for kit lifecycle`