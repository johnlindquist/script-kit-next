# Fill a Save sheet filename

Leave a native Save or Export sheet open, summon Script Kit from its owning app and type **Name this export “Workshop outline”; don’t save.** Select **Inspect filename**, check the name and the window, then choose **Fill filename**. **Leave unchanged** or Esc dismisses the preview. The filename changes; Script Kit does not click Save, paste, take a screenshot or move the pointer.

This route requires a quoted filename, the configured Jev text reader and an identifiable originating macOS window. Jev classifies the requested effect; Kit supplies the quoted value verbatim and only offers inspection for a fill-only reading. Inspection starts when the row runs, not while typing. Release a different staged subject first. This first version supports an attached, English-language native sheet with a Save or Export button and exactly one identifiable writable filename field. Web forms, custom dialogs, standalone Save windows, saving files on disk and selecting a destination folder are not supported.

## Optional helper

From the Script Kit checkout, run `bash scripts/setup-desktop-fill.sh`. This installs both files from the SHA-256-pinned official macOS `agent-desktop` **1.0.2** release under `${SK_PATH:-$HOME/.scriptkit}/tools/agent-desktop/1.0.2/`. An unrecognized existing installation is never overwritten. For a separately managed copy, set `SCRIPT_KIT_AGENT_DESKTOP_BIN` to the absolute executable path in the app’s launch environment. The adapter accepts the v1.0.2 CLI with protocol envelope 2.5. It uses a dedicated ref namespace and local provider state under the kit’s `db/agent-desktop`; native accessibility data there may contain private content. Keep it local.

The provider needs macOS Accessibility access. Setup does not grant it. No Screen Recording permission is needed for this route. The optional helper is disabled in noninteractive runs and cannot operate the desktop in owned evaluation. The owned-renderer fixture uses an in-memory sheet instead.

## Agent interface

Use Script Kit’s authenticated MCP `POST /rpc` and `tools/list`. The three `desktop/filename_*` tools require a per-agent bearer explicitly granted **`desktop:fill`**. The default server token, `mcp:*`, `computer:act` and read-only tokens do **not** confer this permission. To grant it, use the server’s existing scoped-token format: an operator-managed `${SK_PATH:-$HOME/.scriptkit}/agents/<agent>.json` with `tokenHash` (lowercase SHA-256 of that agent’s high-entropy bearer) and `scopes` containing `desktop:fill`. Keep the bearer private. Use a different bearer per caller; two clients sharing one token are the same principal. Remove the scope to revoke it. The existing `computer/*` tools remain observation-only.

1. From `computer/list_native_windows`, choose the specific app, PID and **nativeWindowId**, not a list index, a window title or the active pane. Pass them to `desktop/filename_propose`, e.g. `{"app":"TextEdit","pid":4321,"nativeWindowId":765,"filename":"Workshop outline"}` (illustrative IDs). The response names the window and field and returns `proposal.proposalId`. No write occurs.
2. After the calling workflow authorizes the proposed filename and target, send `{"proposalId":"<id>"}` to `desktop/filename_apply`. Caller input cannot change the target, verb or value at apply time. Proposals expire after 60 seconds. A second apply with the same ID returns the prior state without writing again.
3. If a response is lost, call `desktop/filename_status` with the same ID and bearer. `verified` means the value was read back and the sheet was observed open; it does not mean a document was saved. `refused` means the pre-write checks failed. `uncertain` means delivery could not be verified: **do not retry or issue a replacement write automatically**. Unknown/expired is not evidence that nothing happened.

The provider rechecks the process instance, native window, field and old value. A read-before-write cannot prevent another application or a person changing the field in the interval before the native write; this is not an atomic compare-and-set. Use the app’s own transactional API if that guarantee matters.
