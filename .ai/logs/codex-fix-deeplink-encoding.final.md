Changed `to_deeplink_name` in `src/actions/builders/shared.rs:8` to address both requested issues:

- Non-ASCII slug chars are now percent-encoded (UTF-8 bytes, uppercase hex), e.g. `Café Script -> caf%C3%A9-script`.
- Empty/symbol-only input now returns sentinel slug `_unnamed` instead of `""`.

Also added focused unit tests in `src/actions/builders/shared.rs:83`:

- `test_to_deeplink_name_percent_encodes_non_ascii_when_present`
- `test_to_deeplink_name_returns_unnamed_when_input_is_empty_or_symbols`

**Verification run**

- `cargo check -p script-kit-gpui` (failed due unrelated pre-existing repo errors)
- `cargo clippy -p script-kit-gpui --all-targets -- -D warnings` (failed due unrelated pre-existing repo errors)
- `cargo test --lib test_to_deeplink_name_` (failed before running due unrelated pre-existing repo errors)

Key unrelated blockers observed:

- Duplicate test module files: `src/protocol/message/mod.rs:87`, `src/designs/core/part_04.rs:2`
- Missing `ClipboardEntry` fields in other areas: `src/clipboard_history/database.rs:454`, `src/clipboard_history/db_worker/db_impl.rs:67`
- Non-exhaustive `ContentType` matches: `src/clipboard_history/clipboard.rs:53`, `src/clipboard_history/database.rs:222`

**How to test (once repo baseline compiles again)**

1. `cargo test --lib test_to_deeplink_name_percent_encodes_non_ascii_when_present`
2. `cargo test --lib test_to_deeplink_name_returns_unnamed_when_input_is_empty_or_symbols`
3. `cargo check -p script-kit-gpui`
4. `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`

**Risks / known gaps**

- Existing tests across the repo still expect old empty-string behavior for symbol-only names; those tests were not updated in this scoped task.
- Parser-side percent-decoding (`scriptkit://run/...`) is still not implemented; this change takes the encoding approach instead.

Commits made: none.