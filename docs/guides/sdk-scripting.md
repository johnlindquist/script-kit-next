# SDK Scripting

Script Kit scripts are local TypeScript files executed by Bun that talk to the native app through the Script Kit SDK. The SDK is deliberately small: prompts, feedback, and integration points — bring your own utility libraries.

## Where Scripts Live

```bash
~/.scriptkit/plugins/main/scripts/     # executable scripts
~/.scriptkit/plugins/main/scriptlets/  # scriptlet bundles / snippets
```

## Minimal Script

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "Today",
  description: "Show today's date",
};

await div(`<h1 class="text-3xl p-8">${new Date().toLocaleDateString()}</h1>`);
```

## Metadata

The primary form is a typed `export const metadata = { ... }` object (typed as `ScriptMetadata`, so you get IDE completion):

```typescript
export const metadata = {
  name: "Deploy Preview",
  description: "Build and open the latest preview",
  author: "Your Name",
  shortcut: "cmd shift d",     // global keyboard shortcut
  alias: "dp",                 // short trigger in the launcher
  keyword: ":dep",             // text-expansion trigger ("snippet"/"expand" are aliases)
  placeholder: "Branch name",  // custom input placeholder
  tags: ["work", "deploy"],
  hidden: false,               // hide from the main list
  cron: "0 9 * * *",           // scheduled execution
  schedule: "every tuesday at 2pm", // natural-language schedule (converted to cron)
  watch: ["~/Downloads/*.zip"],// file-watch triggers
  background: false,           // run without UI
  system: false,               // system-level script
  fallback: false,             // offer this script when no search results match
  fallbackLabel: "Search docs for {input}",
};
```

Comment-based metadata is a compatibility-only fallback, read from the top of the file (`// Name:`, `// Description:`, `// Icon:`, `// Alias:`, `// Shortcut:` in the first 20 lines; `// Cron:` and `// Schedule:` in the first 30). Typed values win when both are present.

Avoid duplicate `shortcut`/`alias`/`keyword` values across scripts — colliding entries are excluded so dispatch never races.

## Prompt APIs

| API | Use |
| --- | --- |
| `await arg(prompt, choices?)` | Text input, optionally with searchable choices |
| `await div(html)` | Rich HTML/Tailwind display |
| `await editor(options?)` | Full multi-line editor |
| `await term(command?)` | Interactive terminal |
| `await drop(options?)` | Drag-and-drop file zone |
| `await template(str)` | Fill in a template string |
| `await fields(defs)` / `await form(html)` | Structured / custom HTML forms |
| `await path(options?)` | File/folder picker |
| `await hotkey(prompt?)` | Capture a keyboard shortcut |
| `await mic()` / `await webcam()` | Audio recording / camera capture |

```typescript
import "@scriptkit/sdk";

export const metadata = { name: "Pick a Service" };

const url = await arg("Pick a service", [
  { name: "Script Kit", value: "https://scriptkit.com", description: "Automation" },
  { name: "GPUI", value: "https://gpui.rs", description: "Native UI" },
]);

await div(`<a class="text-blue-500 underline" href="${url}">${url}</a>`);
```

## System, Clipboard, and Feedback

- `exec(command, args?)` — run a shell command and get its output.
- `clipboard.readText()`, `copy(text)`, `paste(text)` — clipboard access.
- `getSelectedText()` / `setSelectedText(text)` — read/replace the selection in the focused app.
- `readFile`, `writeFile`, `home(...paths)` — filesystem helpers.
- `hud(message)` — in-launcher overlay; `notify(message | { title, body })` — macOS Notification Center.
- `beep()` and `say(text)` are **experimental**: they return a dispatch receipt, but audible delivery isn't verified.

## Automation Receipts

Scripts can inspect the app's UI state without screenshots, and run deterministic UI transactions without sleeps:

```typescript
const state = await getState();
const elements = await getElements(100);

await batch([
  { type: "setInput", text: "sdk" },
  { type: "waitFor", condition: "choicesRendered", timeout: 1000 },
  { type: "selectByValue", value: "builtin/sdk-reference", submit: true },
]);
```

## Bring Your Own Packages

```bash
cd ~/.scriptkit
bun add zod date-fns
```

```typescript
import "@scriptkit/sdk";
import { z } from "zod";

const payload = await arg("Paste JSON");
const parsed = z.object({ title: z.string() }).parse(JSON.parse(payload));
await div(`<h1>${parsed.title}</h1>`);
```

## Observation Helpers

`computer.listNativeWindows()` and `computer.captureNativeWindow(...)` give scripts observation-only access to native macOS windows — see [Computer Use](./computer-use.md). MCP client helpers (`mcp.listTools`, `mcp.call`, ...) are covered in [MCP and Agent Context](./mcp-and-agent-context.md).

## Discover APIs In-App

- Search **`sdk`** in the launcher and open **SDK Reference** — it is generated from the same Rust-owned catalog as the `kit://sdk-reference` MCP resource, and marks unsupported/experimental APIs so you don't paste dead code.
- Search **`template`** and use **New Script from Template** to start from a working starter instead of a blank file.

## Related

- [Getting Started](./getting-started.md) — build, hotkey, first script
- [Main Menu Input](./main-menu-input.md) — aliases, keywords, capture handlers, command heads
