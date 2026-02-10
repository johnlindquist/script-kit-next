Implemented the requested favorites storage layer only.

**Changed files**
- `src/favorites/mod.rs:1`
  - Added `Favorites { script_ids: Vec<String> }`.
  - Added `load_favorites()`, `save_favorites(&Favorites)`, `toggle_favorite(&str)`, `is_favorite(&str) -> bool`.
  - Storage path is `~/.scriptkit/favorites.json` (missing file returns empty favorites).
  - Added 6 unit tests for missing file, save/load round-trip, toggle add/remove, and favorite checks.
- `src/lib.rs:112`
  - Added `pub mod favorites;`.

**How to test**
1. Scoped in-repo attempt (currently blocked by unrelated parallel-agent compile errors outside this scope):
   - `cargo test --lib favorites::`
2. Isolated verification (passes for this module):
   - Build a temp crate that includes `src/favorites/mod.rs` via `#[path = "..."]`.
   - Run:
     - `cargo clippy --all-targets -- -D warnings`
     - `cargo test`
   - Result: `6 passed, 0 failed`.

**Risks / known gaps**
- Full workspace gate (`cargo check && cargo clippy --all-targets -- -D warnings && cargo test`) is currently not runnable due unrelated compile failures in other files being edited in parallel.
- `is_favorite` intentionally returns `false` on load/parse errors (it does not surface errors).

**Commit**
- `cdbe73d` `feat(favorites): add JSON favorites storage layer`