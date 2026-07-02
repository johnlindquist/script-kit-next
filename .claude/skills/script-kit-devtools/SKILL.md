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

## Agent Feature-Verification Loop

When an agent is implementing a feature and needs to invoke APIs, push values, and verify UI state, prefer writing a small throwaway verification script over issuing many one-shot CLI calls. Each one-shot CLI invocation costs ~0.5-2s of process/session overhead; a driver script runs the same steps at ~10-50ms per step inside one process.

The loop:

1. Edit Rust code.
2. Build with `./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui`.
3. Write a probe script that imports `Driver` from `scripts/devtools/driver.ts`, drives the exact user path (`setFilterAndWait`, `simulateKey`, `batch`, `waitForState`), asserts on `getState`/`getElements` output, and prints one JSON receipt.
4. Run it in one shell call and read the receipt. Iterate.

Binary-path: with no explicit override, the driver and `session.sh` auto-pick the freshest (by mtime) of `target/debug/script-kit-gpui` (owned by `./dev.sh`) and `target-agent/pools/agent-debug/debug/script-kit-gpui` (agent-cargo's pool), and print which they chose to stderr — read that line and confirm it names the binary you just built. Pin explicitly with `Driver.launch({ binary: ... })` or `SCRIPT_KIT_GPUI_BINARY=...` when the choice matters (e.g. another loop may rebuild either path mid-run). For a stable per-task path that survives later rebuilds, export an artifact clone: `SCRIPT_KIT_AGENT_ARTIFACT_NAME=<task> ./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui` then launch `target-agent/artifacts/<task>/script-kit-gpui`.

### Pi Sidecar Availability (Agent Chat)

Dev/agent-built binaries run outside a `.app` bundle, so the bundled `Contents/MacOS/pi` sidecar never resolves; debug builds fall back to `target/pi-sidecar/pi`, then `~/dev/pi_agent_rust/target/{release,debug}/pi` (see `src/ai/agent_chat/pi/binary.rs`). When this skill is invoked, run `bash scripts/agentic/ensure-pi-sidecar.sh` once before launching or driving the app — it exits instantly when a Pi binary already resolves and otherwise builds the repo-local sidecar via `scripts/prepare-pi-sidecar.sh`. Without it, cmd+enter / Agent Chat surfaces show "Pi Agent Chat is unavailable", which is an environment gap, not an app bug. `./dev.sh` runs the same check automatically.

Driver rules of thumb:

- Use `sandboxHome: true` unless the bug specifically needs real user data; it keeps runs reproducible and protects real Script Kit state.
- Whole-scenario `batch` (setInput → waitFor → select) is one round trip and the fastest proof shape; per-command calls are still fast and easier to interleave with assertions.
- Always `await driver.close()` (use try/finally) so no app instance outlives the probe.
- The one-shot CLIs below remain correct for single inspections, strict target-identity receipts, and red/green compare artifacts. `scripts/agentic/root-source-filter-matrix.ts` is the migration template for porting a session.sh script to the driver.

### Parallel Feature Loops

Multiple feature loops may run simultaneously on the same checkout. The isolation contract:

- Driver sessions are conflict-free by default: `sessionName` is a label, and every `Driver.launch` derives a unique artifact directory (`/tmp/sk-driver-sessions/<name>-<pid>-<n>-<ts>`), so concurrent drivers — even with identical names, even in one process — never share app.log, protocol bus, or sandbox HOME. Read the actual path from `driver.sessionDir`. Only an explicit `sessionDir` opts out.
- Always pair `sandboxHome: true` with parallel loops so app instances never contend on real Script Kit databases.
- Builds: the shared agent pool serializes under one lock and produces ONE binary — two loops building different edits would overwrite each other. Do NOT give each loop its own pool (a pool costs tens of GB and the disk-budget eviction in `agent-cargo.sh` will reap extras). Instead, build in the shared pool and export a per-loop binary clone: `SCRIPT_KIT_AGENT_ARTIFACT_NAME=<loop-name> ./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui`, then `Driver.launch({ binary: "target-agent/artifacts/<loop-name>/script-kit-gpui" })`. The artifact is an instant APFS copy-on-write clone (~0 bytes), is replaced atomically on rebuild, and is never overwritten by other loops' builds. Loops with divergent source edits serialize on the pool lock — acceptable; relaunch to pick up new code.
- Legacy `session.sh` sessions are addressed purely by name: parallel loops MUST pass loop-unique session names (e.g. suffix the loop id/pid); reusing a name resumes or clobbers the other loop's session. Receipt scripts that write fixed `.test-output/<tool>` paths should namespace by session the way `root-source-filter-matrix.ts` does.
- Screen-level proofs do not parallelize: only one window is frontmost, so `show`/`windowFocused` waits, native input, and screenshot identity checks from concurrent loops race each other. Keep parallel loops on hidden-window protocol proofs (state/elements/layout/batch — none require `show`) and serialize the focus/visual proof at the end.

Proven: 3 concurrent drivers with the same label ran 30 batch scenarios in ~1s with distinct artifact dirs; 2 concurrent matrix runs both passed 48/48 with zero cross-talk and no slowdown.

## Use These Skills Together

These are global (user-level) skills, not files in this repo — if one is not
installed in your session, continue with this skill's primitives and say so.

- `$protocol-automation` when changing stdin JSON, `Message`, `getState`, `getElements`, `getLayoutInfo`, `waitFor`, `batch`, parse receipts, or target identity.
- `$agentic-testing` when the task needs a regression recipe, screenshot proof, process/session cleanup, or an existing stress scenario.
- `$platform-windowing-macos` when native windows, AppKit focus, AX, screen capture, or OS-level targeting owns the behavior.
- `$testing-quality-gates` when choosing narrow checks for protocol, Rust, Bun, or runtime proof.

## Current Primitive Families

Prefer direct primitives over prewritten recipes:

- Driver library: `scripts/devtools/driver.ts` is the persistent, event-driven protocol driver for multi-step and high-volume work. It owns the app process directly (stdin pipe in, stdout pipe out, responses matched by `requestId` with no polling and no per-command subprocess), so a full round trip costs single-digit-to-tens of milliseconds instead of the ~0.5-2s per command of subprocess-per-step flows. Import `Driver` from scenario scripts (`Driver.launch({ sandboxHome: true })`, then `setFilterAndWait`, `getState`, `waitForState`, `batch`, `simulateKey`, `close`). `bun scripts/devtools/driver.ts smoke` is the standalone health check; `bun scripts/agentic/driver-benchmark.ts` is the throughput receipt (~100 filter scenarios in ~6s vs ~0.5s/scenario via session.sh). Use the driver when a task runs more than a handful of protocol steps; use the one-shot CLIs below for single inspections and strict-receipt proofs.
- Schema CLI: `bun scripts/devtools/schema.ts` reports the shared fail-closed receipt envelope, classifications, target identity fields, primitive schemas, and minimum acceptance bar for DevTools coverage.
- Targets CLI: `bun scripts/devtools/targets.ts list|inspect` exposes target discovery and strict target identity receipts so later inspect, measure, act, and compare commands do not parse target identity out of broad reports.
- Surface CLI: `bun scripts/devtools/surface.ts inspect --surface <SurfaceKind>` joins strict target identity to the generated surface contract, dismiss policy, runtime surface fields, capabilities, and missing primitive list.
- Elements CLI: `bun scripts/devtools/elements.ts snapshot` returns a target-scoped semantic tree with stable ids, labels, selected/focused ids, duplicate-id checks, and explicit missing bounds warnings.
- Layout CLI: `bun scripts/devtools/layout.ts measure` returns target identity plus component bounds, regions, clipping, overlap pairs, and resize-pressure metrics for visual/layout bug reports.
- Scroll CLI: `bun scripts/devtools/scroll.ts inspect` returns target identity plus scroll top, viewport/content heights, safe viewport, selected row visibility, footer occlusion, and overflow pressure.
- Focus CLI: `bun scripts/devtools/focus.ts inspect` returns target identity plus window focus, focused/selected semantic ids, focused/selected nodes, active footer, and keyboard ownership policy.
- Text CLI: `bun scripts/devtools/text.ts measure` returns target identity plus input/selected text fingerprints, text node lengths, footer label text, and explicit text-bounds gaps.
- Keyboard CLI: `bun scripts/devtools/keyboard.ts inspect` returns target identity plus keyboard policy, input ownership, footer bindings, popup/action shortcuts, duplicate-key checks, and routing warnings.
- Actions CLI: `bun scripts/devtools/actions.ts inspect --start --keep-open --open --open-target-kind notes` starts proof sessions with the actions-popup keep-open guard when requested, opens or targets the ActionsDialog from a parent target, and returns route stack, parent target, popup rect, placement/clipping, target-scoped `getLayoutInfo(actionsDialog)` search/header/list/visible-row/shortcut bounds, runtime row/section bounds, hover-state availability, `--prove-click-select`, `--prove-click-activate`, `--prove-shortcut-open-freshness`, `--prove-shortcut-close-cleanup`, `--prove-escape-close-cleanup`, runtime shortcut layout bounds, visible actions, and explicit missing disabled-reason geometry.
- Act CLI: `bun scripts/devtools/act.ts set-input|select|key|open-actions` performs safe protocol-first user-like actions with strict target identity, pre/post focus and scroll receipts, explicit submit gating, target-scoped popup opening, and no native escalation.
- Main CLI: `bun scripts/devtools/main.ts inspect --start --show --prove-open-close-freshness --prove-early-frame-freshness` proves main-window close/reopen input freshness plus early-frame surface/footer/chrome freshness with target-scoped state, target identity, and sampled stale-view refusal receipts.
- Compare CLI: `bun scripts/devtools/compare.ts redgreen --red <receipt> --green <receipt>` compares primitive stack, user path, target selector, target identity, metric names, and classification deltas for before/after bug proof.
- Events CLI: `bun scripts/devtools/events.ts tail|record|logs|crashes` tails app/response logs, records the event span around any chosen DevTools command, queries the structured JSONL sinks (`logs --since <rfc3339|HH:MM:SS[.mmm]> --marker <text> --level <min> --target <substr> --cid <id>`), and surfaces the newest macOS DiagnosticReports `.ips` crash files with exception type and faulting-thread frames (`crashes`) since the dev.sh crash watchdog is opt-in.
- Notes CLI: `bun scripts/devtools/notes.ts inspect --open` opens or targets the Notes window, captures target/elements/focus/text/layout plus `getState(target notes)` receipts, and reports a redacted runtime envelope for active note id, dirty state, selection, draft snapshot fingerprints, focus surface, focus-owner transitions, counts, autosize state, editor scroll metrics, preview anchor availability, command bar route/action/filter state, shortcut ownership scopes, target-scoped actions/preview activation receipts, layout pressure, and storage generation/sandbox identity. `bun scripts/devtools/notes.ts resize-compare --start --sandbox` launches Notes against a sandbox DB, drives target-scoped editor input, and returns before/grow/shrink autosize measurements with generation ordering, stable-width checks, redaction proof, and cleanup. Pair with `bun scripts/devtools/layout.ts measure --target-kind notes` for focused Notes target-scoped titlebar/editor/footer/panel bounds and resize pressure.
- Dictation CLI: `bun scripts/devtools/dictation.ts inspect` passively inspects Dictation coverage, media requirements, redacted model/microphone/hotkey readiness, recording generation, audio-level availability, cleanup state, provider-resource availability, phases, targets, and delivery gaps without opening the microphone. `bun scripts/devtools/dictation.ts deliver-fixture --target mainWindowFilter --fixture-id short-phrase` injects a synthetic transcript through `pushDictationResult` and proves delivery generation, target routing, transcript length/fingerprint, main-filter insertion range, and redaction without microphone capture or raw transcript output.
- Inspector CLI: `bun scripts/devtools/inspect.ts --session <name> --start --show --main --bug "<report>" --surface <SurfaceKind>` composes the protocol primitives below into one agent-readable DevTools orientation report. It includes target identity, visible-window proof, the exact primitive stack it ran, legacy boolean capabilities, structured `capabilityDetails`, `missingFieldDetails`, fail-closed `classification`, `likelyOwners`, cleanup commands, warnings, errors, `recommendedNextPrimitives`, and `doNotUseRecipeReason` so agents do not hide missing instrumentation behind canned scripts.
- Coverage CLI: `bun scripts/devtools/coverage.ts --surface notes|dictation|main|actions-dialog` reports Chrome-DevTools-inspired domain coverage, supported primitives, missing runtime primitives, required shortcuts, and next API work before an agent reaches for a recipe.
- Surface Inventory CLI: `bun scripts/devtools/surfaces.ts` reads generated surface contracts, the feature-map index, and explicit coverage entries to produce the source-backed DevTools backlog before asking Oracle or building a new primitive.
- Investigation CLI: `bun scripts/devtools/investigate.ts --surface <id> --bug <report>` turns a user-filed UX/UI bug into a fail-closed red/green proof plan with target identity, required receipts, scenario hints, missing primitives, and recipe boundaries.
- Measure CLI: `bun scripts/devtools/measure.ts --inspect <inspect.json> --coverage <coverage.json> --surface <id>` converts inspect and coverage receipts into target identity, screenshot, semantic, layout, text, scroll, focus, media, and missing-primitive measurements.
- Media CLI: `bun scripts/devtools/media.ts --coverage <dictation-coverage.json>` is the fail-closed `devtools.media.inspect` slice for Dictation. It requires passive microphone, device, model, recording, transcript, target-delivery, hotkey, wrong-target, and cleanup receipts before live Dictation bugs can be called green.
- Targeting: `listAutomationWindows`, `inspectAutomationWindow`, exact automation ids, kind/index promotion.
- State: `getState`, `getAgentChatState`, `getAiWindowState`, `getConfigFingerprint`, surface contracts, popup contracts, active footer, visible counts, prompt-specific state.
- Logs: `getLogs {limit, level, target, contains}` returns recent structured entries (rfc3339 timestamp, level, target, correlation_id, message) from the app's in-process 500-entry ring — assert on log content in the same receipt stack as UI state (`Driver.getLogs()`); `events.ts logs|crashes` covers the on-disk JSONL sinks and crash reports.
- Semantics: `getElements(target)`, semantic ids, roles, labels, selected/focused ids, row metadata, warnings.
- Layout: `getLayoutInfo`, layout component bounds, window bounds, footer/input/list/content bounds.
- Interaction: `devtools.act`, `batch`, `waitFor`, `simulateKey`, `triggerBuiltin`, `triggerAction` (fire an actions-dialog action by id, bypassing keyboard navigation), `simulateClick` (window-relative protocol mouse click), `simulateGpuiEvent` (real GPUI event dispatch with dispatchPath/activationProof receipts — the only real-dispatch automation path), `simulateMainHotkeyGesture`, `setMenuSyntaxFormField`, `pasteClipboardIntoAgentChat`, `pushDictationResult` (synthetic dictation delivery, no microphone), target-scoped `batch.setInput`, `batch.openActions`, `batch.selectBySemanticId`, and target-scoped `simulateKey` for surface-owned shortcuts.
- Fixtures: deterministic UI without live agents or credentials — `openConfirmPrompt`, `openAgentChatKitchenSinkFixture`, `openAiWithMockData`/`openMiniAiWithMockData`, `openFocusedTextAgentChat{MockData,FromFocusedField,PiData}`, `openAgentChatDetachedFixture`, `setAgentChatTestFixture`, `openDictationOverlayFixture`, `openCreationFeedback`, `showAiCommandBar`. Prefer these for visual/layout proofs that do not depend on live provider state.
- Visuals: `captureScreenshot`, `captureWindow`, `verify-shot.ts`, `image-diff.ts`, screenshot identity, strict target matching.
- Perf/specialized CLIs: `perf.ts` (protocol timing), `agent_chat.ts` (Agent Chat receipts), `liquid-glass-proof.ts`, `apple-guideline-constants.ts`.
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
