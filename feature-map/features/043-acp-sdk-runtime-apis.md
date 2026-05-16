# 043 ACP SDK Runtime APIs

This chapter maps the declared-but-unproven ACP/Agent Chat SDK runtime APIs for existing-chat mutation, event subscription, unsubscribe, and pushed AI events.

Raw Oracle reference: [answer](../raw-oracle/043-acp-sdk-runtime-apis/answer.md), [prompt](../raw-oracle/043-acp-sdk-runtime-apis/prompt.md), [bundle map](../raw-oracle/043-acp-sdk-runtime-apis/bundle-map.md), [full log](../raw-oracle/043-acp-sdk-runtime-apis/output.log), [session metadata](../raw-oracle/043-acp-sdk-runtime-apis/session.json).

## Executive Summary

Feature 030 maps the broad `ai*` SDK catalog and proves the implemented APIs for Agent Chat reads, storage listing, conversation reading, new chat creation, focus, streaming-status polling, and deletion.

Feature 043 is the runtime gap map for the remaining declared APIs:

- `aiAppendMessage(chatId, content, role)`.
- `aiSendMessage(chatId, content, imagePath?, parts?)`.
- `aiSetSystemPrompt(chatId, prompt)`.
- `aiOn(eventType, handler, chatId?)`.
- Wire messages `aiSubscribe`, `aiUnsubscribe`, `aiStreamChunk`, `aiStreamComplete`, `aiNewMessage`, and `aiError`.

The core finding is now split: existing-chat mutation remains a runtime gap, while live subscription management is implemented for ACP thread events through a script-owned registry. The direct AI SDK handler still returns no result for mutation messages, and the script-to-prompt bridge only maps `AiStartChat` and `AiFocus` into prompt/UI handling.

Safe wording:

> Scripts can rely on the proven feature 030 APIs for reads, delete, new chat creation, focus, streaming-status polling, and scoped ACP live subscriptions. Scripts cannot safely rely on append/send/system-prompt mutation until runtime handlers and receipts exist.

## Scope

| Surface | Include here | Reason |
|---|---:|---|
| `aiAppendMessage(chatId, content, role)` | Yes | Declared in SDK and Rust protocol; app-side runtime mutation is unproven. |
| `aiSendMessage(chatId, content, imagePath?, parts?)` | Yes | Declared and shape-tested; existing-chat send/stream runtime is unproven. |
| `aiSetSystemPrompt(chatId, prompt)` | Yes | Declared; post-creation system-prompt mutation is unproven. |
| `aiOn(eventType, handler, chatId?)` | Yes | SDK-local bookkeeping now pairs with an app-side subscription registry. |
| `aiSubscribe` / `aiUnsubscribe` | Yes | Protocol messages register/remove exact subscriptions; unsubscribe now carries subscription id. |
| `aiStreamChunk`, `aiStreamComplete`, `aiNewMessage`, `aiError` | Yes | ACP thread events fan out through scoped subscription routing. |
| `aiIsOpen`, `aiGetActiveChat`, `aiListChats`, `aiGetConversation`, `aiStartChat`, `aiFocus`, `aiGetStreamingStatus`, `aiDeleteChat` | Contrast only | Feature 030 already documents these proven APIs. |

## Current Safe Capabilities

Script authors can safely rely on the proven feature 030 APIs:

- Read AI window/open state with `aiIsOpen()`.
- Read active or recent chat metadata with `aiGetActiveChat()`.
- List stored chats with `aiListChats()`.
- Read stored messages with `aiGetConversation()`.
- Create a new chat with `aiStartChat()`.
- Focus/open Agent Chat with `aiFocus()`.
- Poll streaming state with `aiGetStreamingStatus()`.
- Delete chats with `aiDeleteChat()`.

For this chapter's APIs, scripts can safely rely only on SDK message-shape emission and Rust protocol shape support. That is not proof of app-side effects.

| Claim | Safe today | Wording |
|---|---:|---|
| `aiAppendMessage` exists as a TypeScript global. | Yes | It sends an `aiAppendMessage` payload and waits for `aiMessageAppended`. |
| `aiAppendMessage` appends a stored message. | No | Runtime mutation is unproven. |
| `aiSendMessage` can serialize text, image data, and parts. | Yes | SDK/protocol shape exists and parts have serde coverage. |
| `aiSendMessage` sends to an existing chat and starts streaming. | No | Existing-chat send/stream runtime is unproven. |
| `aiSetSystemPrompt` sends a protocol message. | Yes | SDK and Rust protocol shapes exist. |
| `aiSetSystemPrompt` changes a chat's system prompt. | No | No runtime handler is proven. |
| `aiOn` sends `aiSubscribe` and has local handler bookkeeping. | Yes | SDK map and dispatch helper exist. |
| `aiOn` receives live Agent Chat events. | Yes | ACP thread events are routed through the app-side subscription registry. |
| The returned unsubscribe removes the local SDK handler. | Yes | The SDK deletes from its local subscription map. |
| The returned unsubscribe unregisters app-side delivery. | Yes | `aiUnsubscribe` carries the captured subscription id and removes that app-side subscription. |

## Layered Contract

Do not collapse these layers:

1. TypeScript global API.
2. SDK request/response/event interfaces.
3. SDK pending-response and local-subscription bookkeeping.
4. Rust protocol request/response/event variants.
5. Direct AI SDK handler.
6. Script-to-prompt bridge.
7. Prompt/UI runtime handler.
8. ACP thread/storage mutation.
9. Subscription registry.
10. Pushed event producer.
11. Runtime receipt or test proving the behavior.

Most false confidence on this surface comes from seeing layers 1-4 and assuming layers 5-11 exist.

## Runtime Support Matrix

| Message/API | TS declared | Rust protocol | SDK sends/dispatches | Direct handler | Prompt bridge | ACP runtime | Subscription manager | Event producer | Current status |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| `aiAppendMessage` | Yes | Yes | Yes | No | No | Unproven | n/a | n/a | Shape only; runtime unproven. |
| `aiMessageAppended` | Yes | Yes | SDK waits | No producer proven | No | Unproven | n/a | n/a | Response shape only. |
| `aiSendMessage` | Yes | Yes | Yes | No | No | Unproven | n/a | n/a | Shape only; existing-chat send unproven. |
| `aiMessageSent` | Yes | Yes | SDK waits | No producer proven | No | Unproven | n/a | n/a | Response shape only. |
| `aiSetSystemPrompt` | Yes | Yes | Yes | No | No | Unproven | n/a | n/a | Shape only; post-creation mutation unproven. |
| `aiSystemPromptSet` | Yes | Yes | SDK waits | No producer proven | No | Unproven | n/a | n/a | Response shape only. |
| `aiSubscribe` | Yes | Yes | Yes | Script reader | No | Proven for ACP thread events | Proven | n/a | Registers script-owned subscription. |
| `aiSubscribed` | Yes | Yes | SDK waits | Script reader producer | No | Proven | Proven | n/a | Returns generated subscription id. |
| `aiUnsubscribe` | Yes | Yes | Yes, with subscription id | Script reader | No | Proven | Proven | n/a | Removes exact subscription. |
| `aiUnsubscribed` | Yes | Yes | Shape exists | Script reader producer | No | Proven | Proven | n/a | Acknowledges removal request. |
| `aiStreamChunk` | Yes | Yes | SDK dispatches by subscription id | n/a | n/a | Proven for ACP thread chunks | Proven | Proven | Scoped pushed event. |
| `aiStreamComplete` | Yes | Yes | SDK dispatches by subscription id | n/a | n/a | Proven for ACP turn completion | Proven | Proven | Scoped pushed event. |
| `aiNewMessage` | Yes | Yes | SDK dispatches by subscription id | n/a | n/a | Proven for ACP user/final assistant messages | Proven | Proven | Scoped pushed event. |
| `aiError` | Yes | Yes | SDK dispatches by subscription id | n/a | n/a | Proven for ACP failures | Proven | Proven | Scoped pushed event. |

## API Details

### `aiAppendMessage`

`aiAppendMessage(chatId, content, role)` is declared in `scripts/kit-sdk.ts` and sends `{ type:"aiAppendMessage", requestId, chatId, content, role }`.

The SDK registers a pending resolver for `aiMessageAppended` and resolves with the returned message id if such a response arrives.

Rust protocol variants exist for `AiAppendMessage` and `AiMessageAppended` in `src/protocol/message/variants/ai.rs`.

Runtime status:

- `src/ai/sdk_handlers.rs` does not directly handle `Message::AiAppendMessage`.
- `src/execute_script/mod.rs` does not map it into a dedicated prompt message.
- `src/main_sections/prompt_messages.rs` has no prompt-message variant for append.
- No ACP storage/thread mutation path or `aiMessageAppended` producer is proven.

Do not claim this persists a user, assistant, or system message. A real script may wait indefinitely unless a test harness or mock resolves the pending callback.

### `aiSendMessage`

`aiSendMessage(chatId, content, imagePath?, parts?)` is declared in `scripts/kit-sdk.ts` and sends `{ type:"aiSendMessage", requestId, chatId, content, image, parts }`.

The SDK reads `imagePath` from disk, base64-encodes it when possible, logs read failures, and still sends the request without image data on failure. The expected response is `aiMessageSent` with `userMessageId`, `chatId`, and `streamingStarted`.

Rust protocol variants exist for `AiSendMessage` and `AiMessageSent`. Protocol tests cover `aiSendMessage` deserialization without parts and `filePath` part round-trip behavior.

Runtime status:

- `src/ai/sdk_handlers.rs` does not directly handle `Message::AiSendMessage`.
- `src/execute_script/mod.rs` does not map it into prompt/UI handling.
- No existing-chat user-message persistence, context resolution, assistant streaming, or `aiMessageSent` producer is proven.

Do not claim `aiSendMessage` appends to an existing chat or starts streaming. Today the safe claim is that the SDK/protocol can shape the request.

### `aiSetSystemPrompt`

`aiSetSystemPrompt(chatId, prompt)` is declared in `scripts/kit-sdk.ts` and sends `{ type:"aiSetSystemPrompt", requestId, chatId, prompt }`.

The SDK expects `aiSystemPromptSet` and resolves void when the pending callback fires. Rust protocol variants exist for `AiSetSystemPrompt` and `AiSystemPromptSet`.

Runtime status:

- `src/ai/sdk_handlers.rs` does not directly handle `Message::AiSetSystemPrompt`.
- No prompt-message variant exists for post-creation system prompt mutation.
- The proven `systemPrompt` path is only `aiStartChat(..., { systemPrompt })` at creation time.

Do not claim it rewrites, prepends, replaces, or otherwise mutates a stored system prompt.

### `aiOn` / `aiSubscribe`

`aiOn(eventType, handler, chatId?)` sends an `aiSubscribe` request with requested event types and optional chat id. After `aiSubscribed`, the SDK stores the handler in a local subscription map keyed by `subscriptionId`.

Rust protocol variants exist for `AiSubscribe` and `AiSubscribed`.

Runtime status:

- `src/execute_script/mod.rs` handles `Message::AiSubscribe` before the stateless direct AI SDK handler because subscription registration needs the executing script's response sender.
- `src/ai/subscriptions.rs` allocates `subscriptionId`, stores owner id, event types, optional chat id, and response sender, then returns `aiSubscribed`.
- `src/ai/acp/thread.rs` publishes ACP thread events to matching subscriptions using the ACP `ui_thread_id` as the chat/session id.

Live Agent Chat subscription works for ACP thread events. It does not make the existing-chat mutation APIs complete.

### `aiUnsubscribe`

The unsubscribe function returned by `aiOn` deletes the SDK-local handler by captured `subscriptionId` and sends `{ type:"aiUnsubscribe", requestId, subscriptionId }`.

Rust protocol variants exist for `AiUnsubscribe` and `AiUnsubscribed`.

Runtime status:

- `src/execute_script/mod.rs` handles `Message::AiUnsubscribe` with the same script owner id used for subscription creation.
- `src/ai/subscriptions.rs` removes the exact app-side subscription when the owner id matches and reports `success` plus optional `error`.
- Reader exit calls owner cleanup so script/process exit drains stale subscriptions.

The safe claim is exact subscription cleanup for the executing script, plus SDK-local handler removal.

### Pushed Events

The SDK declares pushed event messages:

- `aiStreamChunk`.
- `aiStreamComplete`.
- `aiNewMessage`.
- `aiError`.

Rust protocol variants exist for those messages. SDK dispatch can call matching local handlers if such messages arrive.

Runtime status:

- `src/ai/acp/thread.rs` emits `aiNewMessage` for submitted user messages and final assistant messages.
- Assistant deltas emit `aiStreamChunk` with delta and accumulated content.
- `TurnFinished` emits `aiStreamComplete` for the latest assistant message.
- `Failed` emits `aiError` for matching error subscriptions.
- SDK dispatch targets `event.subscriptionId` first and keeps chat-id filtering as defense in depth.

Agent Chat emits these events for ACP thread activity; legacy AI-window mutation APIs remain separate.

## Current State Machines

### Existing-Chat Mutation

Current actual lifecycle for `aiAppendMessage`, `aiSendMessage`, and `aiSetSystemPrompt`:

1. Script calls SDK global.
2. SDK creates `requestId`.
3. SDK sends protocol payload.
4. Rust protocol can deserialize the message.
5. Direct handler declines it by returning no handled response.
6. Script execution bridge does not map it into a dedicated prompt message.
7. Message falls through to unhandled handling.
8. No response producer is proven.
9. Script promise may hang outside test/mock contexts.

The chapter should treat this as a runtime gap, not a missing documentation detail.

### Streaming Snapshot Versus Event Stream

`aiGetStreamingStatus(chatId?)` is a proven polling/snapshot API. It reports the current global streaming snapshot and optionally filters by active chat id.

`aiOn("streamChunk", ...)` is a live event stream for ACP thread activity. It does not replace `aiGetStreamingStatus` as a polling snapshot for legacy AI-window state.

### Subscription Creation

Current runtime lifecycle:

1. Script calls `aiOn(eventType, handler, chatId?)`.
2. SDK sends `aiSubscribe`.
3. `src/execute_script/mod.rs` registers the request with `src/ai/subscriptions.rs` using the current script owner and response sender.
4. The app returns `aiSubscribed` with a generated `subscriptionId`.
5. SDK stores the handler under that id and returns an unsubscribe closure.

### Event Delivery

Current runtime lifecycle:

1. `AcpThread` receives a typed ACP event.
2. The thread publishes the matching SDK event through `src/ai/subscriptions.rs`.
3. The registry filters by event type and optional chat/session id, then sends a pushed message with the target `subscriptionId`.
4. SDK dispatch looks up the handler by `subscriptionId` and checks chat id defensively.

### Unsubscribe And Cleanup

Current SDK-local lifecycle:

1. Returned unsubscribe function deletes the local handler.
2. SDK sends `aiUnsubscribe` with `subscriptionId`.
3. The app removes that subscription for the same script owner and acknowledges `aiUnsubscribed`.
4. Reader exit removes any remaining subscriptions for that script owner.

## Source Proof Paths

| Proof target | File/function | What it proves |
|---|---|---|
| SDK declarations | `scripts/kit-sdk.ts` AI interfaces and globals | Request/response/event shapes exist. |
| SDK globals | `globalThis.aiAppendMessage`, `globalThis.aiSendMessage`, `globalThis.aiSetSystemPrompt`, `globalThis.aiOn` | SDK sends requests and owns pending/subscription bookkeeping. |
| SDK event dispatch | `scripts/kit-sdk.ts` AI event dispatch helper | Events dispatch by server-selected subscription id with chat-id defense in depth. |
| Rust protocol | `src/protocol/message/variants/ai.rs` | Rust request/response/event variants exist. |
| Direct handler boundary | `src/ai/sdk_handlers.rs` | Storage/read APIs are handled directly; live subscriptions are script-reader-owned. |
| Prompt bridge boundary | `src/execute_script/mod.rs` | Subscription messages are handled before unhandled fallback because they need the script response sender. |
| Prompt message enum | `src/main_sections/prompt_messages.rs` | No append/send/set-system/subscribe prompt variants are proven. |
| Prompt handler contrast | `src/prompt_handler/mod.rs` | `AiStartChat` and `AiFocus` have real UI handlers; gap APIs do not. |
| SDK shape tests | `tests/sdk/test-acp-sdk.ts` | Request shapes can be emitted; this is not runtime proof. |
| Protocol serde tests | `tests/protocol_ai_parts.rs` | Parts compatibility exists for `aiStartChat` / `aiSendMessage` shapes. |
| Prior atlas boundary | `feature-map/features/030-acp-chat-sdk-apis.md` | Established broad proven/unproven API split. |

## Unsafe Claims To Avoid

Do not write any of these until implementation and receipts exist:

- `aiAppendMessage` persists a message into an existing ACP chat.
- `aiAppendMessage` supports all roles at runtime.
- `aiSendMessage` appends a user message and starts streaming in an existing chat.
- `aiSendMessage` resolves context parts the same way as `aiStartChat`.
- `aiSetSystemPrompt` replaces or updates a chat's system prompt.
- `aiOn` subscribes to live ACP stream events.
- `aiUnsubscribe` stops app-side event routing.
- `aiStreamChunk`, `aiStreamComplete`, `aiNewMessage`, or `aiError` are emitted by the ACP runtime.
- SDK shape tests prove runtime support.
- A Rust protocol enum means the app handles the message.

## Implementation Plan

### Phase 0: Decide Semantics

Before coding, define these contracts:

- Does `aiAppendMessage` mutate storage headlessly, or does it require/open Agent Chat?
- Does `aiSendMessage` focus/open Agent Chat, or can it stream headlessly?
- Does `aiSetSystemPrompt` replace the first stored system message, append a new system message, or update a dedicated chat setting?
- Is `aiSubscribe` per script process, per request, per chat, or global?
- Does `aiUnsubscribe` need `subscriptionId` on the wire? For robust multi-subscription support, it should.

### Phase 1: Mutation APIs

Add prompt-message or runtime command variants for:

- `AiAppendMessage { request_id, chat_id, content, role }`.
- `AiSendMessage { request_id, chat_id, content, image, parts }`.
- `AiSetSystemPrompt { request_id, chat_id, prompt }`.

Map those messages out of the unhandled path in `src/execute_script/mod.rs`.

Runtime handlers should:

- Validate `chatId`.
- Validate message role instead of accepting arbitrary strings.
- Reject missing or deleted chats with protocol error responses.
- Update storage and in-memory chat state consistently.
- Update preview and message-count caches.
- Preserve request/response correlation.
- Return `aiMessageAppended`, `aiMessageSent`, or `aiSystemPromptSet` with failure details where applicable.

`aiAppendMessage` should persist without starting streaming. `aiSendMessage` should persist a user message and only then start streaming if that is the chosen contract. `aiSetSystemPrompt` should document exact replacement or insertion semantics.

### Phase 2: Subscription Registry

Add an app-side registry keyed by `subscriptionId`.

Store:

- Script/process identity or response sender.
- Event types.
- Optional chat id.
- Created timestamp.
- Cancellation state.

Implement `aiSubscribe` with validation and `aiSubscribed`. Fix `aiUnsubscribe` so it can identify a subscription, preferably by adding `subscriptionId` to the request. Clean up subscriptions when the script exits or the sender disconnects.

### Phase 3: Event Producers

Emit events from runtime points that know the event happened:

| Event | Producer point |
|---|---|
| `aiNewMessage` | After append, send user-message persistence, and assistant-message persistence. |
| `aiStreamChunk` | When streaming emits a chunk for a subscribed chat. |
| `aiStreamComplete` | When the final assistant message is persisted. |
| `aiError` | Invalid chat id, missing/deleted chat, provider error, context resolution failure, streaming failure, or subscription failure. |

Event routing should filter by subscription event type, optional chat id, and subscription id.

### Phase 4: SDK Compatibility

Tighten the SDK surface:

- Ensure response union types include the relevant response messages.
- Add `subscriptionId` to unsubscribe once Rust supports it.
- Filter client-side by `chatId` as defense in depth.
- Reject or timeout pending requests when the app returns `aiError`.
- Surface `aiSetSystemPrompt` success/error instead of resolving void blindly.
- Document that `imagePath` is read locally and converted to base64 before crossing to the app.

## Verification

### Current Negative Proof

The current gap should be proven by source audit and targeted negative runtime probes before implementation:

```bash
bun tests/sdk/test-acp-sdk.ts
```

This proves SDK message shape only. It does not prove runtime support.

Runtime negative probes should call one gap API at a time and expect either:

- a real app-side response path not found in the current bundle, or
- timeout/unhandled behavior that confirms the gap.

Suggested probes:

- `aiAppendMessage` only.
- `aiSendMessage` only.
- `aiSetSystemPrompt` only.
- `aiOn` only.

### Future Positive Proof

Once implemented, require:

| Test/receipt | Expected proof |
|---|---|
| Rust protocol serde tests | Request, response, event, and optional-field wire compatibility. |
| SDK shape tests | Exact payloads, image encoding branch, parts, event names, subscription id handling. |
| Runtime append smoke | `aiAppendMessage` returns real message id and `aiGetConversation` includes appended content and role. |
| Runtime send smoke | `aiSendMessage` returns real user message id; conversation includes message; `streamingStarted` matches runtime state. |
| Runtime send with parts | File/resource parts resolve or fail with clear receipts. |
| System prompt smoke | `aiSetSystemPrompt` changes exactly the documented system prompt representation. |
| Subscription smoke | `aiOn("message")` receives `aiNewMessage` for matching chat only. |
| Stream event smoke | Chunk and completion handlers receive scoped events. |
| Error event smoke | Invalid chat/provider failure emits or returns `aiError` with correlation. |
| Unsubscribe smoke | Handler receives no further events after unsubscribe. |
| Multi-subscription isolation | Different chats/event types do not cross-deliver. |
| Process cleanup | Script exit removes subscriptions; no stale sends remain. |
| Regression guard | Gap APIs no longer fall through to unhandled path. |

## Documentation Caveats

- `chat()` and `ai*` are different surfaces. `chat()` is the older inline prompt flow; `ai*` targets built-in Agent Chat with app-owned providers, history, storage, and streaming state.
- `aiStartChat(..., { systemPrompt })` is proven creation-time behavior. `aiSetSystemPrompt` is not proven post-creation behavior.
- `aiGetStreamingStatus` is polling/snapshot state, not live subscription.
- SDK tests that capture outbound JSON prove request shape only.
- Adding protocol variants without dispatch creates APIs that look real but can hang at runtime.
- The current unsubscribe protocol shape is incomplete for robust multi-subscription support because it lacks `subscriptionId`.
- The current SDK event dispatch should be treated as incomplete for scoped subscriptions because it dispatches by event type rather than by subscription id plus optional chat id.
