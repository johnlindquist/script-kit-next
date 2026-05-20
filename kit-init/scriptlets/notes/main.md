---
name: Notes Helper Examples
description: Create, update, organize, and inspect Notes through canonical runtime ports
author: Script Kit
icon: notebook-pen
---

# Notes Helper Examples

## Create Organized Note

```metadata
description: Create a tagged note through kit/notes_create and open it in Notes
tool:notes-create-organized
```

```typescript
import "@scriptkit/sdk";

async function callNotesTool(toolName: string, args: Record<string, unknown>) {
  const tool = (await mcp.discover(toolName)).find((entry) => entry.name === toolName);
  if (!tool) throw new Error(`${toolName} is not available`);
  return mcp.call(tool.serverId, tool.name, args);
}

const result = await callNotesTool("kit/notes_create", {
  title: "Project Plan",
  body: "# Project Plan\n\n#planning [[Research Notes]]\n\nNext steps...",
  tags: ["planning", "projects/script-kit"],
  aliases: ["Plan"],
  open: true,
  select: true,
});

await copy(JSON.stringify(result.structuredContent ?? result.raw, null, 2));
await arg("Created organized note", [
  { name: "Done", description: "The note was created and the result is on the clipboard", value: "done" },
]);
```

## Copy Create Organized Note Payload

```metadata
description: Copy a kit/notes_create payload with tags, aliases, links, and open/select
tool:copy-notes-create-organized
```

```typescript
import "@scriptkit/sdk";

const request = {
  name: "kit/notes_create",
  arguments: {
    title: "Project Plan",
    body: "# Project Plan\n\n#planning [[Research Notes]]\n\nNext steps...",
    tags: ["planning", "projects/script-kit"],
    aliases: ["Plan"],
    open: true,
    select: true,
  },
};

await copy(JSON.stringify(request, null, 2));
await arg("Copied Notes create payload", [
  { name: "Done", description: "The MCP payload is on the clipboard", value: "done" },
]);
```

## Update Organized Note

```metadata
description: Rewrite a note with tags and aliases through kit/notes_update
tool:notes-update-organized
```

```typescript
import "@scriptkit/sdk";

async function callNotesTool(toolName: string, args: Record<string, unknown>) {
  const tool = (await mcp.discover(toolName)).find((entry) => entry.name === toolName);
  if (!tool) throw new Error(`${toolName} is not available`);
  return mcp.call(tool.serverId, tool.name, args);
}

const id = await arg("Note UUID to update");
const result = await callNotesTool("kit/notes_update", {
  id,
  content: "# Project Plan\n\nUpdated body with [[Decision Log]].",
  tags: ["planning", "decisions"],
  aliases: ["Plan", "Project Plan"],
  open: true,
  select: true,
});

await copy(JSON.stringify(result.structuredContent ?? result.raw, null, 2));
await arg("Updated organized note", [
  { name: "Done", description: "The note was updated and the result is on the clipboard", value: "done" },
]);
```

## Copy Update Organized Note Payload

```metadata
description: Copy a kit/notes_update payload that rewrites visible metadata frontmatter
tool:copy-notes-update-organized
```

```typescript
import "@scriptkit/sdk";

const request = {
  name: "kit/notes_update",
  arguments: {
    id: "NOTE_UUID",
    content: "# Project Plan\n\nUpdated body with [[Decision Log]].",
    tags: ["planning", "decisions"],
    aliases: ["Plan", "Project Plan"],
    open: true,
    select: true,
  },
};

await copy(JSON.stringify(request, null, 2));
await arg("Copied Notes update payload", [
  { name: "Done", description: "The MCP payload is on the clipboard", value: "done" },
]);
```

## Copy Notes Search Resource Payload

```metadata
description: Copy a kit://notes read request for active notes
tool:copy-notes-resource-list
```

```typescript
import "@scriptkit/sdk";

const request = {
  method: "resources/read",
  params: {
    uri: "kit://notes?limit=25",
  },
};

await copy(JSON.stringify(request, null, 2));
await arg("Copied Notes resource payload", [
  { name: "Done", description: "The resource read payload is on the clipboard", value: "done" },
]);
```

## Copy getElements Example

```metadata
description: Copy a targeted getElements request for the Notes window
tool:copy-notes-get-elements
```

```typescript
import "@scriptkit/sdk";

const request = {
  type: "getElements",
  requestId: "elm-notes",
  target: { type: "kind", kind: "notes" },
  limit: 10,
};
await copy(JSON.stringify(request, null, 2));
await arg("Copied Notes getElements example", [
  { name: "Done", description: "The request JSON is on the clipboard", value: "done" },
]);
```

## Copy batch setInput Example

```metadata
description: Copy a batch request that writes into the Notes editor
tool:copy-notes-batch
```

```typescript
import "@scriptkit/sdk";

const request = {
  type: "batch",
  requestId: "b-notes",
  target: { type: "kind", kind: "notes" },
  commands: [{ type: "setInput", text: "Hello from automation" }],
};
await copy(JSON.stringify(request, null, 2));
await arg("Copied Notes batch example", [
  { name: "Done", description: "The request JSON is on the clipboard", value: "done" },
]);
```
