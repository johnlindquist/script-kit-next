# Skill: Script Authoring

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
await Bun.write(
  `${env.HOME}/notes/${Date.now()}.txt`,
  note,
);
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

const home = env.HOME || "~";
const files = await $`find ${home}/Documents -maxdepth 2 -type f -name "*.md"`.text();

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

const path = `${env.HOME}/blog/posts/${slug}.md`;
await Bun.write(path, content);
await notify(`Created: ${path}`);
```

## Verification

After creating a script:
1. Open Script Kit (default: Cmd+Space or your configured hotkey)
2. Type the script name — it should appear in the list
3. Run it to verify behavior

## Common Mistakes

- **Missing SDK import**: Always start with `import "@scriptkit/sdk";`
- **CommonJS imports**: Use ES `import` syntax, never CommonJS
- **Comment metadata**: Use `export const metadata = {...}`, not comment-based metadata
- **Node.js APIs**: Use `Bun.file()` / `Bun.write()` / `` $`cmd` `` instead of `fs` / `child_process`
- **Wrong directory**: Scripts must be in `kit/main/scripts/`, not `scripts/` or elsewhere
