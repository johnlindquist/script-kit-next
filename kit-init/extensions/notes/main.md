---
name: Notes Automation Examples
description: Copy canonical Notes automation payloads for getElements, waitFor, and batch
author: Script Kit
icon: notebook-pen
---

# Notes Automation Examples

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
await notify("Copied Notes getElements example");
```

## Copy waitFor Example

```metadata
description: Copy a waitFor request for the Notes editor
tool:copy-notes-wait-for
```

```typescript
import "@scriptkit/sdk";

const request = {
  type: "waitFor",
  requestId: "w-notes",
  target: { type: "kind", kind: "notes" },
  condition: { type: "elementExists", semanticId: "input:notes-editor" },
  timeout: 3000,
  pollInterval: 25,
};
await copy(JSON.stringify(request, null, 2));
await notify("Copied Notes waitFor example");
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
await notify("Copied Notes batch example");
```
