# Skill: Configuration & Theming

Manage Script Kit settings, hotkeys, and visual theme.

## Configuration File

```
~/.scriptkit/kit/config.ts
```

### Full Config Example

```typescript
import type { Config } from "@scriptkit/sdk";

export default {
  // Global hotkey to open Script Kit
  hotkey: {
    key: "Space",
    modifiers: ["command"],
  },

  // Font sizes
  editorFontSize: 14,
  terminalFontSize: 14,

  // Built-in features
  builtIns: {
    clipboardHistory: true,
    appLauncher: true,
  },
} satisfies Config;
```

### Hotkey Format

```typescript
hotkey: {
  key: "Semicolon",          // Key name
  modifiers: ["meta"],        // "meta" = Cmd on macOS
}
```

Common key names: `Space`, `Semicolon`, `Slash`, `Period`, `Comma`, `A`-`Z`, `0`-`9`
Modifiers: `meta` (Cmd), `alt` (Option), `shift`, `control`

### Editing Config

1. Edit `~/.scriptkit/kit/config.ts`
2. Save — Script Kit reloads automatically
3. New hotkey takes effect immediately

## Theme File

```
~/.scriptkit/kit/theme.json
```

### Theme Structure

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

### Color Formats

All of these work:
- Hex: `"#FBBF24"` or `"FBBF24"`
- RGB: `"rgb(251, 191, 36)"`
- RGBA: `"rgba(251, 191, 36, 1.0)"`

### Editing Theme

1. Edit `~/.scriptkit/kit/theme.json`
2. Save — colors apply immediately, no restart needed

## TypeScript Config

```
~/.scriptkit/kit/tsconfig.json
```

Managed by the app. Maps `@scriptkit/sdk` to the SDK. Do not edit unless you know what you're doing.

## Package Config

```
~/.scriptkit/kit/package.json
```

Sets `"type": "module"` to enable top-level await in all scripts. Do not remove this.

## Harness Config

```
~/.scriptkit/harness.json
```

Controls which AI CLI harness Script Kit uses for Tab AI:

```json
{
  "schemaVersion": 1,
  "backend": "claudeCode",
  "command": "claude",
  "warmOnStartup": true
}
```

Supported backends: `claudeCode`, `codex`, `geminiCli`, `copilotCli`, `custom`

## File Watching Summary

| File | Auto-reloads |
|------|-------------|
| `kit/config.ts` | Yes — hotkeys, settings |
| `kit/theme.json` | Yes — colors |
| `kit/main/scripts/*.ts` | Yes — script list |
| `kit/main/extensions/*.md` | Yes — extensions |
| `harness.json` | Next Tab AI invocation |

## Common Mistakes

- **Wrong config path**: Config is at `kit/config.ts`, not `config.ts` at root
- **Invalid JSON in theme**: Validate JSON before saving `theme.json`
- **Missing type module**: Don't remove `"type": "module"` from `package.json`
