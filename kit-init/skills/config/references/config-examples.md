# Configuration Examples

## Full config.ts Example

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

## Suggested Commands (Frecency)

```typescript
suggested: {
  enabled: true,       // default: true
  maxItems: 10,        // default: 10
  minScore: 0.1,       // default: 0.1
  halfLifeDays: 7,     // default: 7.0
  trackUsage: true,    // default: true
  excludedCommands: ["builtin-quit-script-kit"]
}
```

## File Watcher

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
  path: "claude",                    // CLI binary (default: "claude" from PATH)
  permissionMode: "plan",            // "default" | "plan" | "acceptEdits"
  allowedTools: "Read,Edit,Bash(git:*)",
  addDirs: ["/Users/you/projects"],
}
```

## Theme File (`~/.scriptkit/kit/theme.json`)

```json
{
  "colors": {
    "background": { "main": "#1e1e2e", "panel": "#181825" },
    "text": { "primary": "#cdd6f4", "secondary": "#a6adc8" },
    "accent": { "primary": "#89b4fa", "secondary": "#74c7ec", "selected": "#fbbf24" },
    "ui": { "border": "#313244", "divider": "#45475a" }
  }
}
```

## Dictation

Two files involved:
- Shortcut: `~/.scriptkit/kit/config.ts` — `dictationHotkey` + `dictationHotkeyEnabled`
- Microphone: `~/.scriptkit/kit/settings.json` — `dictation.selectedDeviceId`

Use the built-in **Select Microphone** action to persist a device.
