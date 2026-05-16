# 031 Legacy `chat()` Prompt

This chapter maps the legacy SDK `chat()` conversational prompt, including script callbacks, built-in AI mode, ChatPrompt UI state, Mini AI reuse, actions, persistence, and handoff behavior.

Raw Oracle reference: [answer](../raw-oracle/031-legacy-chat-prompt/answer.md), [prompt](../raw-oracle/031-legacy-chat-prompt/prompt.md), [bundle map](../raw-oracle/031-legacy-chat-prompt/bundle-map.md), [full log](../raw-oracle/031-legacy-chat-prompt/output.log), [session metadata](../raw-oracle/031-legacy-chat-prompt/session.json).

## Executive Summary

The legacy `chat()` prompt is Script Kit GPUI's SDK-driven conversational prompt surface. It is exposed from `scripts/kit-sdk.ts` and rendered by Rust as `prompts::ChatPrompt` inside `AppView::ChatPrompt`.

Do not collapse it into ACP Chat. Legacy `chat()` uses prompt protocol messages such as `chat`, `chatMessage`, `chatStreamStart`, `chatStreamChunk`, `chatStreamComplete`, `chatSetError`, `chatClearError`, `chatClear`, and app-to-SDK `chatSubmit`. The newer ACP/AI Chat APIs use the separate `ai*` surface mapped in [030 ACP Chat SDK APIs](./030-acp-chat-sdk-apis.md).

Feature 031 covers:

- SDK `chat(options)` and controller helpers.
- Script-controlled `onInit` and `onMessage` callback mode.
- Built-in provider-backed AI mode when no callbacks are supplied.
- Setup-card mode when built-in AI is requested but no provider is configured.
- ChatPrompt rendering, input, streaming, scroll follow, markdown turns, error state, image/paste state, and setup card.
- Parent app actions dialog, model selection, copy/clear/capture actions, Mini AI reuse, persistence, and transfer to the separate AI window.

## Entry Points

| Entry | Owner | Result |
|---|---|---|
| `await chat(options?)` | `scripts/kit-sdk.ts` | Opens `AppView::ChatPrompt` and returns a `ChatResult` through SDK pending-result machinery. |
| `chat.addMessage(message)` | SDK controller | Sends `chatMessage` and appends to SDK-local `chatMessages`. |
| `chat.startStream(position?)` | SDK controller | Creates a streaming message id and sends `chatStreamStart`. |
| `chat.appendChunk(messageId, chunk)` | SDK controller | Appends chunk content and sends `chatStreamChunk`. |
| `chat.completeStream(messageId)` | SDK controller | Marks stream complete and sends `chatStreamComplete`. |
| `chat.clear()` | SDK controller | Clears local state and sends `chatClear`. |
| `chat.setError(messageId, error)` | SDK controller | Marks message error and sends `chatSetError`. |
| `chat.clearError(messageId)` | SDK controller | Clears message error and sends `chatClearError`. |
| Inline Mini AI | `src/app_impl/prompt_ai.rs` | Opens the same `ChatPrompt` surface as `inline-ai` or `inline-ai-setup`. |

`chat()` sets `currentChatId`, creates `currentConversationId = conv-${id}`, normalizes initial messages, prepends the `system` shorthand when provided, and sends a `type: "chat"` protocol message with placeholder, messages, hint, footer, actions, model list, `saveHistory`, and `useBuiltinAi`.

## SDK Modes

| Mode | Trigger | Runtime behavior |
|---|---|---|
| Script-controlled callback mode | `onMessage` is supplied | User submissions arrive as `chatSubmit`; the SDK appends the user message locally, invokes `onMessage(text)`, and the script drives assistant output through controller helpers. |
| Init-assisted mode | `onInit` is supplied | The prompt opens first, `stdin` is ref'd, then `onInit` can add messages or start work. Because `onInit` exists, `useBuiltinAi` is false. |
| Built-in AI mode | No `onInit` and no `onMessage` | SDK sends `useBuiltinAi: true`; Rust handles provider-backed AI streaming when providers exist. |
| Setup mode | Built-in AI requested and no providers exist | Rust shows an API-key setup card with Configure and Claude Code actions. |
| Simple wait mode | No `onMessage` | SDK waits for one pending completion, but Oracle flagged the exact result semantics as a verification risk. |

Controller helpers throw if there is no active `currentChatId`. Prompt-control messages are applied only when the current app view is `AppView::ChatPrompt` with a matching id; stale or mismatched ids are silently ignored.

## Runtime Path

The Rust protocol variant `Message::Chat` lives in `src/protocol/message/variants/prompts_media.rs`. Chat message data is represented by `ChatPromptMessage` in `src/protocol/types/chat.rs`, supporting both AI SDK style `role/content` and legacy Script Kit style `position/text`.

`src/prompt_handler/mod.rs` translates `Message::Chat` to `PromptMessage::ShowChat`, constructs a `ChatPrompt`, wires a `ChatSubmitCallback` that sends `Message::ChatSubmit { id, text }` back to the script, applies model and history settings, sets `current_view = AppView::ChatPrompt { id, entity }`, focuses `FocusTarget::ChatPrompt`, and resizes through `compact_ai_view_type_for_mode`.

The `ChatPrompt` entity owns the visible UI state: messages, input, callbacks, selected model/model list, provider registry, streaming state, setup-card state, image/paste state, turn-cache/list state, mini-mode rendering, and history settings.

## Human Interactions

| Interaction | Behavior |
|---|---|
| Enter | Submits input unless Shift is held. |
| Shift+Enter | Inserts a newline. |
| Escape | Stops active streaming and preserves partial content; otherwise closes the prompt. |
| Cmd+. | Stops active streaming. |
| Cmd+K | Opens chat actions through the parent actions dialog. |
| Cmd+Enter | Continues/transfers to the AI window path; in script-generation mode it can save/run generated script output. |
| Cmd+C | Copies the last assistant response. |
| Cmd+Backspace | Clears the conversation after destructive confirmation when routed through the action path. |
| Cmd+V | Handles image paste first, then text paste. |
| End / Cmd+Down | Jumps to the latest turn. |
| Setup Tab/Arrows | Moves focus between Configure API Key and Connect to Claude Code. |
| Setup Enter/Space | Activates the focused setup action. |

Chat turns render markdown responses, user prompts, image thumbnails, streaming "Thinking..." state, copy buttons, and error text. Scroll follow stays at the bottom while the user has not manually scrolled away; streaming invalidates the active turn height so the list remeasures.

## Actions

ChatPrompt renders its own footer, so `render_chat_prompt` passes no outer footer to the shared prompt shell. The parent shell still intercepts Cmd+K and routes action-dialog keys for `ActionsDialogHost::ChatPrompt`.

`toggle_chat_actions` builds a chat actions dialog from the current model, available models, whether messages exist, and whether an assistant response exists. Actions include:

- Select a model with `select_model_*`.
- Continue in chat / transfer to the mini AI window path.
- Expand into the full AI window.
- Copy the latest assistant response.
- Clear conversation with destructive confirmation.
- Capture a screen area attachment.

Oracle flagged model selection as a risk: the action path updates `chat.model`, but the bundle did not prove it also updates every built-in provider-selection field used for subsequent requests.

## Built-in AI

When `useBuiltinAi` is true and providers exist, the prompt handles generation directly:

- Empty input without image is ignored.
- Pasted text tokens are expanded before submit.
- Context mentions are expanded.
- Slash commands can transform the prompt.
- The user message is inserted into the UI.
- An empty streaming assistant message is inserted.
- Provider/model selection decides the request target.
- Streaming reveal updates the assistant turn until completion.

When providers or selected model data are missing, the prompt adds assistant error messages instead of crashing. If initial messages include a user message, the handler can set `needs_initial_response` so the prompt auto-generates an initial assistant response after first render.

## Images

ChatPrompt supports image attachment through paste, dropped images, and screen-area capture. Image data is normalized/encoded as PNG base64, and paste/screen capture paths reject images larger than 10 MB. Built-in AI sends pending image data to the provider.

Oracle marked image persistence and transfer semantics as ambiguous: the bundle proves provider use and transfer image counts, but it does not prove every image path is persisted to the AI database or preserved through every handoff.

## Persistence And Handoff

On inline close, `ChatPrompt::handle_escape` saves non-empty conversations to the AI database when `saveHistory` is true. Saved conversations use `ChatSource::ChatPrompt`.

On transfer to the AI window, ChatPrompt intentionally skips the inline DB save even when history is enabled, resets inline prompt state, dismisses the main prompt, opens the target AI window, and hands pending messages/images to that window. This avoids duplicate persistence while allowing the destination AI window to initialize with the conversation.

Inline Mini AI uses the same ChatPrompt:

- `inline-ai` uses built-in AI with providers, title "Ask AI", optional pending submit, and history enabled.
- `inline-ai-setup` shows setup actions when no provider is configured and does not save setup state to history.
- Escape emits a Mini AI close snapshot and returns to the main menu.
- Continue hides/removes the main window for the AI handoff.
- Configure opens API key setup.
- Claude Code enables Claude Code configuration.
- Actions dispatch through the parent window via typed `MiniAiUiRequest::ToggleActions`.

## Agent Observability

Agents can verify:

- `AppView::ChatPrompt` / prompt type identity.
- Mini AI state and close snapshots through `getState`.
- Draft changes through `setInput` and close snapshots.
- Protocol-level `chatSubmit` and SDK controller behavior in harness tests.
- Logs for chat actions, streaming, transfer, screen capture, and Mini AI close/actions.
- Visual/layout behavior through chat smoke tests when screenshots are needed.

Not proven directly observable from the bundle:

- Full message list through `getState`.
- Per-message streaming/error fields through state API.
- Provider request receipts.
- Retry dispatch.
- Image persistence across every path.
- Stale-id drops except by source inspection.

## Verification Map

Existing coverage includes:

- `tests/sdk/test-chat.ts` for basic `chat()` SDK presence and result shape.
- `tests/smoke/run-chat-tests.ts` chat smoke suite registration.
- `tests/smoke/test-chat-prompt.ts` for initial messages, programmatic add, streaming, right-aligned user messages, and clear behavior.
- Chat smoke tests for `onInit`, callbacks, errors, AI SDK compatibility, visual layout/content, and edge cases.
- `tests/mini_ai_snapshot_contract.rs` for close snapshots and `setInput` behavior.
- `tests/mini_window_sizing_contract.rs` for Mini AI sizing.
- `tests/mini_ai_actions_contract.rs` for typed parent action dispatch.

Recommended focused gates before changing this surface:

```bash
lat check
cargo test mini_ai_snapshot_contract mini_window_sizing_contract mini_ai_actions_contract
bun tests/smoke/run-chat-tests.ts
```

Use state-first agentic receipts for prompt identity, draft state, submit behavior, and close snapshots; use screenshots only for visual turn rendering, image thumbnails, or layout regressions.

## Risks And Gaps

| Risk | Why it matters |
|---|---|
| `onMessage` lifecycle and pending resolution | Oracle flagged that pending `chatSubmit` resolution may race or clear `currentChatId` before the script callback in some paths. |
| Listener cleanup | The bundle did not prove `process.on("chatSubmit")` listeners are removed after chat completion. |
| `options.system` mutation | SDK appears to prepend into the caller-provided message array, which can surprise scripts that reuse `options.messages`. |
| `saveHistory` defaults | SDK, protocol, and `ChatPromptConfig` defaults differ; effective behavior should be tested per path. |
| Model selection fields | Action model selection updates `chat.model`; built-in provider selection may use separate state. |
| Simple `chat()` return semantics | `chat.getResult()` hardcodes `action: "escape"`; actual submit/escape result semantics need proof. |
| Retry path | Types and error UI exist, but full retry callback/render/dispatch path was not fully proven in the bundle. |
| Image persistence | Provider use is proven; persistence and handoff are not fully proven for every image route. |
| Mini AI naming in generic SDK chat | Generic SDK ChatPrompt escape wiring uses Mini AI-named channel/logging in the inspected path. |
| `chatSubmit` backpressure | Rust uses `try_send`; a full channel can drop submissions with a warning. |
| Silent stale-id no-ops | Mismatched prompt ids are ignored without SDK-visible error. |
| Source-string tests | Several contracts are source audits; useful guardrails, but weaker than runtime proofs. |

## Boundaries

Legacy `chat()` is not:

- The ACP Chat SDK API surface (`aiIsOpen`, `aiStartChat`, `aiGetConversation`, etc.).
- The detached/full ACP Chat thread UI.
- The Quick Terminal or harness terminal.
- General provider configuration, except through the no-provider setup path.
- The newer script-generation harness path, even though some script-generation status/action code still exists in ChatPrompt.

