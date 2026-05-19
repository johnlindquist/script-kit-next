# MCP and Agent Context

Script Kit GPUI uses MCP in both directions: the running app exposes local resources/tools to agents, and scripts/Agent Chat can call configured MCP servers.

## Two MCP Roles

| Role | What it means |
| --- | --- |
| Script Kit as an MCP server | External agents read `kit://...` resources and call safe app tools over the app's local HTTP MCP server. |
| Script Kit as an MCP client | Scripts and Agent Chat use `~/.scriptkit/config.ts` MCP server definitions through SDK helpers and agent configuration. |

## Script Kit as an MCP Server

When the app runs, it writes discovery metadata to:

```bash
~/.scriptkit/server.json
```

That file contains the local URL and bearer token for the live app server. Treat it as sensitive. Do not paste it into issues, logs, screenshots, commits, or shared docs.

The `.url` value is the base server URL; JSON-RPC clients use the `/rpc` endpoint. The app may remove or rotate discovery metadata when it stops or restarts, so always reread `server.json` instead of hard-coding a port or token.

See [Codex MCP Setup](../codex-mcp-setup.md) for a complete registration and verification flow.

## High-Value Resources

| Resource | Use |
| --- | --- |
| `kit://context` | AI-relevant desktop snapshot |
| `kit://context/schema` | self-describing schema, profiles, query flags, diagnostics |
| `kit://sdk-reference` | concise SDK function reference and script conventions |
| `kit://script-templates` | starter templates shared with the launcher |
| `kit://scripts` | schema-versioned discovered script metadata |
| `kit://scriptlets` | schema-versioned scriptlet metadata |
| `kit://clipboard-history` | newest clipboard entries with previews/metadata |
| `kit://dictation` | most recent dictated text envelope |
| `kit://dictation-history` | saved dictation summaries or a transcript by `?id=` |
| `kit://focused-item` | active surface selection/focus metadata |
| `kit://state` | app state for safe setup/proof checks |

`kit://context` supports profiles and field flags:

```text
kit://context
kit://context?profile=minimal
kit://context?profile=full
kit://context?diagnostics=1
kit://context?selectedText=1&browserUrl=1&menuBar=0
```

Common fields include selected text, frontmost app, menu bar, browser URL, focused window, screenshot, and panel screenshot.

## High-Value Tools

Mutation tools return structured JSON inside MCP text content and are audited to `~/.scriptkit/mcp-audit.jsonl`. Destructive tools require `confirm:true`, and token scopes can narrow access by domain.

| Tool group | Examples | Scope |
| --- | --- | --- |
| App/runtime control | `kit/show`, `kit/hide`, `kit/state`, `kit/trigger_builtin` | `ui:control` for show/hide/trigger |
| Notes | `kit/notes_create`, `kit/notes_update`, `kit/notes_delete` | `notes:write` |
| Scripts | `kit/scripts_create`, `kit/scripts_update`, `kit/scripts_delete`, `kit/scripts_run` | `scripts:write`, `scripts:run` |
| Clipboard history | `kit/clipboard_copy`, `kit/clipboard_pin`, `kit/clipboard_unpin`, `kit/clipboard_delete`, `kit/clipboard_clear_unpinned` | `clipboard:write` |
| Config/preferences | `kit/config_get`, `kit/config_set`, `kit/config_validate`, `kit/config_reset`, `kit/config_set_command_shortcut` | `config:read`, `config:write` |

Use `kit://trigger-builtins` to discover canonical `builtin/...` IDs before calling `kit/trigger_builtin`.

## Agent Chat Context

Open Agent Chat with `Tab` from the launcher. Agent Chat can stage context from the current surface and attach additional context parts at submit time.

Useful slash commands:

| Command | Attached context |
| --- | --- |
| `/context` | minimal desktop snapshot |
| `/context-full` | full desktop snapshot |
| `/selection` | selected text |
| `/browser` | browser URL |
| `/window` | focused window info |

Context parts can also be files, focused targets, text blocks, skill files, and resource URIs such as `kit://dictation-history?id=...`.

Resolution is tolerant: if one attachment fails, successful parts still submit and the failure is recorded in the context resolution receipt.

## Script Kit as an MCP Client

Add MCP servers to `~/.scriptkit/config.ts`:

```ts
export default {
  mcp: {
    enabled: true,
    servers: {
      localTools: {
        transport: "stdio",
        command: "my-mcp-server",
        args: [],
        env: {},
      },
      remoteTools: {
        transport: "http",
        endpoint: "http://127.0.0.1:3000/rpc",
        headers: {
          authorization: `Bearer ${process.env.MY_MCP_TOKEN ?? ""}`,
        },
      },
    },
  },
};
```

Then call tools from a script:

```ts
import "@scriptkit/sdk";

export const metadata = {
  name: "Ask MCP",
  description: "Discover and call configured MCP tools",
};

const matches = await mcp.discover("calendar");
const selected = await arg(
  "Pick a tool",
  matches.map((tool) => ({
    name: `${tool.serverId}/${tool.name}`,
    description: tool.description,
    value: tool,
  }))
);

const result = await mcp.call(selected.serverId, selected.name, {});
const json = JSON.stringify(result, null, 2).replace(/[&<>"']/g, (char) => `&#${char.charCodeAt(0)};`);

await div(`<pre class="p-4 text-xs overflow-auto">${json}</pre>`);
```

## Safe Setup Check

Use read-only checks first:

```bash
SERVER_JSON="$HOME/.scriptkit/server.json"
test -f "$SERVER_JSON"
jq -e '.url and .token and .version and (.capabilities.tools == true)' "$SERVER_JSON"
```

For direct JSON-RPC verification, prefer `kit/state` or resource reads. Avoid show/hide tools unless you intentionally want to change app visibility.

## Best Practices

- Reread `~/.scriptkit/server.json` after every app restart.
- Never copy bearer-token values into static config files.
- Prefer `kit://context?profile=minimal` unless the agent really needs more.
- Use `kit://context?diagnostics=1` when debugging missing fields.
- Use resource URIs for read-only context; use tools only when an action is required.
- In scripts, call `mcp.discover(query)` before hard-coding a tool name if the server set is user-configurable.
