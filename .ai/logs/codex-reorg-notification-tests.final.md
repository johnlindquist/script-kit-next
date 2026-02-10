Consolidation is complete: notification service tests are now merged into a single `src/notification/tests.rs` module, and `mod.rs` only declares that one test module.

Changed files:
- `src/notification/tests.rs`  
  Merged all tests from former `service_tests.rs` into this file.
- `src/notification/mod.rs`  
  Removed `#[cfg(test)] mod service_tests;`.
- `src/notification/service_tests.rs`  
  Deleted.

Verification run:
- `cargo check -p script-kit-gpui` ✅ passed.
- `cargo test -p script-kit-gpui notification::tests` ⚠️ blocked by unrelated compile errors in `src/actions/tests/dialog_builtin_validation/dialog_builtin_action_validation_tests_11/part_01.rs` (inner doc comments), outside this scope.

How to test:
1. `cargo check -p script-kit-gpui`
2. `cargo test -p script-kit-gpui notification::tests` (after unrelated actions test compile errors are fixed)

Risks / known gaps:
- Notification test execution could not be fully validated due unrelated in-progress changes in another module.
- This change is structural only (test file/module organization), with no intended behavior change.

Commits made:
- `d3d3e46` `refactor(notification): consolidate service tests into tests module`