# Script Kit DevTools API Coverage Map

This map defines the agent-facing DevTools surface that should grow before more recipe-heavy test scripts are added.

## Design Principle

Script Kit DevTools should feel closer to Chrome DevTools Protocol than to a folder of prewritten flows. Agents should discover targets, inspect semantic and visual state, measure layout, act through safe user-like protocol channels, compare red and green receipts, and record the investigation before promoting anything to a regression recipe.

## Checked-In Coverage Primitive

Use `bun scripts/devtools/coverage.ts` to ask what DevTools can currently inspect, what remains partial, and which runtime primitive should be built next.

Examples:

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
- Sources, Scripts, and Owners: prompt type, script provenance, source owners, and `lat.md` context for likely fixes.
- Performance and Timeline: input-to-paint, resize timelines, provider refresh, focus transitions, and layout shifts.
- Storage, Resources, and Privacy: context resources, redaction, cache/store generations, attachment provenance, and privacy boundaries.
- Accessibility: semantic-to-AX parity, focus order, disabled state, labels, and keyboard activation.
- Input, Focus, and Actions: protocol-first user actions, focus ownership, shortcut routing, safe clicks, and wrong-target refusal.
- Media, Sensors, and Permissions: dictation readiness, microphone permission, model readiness, recording state, delivery, and cleanup.
- Screenshots and Visual Proof: strict target capture, nonblank checks, pixel probes, semantic agreement, occlusion, and before/after proof.
- Investigation Records: bug intake, hypothesis trail, action transcript, receipts, classification, owner hints, and red/green artifacts.

## Notes Coverage Requirements

Notes must be treated as a first-class target, not as a launcher proxy. DevTools coverage must include the floating host, editor mode, browse/list mode, trash mode, markdown editor, markdown preview, editor find, global search, format toolbar, focus mode, pinning, sort cycling, command bar, actions panel, recent-note switcher, note cart, clipboard-backed note creation, embedded ACP mode, ACP actions popup, ACP history portal, attachment and context chips, draft snapshots, auto-resize, autosave and dirty state, history back/forward, scroll collapse after deleting trailing lines, and independent app-hide behavior.

Required Notes shortcuts and ownership paths include `Cmd+K`, `Cmd+P`, `Cmd+Shift+P`, `Cmd+F`, `Cmd+Shift+F`, `Cmd+N`, `Cmd+Shift+N`, `Cmd+Shift+T`, `Cmd+W`, `Cmd+.`, `Cmd+Shift+.`, `Cmd+Shift+S`, `Cmd+Z`, `Cmd+D`, `Cmd+Shift+D`, `Cmd+Shift+X`, `Cmd+Shift+L`, `Cmd+L`, `Cmd+Shift+-`, `Cmd+Shift+H`, `Cmd+V`, `Cmd+Shift+C`, `Cmd+E`, `Cmd+/`, `Cmd+J`, `Cmd+Shift+U`, `Cmd+B`, `Cmd+I`, `Cmd+Shift+I`, `Cmd+Enter`, `Cmd+Shift+A`, `Cmd+Shift+O`, `Cmd+Up`, `Cmd+Down`, `Cmd+Shift+Up`, `Cmd+Shift+Down`, `Cmd+[`, `Cmd+]`, `Cmd+Shift+Backspace`, `Cmd+Shift+Delete`, `Cmd+Shift+7`, `Cmd+Shift+8`, `Cmd+1..Cmd+9`, `Tab`, `Shift+Tab`, `Alt+Up`, `Alt+Down`, `Alt+Shift+Up`, `Alt+Shift+Down`, `Ctrl+Shift+K`, `Escape`, `Enter`, arrows, paging, Home/End, Backspace, and Delete.

Missing runtime primitives that block full Notes proof include target-scoped layout info, editor and preview scroll anchors, cursor and selection ranges, note store generation and sandbox identity, active note id and dirty state, command bar route stack, ACP embedded generation and origin receipts, portal session provenance, draft snapshot fingerprint, shortcut registry snapshots, focus owner transitions, and auto-resize before/after comparison.

## Dictation Coverage Requirements

Dictation needs media-aware DevTools instead of generic screenshots or scripts. DevTools coverage must include idle/hidden, recording, quiet recording, active speech, confirming, stop confirmation, transcribing, delivering, finished, failed/error, every phase transition from idle through failed, Script Kit target delivery, Notes editor target delivery, ACP target delivery, Tab AI target delivery, external/frontmost-app delivery, waveform/audio level bars, microphone permission, microphone device selection, preferred-device fallback, model readiness, model download/extract/failure status, hotkey readiness, hotkey registration, hotkey conflict detection, target identity, transcript generation, cursor insertion range, wrong-target rejection, and cleanup without mutating System Settings or TCC.

Required Dictation shortcut and input paths include the configured dictation hotkey, `Escape`, `Enter`, `Space`, and `Cmd+W`.

Missing runtime primitives that block full Dictation proof include `devtools.media.inspect`, passive microphone permission status, microphone device snapshots, model readiness generation, recording state generation, audio level metrics, target delivery generation, transcript fingerprints, cursor insertion range, wrong-target refusal receipts, hotkey binding snapshots, and media cleanup receipts.

Dictation History coverage must also expose fixture store identity, transcript row generation, preview generation, redacted transcript fingerprints, audio path redaction proof, scroll and selection anchor metrics, and portal attachment receipts.

## Build Order

1. Keep `devtools.inspect` as the first-orientation primitive.
2. Use `devtools.coverage` to classify the target and missing primitive before using a recipe.
3. Use `bun scripts/devtools/measure.ts --inspect <inspect.json> --coverage <coverage.json> --surface <id>` to turn inspect and coverage receipts into fail-closed layout, text fit, scroll, overlap, contrast, focus, media, and missing-primitive measurements.
4. Build `devtools.act` for safe protocol-first input, shortcut, and click receipts.
5. Use `bun scripts/devtools/media.ts --coverage <dictation-coverage.json>` as the first passive `devtools.media.inspect` slice before claiming live Dictation bugs are verifiable.
6. Build `devtools.compare` and `devtools.investigate` once measurement and action receipts have stable metric names.

Recipes should only wrap these primitives for smoke tests, common regressions, or CI-safe repros.
