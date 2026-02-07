# API Surface Cleanup Report

Date: 2026-02-07
Agent: codex-api-surface
Scope: `src/**/*.rs`

## Summary

The crate currently exposes a very broad public surface (`2181` `pub*` declarations; `90` `pub mod` exports in `src/lib.rs`).
I applied low-risk encapsulation changes in areas where internals were already being treated as implementation details, and documented additional high-value cleanup opportunities.

## Implemented Changes

### 1) Removed leaked exports from enhanced window-control internals

- File: `src/window_control_enhanced/mod.rs:18`
- Change:
  - Added an API visibility contract as a `compile_fail` doctest.
  - Removed module-level re-exports of `bounds`, `capabilities`, `coords`, `display`, and `spaces` internals.
  - Kept only private submodules (`mod ...`) and marked module with `#![allow(dead_code)]` since this subsystem is currently internal-only.
- Why:
  - These types/functions were backend internals, not stable API contracts.
  - Re-exports made raw AX/backend details externally reachable.

### 2) Tightened crate root visibility for window-control enhanced module + state statics

- File: `src/lib.rs:69`
- Change:
  - `pub mod window_control_enhanced;` -> `pub(crate) mod window_control_enhanced;`
- Why:
  - After removing re-exports, the module should not remain publicly discoverable.

- File: `src/lib.rs:271`, `src/lib.rs:276`
- Change:
  - `MAIN_WINDOW_VISIBLE` and `SCRIPT_REQUESTED_HIDE` changed from `pub static` to private `static`.
- Why:
  - External callers already have function APIs (`is_main_window_visible`, `set_main_window_visible`, etc.).
  - Direct global-state access is an unnecessary API leak and bypasses invariants.

### 3) Converted resolver internals from public fields to semantic methods

- File: `src/theme/color_resolver.rs:28`
- Change:
  - Added `compile_fail` doctest asserting direct field access is invalid.

- File: `src/theme/color_resolver.rs:54`
- Change:
  - Made fields private for:
    - `ColorResolver`
    - `TypographyResolver`
    - `SpacingResolver`
  - Added semantic accessors needed by call sites:
    - `dimmed_text_color()`
    - `secondary_background_color()`
    - `font_size_xl()`
    - `margin_lg()`
  - Updated unit tests to assert via accessors, not fields.
- Why:
  - Public fields leaked token storage details and made future refactors risky.
  - Method-based API allows stable semantics while internal representation can evolve.

### 4) Migrated existing call sites to accessor-based API

- File: `src/render_script_list.rs:856`
- Change:
  - Replaced direct resolver field access with accessor methods.
- Why:
  - Keeps rendering code bound to semantics (`primary_text_color`, `border_color`, etc.) instead of storage names.

## Verification

### Targeted tests (pass)

- `cargo test --doc window_control_enhanced -- --nocapture`
  - Confirms `window_control_enhanced` internals are no longer importable from public API.
- `cargo test --doc color_resolver -- --nocapture`
  - Confirms resolver fields are private and accessor-based API is enforced.

### Runtime sanity (pass)

- `cargo build`
- `timeout 8 bash -c 'echo "{\"type\":\"show\"}" | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1'`
  - Logs show stdin protocol parsing succeeded (`Parsed command: Show`) and app startup/render paths remained functional.

### Full gate status (blocked by unrelated workspace issues)

Attempted:
- `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`

Result:
- `cargo check` passed.
- `cargo clippy`/`cargo test` failed due pre-existing unrelated errors outside this task scope, including:
  - duplicate `mod tests` conflict in `src/render_prompts/arg.rs` vs `src/main.rs`
  - multiple unused-import errors in generated action validation test files
  - missing `AiApp::message_body_content` in `src/ai/window.rs`

## Additional API-Surface Opportunities (Not Applied in This Change)

### High priority

1. Replace wildcard public re-exports with explicit symbols
- Files:
  - `src/protocol/mod.rs:56`
  - `src/icons/mod.rs:50`
  - `src/notes/mod.rs:46`
  - `src/components/unified_list_item/mod.rs:8`
- Why:
  - `pub use ...::*` makes the exported surface hard to audit and easy to accidentally widen.

2. Split internal vs external crate API in `src/lib.rs`
- File: `src/lib.rs:11`
- Observation:
  - `90` modules are exported publicly.
- Recommendation:
  - Keep internal modules as `pub(crate) mod`.
  - Add a narrow public facade (`pub mod api` or targeted `pub use` list) for intentional external consumers.

3. Reduce parser utility leakage from `scriptlets`
- File: `src/scriptlets.rs`
- Observation:
  - Many parsing helpers/constants are `pub` and likely used as implementation utilities.
- Recommendation:
  - Convert non-contract helpers to `pub(crate)` and expose only stable entry points.

### Medium priority

4. Audit `platform` exports for internal-only functions/constants
- File: `src/platform.rs`
- Observation:
  - Very large number of `pub fn` and macOS constants likely intended for app internals.
- Recommendation:
  - Keep only intentional SDK/public operations public; downgrade the rest to `pub(crate)`.

5. Limit direct state-channel exposure patterns
- File: `src/lib.rs:306`
- Observation:
  - `show_window_channel()` exposes sender/receiver tuple directly.
- Recommendation:
  - Prefer narrower operation APIs (`request_show_main_window`) and keep channel internals crate-private when feasible.

## Suggested Follow-up Sequence

1. Ban new wildcard re-exports in lint/review policy and migrate existing wildcard exports to explicit lists.
2. Introduce an intentional public facade in `lib.rs` and progressively flip internal modules to `pub(crate)`.
3. Run a focused `pub -> pub(crate)` migration for `scriptlets` and `platform` with compatibility tests.
4. Add API-surface regression checks (e.g., `cargo public-api` or a scripted `rg '^pub '` baseline diff in CI).

