# Script Kit User Guide

Welcome to Script Kit! This guide will help you get started and master the powerful automation capabilities of Script Kit.

---

## Table of Contents

1. [Welcome & Quick Start](#welcome--quick-start)
2. [Directory Structure](#directory-structure)
3. [Writing Scripts](#writing-scripts)
4. [SDK Functions (Core)](#sdk-functions-core)
5. [Extensions](#extensions)
6. [Configuration](#configuration-configts)
7. [Themes](#themes-themejson)
8. [Built-in Features](#built-in-features)
9. [AI Window (BYOK)](#ai-window-byok)
10. [Notes Window](#notes-window)
11. [File Watching](#file-watching)
12. [Multiple Environments](#multiple-environments)
13. [SDK Quick Reference](#sdk-quick-reference)

---

## Welcome & Quick Start

### What is Script Kit?

Script Kit is a powerful automation tool that lets you create scripts to automate your daily workflows. Built with the GPUI framework from Zed, it offers:

- **Blazing fast performance** - Native Rust with GPU-accelerated rendering
- **Beautiful UI prompts** - Text inputs, editors, forms, file pickers, and more
- **Global hotkey access** - Trigger scripts from anywhere
- **Bun runtime** - Fast JavaScript/TypeScript execution
- **Tailwind CSS** - Style your prompts with familiar utility classes

### Your First Script in 60 Seconds

1. **Create the scripts directory** (if it doesn't exist):
   ```bash
   mkdir -p ~/.scriptkit/kit/main/scripts
   ```

2. **Create your first script** at `~/.scriptkit/kit/main/scripts/hello.ts`:
   ```typescript
   import "@scriptkit/sdk";

   export const metadata = {
     name: "Hello World",
     description: "My first Script Kit script",
   };

   const name = await arg("What's your name?");
   await div(`<h1 class="text-4xl p-8 text-center">Hello, ${name}! 👋</h1>`);
   ```

3. **Open Script Kit** by pressing the global hotkey (default: `Cmd+;`)

4. **Type "hello"** and press Enter

5. **Enter your name** and see the greeting!

### The Main Hotkey

The global hotkey opens the Script Kit launcher from anywhere:

| Platform | Default Hotkey |
|----------|----------------|
| macOS    | `Cmd+;`        |
| Windows  | `Ctrl+;`       |
| Linux    | `Ctrl+;`       |

You can customize this in `~/.scriptkit/kit/config.ts` (see [Configuration](#configuration-configts)).

---

## Directory Structure

Script Kit stores all its data in `~/.scriptkit/`. Here's the layout:

```
~/.scriptkit/
├── CLAUDE.md                    # Canonical harness instructions
├── AGENTS.md                    # SDK reference for agentic authoring
├── GUIDE.md                     # User guide
├── skills/
│   ├── script-authoring/SKILL.md
│   ├── scriptlets/SKILL.md
│   ├── agents/SKILL.md
│   ├── config/SKILL.md
│   └── troubleshooting/SKILL.md
├── examples/
│   ├── scripts/
│   ├── extensions/
│   └── agents/
├── kit/
│   ├── main/
│   │   ├── scripts/
│   │   ├── extensions/
│   │   └── agents/
│   ├── config.ts
│   ├── theme.json
│   ├── package.json
│   └── tsconfig.json
├── sdk/
│   └── kit-sdk.ts
├── db/
├── logs/
└── cache/
```

### Key Directories

| Directory | Purpose |
|-----------|---------|
| `kit/main/scripts/` | Your primary scripts - create `.ts` files here |
| `kit/main/extensions/` | Markdown extension files with shell commands |
| `kit/main/agents/` | AI agent definitions |
| `sdk/` | Runtime SDK (auto-extracted, don't edit) |
| `db/` | SQLite databases for Notes and AI |
| `logs/` | Debug logs in JSONL format |

---

## Writing Scripts

### Creating a Script File

Create a `.ts` file in `~/.scriptkit/kit/main/scripts/`:

```typescript
// ~/.scriptkit/kit/main/scripts/my-script.ts
import "@scriptkit/sdk";

export const metadata = {
  name: "My Script",
  description: "What this script does",
  shortcut: "cmd shift m",
};

const result = await arg("Pick an option", ["Option A", "Option B", "Option C"]);
await div(`<div class="p-8 text-xl">${result}</div>`);
```

### Script Metadata

Use `export const metadata = { ... }` for new scripts:

```typescript
export const metadata = {
  name: "My Script",
  description: "Shown in the launcher",
  shortcut: "cmd shift m",
  alias: "ms",
};
```

### Harness-Safe Authoring Rules

- Start new scripts with `import "@scriptkit/sdk";`.
- Use `export const metadata = { name, description }`.
- Write new scripts as `.ts` files in `kit/main/scripts/`.
- Treat legacy comment headers as compatibility-only for older files; do not generate them for new harness-authored scripts.

---

## SDK Functions (Core)

### arg() - Text Input with Choices

The most versatile prompt - get text input with optional choice list:

```typescript
// Simple text input
const name = await arg("What's your name?");

// Text input with choices
const fruit = await arg("Pick a fruit", ["Apple", "Banana", "Cherry"]);

// Rich choices with metadata
const app = await arg("Launch app", [
  { name: "VS Code", value: "code", description: "Code editor" },
  { name: "Terminal", value: "term", description: "Command line" },
  { name: "Browser", value: "chrome", description: "Web browser" },
]);

// Dynamic choices (async function)
const repo = await arg("Select repo", async () => {
  const response = await fetch("https://api.github.com/users/me/repos");
  const repos = await response.json();
  return repos.map(r => ({ name: r.name, value: r.clone_url }));
});

// Filter function (called on each keystroke)
const file = await arg("Search files", (input) => {
  return files.filter(f => f.includes(input));
});
```

**Configuration Object:**

```typescript
const result = await arg({
  placeholder: "Type something...",
  hint: "Press Enter to submit",
  choices: ["Option 1", "Option 2"],
  onInit: () => console.log("Prompt opened"),
  onSubmit: (value) => console.log("Submitted:", value),
  actions: [
    {
      name: "Copy",
      shortcut: "cmd+c",
      onAction: async (input) => {
        await copy(input);
        hud("Copied!");
      }
    }
  ]
});
```

### div() - HTML Display with Tailwind

Display rich HTML content styled with Tailwind CSS:

```typescript
// Simple HTML
await div("<h1 class='text-4xl p-8'>Hello World!</h1>");

// Complex layout
await div(`
  <div class="flex flex-col gap-4 p-8">
    <h1 class="text-3xl font-bold text-blue-500">Dashboard</h1>
    <div class="grid grid-cols-2 gap-4">
      <div class="bg-gray-800 p-4 rounded-lg">
        <p class="text-gray-400">Total Scripts</p>
        <p class="text-2xl font-bold">42</p>
      </div>
      <div class="bg-gray-800 p-4 rounded-lg">
        <p class="text-gray-400">Runs Today</p>
        <p class="text-2xl font-bold">128</p>
      </div>
    </div>
  </div>
`);

// With configuration
await div({
  html: "<p>Custom styled content</p>",
  placeholder: "My Title",
  containerClasses: "bg-gradient-to-r from-purple-500 to-pink-500",
  containerBg: "transparent",
  containerPadding: "none",
  opacity: 95,
});
```

### md() - Markdown to HTML

Convert Markdown to HTML for use with `div()`:

```typescript
const html = md(`
# Hello World

This is **bold** and this is *italic*.

- List item 1
- List item 2
- List item 3

\`\`\`javascript
const greeting = "Hello!";
console.log(greeting);
\`\`\`
`);

await div(html);
```

Supported Markdown features:
- Headings (h1-h6)
- Bold, italic, strikethrough
- Ordered and unordered lists
- Code blocks and inline code
- Links and images
- Blockquotes
- Horizontal rules

### editor() - Code Editor

Open a Monaco-style code editor:

```typescript
// Basic editor
const code = await editor();

// With initial content and language
const typescript = await editor(`
function greet(name: string) {
  return \`Hello, \${name}!\`;
}
`, "typescript");

// Supported languages
// "typescript", "javascript", "json", "html", "css", "markdown", "python", "rust", etc.
```

### fields() - Multi-Field Forms

Create forms with multiple input fields:

```typescript
// Simple fields (strings become both name and label)
const [firstName, lastName] = await fields(["First Name", "Last Name"]);

// Rich field definitions
const [name, email, age] = await fields([
  { name: "name", label: "Full Name", placeholder: "John Doe" },
  { name: "email", label: "Email", type: "email", placeholder: "john@example.com" },
  { name: "age", label: "Age", type: "number" },
]);

// All supported field types
const values = await fields([
  { name: "text", label: "Text", type: "text" },
  { name: "password", label: "Password", type: "password" },
  { name: "email", label: "Email", type: "email" },
  { name: "number", label: "Number", type: "number" },
  { name: "date", label: "Date", type: "date" },
  { name: "time", label: "Time", type: "time" },
  { name: "url", label: "URL", type: "url" },
  { name: "tel", label: "Phone", type: "tel" },
  { name: "color", label: "Color", type: "color" },
]);
```

### path() - File/Folder Picker

Browse and select files or folders:

```typescript
// Basic file picker
const filePath = await path();

// Start in a specific directory
const document = await path({
  startPath: "~/Documents",
  hint: "Select a document to open",
});

// Common patterns
const image = await path({ startPath: "~/Pictures" });
const project = await path({ startPath: "~/Projects" });
```

### hotkey() - Capture Keyboard Shortcuts

Capture a keyboard shortcut from the user:

```typescript
const shortcut = await hotkey("Press a keyboard shortcut");

console.log(shortcut);
// {
//   key: "k",
//   command: true,
//   shift: true,
//   option: false,
//   control: false,
//   shortcut: "cmd+shift+k",
//   keyCode: "KeyK"
// }
```

### drop() - Drag and Drop

Create a drop zone for files:

```typescript
const files = await drop();

for (const file of files) {
  console.log(`File: ${file.name}`);
  console.log(`Path: ${file.path}`);
  console.log(`Size: ${file.size} bytes`);
}
```

### term() - Terminal Emulator

Open an interactive terminal:

```typescript
// Open empty terminal
await term();

// Run a command
await term("htop");

// Run with environment setup
await term("cd ~/Projects && npm start");
```

### Additional Prompts

```typescript
// Compact prompt variants
const result1 = await mini("Pick one", ["A", "B", "C"]);
const result2 = await micro("Pick one", ["A", "B", "C"]);

// Multi-select
const selected = await select("Pick multiple", ["A", "B", "C", "D"]);
// Returns: ["A", "C"] (array of selected values)

// Custom HTML form
const formData = await form(`
  <form>
    <input type="text" name="username" placeholder="Username">
    <input type="password" name="password" placeholder="Password">
    <button type="submit">Login</button>
  </form>
`);
// Returns: { username: "john", password: "secret" }

// Template with tabstops (VSCode snippet syntax)
const filled = await template(`
Hello \${1:name},

Thank you for \${2:reason}.

Best regards,
\${3:Your Name}
`);
```

---

## Extensions (Scriptlets)

Extensions (also called "scriptlets") are markdown files containing one or more mini-scripts. They're perfect for quick automations that don't need a full TypeScript file, and support multiple command types including bash, TypeScript, AppleScript, and more.

### Creating an Extension File

Create a `.md` file in `~/.scriptkit/kit/main/extensions/`:

~~~md
---
name: My Tools
description: A collection of useful tools
icon: sparkles
---

## Say Hello

```metadata
keyword: !hello
description: Greet the user
```

```paste
Hello, World!
```

## Quick Note

```metadata
description: Save a quick note
```

```tool:quick-note
import "@scriptkit/sdk";

const note = await arg("Note");
await Bun.write(`${env.HOME}/quick-note.txt`, note);
await notify("Saved");
```
~~~

### Scriptlet Metadata

Prefer `metadata` code fences for new bundles:

~~~md
## Open Dashboard

```metadata
description: Open the admin dashboard
shortcut: cmd shift d
```

```open
https://admin.example.com/dashboard
```
~~~

Legacy HTML comment metadata still works for backwards compatibility, but do not use it for new harness-authored bundles.

Available scriptlet metadata:
- `description` - Shown below the scriptlet name
- `shortcut` - Global keyboard shortcut (e.g., `cmd+shift+k`)

### Code Block Types

Script Kit supports several code block types:

#### `bash` - Shell Commands
```markdown
\`\`\`bash
echo "Hello from bash"
ls -la ~/Projects
\`\`\`
```

#### `open` - Open URLs/Files/Apps
```markdown
\`\`\`open
https://github.com
\`\`\`

\`\`\`open
file:///Applications/Safari.app
\`\`\`
```

#### `ts` or `typescript` - TypeScript Code
```markdown
\`\`\`ts
const name = await arg("What's your name?");
await div(\`<h1>Hello, \${name}!</h1>\`);
\`\`\`
```

#### `applescript` - AppleScript Commands
```markdown
\`\`\`applescript
tell application "Finder"
    activate
end tell
\`\`\`
```

#### `template` - Text Templates
```markdown
\`\`\`template
Hello {{name}},

Thank you for your inquiry about {{topic}}.

Best regards,
{{sender}}
\`\`\`
```

### Variable Substitution

Use `{{variableName}}` syntax for user input. Script Kit will prompt for each variable before execution:

```markdown
## Git Clone
<!-- description: Clone a repository -->

\`\`\`bash
cd ~/Projects
git clone {{url}}
cd $(basename {{url}} .git)
code .
\`\`\`
```

When this scriptlet runs, you'll be prompted for `url` before the commands execute.

### Multiple Scriptlets Per File

A single markdown file can contain many scriptlets, organized under one bundle:

```markdown
---
name: Quick Links
description: Common websites and tools
---

# Quick Links

## GitHub
<!-- description: Open GitHub -->
\`\`\`open
https://github.com
\`\`\`

## Gmail
<!-- description: Open Gmail -->
\`\`\`open
https://mail.google.com
\`\`\`

## Calendar
<!-- description: Open Google Calendar -->
\`\`\`open
https://calendar.google.com
\`\`\`
```

All three scriptlets appear in Script Kit as separate commands, grouped under "Quick Links".

---

## Shared Actions

Shared actions let you define reusable actions that automatically apply to ALL scriptlets in an extension file. This is perfect for common operations like "Copy URL", "Open in Browser", etc.

### Creating Shared Actions

Create a companion `.actions.md` file with the same base name as your extension:

```
~/.scriptkit/kit/main/extensions/
├── quicklinks.md           # Main extension file
└── quicklinks.actions.md   # Shared actions for all quicklinks
```

### Shared Actions File Format

Shared actions use H3 headers (###) instead of H2:

```markdown
# URL Actions

### Copy URL
<!-- shortcut: cmd+c -->
<!-- description: Copy the URL to clipboard -->
\`\`\`bash
echo -n "{{content}}" | pbcopy
\`\`\`

### Open in Safari
<!-- shortcut: cmd+shift+s -->
<!-- description: Open URL in Safari -->
\`\`\`bash
open -a Safari "{{content}}"
\`\`\`

### Open in Chrome
<!-- description: Open URL in Google Chrome -->
\`\`\`bash
open -a "Google Chrome" "{{content}}"
\`\`\`
```

### The `{{content}}` Variable

In shared actions, `{{content}}` is automatically replaced with the parent scriptlet's code content. For example, if your scriptlet is:

```markdown
## GitHub
\`\`\`open
https://github.com
\`\`\`
```

And your shared action is:
```markdown
### Copy URL
\`\`\`bash
echo -n "{{content}}" | pbcopy
\`\`\`
```

Then `{{content}}` becomes `https://github.com` when the action runs.

### How Shared Actions Work

1. When you open the Actions menu (Tab key) on a scriptlet
2. Script Kit loads both the scriptlet's inline actions (H3 headers within the scriptlet) AND the shared actions from the companion `.actions.md` file
3. Inline actions take precedence over shared actions with the same name
4. All actions appear together in the Actions menu

### Action Precedence

If a scriptlet has an inline action with the same command name as a shared action, the inline action wins:

```markdown
## Special Link
<!-- This inline action overrides the shared "Copy URL" action -->

\`\`\`open
https://example.com/special
\`\`\`

### Copy URL
<!-- This takes precedence over shared Copy URL -->
\`\`\`bash
echo "Custom copy for this link: {{content}}" | pbcopy
\`\`\`
```

### Built-in Shared Actions

Script Kit ships with shared actions for several built-in extensions:

| Extension | Shared Actions |
|-----------|---------------|
| Quick Links | Copy URL, Open in Safari/Chrome/Firefox |
| CleanShot | Copy URL scheme, Open Settings |
| Conductor | Copy URL, Open in Browser |

### Creating Your Own Shared Actions

1. Create your extension file: `my-tools.md`
2. Create the companion: `my-tools.actions.md`
3. Add H3 actions with optional shortcuts and descriptions
4. Use `{{content}}` to access the parent scriptlet's code

Example for a code snippets extension:

```markdown
# Snippet Actions

### Copy to Clipboard
<!-- shortcut: cmd+c -->
\`\`\`bash
echo -n "{{content}}" | pbcopy
\`\`\`

### Insert at Cursor
<!-- shortcut: cmd+shift+v -->
\`\`\`ts
await setSelectedText("{{content}}");
\`\`\`

### Open in Editor
\`\`\`ts
const tmp = \`/tmp/snippet-\${Date.now()}.txt\`;
await Bun.write(tmp, "{{content}}");
await \$\`code \${tmp}\`;
\`\`\`
```

---

## Agents (mdflow)

Agent files are reusable backend-specific markdown prompts. They are not TypeScript scripts and they do not use `export const metadata`.

Write them in `~/.scriptkit/kit/main/agents/` with a backend suffix in the filename:

- `review.claude.md`
- `plan.gemini.md`
- `code.codex.md`
- `assist.copilot.md`
- `chat.i.gemini.md` for interactive Gemini agents

Minimal example:

```markdown
---
_sk_name: "Review PR"
_sk_description: "Review staged changes and call out risks"
_sk_icon: "git-pull-request"
model: sonnet
---

Review the current git diff.

Return:
1. findings ordered by severity
2. concrete fixes
3. tests to add
```

For more patterns, read `~/.scriptkit/examples/agents/`.

---

## Configuration (config.ts)

Create `~/.scriptkit/kit/config.ts` to customize Script Kit:

```typescript
import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: {
    modifiers: ["meta"],
    key: "Semicolon",
  },

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

  suggested: {
    enabled: true,
    maxItems: 10,
    minScore: 0.1,
    halfLifeDays: 7,
    trackUsage: true,
    excludedCommands: ["builtin-quit-script-kit"],
  },

  // No default notes shortcut — set one only if you want it.
  // notesHotkey: { modifiers: ["meta", "shift"], key: "KeyN" },

  // Defaults to Cmd+Shift+Space when enabled and unset.
  // aiHotkey: { modifiers: ["meta", "shift"], key: "Space" },
  aiHotkeyEnabled: true,

  // Defaults to Cmd+Shift+L when enabled and unset.
  // logsHotkey: { modifiers: ["meta", "shift"], key: "KeyL" },
  logsHotkeyEnabled: true,

  // No default dictation shortcut — set one only if you want it.
  // dictationHotkey: { modifiers: ["meta", "shift"], key: "KeyD" },
  dictationHotkeyEnabled: true,

  watcher: {
    debounceMs: 500,
    stormThreshold: 200,
    initialBackoffMs: 100,
    maxBackoffMs: 30000,
    maxNotifyErrors: 10,
  },

  layout: {
    standardHeight: 500,
    maxHeight: 700,
  },

  bun_path: "/opt/homebrew/bin/bun",
  editor: "code",

  commands: {
    "builtin/clipboard-history": {
      shortcut: {
        modifiers: ["meta", "shift"],
        key: "KeyV",
      },
    },
    "builtin/empty-trash": {
      confirmationRequired: true,
    },
  },
} satisfies Config;
```

Dictation microphone selection is stored separately in `~/.scriptkit/kit/settings.json`:

```json
{
  "dictation": {
    "selectedDeviceId": "usb-mic"
  }
}
```

Behavior:
- `dictationHotkeyEnabled: true` does not create a shortcut by itself; set `dictationHotkey` to register one
- No `selectedDeviceId` → use the macOS default microphone
- Use the built-in **Select Microphone** action to persist a device
- Saved microphone missing → fall back to the best available device and clear the stale preference

### Hotkey Configuration

Valid modifier keys:
- `"meta"` - Cmd on macOS, Win on Windows
- `"ctrl"` - Control key
- `"alt"` - Option on macOS, Alt on Windows
- `"shift"` - Shift key

Common key codes:
- Letters: `"KeyA"` through `"KeyZ"`
- Numbers: `"Digit0"` through `"Digit9"`
- Special: `"Space"`, `"Enter"`, `"Semicolon"`, `"Comma"`, `"Period"`, `"Slash"`
- Function keys: `"F1"` through `"F12"`

### Common Mistakes

- Putting `skills/` under `kit/` instead of at `~/.scriptkit/skills/`
- Editing `~/.scriptkit/config.ts` instead of `~/.scriptkit/kit/config.ts`
- Using `command` / `control` instead of `meta` / `ctrl`
- Putting dictation microphone selection in `config.ts` instead of `kit/settings.json`

---

## Themes (theme.json)

Customize the look and feel with `~/.scriptkit/kit/theme.json`:

```json
{
  "colors": {
    "background": {
      "main": 1973790,
      "titleBar": 1973790,
      "searchBox": 2500134,
      "logPanel": 1579032
    },
    "text": {
      "primary": 15066597,
      "secondary": 10066329,
      "tertiary": 7829367,
      "muted": 6710886,
      "dimmed": 5592405
    },
    "accent": {
      "selected": 3447003,
      "selectedSubtle": 2236962,
      "buttonText": 16777215
    },
    "ui": {
      "border": 3355443,
      "success": 5025616
    }
  },
  "opacity": {
    "background": 0.95
  },
  "vibrancy": {
    "enabled": true,
    "style": "popover"
  },
  "dropShadow": {
    "enabled": true,
    "color": 0,
    "opacity": 0.5,
    "blur": 20,
    "spread": 0
  }
}
```

### Color Formats

Colors can be specified as:
- **Decimal integers**: `16777215` (white)
- **Hex strings**: `"#FFFFFF"` or `"FFFFFF"`
- **RGB objects**: `{"r": 255, "g": 255, "b": 255}`
- **RGBA objects**: `{"r": 255, "g": 255, "b": 255, "a": 1.0}`

### Vibrancy Styles (macOS)

Available vibrancy styles:
- `"popover"` - Popover-style blur (default)
- `"menu"` - Menu-style blur
- `"sidebar"` - Sidebar-style blur
- `"header"` - Header-style blur
- `"sheet"` - Sheet-style blur
- `"window"` - Window-style blur
- `"hud"` - HUD-style blur

### Focus-Aware Colors

For windows that dim when unfocused, use `focusAware`:

```json
{
  "focusAware": {
    "focused": {
      "background": { "main": 1973790 },
      "text": { "primary": 15066597 }
    },
    "unfocused": {
      "background": { "main": 1579032 },
      "text": { "primary": 10066329 }
    }
  }
}
```

---

## Built-in Features

### Clipboard History

Access your clipboard history from any script:

```typescript
// Get clipboard history entries
const entries = await clipboardHistory();

for (const entry of entries) {
  console.log(entry.entryId);
  console.log(entry.content);
  console.log(entry.contentType);  // "text" or "image"
  console.log(entry.timestamp);
  console.log(entry.pinned);
}

// Pin an entry (prevents auto-removal)
await clipboardHistoryPin(entryId);

// Unpin an entry
await clipboardHistoryUnpin(entryId);

// Remove a specific entry
await clipboardHistoryRemove(entryId);

// Clear all entries (except pinned)
await clipboardHistoryClear();

// Remove oversized text entries
await clipboardHistoryTrimOversize();
```

Enable in config:
```typescript
builtIns: {
  clipboardHistory: true
}
```

### App Launcher

Launch applications from Script Kit:

Enable in config:
```typescript
builtIns: {
  appLauncher: true
}
```

### Window Switcher

Manage system windows programmatically:

```typescript
// Get all windows
const windows = await getWindows();

for (const win of windows) {
  console.log(win.windowId);
  console.log(win.title);
  console.log(win.appName);
  console.log(win.bounds);  // { x, y, width, height }
  console.log(win.isMinimized);
  console.log(win.isActive);
}

// Window actions
await focusWindow(windowId);
await closeWindow(windowId);
await minimizeWindow(windowId);
await maximizeWindow(windowId);
await moveWindow(windowId, x, y);
await resizeWindow(windowId, width, height);

// Tile positions: "left", "right", "top", "bottom",
// "top-left", "top-right", "bottom-left", "bottom-right",
// "center", "maximize"
await tileWindow(windowId, "left");
```

---

## AI Window (BYOK)

Script Kit includes a built-in AI chat window that uses your own API keys (BYOK = Bring Your Own Key).

### Opening the AI Window

- **Hotkey**: `Cmd+Shift+Space` (default, configurable)
- **From script**: See [SDK Reference](#sdk-quick-reference)

### API Key Setup

Set one of these environment variables:

| Provider | Environment Variable |
|----------|---------------------|
| Anthropic (Claude) | `SCRIPT_KIT_ANTHROPIC_API_KEY` |
| OpenAI (GPT) | `SCRIPT_KIT_OPENAI_API_KEY` |

**Where to set keys:**

1. **Shell profile** (recommended):
   ```bash
   # ~/.zshrc or ~/.bashrc
   export SCRIPT_KIT_ANTHROPIC_API_KEY="sk-ant-..."
   ```

2. **Environment file**:
   ```bash
   # ~/.scriptkit/.env
   SCRIPT_KIT_ANTHROPIC_API_KEY=sk-ant-...
   ```

3. **macOS Keychain** (for extra security):
   ```bash
   security add-generic-password -a "$USER" -s "SCRIPT_KIT_ANTHROPIC_API_KEY" -w "sk-ant-..."
   ```

### Features

- **Streaming responses** with real-time token display
- **Markdown rendering** for formatted AI responses
- **Model picker** to select AI models
- **Chat history** with sidebar navigation
- **Multi-provider support** (Anthropic Claude, OpenAI GPT)

### Configuring the Hotkey

```typescript
// ~/.scriptkit/kit/config.ts
aiHotkey: {
  modifiers: ["meta", "shift"],
  key: "Space"
}
```

---

## Notes Window

A floating notes window with Markdown support for quick note-taking.

### Opening the Notes Window

- **Hotkey**: No default — set `notesHotkey` in `kit/config.ts` to enable
- **From script**: See [SDK Reference](#sdk-quick-reference)

### Features

- **Markdown editing** with formatting toolbar
- **Formatting shortcuts**:
  - `Cmd+B` - Bold
  - `Cmd+I` - Italic
  - `Cmd+K` - Link
  - `Cmd+Shift+C` - Code block
- **Multiple notes** with sidebar navigation
- **Full-text search** across all notes
- **Soft delete** with trash and restore
- **Export** to plain text, Markdown, or HTML (copies to clipboard)
- **Character count** in footer

### Configuring the Hotkey

```typescript
// ~/.scriptkit/kit/config.ts
notesHotkey: {
  modifiers: ["meta", "shift"],
  key: "KeyN"
}
```

### Storage

Notes are stored in SQLite at `~/.scriptkit/db/notes.sqlite`.

---

## File Watching

Script Kit automatically watches for changes and reloads:

### Watched Files and Directories

| Path | What Happens |
|------|--------------|
| `kit/main/scripts/*.ts` | Scripts reload in launcher |
| `kit/main/scripts/*.js` | Scripts reload in launcher |
| `kit/main/extensions/*.md` | Extensions reload in launcher |
| `kit/config.ts` | Most settings reload live (hotkey needs restart) |
| `kit/theme.json` | Theme reloads live (no restart) |

### Auto-Reload Behavior

- **Scripts**: Changes appear immediately in the launcher
- **Theme**: Colors update live without restart
- **Config**: Most settings reload live (font sizes, padding, built-ins)
- **Main hotkey**: Requires app restart to take effect

### File Watch Triggers

Use `watch` in script metadata to trigger on file changes:

```typescript
export const metadata = {
  name: "Watch Downloads",
  watch: ["~/Downloads/*"],
};

// This script runs when files change in ~/Downloads
const changedFile = process.argv[2];  // Path of changed file
await notify(`New download: ${changedFile}`);
```

---

## Multiple Environments

### Adding Additional Kits

Beyond the default `main/` kit, you can add additional kits under `~/.scriptkit/kit/`:

```
~/.scriptkit/kit/
├── main/              # Default kit
│   ├── scripts/
│   ├── extensions/
│   └── agents/
├── work/              # Work scripts
│   ├── scripts/
│   └── extensions/
├── personal/          # Personal scripts
│   ├── scripts/
│   └── extensions/
└── experiments/       # Experimental scripts
    └── scripts/
```

### SK_PATH Environment Variable

Override the kit path for different environments:

```bash
# Use a custom kit location
export SK_PATH="~/my-custom-kit"

# Or in a script
SK_PATH=~/work-kit ./script-kit-gpui
```

### Per-Project Kits

Create a kit in your project directory:

```
~/Projects/my-app/
├── .kit/
│   └── scripts/
│       └── dev-server.ts
├── src/
└── package.json
```

Then use `SK_PATH` to switch to it:

```bash
export SK_PATH=~/Projects/my-app/.kit
```

---

## SDK Quick Reference

### Prompt Functions

| Function | Description | Returns |
|----------|-------------|---------|
| `arg(placeholder?, choices?)` | Text input with optional choices | `Promise<string>` |
| `div(html?, config?)` | Display HTML/Tailwind content | `Promise<void>` |
| `md(markdown)` | Convert Markdown to HTML | `string` |
| `editor(content?, language?)` | Code editor | `Promise<string>` |
| `fields(definitions)` | Multi-field form | `Promise<string[]>` |
| `form(html)` | Custom HTML form | `Promise<Record<string, string>>` |
| `path(options?)` | File/folder picker | `Promise<string>` |
| `hotkey(placeholder?)` | Capture keyboard shortcut | `Promise<HotkeyInfo>` |
| `drop()` | Drag and drop zone | `Promise<FileInfo[]>` |
| `template(template, options?)` | VSCode snippet-style editor | `Promise<string>` |
| `env(key, promptFn?)` | Get/set environment variable | `Promise<string>` |
| `mini(placeholder, choices)` | Compact prompt | `Promise<string>` |
| `micro(placeholder, choices)` | Tiny prompt | `Promise<string>` |
| `select(placeholder, choices)` | Multi-select | `Promise<string[]>` |
| `term(command?)` | Terminal emulator | `Promise<string>` |
| `chat(options?)` | Chat interface | `Promise<string>` |
| `widget(html, options?)` | Floating widget window | `Promise<WidgetController>` |
| `webcam()` | Camera capture | `Promise<Buffer>` |
| `mic()` | Audio recording | `Promise<Buffer>` |
| `eyeDropper()` | Color picker | `Promise<ColorInfo>` |
| `find(placeholder, options?)` | File search (Spotlight) | `Promise<string>` |

### System Functions

| Function | Description |
|----------|-------------|
| `beep()` | Play system beep |
| `say(text, voice?)` | Text-to-speech |
| `notify(options)` | System notification |
| `hud(message, options?)` | Brief HUD notification |
| `setStatus(options)` | Set app status |
| `menu(icon, scripts?)` | Set system menu |
| `copy(text)` | Copy to clipboard |
| `paste()` | Paste from clipboard |
| `setSelectedText(text)` | Replace selected text |
| `getSelectedText()` | Get selected text |
| `hasAccessibilityPermission()` | Check accessibility permission |
| `requestAccessibilityPermission()` | Request accessibility permission |

### Clipboard Object

| Method | Description |
|--------|-------------|
| `clipboard.readText()` | Read text from clipboard |
| `clipboard.writeText(text)` | Write text to clipboard |
| `clipboard.readImage()` | Read image from clipboard |
| `clipboard.writeImage(buffer)` | Write image to clipboard |

### Keyboard Object

| Method | Description |
|--------|-------------|
| `keyboard.type(text)` | Type text |
| `keyboard.tap(...keys)` | Press key combination |

### Mouse Object

| Method | Description |
|--------|-------------|
| `mouse.move(positions)` | Move mouse along path |
| `mouse.leftClick()` | Left click |
| `mouse.rightClick()` | Right click |
| `mouse.setPosition(pos)` | Set mouse position |

### Window Control

| Function | Description |
|----------|-------------|
| `show()` | Show main window |
| `hide()` | Hide main window |
| `blur()` | Return focus to previous app |
| `getWindowBounds()` | Get window bounds |
| `submit(value)` | Force submit |
| `exit(code?)` | Exit script |
| `wait(ms)` | Delay |
| `setPanel(html)` | Set panel content |
| `setPreview(html)` | Set preview content |
| `setPrompt(html)` | Set prompt content |
| `setActions(actions)` | Set prompt actions |
| `setInput(text)` | Set input text |

### Path Utilities

| Function | Description |
|----------|-------------|
| `home(...segments)` | Path relative to ~ |
| `skPath(...segments)` | Path relative to ~/.scriptkit |
| `kitPath(...segments)` | Alias for skPath |
| `tmpPath(...segments)` | Path in temp directory |

### File Utilities

| Function | Description |
|----------|-------------|
| `isFile(path)` | Check if path is a file |
| `isDir(path)` | Check if path is a directory |
| `isBin(path)` | Check if file is executable |
| `fileSearch(query, options?)` | Search for files |

### Clipboard History

| Function | Description |
|----------|-------------|
| `clipboardHistory()` | Get clipboard history |
| `clipboardHistoryPin(entryId)` | Pin an entry |
| `clipboardHistoryUnpin(entryId)` | Unpin an entry |
| `clipboardHistoryRemove(entryId)` | Remove an entry |
| `clipboardHistoryClear()` | Clear all entries |
| `clipboardHistoryTrimOversize()` | Remove oversized entries |

### Window Management

| Function | Description |
|----------|-------------|
| `getWindows()` | Get all system windows |
| `focusWindow(windowId)` | Focus a window |
| `closeWindow(windowId)` | Close a window |
| `minimizeWindow(windowId)` | Minimize a window |
| `maximizeWindow(windowId)` | Maximize a window |
| `moveWindow(windowId, x, y)` | Move a window |
| `resizeWindow(windowId, width, height)` | Resize a window |
| `tileWindow(windowId, position)` | Tile a window |

### Miscellaneous

| Function | Description |
|----------|-------------|
| `uuid()` | Generate a UUID |
| `compile(template)` | Compile a template string |
| `browse(url)` | Open URL in browser |
| `editFile(path)` | Open file in editor |
| `run(scriptName, ...args)` | Run another script |
| `inspect(data)` | Pretty-print data |

---

## Getting Help

- **Documentation**: You're reading it!
- **Source Code**: https://github.com/johnlindquist/script-kit-gpui
- **Community**: https://scriptkit.com
- **Issues**: https://github.com/johnlindquist/script-kit-gpui/issues

---

*This guide covers Script Kit GPUI. For the original Script Kit (Electron-based), visit https://scriptkit.com.*
