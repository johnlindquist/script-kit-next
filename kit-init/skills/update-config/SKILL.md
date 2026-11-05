---
name: update-config
description: Update Script Kit configuration files — hotkeys, theme, layout, dictation, Claude Code, and workspace settings.
---

# Update Config

The active workspace is `$SK_PATH`, or `~/.scriptkit` when unset. Use `kit/config_get` for the exact key and revision, then `kit/config_set` with a JSON value and that `expectedRevision`. Reread after a stale revision; never retry a stale proposal blindly. Inspect `impact` and `manualOnly` in `kit/config_list` before proposing changes. A valid value is not permission.

The installed CLI in `$SK_PATH/sdk/config-cli.ts` and adjacent `kit-sdk-config.ts` SDK use the same data-only writer. The writer supports a literal default-exported object (including the stock type import, comments and `satisfies Config`). Executable or ambiguous TypeScript, unreviewed paths and changes that would discard comments are refused. A refusal leaves the file alone: do not bypass it with a script, a regex replacement, or by importing the user's config. Executable paths, MCP server definitions, credentials, permissions and remote access are manual settings, not ordinary agent preferences.

## Launcher behavior versus a global setting

To brighten only the pinned “Stays open” label or put an existing action first for one subject, use the launcher preference catalog, not `theme.json` or `config.ts`. Profiles explicitly granted `launcher_read` can inspect, preview, explain, and read the adopted revision. Only `launcher_apply` can save a revision-bound request. If these tools are unavailable, say so; do not reach for shell access. Editor and terminal text size still use the ordinary config settings.

## Files That Matter

| File | Purpose | Reload |
|------|---------|--------|
| `~/.scriptkit/plugins/scriptkit/skills/` | Agent-readable skills | Read from Script Kit plugin |
| `~/.scriptkit/config.ts` | Full config: hotkeys, built-ins, runtime preferences, Claude Code | Auto-reloads |
| `~/.scriptkit/theme.json` | Theme colors | Auto-reloads |

## Editing one setting

For the settings SDK, pass the exact key and revision you read: `await config.set(key, value, revision)` or `await config.reset(key, revision)`. An empty selection or a supplied `undefined` revision is refused; do not remove an argument to make the edit go through. `config.get()` reads all settings. To deliberately reset all eligible settings at the reviewed revision, use `await config.resetAll(revision)`, not `config.reset(undefined, revision)`.

## What Goes Where

| Setting | File | Key |
|---------|------|-----|
| Launcher hotkey | `config.ts` | `hotkey` |
| AI/Logs/Dictation shortcuts | `config.ts` | `aiHotkey`, `logsHotkey`, `dictationHotkey` |
| Dictation microphone | `config.ts` | `dictation.selectedDeviceId` |
| Theme preset | `config.ts` | `theme.presetId` |
| Agent Chat defaults | `config.ts` | `ai.selectedAgentChatAgentId`, `ai.selectedModelId` |
| Snap mode | `config.ts` | `windowManagement.snapMode` |
| External MCP servers | `config.ts` | `mcp` |
| Theme colors | `theme.json` | `colors.*` |
| Claude Code | `config.ts` | `claudeCode` |

## Hotkey Format

Use `KeyboardEvent.code` values. Valid modifiers: `meta`, `ctrl`, `alt`, `shift`.

Common keys: `Semicolon`, `Space`, `Enter`, `KeyK`, `KeyL`, `KeyD`, `Digit1`.

**Do not** use `command`/`control` — use `meta`/`ctrl`.

## UI Settings (config.ts)

```typescript
editor: "code",
padding: { top: 8, left: 12, right: 12 },
editorFontSize: 16,
terminalFontSize: 14,
uiScale: 1.0,
builtIns: {
  clipboardHistory: true,
  appLauncher: true,
  windowSwitcher: true,
},
```

## Command Overrides (config.ts)

```typescript
commands: {
  "script/my-workflow": {
    shortcut: { modifiers: ["meta", "shift"], key: "KeyW" }
  },
  "builtin/app-launcher": { hidden: true },
  "builtin/empty-trash": { confirmationRequired: true }
}
```

Command deeplinks: `scriptkit://commands/{id}`

## Auxiliary Hotkeys

- `notesHotkey` — no default; set explicitly
- `aiHotkey` — defaults to Cmd+Shift+Space when enabled
- `logsHotkey` — defaults to Cmd+Shift+L when enabled
- `dictationHotkey` — no default; set explicitly
- All have `*Enabled` boolean (default `true`)

## Notes Folder

- `notesFolder` — where notes live (default `~/.scriptkit/brain`): `days/YYYY-MM-DD.md` and `notes/<Title>.md`, plain Markdown other apps (Obsidian) can open. `~` and paths relative to `~/.scriptkit` work. Takes effect the next time Script Kit starts.

## Runtime Preference Groups

```typescript
theme: { presetId: "nord" },
dictation: { selectedDeviceId: "usb-mic" },
ai: {
  selectedAgentChatAgentId: "codex-agent_chat",
  selectedModelId: "gpt-5.4",
},
windowManagement: { snapMode: "expanded" },
```

## MCP Servers

Use the top-level `mcp` key in `config.ts` for external MCP servers shared by scripts and Agent Chat.

```typescript
mcp: {
  enabled: true,
  servers: {
    github: {
      transport: "stdio",
      command: "npx",
      args: ["-y", "@modelcontextprotocol/server-github"],
    },
  },
},
```

For MCP-specific workflows, use the `mcp` skill.

## Common Mistakes

- Putting skills at the workspace root instead of in `~/.scriptkit/plugins/scriptkit/skills/`
- Editing `~/.scriptkit/config.ts` instead of `~/.scriptkit/config.ts`
- Using `command`/`control` instead of `meta`/`ctrl`
- Editing `theme.json` when you meant to change `theme.presetId` in `config.ts`

## References

See `references/config-examples.md` for full config.ts example, theme file format, frecency settings, watcher config, layout defaults, and Claude Code provider options.
