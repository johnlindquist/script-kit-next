Purged low-value tests in the assigned component/UI suites and kept only meaningful regression coverage.

Changed files:
- `src/components/form_fields_tests.rs`
- `src/components/unified_list_item_tests.rs`
- `src/clipboard_actions_tests.rs`
- `src/webcam_actions_consistency_tests.rs`

What was purged:
- `src/components/form_fields_tests.rs`: removed trivial `char_len` behavior tests and the language-level panic demo test that didn’t validate app logic.
- `src/components/unified_list_item_tests.rs`: removed constructor/default/helper-only tests (including local split/a11y helpers that didn’t exercise production code) and kept focused tests for UTF-8 boundary enforcement and layout invariants.
- `src/clipboard_actions_tests.rs`: removed broad duplicate string-scan tests and unrelated clipboard/chat/path checks; kept targeted destructive-action confirmation safeguards.
- `src/webcam_actions_consistency_tests.rs`: removed unrelated non-webcam coverage; kept webcam-dialog consistency assertions.

Reviewed with no net change:
- `tests/footer_button.rs` (already minimal and not containing dead/stub tests in current tree).

How to test:
1. Fix the current unrelated compile blocker in `src/prompts/select.rs:376` (`let mut ranges = Vec::new();` needs explicit type).
2. Run:
```bash
cargo check && cargo clippy --all-targets -- -D warnings && cargo test
```

Verification status:
- I ran the full gate twice; both runs failed before test execution due the same external error:
  - `E0282` at `src/prompts/select.rs:376`
  - plus warnings at `src/prompts/select.rs:605` and `src/prompts/select.rs:649`

Risks / known gaps:
- Full repo verification is currently blocked by the unrelated `select.rs` compile failure, so I could not complete a passing end-to-end cargo gate for this task.