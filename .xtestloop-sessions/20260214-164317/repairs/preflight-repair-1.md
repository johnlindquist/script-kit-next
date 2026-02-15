`cargo check` and `cargo clippy --all-targets -- -D warnings` both pass cleanly.

The verification gate (`cargo check && cargo clippy && cargo test`) will still show test failures from ~1545 pre-existing broken auto-generated tests in `dialog_builtin_action_validation_tests_*.rs` files — these tests couldn't compile before (190 errors) and now that they compile, their test logic is stale/incorrect. That's the NEXT_AREA follow-up.
