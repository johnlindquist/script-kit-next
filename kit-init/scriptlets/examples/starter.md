---
name: My Bundle
description: Personal helpers
icon: sparkles
---

## Hello Snippet

```metadata
keyword: !hi
description: Quick greeting
```

```paste
Hello!
```

## Quick Note

```metadata
description: Save a quick note
```

```tool:quick-note
import "@scriptkit/sdk";

const note = await arg("Note");
await Bun.write(`${env.HOME}/quick-note.txt`, note);
await arg("Saved quick note", [
  { name: "Done", description: `${env.HOME}/quick-note.txt`, value: "done" },
]);
```
