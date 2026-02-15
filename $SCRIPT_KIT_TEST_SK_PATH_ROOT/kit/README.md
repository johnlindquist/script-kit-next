# Script Kit

Welcome to Script Kit! This directory contains your scripts, configuration, and data.

## Directory Structure

```
~/.scriptkit/
├── kit/                    # All kits (version control friendly)
│   ├── main/               # Your default kit
│   │   ├── scripts/        # TypeScript/JavaScript scripts (.ts, .js)
│   │   ├── extensions/     # Markdown extension files (.md)
│   │   └── agents/         # AI agent definitions (.md)
│   ├── package.json        # Node.js module config (enables top-level await)
│   └── tsconfig.json       # TypeScript path mappings
├── sdk/                    # Runtime SDK (managed by app)
├── db/                     # Databases (clipboard history, etc.)
├── logs/                   # Application logs
├── cache/                  # Cached data (app icons, etc.)
├── config.ts               # Your configuration
├── theme.json              # Theme customization
└── README.md               # This file
```

## File Watching

Script Kit watches these files and reloads automatically:

| File/Directory | What happens on change |
|----------------|------------------------|
| `config.ts` | Reloads configuration (hotkeys, settings) |
| `theme.json` | Applies new theme colors immediately |
| `main/scripts/*.ts` | Updates script list and metadata |
| `main/extensions/*.md` | Updates extension list |

## Scripts

Scripts are TypeScript files in `main/scripts/`. They have full access to the Script Kit SDK.

### Example Script

```typescript
// main/scripts/my-script.ts

export const metadata = {
  name: "My Script",
  description: "Does something useful",
  shortcut: "cmd shift m",  // Optional global hotkey
};

// Prompt for input
const choice = await arg("Pick an option", ["Option 1", "Option 2"]);

// Show result
await div(`<div class="p-4">You chose: ${choice}</div>`);
```

### Script Metadata

Use the `metadata` export for type-safe configuration:

```typescript
export const metadata = {
  name: "Script Name",           // Display name in menu
  description: "What it does",   // Shown below the name
  shortcut: "cmd shift x",       // Global hotkey (optional)
  alias: "sn",                   // Quick search alias (optional)
};
```

## Scriptlets

Scriptlets are Markdown files containing quick shell commands. Each code block becomes a menu item.

### Example Scriptlet

```markdown
# My Scriptlets

## Open Project
<!-- shortcut: cmd shift p -->

\`\`\`bash
cd ~/projects/myapp && code .
\`\`\`

## Git Status
<!-- name: Check Git Status -->

\`\`\`bash
git status
\`\`\`
```

### Scriptlet Metadata

Add HTML comments before code blocks:

```markdown
<!-- 
name: Display Name
description: What this does
shortcut: cmd shift x
-->
```

## Configuration (config.ts)

Your `config.ts` controls Script Kit behavior:

```typescript
export default {
  // Global hotkey to open Script Kit
  hotkey: {
    key: "Semicolon",
    modifiers: ["meta"],  // cmd+;
  },
  
  // UI Settings
  editorFontSize: 16,
  terminalFontSize: 14,
  
  // Built-in features
  builtIns: {
    clipboardHistory: true,
    appLauncher: true,
  },
} satisfies Config;
```

## Theme (theme.json)

Customize colors in `theme.json`:

```json
{
  "colors": {
    "background": {
      "main": "#1E1E1E"
    },
    "text": {
      "primary": "#FFFFFF",
      "secondary": "#CCCCCC"
    },
    "accent": {
      "selected": "#FBBF24"
    }
  }
}
```

Colors can be specified as:
- Hex strings: `"#FBBF24"` or `"FBBF24"`
- RGB: `"rgb(251, 191, 36)"`
- RGBA: `"rgba(251, 191, 36, 1.0)"`

## Environment Variable

Set `SK_PATH` to use a different directory:

```bash
export SK_PATH=~/my-scripts
```

## Quick Tips

1. **Create a new script**: Add a `.ts` file to `main/scripts/`
2. **Add a hotkey**: Set `shortcut` in the metadata
3. **Test changes**: Scripts reload automatically on save
4. **View logs**: Check `logs/script-kit-gpui.jsonl` for debugging
5. **Complete guide**: See `GUIDE.md` for comprehensive tutorials and documentation

## Links

- Documentation: https://scriptkit.com/docs
- GitHub: https://github.com/johnlindquist/kit

---

Happy scripting! 🚀
