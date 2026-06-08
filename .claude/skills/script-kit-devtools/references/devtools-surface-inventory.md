# DevTools Surface Inventory

The surface inventory is the source-backed backlog for expanding Script Kit DevTools across every app surface without turning bug investigation into recipe execution.

## Inventory Command

Run `bun scripts/devtools/surfaces.ts` to emit `script-kit-devtools.surfaces` JSON. Run `bun scripts/devtools/surfaces.ts --markdown` when a human-readable audit table is easier to review.

The command reads `docs/ai/contracts/surface-contracts.json`, `feature-map/index.md`, and `scripts/devtools/coverage.ts`. It reports generated surface contracts, AppView variants, feature-map ownership, current explicit DevTools coverage, uncovered contract surfaces, and Oracle buildout batches.

## Why This Exists

User-filed UX/UI bugs often arrive as screenshots, vague reports, or interaction failures. The DevTools layer should first identify the exact target, surface contract, owner skills, measurements, and missing primitives needed to reproduce the bug. Only stable red/green flows should become `agentic-testing` regression recipes.

The buildout target is a protocol/MCP/CLI DevTools primitives layer that agents can understand without being pulled toward prewritten scenario scripts. In this model, scripted recipes remain regression packs after direct proof exists.

## Required Coverage Shape

Every major surface family should eventually have direct primitives for target discovery, semantic elements, layout and scroll measurement, style and text fit, input and focus, resources and storage, visual comparison, event timelines, and investigation records.

The first checked-in inventory groups buildout work into launcher/actions, prompt runtime, built-in filterable surfaces, portals/resources, AgentChat chat, Notes/Dictation/media, platform/windowing/permissions, storybook/design/theme, and observability/security/storage batches.

## Current Interpretation

Treat entries in `existingDevToolsCoverage.surfaceIds` as explicitly modeled in `coverage.ts`. Treat everything in `uncoveredContractSurfaceKinds`, plus every feature-map row not covered by a direct primitive, as a backlog candidate until a protocol/MCP/CLI receipt exists.
