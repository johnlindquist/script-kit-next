# Tests

This directory holds focused test specifications that lock behavior the codebase should keep across refactors and host variations.

## Context picker provider hints

These tests lock provider-backed mention gating so ACP only advertises rows like `@dictation` when real slot or env-backed data exists.

The suite must serialize provider-slot and `SCRIPT_KIT_*_JSON` mutation because those inputs are process-global and can otherwise race under parallel `cargo test --lib` runs.

- [[acp-portal-contract]] - ACP portal contract coverage for exact replacement, preview text, and host query seeding.
- [[acp-dictation]] - ACP dictation delivery coverage for transcript handoff and focus behavior.
- [[dictation-setup-nux]] - Dictation setup NUX flow coverage for first-run model download, mic permission, and ready-state receipts.
