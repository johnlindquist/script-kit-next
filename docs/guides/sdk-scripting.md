# SDK Scripting

Script Kit scripts are local TypeScript files that run through Bun and talk to the native app through the Script Kit SDK.

## Where Scripts Live

The default personal plugin is:

```bash
~/.scriptkit/plugins/main/scripts/
```

Scriptlets live beside scripts:

```bash
~/.scriptkit/plugins/main/scriptlets/
```

## Minimal Script

```ts
import "@scriptkit/sdk";

export const metadata = {
  name: "Today",
  description: "Show today's date",
  keyword: "date",
};

await div(`
  <div class="p-8">
    <h1 class="text-3xl font-bold">${new Date().toLocaleDateString()}</h1>
  </div>
`);
```

## Metadata

Use `export const metadata = { ... }` for script properties:

```ts
export const metadata = {
  name: "Deploy Preview",
  description: "Build and open the latest preview",
  author: "Your Name",
  shortcut: "cmd+shift+d",
  tags: ["work", "deploy"],
};
```

Avoid duplicate `shortcut`, `alias`, `keyword`, and `trigger` values across scripts. Script Kit validates loaded scripts and excludes colliding entries so dispatch never races on ambiguous bindings.

## Prompt APIs

| API | Use |
| --- | --- |
| `arg()` | text input and searchable choices |
| `div()` | rich HTML/Tailwind display |
| `editor()` | multiline editor |
| `fields()` | structured fields |
| `form()` | custom HTML form |
| `path()` | file/folder picker |
| `drop()` | drag-and-drop files |
| `hotkey()` | capture a keyboard shortcut |
| `term()` | interactive terminal |
| `mic()` | audio recording prompt |
| `webcam()` | camera capture prompt |

Example:

```ts
import "@scriptkit/sdk";

export const metadata = {
  name: "Pick a Service",
  description: "Choose a service and show its docs link",
};

const service = await arg("Pick a service", [
  { name: "Script Kit", value: "https://scriptkit.com", description: "Automation" },
  { name: "GPUI", value: "https://gpui.rs", description: "Native UI" },
]);

await div(`<a class="text-blue-500 underline" href="${service}">${service}</a>`);
```

## Bring Your Own Packages

Install dependencies into `~/.scriptkit`:

```bash
cd ~/.scriptkit
bun add zod date-fns
```

Then import them normally:

```ts
import "@scriptkit/sdk";
import { z } from "zod";

const payload = await arg("Paste JSON");
const parsed = z.object({ title: z.string() }).parse(JSON.parse(payload));

await div(`<h1>${parsed.title}</h1>`);
```

## Automation Receipts

Scripts can inspect Script Kit UI state without screenshots:

```ts
import "@scriptkit/sdk";

export const metadata = {
  name: "Inspect Script Kit",
  description: "Show visible Script Kit state and elements",
};

const state = await getState();
const elements = await getElements(100);
const json = JSON.stringify({ state, elements }, null, 2).replace(/[&<>"']/g, (char) => `&#${char.charCodeAt(0)};`);

await div(`
  <pre class="p-4 text-xs overflow-auto">${json}</pre>
`);
```

For deterministic UI transactions, prefer `waitFor` and `batch` over fixed sleeps:

```ts
await batch([
  { type: "setInput", text: "sdk" },
  { type: "waitFor", condition: "choicesRendered", timeout: 1000 },
  { type: "selectByValue", value: "builtin/sdk-reference", submit: true },
]);
```

## MCP Client Helpers

Configure MCP servers in `~/.scriptkit/config.ts`, then call them from scripts:

```ts
export default {
  mcp: {
    enabled: true,
    servers: {
      myTools: {
        transport: "stdio",
        command: "my-mcp-server",
        args: [],
        env: {},
      },
    },
  },
};
```

Script:

```ts
import "@scriptkit/sdk";

export const metadata = {
  name: "MCP Tools",
  description: "List configured MCP tools",
};

const tools = await mcp.listTools();
const json = JSON.stringify(tools, null, 2).replace(/[&<>"']/g, (char) => `&#${char.charCodeAt(0)};`);

await div(`
  <pre class="p-4 text-xs overflow-auto">${json}</pre>
`);
```

Useful helpers:

- `mcp.listServers()`
- `mcp.getServer(id)`
- `mcp.listTools(serverId?)`
- `mcp.discover(query)`
- `mcp.call(serverId, toolName, args?)`

## Computer-Use Helpers

Computer-use helpers let scripts safely observe native windows:

```ts
import "@scriptkit/sdk";

const windows = await computer.listNativeWindows();
const rows = windows.apps.flatMap(({ app, windows }) =>
  windows.map((window) => ({
    name: `${app.name}: ${window.title ?? "Untitled"}`,
    value: { pid: app.pid, nativeWindowId: window.nativeWindowId, bundleId: app.bundleId },
  }))
);

const target = await arg("Capture a window", rows);

const capture = await computer.captureNativeWindow({
  pid: target.pid,
  nativeWindowId: target.nativeWindowId,
  expectedBundleId: target.bundleId ?? undefined,
  includeImage: false,
});
const json = JSON.stringify(capture, null, 2).replace(/[&<>"']/g, (char) => `&#${char.charCodeAt(0)};`);

await div(`<pre class="p-4 text-xs">${json}</pre>`);
```

See [Computer Use](./computer-use.md).

## Feedback APIs

Use `hud()` for in-launcher feedback and `notify()` for macOS Notification Center:

```ts
hud("Saved");
await notify({ title: "Script Kit", body: "Background job finished" });
```

## Discover APIs In-App

Search `sdk` in the launcher and open **SDK Reference**. It is generated from the same Rust-owned catalog as the `kit://sdk-reference` MCP resource and marks unsupported APIs before you paste them into scripts.
