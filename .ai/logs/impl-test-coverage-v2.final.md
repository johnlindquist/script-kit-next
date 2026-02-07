Implemented the requested top-5 coverage additions and verified with `cargo check`.

**Changed files**
- `src/protocol/io.rs:74` added `is_unknown_message_type_error(...)` and refined classification logic so known message types with bad enum payloads are `InvalidPayload` (not `UnknownType`).
- `src/protocol/io.rs:679` added parser test: `test_parse_message_graceful_classifies_invalid_enum_payload_as_invalid_payload_when_type_known`.
- `src/fallbacks/builtins.rs:538` added builtin executor test: `test_execute_search_url_percent_encodes_reserved_chars_when_query_has_symbols`.
- `src/scripts/search.rs:2659` added search/filter test: `test_builtin_prefix_filter_allows_command_type_and_rejects_non_builtin_types`.
- `src/script_creation.rs:657` added script creation validation test: `test_create_new_script_in_dir_rejects_windows_reserved_name_after_sanitization`.
- `tests/window_resize_logic.rs:1` added window resize logic test: `test_height_for_view_uses_single_row_baseline_when_arg_choices_are_empty`.

**How to test**
1. `cargo test --lib test_parse_message_graceful_classifies_invalid_enum_payload_as_invalid_payload_when_type_known`
2. `cargo test --lib test_execute_search_url_percent_encodes_reserved_chars_when_query_has_symbols`
3. `cargo test --lib test_builtin_prefix_filter_allows_command_type_and_rejects_non_builtin_types`
4. `cargo test --lib test_create_new_script_in_dir_rejects_windows_reserved_name_after_sanitization`
5. `cargo test --test window_resize_logic`
6. `cargo check`

I ran all commands above successfully. `cargo check` passed with existing warnings in unrelated files.

**Risks / known gaps**
- I did not run full `cargo test` or clippy across the entire repo in this pass; verification was targeted plus `cargo check`.
- The workspace has many unrelated in-progress edits by other agents, so I kept changes scoped to the requested files/tests.

**Commits**
- No commits were made.