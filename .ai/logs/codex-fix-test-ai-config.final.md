Implemented a test-only fix in `src/ai/config.rs` to make mock-provider visibility assertions deterministic under parallel test execution.

Changed files:
- `src/ai/config.rs:415`
- `src/ai/config.rs:639`
- `src/ai/config.rs:656`

What changed:
- Added `SHOW_MOCK_PROVIDERS_TEST_LOCK` + `with_show_mock_providers_env_for_test(...)` helper to serialize `SHOW_MOCK_PROVIDERS` mutation and restore prior env state via `Drop`.
- Updated:
  - `test_mock_providers_hidden_by_default`
  - `test_mock_providers_shown_when_enabled`
  to run through that helper.

How to test:
1. `cargo test ai::config::tests::test_mock_providers_hidden_by_default`
2. `cargo test ai::config::tests::test_mock_providers_`

Risks / known gaps:
- This lock is local to `src/ai/config.rs` tests. Other test modules that mutate `SHOW_MOCK_PROVIDERS` without the same lock can still race with each other.

Commits:
- `5467c0d` `test(ai-config): stabilize mock provider visibility tests`