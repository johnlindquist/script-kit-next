---
name: configure-mcp
description: Configure external MCP servers for scripts and Agent Chat. Use when the user wants to add, remove, enable, disable, or inspect MCP servers in Script Kit.
---

# Configure MCP

Manage Script Kit's external MCP server definitions in `~/.scriptkit/config.ts`.

## What This Controls

The `mcp` block in `config.ts` is the shared source of truth for:

- `mcp.*` script SDK calls
- Agent Chat access to Script Kit-managed MCP servers

Script Kit syncs enabled servers into Claude Code's user MCP config when Agent Chat loads that provider.

## File and Key

| File | Key | Purpose |
|------|-----|---------|
| `~/.scriptkit/config.ts` | `mcp` | External MCP server definitions |

## Shape

```typescript
mcp: {
  enabled: true,
  servers: {
    notion: {
      transport: "http",
      endpoint: "https://mcp.notion.com/mcp",
    },
    github: {
      transport: "stdio",
      command: "npx",
      args: ["-y", "@modelcontextprotocol/server-github"],
    },
  },
},
```

## Supported Transports

### HTTP

```typescript
linear: {
  transport: "http",
  endpoint: "https://mcp.linear.app/sse",
  headers: {
    Authorization: `Bearer ${process.env.LINEAR_API_KEY ?? ""}`,
  },
},
```

Fields:
- `endpoint`
- optional `headers`
- optional `enabled`
- optional `name`
- optional `description`

### stdio

```typescript
filesystem: {
  transport: "stdio",
  command: "npx",
  args: ["-y", "@modelcontextprotocol/server-filesystem", "~/Documents"],
  env: {},
  cwd: "~/.scriptkit",
},
```

Fields:
- `command`
- optional `args`
- optional `env`
- optional `cwd`
- optional `enabled`
- optional `name`
- optional `description`

## Workflow

1. Edit `~/.scriptkit/config.ts`
2. Add or update entries under `mcp.servers`
3. Keep server ids stable; they are how scripts address servers with `mcp.call("server-id", ...)`
4. Disable a server with `enabled: false` instead of deleting it when the user wants a reversible change

## Script Usage

```typescript
const servers = await mcp.listServers()
const tools = await mcp.listTools("github")
const result = await mcp.call("github", "search_repositories", { query: "script kit" })
```

## Common Mistakes

- Editing `~/.scriptkit/config.ts` instead of `~/.scriptkit/config.ts`
- Putting MCP server config under `claudeCode` instead of top-level `mcp`
- Renaming a server id without updating scripts that call it
- Leaving `command` or `endpoint` empty
- Storing secrets inline when `process.env.*` is available

## Related Skills

- `config` — broader launcher and workspace configuration
- `agent_chat-chat` — Agent Chat behavior and agent integration
