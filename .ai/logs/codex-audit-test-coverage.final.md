**Findings**
1. High: Critical-path coverage is still missing for script execution and config loading in this scope. The scoped tests exercise action builders and `ProtocolAction` behavior, but do not invoke execution/config paths (`execute_script`/config load flows). Evidence: `src/actions/tests/core_part_01.rs`, `src/actions/tests/core_part_02.rs`, `src/actions/tests/core_part_03.rs`, `src/actions/builders_tests/part_01.rs`, `src/actions/builders_tests/part_02.rs`, `src/actions/builders_tests/part_03.rs`, `src/actions/builders_tests/part_04.rs`.
2. Medium: `test_filter_ranking_scoring` validates a reimplemented local scorer instead of production logic, so it can pass while real ranking regresses. `src/actions/tests/core_part_01.rs:109`, `src/actions/tests/core_part_01.rs:143`.
3. Low: Several tests are tightly order-coupled via `actions[0]` assertions, which can create brittle failures on benign ordering changes. `src/actions/builders_tests/part_01.rs:42`, `src/actions/builders_tests/part_01.rs:70`, `src/actions/builders_tests/part_01.rs:181`, `src/actions/builders_tests/part_01.rs:191`.

**Changes Made**
- Added targeted coverage for missing branches in `src/actions/tests/core_part_03.rs`:
- `src/actions/tests/core_part_03.rs:160` ProtocolAction serde default flags (`hasAction`, visibility, close).
- `src/actions/tests/core_part_03.rs:171` ProtocolAction camelCase explicit flag parsing.
- `src/actions/tests/core_part_03.rs:192` Scriptlet custom action conversion (`has_action`, value routing, shortcut formatting).
- `src/actions/tests/core_part_03.rs:225` Note switcher preview truncation + relative time formatting.
- `src/actions/tests/core_part_03.rs:246` AI command bar extended actions (`export_markdown`, `branch_from_last`, `toggle_shortcuts_help`).
- `src/actions/tests/core_part_03.rs:272` Custom `action_verb` propagation into `run_script` title/description.

**How To Test**
1. `cargo test core_part_03:: -- --nocapture`

**Verification Run**
- Passed: `cargo test core_part_03:: -- --nocapture` (10 tests passed in `actions::tests::core_part_03::*`).

**Risks / Known Gaps**
- End-to-end script execution and config loading are still not covered by this scoped suite.
- Global `cargo check/clippy/test` is not run here because this parallel-agent lane requires scoped verification only.

**Changed Files**
- `src/actions/tests/core_part_03.rs`

**Commit**
- `b102283` `test(actions): expand core_part_03 coverage`