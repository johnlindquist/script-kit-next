---
name: script-authoring
description: Create and manage TypeScript scripts for Script Kit. Use when the user wants to write a new script, edit an existing script, or understand Script Kit's SDK and metadata system.
---

# Script Authoring

Create and manage TypeScript scripts for Script Kit.

## Where Scripts Live

```
~/.scriptkit/kit/main/scripts/*.ts
```

Scripts are automatically discovered by Script Kit when saved to this directory.

## Creating a New Script

1. Create a `.ts` file in `~/.scriptkit/kit/main/scripts/`
2. Add the SDK import and metadata export
3. Save — Script Kit detects it immediately

### Minimal Template

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "My Script",
  description: "What this script does",
};

// Your code here
```

### With Global Hotkey

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "Quick Capture",
  description: "Capture a quick note",
  shortcut: "cmd shift n",
};

const note = await arg("Quick note:");
const filePath = home("notes", `${Date.now()}.txt`);
await Bun.write(filePath, note);
await notify("Note saved!");
```

### With Search Alias

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "Open Project",
  description: "Open a project in VS Code",
  alias: "op",
};

const projects = await $`ls ~/projects`.text();
const project = await arg(
  "Which project?",
  projects.trim().split("\n"),
);
await $`code ~/projects/${project}`;
```

## Script Patterns

### Prompt → Transform → Output

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "JSON Formatter",
  description: "Format JSON from clipboard",
};

const raw = await paste();
try {
  const formatted = JSON.stringify(JSON.parse(raw), null, 2);
  await copy(formatted);
  await notify("JSON formatted and copied!");
} catch {
  await div(`<div class="p-4 text-red-400">Invalid JSON in clipboard</div>`);
}
```

### Dynamic Choices with Preview

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "File Browser",
  description: "Browse and open files",
};

const documentsDir = home("Documents");
const files = await $`find ${documentsDir} -maxdepth 2 -type f -name "*.md"`.text();

const file = await arg(
  "Open file",
  files.trim().split("\n").map((f) => ({
    name: f.split("/").pop() || f,
    description: f,
    value: f,
    preview: `<pre class="p-4 text-sm">${f}</pre>`,
  })),
);

await open(file);
```

### Multi-Step Workflow

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "New Blog Post",
  description: "Scaffold a new blog post",
};

const [title, category] = await fields([
  { name: "title", label: "Post Title" },
  { name: "category", label: "Category" },
]);

const slug = title.toLowerCase().replace(/\s+/g, "-").replace(/[^a-z0-9-]/g, "");
const date = new Date().toISOString().split("T")[0];
const content = `---
title: ${title}
date: ${date}
category: ${category}
---

# ${title}

Write your post here.
`;

const filePath = home("blog", "posts", `${slug}.md`);
await Bun.write(filePath, content);
await notify(`Created: ${filePath}`);
```

## Verification

When you create or edit a script from the Tab AI harness, writing the file is not enough. You must verify the actual script inside the current Claude Code terminal session before you report success.

Required loop for every script:

1. Save the script to `~/.scriptkit/kit/main/scripts/<name>.ts`
2. If the script normally uses UI or typed input (`arg`, `div`, `editor`, `fields`, etc.), add a non-interactive smoke path behind `process.env.SK_VERIFY === "1"`
3. Syntax-check / transpile it with Bun:
   ```bash
   bun build ~/.scriptkit/kit/main/scripts/<name>.ts --target=bun --outfile ~/.scriptkit/tmp/test-scripts/<name>.verify.mjs
   ```
4. Execute it with Bun:
   ```bash
   SK_VERIFY=1 bun ~/.scriptkit/kit/main/scripts/<name>.ts
   ```
5. Confirm the stdout, written file, or other observable result matches the request
6. If either command fails, fix the script and rerun both commands
7. Never report success until both commands pass and the observed behavior is correct

### Verification-Friendly Pattern

Use this when the real script flow is interactive but the harness still needs a terminal-only execution path:

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "My Script",
  description: "What it does",
};

const isVerify = process.env.SK_VERIFY === "1";

const value = isVerify
  ? "verification input"
  : await arg("What should this script do?");

const output = `Result: ${value}`;

if (isVerify) {
  console.log(JSON.stringify({ ok: true, output }));
} else {
  await div(`<div class="p-8 text-2xl">${output}</div>`);
}
```

For UI-heavy requests, the Bun gate is still mandatory. If you also want to manually open Script Kit afterward, do that **after** the Bun gate — not instead of it.

## Prompt Sequencing

Script Kit prompt APIs are stateful interactive surfaces. Never call them concurrently.

- Do not use `Promise.all`, `Promise.race`, `Promise.any`, or `Promise.allSettled` with `arg`, `fields`, `editor`, `div`, `form`, `drop`, `find`, `path`, `textarea`, `select`, or `grid`
- Multi-step prompt flows must be sequential

Wrong:

```typescript
const [url1, url2, url3] = await Promise.all([
  arg("URL 1"),
  arg("URL 2"),
  arg("URL 3"),
]);
```

Right:

```typescript
const url1 = await arg("URL 1");
const url2 = await arg("URL 2");
const url3 = await arg("URL 3");
```

## Path Safety

- Prefer `home(...)` for user-relative paths such as `Documents`, `Downloads`, and `.scriptkit`
- Use `home(".scriptkit", "kit", "main", ...)` when you need the Script Kit workspace explicitly
- Do not build user paths from `env.HOME`; it may be unset in generated scripts and can produce broken paths like `undefined/...`

### Sample Input and Expected Output

- Sample file: `~/.scriptkit/kit/main/scripts/hello-world.ts`
- Sample command 1:
  ```bash
  bun build ~/.scriptkit/kit/main/scripts/hello-world.ts --target=bun --outfile ~/.scriptkit/tmp/test-scripts/hello-world.verify.mjs
  ```
  Expected result: exit code 0; file `~/.scriptkit/tmp/test-scripts/hello-world.verify.mjs` exists.
- Sample command 2:
  ```bash
  SK_VERIFY=1 bun ~/.scriptkit/kit/main/scripts/hello-world.ts
  ```
  Expected stdout: `{"ok":true,"greeting":"Hello, verification!"}`

## Common Mistakes

- **Missing SDK import**: Always start with `import "@scriptkit/sdk";`
- **CommonJS imports**: Use ES `import` syntax, never CommonJS
- **Comment metadata**: Use `export const metadata = {...}`, not comment-based metadata
- **Node.js APIs**: Use `Bun.file()` / `Bun.write()` / `` $`cmd` `` instead of `fs` / `child_process`
- **Wrong directory**: Scripts must be in `kit/main/scripts/`, not `scripts/` or elsewhere
- **Unsafe home paths**: Prefer `home(...)` over `env.HOME` for user-relative locations

## Related Skills

- [scriptlets](../scriptlets/SKILL.md) — package scripts as scriptlet bundles
- [acp-chat](../acp-chat/SKILL.md) — programmatic ACP Chat workflows using the ACP SDK
- [custom-actions](../custom-actions/SKILL.md) — expose script helpers through the Actions Menu
- [notes](../notes/SKILL.md) — automate the Notes window from scripts
- [agents](../agents/SKILL.md) — mdflow-backed agent files (compatibility path)
