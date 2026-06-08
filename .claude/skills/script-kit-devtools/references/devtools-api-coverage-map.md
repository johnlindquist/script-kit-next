# Script Kit DevTools API Coverage Map

This map defines the agent-facing DevTools surface that should grow before more recipe-heavy test scripts are added.

## Design Principle

Script Kit DevTools should feel closer to Chrome DevTools Protocol than to a folder of prewritten flows. Agents should discover targets, inspect semantic and visual state, measure layout, act through safe user-like protocol channels, compare red and green receipts, and record the investigation before promoting anything to a regression recipe.

## Checked-In Coverage Primitive

Use `bun scripts/devtools/coverage.ts` to ask what DevTools can currently inspect, what remains partial, and which runtime primitive should be built next.


```bash
bun scripts/devtools/coverage.ts --surface notes
bun scripts/devtools/coverage.ts --surface dictation
bun scripts/devtools/coverage.ts --domain media
bun scripts/devtools/coverage.ts --markdown
```

The command emits `script-kit-devtools.coverage` JSON with Chrome-inspired domains, surface feature coverage, shortcuts, supported primitives, missing runtime primitives, and recommended next work.

Each surface also carries `sourceFiles` so agents can jump from a missing DevTools primitive to the modules that own the behavior instead of guessing from screenshots or recipes.

## Domain Model

The coverage map intentionally mirrors Chrome DevTools breadth while using Script Kit language:

- Targets and Windows: exact automation windows, attached popups, detached panels, bounds, parentage, and screenshot identity.
- Elements and Semantics: semantic ids, roles, labels, selected and focused nodes, disabled reasons, action ids, and owners.
- Layout and Box Model: target-scoped bounds, scroll extents, anchor rects, safe areas, overlap pairs, and resize deltas.
- Styles, Theme, and Text Fit: theme tokens, computed colors, contrast, font metrics, wrapping, truncation, and clipping.
- Console, Logs, and Events: app logs, response logs, parse failures, warnings, traces, and action-correlated spans.
- Sources, Scripts, and Owners: prompt type, script provenance, source owners, generated contracts, feature-map chapters, and direct source file references for likely fixes.
- Performance and Timeline: input-to-paint, resize timelines, provider refresh, focus transitions, and layout shifts.
- Storage, Resources, and Privacy: context resources, redaction, cache/store generations, attachment provenance, and privacy boundaries.
- Accessibility: semantic-to-AX parity, focus order, disabled state, labels, and keyboard activation.
- Input, Focus, and Actions: protocol-first user actions, focus ownership, shortcut routing, safe clicks, and wrong-target refusal.
- Media, Sensors, and Permissions: dictation readiness, microphone permission, model readiness, recording state, delivery, and cleanup.
- Screenshots and Visual Proof: strict target capture, nonblank checks, pixel probes, semantic agreement, occlusion, and before/after proof.
- Investigation Records: bug intake, hypothesis trail, action transcript, receipts, classification, owner hints, and red/green artifacts.

## Notes Coverage Requirements

Notes must be treated as a first-class target, not as a launcher proxy. DevTools coverage must include the floating host, editor mode, browse/list mode, trash mode, markdown editor, markdown preview, editor find, global search, format toolbar, focus mode, pinning, sort cycling, command bar, actions panel, recent-note switcher, note cart, clipboard-backed note creation, embedded AgentChat mode, AgentChat actions popup, AgentChat history portal, attachment and context chips, draft snapshots, auto-resize, autosave and dirty state, history back/forward, scroll collapse after deleting trailing lines, and independent app-hide behavior.

Required Notes shortcuts and ownership paths include `Cmd+K`, `Cmd+P`, `Cmd+Shift+P`, `Cmd+F`, `Cmd+Shift+F`, `Cmd+N`, `Cmd+Shift+N`, `Cmd+Shift+T`, `Cmd+W`, `Cmd+.`, `Cmd+Shift+.`, `Cmd+Shift+S`, `Cmd+Z`, `Cmd+D`, `Cmd+Shift+D`, `Cmd+Shift+X`, `Cmd+Shift+L`, `Cmd+L`, `Cmd+Shift+-`, `Cmd+Shift+H`, `Cmd+V`, `Cmd+Shift+C`, `Cmd+E`, `Cmd+/`, `Cmd+J`, `Cmd+Shift+U`, `Cmd+B`, `Cmd+I`, `Cmd+Shift+I`, `Cmd+Enter`, `Cmd+Shift+A`, `Cmd+Shift+O`, `Cmd+Up`, `Cmd+Down`, `Cmd+Shift+Up`, `Cmd+Shift+Down`, `Cmd+[`, `Cmd+]`, `Cmd+Shift+Backspace`, `Cmd+Shift+Delete`, `Cmd+Shift+7`, `Cmd+Shift+8`, `Cmd+1..Cmd+9`, `Tab`, `Shift+Tab`, `Alt+Up`, `Alt+Down`, `Alt+Shift+Up`, `Alt+Shift+Down`, `Ctrl+Shift+K`, `Escape`, `Enter`, arrows, paging, Home/End, Backspace, and Delete.

Runtime primitives now expose target-scoped layout info, cursor and selection ranges, note store generation and sandbox identity, active note id and dirty state, command bar route stack, shortcut registry snapshots, focus owner transitions, redacted draft snapshot fingerprints, real editor scroll metrics, target-scoped `batch.togglePreview`, target-scoped `simulateKey` for the Notes `Cmd+Shift+P` preview shortcut, and sandboxed auto-resize before/after comparison through `bun scripts/devtools/notes.ts resize-compare --start --sandbox`. Missing runtime primitives that block full Notes proof include populated markdown preview scroll content bounds, AgentChat embedded generation and origin receipts, portal session provenance, and remaining Notes shortcut activation parity receipts beyond `Cmd+Shift+P`.

## Dictation Coverage Requirements

Dictation needs media-aware DevTools instead of generic screenshots or scripts. DevTools coverage must include idle/hidden, recording, quiet recording, active speech, confirming, stop confirmation, transcribing, delivering, finished, failed/error, every phase transition from idle through failed, Script Kit target delivery, Notes editor target delivery, AgentChat target delivery, Tab AI target delivery, external/frontmost-app delivery, waveform/audio level bars, microphone permission, microphone device selection, preferred-device fallback, model readiness, model download/extract/failure status, hotkey readiness, hotkey registration, hotkey conflict detection, target identity, transcript generation, cursor insertion range, wrong-target rejection, and cleanup without mutating System Settings or TCC.

Required Dictation shortcut and input paths include the configured dictation hotkey, `Escape`, `Enter`, `Space`, and `Cmd+W`.

Runtime primitives now support passive Dictation readiness without opening the microphone: `getState` publishes redacted model readiness, passive microphone permission and device snapshots, hotkey state, recording generation, idle audio-level availability, cleanup state, and the last redacted delivery receipt. `devtools.media.inspect` consumes passive readiness, and `dictation.deliver-fixture` uses `pushDictationResult` to prove delivery generation, transcript length/fingerprint, and main-filter insertion range without raw transcript output. Missing runtime primitives that still block full Dictation proof include insertion ranges for Notes/AgentChat/frontmost destinations and wrong-target refusal receipts.

Dictation History coverage must also expose fixture store identity, transcript row generation, preview generation, redacted transcript fingerprints, audio path redaction proof, scroll and selection anchor metrics, and portal attachment receipts.

## Actions Popup Layout Coverage

ActionsDialog and attached action menus must expose runtime-owned layout, not TypeScript-inferred boxes. `getLayoutInfo(target actionsDialog)` returns the ActionsDialog root, search input, optional context header, list viewport, visible grouped rows, and shortcut hint bounds from the same runtime row-geometry model used by `devtools.actions.inspect`. Disabled reason text bounds remain the explicit missing primitive only for routes that render visible disabled explanations.

## Build Order

1. Keep `devtools.inspect --bug "<report>" --surface <SurfaceKind>` as the first-orientation primitive. Its `inspect.orchestrate` receipt should preserve the bug text, visible target proof, primitive stack, capability details, missing primitive details, likely owners, cleanup commands, and the recipe boundary.
2. Use `devtools.coverage` to classify the target and missing primitive before using a recipe.
3. Use `bun scripts/devtools/measure.ts --inspect <inspect.json> --coverage <coverage.json> --surface <id>` to turn inspect and coverage receipts into fail-closed layout, text fit, scroll, overlap, contrast, focus, media, and missing-primitive measurements.
4. Build `devtools.act` for safe protocol-first input, shortcut, and click receipts.
5. Use `bun scripts/devtools/dictation.ts inspect --start --show` or `bun scripts/devtools/media.ts --coverage <dictation-coverage.json> --receipt <state-receipt.json>` as the passive `devtools.media.inspect` slice before claiming live Dictation bugs are verifiable.
6. Build `devtools.compare` and `devtools.investigate` once measurement and action receipts have stable metric names.

Recipes should only wrap these primitives for smoke tests, common regressions, or CI-safe repros.
