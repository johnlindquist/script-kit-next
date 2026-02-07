/// Embedded CLAUDE.md for Claude-specific guidance
const EMBEDDED_CLAUDE_MD: &str = r###"# Script Kit - Claude Instructions

This file provides Claude-specific guidance for working with Script Kit GPUI.

## ⚠️ Critical: This is Script Kit GPUI (v2), NOT the original Script Kit

Script Kit GPUI is a **complete rewrite** of the original Script Kit:
- **Old Script Kit (v1)**: Electron + Node.js
- **Script Kit GPUI (v2)**: GPUI (Rust) + Bun

If your training data includes the old Script Kit, **ignore those patterns**. Use only what's documented here.

---

## Directory Structure

```
~/.scriptkit/
├── kit/                          # Version-controllable kit directory
│   ├── main/                     # Main kit (default)
│   │   ├── scripts/             # Your TypeScript scripts
│   │   ├── extensions/          # Markdown files with embedded commands
│   │   └── agents/              # AI agent definitions
│   ├── config.ts                # Configuration (hotkey, font sizes, etc.)
│   ├── theme.json               # Theme customization (colors, etc.)
│   ├── package.json             # Enables top-level await ("type": "module")
│   ├── tsconfig.json            # TypeScript configuration
│   ├── AGENTS.md                # SDK documentation for AI agents
│   └── CLAUDE.md                # This file
├── sdk/                          # SDK (managed by app, do not edit)
│   └── kit-sdk.ts
├── db/                           # SQLite databases
├── logs/                         # Application logs
└── GUIDE.md                      # User guide
```

---

## Writing Scripts

### Minimal Script Template

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "My Script",
  description: "What this script does",
};

// Your code here - top-level await is supported
const result = await arg("Choose an option", ["A", "B", "C"]);
console.log(result);
```

### Key Points

1. **Always import the SDK first**: `import "@scriptkit/sdk";`
2. **Use `export const metadata`**: NOT comment-based metadata (deprecated)
3. **Top-level await**: Works out of the box (thanks to `package.json` `"type": "module"`)
4. **Bun APIs**: Use `Bun.file()`, `Bun.write()`, `$\`command\`` - NOT Node.js fs/child_process

### Common SDK Functions

```typescript
// User input
const text = await arg("Enter something");
const choice = await arg("Pick one", ["Option 1", "Option 2"]);

// Display content
await div("<h1 class='text-2xl'>Hello</h1>");  // HTML with Tailwind

// Editor
const code = await editor("// Edit this", "typescript");

// Forms
const [name, email] = await fields([
  { name: "name", label: "Name" },
  { name: "email", label: "Email", type: "email" },
]);

// Clipboard
const text = await paste();
await copy("Copied this text");

// Open URLs/files
await open("https://example.com");
```

---

## Configuration (config.ts)

Located at `~/.scriptkit/kit/config.ts`:

```typescript
import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: { key: "Space", modifiers: ["command"] },
  editorFontSize: 14,
  terminalFontSize: 14,
  builtIns: { clipboardHistory: true, appLauncher: true },
} satisfies Config;
```

---

## Theme (theme.json)

Located at `~/.scriptkit/kit/theme.json`:

```json
{
  "colors": {
    "background": { "main": "#1e1e2e", "panel": "#181825" },
    "text": { "primary": "#cdd6f4", "secondary": "#a6adc8" },
    "accent": { "primary": "#89b4fa", "secondary": "#74c7ec" },
    "ui": { "border": "#313244", "divider": "#45475a" }
  }
}
```

---

## Extensions (formerly Scriptlets)

Markdown files in `~/.scriptkit/kit/main/extensions/*.md` with embedded code:

```markdown
---
name: My Tools
description: Collection of useful tools
---

# My Tools

## Say Hello
\`\`\`tool:hello
import "@scriptkit/sdk";
const name = await arg("Name?");
await div(`<h1>Hello, ${name}!</h1>`);
\`\`\`

## Quick Template
\`\`\`template:greeting
Hello {{name}}, welcome to {{place}}!
\`\`\`
```

---

## DO NOT

- Use `require()` - use ES imports
- Use Node.js `fs` - use `Bun.file()` and `Bun.write()`
- Use Node.js `child_process` - use `$\`command\`` (Bun shell)
- Use comment-based metadata (`// Name:`) - use `export const metadata`
- Modify files in `~/.scriptkit/sdk/` - they're managed by the app
- Reference old Script Kit v1 patterns (Electron, Kit SDK, @johnlindquist/kit)

---

## File Locations Summary

| Purpose | Path |
|---------|------|
| Scripts | `~/.scriptkit/kit/main/scripts/*.ts` |
| Extensions | `~/.scriptkit/kit/main/extensions/*.md` |
| Agents | `~/.scriptkit/kit/main/agents/*.md` |
| Config | `~/.scriptkit/kit/config.ts` |
| Theme | `~/.scriptkit/kit/theme.json` |
| SDK Docs | `~/.scriptkit/kit/AGENTS.md` |
"###;
/// Environment variable to override the default ~/.scriptkit path
pub const SK_PATH_ENV: &str = "SK_PATH";
/// Result of setup process
#[derive(Debug)]
pub struct SetupResult {
    /// Whether ~/.scriptkit didn't exist before this run
    pub is_fresh_install: bool,
    /// Path to ~/.scriptkit (or SK_PATH override, or fallback if home dir couldn't be resolved)
    pub kit_path: PathBuf,
    /// Whether bun looks discoverable on this machine
    pub bun_available: bool,
    /// Any warnings encountered during setup
    pub warnings: Vec<String>,
}
/// Get the kit path, respecting SK_PATH environment variable
///
/// Priority:
/// 1. SK_PATH environment variable (if set)
/// 2. ~/.scriptkit (default)
/// 3. Temp directory fallback (if home dir unavailable)
pub fn get_kit_path() -> PathBuf {
    // Check for SK_PATH override first
    if let Ok(sk_path) = std::env::var(SK_PATH_ENV) {
        if let Ok(expanded) = shellexpand::full(&sk_path) {
            return PathBuf::from(expanded.as_ref());
        }
        return PathBuf::from(shellexpand::tilde(&sk_path).as_ref());
    }

    // Default: ~/.scriptkit
    match dirs::home_dir() {
        Some(home) => home.join(".scriptkit"),
        None => std::env::temp_dir().join("script-kit"),
    }
}
/// Migrate from legacy ~/.kenv to new ~/.scriptkit structure
///
/// This function handles one-time migration from the old directory structure:
/// - Moves ~/.kenv contents to ~/.scriptkit
/// - Moves ~/.kenv/scripts to ~/.scriptkit/kit/main/scripts
/// - Moves ~/.kenv/scriptlets to ~/.scriptkit/kit/main/extensions
/// - Creates a symlink ~/.kenv -> ~/.scriptkit for backwards compatibility
///
/// Returns true if migration was performed, false if not needed
#[instrument(level = "info", name = "migrate_from_kenv")]
pub fn migrate_from_kenv() -> bool {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return false,
    };

    let old_kenv = home.join(".kenv");
    let new_scriptkit = home.join(".scriptkit");

    // Only migrate if old path exists and new path doesn't
    if !old_kenv.exists() || new_scriptkit.exists() {
        return false;
    }

    info!(
        old_path = %old_kenv.display(),
        new_path = %new_scriptkit.display(),
        "Migrating from ~/.kenv to ~/.scriptkit"
    );

    // Create the new structure (under kit/ subdirectory)
    let main_scripts = new_scriptkit.join("kit").join("main").join("scripts");
    let main_extensions = new_scriptkit.join("kit").join("main").join("extensions");

    if let Err(e) = fs::create_dir_all(&main_scripts) {
        warn!(error = %e, "Failed to create main/scripts directory");
        return false;
    }

    if let Err(e) = fs::create_dir_all(&main_extensions) {
        warn!(error = %e, "Failed to create main/extensions directory");
        return false;
    }

    // Move scripts from ~/.kenv/scripts to ~/.scriptkit/kit/main/scripts
    let old_scripts = old_kenv.join("scripts");
    if old_scripts.exists() && old_scripts.is_dir() {
        if let Ok(entries) = fs::read_dir(&old_scripts) {
            for entry in entries.flatten() {
                let old_path = entry.path();
                let file_name = old_path.file_name().unwrap_or_default();
                let new_path = main_scripts.join(file_name);

                if let Err(e) = fs::rename(&old_path, &new_path) {
                    warn!(
                        error = %e,
                        old = %old_path.display(),
                        new = %new_path.display(),
                        "Failed to move script"
                    );
                }
            }
        }
    }

    // Move scriptlets from ~/.kenv/scriptlets to ~/.scriptkit/kit/main/extensions
    let old_scriptlets = old_kenv.join("scriptlets");
    if old_scriptlets.exists() && old_scriptlets.is_dir() {
        if let Ok(entries) = fs::read_dir(&old_scriptlets) {
            for entry in entries.flatten() {
                let old_path = entry.path();
                let file_name = old_path.file_name().unwrap_or_default();
                let new_path = main_extensions.join(file_name);

                if let Err(e) = fs::rename(&old_path, &new_path) {
                    warn!(
                        error = %e,
                        old = %old_path.display(),
                        new = %new_path.display(),
                        "Failed to move scriptlet"
                    );
                }
            }
        }
    }

    // Move config files to new root
    let config_files = ["config.ts", "theme.json", "tsconfig.json", ".gitignore"];
    for file in config_files {
        let old_path = old_kenv.join(file);
        let new_path = new_scriptkit.join(file);
        if old_path.exists() && !new_path.exists() {
            if let Err(e) = fs::rename(&old_path, &new_path) {
                warn!(error = %e, file = file, "Failed to move config file");
            }
        }
    }

    // Move data directories to new root
    let data_dirs = ["logs", "cache", "db", "sdk"];
    for dir in data_dirs {
        let old_path = old_kenv.join(dir);
        let new_path = new_scriptkit.join(dir);
        if old_path.exists() && old_path.is_dir() && !new_path.exists() {
            if let Err(e) = fs::rename(&old_path, &new_path) {
                warn!(error = %e, dir = dir, "Failed to move data directory");
            }
        }
    }

    // Move data files to new root
    let data_files = [
        "frecency.json",
        "store.json",
        "server.json",
        "agent-token",
        "notes.db",
        "ai-chats.db",
        "clipboard-history.db",
    ];
    for file in data_files {
        let old_path = old_kenv.join(file);
        let new_path = new_scriptkit.join(file);
        if old_path.exists() && !new_path.exists() {
            if let Err(e) = fs::rename(&old_path, &new_path) {
                warn!(error = %e, file = file, "Failed to move data file");
            }
        }
    }

    // Remove the old ~/.kenv directory (should be mostly empty now)
    if let Err(e) = fs::remove_dir_all(&old_kenv) {
        warn!(error = %e, "Failed to remove old ~/.kenv directory, may have remaining files");
    }

    // Create symlink for backwards compatibility (Unix only)
    #[cfg(unix)]
    {
        if let Err(e) = std::os::unix::fs::symlink(&new_scriptkit, &old_kenv) {
            warn!(error = %e, "Failed to create ~/.kenv symlink for backwards compatibility");
        } else {
            info!("Created ~/.kenv -> ~/.scriptkit symlink for backwards compatibility");
        }
    }

    info!("Migration from ~/.kenv to ~/.scriptkit complete");
    true
}
