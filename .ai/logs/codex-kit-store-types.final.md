Implemented the `kit_store` module with types, registry storage, and manifest parsing.

Changed files:
- `src/kit_store/mod.rs`
- `src/kit_store/storage.rs`
- `src/kit_store/manifest.rs`
- `src/lib.rs`

What was added:
- `KitManifest` and `InstalledKit` types in `src/kit_store/mod.rs`.
- Registry persistence in `src/kit_store/storage.rs` at `~/.scriptkit/kit-store.json`:
  - `list_installed_kits()`
  - `get_kit(name)`
  - `save_kit_registry(kits)`
  - `remove_kit(name)`
- Manifest parsing in `src/kit_store/manifest.rs`:
  - `parse_kit_manifest(repo_path)` with precedence: `kit.json` then `package.json`.
- Module export via `pub mod kit_store;` in `src/lib.rs`.
- Unit tests for storage and manifest behavior.

How to test:
1. Scoped tests (in a healthy tree):
   - `cargo test --lib kit_store::manifest::tests`
   - `cargo test --lib kit_store::storage::tests`
2. In this run, shared-tree `cargo test` was blocked by unrelated compile errors in other modules, so I validated `src/kit_store/*` via an isolated harness crate where all 9 kit_store tests passed.

Risks / known gaps:
- `InstalledKit.installed_at` is stored as `String` (not a typed timestamp).
- `package.json` parsing treats object-form `scripts` as key names (useful fallback, but not always script file paths).
- Shared workspace currently has unrelated compile failures, so in-repo scoped cargo commands could not complete here.

Commits:
- `ebb1d11` `feat(kit_store): add kit manifest and registry storage primitives`