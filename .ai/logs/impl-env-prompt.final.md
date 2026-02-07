Implemented in `src/prompts/env.rs`:

1. Secret mask correctness and cursor/selection parity
- Added Unicode-safe masking via `masked_secret_value_for_display` (`src/prompts/env.rs:99`).
- Refactored input rendering so secret and non-secret paths both use the same cursor/selection renderer (`src/prompts/env.rs:317`, `src/prompts/env.rs:386`).

2. Validation and persistence failure handling
- Added explicit submit validation (`env_submit_validation_error`) and blocked empty/whitespace submits (`src/prompts/env.rs:111`, `src/prompts/env.rs:227`).
- On secret persistence/delete failure, prompt now keeps focus and shows inline error instead of silently succeeding (`src/prompts/env.rs:245`, `src/prompts/env.rs:294`, `src/prompts/env.rs:626`).
- Validation errors clear after text edits or successful actions (`src/prompts/env.rs:272`, `src/prompts/env.rs:443`).

3. Storage copy consistency and logging context
- Updated storage hint text to match actual backend (`~/.scriptkit/secrets.age`) and non-secret ephemeral behavior (`src/prompts/env.rs:103`, `src/prompts/env.rs:623`).
- Added `correlation_id` context to env prompt log lines (`src/prompts/env.rs:119`, `src/prompts/env.rs:176`, `src/prompts/env.rs:215`).

4. Tests added (TDD)
- `test_env_secret_mask_uses_char_count_when_input_contains_unicode`
- `test_env_storage_hint_describes_encrypted_store_when_secret`
- `test_env_storage_hint_describes_ephemeral_mode_when_not_secret`
- `test_env_validation_returns_error_when_submit_value_is_empty`
- Tests are in `src/prompts/env.rs:787`.

How to test:
1. Run the required gate:
- `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
2. Current blocker in repo (outside this scope):
- `src/prompts/select.rs:857` (`AnyElement` missing import)
- `src/prompts/select.rs:870` (missing closure type annotation)
- Because `cargo check` fails there, clippy/test do not proceed in the chained command.
3. After those are fixed, rerun the same gate and verify new env tests pass.

Risks / known gaps:
1. Full verification is blocked by unrelated compile errors in `src/prompts/select.rs`.
2. P0 protocol-level improvements from the plan (e.g. `auto_submit_existing` / force-prompt semantics) were not implemented because scope was limited to `src/prompts/env.rs`.

Changed files:
- `src/prompts/env.rs`

Commits made:
- None (no commit created; full verification gate is currently blocked by unrelated compile failures).