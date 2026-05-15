# Computer Use

Computer-use helpers give scripts safe visual awareness of macOS windows without granting them input control.

## Safety Model

The SDK exposes two helpers:

- `computer.listNativeWindows(options?)`
- `computer.captureNativeWindow(options)`

They are observation/capture-only. They do not:

- focus or activate apps
- send keyboard or mouse input
- move or resize windows
- click buttons
- read browser cookies or page internals

The helpers reuse Script Kit's own local MCP server discovery file at `~/.scriptkit/server.json`, so Script Kit must be running before a script calls `computer.*`.

## List Native Windows

```ts
import "@scriptkit/sdk";

export const metadata = {
  name: "List Windows",
  description: "Show native macOS windows visible to Script Kit",
};

const result = await computer.listNativeWindows({
  includeHidden: false,
  includeBackground: false,
});
const json = JSON.stringify(result, null, 2).replace(/[&<>"']/g, (char) => `&#${char.charCodeAt(0)};`);

await div(`<pre class="p-4 text-xs overflow-auto">${json}</pre>`);
```

The result includes running apps, window IDs, titles, bounds, z-order, visibility flags, and warnings. Use those receipts to pick a target before capturing.

## Capture an Exact Window

Capture requires the app PID and native window ID. Passing `expectedBundleId` adds an ownership check so a stale row cannot silently capture a different app.

```ts
import "@scriptkit/sdk";

export const metadata = {
  name: "Capture Window",
  description: "Select a native window and capture a structured receipt",
};

const listed = await computer.listNativeWindows();

const rows = listed.apps.flatMap(({ app, windows }) =>
  windows.map((window) => ({
    name: `${app.name}: ${window.title ?? "Untitled"}`,
    description: `${window.bounds.width}×${window.bounds.height}`,
    value: {
      pid: app.pid,
      nativeWindowId: window.nativeWindowId,
      expectedBundleId: app.bundleId ?? undefined,
    },
  }))
);

const target = await arg("Capture which window?", rows);

const receipt = await computer.captureNativeWindow({
  ...target,
  hiDpi: true,
  includeImage: false,
});
const json = JSON.stringify(receipt, null, 2).replace(/[&<>"']/g, (char) => `&#${char.charCodeAt(0)};`);

await div(`<pre class="p-4 text-xs overflow-auto">${json}</pre>`);
```

Set `includeImage: true` only when you need the PNG bytes as base64. The receipt always includes structured capture metadata such as dimensions, hash, HiDPI status, pixel audit, warnings, and a status code.

## Status Codes to Expect

`captureNativeWindow` can return statuses such as:

- `captured`
- `appNotFound`
- `windowNotFound`
- `ownershipMismatch`
- `notCaptureCandidate`
- `ambiguousNativeWindowRows`
- `ambiguousNativeWindowId`
- `permissionDenied`
- `blankImageRejected`
- `captureFailed`

Treat non-`captured` statuses as actionable diagnostics rather than generic failures.

## When to Use Computer Use

Good fits:

- capture a receipt for the current browser/editor window
- let Agent Chat inspect a screenshot the user explicitly selected
- compare visible window bounds before/after a workflow
- build local debugging tools that need screenshots without touching input

Poor fits:

- clicking through UI
- typing into another app
- scraping browser data behind the user's back
- bypassing macOS Screen Recording permission

## Troubleshooting

| Symptom | Try |
| --- | --- |
| `server.json` missing | Start Script Kit and rerun the script. |
| `permissionDenied` | Grant Screen Recording permission to the Script Kit app/binary in macOS settings, then restart it. |
| `ownershipMismatch` | Relist windows immediately before capture and pass the current `pid`, `nativeWindowId`, and `expectedBundleId`. |
| `blankImageRejected` | Make sure the target window is visible and not fully black/transparent. |
| Huge output | Keep `includeImage: false` until you actually need the base64 PNG. |

## Related Guides

- [SDK Scripting](./sdk-scripting.md)
- [MCP and Agent Context](./mcp-and-agent-context.md)
