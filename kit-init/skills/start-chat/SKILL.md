---
name: start-chat
description: Start and manage Agent Chat conversations from scripts, including typed context parts, streaming status, and chat lifecycle operations.
---

# Start Chat

Use this skill when the user wants a script that opens Agent Chat, continues a chat, injects typed context parts, checks streaming, or deletes/focuses chats.

## Write Here

`~/.scriptkit/plugins/main/scripts/<name>.ts`

## Canonical ACP SDK Flow

```typescript
import "@scriptkit/sdk";

const result = await aiStartChat("Summarize this context", {
  systemPrompt: "Be concise",
  modelId: "claude-3-5-sonnet-20241022",
  parts: [
    { kind: "resourceUri", uri: "kit://context?profile=minimal", label: "Current Context" },
  ],
});

await aiSendMessage(result.chatId, "Now inspect this file", undefined, [
  { kind: "filePath", path: "/tmp/example.rs", label: "example.rs" },
]);

const status = await aiGetStreamingStatus(result.chatId);
await aiFocus();

if (!status.isStreaming) {
  await aiDeleteChat(result.chatId, false);
}
```

## Use These Functions

- `aiStartChat()` — start a new Agent Chat thread
- `aiSendMessage()` — continue an existing thread
- `aiAppendMessage()` — seed history without triggering a response
- `aiOn()` — subscribe to streaming events
- `aiGetStreamingStatus()` — poll stream state
- `aiFocus()` — bring Agent Chat forward
- `aiDeleteChat()` — soft-delete or permanently delete a chat

## Context Parts

Supported `parts` entries:

- `{ kind: "resourceUri", uri, label }`
- `{ kind: "filePath", path, label }`

Use `kit://context?profile=minimal` for current desktop context.

## Common Pitfalls

- Use ACP SDK functions for programmatic chat workflows. Use prompt-level `chat()` only for inline prompt UIs.
- `parts` are typed attachments, not free-form text blobs.
- Do not invent Notes globals or screenshot globals here.

## Related Examples

- **Canonical**: `~/.scriptkit/plugins/examples/scriptlets/acp-chat/main.md` — start chats, attach typed context parts, and inspect streaming status
- **Compatibility mirror**: `~/.scriptkit/plugins/examples/scriptlets/acp-chat.md` — auto-generated flat copy of the canonical source

## Related Skills

- [add-actions](../add-actions/SKILL.md) — expose ACP helpers through the Actions Menu
- [manage-notes](../manage-notes/SKILL.md) — hand off note content into Agent Chat
- [new-scriptlet](../new-scriptlet/SKILL.md) — package ACP helpers as scriptlet bundles
