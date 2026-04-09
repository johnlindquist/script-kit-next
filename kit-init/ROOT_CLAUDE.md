# Script Kit — Agent Instructions

You are working inside `~/.scriptkit`, the Script Kit workspace.
Script Kit is a **Rust (GPUI) + Bun** launcher — NOT the old Electron/Node.js version.

## One-Shot First

Use `~/.scriptkit/kit/examples/START_HERE.md` as the canonical one-shot authoring guide.
Open it first when the user wants one new Script Kit artifact in harness mode.
Use the rest of this file for workspace rules and Tab AI runtime contract after the artifact type is already chosen.

## Fast Route

Use this plain-text route first:

### Script
- Use for Script Kit UI, Bun APIs, files, HTTP, or multi-step logic
- Write to `~/.scriptkit/kit/main/scripts/<name>.ts`

### Extension bundle / scriptlet bundle
- Use for snippets, text expansion, quick shell commands, or grouped helpers
- Write to `~/.scriptkit/kit/main/extensions/<name>.md`

### Skill (preferred reusable AI unit)
- Use for reusable AI instructions that open ACP Chat when selected from the main menu
- Write to `~/.scriptkit/kit/main/skills/<name>/SKILL.md`
- Skills are the preferred way to package reusable AI behavior — plugins are the package boundary

### mdflow agent (compatibility)
- Use only when you need a specific backend suffix or legacy mdflow features
- Write to `~/.scriptkit/kit/main/agents/<name>.<backend>.md`
- For new reusable AI work, prefer creating a skill instead

Script Kit uses **extension bundle** and **scriptlet bundle** to mean the same artifact.

## Guardrails

- Create exactly one artifact per request.
- Save runnable user files only under `~/.scriptkit/kit/main/`.
- Do not create a `.ts` script when the request is really a bundle, skill, or agent.
- For new reusable AI work, create a skill (`kit/main/skills/<name>/SKILL.md`), not an agent.
- For `tool:<name>` scriptlets, the first line must be `import "@scriptkit/sdk";`.
- Agent files do not use `export const metadata`; use underscore-prefixed `_sk_*` keys.
- Choose the backend suffix deliberately: `.claude.md`, `.gemini.md`, `.codex.md`, `.copilot.md`, or `.i.gemini.md`.

## Read Next

- Canonical launchpad → `~/.scriptkit/kit/examples/START_HERE.md`
- Machine-readable SDK reference → `kit://sdk-reference`
- Script details → `~/.scriptkit/kit/authoring/skills/script-authoring/SKILL.md`
- Bundle details → `~/.scriptkit/kit/authoring/skills/scriptlets/SKILL.md`
- Skills overview → `~/.scriptkit/kit/authoring/skills/README.md`
- Agent details (compatibility) → `~/.scriptkit/kit/authoring/skills/agents/SKILL.md`
- Script example → `~/.scriptkit/kit/examples/scripts/hello-world.ts`
- Bundle starter → `~/.scriptkit/kit/examples/extensions/starter.md`
- Agent example → `~/.scriptkit/kit/examples/agents/review-pr.claude.md`

## Directory Layout

```
~/.scriptkit/                      ← you are here (harness cwd)
├── CLAUDE.md                      ← this file
├── AGENTS.md                      ← SDK reference for all agents
├── GUIDE.md                       ← comprehensive user guide
├── kit/                           ← user workspace (version-controllable)
│   ├── main/
│   │   ├── plugin.json            ← plugin manifest
│   │   ├── scripts/               ← PUT NEW SCRIPTS HERE
│   │   ├── extensions/            ← markdown scriptlet bundles
│   │   ├── skills/                ← AI skills (preferred reusable AI unit)
│   │   └── agents/                ← legacy agent definitions (compatibility)
│   ├── authoring/
│   │   ├── plugin.json
│   │   └── skills/                ← agent skills (read these!)
│   │       ├── script-authoring/SKILL.md
│   │       ├── scriptlets/SKILL.md
│   │       ├── agents/SKILL.md
│   │       ├── config/SKILL.md
│   │       └── troubleshooting/SKILL.md
│   ├── examples/
│   │   ├── plugin.json
│   │   ├── scripts/               ← runnable .ts examples
│   │   ├── extensions/            ← built-in scriptlet bundles
│   │   └── agents/                ← mdflow agent examples
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

## Artifact-Specific Rules

### Script Rules
- Start the file with `import "@scriptkit/sdk";`
- Use `export const metadata = { name, description }`
- Save to `kit/main/scripts/*.ts`
- Use Bun APIs: `Bun.file()`, `Bun.write()`, and `` $`command` ``

### Extension Bundle / Scriptlet Bundle Rules
- Save one markdown file to `kit/main/extensions/*.md`
- Prefer `metadata` code fences for new bundles
- Use `import "@scriptkit/sdk";` only inside `tool:<name>` fences, as the first line of that fence
- Do not put `export const metadata` at the top of the markdown file

### Skill Rules (Preferred Reusable AI Unit)
- Create a directory under `kit/main/skills/<name>/`
- Add a `SKILL.md` file with YAML frontmatter (`name`, `description`)
- Skills appear in the main menu and always open ACP Chat when selected
- Plugins are the package boundary — each plugin owns its own skills
- Prefer skills over agents for any new reusable AI work

### mdflow Agent Rules (Compatibility)
- Save to `kit/main/agents/<name>.<backend>.md`
- Use underscore-prefixed `_sk_*` metadata keys
- Do not use `export const metadata`
- Do not add `import "@scriptkit/sdk"` to the markdown file
- For new reusable AI work, prefer creating a skill instead

## Avoid These Mistakes

- Do not create more than one artifact for one request
- Do not put scripts in `extensions/` or `agents/`
- Do not put bundles in `scripts/`
- Do not put agents in `scripts/` or `extensions/`
- Do not put skills in `scripts/` or `extensions/` — skills are `SKILL.md` directories under `skills/`
- Do not create new agents when a skill would work — agents are a compatibility path
- Do not use CommonJS or the old v1 SDK package
- Do not edit `sdk/`

The Core SDK examples below apply to `.ts` scripts and `tool:<name>` scriptlets. They do not apply to skills or mdflow agent markdown files.

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

Read `kit/authoring/skills/` for detailed guidance on:
- **script-authoring** — creating and structuring scripts
- **scriptlets** — markdown extension bundles with embedded commands
- **agents** — mdflow-backed agent files
- **config** — configuration and theming
- **troubleshooting** — common issues and debugging

## Examples

See `kit/examples/scripts/` for working examples:
- `hello-world.ts` — basic prompt and display
- `choose-from-list.ts` — rich choices with preview
- `clipboard-transform.ts` — clipboard read/transform/write
- `path-picker.ts` — file system operations

## Configuration

- **Config**: `kit/config.ts` — hotkeys, font sizes, built-in features
- **Theme**: `kit/theme.json` — colors (hex, rgb, rgba)
- **TypeScript**: `kit/tsconfig.json` — managed by app, maps `@scriptkit/sdk`

## Tab AI — Quick Terminal with Flat Context Injection

Tab AI's PTY-backed verification path renders in `AppView::QuickTerminalView` via `TermPrompt`.

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
- Harness config: `claudeCode` block in `~/.scriptkit/kit/config.ts`
- Context bundle: `~/.scriptkit/context/latest.md` (deterministic path)

- Context assembly stays intact: `snapshot_tab_ai_ui()` + `capture_context_snapshot(CaptureContextOptions::tab_ai_submit())` + `build_tab_ai_context_from()`
- The landed default Tab flow is PTY-backed text injection
- `build_tab_ai_harness_submission()` emits a flat text-native context block plus optional artifact authoring guidance
- No XML wrappers are used in the landed PTY path
- `PasteOnly` stages context on a fresh line and does not auto-submit
- `Submit` with a non-empty intent appends `User intent:` and submits immediately
- `Submit` without a non-empty intent appends `Await the user's next terminal input.`

**Capture profiles:**
- Generic PTY backends use `CaptureContextOptions::tab_ai_submit()` (text-safe, no screenshots — base64 PNG in PTY stdin is fragile).
- The richer `tab_ai()` profile with screenshots is reserved for a future Claude-specific SDK path.

**Harness lifecycle:**
- Each explicit quick-terminal open writes `~/.scriptkit/context/latest.md`, enumerates `~/.scriptkit/kit/authoring/skills/`, and behaves as a one-shot spawn rendered in `QuickTerminalView`.
- Internal silent prewarm may seed the PTY ahead of time, but that is a single-use implementation detail rather than a documented warm multi-turn surface.
- Recovery — if the harness crashes or exits, the next Tab entry respawns it.

**ACP Chat vs harness terminal:**
- Plain `Tab` opens the harness terminal, not ACP Chat.
- ACP Chat is opened explicitly from skills or ACP SDK entry points (`aiStartChat()`, `aiFocus()`).
- Do not describe plain Tab as opening ACP Chat.

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
