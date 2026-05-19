# DevTools Truth Scenarios

This directory records Oracle-reviewed, brand-new candidate scenarios for auditing whether Script Kit GPUI internal logic, built-in commands, footer text, action rows, disabled reasons, focus ownership, and route state do what their visible text says.

The ledger is receipt-shaped, not runner-shaped. It defines scenario IDs, user-story intent, safety constraints, transition requirements, and required receipt fields. It does not execute scenarios and must not be wired to an existing agentic recipe catalog.

The current Oracle evidence is the browser-backed session `new-devtools-scenarios-plan`, with output recorded at `~/.oracle/sessions/new-devtools-scenarios-plan/output.log`.

First-slice verification is source-only:

```sh
cargo test --test devtools_truth_scenario_ledger_contract -- --nocapture
cargo test devtools_coverage_contract -- --nocapture
cargo test devtools_oracle_iterations_contract -- --nocapture
```

Do not use these as executors for this ledger:

- `bun scripts/agentic/index.ts ...`
- `bun scripts/agentic/user-story-audit.ts ...`
- `bun scripts/agentic/surface-navigator.ts ...`
- `tests/smoke/*`
