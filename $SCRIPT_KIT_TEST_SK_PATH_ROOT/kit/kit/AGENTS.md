# Script Kit User Scripts Guide

This guide is for AI agents and developers writing scripts for Script Kit.
Script Kit is a productivity tool that runs TypeScript scripts with a rich UI.

---

## Quick Start

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "My Script",
  description: "Does something useful",
};

const choice = await arg("Pick an option", ["Option 1", "Option 2"]);
await div(`<h1>You chose: ${choice}</h1>`);
```

---

## Table of Contents

1. [Script Metadata](#script-metadata)
2. [SDK Import](#sdk-import)
3. [Core SDK Functions](#core-sdk-functions)
4. [Scriptlet Format](#scriptlet-format)
5. [Configuration (config.ts)](#configuration-configts)
6. [Testing Scripts](#testing-scripts)
7. [Examples](#examples)

---

## Script Metadata

Scripts use the `metadata` export for configuration. This is the **preferred format** over comment-based metadata.

### Basic Metadata

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "My Script",           // Display name in menu
  description: "What it does", // Shown below the name
  shortcut: "cmd shift m",     // Global hotkey (optional)
  alias: "ms",                 // Quick search alias (optional)
};
```

### All Metadata Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Display name in the Script Kit menu |
| `description` | string | Description shown below the name |
| `shortcut` | string | Global keyboard shortcut (e.g., "cmd shift k") |
| `alias` | string | Short alias for quick triggering |
| `author` | string | Script author |
| `enter` | string | Custom text for Enter/Submit button |
| `icon` | string | Icon name (e.g., "file", "terminal", "star") |
| `tags` | string[] | Tags for categorization and search |
| `hidden` | boolean | Hide from the main script list |
| `background` | boolean | Run without UI (background process) |
| `schedule` | string | Cron expression for scheduled execution |
| `watch` | string | File path pattern that triggers the script |

### Legacy Comment-Based Metadata (Deprecated)

Comment-based metadata is supported but **deprecated** for new scripts:

```typescript
// Name: My Script
// Description: What it does
// Shortcut: cmd shift m
```

**Why prefer `export const metadata`?**
- Full TypeScript type safety
- IDE autocomplete and error checking
- Access to more metadata fields
- Easier to read and maintain

---

## SDK Import

Import the Script Kit SDK to get global functions:

```typescript
import "@scriptkit/sdk";
```

This import:
- Makes all SDK functions available globally (arg, div, editor, etc.)
- Provides TypeScript types for IDE support
- Is resolved via tsconfig.json path mapping

---

## Core SDK Functions

### Prompts

#### `arg()` - Text Input with Choices

```typescript
// Simple text input
const name = await arg("What's your name?");

// With string choices
const color = await arg("Pick a color", ["Red", "Green", "Blue"]);

// With rich choices
const file = await arg("Select a file", [
  { name: "Document.pdf", value: "/path/to/doc.pdf", description: "PDF file" },
  { name: "Image.png", value: "/path/to/img.png", description: "Image file" },
]);

// With dynamic choices (async function)
const repo = await arg("Select repo", async () => {
  const response = await fetch("https://api.github.com/user/repos");
  const repos = await response.json();
  return repos.map((r: any) => ({ name: r.name, value: r.html_url }));
});
```

#### `div()` - Display HTML Content

```typescript
// Simple HTML
await div("<h1>Hello World!</h1>");

// With Tailwind CSS classes
await div(`
  <div class="flex flex-col items-center p-8">
    <h1 class="text-4xl font-bold text-yellow-400">Welcome!</h1>
    <p class="text-gray-400 mt-4">Press Escape to close</p>
  </div>
`);
```

#### `editor()` - Code Editor

```typescript
// Open editor with content
const code = await editor("// Write your code here", "typescript");

// Edit existing file content
const edited = await editor(existingContent, "json");
```

#### `fields()` - Multi-Field Form

```typescript
const [name, email, age] = await fields([
  { name: "name", label: "Name", type: "text", placeholder: "John Doe" },
  { name: "email", label: "Email", type: "email" },
  { name: "age", label: "Age", type: "number" },
]);
```

### File System

#### `path()` - File/Folder Picker

```typescript
// Pick a file
const filePath = await path("Select a file");

// Pick with starting directory
const docPath = await path({ startPath: "~/Documents", hint: "Choose a document" });
```

#### `drop()` - Drag and Drop

```typescript
// Accept dropped files
const files = await drop("Drop files here");
for (const file of files) {
  console.log(file.path, file.name, file.size);
}
```

### Input Capture

#### `hotkey()` - Capture Keyboard Shortcut

```typescript
const shortcut = await hotkey("Press a keyboard shortcut");
console.log(shortcut.key, shortcut.command, shortcut.shift);
```

### Display

#### `md()` - Render Markdown

```typescript
const html = md(`
# Hello World
This is **bold** and this is *italic*.
`);
await div(html);
```

### Advanced

#### `term()` - Terminal Emulator

```typescript
await term("htop");  // Run interactive command
await term({ command: "npm install", cwd: "/path/to/project" });
```

#### `chat()` - Chat Interface

```typescript
await chat({
  onSubmit: async (input) => {
    // Handle user message
    return { text: `You said: ${input}`, position: "left" };
  }
});
```

#### `widget()` - Floating Widget Window

```typescript
const w = await widget(`<h1>Floating Widget</h1>`, {
  width: 300,
  height: 200,
  draggable: true,
  alwaysOnTop: true,
});
```

---

## Scriptlet Format

Extensions are markdown files with embedded commands. They live in `~/.scriptkit/kit/main/extensions/`.

### Basic Scriptlet

```markdown
---
name: My Scriptlet
description: A quick tool
author: Your Name
---

# My Scriptlet

## Greeting Tool
\`\`\`tool:greet
import "@scriptkit/sdk";
const name = await arg("Enter name");
await div(`<h1>Hello, ${name}!</h1>`);
\`\`\`
```

### Frontmatter Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Display name for the bundle |
| `description` | string | Brief description |
| `author` | string | Author name |
| `icon` | string | Icon identifier |

### Code Block Types

- `tool:name` - Executable tool
- `template:name` - Text expansion template
- `snippet:name` - Code snippet
- `prompt:name` - AI prompt

### Variable Substitution

Templates support `{{variable}}` substitution:

```markdown
\`\`\`template:email-reply
Hi {{name}},

Thank you for your email about {{topic}}.

Best regards
\`\`\`
```

---

## Configuration (config.ts)

The `~/.scriptkit/kit/config.ts` file configures Script Kit:

```typescript
import type { Config } from "@scriptkit/sdk";

export default {
  // Global hotkey to show Script Kit
  hotkey: {
    key: "Space",
    modifiers: ["command"],
  },

  // UI settings
  editorFontSize: 14,
  terminalFontSize: 14,

  // Built-in features
  builtIns: {
    clipboardHistory: true,
    appLauncher: true,
  },
} satisfies Config;
```

### Config Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `hotkey` | object | cmd+; | Global activation hotkey |
| `editorFontSize` | number | 14 | Editor font size |
| `terminalFontSize` | number | 14 | Terminal font size |
| `builtIns.clipboardHistory` | boolean | true | Enable clipboard history |
| `builtIns.appLauncher` | boolean | true | Enable app launcher |

---

## Testing Scripts

### Run from Script Kit

1. Open Script Kit (default: Cmd+;)
2. Type your script name
3. Press Enter to run

### Run from Terminal

```bash
# Using bun directly
bun run ~/.scriptkit/kit/main/scripts/my-script.ts

# With the kit CLI (if installed)
kit run my-script
```

### Debugging

Add console.error() for debug output:

```typescript
import "@scriptkit/sdk";

console.error("[DEBUG] Script starting...");
const result = await arg("Choose", ["A", "B"]);
console.error("[DEBUG] User chose:", result);
```

---

## Examples

### Example 1: Quick Note

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "Quick Note",
  description: "Save a quick note to a file",
  shortcut: "cmd shift n",
};

const note = await arg("Enter your note");
const timestamp = new Date().toISOString();
const entry = `\n## ${timestamp}\n${note}\n`;

await Bun.write(
  Bun.file(`${home()}/notes.md`),
  (await Bun.file(`${home()}/notes.md`).text().catch(() => "# Notes\n")) + entry
);

await div(`<p class="text-green-400">Note saved!</p>`);
```

### Example 2: GitHub Repo Opener

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "Open GitHub Repo",
  description: "Search and open a GitHub repository",
  shortcut: "cmd shift g",
};

const repo = await arg("Search repos", async (input) => {
  if (!input) return [];
  const res = await fetch(`https://api.github.com/search/repositories?q=${input}`);
  const data = await res.json();
  return data.items?.map((r: any) => ({
    name: r.full_name,
    value: r.html_url,
    description: r.description || "No description",
  })) || [];
});

await open(repo);
```

### Example 3: JSON Formatter

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "Format JSON",
  description: "Pretty-print JSON from clipboard",
};

const clipboard = await paste();
try {
  const formatted = JSON.stringify(JSON.parse(clipboard), null, 2);
  await copy(formatted);
  await div(`<pre class="text-green-400">${formatted}</pre>`);
} catch {
  await div(`<p class="text-red-400">Invalid JSON in clipboard</p>`);
}
```

### Example 4: System Info Widget

```typescript
import "@scriptkit/sdk";
import os from "os";

export const metadata = {
  name: "System Info",
  description: "Show system information",
};

const info = `
  <div class="p-4 space-y-2">
    <p><strong>Platform:</strong> ${os.platform()}</p>
    <p><strong>Arch:</strong> ${os.arch()}</p>
    <p><strong>CPUs:</strong> ${os.cpus().length}</p>
    <p><strong>Memory:</strong> ${Math.round(os.totalmem() / 1024 / 1024 / 1024)}GB</p>
    <p><strong>Uptime:</strong> ${Math.round(os.uptime() / 3600)} hours</p>
  </div>
`;

await div(info);
```

### Example 5: File Search

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "Find Files",
  description: "Search for files by name",
  shortcut: "cmd shift f",
};

const query = await arg("Search for files");
const { stdout } = await $`find ~ -name "*${query}*" -type f 2>/dev/null | head -20`;

const files = stdout.trim().split("\n").filter(Boolean);

if (files.length === 0) {
  await div(`<p class="text-yellow-400">No files found</p>`);
} else {
  const selected = await arg("Select file", files.map(f => ({
    name: f.split("/").pop() || f,
    value: f,
    description: f,
  })));
  
  await open(selected);
}
```

---

## Best Practices

1. **Always use `export const metadata`** - Get type safety and IDE support
2. **Import the SDK first** - `import "@scriptkit/sdk"` at the top
3. **Use Tailwind classes** - Built-in support for styling in div()
4. **Handle errors gracefully** - Wrap async operations in try/catch
5. **Keep scripts focused** - One script, one task
6. **Use meaningful names** - Clear metadata.name and description
7. **Add shortcuts sparingly** - Only for frequently used scripts

---

## File Locations

| Path | Purpose |
|------|---------|
| `~/.scriptkit/kit/main/scripts/` | Your scripts |
| `~/.scriptkit/kit/main/extensions/` | Your extensions |
| `~/.scriptkit/kit/main/agents/` | Your AI agent definitions |
| `~/.scriptkit/kit/config.ts` | Configuration |
| `~/.scriptkit/kit/theme.json` | Theme customization |
| `~/.scriptkit/sdk/` | SDK (managed by app) |
| `~/.scriptkit/kit/AGENTS.md` | This guide (for AI agents) |
| `~/.scriptkit/kit/CLAUDE.md` | Claude-specific instructions |
