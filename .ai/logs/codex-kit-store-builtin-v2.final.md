Implemented the builtins-only Kit Store command type additions.

**Changed files**
- `src/builtins/part_000.rs:162`
  - Added `KitStoreCommandType { BrowseKits, InstalledKits, UpdateAllKits }`.
- `src/builtins/part_000.rs:229`
  - Added `BuiltInFeature::KitStoreCommand(KitStoreCommandType)`.
- `src/builtins/part_001_entries/entries_003.rs:153`
  - Added built-in entry: `Browse Kit Store` with icon `"search"` (magnifying glass).
- `src/builtins/part_001_entries/entries_003.rs:169`
  - Added built-in entry: `Manage Installed Kits` with icon `"package"`.
- `src/builtins/part_001_entries/entries_003.rs:185`
  - Added built-in entry: `Update All Kits` with icon `"refresh-cw"`.

**How to test**
1. Run:
   - `cargo test --lib builtins::tests::part_000`
   - `cargo test --lib builtins::tests::part_001 --no-default-features`
2. Verify in builtins list output (or via tests once repo compiles) that IDs exist:
   - `builtin-browse-kit-store`
   - `builtin-manage-installed-kits`
   - `builtin-update-all-kits`

**Verification status**
- I ran both scoped commands above.
- Both failed before builtins execution due unrelated pre-existing compile errors in other modules (`actions`, `protocol`, `clipboard_history`), not in the edited files.

**Risks / known gaps**
- This task intentionally only added builtins types/entries. No execution/render/AppView wiring was added.
- If other areas match `BuiltInFeature` exhaustively, they may need follow-up handling for `KitStoreCommand` outside this scoped task.