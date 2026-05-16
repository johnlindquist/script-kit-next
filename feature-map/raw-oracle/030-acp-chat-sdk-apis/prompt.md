# Oracle Prompt: 030 ACP Chat SDK APIs

[acp-chat-sdk-apis-atlas]

Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.

## Project Briefing

This is the Script Kit GPUI repository. It is a Rust/GPUI desktop app with a TypeScript/Bun Script Kit SDK shim in `scripts/kit-sdk.ts`. The project uses `lat.md/` as its architecture and verification knowledge graph. Repo process requires:

- Run `lat expand` and `lat search` before feature-map work.
- Include `AGENTS.md`, `CLAUDE.md`, owning skills, relevant `lat.md/` pages, and `lat.md/verification.md` in Oracle context or prompt.
- Preserve raw Oracle output under `feature-map/raw-oracle/<feature-id>/`.
- Distill the maintained chapter under `feature-map/features/<feature-id>.md`.
- Run `lat check` after each task.

Owning skills for this pass:

- `acp-chat-core`
- `acp-context-composer`
- `sdk-script-execution`
- `protocol-automation`
- `storage-cache-security`
- `agentic-testing`

## Feature Scope

Map feature `030-acp-chat-sdk-apis`: the script-facing ACP/AI Chat SDK APIs.

Cover these APIs and their full behavior:

- `aiIsOpen()`
- `aiGetActiveChat()`
- `aiListChats(limit?, includeDeleted?)`
- `aiGetConversation(chatId?, limit?)`
- `aiStartChat(message, options?)`
- `aiAppendMessage(chatId, content, role)`
- `aiSendMessage(chatId, content, imagePath?, parts?)`
- `aiSetSystemPrompt(chatId, prompt)`
- `aiFocus()`
- `aiGetStreamingStatus(chatId?)`
- `aiDeleteChat(chatId, permanent?)`
- `aiOn(eventType, handler, chatId?)`
- `aiSubscribe` / `aiUnsubscribe` protocol messages and pushed event variants if relevant.

This pass is specifically the script-facing SDK/protocol/runtime contract. Do not repeat the whole ACP UI/context-composer chapter unless needed to explain SDK side effects. Distinguish:

- SDK globals and TypeScript message/result shapes.
- Rust protocol variants and request ids.
- Direct storage-backed handlers.
- UI-thread prompt-message handlers.
- Declared-but-unhandled or partially handled protocol messages.
- ACP window vs older `src/ai/window` chat storage paths.
- Stored chat metadata and message content boundaries.
- Context parts and image attachment behavior.
- Streaming status and event subscription behavior.
- False-positive tests that prove only message shape rather than runtime support.

## Current Evidence From Local Inspection

`lat expand` was run with:

```bash
lat expand "030 ACP Chat SDK APIs: aiIsOpen aiGetActiveChat aiListChats aiGetConversation aiStartChat aiAppendMessage aiSendMessage aiSetSystemPrompt aiFocus aiGetStreamingStatus aiDeleteChat aiOn"
```

`lat search` was run with:

```bash
lat search "ACP Chat SDK AI APIs aiIsOpen aiGetActiveChat aiListChats aiGetConversation aiStartChat aiAppendMessage aiSendMessage aiSetSystemPrompt aiFocus aiGetStreamingStatus aiDeleteChat aiOn"
```

Top relevant `lat.md` sections:

- `lat.md/acp-chat#ACP Chat`
- `lat.md/ai-context#AI Context and MCP#ACP handoff`
- `lat.md/acp-chat#ACP Chat#Mini AI And Full ACP Handoff Parity`
- `lat.md/acp-chat#ACP Chat#Entry paths`
- `lat.md/acp-chat#ACP Chat#ACP composer`

Local scan findings that need Oracle scrutiny:

- `scripts/kit-sdk.ts` declares all listed `ai*` globals and pushed event types.
- `src/protocol/message/variants/ai.rs` defines protocol variants for request/result/event messages.
- `src/ai/sdk_handlers.rs` directly handles `aiIsOpen`, `aiGetActiveChat`, `aiListChats`, `aiGetConversation`, `aiDeleteChat`, and `aiGetStreamingStatus`.
- `src/execute_script/mod.rs` forwards direct AI handler responses before UI-thread prompt dispatch.
- `src/execute_script/mod.rs` maps only `AiStartChat` and `AiFocus` into `PromptMessage`.
- `src/main_sections/prompt_messages.rs` only has `AiStartChat` and `AiFocus` prompt messages.
- `src/prompt_handler/mod.rs` handles `AiStartChat` by opening the AI window, queuing `start_ai_chat`, and sending `AiChatCreated`; it handles `AiFocus` by opening/focusing AI and sending `AiFocusResult`.
- `AiAppendMessage`, `AiSendMessage`, `AiSetSystemPrompt`, `AiSubscribe`, and `AiUnsubscribe` are protocol-shaped and SDK-shaped, but local scan did not prove active app-side handling.
- `tests/sdk/test-acp-sdk.ts` mostly validates SDK message shapes by capturing stdout.
- `tests/protocol_ai_parts.rs` validates serde round-trip for `parts`.
- `tests/smoke/test-ai-start-chat.ts` covers `aiStartChat` and `aiFocus` at smoke-script level.

## Bundle Map

The attached bundle is a symbol-window packx bundle generated around the `ai*` SDK/protocol/runtime symbols. It includes focused extracts from:

- `scripts/kit-sdk.ts`
- `src/protocol/message/variants/ai.rs`
- `src/protocol/types/ai.rs`
- `src/protocol/message/constructors/prompts.rs`
- `src/ai/sdk_handlers.rs`
- `src/execute_script/mod.rs`
- `src/main_sections/prompt_messages.rs`
- `src/prompt_handler/mod.rs`
- `src/ai/window/chat.rs`
- `src/ai/window/window_api.rs`
- `src/ai/window/types.rs`
- `src/ai/window/render_root.rs`
- `src/ai/model.rs`
- `src/ai/storage.rs`
- `src/ai/acp/history.rs`
- `src/ai/acp/thread.rs`
- `tests/sdk/test-acp-sdk.ts`
- `tests/sdk/test-ai-context-parts.ts`
- `tests/protocol_ai_parts.rs`
- `tests/smoke/test-ai-start-chat.ts`

Process and domain context is included in this prompt because the focused bundle intentionally excludes full `AGENTS.md`, `CLAUDE.md`, skills, and full `lat.md` pages to keep the bundle readable.

## Required Output Shape

Return this exact structure:

```markdown
## 030 ACP Chat SDK APIs

### Executive Summary

### What Users Can Do

### Core Concepts

### Entry Points

### User Workflows

### Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|

### State Machine

### Visual And Focus States

### Keystrokes And Commands

### Actions And Menus

### Automation And Protocol Surface

### Data, Storage, And Privacy Boundaries

### Error, Empty, Loading, And Disabled States

### Code Ownership

### Invariants And Regression Risks

### Verification Recipes

### Agent Notes

### Related Features

### Open Questions And Gaps
```

## Specific Questions To Answer

1. Which APIs are currently fully request/response implemented end-to-end?
2. Which APIs are storage-backed and do not need the AI/ACP window open?
3. Which APIs need the UI thread and what user-visible window/focus changes do they cause?
4. Which SDK/protocol APIs are declared but not proven handled by the app?
5. How do `parts` and `imagePath` flow into runtime state and persistence?
6. How does `noResponse` change `aiStartChat` behavior?
7. What does `aiIsOpen` actually mean: AI window handle, ACP embedded state, detached ACP, or something else?
8. What are the data boundaries for `aiListChats`, `aiGetConversation`, and `aiDeleteChat`?
9. Are `aiOn`/subscriptions real live streaming subscriptions or only SDK-local handler bookkeeping plus unproven protocol shapes?
10. What exact verification recipes should an agent run to prove the implemented pieces without screenshots?
