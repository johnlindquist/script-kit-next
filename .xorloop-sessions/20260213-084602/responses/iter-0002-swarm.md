`cargo check` and `cargo clippy` both pass cleanly (only a pre-existing nom deprecation warning). The 190 test compilation errors are all in pre-existing files not touched by this swarm.

## Swarm Results Summary

All 3 workers completed successfully:

| Worker | Status | File Changed | What it did |
|--------|--------|-------------|-------------|
| `unify-applescript-escape` | done | `src/utils/applescript.rs` | Strengthened `escape_applescript_string()` to also escape `
`, ``, `	` (not just `\` and `"`) + added test |
| `fix-execute-type-escape` | done | `src/executor/scriptlet.rs` | Replaced inline `.replace()` escaping in macOS `execute_type()` with `crate::utils::escape_applescript_string()` |
| `dedupe-builtin-applescript-escape` | done | `src/app_execute/builtin_execution.rs` | Deleted duplicate `applescript_escape()` function, replaced all call sites with `crate::utils::escape_applescript_string()` |

**Verification**: `cargo check` and `cargo clippy -D warnings` both pass. The test compilation failures are all pre-existing in unrelated files (`scripts/search/tests`, `actions/builders_tests.rs`, etc.) — none in our 3 changed files.

**Security fix**: The critical AppleScript injection vector in `execute_type()` is now closed — scriptlet `type:` values with embedded newlines can no longer break out of the AppleScript string literal.
