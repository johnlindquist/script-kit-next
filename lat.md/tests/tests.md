# Tests

This directory holds focused test specifications that lock behavior the codebase should keep across refactors and host variations.

## Context picker provider hints

These tests lock provider-backed mention gating so ACP only advertises rows like `@dictation` when real slot or env-backed data exists.

The suite must serialize provider-slot and `SCRIPT_KIT_*_JSON` mutation because those inputs are process-global and can otherwise race under parallel `cargo test --lib` runs.

- [[acp-portal-contract]] - ACP portal contract coverage for exact replacement, preview text, and host query seeding.
- [[notes-acp]] - Notes-hosted ACP coverage for draft snapshots, replay guards, cart handoff, actions, Escape, and bounds contracts.
- [[acp-dictation]] - ACP dictation delivery coverage for transcript handoff and focus behavior.
- [[dictation-setup-nux]] - Dictation setup NUX flow coverage for first-run model download, mic permission, and ready-state receipts.
- [[permission-assistant]] - Permission Assistant coverage for passive macOS permission detection, native overlay entry points, and teardown.
- [[mini-window-contract]] - Mini main-window and Mini AI sizing, lifecycle, snapshot, mode-toggle, and popup-dismiss parity.

## ACP Chat

ACP Chat tests pin Agent Chat runtime boundaries that are easy to mistake for SDK-only shape support.

### Live subscription runtime

Live subscription runtime tests prove `aiSubscribe`/`aiUnsubscribe` are owned by the executing script response channel and that ACP thread events fan out through scoped subscription hooks.
