# Bundle Map: 030 ACP Chat SDK APIs

Slug: `acp-chat-sdk-apis-atlas`

Bundle path: `~/.oracle/bundles/acp-chat-sdk-apis-atlas.txt`

Packx command:

```bash
packx -l 120 -s "aiStartChat" -s "aiSendMessage" -s "aiAppendMessage" -s "aiSetSystemPrompt" -s "aiFocus" -s "aiIsOpen" -s "aiListChats" -s "aiGetConversation" -s "aiGetStreamingStatus" -s "aiDeleteChat" -s "aiSubscribe" -s "aiStreamChunk" -f markdown --no-interactive --stdout scripts/kit-sdk.ts src/protocol/message/variants/ai.rs src/protocol/types/ai.rs src/protocol/message/constructors/prompts.rs src/ai/sdk_handlers.rs src/execute_script/mod.rs src/main_sections/prompt_messages.rs src/prompt_handler/mod.rs src/ai/window/chat.rs src/ai/window/window_api.rs src/ai/window/types.rs src/ai/window/render_root.rs src/ai/model.rs src/ai/storage.rs src/ai/acp/history.rs src/ai/acp/thread.rs tests/sdk/test-acp-sdk.ts tests/sdk/test-ai-context-parts.ts tests/protocol_ai_parts.rs tests/smoke/test-ai-start-chat.ts > ~/.oracle/bundles/acp-chat-sdk-apis-atlas.txt
```

Pack summary:

- 20 files.
- 289 matches.
- 17 context windows.
- 64,887 tokens.
- 233,297 chars.

Included source windows:

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

Relevant `lat.md` commands:

```bash
lat expand "030 ACP Chat SDK APIs: aiIsOpen aiGetActiveChat aiListChats aiGetConversation aiStartChat aiAppendMessage aiSendMessage aiSetSystemPrompt aiFocus aiGetStreamingStatus aiDeleteChat aiOn"
lat search "ACP Chat SDK AI APIs aiIsOpen aiGetActiveChat aiListChats aiGetConversation aiStartChat aiAppendMessage aiSendMessage aiSetSystemPrompt aiFocus aiGetStreamingStatus aiDeleteChat aiOn"
```

Top relevant sections:

- `lat.md/acp-chat#ACP Chat`
- `lat.md/ai-context#AI Context and MCP#ACP handoff`
- `lat.md/acp-chat#ACP Chat#Mini AI And Full ACP Handoff Parity`
- `lat.md/acp-chat#ACP Chat#Entry paths`
- `lat.md/acp-chat#ACP Chat#ACP composer`
