# Computer Use

Computer-use helpers give scripts and agents safe visual awareness of macOS windows — listing and capturing exact native windows — **without** granting them input control. To fill (but never submit) a native Save sheet, see [Fill a Save sheet filename](desktop-filename.md). That separate `desktop/*` interface requires its own grant.

## Safety Model

Computer use in Script Kit is observation-only. The helpers do **not**:

- focus or activate apps
- synthesize keyboard or mouse input
- move, resize, or close windows
- read browser cookies or page internals

## Readiness

Computer use is gated on two macOS permissions:

- **Accessibility** — required to observe other apps
- **Screen Recording** — required for window capture

Check readiness before use. The launcher's **Permissions wizard** (search `permissions`) walks through granting both, and agents can read the receipt directly:

```text
kit://computer-use/readiness
```

The receipt reports `ready`, per-permission status, and machine-readable attention codes (`accessibility_missing`, `screen_recording_missing`).

## SDK: List Native Windows

```typescript
import "@scriptkit/sdk";

export const metadata = { name: "List Windows" };

const result = await computer.listNativeWindows({
  includeHidden: false,
  includeBackground: false,
});

const json = JSON.stringify(result, null, 2)
  .replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);
await div(`<pre class="p-4 text-xs overflow-auto">${json}</pre>`);
```

The result includes running apps, native window IDs, titles, bounds, visibility flags, and warnings — use these receipts to pick a target before capturing.

## SDK: Capture an Exact Window

Capture requires the app `pid` and `nativeWindowId`. Pass `expectedBundleId` as an ownership check so a stale row can't silently capture a different app:

```typescript
import "@scriptkit/sdk";

const listed = await computer.listNativeWindows();
const rows = listed.apps.flatMap(({ app, windows }) =>
  windows.map((w) => ({
    name: `${app.name}: ${w.title ?? "Untitled"}`,
    value: {
      pid: app.pid,
      nativeWindowId: w.nativeWindowId,
      expectedBundleId: app.bundleId ?? undefined,
    },
  }))
);

const target = await arg("Capture which window?", rows);
const receipt = await computer.captureNativeWindow({
  ...target,
  hiDpi: true,
  includeImage: false, // set true only when you need the base64 PNG
});
```

The receipt always includes structured metadata (dimensions, pixel audit, warnings) and a status code:

`captured` · `appNotFound` · `windowNotFound` · `ownershipMismatch` · `notCaptureCandidate` · `ambiguousNativeWindowRows` · `ambiguousNativeWindowId` · `permissionDenied` · `blankImageRejected` · `captureFailed`

Treat non-`captured` statuses as actionable diagnostics, not generic failures.

## MCP Computer-Use Tools

External agents get the same observation surface over the app's MCP server (see [MCP and Agent Context](./mcp-and-agent-context.md)). The `computer/` namespace includes `computer/see` (state-first observation of Script Kit's own windows), `computer/list_native_windows`, `computer/capture_native_window`, `computer/list_windows`, `computer/get_focused_window`, `computer/list_apps`, `computer/get_frontmost_app`, `computer/list_menus`, and related get/list variants. The SDK's `computer.*` helpers call these tools, so Kit must be running. A normal personal script uses the operator connection in its active workspace’s `server.json`.

To run a script with a particular grant, set `SCRIPT_KIT_AGENT_CREDENTIAL` to its absolute private token-file path, or supply `SCRIPT_KIT_MCP_TOKEN`. The file takes precedence, even if it cannot be read. On macOS use a regular file readable only by its owner, not a link; give its path without `..`. With a supplied credential the SDK reads only the address from `server-location.json`, not the operator token. To use another loopback server, set `SCRIPT_KIT_MCP_ENDPOINT` and supply that server’s credential. A missing credential file, empty token, or denied call does not trigger a retry as the operator. Listing and capturing remain separate permissions; a notes-only grant permits neither. Choosing a credential does not grant macOS permissions or sandbox a script running locally as you.

## Good and Poor Fits

Good fits: capturing a receipt of the current editor/browser window, letting Agent Chat inspect a screenshot the user explicitly selected, comparing window bounds before/after a workflow, local debugging tools.

Poor fits: clicking through UI, typing into other apps, scraping data behind the user's back — input synthesis is deliberately not part of this surface.

## Troubleshooting

| Symptom | Try |
| --- | --- |
| `server.json` missing | Start Script Kit, then rerun the script |
| `permissionDenied` | Grant Screen Recording to Script Kit in macOS Settings, then restart it |
| `ownershipMismatch` | Re-list windows immediately before capture; pass current `pid`, `nativeWindowId`, `expectedBundleId` |
| `blankImageRejected` | Make sure the target window is visible and not fully black/transparent |
| Huge output | Keep `includeImage: false` until you actually need pixels |

## Related

- [SDK Scripting](./sdk-scripting.md) — the rest of the SDK
- [MCP and Agent Context](./mcp-and-agent-context.md) — the server these tools live on
