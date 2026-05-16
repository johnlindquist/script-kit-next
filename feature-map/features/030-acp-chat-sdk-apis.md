# 030 ACP Chat SDK APIs

This chapter maps the script-facing `ai*` SDK APIs for ACP/AI Chat state, storage reads, chat creation, focus, streaming status, deletion, and subscription gaps.

Raw Oracle reference: [answer](../raw-oracle/030-acp-chat-sdk-apis/answer.md), [prompt](../raw-oracle/030-acp-chat-sdk-apis/prompt.md), [bundle map](../raw-oracle/030-acp-chat-sdk-apis/bundle-map.md), [full log](../raw-oracle/030-acp-chat-sdk-apis/output.log), [session metadata](../raw-oracle/030-acp-chat-sdk-apis/session.json).

## Executive Summary

Feature 030 covers:

- `aiIsOpen()`.
- `aiGetActiveChat()`.
- `aiListChats(limit?, includeDeleted?)`.
- `aiGetConversation(chatId?, limit?)`.
- `aiStartChat(message, options?)`.
- `aiAppendMessage(chatId, content, role)`.
- `aiSendMessage(chatId, content, imagePath?, parts?)`.
- `aiSetSystemPrompt(chatId, prompt)`.
- `aiFocus()`.
- `aiGetStreamingStatus(chatId?)`.
- `aiDeleteChat(chatId, permanent?)`.
- `aiOn(eventType, handler, chatId?)`.

These APIs are script-facing TypeScript globals in `scripts/kit-sdk.ts`. They send JSON protocol messages to the Rust/GPUI app over the script protocol. Rust protocol shapes live in `src/protocol/message/variants/ai.rs`, with shared result data types in `src/protocol/types/ai.rs`.

The currently proven end-to-end APIs are `aiIsOpen`, `aiGetActiveChat`, `aiListChats`, `aiGetConversation`, `aiStartChat`, `aiFocus`, `aiGetStreamingStatus`, `aiDeleteChat`, and scoped ACP live subscriptions through `aiOn`.

Storage-backed APIs that do not need the AI window open are `aiGetActiveChat`, `aiListChats`, `aiGetConversation`, and `aiDeleteChat`. `aiGetStreamingStatus` reads a global SDK-visible streaming snapshot and also does not open the window.

UI-thread APIs are `aiStartChat` and `aiFocus`. `aiStartChat` opens/focuses the AI window, creates a script-sourced chat, saves optional system and user messages, attaches optional image/context parts, and optionally starts streaming. `aiFocus` opens/focuses the AI window and reports whether it was already open.

Declared but not proven handled by the app: `aiAppendMessage`, `aiSendMessage`, and `aiSetSystemPrompt`. `aiSubscribe`, `aiUnsubscribe`, and pushed events now have app-side ACP thread routing proof.

## What Users Can Do

| Capability | Entry | Current result |
|---|---|---|
| Check whether the AI window is open. | `await aiIsOpen()` | Returns `{ isOpen, activeChatId? }`. |
| Read active or fallback chat metadata. | `await aiGetActiveChat()` | Returns active chat info, most recent stored chat, or `null`. |
| List stored chats. | `await aiListChats(limit, includeDeleted)` | Returns `AiChatInfo[]`; SDK drops protocol `totalCount`. |
| Read stored messages. | `await aiGetConversation(chatId, limit)` | Returns `AiMessageInfo[]`; SDK drops protocol `chatId` and `hasMore`. |
| Create a new chat. | `await aiStartChat(message, options)` | Opens/focuses AI window, creates chat, optionally streams. |
| Focus AI Chat. | `await aiFocus()` | Opens/focuses AI window and returns `{ wasOpen }`. |
| Query streaming snapshot. | `await aiGetStreamingStatus(chatId?)` | Returns streaming state from global snapshot. |
| Delete a chat. | `await aiDeleteChat(chatId, permanent)` | Soft or permanent delete; SDK resolves void and drops protocol error. |
| Append message without AI response. | `await aiAppendMessage(...)` | SDK/protocol shaped, app-side handling unproven. |
| Send message to existing chat. | `await aiSendMessage(...)` | SDK/protocol shaped, app-side handling unproven. |
| Set system prompt after creation. | `await aiSetSystemPrompt(...)` | SDK/protocol shaped, app-side handling unproven. |
| Subscribe to events. | `await aiOn(...)` | Registers a script-owned subscription and receives scoped ACP thread events. |

## Core Concepts

### Layered Contract

Do not collapse these layers:

- SDK globals and TypeScript result types.
- Rust protocol variants and request ids.
- Direct storage/window-state handlers.
- UI-thread prompt-message handlers.
- AI window persistence, context preparation, and streaming.
- SDK-local event handler bookkeeping.

Most false confidence in this surface comes from seeing SDK/protocol declarations and assuming runtime handling exists.

### AI Window Meaning

`aiIsOpen()` checks the Script Kit AI window handle through `is_ai_window_open()`. If open, it includes the globally published active chat id. It does not mean “chat storage exists,” and it is not proof of detached ACP state.

### Storage-Backed Reads

`aiGetActiveChat`, `aiListChats`, `aiGetConversation`, and `aiDeleteChat` are handled directly in `src/ai/sdk_handlers.rs`. These do not need the AI window to be open.

`aiGetActiveChat` first tries active window state, then falls back to the most recently updated stored chat. That means “active” can mean “most recent stored” when no window state exists.

### UI-Thread Operations

`aiStartChat` and `aiFocus` cross into prompt/UI handling. `src/execute_script/mod.rs` maps only these two AI SDK messages into `PromptMessage`, and `src/main_sections/prompt_messages.rs` only defines prompt-message variants for them.

### ACP vs `chat()`

`chat()` is the older inline main-window chat prompt where the script owns generation. The `ai*` APIs control the built-in AI/ACP Chat surface with app-owned providers, history, window focus, storage, and streaming state.

## Entry Points

| Entry | Payload | Response | Runtime status |
|---|---|---|---|
| `aiIsOpen()` | `{ type:"aiIsOpen", requestId }` | `aiIsOpenResult` | Direct handler implemented. |
| `aiGetActiveChat()` | `{ type:"aiGetActiveChat", requestId }` | `aiActiveChatResult` | Direct handler implemented. |
| `aiListChats(limit?, includeDeleted?)` | `{ type:"aiListChats", requestId, limit, includeDeleted }` | `aiChatListResult` | Direct handler implemented. |
| `aiGetConversation(chatId?, limit?)` | `{ type:"aiGetConversation", requestId, chatId, limit }` | `aiConversationResult` | Direct handler implemented. |
| `aiStartChat(message, options?)` | `{ type:"aiStartChat", requestId, message, systemPrompt, image, modelId, noResponse, parts }` | `aiChatCreated` | UI-thread handler implemented. |
| `aiAppendMessage(chatId, content, role)` | `{ type:"aiAppendMessage", requestId, chatId, content, role }` | `aiMessageAppended` | App-side handling unproven. |
| `aiSendMessage(chatId, content, imagePath?, parts?)` | `{ type:"aiSendMessage", requestId, chatId, content, image, parts }` | `aiMessageSent` | App-side handling unproven. |
| `aiSetSystemPrompt(chatId, prompt)` | `{ type:"aiSetSystemPrompt", requestId, chatId, prompt }` | `aiSystemPromptSet` | App-side handling unproven. |
| `aiFocus()` | `{ type:"aiFocus", requestId }` | `aiFocusResult` | UI-thread handler implemented. |
| `aiGetStreamingStatus(chatId?)` | `{ type:"aiGetStreamingStatus", requestId, chatId }` | `aiStreamingStatusResult` | Direct handler implemented. |
| `aiDeleteChat(chatId, permanent?)` | `{ type:"aiDeleteChat", requestId, chatId, permanent }` | `aiChatDeleted` | Direct handler implemented. |
| `aiOn(eventType, handler, chatId?)` | `{ type:"aiSubscribe", requestId, events, chatId }` | `aiSubscribed` | Subscription handling implemented for ACP thread events. |

## User Workflows

### Check Whether AI Chat Is Open

A script calls:

```ts
const state = await aiIsOpen()
```

The SDK sends `aiIsOpen`. Rust handles it in `handle_ai_is_open`, checks `is_ai_window_open()`, and includes `activeChatId` only when the window is open.

This is a window-open check. It does not prove a stored conversation exists.

### Inspect Current Or Recent Chat

A script calls:

```ts
const chat = await aiGetActiveChat()
```

The direct handler first tries the published active chat id from window state. If none exists, it falls back to the first result from `storage::get_all_chats()`. The result can therefore be the most recent stored chat even when no chat window is open.

### List Stored Chats

A script calls:

```ts
const chats = await aiListChats(25, true)
```

The handler reads all chats from storage, optionally extends with deleted chats, computes `totalCount`, applies `limit`, maps each chat to `AiChatInfo`, and returns `aiChatListResult`.

The TypeScript SDK returns only `chats`, so scripts cannot see `totalCount` even though the protocol includes it.

### Read Conversation Messages

A script calls:

```ts
const messages = await aiGetConversation(chatId, 100)
```

If `chatId` is provided, Rust parses it. Invalid ids return an empty result instead of crashing. If `chatId` is omitted, the handler tries active chat id, then most recent stored chat. With a limit, it calls `storage::get_recent_messages`; without one, it calls `storage::get_chat_messages`.

The SDK returns only message objects, so scripts cannot see protocol `chatId` or `hasMore`.

### Start A New Chat

A script calls:

```ts
await aiStartChat("Review this", {
  systemPrompt: "You are a reviewer.",
  imagePath: "/tmp/screenshot.png",
  modelId: "claude-3-5-sonnet-20241022",
  noResponse: false,
  parts: [
    { kind: "resourceUri", uri: "kit://context?profile=minimal", label: "Current Context" },
    { kind: "filePath", path: "/tmp/example.rs", label: "example.rs" },
  ],
})
```

The SDK reads `imagePath` from disk and base64-encodes it. If the read fails, it logs and sends the chat request without image data.

Rust maps `AiStartChat` to `PromptMessage::AiStartChat`. The prompt handler opens/focuses the AI window, pre-generates a real `ChatId`, resolves provider metadata from `modelId` when possible, converts protocol context parts into runtime `AiContextPart`, queues `start_ai_chat`, and sends `AiChatCreated`.

The AI window later consumes `AiCommand::StartChat`. `AiApp::handle_start_chat` creates the stored chat, saves optional system prompt, saves the user message, attaches optional image data, publishes active chat id, updates preview/count caches, clears the composer, and starts streaming only when `submit` is true.

`noResponse: true` sets `submit` false. The chat and message are created, but streaming does not start.

### Focus AI Chat

A script calls:

```ts
const result = await aiFocus()
```

The handler records whether the AI window was already open, opens/focuses it, and returns `AiFocusResult`. The SDK exposes only `{ wasOpen }`, dropping the protocol `success` field.

### Query Streaming Status

A script calls:

```ts
const streaming = await aiGetStreamingStatus(chatId)
```

The direct handler reads `get_streaming_snapshot()`. If `chatId` is provided, it only reports true when the requested id matches the active streaming snapshot.

This is not a subscription and not storage. It is a global SDK-visible snapshot.

### Delete A Chat

A script calls:

```ts
await aiDeleteChat(chatId, false)
await aiDeleteChat(chatId, true)
```

The first call soft-deletes through `storage::delete_chat`. The second permanently deletes through `storage::delete_chat_permanently`.

Invalid ids return protocol failure, but the current SDK return type resolves void and ignores success/error details.

### Subscribe To AI Events

A script calls:

```ts
const unsubscribe = await aiOn("streamChunk", handler, chatId)
```

The SDK sends `aiSubscribe`, stores handlers after `aiSubscribed`, and returns an unsubscribe function that sends `aiUnsubscribe` with the captured `subscriptionId`. `_handleAiEvent` dispatches `aiStreamChunk`, `aiStreamComplete`, `aiNewMessage`, and `aiError` by that subscription id.

The app-side subscription manager now lives in `src/ai/subscriptions.rs`, and ACP thread events fan out from `src/ai/acp/thread.rs`.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Check window open state. | `aiIsOpen()` | No UI change. | None. | SDK -> `AiIsOpen` -> `handle_ai_is_open`. | `{ isOpen, activeChatId? }`. | Direct handler/source test. |
| Read active/fallback chat. | `aiGetActiveChat()` | No UI change. | None. | Direct handler -> active id or storage fallback. | `AiChatInfo | null`. | Storage/direct handler proof. |
| List chats. | `aiListChats()` | No UI change. | None. | Direct handler -> storage read. | `AiChatInfo[]`. | Handler and SDK shape tests. |
| Read messages. | `aiGetConversation()` | No UI change. | None. | Direct handler -> storage messages. | `AiMessageInfo[]`. | Handler and protocol tests. |
| Start chat and stream. | `aiStartChat(..., { noResponse:false })` | Opens/focuses AI window. | None. | SDK -> prompt handler -> `start_ai_chat` -> `AiApp::handle_start_chat`. | Stored chat/message and streaming. | Smoke script plus state/storage proof. |
| Start chat without response. | `aiStartChat(..., { noResponse:true })` | Opens/focuses AI window. | None. | Same path, `submit:false`. | Stored chat/message, no streaming. | `streamingStarted:false`, storage proof. |
| Focus AI window. | `aiFocus()` | Opens/focuses AI window. | None. | SDK -> prompt handler -> `open_ai_window`. | `{ wasOpen }`. | Smoke script and window state proof. |
| Query streaming. | `aiGetStreamingStatus()` | No UI change. | None. | Direct handler -> `get_streaming_snapshot`. | Streaming snapshot. | Direct handler/state proof. |
| Delete chat. | `aiDeleteChat()` | No UI change. | None. | Direct handler -> soft/permanent storage delete. | Protocol success/error; SDK void. | Storage proof. |
| Append message. | `aiAppendMessage()` | Unproven. | None. | SDK/protocol only in captured context. | Runtime support unproven. | Negative timeout/unhandled proof needed. |
| Send existing-chat message. | `aiSendMessage()` | Unproven. | None. | SDK/protocol only in captured context. | Runtime support unproven. | Negative timeout/unhandled proof needed. |
| Set system prompt. | `aiSetSystemPrompt()` | Unproven. | None. | SDK/protocol only in captured context. | Runtime support unproven. | Negative timeout/unhandled proof needed. |
| Subscribe to events. | `aiOn()` | No UI change. | Optional ACP thread id filter. | SDK -> script-reader subscription registry -> ACP thread event fanout. | Scoped pushed events. | Source contract + registry proof. |

## State Machine

### AI Window State

| State | Trigger | Transition |
|---|---|---|
| Closed. | `aiIsOpen()`. | Returns `isOpen:false`. |
| Closed. | `aiFocus()`. | Opens/focuses AI window, returns `wasOpen:false`. |
| Closed. | `aiStartChat()`. | Opens/focuses AI window, queues chat creation. |
| Open. | `aiIsOpen()`. | Returns `isOpen:true` and maybe active chat id. |
| Open. | `aiFocus()`. | Refocuses window, returns `wasOpen:true`. |
| Open. | `aiStartChat()`. | Creates/selects a new script-sourced chat. |

### Chat Creation

| State | Trigger | Transition |
|---|---|---|
| SDK call. | `aiStartChat(message, options)`. | Request id generated; image path read; message sent. |
| Protocol dispatch. | `AiStartChat`. | `execute_script` maps to `PromptMessage::AiStartChat`. |
| UI handler. | Prompt message. | AI window opens/focuses; `ChatId` generated; `start_ai_chat` queued. |
| Immediate SDK response. | Handler sends `AiChatCreated`. | Script receives chat id/title/model/provider/streaming flag. |
| AI command consumed. | `AiCommand::StartChat`. | Storage chat and messages are created. |
| Submit true. | `noResponse` absent or false. | Streaming response starts. |
| Submit false. | `noResponse:true`. | Chat remains created without streaming. |

### Streaming Snapshot

| State | Trigger | Transition |
|---|---|---|
| Idle. | No active stream. | `aiGetStreamingStatus()` returns `isStreaming:false`. |
| Streaming. | AI window publishes streaming state. | Snapshot carries chat id and optional partial content. |
| Chat id filter mismatch. | `aiGetStreamingStatus(otherChatId)`. | Returns `isStreaming:false`. |
| Selected chat changes. | AI window clears displayed streaming state. | Snapshot can report idle even if older background task is finishing. |

## Visual And Focus States

| State | How it appears | Proof path |
|---|---|---|
| AI window closed. | No AI window handle. | `aiIsOpen().isOpen === false`; window registry if needed. |
| AI window open. | AI Chat window exists. | `aiIsOpen().isOpen === true`; `aiFocus().wasOpen`. |
| AI window focused. | Window brought forward. | `aiFocus()` plus state/focus proof if runtime UI proof is needed. |
| New chat selected. | Chat appears in AI window. | `aiStartChat()` result plus storage/active chat id. |
| Streaming active. | Assistant response in progress. | `aiGetStreamingStatus()` or AI state receipt. |
| Storage read only. | No visible change. | Direct handler result; assert no window opened. |

## Keystrokes And Commands

These APIs are script commands, not user keystrokes. They may be called by scripts assigned to shortcuts, but shortcut assignment belongs to the launcher/config feature, not this SDK API surface.

| Command | Proof rule |
|---|---|
| `aiFocus` | Prove window open/focus result, not just message shape. |
| `aiStartChat` | Prove `AiChatCreated`, stored chat/message, and streaming/no-stream branch. |
| `aiListChats` | Prove storage result and SDK return truncation behavior. |
| `aiGetConversation` | Prove message content return and invalid-id behavior. |
| `aiOn` | Prove app-side subscription id routing, chat scoping, unsubscribe, and cleanup. |

## Actions And Menus

No Actions dialog or menu ownership is proven for this API cluster. `aiStartChat` and `aiFocus` are protocol/prompt-message routes, not menu actions. Any script that exposes these calls through a launcher row or action belongs to that script/action feature, while the API behavior belongs here.

## Automation And Protocol Surface

| Surface | Status | Notes |
|---|---|---|
| Request ids. | Implemented in protocol shapes. | `request_id()` recognizes AI request/response variants. |
| `AiContextPartInput`. | Implemented. | `resourceUri` and `filePath`; optional parts default empty and skip serialization when empty. |
| Direct handlers. | Implemented for read/delete/status APIs. | `src/ai/sdk_handlers.rs`. |
| UI prompt handlers. | Implemented for `aiStartChat` and `aiFocus`. | `src/prompt_handler/mod.rs`. |
| `AiCommand::StartChat`. | Implemented. | Consumed in AI window render root and handled by `AiApp::handle_start_chat`. |
| `aiAppendMessage`. | Unproven. | Protocol/SDK shapes exist; no handler proven. |
| `aiSendMessage`. | Unproven. | Protocol/SDK shapes exist; no handler proven. |
| `aiSetSystemPrompt`. | Unproven. | Protocol/SDK shapes exist; no handler proven. |
| `aiSubscribe` / `aiUnsubscribe`. | Implemented for ACP thread events. | Script-reader registry owns response sender and cleanup. |
| Pushed events. | Implemented for ACP thread events. | `AcpThread` produces message/chunk/complete/error events. |

## Data, Storage, And Privacy Boundaries

`AiChatInfo` exposes chat id, title, model id, provider, timestamps, deletion state, optional preview, and message count. `chat_to_info` currently sets preview to `None`.

`AiMessageInfo` exposes full message content for user, assistant, and system roles, with timestamp and optional token count. It does not expose image attachments or context part receipts.

`aiStartChat` saves a provided system prompt as a stored system message before saving the user message. Post-creation `aiSetSystemPrompt` mutation is not proven.

`imagePath` is read by the SDK from local disk and sent as base64 image data. Runtime attaches it as a PNG image payload. If file read fails, the SDK logs and continues without image.

`parts` are converted from protocol `AiContextPartInput` into runtime `AiContextPart`, then resolved through outbound message preparation. Blocked context resolution saves cleaned authored content rather than unresolved injected context.

`aiDeleteChat(false)` soft-deletes; `aiDeleteChat(true)` permanently deletes. The SDK hides protocol failure details, so scripts cannot currently observe deletion errors through the typed return value.

## Error, Empty, Loading, And Disabled States

| API | Failure or ambiguous state |
|---|---|
| `aiIsOpen()` | Closed window returns no active chat id even if storage has chats. |
| `aiGetActiveChat()` | With no active window state, fallback is most recent stored chat. |
| `aiListChats()` | Storage failure returns empty list; SDK drops `totalCount`. |
| `aiGetConversation()` | Invalid chat id returns empty messages; SDK drops `chatId` and `hasMore`. |
| `aiStartChat()` | Open-window failure returns empty/default `AiChatCreated` so SDK does not hang. |
| `aiStartChat()` | Response can be sent before full persistence success is proven. |
| `aiFocus()` | Protocol has `success`, but SDK returns only `wasOpen`. |
| `aiGetStreamingStatus()` | Snapshot can report idle after selected chat changes even if older background work is finishing. |
| `aiDeleteChat()` | Protocol has success/error, but SDK resolves void. |
| `aiAppendMessage()` | Runtime handling unproven; may hang outside test/autosubmit contexts. |
| `aiSendMessage()` | Runtime handling unproven; may hang outside test/autosubmit contexts. |
| `aiSetSystemPrompt()` | Runtime handling unproven; may hang outside test/autosubmit contexts. |
| `aiOn()` | Live ACP thread event production/routing is implemented; existing-chat mutation remains separate. |

## Code Ownership

| Area | Owner skill | Files and references |
|---|---|---|
| SDK globals, types, pending callbacks, image read, subscription map. | `sdk-script-execution` | `scripts/kit-sdk.ts`. |
| Protocol variants and result structs. | `protocol-automation` | `src/protocol/message/variants/ai.rs`, `src/protocol/types/ai.rs`, `src/protocol/message/constructors/prompts.rs`. |
| Direct storage/window-state handlers. | `sdk-script-execution`, `storage-cache-security` | `src/ai/sdk_handlers.rs`. |
| Script-to-UI dispatch. | `sdk-script-execution`, `prompt-runtime` | `src/execute_script/mod.rs`, `src/main_sections/prompt_messages.rs`. |
| UI-thread open/focus/start behavior. | `acp-chat-core` | `src/prompt_handler/mod.rs`, `src/ai/window/window_api.rs`. |
| Chat persistence and streaming start. | `acp-chat-core`, `storage-cache-security` | `src/ai/window/chat.rs`, `src/ai/window/types.rs`, `src/ai/storage.rs`. |
| Context part handoff. | `acp-context-composer`, `mcp-context-resources` | `src/ai/window/chat.rs`, `src/protocol/types/ai.rs`. |
| Verification. | `agentic-testing`, `protocol-automation` | `tests/sdk/test-acp-sdk.ts`, `tests/sdk/test-ai-context-parts.ts`, `tests/protocol_ai_parts.rs`, `tests/smoke/test-ai-start-chat.ts`. |

## Invariants And Regression Risks

- Every request/response API must carry and preserve `requestId`.
- `parts` must stay optional and omitted when empty for wire compatibility.
- `resourceUri` and `filePath` context parts must preserve order.
- `aiStartChat({ noResponse:true })` must create chat/message without starting streaming.
- `aiStartChat({ noResponse:false })` must start streaming after persistence.
- `aiIsOpen()` must remain a window-open check, not a storage-exists check.
- Storage-backed APIs must not open the AI window.
- Soft delete and permanent delete must remain distinct.
- SDK tests that only capture stdout are not runtime support proof.
- Adding protocol variants without dispatch creates APIs that look typed but can hang at runtime.
- SDK return types currently drop useful protocol fields and errors.
- `aiStartChat` can report created before storage completion is fully proven.
- Subscription APIs are high-risk because SDK-local dispatch makes them appear complete.

## Verification Recipes

### Protocol Serde

Run:

```bash
cargo test protocol_ai_parts
```

Expected proof:

- AI messages without `parts` deserialize.
- Non-empty `parts` serialize.
- Mixed resource/file parts round-trip.
- `systemPrompt`, `image`, `modelId`, and `noResponse` fields survive.

### SDK Message Shapes

Run:

```bash
bun tests/sdk/test-acp-sdk.ts
bun tests/sdk/test-ai-context-parts.ts
```

Expected proof:

- SDK sends correct JSON shapes for declared APIs.
- `aiStartChat` and `aiSendMessage` include parts.
- Omitted parts do not create non-empty payloads.
- `aiOn` sends `aiSubscribe`.

These tests do not prove app runtime support.

### UI-Thread Implemented APIs

Run:

```bash
echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-ai-start-chat.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```

Expected proof:

- `aiStartChat` is not an unhandled message.
- `aiStartChat` returns chat id/title/model/provider/streamingStarted.
- `aiFocus` returns `wasOpen`.

### Direct Handler APIs

Use scripts or protocol fixtures that call:

- `aiIsOpen`.
- `aiListChats`.
- `aiGetActiveChat`.
- `aiGetConversation`.
- `aiGetStreamingStatus`.
- `aiDeleteChat`.

Expected proof:

- Storage-backed reads/deletes return JSON responses.
- Reads/deletes do not open the AI window.
- Invalid ids do not crash.
- Soft and permanent delete differ.

### Unproven API Negative Tests

Run scripts that call only:

- `aiAppendMessage`.
- `aiSendMessage`.
- `aiSetSystemPrompt`.
- `aiOn`.

Expected proof target:

- Either locate a real app-side response path not present in the bundle, or confirm timeout/unhandled behavior.
- Do not accept SDK stdout-capture tests as support proof.

## Agent Notes

- Do not assume an SDK global is runtime-supported just because TypeScript and Rust protocol variants exist.
- To verify implementation, prove the app sends the expected response, not only that the SDK writes a request.
- If `aiAppendMessage`, `aiSendMessage`, `aiSetSystemPrompt`, or `aiOn` hangs, inspect `src/execute_script/mod.rs`, `src/main_sections/prompt_messages.rs`, and `src/prompt_handler/mod.rs`.
- This belongs to `sdk-script-execution` for global API shape, `acp-chat-core` for AI window side effects, and `storage-cache-security` for persisted chat/message data.
- This does not belong to the inline `chat()` prompt, except for documentation contrast.
- Screenshots are not needed for storage/query APIs; use protocol responses, state, and storage receipts first.

## Related Features

- [003 Agent Chat Context Composer](./003-agent-chat-context.md).
- [004 MCP Context Resources / SDK / Protocol Automation](./004-mcp-sdk-protocol.md).
- [010 Root Unified Search ACP History](./010-root-acp-history.md).
- [016 Prompt Runtime Core](./016-prompt-runtime-core.md).
- [025 System Feedback and Prompt Control APIs](./025-system-feedback-and-prompt-control.md).

## Open Questions And Gaps

- `aiAppendMessage` is declared in SDK and protocol, but app-side handling is not proven.
- `aiSendMessage` is declared in SDK and protocol, but app-side handling is not proven.
- `aiSetSystemPrompt` is declared in SDK and protocol, but app-side handling is not proven.
- SDK return types discard protocol metadata and errors.
- `aiStartChat` reports created before storage completion is fully proven.
- `aiIsOpen` appears to mean the Script Kit AI window is open, not detached ACP state or storage activity.
- `aiGetActiveChat` may return most recent stored chat when no window-active chat exists.
- Runtime image attachment is treated as PNG even though SDK `imagePath` does not validate MIME type.
