# Skill: Configuration & Workspace Settings

Manage the files under `~/.scriptkit` that control launcher behavior, hotkeys, dictation, theming, and Tab AI.

## Files That Matter

| File | Purpose | Reload behavior |
| --- | --- | --- |
| `~/.scriptkit/skills/` | Agent-readable skills. Claude Code opens the workspace at the root and expects `./skills` here. | Read from workspace root |
| `~/.scriptkit/kit/config.ts` | Static app config: launcher hotkey, built-ins, command overrides, AI/logs/dictation hotkeys, Claude Code. | Auto-reloads |
| `~/.scriptkit/kit/settings.json` | Runtime-persisted preferences: layout, theme preset, dictation microphone selection. | Read by runtime |
| `~/.scriptkit/kit/theme.json` | Theme colors. | Auto-reloads |
| `~/.scriptkit/harness.json` | Tab AI harness backend and startup behavior. | Read on next Tab AI invocation |

## What Goes Where

| Setting | File | Key |
| --- | --- | --- |
| Launcher hotkey | `~/.scriptkit/kit/config.ts` | `hotkey` |
| Notes / AI / Logs / Dictation shortcuts | `~/.scriptkit/kit/config.ts` | `notesHotkey`, `aiHotkey`, `logsHotkey`, `dictationHotkey` |
| Dictation microphone | `~/.scriptkit/kit/settings.json` | `dictation.selectedDeviceId` |
| Theme preset selection | `~/.scriptkit/kit/settings.json` | `theme.presetId` |
| Theme colors | `~/.scriptkit/kit/theme.json` | `colors.*` |
| Layout defaults | `~/.scriptkit/kit/config.ts` | `layout` |
| Runtime-persisted layout | `~/.scriptkit/kit/settings.json` | `layout` |
| Claude Code provider | `~/.scriptkit/kit/config.ts` | `claudeCode` |

## UI, Editor, Built-ins, and Limits

These all live in `~/.scriptkit/kit/config.ts`:

```typescript
editor: "code",
padding: {
  top: 8,
  left: 12,
  right: 12,
},
editorFontSize: 16,
terminalFontSize: 14,
uiScale: 1.0,
builtIns: {
  clipboardHistory: true,
  appLauncher: true,
  windowSwitcher: true,
},
clipboardHistoryMaxTextLength: 100000,
processLimits: {
  maxMemoryMb: 512,
  maxRuntimeSeconds: 300,
  healthCheckIntervalMs: 5000,
},
```

## Main Config File

```typescript
import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: {
    modifiers: ["meta"],
    key: "Semicolon",
  },

  aiHotkey: {
    modifiers: ["meta", "shift"],
    key: "Space",
  },
  aiHotkeyEnabled: true,

  logsHotkey: {
    modifiers: ["meta", "shift"],
    key: "KeyL",
  },
  logsHotkeyEnabled: true,

  dictationHotkey: {
    modifiers: ["meta", "shift"],
    key: "KeyD",
  },
  dictationHotkeyEnabled: true,

  commands: {
    "builtin/clipboard-history": {
      shortcut: { modifiers: ["meta", "shift"], key: "KeyV" },
    },
    "builtin/empty-trash": {
      confirmationRequired: true,
    },
  },

  claudeCode: {
    enabled: true,
    permissionMode: "plan",
  },
} satisfies Config;
```

## Hotkey Format

Use `KeyboardEvent.code` values for keys. Valid modifiers: `meta`, `ctrl`, `alt`, `shift`

Common keys: `Semicolon`, `Space`, `Enter`, `KeyK`, `KeyL`, `KeyD`, `Digit1`

**Do not** use `command` or `control` in config.ts — use `meta` and `ctrl`.

## Auxiliary Hotkeys

These all live in `~/.scriptkit/kit/config.ts`:

- `notesHotkey` — no default; set it explicitly if you want one
- `aiHotkey` — defaults to Cmd+Shift+Space when enabled and unset
- `aiHotkeyEnabled` — defaults to `true`
- `logsHotkey` — defaults to Cmd+Shift+L when enabled and unset
- `logsHotkeyEnabled` — defaults to `true`
- `dictationHotkey` — no default; set it explicitly if you want one
- `dictationHotkeyEnabled` — defaults to `true`

## Dictation

Two different files are involved:
- Shortcut registration lives in `~/.scriptkit/kit/config.ts`
- Microphone selection lives in `~/.scriptkit/kit/settings.json`

```typescript
dictationHotkey: {
  modifiers: ["meta", "shift"],
  key: "KeyD",
},
dictationHotkeyEnabled: true,
```

```json
{
  "dictation": {
    "selectedDeviceId": "usb-mic"
  }
}
```

Behavior:
- `dictationHotkeyEnabled: true` does not create a shortcut by itself; set `dictationHotkey` too
- No `selectedDeviceId` means use the macOS default microphone
- Use the built-in **Select Microphone** action to persist a device
- If the saved microphone is missing, Script Kit falls back to the best available device and clears the stale preference

## Command Overrides

Configure per-command behavior in `config.ts`:

```typescript
commands: {
  "script/my-workflow": {
    shortcut: { modifiers: ["meta", "shift"], key: "KeyW" }
  },
  "builtin/app-launcher": {
    hidden: true
  },
  "builtin/empty-trash": {
    confirmationRequired: true
  }
}
```

Command deeplinks use: `scriptkit://commands/{id}`

## Suggested Commands (Frecency)

Controls the "Suggested" section in the main menu:

```typescript
suggested: {
  enabled: true,       // default: true
  maxItems: 10,        // default: 10
  minScore: 0.1,       // default: 0.1
  halfLifeDays: 7,     // default: 7.0
  trackUsage: true,    // default: true
  excludedCommands: ["builtin-quit-script-kit"] // default: ["builtin-quit-script-kit"]
}
```

## File Watcher

Debounce and back-off settings for the file watcher:

```typescript
watcher: {
  debounceMs: 500,        // default: 500
  stormThreshold: 200,    // default: 200
  initialBackoffMs: 100,  // default: 100
  maxBackoffMs: 30000,    // default: 30000
  maxNotifyErrors: 10,    // default: 10
}
```

## Window Layout

Sizing defaults for the launcher window:

```typescript
layout: {
  standardHeight: 500,  // default: 500
  maxHeight: 700,       // default: 700
}
```

## Claude Code Provider

```typescript
claudeCode: {
  enabled: true,
  permissionMode: "plan",
  allowedTools: "Read,Edit,Bash(git:*)",
  addDirs: ["/Users/you/projects"]
}
```

## Theme File

`~/.scriptkit/kit/theme.json`

```json
{
  "colors": {
    "background": {
      "main": "#1e1e2e",
      "panel": "#181825"
    },
    "text": {
      "primary": "#cdd6f4",
      "secondary": "#a6adc8"
    },
    "accent": {
      "primary": "#89b4fa",
      "secondary": "#74c7ec",
      "selected": "#fbbf24"
    },
    "ui": {
      "border": "#313244",
      "divider": "#45475a"
    }
  }
}
```

## Harness Config

`~/.scriptkit/harness.json`

```json
{
  "schemaVersion": 1,
  "backend": "claudeCode",
  "command": "claude",
  "warmOnStartup": true
}
```

Supported backends: `claudeCode`, `codex`, `geminiCli`, `copilotCli`, `custom`.

## Common Mistakes

- Putting `skills/` under `kit/` instead of at `~/.scriptkit/skills/`
- Editing `~/.scriptkit/config.ts` instead of `~/.scriptkit/kit/config.ts`
- Using `command` / `control` instead of `meta` / `ctrl`
- Putting dictation microphone selection in `config.ts` instead of `kit/settings.json`
- Using `kit://commands/...` instead of `scriptkit://commands/...`
