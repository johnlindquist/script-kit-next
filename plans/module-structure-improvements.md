# Module Structure Improvements Report

Date: 2026-02-07  
Agent: `codex-module-structure`  
Scope: `src/**/*.rs`

## Executive Summary

The codebase has strong feature coverage but currently pays a high navigation and maintenance cost from:

1. A root-level pseudo-monolith built with `include!` fragments in `src/main.rs`.
2. Several large, tightly coupled modules with bidirectional dependencies.
3. Heavy `mod.rs` files that combine production exports and large in-file test registries.
4. Duplicate root module declarations in both `src/main.rs` and `src/lib.rs`.

Primary recommendation: introduce explicit feature boundaries (especially for app shell, windowing, script catalog, and design/theme APIs), then break dependency cycles by extracting shared types/traits into small boundary modules.

## Method

- Scanned all Rust files under `src/` (`381` files, ~`319k` lines).
- Ranked largest files by line count and function density.
- Audited `mod.rs` and root `lib.rs`/`main.rs` for declaration and re-export patterns.
- Built top-level dependency edges from `use crate::...` and detected cycle candidates.

## High-Priority Findings

### 1) Root app composition is physically split but logically monolithic

Evidence:

- `src/main.rs` is very large and injects many implementation files via `include!`:
  - `app_impl.rs`
  - `execute_script.rs`
  - `prompt_handler.rs`
  - `app_navigation.rs`
  - `app_execute.rs`
  - `app_actions.rs`
  - `app_layout.rs`
  - `app_render.rs`
  - `render_builtins.rs`
  - `render_prompts/*.rs`
  - `render_script_list.rs`
- `src/main.rs` has ~`90` `mod ...;` declarations while `src/lib.rs` has ~`89` `pub mod ...;` declarations.

Impact:

- Namespace remains effectively flat despite physical file splits.
- Refactors are riskier because visibility and ownership boundaries are implicit.
- Navigation cost is high: behavior is distributed across includes but compiled as one unit.

Recommendation:

- Replace `include!` composition with true module boundaries:
  - `src/app/mod.rs`
  - `src/app/state.rs`
  - `src/app/actions.rs`
  - `src/app/navigation.rs`
  - `src/app/render/mod.rs`
  - `src/app/render/prompts.rs`
  - `src/app/render/builtins.rs`
- Keep `main.rs` as bootstrap only (init/logging/wiring).

### 2) Windowing/platform cycle (`platform -> window_manager -> window_state -> platform`)

Evidence:

- `src/platform.rs` imports `window_manager`.
- `src/window_manager.rs` re-exports/depends on `window_state`.
- `src/window_state.rs` imports platform types (e.g. display bounds abstractions).

Impact:

- Hard to reason about ownership of display/window state.
- Changes in one layer cascade through all three modules.

Recommendation:

- Introduce `src/windowing/types.rs` for pure data contracts (`DisplayBounds`, IDs, geometry wrappers).
- Move platform-specific querying into `src/platform/*` and keep window state platform-agnostic.
- Depend direction: `platform -> windowing/types`, `window_manager -> windowing/types`, `window_state -> windowing/types` (no back-edge to `platform`).

### 3) Design/theme/list-item cycle and oversized design hub

Evidence:

- `src/designs/mod.rs` is very large (~`2249` lines) and acts as both registry and behavior surface.
- Mutual edges exist between:
  - `designs <-> list_item`
  - `designs <-> theme`
- `src/theme/color_resolver.rs` references design-layer concepts.

Impact:

- Theme and visual variants are not clearly separated from rendering components.
- Circular imports increase risk of accidental cross-layer leaks.

Recommendation:

- Define one-way UI layering:
  - `theme` (tokens/resolution only)
  - `design_system` (style recipes using theme)
  - `components` (rendering)
- Move variant resolution into `design_system/variants.rs` and keep `theme` free of component/design imports.
- Move list-item-specific style adapters out of `designs/mod.rs` into `components/list_item_style.rs`.

### 4) Script catalog boundary cycle (`scripts <-> fallbacks`, `scripts <-> agents`)

Evidence:

- `src/scripts/types.rs` imports fallback/agent concerns.
- `src/fallbacks/*` imports script types.
- `src/agents/mod.rs` re-exports script-facing types creating back-coupling.

Impact:

- Script domain is not a stable core; fallbacks/agent concerns leak into base types.
- Hard to evolve script loading independent of fallback strategies.

Recommendation:

- Create a minimal `script_catalog` domain module with core entities only.
- Move fallback resolution to `script_resolution` module that depends on `script_catalog`.
- Make agents consume catalog traits/interfaces rather than re-exporting catalog types.

### 5) Action/Prompt/AI/Notes coupling cycle (`actions -> prompts -> ai -> actions`, `actions -> prompts -> notes -> actions`)

Evidence:

- `src/ai/window.rs` imports action concepts.
- `src/prompts/chat.rs` imports AI internals.
- `src/notes/window.rs` imports action-layer items.

Impact:

- UI surfaces (prompt, notes, AI) and command/action orchestration are interwoven.
- Difficult to test each surface in isolation.

Recommendation:

- Introduce `ui_events` interfaces (command enums + payloads) in neutral module.
- `actions` emits events; `prompts`/`ai`/`notes` handle via adapters.
- Replace direct cross-feature imports with event dispatch or trait objects.

## Medium-Priority Findings

### 6) `mod.rs` files are overloaded with re-exports and test plumbing

Evidence:

- `src/actions/mod.rs` (~`922` lines) includes many `#[path = ...] mod ...` test registrations.
- `src/components/mod.rs`, `src/prompts/mod.rs`, `src/theme/mod.rs`, `src/clipboard_history/mod.rs` contain many `pub use` + `#[allow(unused_imports)]` suppressions.

Impact:

- API surface is hard to discover and easy to accidentally expand.
- `allow(unused_imports)` can hide stale exports.

Recommendation:

- Keep `mod.rs` as index only.
- Move integration test registration out of production `mod.rs` into dedicated test harness modules.
- Replace wildcard-ish export surfaces with explicit curated prelude modules:
  - `actions/prelude.rs`
  - `prompts/prelude.rs`
  - `theme/prelude.rs`

### 7) Large feature files should be segmented by responsibility

Largest modules by line count include:

- `src/ai/window.rs` (~8552)
- `src/app_impl.rs` (~7383)
- `src/notes/window.rs` (~5097)
- `src/render_builtins.rs` (~4678)
- `src/main.rs` (~3866)
- `src/platform.rs` (~3488)
- `src/prompts/chat.rs` (~3387)
- `src/protocol/message.rs` (~2909)
- `src/watcher.rs` (~2789)
- `src/ai/providers.rs` (~2767)

Recommendation split targets:

- `ai/window.rs` -> `ai/window/{state.rs,events.rs,render.rs,controller.rs}`
- `notes/window.rs` -> `notes/window/{state.rs,commands.rs,render.rs,persistence_bridge.rs}`
- `protocol/message.rs` -> `protocol/message/{types.rs,parse.rs,validate.rs,serde_helpers.rs}`
- `watcher.rs` -> `watcher/{fs.rs,debounce.rs,event_map.rs,runtime.rs}`
- `platform.rs` -> `platform/{macos.rs,linux.rs,windows.rs,common.rs}` with thin facade

## Navigation and API Surface Improvements

1. Introduce per-feature `README.md` or module docs (`//!`) in:
   - `src/ai/`
   - `src/prompts/`
   - `src/scripts/`
   - `src/windowing/` (new)
2. Replace broad root exports in `src/lib.rs` with grouped feature exports.
3. Add architectural lint guidance:
   - forbid `theme -> designs/components` dependency direction
   - forbid `window_state -> platform`
4. Add `cargo modules` or custom dependency check script in CI to detect new cycles.

## Suggested Refactor Plan (Incremental)

### Phase 1: Boundary extraction (low behavior risk)

- Create pure type modules for windowing and script catalog domain.
- Update imports to use extracted types.
- Keep runtime behavior unchanged.

### Phase 2: Cycle breaking

- Break `platform/window_*` cycle.
- Break `designs/theme/list_item` cycle.
- Break `scripts/fallbacks/agents` cycle.

### Phase 3: App-shell modularization

- Convert `include!`-based app composition into true `app/*` modules.
- Reduce `main.rs` to startup/wiring.

### Phase 4: API cleanup

- Simplify `mod.rs` files.
- Remove stale re-exports and `allow(unused_imports)` suppressions where possible.
- Add curated prelude exports per feature.

## Risks / Migration Notes

- Large-file splits will create broad import churn; do in small PRs with behavior-preserving moves first.
- Cycle breaking may expose latent ownership issues; enforce directionality with compile-time module boundaries.
- Moving test registrations out of `mod.rs` requires care to preserve existing test discovery.

## Success Criteria

- No top-level module cycles in `use crate::...` dependency graph.
- `main.rs` reduced to bootstrap responsibilities only.
- `mod.rs` files primarily index modules and minimal re-export policy.
- Largest critical files reduced below ~2k lines where practical.
- Clear one-way architecture docs for theme/design/components/windowing/script domains.
