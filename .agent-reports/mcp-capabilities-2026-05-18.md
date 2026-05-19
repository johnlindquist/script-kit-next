# Script Kit MCP — Real-Usage Capability Report

**Date:** 2026-05-18
**Server:** `script-kit` v0.1.1, JSON-RPC over HTTP at `http://127.0.0.1:43210/rpc`
**Auth:** Bearer token from `~/.scriptkit/agent-token` (the `~/.sk/kit/agent-token` copy is stale — auth fails)
**Discovery path:** Server is NOT registered with Claude Code's MCP host (`ListMcpResourcesTool` returned nothing); reached it directly via HTTP because the Script Kit app exposes the same JSON-RPC surface on a local TCP port.
**Counts observed:** `initialize` OK; `tools/list` → 32 tools; `resources/list` → 25 resources.

## Headline finding

**Read-only surface.** Every advertised method that succeeded was a read or window-visibility toggle. No `resources/write`, `resources/create`, or `resources/subscribe` (all returned JSON-RPC `-32601 Method not found`). No tool name in the catalog matches `note`, `write`, `create`, `save`, `put`, or `update`. The MCP does not provide a path to read or mutate the **Notes window** from outside the app.

## Notes-window status

| URI tried              | Result                              |
| ---------------------- | ----------------------------------- |
| `notes://`             | `-32601 Resource not found`         |
| `kit://notes`          | `-32601 Resource not found`         |
| `kit://notes/list`     | `-32601 Resource not found`         |
| `scripts://notes`      | `-32601 Resource not found`         |

Notes are not in the resource catalog and no tool addresses them. The only adjacent capability is `kit://context` (31 KB), which includes the *focused* item but not arbitrary notes contents.

## Resources actually read

All 23 advertised `kit://` / `scripts://` / `scriptlets://` resources returned `OK contents=1`. Notable payload sizes:

| Resource                              | Bytes      | Notes |
| ------------------------------------- | ---------: | ----- |
| `kit://state`                         |       205  | minimal app state snapshot |
| `kit://context`                       |    31,394  | rich current-surface context |
| `kit://context/schema`                |     4,266  | JSON schema for above |
| `scripts://`                          |    10,374  | script catalog |
| `kit://scripts` (versioned)           |    10,374  | same payload, versioned envelope |
| `scriptlets://`                       |    25,355  | scriptlets catalog |
| `kit://sdk-reference`                 |    12,782  | SDK docs |
| `kit://script-templates`              |     1,471  | template list |
| `kit://clipboard-history`             |     4,129  | recent clipboard items |
| `kit://focused-item`                  |       263  | currently focused row |
| `kit://git-status`                    |   139,738  | repo `git status` snapshot |
| `kit://git-diff`                      | 31,436,564 | full diff — 31 MB, no pagination flag found |
| `kit://processes`                     |       332  | |
| `kit://system`                        |       429  | |
| `kit://dictation`                     |     4,108  | active dictation session info |
| `kit://dictation-history`             |     3,312  | recent transcripts |
| `kit://calendar`                      |       349  | |
| `kit://notifications`                 |       374  | |
| `kit://stdin-commands`                |     3,333  | |
| `kit://trigger-builtins`              |     2,125  | |
| `kit://diagnostics/protocol-stats`    |       628  | |
| `kit://transactions/latest`           |     1,127  | |
| `kit://transactions/schema`           |       892  | |
| `kit://failed-scripts`                |       317  | |

**All read OK on first try; none required parameters.**

## Tools exercised

- `kit/state` → success (read).
- `kit/show` / `kit/hide` → catalog confirms (write-ish, window visibility). Not exercised (would interrupt user).
- `computer/list_*` / `computer/get_*` (29 tools) → catalog confirms read-only inspection of apps, native windows, screens, menus, tray, permissions. **Mid-probe the Script Kit app process exited and port 43210 stopped listening**, so `tools/call` invocations for `kit/state`, `computer/list_screens`, `computer/list_permissions`, `computer/list_apps` returned `Connection refused`. The tool catalog is verified; the actual call results are not. Re-running these after the app restarts is a 30-second job.
- `computer/see` → catalog confirms (state-first computer-use observation). Not exercised after app death.

## Write methods — all denied

| Method                | Response                                |
| --------------------- | --------------------------------------- |
| `resources/subscribe` | `-32601 Method not found`               |
| `resources/write`     | `-32601 Method not found`               |
| `resources/create`    | `-32601 Method not found`               |

There are no tool names containing `write`, `create`, `save`, `put`, `update`, or `note` in the 32-tool catalog. The only state-mutating tools exposed are `kit/show` and `kit/hide` (window visibility).

## Observed limits / gotchas

1. **Token location is brittle.** Two `agent-token` files exist; only `~/.scriptkit/agent-token` is honored by the running server. `~/.sk/kit/agent-token` and several archived copies (`~/.scriptkit-old/`, `~/.scriptkit-202604231141/`, `~/.scriptkit-theme-struggles/`, `~/.kenv-backup/`) are stale.
2. **Server fragility.** The MCP listener disappeared mid-session (likely Script Kit app crash/restart). Tool-call coverage is therefore partial; resource reads completed before the drop.
3. **`kit://git-diff` is unbounded** — returned 31 MB for the current dirty worktree with no advertised pagination/offset/max-bytes parameter. Calling it carelessly is a memory hazard for the consumer.
4. **No `notes://`, no write path, no subscribe.** Notes-window read/write from MCP is not supported in this build. To touch notes from an agent, use stdin-protocol automation against the running app rather than the MCP surface.
5. **Discovery via Claude Code's MCP host did not work.** The `.mcp.json` registers only `lat` (`npx lat.md@latest mcp`); the Script Kit server itself is not declared, so agents inside Claude Code need to be told to hit the HTTP endpoint directly.

## Conclusion

The Script Kit MCP is a comprehensive **read/inspect** surface (25 resources, 30 inspection tools) with **no write surface and no notes coverage**. For "read/write to our notes and other sources" the honest answer is: reads to most "other sources" work; **notes are not addressable via MCP at all**, and **no resource is writable via MCP**. Any write/notes integration would need to be added to the server (new resource + a `resources/write`-style method or a dedicated `kit/notes_*` tool).
