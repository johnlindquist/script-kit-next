# Script Kit SDK Reference

Complete reference for AI agents writing Script Kit scripts.

> **Package**: `@scriptkit/sdk` — **Runtime**: Bun — **Scripts**: `~/.scriptkit/kit/main/scripts/*.ts`

## Script Template

Every script follows this structure:

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "Script Name",
  description: "What this script does",
  // shortcut: "cmd shift x",   // optional global hotkey
  // alias: "sn",               // optional search alias
};

// Your code here — top-level await works
```

## Prompts

### arg — Universal Input

```typescript
// Simple text input
const text = await arg("Enter something");

// Choices (string array)
const pick = await arg("Choose", ["Red", "Green", "Blue"]);

// Rich choices (objects)
const item = await arg("Search items", [
  { name: "GitHub", description: "Open GitHub", value: "gh" },
  { name: "Docs", description: "Open docs", value: "docs" },
]);

// Choices with preview panel
const selected = await arg(
  "Preview example",
  [
    { name: "Alpha", value: "a", preview: "<h1>Alpha Details</h1>" },
    { name: "Beta", value: "b", preview: "<h1>Beta Details</h1>" },
  ],
);

// Dynamic/async choices
const result = await arg("Search GitHub", async (input) => {
  if (!input) return [];
  const res = await fetch(`https://api.github.com/search/repositories?q=${input}`);
  const data = await res.json();
  return data.items.map((r: any) => ({
    name: r.full_name,
    description: r.description || "",
    value: r.html_url,
  }));
});
```

### div — HTML Display

```typescript
// Display HTML with Tailwind CSS
await div(`
  <div class="flex flex-col items-center justify-center h-full p-8">
    <h1 class="text-4xl font-bold text-yellow-400">Hello!</h1>
    <p class="text-gray-400 mt-4">Press Escape to close</p>
  </div>
`);
```

### editor — Code Editor

```typescript
const code = await editor("// Write code here", "typescript");
const json = await editor('{"key": "value"}', "json");
```

### fields — Form Input

```typescript
const [name, email, age] = await fields([
  { name: "name", label: "Full Name" },
  { name: "email", label: "Email", type: "email" },
  { name: "age", label: "Age", type: "number" },
]);
```

### path — File Picker

```typescript
const file = await path("Choose a file");
const dir = await path("Choose a directory");
```

## Clipboard

```typescript
const text = await paste();       // Read clipboard
await copy("Hello clipboard!");   // Write to clipboard
```

## Shell Commands (Bun Shell)

```typescript
// Simple command
const output = await $`ls -la ~/Desktop`.text();

// Piped commands
const count = await $`find ~/Documents -name "*.md" | wc -l`.text();

// With error handling
try {
  await $`git status`;
} catch (e) {
  console.error("Not a git repo");
}
```

## File Operations (Bun APIs)

```typescript
// Read file
const content = await Bun.file("~/notes.txt").text();

// Write file
await Bun.write("~/output.txt", "Hello, world!");

// Check existence
const exists = await Bun.file("~/config.json").exists();

// Read JSON
const data = await Bun.file("~/data.json").json();

// Write JSON
await Bun.write("~/data.json", JSON.stringify(data, null, 2));
```

## Notifications & Feedback

```typescript
await notify("Task complete!");          // System notification
await copy("Copied to clipboard");       // Copy + implicit feedback
await open("https://example.com");       // Open URL/file in default app
```

## Extensions (Scriptlet Bundles)

Markdown files at `~/.scriptkit/kit/main/extensions/*.md`:

```markdown
---
name: My Tools
description: Useful shell commands
---

# My Tools

## Say Hello
<!-- name: Say Hello -->
<!-- description: Display a greeting -->
<!-- shortcut: ctrl h -->

\`\`\`bash
echo "Hello from Script Kit!"
\`\`\`

## Copy Date
<!-- name: Copy Date -->

\`\`\`bash
date +"%Y-%m-%d" | pbcopy && echo "Date copied"
\`\`\`
```

### Scriptlet Types

| Fence tag | Behavior |
|-----------|----------|
| `` ```bash `` | Run shell command |
| `` ```tool:name `` | Run TypeScript with SDK |
| `` ```template:name `` | Text template with `{{placeholders}}` |

## Metadata Fields

```typescript
export const metadata = {
  name: "Display Name",           // Required: shown in menu
  description: "What it does",    // Required: shown when focused
  shortcut: "cmd shift x",        // Global hotkey
  alias: "dn",                    // Quick search alias
  // trigger: "!hello",           // Snippet trigger
  // background: true,            // Run without showing UI
};
```

## Config Reference

`~/.scriptkit/kit/config.ts`:

```typescript
import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: { key: "Space", modifiers: ["command"] },
  editorFontSize: 14,
  terminalFontSize: 14,
  builtIns: {
    clipboardHistory: true,
    appLauncher: true,
  },
} satisfies Config;
```

## Theme Reference

`~/.scriptkit/kit/theme.json`:

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

## Path Reference

| Purpose | Path |
|---------|------|
| Scripts | `~/.scriptkit/kit/main/scripts/*.ts` |
| Extensions | `~/.scriptkit/kit/main/extensions/*.md` |
| Agents | `~/.scriptkit/kit/main/agents/*.md` |
| Config | `~/.scriptkit/kit/config.ts` |
| Theme | `~/.scriptkit/kit/theme.json` |
| SDK | `~/.scriptkit/sdk/kit-sdk.ts` (do not edit) |
| Logs | `~/.scriptkit/logs/` |
| Skills | `~/.scriptkit/skills/` |
| Examples | `~/.scriptkit/examples/scripts/` |

## Tab AI — Quick Terminal with Context Injection

Tab AI is not the old inline chat surface anymore. The primary Tab AI experience is a warm harness terminal rendered in `AppView::QuickTerminalView` via `TermPrompt`.

**Entry path:**
- Plain `Tab` opens the harness terminal, captures hierarchical context, and stages a schema-versioned `<scriptKitContext>` block in the running harness using `TabAiHarnessSubmissionMode::PasteOnly`.
- `Shift+Tab` in `AppView::ScriptList` with non-empty filter text opens the same harness surface and submits that filter text as `User intent:` using `TabAiHarnessSubmissionMode::Submit`.
- `Tab` / `Shift+Tab` inside `AppView::QuickTerminalView` are forwarded to the PTY. Do not describe them as focus-navigation keys once the harness terminal is open.

**Close semantics:**
- `Cmd+W` closes the wrapper and restores the previous view and focus.
- Plain `Escape` is forwarded to the PTY. The harness TUI owns Escape behavior.
- The footer hint strip advertises only `⌘W Close`.

**Runtime contract:**
- Entry path: `open_tab_ai_chat()` → `open_tab_ai_chat_with_entry_intent()` → `open_tab_ai_harness_terminal()`
- Harness session state: `TabAiHarnessSessionState`
- Harness config: `~/.scriptkit/harness.json`
- Supported backends: Claude Code, Codex, Gemini CLI, Copilot CLI, and custom commands
- `warmOnStartup` defaults to `true`
- Context assembly stays intact: `snapshot_tab_ai_ui()` + `capture_context_snapshot(CaptureContextOptions::tab_ai_submit())` + `build_tab_ai_context_from()`
- The `kit://context` MCP resource system still exists, but the landed default Tab flow is PTY-backed text injection
- `build_tab_ai_harness_submission()` emits `<scriptKitContext>` and optional `<scriptKitHints>`
- `PasteOnly` stages context on a fresh line and does not auto-submit
- `Submit` with a non-empty intent appends `User intent:` and submits immediately
- `Submit` without a non-empty intent appends `Await the user's next terminal input.`

**Do not describe as current behavior:**
- Do not describe the old inline chat entity or custom streaming UI as the primary Tab AI surface
- Do not describe the old inline chat or custom streaming UI as the default path
- Do not describe Claude Agent SDK V2 or screenshot attachment support as already landed in the default Tab flow; today's default flow is PTY-backed text injection

## DO NOT

- Use the old v1 SDK package — replaced by `@scriptkit/sdk`
- Use CommonJS imports — use ES `import`
- Use Node.js `fs` / `child_process` — use Bun APIs
- Use comment-based metadata — use `export const metadata`
- Edit `sdk/` files — managed by the app
- Reference legacy v1 paths — scripts live in `kit/main/scripts/`
