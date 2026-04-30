---
name: Agent Chat Examples
description: Programmatic Agent Chat examples using the chat SDK
author: Script Kit
icon: message-square
---

# Agent Chat Examples

## Ask Current Context

```metadata
description: Start Agent Chat with minimal desktop context
tool:ask-current-context
```

```typescript
import "@scriptkit/sdk";

const message = await arg("Ask Agent Chat", "What should I know about this context?");
const result = await aiStartChat(message, {
  parts: [
    { kind: "resourceUri", uri: "kit://context?profile=minimal", label: "Current Context" },
  ],
});
await aiFocus();
await arg("Opened Agent Chat", [
  { name: "Done", description: result.title, value: "done" },
]);
```

## Review File

```metadata
description: Start Agent Chat with a file attachment part
tool:review-file
```

```typescript
import "@scriptkit/sdk";

const filePath = await arg("File path");
const prompt = await arg("Prompt", "Review this file");
const result = await aiStartChat(prompt, {
  parts: [
    {
      kind: "filePath",
      path: filePath,
      label: filePath.split("/").pop() || filePath,
    },
  ],
});
await aiFocus();
await arg("Started Agent Chat", [
  { name: "Done", description: result.chatId, value: "done" },
]);
```

## Check Streaming Status

```metadata
description: Inspect whether the active Agent Chat is streaming
tool:check-streaming-status
```

```typescript
import "@scriptkit/sdk";

const status = await aiGetStreamingStatus();
await div(md([
  "# Agent Chat Streaming Status",
  "",
  `- isStreaming: ${status.isStreaming}`,
  `- chatId: ${status.chatId ?? "none"}`,
  `- partialContent: ${status.partialContent ?? "none"}`,
].join("\n")));
```
