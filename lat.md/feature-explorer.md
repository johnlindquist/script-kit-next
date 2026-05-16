# Feature Explorer

The Feature Explorer is a standalone XState-driven mockup for browsing the maintained feature atlas before refactoring production surfaces.

## Atlas Source

The explorer treats `feature-map/features/*.md` as the current human-maintained source and regenerates structured JSON before dev or build runs.

`feature_explorer/scripts/build-feature-data.mjs` parses each completed chapter into capabilities, concepts, entry points, workflows, interaction rows, state-machine rows, keystrokes, visual states, risks, gaps, and a generic table map keyed by section heading. Raw Oracle coverage counts only directories with `answer.md`; failed or incomplete raw attempts stay visible as incomplete rows. The generated `feature_explorer/src/data/features.generated.json` is a snapshot, not the authored source.

## XState Shell

The app shell represents explorer navigation with a single XState machine instead of ad hoc DOM state.

`feature_explorer/src/state/featureMachine.ts` owns selected feature, selected state, selected workflow, selected interaction, feature filtering, and mode transitions. `feature_explorer/src/state/authoredFeatureMachines.ts` stores authored machines for feature chapters that need richer runtime/wireframe semantics than table derivation can provide. `feature_explorer/src/state/featureRuntime.ts` loads those authored machines for Features 001 through 040, then falls back to deriving one executable XState machine per remaining feature from the chapter state rows, interaction matrix, and keystroke table. The fallback treats `State Machine` row exits as explicit state transitions and infers scenario-event targets from interaction/result text before using a next-state default. The UI in `feature_explorer/src/main.ts` renders from actor snapshots and sends explicit machine events for feature, tab, state, workflow, interaction, and runtime event navigation.

## Root Unified Launcher Wireframe

The Wireframe tab hosts playable, feature-map-backed mockups for selected feature groups.

The first registered wireframe covers Features 001, 008, 009, 010, 011, and 012 as a Root Unified Search launcher slice. It models the ScriptList input, source heads, grouped rows, stable selected row keys, Enter receipts, source-filter stripped text, and Cmd+K MainList actions with captured root subjects. Mock rows are content-light and use safe sample metadata rather than local clipboard, dictation, file, or conversation payloads.

Features without a custom registration render a generic feature-map wireframe that maps entry points, workflows, interactions, state, and proof hints into a launcher, prompt, chat, terminal, notes, or settings-shaped mock surface.

The wireframe registry lives in `feature_explorer/src/wireframes/` so future feature groups can add their own render, bind, and machine registration without changing the explorer shell.

Feature 010 stays authored because root ACP History needs explicit source-filter, cache-stability, passive ordering, non-bindable identity, root action, and shared ACP resume states that table derivation would flatten.

Feature 013 stays authored because first-character ScriptList handoffs need exact trigger, literal fallback, destination ownership, disabled action, and stale-decoration states.

Feature 014 stays authored because Quick Terminal needs explicit warm-pool, cwd handoff, PTY input ownership, native footer, apply-back, and SDK/ACP boundary states.

Feature 015 stays authored because SDK TermPrompt needs explicit SDK request, prompt routing, full-height terminal, action-mode, output-capture, and Quick Terminal boundary states.

Feature 016 stays authored because core prompt runtime needs explicit prompt-id continuity, arg/select/div focus, action-host, automation, and ForceSubmit boundary states.

Feature 017 stays authored because form and fields prompts have intentionally different maturity: `form()` needs explicit FormPrompt focus, validation, actions, automation, and privacy states, while `fields()` must remain modeled as SDK-visible but GPUI-incomplete until a real prompt view exists.

Feature 018 stays authored because `editor()` and `template()` are separate prompt surfaces: editor snippet mode must not be collapsed into TemplatePrompt, and TemplatePrompt's simulateKey, ForceSubmit, and actions gaps must remain visible.

Feature 019 stays authored because `path()` must remain distinct from File Search while preserving explicit PathPrompt browsing, Select-footer ownership, typed path actions, automation receipts, privacy, sizing, and filesystem-proof gaps.

Feature 020 stays authored because `drop()` has prompt-owned empty-submit guards, disabled footer state, native file-drop event-wiring proof gaps, metadata privacy boundaries, and adjacent ACP/File Search drop flows that table derivation would blur together.

Feature 021 stays authored because `env()` splits SDK-only preflight, custom prompt bypass, EnvPrompt UI, encrypted local secret storage, auto-submit, update/delete, footer ownership, and secret-redaction receipts across several runtime owners.

Feature 022 stays authored because SDK `hotkey()` is only a coming-soon stub, while the separate shortcut recorder mutates persistent command shortcuts through config writes, live registry updates, detached popup focus, and restart-required failure states. The authored machine keeps host toasts, SDK fallback HotkeyInfo, action-palette entry, recorder capture, config source-of-truth, startup priority, scriptlet refresh, and runtime proof gaps separate so the explorer cannot treat recorder behavior as proof of SDK hotkey capture.

Feature 023 stays authored because MiniPrompt and MicroPrompt are implemented compact choice prompts despite stale SDK warnings, and they need explicit sizing, footer, filtering, cancellation, automation, and boundary states to avoid conflating Mini window mode or microphone features.

Feature 024 stays authored because confirm has fail-closed SDK semantics, in-window and attached-popup routes, previous-view restore, native footer focus, popup automation, destructive caller gates, and key/registry regression states that table derivation would flatten.

Feature 025 stays authored because system feedback and prompt control APIs mix implemented HUD/actions/input behavior with fire-and-forget receipts and typed stub calls, so the explorer must preserve verification boundaries instead of treating protocol serialization as behavior.

Feature 026 stays authored because clipboard, selected-text, and Accessibility helpers have no prompt surface but cross SDK aliases, executor Submit responses, stdin typed receipts, macOS AX/focus handoff, clipboard restoration, privacy logging, platform unsupported paths, and incomplete image clipboard support. The authored machine keeps text/image clipboard ambiguity, selected-text native proof, check-vs-request permission behavior, and adjacent Clipboard History/Emoji/Sharing/AI boundaries explicit.

Feature 027 stays authored because keyboard and mouse helpers are SDK-visible unsupported stubs with fire-and-forget messages, payload-shape mismatches, no result receipts, no proven native input handler, and receipt-backed automation alternatives. The authored machine keeps unsupported warnings, protocol-only variants, weak-test pitfalls, simulateKey boundaries, and future native-input safety policy explicit.

Feature 028 stays authored because window controls mix fire-and-forget show/hide/blur/grid commands with request-backed bounds, screenshot, and layout queries whose proof semantics differ. The authored machine keeps show/hide no-envelope proof, hide reset/rekey, main-panel-only boundaries, grid screenshot requirements, bounds response-shape drift, screenshot privacy/pixel-audit gates, layout-vs-pixel limits, and the `window_visibility_ack` discrepancy visible.

Oracle reconciliation for Feature 028 added explicit gap states for unexpected fire-and-forget result envelopes, SDK screenshot target exposure, layout-only visual-proof overclaims, secondary-window `getState` misuse, and grid color-scheme path drift so those proof boundaries stay selectable in the atlas.

Feature 029 stays authored because widget, media, eye-dropper, and find APIs are mostly support-status boundaries rather than working surfaces. The authored machine keeps widget controller shape separate from visible widget support, widgetAction/event backend gaps, missing moved-event dispatch proof, media and eye-dropper throw-before-send behavior, mic/micro/dictation separation, `find()` submit semantics, missing backend route/`onlyin` gaps, and false-positive availability or generated-positive tests visible.

Feature 030 stays authored because ACP Chat SDK APIs cross SDK globals, protocol request IDs, direct storage/window handlers, UI-thread prompt routes, AI window runtime, and SDK-local subscription bookkeeping. The authored machine keeps direct APIs separate from `aiStartChat`/`aiFocus`, preserves SDK return-shape metadata drops, models `aiStartChat` image/context/no-response edges, and keeps declared-but-unhandled append/send/system-prompt/subscription APIs from being treated as runtime support.

Oracle reconciliation for Feature 030 added explicit protocol-correlation, protocol-only support overclaim, combined SDK return-shape loss, subscription-manager, and pushed-event producer gap states so shape tests cannot be mistaken for app-side ACP runtime support.

Feature 031 stays authored because legacy `chat()` is a prompt-centered SDK/runtime surface that must remain distinct from ACP `ai*` APIs. The authored machine keeps SDK global chat state, callback mode, built-in AI/setup mode, controller helper messages, ChatPrompt UI state, image paths, actions, persistence, Mini AI reuse, handoff, and known lifecycle gaps visible instead of flattening them into generic chapter rows.

Oracle reconciliation for Feature 031 added explicit states for `chat.getResult()` escape-shaped results, Mini-AI-named generic ChatPrompt telemetry, source-string test weakness, and AI-window handoff timeout so the explorer does not overclaim callback, persistence, or verification guarantees.

Feature 032 stays authored because the script catalog crosses plugin discovery, script metadata extraction, validation-safe kept catalogs, launcher diagnostics, scriptlet markdown/action parsing, scriptlet execution, scheduler boundaries, and MCP resources. The authored machine keeps validation bypasses, parse-error gaps, scriptlet duplicate-binding gaps, resource cache/casing boundaries, index-drift regressions, resource/UI drift, and template collision safeguards visible.

Feature 033 stays authored because clipboard sharing crosses action dispatch, portable bundle encoding, text-only collection, clipboard watcher privacy boundaries, parent confirmation, plugin install validation, and post-install refresh. The authored machine keeps shareable-vs-deeplink routing, recent-share suppression, trust fail-closed semantics, path/root validation, non-atomic install risk, logging sensitivity, and unproven provenance/binary/agent-refresh gaps visible.

Feature 034 stays authored because permission setup crosses built-in command routing, Settings reuse, native AppKit overlay lifetime, host-app drag payloads, passive macOS status APIs, MCP read-only permission tools, dictation microphone preflight, and screenshot proof boundaries. The authored machine keeps non-prompting invariants, manual setup semantics, overlay/locator proof gaps, `.app` payload requirements, MCP side-effect regressions, and adjacent legacy permission utilities distinct.

Feature 035 stays authored because Settings Hub, Theme Chooser, config preferences, theme payloads, user themes, and config fingerprint proof share storage concepts but not ownership. The authored machine keeps Settings visible-row parity, native footer ownership, Theme Chooser input isolation, explicit dismiss policy, config Rust/TypeScript alignment, schema-vs-runtime gaps, preset-vs-payload boundaries, theme validation, and metadata-only config proof distinct.

Feature 036 stays authored because tray lifecycle spans native status-menu rows, shared update state, About route focus, release distribution, and read-only MCP observation. The authored machine keeps stable tray ids, deferred startup, current-app row refresh, update detection vs self-install gaps, About keyboard/focus ownership, model-only tray inspection, release-manifest boundaries, and stale tray action paths distinct.

Feature 037 stays authored because Storybook visual coverage, product Design Gallery routes, Design Picker persistence, and agentic screenshot proof are related but not interchangeable. The authored machine keeps catalog role honesty, representation metadata, compare/adoption boundaries, Storybook window lifecycle, Storybook-vs-product proof separation, strict screenshot receipts, Design Gallery footer semantics, and preview-vs-commit design selection distinct.

Feature 038 stays authored because agent skills and AI context share the composer but have separate catalog, staging, preview, portal, MCP resource, SDK ordering, and submit-resolution contracts. The authored machine keeps plugin skill discovery/search, compact `SkillFile` staging, typed `AiContextPart` variants, `kit://context` schema/resource reads, `ContextPreviewInfo`, attachment portal return, explicit `FocusedTarget`, `AmbientContext` display-only behavior, ordered `aiStartChat` parts, and `ContextResolutionReceipt` success/failure states separate so the explorer does not flatten typed staged inputs into loose strings or hidden side effects.

Feature 039 stays authored because observability is proof infrastructure rather than a single user-facing surface. The authored machine keeps compact dev-loop logs, safe user-value previews, rate-limit suppression, stable debug markers, protocol-stats MCP health, transaction receipts/traces/resources/replay, AI preflight audit persistence, bounded JSONL recovery, and the boundary that logs support but do not replace domain-owned UI proof distinct.

Feature 040 stays authored because main-window sizing and surface contracts are a cross-cutting ownership boundary rather than a single visible screen. The authored machine keeps `MainWindowMode`/`ViewType` sizing, `SurfaceKind`/`LauncherSurfaceContract` semantics, native footer identity, attached actions-popup contracts, `triggerBuiltin` route mutation and automation re-keying, hide/reset bounds, deferred resize authority, generated surface matrices, and screenshot-only proof regressions separate so the explorer cannot infer Full/Mini mode from row count, entry source, or stale visual evidence.

## Verification

Explorer verification is static and build-focused until it needs visual parity with the production app.

Run `npm run build` from `feature_explorer/` to regenerate the atlas snapshot, type-check the XState-driven UI, and build the Vite app. Run `npm run dev -- --port 5177` to inspect the live explorer at `http://127.0.0.1:5177/`.
