---
name: config
description: Manage Script Kit configuration files — hotkeys, theme, layout, dictation, Claude Code, and workspace settings. Use when the user wants to change settings, configure shortcuts, or customize appearance.
---

# Configuration & Workspace Settings

Manage files under `~/.scriptkit` that control launcher behavior, hotkeys, dictation, theming, and Tab AI.

## Files That Matter

| File | Purpose | Reload |
|------|---------|--------|
| `~/.scriptkit/kit/authoring/skills/` | Agent-readable skills | Read from authoring plugin |
| `~/.scriptkit/kit/config.ts` | Full config: hotkeys, built-ins, runtime preferences, Claude Code | Auto-reloads |
| `~/.scriptkit/kit/theme.json` | Theme colors | Auto-reloads |

## What Goes Where

| Setting | File | Key |
|---------|------|-----|
| Launcher hotkey | `config.ts` | `hotkey` |
| AI/Logs/Dictation shortcuts | `config.ts` | `aiHotkey`, `logsHotkey`, `dictationHotkey` |
| Dictation microphone | `config.ts` | `dictation.selectedDeviceId` |
| Theme preset | `config.ts` | `theme.presetId` |
| ACP defaults | `config.ts` | `ai.selectedAcpAgentId`, `ai.selectedModelId` |
| Snap mode | `config.ts` | `windowManagement.snapMode` |
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

## Runtime Preference Groups

```typescript
theme: { presetId: "nord" },
dictation: { selectedDeviceId: "usb-mic" },
ai: {
  selectedAcpAgentId: "codex-acp",
  selectedModelId: "gpt-5.4",
},
windowManagement: { snapMode: "expanded" },
```

## Common Mistakes

- Putting skills at the workspace root instead of in `~/.scriptkit/kit/authoring/skills/`
- Editing `~/.scriptkit/config.ts` instead of `~/.scriptkit/kit/config.ts`
- Using `command`/`control` instead of `meta`/`ctrl`
- Editing `theme.json` when you meant to change `theme.presetId` in `config.ts`

## References

See `references/config-examples.md` for full config.ts example, theme file format, frecency settings, watcher config, layout defaults, and Claude Code provider options.
