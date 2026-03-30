# Script Kit — Agent Instructions

You are working inside `~/.scriptkit`, the Script Kit workspace.
Script Kit is a **Rust (GPUI) + Bun** launcher — NOT the old Electron/Node.js version.

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
│   ├── config/SKILL.md
│   └── troubleshooting/SKILL.md
├── examples/
│   ├── scripts/                   ← runnable .ts examples
│   └── extensions/                ← built-in scriptlet bundles
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

## Tab AI — Quick Terminal with Context Injection

You may be running inside a Script Kit harness terminal. When the user presses **Tab** in Script Kit, a pre-running CLI harness (Claude Code, Codex, Gemini CLI, Copilot CLI, or a custom command) receives hierarchical context via PTY stdin and renders its TUI directly in a `TermPrompt` widget.

**How context reaches you:**
- Script Kit captures UI state, selected items, frontmost app, browser URL, and clipboard
- Context is formatted as a `<scriptKitContext>` / `<scriptKitHints>` block and injected into the harness PTY
- Context assembly pipeline: `snapshot_tab_ai_ui()` + `capture_context_snapshot(CaptureContextOptions::tab_ai_submit())` + `build_tab_ai_context_from()` → `TabAiResolvedContext` (`context`, `invocationReceipt`, `suggestedIntents`)
- Target resolution: `resolve_tab_ai_surface_targets_for_view()` extracts focused/visible targets per surface
- Context injection: `build_tab_ai_harness_submission()` → `<scriptKitContext>` / `<scriptKitHints>` → `inject_tab_ai_harness_submission()` via PTY paste or line submit

**Submission modes** (`TabAiHarnessSubmissionMode`):
- `PasteOnly` — default for plain Tab entry and for any entry whose normalized intent is empty after trimming. Stages context in the PTY without submitting; user types intent next.
- `Submit` — selected when an entry intent survives trimming. With a non-empty intent, Script Kit appends `User intent:` and submits immediately.
- Sentinel behavior — `Await the user's next terminal input.` is emitted only when `TabAiHarnessSubmissionMode::Submit` is used without a non-empty intent.

**Harness configuration:**
- `HarnessConfig` — persisted at `~/.scriptkit/harness.json`, supports Claude Code, Codex, Gemini CLI, Copilot CLI, Custom backends; `warmOnStartup` defaults to `true`

**Harness lifecycle:**
- Default path — `warmOnStartup` defaults to `true`, so Script Kit silently prewarms the configured harness at app launch.
- Cold-start fallback — if prewarm is disabled, config validation fails, or the PTY has exited, the next Tab entry cold-starts the harness and waits for readiness before injecting context.
- Reuse — while the PTY stays alive, subsequent Tab presses reuse the same session.
- Recovery — if the harness crashes or exits, the next Tab entry respawns it.

**Key bindings inside the terminal:**
- `Cmd+W` closes the wrapper and restores the previous view/focus.
- Plain `Escape` is forwarded to the PTY (harness TUI owns it).
- Tab / Shift+Tab inside the terminal are forwarded to the PTY as raw bytes.

## File Watching

Script Kit watches and auto-reloads:
| Path | Effect |
|------|--------|
| `kit/config.ts` | Reloads configuration |
| `kit/theme.json` | Applies new theme |
| `kit/main/scripts/*.ts` | Updates script list |
| `kit/main/extensions/*.md` | Updates extensions |
