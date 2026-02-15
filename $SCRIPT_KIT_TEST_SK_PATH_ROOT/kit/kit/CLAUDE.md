# Script Kit - Claude Instructions

This file provides Claude-specific guidance for working with Script Kit GPUI.

## ⚠️ Critical: This is Script Kit GPUI (v2), NOT the original Script Kit

Script Kit GPUI is a **complete rewrite** of the original Script Kit:
- **Old Script Kit (v1)**: Electron + Node.js
- **Script Kit GPUI (v2)**: GPUI (Rust) + Bun

If your training data includes the old Script Kit, **ignore those patterns**. Use only what's documented here.

---

## Directory Structure

```
~/.scriptkit/
├── kit/                          # Version-controllable kit directory
│   ├── main/                     # Main kit (default)
│   │   ├── scripts/             # Your TypeScript scripts
│   │   ├── extensions/          # Markdown files with embedded commands
│   │   └── agents/              # AI agent definitions
│   ├── config.ts                # Configuration (hotkey, font sizes, etc.)
│   ├── theme.json               # Theme customization (colors, etc.)
│   ├── package.json             # Enables top-level await ("type": "module")
│   ├── tsconfig.json            # TypeScript configuration
│   ├── AGENTS.md                # SDK documentation for AI agents
│   └── CLAUDE.md                # This file
├── sdk/                          # SDK (managed by app, do not edit)
│   └── kit-sdk.ts
├── db/                           # SQLite databases
├── logs/                         # Application logs
└── GUIDE.md                      # User guide
```

---

## Writing Scripts

### Minimal Script Template

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "My Script",
  description: "What this script does",
};

// Your code here - top-level await is supported
const result = await arg("Choose an option", ["A", "B", "C"]);
console.log(result);
```

### Key Points

1. **Always import the SDK first**: `import "@scriptkit/sdk";`
2. **Use `export const metadata`**: NOT comment-based metadata (deprecated)
3. **Top-level await**: Works out of the box (thanks to `package.json` `"type": "module"`)
4. **Bun APIs**: Use `Bun.file()`, `Bun.write()`, `$\`command\`` - NOT Node.js fs/child_process

### Common SDK Functions

```typescript
// User input
const text = await arg("Enter something");
const choice = await arg("Pick one", ["Option 1", "Option 2"]);

// Display content
await div("<h1 class='text-2xl'>Hello</h1>");  // HTML with Tailwind

// Editor
const code = await editor("// Edit this", "typescript");

// Forms
const [name, email] = await fields([
  { name: "name", label: "Name" },
  { name: "email", label: "Email", type: "email" },
]);

// Clipboard
const text = await paste();
await copy("Copied this text");

// Open URLs/files
await open("https://example.com");
```

---

## Configuration (config.ts)

Located at `~/.scriptkit/kit/config.ts`:

```typescript
import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: { key: "Space", modifiers: ["command"] },
  editorFontSize: 14,
  terminalFontSize: 14,
  builtIns: { clipboardHistory: true, appLauncher: true },
} satisfies Config;
```

---

## Theme (theme.json)

Located at `~/.scriptkit/kit/theme.json`:

```json
{
  "colors": {
    "background": { "main": "#1e1e2e", "panel": "#181825" },
    "text": { "primary": "#cdd6f4", "secondary": "#a6adc8" },
    "accent": { "primary": "#89b4fa", "secondary": "#74c7ec" },
    "ui": { "border": "#313244", "divider": "#45475a" }
  }
}
```

---

## Extensions (formerly Scriptlets)

Markdown files in `~/.scriptkit/kit/main/extensions/*.md` with embedded code:

```markdown
---
name: My Tools
description: Collection of useful tools
---

# My Tools

## Say Hello
\`\`\`tool:hello
import "@scriptkit/sdk";
const name = await arg("Name?");
await div(`<h1>Hello, ${name}!</h1>`);
\`\`\`

## Quick Template
\`\`\`template:greeting
Hello {{name}}, welcome to {{place}}!
\`\`\`
```

---

## DO NOT

- Use `require()` - use ES imports
- Use Node.js `fs` - use `Bun.file()` and `Bun.write()`
- Use Node.js `child_process` - use `$\`command\`` (Bun shell)
- Use comment-based metadata (`// Name:`) - use `export const metadata`
- Modify files in `~/.scriptkit/sdk/` - they're managed by the app
- Reference old Script Kit v1 patterns (Electron, Kit SDK, @johnlindquist/kit)

---

## File Locations Summary

| Purpose | Path |
|---------|------|
| Scripts | `~/.scriptkit/kit/main/scripts/*.ts` |
| Extensions | `~/.scriptkit/kit/main/extensions/*.md` |
| Agents | `~/.scriptkit/kit/main/agents/*.md` |
| Config | `~/.scriptkit/kit/config.ts` |
| Theme | `~/.scriptkit/kit/theme.json` |
| SDK Docs | `~/.scriptkit/kit/AGENTS.md` |
