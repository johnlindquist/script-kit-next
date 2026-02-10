Implemented the export consistency fix across the requested actions files.

Changed files:
- `src/actions/mod.rs`:17
  - Switched from `#[path = "builders_rewire.rs"] mod builders;` to `mod builders;`
  - Consolidated/reordered public re-exports
  - Added missing public callback type exports: `ActionCallback`, `CloseCallback`, `CommandBarActionCallback`
  - Kept internal builder access as `pub(crate)` (`get_global_actions`, etc.)
- `src/actions/builders.rs`:5
  - Added `pub(super) mod types` re-export shim so builder submodules can consistently import `super::types::*`
- `src/actions/prelude.rs`:5
  - Expanded prelude type exports to include:
    - `ClipboardEntryInfo`
    - `CommandBar`, `CommandBarConfig`, `CommandBarHost`, `CommandBarActionCallback`
    - `ActionCallback`, `CloseCallback`
  - Added regression test `test_prelude_exports_include_public_actions_types_for_consumers` to enforce prelude type coverage

How to test:
1. `cargo check -p script-kit-gpui`
2. `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`

Verification results:
- Both commands were run and currently fail due unrelated in-progress tree issues outside this task’s scope (examples: duplicate `tests` module files under `src/protocol/message` and `src/designs/core`, clipboard enum exhaustiveness/missing fields across `src/clipboard_history/*`, and other pre-existing warning-as-error failures).

Risks / known gaps:
- Full gate is blocked by unrelated parallel-agent changes, so I could not get a clean global check/clippy pass from current branch state.
- `src/actions/builders_rewire.rs` is now unused but intentionally left untouched to stay within scoped, non-destructive changes.

Commits:
- None (verification gate is currently blocked by unrelated compile/lint failures).