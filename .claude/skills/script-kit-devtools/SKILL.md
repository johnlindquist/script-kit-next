---
name: script-kit-devtools
description: >-
  Agent-facing DevTools for Script Kit GPUI: use protocol, MCP, and CLI primitives to inspect, control, measure, debug, benchmark, and prove real app UI behavior from bug reports without defaulting to prewritten agentic-testing recipes.
---

# Script Kit DevTools

This skill owns the agent-facing DevTools layer for Script Kit GPUI. It treats `agentic-testing` recipes as regression packs and smoke tests; the primary interface is direct protocol/MCP/CLI inspection, interaction, measurement, comparison, and proof against the real app.

Use this skill when a user reports a UX/UI bug, shares a screenshot, asks an agent to investigate the app like DevTools, or wants to expand the automation surface that agents use to debug Script Kit.

## Core Model

Think Chrome DevTools for Script Kit, not a script catalog.

The loop is:

1. Intake the bug report, screenshot, observed behavior, expected behavior, surface hints, and safety constraints.
2. Form a lightweight hypothesis.
3. Open or reveal the real app through the real user entry path.
4. Use DevTools primitives to inspect semantic state, layout, text, focus, popups, scroll, screenshots, and target identity.
5. Produce red proof or classify the blocker.
6. Identify likely code owner and missing primitive if proof is blocked.
7. After a code fix, rerun the same primitive stack for green proof.
8. Promote only stable, valuable flows into `agentic-testing` recipes.

## Use These Skills Together

- `$protocol-automation` when changing stdin JSON, `Message`, `getState`, `getElements`, `getLayoutInfo`, `waitFor`, `batch`, parse receipts, or target identity.
- `$agentic-testing` when the task needs a regression recipe, screenshot proof, process/session cleanup, or an existing stress scenario.
- `$platform-windowing-macos` when native windows, AppKit focus, AX, screen capture, or OS-level targeting owns the behavior.
- `$testing-quality-gates` when choosing narrow checks for protocol, Rust, Bun, or runtime proof.

## Current Primitive Families

Prefer direct primitives over prewritten recipes:

- Schema CLI: `bun scripts/devtools/schema.ts` reports the shared fail-closed receipt envelope, classifications, target identity fields, primitive schemas, and minimum acceptance bar for DevTools coverage.
- Targets CLI: `bun scripts/devtools/targets.ts list|inspect` exposes target discovery and strict target identity receipts so later inspect, measure, act, and compare commands do not parse target identity out of broad reports.
- Surface CLI: `bun scripts/devtools/surface.ts inspect --surface <SurfaceKind>` joins strict target identity to the generated surface contract, dismiss policy, runtime surface fields, capabilities, and missing primitive list.
- Elements CLI: `bun scripts/devtools/elements.ts snapshot` returns a target-scoped semantic tree with stable ids, labels, selected/focused ids, duplicate-id checks, and explicit missing bounds warnings.
- Layout CLI: `bun scripts/devtools/layout.ts measure` returns target identity plus component bounds, regions, clipping, overlap pairs, and resize-pressure metrics for visual/layout bug reports.
- Scroll CLI: `bun scripts/devtools/scroll.ts inspect` returns target identity plus scroll top, viewport/content heights, safe viewport, selected row visibility, footer occlusion, and overflow pressure.
- Focus CLI: `bun scripts/devtools/focus.ts inspect` returns target identity plus window focus, focused/selected semantic ids, focused/selected nodes, active footer, and keyboard ownership policy.
- Text CLI: `bun scripts/devtools/text.ts measure` returns target identity plus input/selected text fingerprints, text node lengths, footer label text, and explicit text-bounds gaps.
- Keyboard CLI: `bun scripts/devtools/keyboard.ts inspect` returns target identity plus keyboard policy, input ownership, footer bindings, popup/action shortcuts, duplicate-key checks, and routing warnings.
- Actions CLI: `bun scripts/devtools/actions.ts inspect --start --keep-open --open --open-target-kind notes` starts proof sessions with the actions-popup keep-open guard when requested, opens or targets the ActionsDialog from a parent target, and returns route stack, parent target, popup rect, placement/clipping, target-scoped `getLayoutInfo(actionsDialog)` search/header/list/visible-row/shortcut bounds, runtime row/section bounds, hover-state availability, `--prove-click-select`, `--prove-click-activate`, runtime shortcut layout bounds, visible actions, and explicit missing disabled-reason geometry.
- Act CLI: `bun scripts/devtools/act.ts set-input|select|key|open-actions` performs safe protocol-first user-like actions with strict target identity, pre/post focus and scroll receipts, explicit submit gating, target-scoped popup opening, and no native escalation.
- Main CLI: `bun scripts/devtools/main.ts inspect --start --show --prove-open-close-freshness` proves main-window close/reopen freshness with target-scoped state, target identity, and sampled stale-view refusal receipts.
- Compare CLI: `bun scripts/devtools/compare.ts redgreen --red <receipt> --green <receipt>` compares primitive stack, user path, target selector, target identity, metric names, and classification deltas for before/after bug proof.
- Events CLI: `bun scripts/devtools/events.ts tail|record` tails app/response logs or records the event span around any chosen DevTools command so agents can correlate interactions with protocol parsing, warnings, and responses.
- Notes CLI: `bun scripts/devtools/notes.ts inspect --open` opens or targets the Notes window, captures target/elements/focus/text/layout plus `getState(target notes)` receipts, and reports a redacted runtime envelope for active note id, dirty state, selection, draft snapshot fingerprints, focus surface, focus-owner transitions, counts, autosize state, editor scroll metrics, preview anchor availability, command bar route/action/filter state, shortcut ownership scopes, target-scoped actions/preview activation receipts, layout pressure, and storage generation/sandbox identity. `bun scripts/devtools/notes.ts resize-compare --start --sandbox` launches Notes against a sandbox DB, drives target-scoped editor input, and returns before/grow/shrink autosize measurements with generation ordering, stable-width checks, redaction proof, and cleanup. Pair with `bun scripts/devtools/layout.ts measure --target-kind notes` for focused Notes target-scoped titlebar/editor/footer/panel bounds and resize pressure.
- Dictation CLI: `bun scripts/devtools/dictation.ts inspect` passively inspects Dictation coverage, media requirements, redacted model/microphone/hotkey readiness, recording generation, audio-level availability, cleanup state, provider-resource availability, phases, targets, and delivery gaps without opening the microphone. `bun scripts/devtools/dictation.ts deliver-fixture --target mainWindowFilter --fixture-id short-phrase` injects a synthetic transcript through `pushDictationResult` and proves delivery generation, target routing, transcript length/fingerprint, main-filter insertion range, and redaction without microphone capture or raw transcript output.
- Inspector CLI: `bun scripts/devtools/inspect.ts --session <name> --start --show --main --bug "<report>" --surface <SurfaceKind>` composes the protocol primitives below into one agent-readable DevTools orientation report. It includes target identity, visible-window proof, the exact primitive stack it ran, legacy boolean capabilities, structured `capabilityDetails`, `missingFieldDetails`, fail-closed `classification`, `likelyOwners`, cleanup commands, warnings, errors, `recommendedNextPrimitives`, and `doNotUseRecipeReason` so agents do not hide missing instrumentation behind canned scripts.
- Coverage CLI: `bun scripts/devtools/coverage.ts --surface notes|dictation|main|actions-dialog` reports Chrome-DevTools-inspired domain coverage, supported primitives, missing runtime primitives, required shortcuts, and next API work before an agent reaches for a recipe.
- Surface Inventory CLI: `bun scripts/devtools/surfaces.ts` reads generated surface contracts, the feature-map index, and explicit coverage entries to produce the source-backed DevTools backlog before asking Oracle or building a new primitive.
- Investigation CLI: `bun scripts/devtools/investigate.ts --surface <id> --bug <report>` turns a user-filed UX/UI bug into a fail-closed red/green proof plan with target identity, required receipts, scenario hints, missing primitives, and recipe boundaries.
- Measure CLI: `bun scripts/devtools/measure.ts --inspect <inspect.json> --coverage <coverage.json> --surface <id>` converts inspect and coverage receipts into target identity, screenshot, semantic, layout, text, scroll, focus, media, and missing-primitive measurements.
- Media CLI: `bun scripts/devtools/media.ts --coverage <dictation-coverage.json>` is the fail-closed `devtools.media.inspect` slice for Dictation. It requires passive microphone, device, model, recording, transcript, target-delivery, hotkey, wrong-target, and cleanup receipts before live Dictation bugs can be called green.
- Targeting: `listAutomationWindows`, `inspectAutomationWindow`, exact automation ids, kind/index promotion.
- State: `getState`, `getAcpState`, surface contracts, popup contracts, active footer, visible counts, prompt-specific state.
- Semantics: `getElements(target)`, semantic ids, roles, labels, selected/focused ids, row metadata, warnings.
- Layout: `getLayoutInfo`, layout component bounds, window bounds, footer/input/list/content bounds.
- Interaction: `devtools.act`, `batch`, `waitFor`, `simulateKey`, `triggerBuiltin`, target-scoped `batch.setInput`, `batch.openActions`, `batch.selectBySemanticId`, and target-scoped `simulateKey` for surface-owned shortcuts.
- Visuals: `captureScreenshot`, `captureWindow`, `verify-shot.ts`, screenshot identity, strict target matching.
- Native observation: MCP `computer/*` read-only tools for windows, apps, screenshots, menus, screens, and permissions.
- Sessions: `scripts/agentic/session.sh`, response logs, app logs, cleanup and health checks.

## Investigation Contract

Every user-filed UX/UI bug investigation should produce an artifact or report with:

- intake: user report, screenshot paths, observed/expected behavior, suspected surface, safety constraints
- hypothesis log: current hypothesis, pivots, disproved explanations
- interaction transcript: action intent, control channel, target, command/input, visible result, receipt
- measurements: layout/text/scroll/focus/popup/screenshot metrics using stable field names
- classification: `reproduced`, `not-reproduced`, `fixed`, `blocked-by-missing-primitive`, `blocked-by-unsafe-operation`, `needs-user-info`
- likely owner: file/function/surface contract to inspect before editing
- proof plan: exact red and green commands/receipts
- cleanup: stopped sessions/processes and remaining windows

Do not call an investigation green because a recipe passed. Green means the same user-path symptom or measurement that failed red now passes after the fix.

## Decision Rules

- Start protocol/MCP-first. Use direct DevTools primitives before reaching for a long recipe.
- Use `devtools.inspect` first when the surface is unfamiliar, because it tells the agent what target it actually found and which follow-up primitives are supported or missing.
- Pass the user bug text into `devtools.inspect --bug` when available. The report should preserve the suspected surface, primitive stack, visible-window proof, missing primitive details, and recipe boundary before any fix is attempted.
- Open the real app for UX/UI reports. Protocol-only setup is fine, but visible/user-path proof must be present for visual or interaction bugs.
- Use recipes only when they match the bug directly or as regression proof after the investigation has isolated the issue.
- If direct primitives are missing, stop and name the missing primitive precisely.
- Never hide missing DevTools coverage behind screenshots, sleeps, native input, or broader recipes.
- Escalate to native input only when the bug depends on OS delivery, real focus, pointer behavior, AX, or screen-capture permission.

## Coverage Audit

Read `references/devtools-coverage-audit.md` when planning new DevTools surface work, expanding coverage, or deciding whether a user bug is blocked by missing instrumentation.

Read `references/devtools-api-coverage-map.md` when the work needs Chrome-DevTools-level breadth, Notes coverage, Dictation coverage, or the next protocol/API primitive. The checked-in coverage command is the machine-readable companion to that map.

Read `references/devtools-surface-inventory.md` when planning broad DevTools expansion, preparing Oracle prompts, or deciding which app surface families still lack direct primitives.

Read `references/devtools-oracle-buildout-plan.md` for the Oracle-reviewed primitive sequence, shared receipt envelope, and surface-specific proof requirements.

Read `references/oracle-devtools-scenario-iterations.md` when translating user bug reports into coverage work. It preserves the 50 Oracle-planned UX/UI bug scenarios and the DevTools fields needed to investigate them.

Use the audit as a living map, not a final verdict. Update it when new protocol/MCP primitives land or when a recipe reveals a missing primitive.

## Completion Gates

For DevTools surface changes:

- Add source-contract tests for protocol schema, routing, target identity, and fail-closed behavior.
- Produce at least one direct primitive receipt, not only a scripted recipe receipt, when behavior is runtime-observable.
