
## Slug

`legacy-chat-prompt-atlas`

## Project Brief

Script Kit GPUI is a Rust/GPUI desktop runtime with a TypeScript SDK (`scripts/kit-sdk.ts`) and stdin JSON protocol. The feature map is intended for humans and AI agents to fully understand Script Kit capabilities, including all states, user stories, script APIs, runtime routing, interaction details, persistence, verification, and known gaps.


- `removed-docs/` is the architecture knowledge graph. Feature-map work must stay aligned with existing `removed-docs` pages and run `source checks`.
- Preserve the complete Oracle session output locally under `feature-map/raw-oracle/<feature-id>/`.
- The local agent writes maintained feature chapters under `feature-map/features/`; Oracle should return text only.

## Feature Scope

Map feature `031 Legacy chat() Prompt`.


- The TypeScript SDK `chat()` call and controller helpers.
- Initial messages, system shorthand, model/model-list options, placeholder/hint/footer/actions options, `saveHistory`, and callback modes.
- App-to-SDK `chatSubmit` responses.
- SDK-to-app messages for adding messages, streaming chunks, clearing, setting/clearing errors, and completing streams.
- The Rust `ChatPrompt` UI, input handling, keyboard shortcuts, streaming behavior, setup card, paste/image/screen capture behavior, markdown turn rendering, scroll follow, and error/retry affordances.
- Built-in AI mode when no `onInit`/`onMessage` callbacks are supplied, including provider setup and initial auto-response behavior.
- Test and verification surfaces.

## Current Evidence

The previous feature `030 ACP Chat SDK APIs` established that `ai*` functions are a separate capability surface. This feature maps the legacy prompt surface that still backs SDK `chat()` and the inline Mini AI prompt path.


- With `onMessage`, submissions are looped back to the script through `chatSubmit`; scripts are responsible for assistant generation and controller updates.
- Without callbacks, `useBuiltinAi` is set and Rust handles model/provider backed AI streaming where providers exist; otherwise setup mode appears.
- Rust `ChatPrompt` owns the visual conversation, input, streaming, scroll behavior, setup card, keyboard actions, and persistence/handoff routines.
- Parent app code owns global actions popup routing and inline Mini AI signal handling.

## Bundle Map



The bundle is intentionally focused around the legacy chat prompt and adjacent lifecycle owners. If you need to infer broader system behavior, call out the inference explicitly.

## Deliverable

Return an operator-grade feature atlas for `031 Legacy chat() Prompt`.



Be comprehensive. Prefer exact file/function references from the bundle. Explicitly mark inferred behavior when the bundle does not directly prove it.

Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.
