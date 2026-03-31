# Script Kit — Agent Instructions

You are working inside `~/.scriptkit`, the Script Kit workspace.
Script Kit is a **Rust (GPUI) + Bun** launcher — NOT the old Electron/Node.js version.

## Pick the Artifact Type First

Before writing files, decide which artifact the user actually asked for.

| Artifact | Use when | Write here | Learn here | Reference here |
|----------|----------|------------|------------|----------------|
| Script | Full TypeScript workflow with Script Kit UI or Bun APIs | `~/.scriptkit/kit/main/scripts/<name>.ts` | `~/.scriptkit/skills/script-authoring/SKILL.md` | `~/.scriptkit/examples/scripts/` |
| Extension bundle | One markdown file containing multiple scriptlets, snippets, or quick commands | `~/.scriptkit/kit/main/extensions/<name>.md` | `~/.scriptkit/skills/scriptlets/SKILL.md` | `~/.scriptkit/examples/extensions/` |
| mdflow agent | Backend-specific markdown prompt/automation | `~/.scriptkit/kit/main/agents/<name>.<backend>.md` | `~/.scriptkit/skills/agents/SKILL.md` | `~/.scriptkit/examples/agents/` |

Do not create a `.ts` script when the request is really a scriptlet bundle or mdflow agent.
Do not write runnable user files outside `~/.scriptkit/kit/main/`.

## Minimal Starter Templates

Use one of these and stop at the smallest working version.

### Script → `~/.scriptkit/kit/main/scripts/<name>.ts`

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "My Script",
  description: "What it does",
};

const value = await arg("What should this script do?");
await div(`<div class="p-8 text-2xl">${value}</div>`);
```

### Extension bundle → `~/.scriptkit/kit/main/extensions/<name>.md`

~~~md
---
name: My Bundle
description: Personal helpers
icon: sparkles
---

## Hello Snippet

```metadata
keyword: !hello
description: Quick greeting
```

```paste
Hello!
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

### mdflow agent → `~/.scriptkit/kit/main/agents/<name>.<backend>.md`

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

### One-shot Rules

- Pick the smallest artifact that fits.
- Save only under `~/.scriptkit/kit/main/`.
- For scripts, always start with `import "@scriptkit/sdk";`.
- For extension bundles, prefer `metadata` code fences instead of HTML comments.
- For `tool:<name>` scriptlets, the first line must be `import "@scriptkit/sdk";`.

## Fastest One-Shot Path

When the request is "make one new Script Kit artifact fast," open:

- `~/.scriptkit/examples/START_HERE.md`

That file gives one starter per artifact, exact destinations, and ready-to-run copy commands.

## Quick Start

```typescript
// ~/.scriptkit/kit/main/scripts/my-script.ts
import "@scriptkit/sdk";

export const metadata = {
  name: "My Script",
  description: "What it does",
};

const choice = await arg("Pick one", ["A", "B", "C"]);
await div(`<h1>${choice}</h1>`);
```

## Directory Layout

```
~/.scriptkit/                      ← you are here (harness cwd)
├── CLAUDE.md                      ← this file
├── AGENTS.md                      ← SDK reference for all agents
├── GUIDE.md                       ← comprehensive user guide
├── skills/                        ← agent skills (read these!)
│   ├── script-authoring/SKILL.md
│   ├── scriptlets/SKILL.md
│   ├── agents/SKILL.md
│   ├── config/SKILL.md
│   └── troubleshooting/SKILL.md
├── examples/
│   ├── scripts/                   ← runnable .ts examples
│   ├── extensions/                ← built-in scriptlet bundles
│   └── agents/                    ← mdflow agent examples
├── kit/                           ← user workspace (version-controllable)
│   ├── main/
│   │   ├── scripts/               ← PUT NEW SCRIPTS HERE
│   │   ├── extensions/            ← markdown scriptlet bundles
│   │   └── agents/                ← AI agent definitions
│   ├── config.ts                  ← user configuration
│   ├── theme.json                 ← theme colors
│   ├── package.json               ← enables top-level await
│   └── tsconfig.json              ← TypeScript + SDK path mapping
├── sdk/                           ← managed by app (DO NOT EDIT)
│   └── kit-sdk.ts
├── db/                            ← databases
├── logs/                          ← app logs
└── cache/                         ← cached data
```

## Rules

1. **Always** `import "@scriptkit/sdk";` as the first line
2. **Always** use `export const metadata = { name, description }` — NOT comment metadata
3. **Scripts go in** `kit/main/scripts/*.ts`
4. **Extensions go in** `kit/main/extensions/*.md`
5. **Use Bun APIs**: `Bun.file()`, `Bun.write()`, `` $`command` `` — NOT Node.js fs/child_process
6. **Top-level await** works everywhere (package.json has `"type": "module"`)

## DO NOT

- Use CommonJS imports — use ES `import` syntax
- Use the old v1 SDK package — use `@scriptkit/sdk`
- Use Node.js `fs` or `child_process` — use Bun APIs
- Use comment-based metadata — use `export const metadata`
- Edit files in `sdk/` — they are managed by the app
- Reference legacy v1 paths — scripts live in `kit/main/scripts/`
- Create scripts outside `kit/main/scripts/`

## Core SDK Functions

```typescript
// Prompt for input
const text = await arg("Enter something");
const choice = await arg("Pick one", ["Option 1", "Option 2"]);

// Rich choices with metadata
const item = await arg("Search", [
  { name: "First", description: "The first option", value: "first" },
  { name: "Second", description: "The second option", value: "second" },
]);

// Display HTML (Tailwind CSS available)
await div(`<div class="p-8"><h1 class="text-2xl font-bold">Hello</h1></div>`);

// Code editor
const code = await editor("// Edit this", "typescript");

// Form fields
const [name, email] = await fields([
  { name: "name", label: "Name" },
  { name: "email", label: "Email", type: "email" },
]);

// Clipboard
const text = await paste();
await copy("Copied!");

// File picker
const file = await path("Choose a file");

// Shell commands (Bun shell)
const result = await $`ls -la ~/Desktop`.text();

// Open URLs/apps
await open("https://example.com");

// Notifications
await notify("Task complete!");
```

## Skills

Read `skills/` for detailed guidance on:
- **script-authoring** — creating and structuring scripts
- **scriptlets** — markdown extension bundles with embedded commands
- **agents** — mdflow-backed agent files
- **config** — configuration and theming
- **troubleshooting** — common issues and debugging

## Examples

See `examples/scripts/` for working examples:
- `hello-world.ts` — basic prompt and display
- `choose-from-list.ts` — rich choices with preview
- `clipboard-transform.ts` — clipboard read/transform/write
- `path-picker.ts` — file system operations

## Configuration

- **Config**: `kit/config.ts` — hotkeys, font sizes, built-in features
- **Theme**: `kit/theme.json` — colors (hex, rgb, rgba)
- **TypeScript**: `kit/tsconfig.json` — managed by app, maps `@scriptkit/sdk`

## Tab AI — Quick Terminal with Flat Context Injection

Tab AI is a warm harness terminal rendered in `AppView::QuickTerminalView` via `TermPrompt`.

**Entry path:**
- Plain `Tab` opens the harness terminal and stages a flat labeled `Script Kit context` block using `TabAiHarnessSubmissionMode::PasteOnly`.
- `Shift+Tab` in `AppView::ScriptList` with non-empty filter text opens the same harness surface and submits that filter text as `User intent:` using `TabAiHarnessSubmissionMode::Submit`.
- `Tab` / `Shift+Tab` inside `AppView::QuickTerminalView` are forwarded to the PTY. Do not describe them as focus-navigation keys once the harness terminal is open.

**Close semantics:**
- `Cmd+W` closes the wrapper and restores the previous view and focus.
- Plain `Escape` is forwarded to the PTY. The harness TUI owns Escape behavior.
- The footer hint strip advertises only `⌘W Close`.

**Runtime contract:**
- Entry path: `open_tab_ai_chat()` → `begin_tab_ai_harness_entry()` → `open_tab_ai_harness_terminal_from_request()`
- Harness session state: `TabAiHarnessSessionState`
- Harness config: `~/.scriptkit/harness.json`
- Supported backends: Claude Code, Codex, Gemini CLI, Copilot CLI, and custom commands
- `warmOnStartup` defaults to `true`
- Context assembly stays intact: `snapshot_tab_ai_ui()` + `capture_context_snapshot(CaptureContextOptions::tab_ai_submit())` + `build_tab_ai_context_from()`
- The landed default Tab flow is PTY-backed text injection
- `build_tab_ai_harness_submission()` emits a flat text-native context block plus optional artifact authoring guidance
- No XML wrappers are used in the landed PTY path
- `PasteOnly` stages context on a fresh line and does not auto-submit
- `Submit` with a non-empty intent appends `User intent:` and submits immediately
- `Submit` without a non-empty intent appends `Await the user's next terminal input.`

**Do not describe as current behavior:**
- Do not describe the old inline chat entity or custom streaming UI as the primary Tab AI surface
- Do not describe the old inline chat or custom streaming UI as the default path
- Do not describe Claude Agent SDK V2 or screenshot attachment support as already landed in the default Tab flow

## File Watching

Script Kit watches and auto-reloads:
| Path | Effect |
|------|--------|
| `kit/config.ts` | Reloads configuration |
| `kit/theme.json` | Applies new theme |
| `kit/main/scripts/*.ts` | Updates script list |
| `kit/main/extensions/*.md` | Updates extensions |
