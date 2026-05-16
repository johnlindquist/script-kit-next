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
- `$lat-md` whenever changing `lat.md/` sections, refs, or test specs.

## Current Primitive Families

Prefer direct primitives over prewritten recipes:

- Inspector CLI: `bun scripts/devtools/inspect.ts --session <name> --start --show --main` composes the protocol primitives below into one agent-readable target report with capabilities, missing fields, warnings, errors, and recommended next primitives.
- Targeting: `listAutomationWindows`, `inspectAutomationWindow`, exact automation ids, kind/index promotion.
- State: `getState`, `getAcpState`, surface contracts, popup contracts, active footer, visible counts, prompt-specific state.
- Semantics: `getElements(target)`, semantic ids, roles, labels, selected/focused ids, row metadata, warnings.
- Layout: `getLayoutInfo`, layout component bounds, window bounds, footer/input/list/content bounds.
- Interaction: `batch`, `waitFor`, `simulateKey`, `triggerBuiltin`, target-scoped `batch.setInput`, `batch.selectBySemanticId`.
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
- Open the real app for UX/UI reports. Protocol-only setup is fine, but visible/user-path proof must be present for visual or interaction bugs.
- Use recipes only when they match the bug directly or as regression proof after the investigation has isolated the issue.
- If direct primitives are missing, stop and name the missing primitive precisely.
- Never hide missing DevTools coverage behind screenshots, sleeps, native input, or broader recipes.
- Escalate to native input only when the bug depends on OS delivery, real focus, pointer behavior, AX, or screen-capture permission.

## Coverage Audit

Read `references/devtools-coverage-audit.md` when planning new DevTools surface work, expanding coverage, or deciding whether a user bug is blocked by missing instrumentation.

Use the audit as a living map, not a final verdict. Update it when new protocol/MCP primitives land or when a recipe reveals a missing primitive.

## Completion Gates

For DevTools surface changes:

- Update `lat.md/protocol.md`, `lat.md/automation.md`, or `lat.md/verification.md` as appropriate.
- Add source-contract tests for protocol schema, routing, target identity, and fail-closed behavior.
- Run the narrowest Rust/Bun checks plus `lat check`.
- Produce at least one direct primitive receipt, not only a scripted recipe receipt, when behavior is runtime-observable.
