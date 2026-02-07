# codex-audit-act-types-scriptinfo

## Scope
- Audited: `src/actions/types/script_info.rs`
- Updated tests: `src/actions/types/tests.rs`

## Findings

1. **Missing `Default` implementation (fixed)**
- `ScriptInfo` had many constructor paths but no canonical default.
- Added `impl Default for ScriptInfo` with safe empty context values:
  - `is_script=false`, `is_scriptlet=false`, `is_agent=false`
  - `action_verb="Run"`
  - optional metadata unset

2. **Edge case: empty `shortcut`/`alias` treated as present (fixed)**
- Prior constructors accepted `Some("")`/whitespace and preserved it.
- Action builders gate on `is_some()`, so blank values incorrectly enabled "update/remove" actions.
- Added normalization to collapse blank/whitespace-only optional strings to `None`.

3. **Edge case: inconsistent frecency state (fixed)**
- `with_frecency(true, None)` previously yielded `is_suggested=true` with no `frecency_path`.
- Added invariant enforcement in `with_frecency`:
  - normalize blank path to `None`
  - `is_suggested` is true only when a non-empty path exists

4. **No conversion impls for common tuple inputs (fixed)**
- Added:
  - `impl From<(&str, &str)> for ScriptInfo`
  - `impl From<(String, String)> for ScriptInfo`
- Both map to `ScriptInfo::new(...)`.

5. **Field docs clarity (improved)**
- Clarified `frecency_path` doc to state expected non-empty path when `is_suggested=true`.

## Additional implementation notes
- Consolidated constructor behavior via a shared private `build(...)` function to keep defaults/invariants consistent across all public constructors.

## Verification
- `rustfmt src/actions/types/script_info.rs src/actions/types/tests.rs`
- `cargo test --lib script_info`
  - Result: **62 passed, 0 failed**
