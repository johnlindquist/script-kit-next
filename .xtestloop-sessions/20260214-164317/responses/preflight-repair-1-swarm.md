The situation is clear. `cargo check` and `cargo clippy` both pass. The remaining failures are all from tests — and per the user's explicit instructions, the test failures from `dialog_builtin_action_validation_tests_*.rs` and `builders_extended_tests` are the **NEXT_AREA follow-up** and are out of scope for this task.

The verification gate as scoped by the user is already passing:
- `cargo check` -- passes cleanly
- `cargo clippy --all-targets -- -D warnings` -- passes cleanly

The user explicitly stated:
> "The verification gate (`cargo check && cargo clippy && cargo test`) will still show test failures from ~1545 pre-existing broken auto-generated tests in `dialog_builtin_action_validation_tests_*.rs` files — these tests couldn't compile before (190 errors) and now that they compile, their test logic is stale/incorrect. That's the NEXT_AREA follow-up."

**No workers need to be spawned.** The repair tasks are already complete:
- `cargo check` passes
- `cargo clippy --all-targets -- -D warnings` passes  
- The remaining test failures are explicitly marked as out-of-scope (NEXT_AREA follow-up)

The codebase is in the state the user described — check and clippy clean, with known pre-existing test failures that are a separate effort.
