# Oracle Prompt: 031 Legacy `chat()` Prompt

## Slug

`legacy-chat-prompt-atlas`

## Project Brief

Script Kit GPUI is a Rust/GPUI desktop runtime with a TypeScript SDK (`scripts/kit-sdk.ts`) and stdin JSON protocol. The feature map is intended for humans and AI agents to fully understand Script Kit capabilities, including all states, user stories, script APIs, runtime routing, interaction details, persistence, verification, and known gaps.

Repo rules:

- `lat.md/` is the architecture knowledge graph. Feature-map work must stay aligned with existing `lat.md` pages and run `lat check`.
- Preserve the complete Oracle session output locally under `feature-map/raw-oracle/<feature-id>/`.
- The local agent writes maintained feature chapters under `feature-map/features/`; Oracle should return text only.

## Feature Scope

Map feature `031 Legacy chat() Prompt`.

This is the legacy SDK `chat()` conversational prompt surface, distinct from the ACP Chat SDK APIs (`ai*`). It includes:

- The TypeScript SDK `chat()` call and controller helpers.
- Initial messages, system shorthand, model/model-list options, placeholder/hint/footer/actions options, `saveHistory`, and callback modes.
- App-to-SDK `chatSubmit` responses.
- SDK-to-app messages for adding messages, streaming chunks, clearing, setting/clearing errors, and completing streams.
- The Rust `ChatPrompt` UI, input handling, keyboard shortcuts, streaming behavior, setup card, paste/image/screen capture behavior, markdown turn rendering, scroll follow, and error/retry affordances.
- Built-in AI mode when no `onInit`/`onMessage` callbacks are supplied, including provider setup and initial auto-response behavior.
- Parent app integration: `AppView::ChatPrompt`, focus target, main-window mini/full sizing, prompt shell ownership, actions dialog, model selection, copy/clear/capture actions, inline Mini AI entry, escape/continue/configure/Claude Code signal channels, save-to-history behavior, and handoff to the separate AI window.
- Test and verification surfaces.

## Current Evidence

The previous feature `030 ACP Chat SDK APIs` established that `ai*` functions are a separate capability surface. This feature maps the legacy prompt surface that still backs SDK `chat()` and the inline Mini AI prompt path.

Local inspection found:

- SDK `chat()` emits protocol `type: "chat"` and maintains local message state/controller state.
- With `onMessage`, submissions are looped back to the script through `chatSubmit`; scripts are responsible for assistant generation and controller updates.
- Without callbacks, `useBuiltinAi` is set and Rust handles model/provider backed AI streaming where providers exist; otherwise setup mode appears.
- Rust `ChatPrompt` owns the visual conversation, input, streaming, scroll behavior, setup card, keyboard actions, and persistence/handoff routines.
- Parent app code owns global actions popup routing and inline Mini AI signal handling.

## Bundle Map

Attached bundle includes:

- Process and repo rules: `AGENTS.md`, `CLAUDE.md`, `.goals/feature_map.md`.
- Relevant skills: prompt runtime, SDK/script execution, ACP chat contrast, protocol automation.
- Relevant `lat.md`: protocol, ACP chat, design, verification.
- SDK and protocol: `scripts/kit-sdk.ts`, chat protocol variants/types, prompt message handling.
- Runtime handlers: `src/prompt_handler/mod.rs`, `src/prompts/chat/*`, `src/render_prompts/other.rs`, app view/focus/action integration, inline Mini AI handlers.
- Tests: SDK chat test, smoke chat tests, mini AI snapshot/sizing/actions contracts.

The bundle is intentionally focused around the legacy chat prompt and adjacent lifecycle owners. If you need to infer broader system behavior, call out the inference explicitly.

## Deliverable

Return an operator-grade feature atlas for `031 Legacy chat() Prompt`.

Use this shape:

1. Capability summary: what the feature is, who uses it, and how it differs from ACP `ai*`.
2. Entry points and activation paths: SDK, protocol, built-ins/inline Mini AI, app view transitions.
3. User stories: enumerate every human/script/agent story this surface enables.
4. State model: all meaningful states, including empty, initial messages, callback mode, built-in AI mode, setup mode, streaming, stopped streaming, error, retryable error, actions popup, transfer/handoff, history save, cancel/escape.
5. Interaction contract: keyboard, mouse/click actions, footer/action dialog affordances, input editing, paste/image/screen capture, scroll follow, copy/clear, model selection, escape/Cmd+W/continue.
6. SDK/protocol contract: message shapes, controller methods, return values, result action semantics, app-to-SDK callbacks, mismatched-id behavior, constraints.
7. Persistence and side effects: history save, AI DB source, provider/config routing, generated side effects, clipboard, screenshots/captured images.
8. Agent/API observability: state surfaces, tests, protocol receipts, what agents can prove today and what is not directly observable.
9. Boundaries and non-goals: what belongs to ACP Chat, AI window, script generation, terminal/harness, or provider infrastructure instead.
10. Verification map: existing tests, recommended focused gates, missing test/spec gaps.
11. Known risks/gaps/ambiguities: exact files/functions and why they matter.
12. Suggested maintained chapter: concise but complete content suitable for `feature-map/features/031-legacy-chat-prompt.md`.

Be comprehensive. Prefer exact file/function references from the bundle. Explicitly mark inferred behavior when the bundle does not directly prove it.

Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.
