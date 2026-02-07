Implemented the highest-priority button fixes in scope:

1. `src/components/button.rs`
- Added `id`, `loading`, `loading_label` state/builders (`src/components/button.rs:159`, `src/components/button.rs:205`, `src/components/button.rs:217`, `src/components/button.rs:223`).
- Added interaction gating helpers for pointer/keyboard activation (`src/components/button.rs:252`, `src/components/button.rs:256`, `src/components/button.rs:267`).
- Updated render behavior:
  - explicit id fallback to label (`src/components/button.rs:299`, `src/components/button.rs:382`)
  - pointer/hover only when interactive (`src/components/button.rs:416`)
  - block click + keyboard activation when disabled/loading/no handler (`src/components/button.rs:427`, `src/components/button.rs:440`)
  - loading visual state + optional loading label (`src/components/button.rs:301`, `src/components/button.rs:401`)
  - added consistent ghost sizing tokens (`src/components/button.rs:12`, `src/components/button.rs:390`).
- Added unit tests for new helper behavior (`src/components/button.rs:459`).

2. `src/components/footer_button.rs`
- Added `disabled`, `loading`, `loading_label` state/builders (`src/components/footer_button.rs:22`, `src/components/footer_button.rs:55`, `src/components/footer_button.rs:61`, `src/components/footer_button.rs:67`).
- Added id/clickability helpers (`src/components/footer_button.rs:83`, `src/components/footer_button.rs:87`).
- Updated render behavior:
  - explicit id fallback (`src/components/footer_button.rs:112`)
  - pointer/hover only when clickable (`src/components/footer_button.rs:136`)
  - block click when disabled/loading/no handler (`src/components/footer_button.rs:163`)
  - loading visual state + optional loading label (`src/components/footer_button.rs:113`, `src/components/footer_button.rs:144`).
- Added unit tests for new helper behavior (`src/components/footer_button.rs:175`).

Verification run:
- Ran required gate: `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
- Result: blocked by unrelated compile errors outside scope, currently in `src/prompts/select.rs:864` (`AnyElement` import/type issue) and `src/prompts/select.rs:877` (missing type annotation in closure).
- Because of that global blocker, full repo verification and fresh UI binary rebuild could not complete from this branch state.

How to test once repo is unblocked:
1. `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
2. UI smoke (stdin protocol):
`echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-footer-button.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`

Risks / known gaps:
1. Full verification is currently blocked by unrelated parallel-agent compile errors in `src/prompts/select.rs`.
2. Pressed/active visual token unification and full FooterButton-to-Button consolidation were not implemented (out of this quick-win pass).

Commits made:
- None (no verified commit possible while global gate is broken by unrelated files).