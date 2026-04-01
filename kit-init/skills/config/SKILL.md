---
name: config
description: Manage Script Kit configuration files — hotkeys, theme, layout, dictation, Claude Code, and workspace settings. Use when the user wants to change settings, configure shortcuts, or customize appearance.
---

# Configuration & Workspace Settings

Manage files under `~/.scriptkit` that control launcher behavior, hotkeys, dictation, theming, and Tab AI.

## Files That Matter

| File | Purpose | Reload |
|------|---------|--------|
| `~/.scriptkit/skills/` | Agent-readable skills | Read at workspace root |
| `~/.scriptkit/kit/config.ts` | Static config: hotkeys, built-ins, Claude Code | Auto-reloads |
| `~/.scriptkit/kit/settings.json` | Runtime preferences: layout, theme, microphone | Read by runtime |
| `~/.scriptkit/kit/theme.json` | Theme colors | Auto-reloads |

## What Goes Where

| Setting | File | Key |
|---------|------|-----|
| Launcher hotkey | `config.ts` | `hotkey` |
| AI/Logs/Dictation shortcuts | `config.ts` | `aiHotkey`, `logsHotkey`, `dictationHotkey` |
| Dictation microphone | `settings.json` | `dictation.selectedDeviceId` |
| Theme preset | `settings.json` | `theme.presetId` |
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

## Common Mistakes

- Putting `skills/` under `kit/` instead of at `~/.scriptkit/skills/`
- Editing `~/.scriptkit/config.ts` instead of `~/.scriptkit/kit/config.ts`
- Using `command`/`control` instead of `meta`/`ctrl`
- Putting microphone selection in `config.ts` instead of `settings.json`

## References

See `references/config-examples.md` for full config.ts example, theme file format, frecency settings, watcher config, layout defaults, and Claude Code provider options.
