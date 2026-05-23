# Script Kit — Agent Instructions

You are working inside `~/.scriptkit`, the Script Kit workspace.
Script Kit is a **Rust (GPUI) + Bun** launcher — NOT the old Electron/Node.js version.

## One-Shot First

Use `~/.scriptkit/plugins/examples/START_HERE.md` as the canonical one-shot creation guide.
Open it first when the user wants one new Script Kit artifact in harness mode.
Use the rest of this file for workspace rules and Tab AI runtime contract after the artifact type is already chosen.

## Fast Route

Use this plain-text route first:

### Script
- Use for Script Kit UI, Bun APIs, files, HTTP, or multi-step logic
- Write to `~/.scriptkit/plugins/main/scripts/<name>.ts`

### Scriptlet bundle
- Use for snippets, text expansion, quick shell commands, or grouped helpers
- Write to `~/.scriptkit/plugins/main/scriptlets/<name>.md`

### Skill (preferred reusable AI unit)
- Use for reusable AI instructions that open Agent Chat when selected from the main menu
- Write to `~/.scriptkit/plugins/main/skills/<name>/SKILL.md`
- Skills are the preferred way to package reusable AI behavior — plugins are the package boundary

### mdflow agent (compatibility)
- Use only when you need a specific backend suffix or legacy mdflow features
- Write to `~/.scriptkit/plugins/main/agents/<name>.<backend>.md`
- For new reusable AI work, prefer creating a skill instead

## Guardrails

- Create exactly one artifact per request.
- Save runnable user files only under `~/.scriptkit/plugins/main/`.
- Do not create a `.ts` script when the request is really a bundle, skill, or agent.
- For new reusable AI work, create a skill (`plugins/main/skills/<name>/SKILL.md`), not an agent.
- For `tool:<name>` scriptlets, the first line must be `import "@scriptkit/sdk";`.
- Agent files do not use `export const metadata`; use underscore-prefixed `_sk_*` keys.
- Choose the backend suffix deliberately: `.claude.md`, `.agy.md`, `.codex.md`, `.copilot.md`, or `.i.agy.md`.

## Read Next

- Canonical launchpad → `~/.scriptkit/plugins/examples/START_HERE.md`
- Machine-readable SDK reference → `kit://sdk-reference`
- Script details → `~/.scriptkit/plugins/scriptkit/skills/new-script/SKILL.md`
- Bundle details → `~/.scriptkit/plugins/scriptkit/skills/new-scriptlet/SKILL.md`
- Skills overview → `~/.scriptkit/plugins/scriptkit/skills/README.md`
- Agent details (compatibility) → `~/.scriptkit/plugins/scriptkit/skills/new-agent/SKILL.md`
- Script example → `~/.scriptkit/plugins/examples/scripts/todo-app.ts`

## Directory Layout

```
~/.scriptkit/                      ← you are here (harness cwd)
├── CLAUDE.md                      ← this file
├── AGENTS.md                      ← SDK reference for all agents
├── GUIDE.md                       ← comprehensive user guide
├── plugins/                       ← plugin roots
│   ├── main/
│   │   ├── plugin.json            ← plugin manifest
│   │   ├── scripts/               ← PUT NEW SCRIPTS HERE
│   │   ├── scriptlets/            ← markdown scriptlet bundles
│   │   ├── skills/                ← AI skills (preferred reusable AI unit)
│   │   └── agents/                ← legacy agent definitions (compatibility)
│   ├── scriptkit/
│   │   ├── plugin.json
│   │   └── skills/                ← agent skills (read these!)
│   │       ├── new-script/SKILL.md
│   │       ├── new-scriptlet/SKILL.md
│   │       ├── new-agent/SKILL.md
│   │       ├── update-config/SKILL.md
│   │       └── troubleshoot/SKILL.md
│   ├── examples/
│   │   ├── plugin.json
│   │   ├── README.md
│   │   ├── START_HERE.md
│   │   └── scripts/
│   │       └── todo-app.ts        ← runnable Todo app example
├── config.ts                      ← user configuration
├── theme.json                     ← theme colors
├── package.json                   ← enables top-level await
├── tsconfig.json                  ← TypeScript + SDK path mapping
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
- Save to `plugins/main/scripts/*.ts`
- Use Bun APIs: `Bun.file()`, `Bun.write()`, and `` $`command` ``

### Scriptlet Bundle Rules
- Save one markdown file to `plugins/main/scriptlets/*.md`
- Prefer `metadata` code fences for new bundles
- Use `import "@scriptkit/sdk";` only inside `tool:<name>` fences, as the first line of that fence
- Do not put `export const metadata` at the top of the markdown file

### Skill Rules (Preferred Reusable AI Unit)
- Create a directory under `plugins/main/skills/<name>/`
- Add a `SKILL.md` file with YAML frontmatter (`name`, `description`)
- Skills appear in the main menu and always open Agent Chat when selected
- Plugins are the package boundary — each plugin owns its own skills
- Prefer skills over agents for any new reusable AI work

### mdflow Agent Rules (Compatibility)
- Save to `plugins/main/agents/<name>.<backend>.md`
- Use underscore-prefixed `_sk_*` metadata keys
- Do not use `export const metadata`
- Do not add `import "@scriptkit/sdk"` to the markdown file
- For new reusable AI work, prefer creating a skill instead

## Avoid These Mistakes

- Do not create more than one artifact for one request
- Do not put scripts in `scriptlets/` or `agents/`
- Do not put bundles in `scripts/`
- Do not put agents in `scripts/` or `scriptlets/`
- Do not put skills in `scripts/` or `scriptlets/` — skills are `SKILL.md` directories under `skills/`
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

// Feedback: `hud(message)` for in-launcher overlay, `notify(message)`
// for an OS-level system notification (Notification Center).
hud("Task complete!");
```

## Skills

Read `plugins/scriptkit/skills/` for detailed guidance on:
- **new-script** — creating and updating scripts
- **new-scriptlet** — markdown scriptlet bundles with embedded commands
- **new-agent** — mdflow-backed agent files
- **update-config** — configuration and theming
- **troubleshoot** — common issues and debugging

## Examples

See `plugins/examples/` for working examples:
- `todo-app.ts` — local Todo app with projects, labels, priorities, due dates, and `;todo` capture sync

## Configuration

- **Config**: `config.ts` — hotkeys, font sizes, built-in features
- **Theme**: `theme.json` — colors (hex, rgb, rgba)
- **TypeScript**: `tsconfig.json` — managed by app, maps `@scriptkit/sdk`

## Tab AI — Quick Terminal with Flat Context Injection

Tab AI now has two distinct surfaces: Agent Chat is the default AI chat UI, while `AppView::QuickTerminalView` remains the PTY-backed harness surface rendered by `TermPrompt`.

**Entry path:**
- Plain `Tab` in `AppView::ScriptList` routes through the ACP entry path and context-capture helpers. Do not describe plain Tab as opening the harness terminal by default.
- `QuickTerminalView` is opened by explicit harness / verification flows that need a PTY-backed surface.
- `Tab` / `Shift+Tab` inside `AppView::QuickTerminalView` are forwarded to the PTY. Do not describe them as focus-navigation keys once the harness terminal is open.

**Close semantics:**
- `Cmd+W` closes the wrapper and restores the previous view and focus.
- Plain `Escape` is forwarded to the PTY. The harness TUI owns Escape behavior.
- The footer hint strip advertises only `⌘W Close`.

**Runtime contract:**
- Canonical chat entry: `open_tab_ai_acp_with_entry_intent(...)`
- Quick-terminal path: `begin_tab_ai_harness_entry()` → `open_tab_ai_harness_terminal_from_request()`
- Harness session state: `TabAiHarnessSessionState`
- Harness config: `claudeCode` block in `~/.scriptkit/config.ts`
- Context bundle: `~/.scriptkit/context/latest.md` (deterministic path)
- Context assembly stays intact: `snapshot_tab_ai_ui()` + `capture_context_snapshot(CaptureContextOptions::tab_ai_submit())` + `build_tab_ai_context_from()`
- `build_tab_ai_harness_submission()` emits a flat text-native context block plus optional artifact authoring guidance
- Quick-terminal submissions are encoded with `TabAiHarnessSubmissionMode` and currently use the `PasteOnly` / `Submit` variants.
- No XML wrappers are used in the landed PTY path
- `PasteOnly` stages context on a fresh line and does not auto-submit
- `Submit` with a non-empty intent appends `User intent:` and submits immediately
- `Submit` without a non-empty intent appends `Await the user's next terminal input.`

**Capture profiles:**
- Generic PTY backends use `CaptureContextOptions::tab_ai_submit()` (text-safe, no screenshots — base64 PNG in PTY stdin is fragile).
- The richer `tab_ai()` profile with screenshots is reserved for a future Claude-specific SDK path.

**Harness lifecycle:**
- Each explicit quick-terminal open writes `~/.scriptkit/context/latest.md`, enumerates plugin-owned skills under `~/.scriptkit/plugins/*/skills/`, and behaves as a one-shot spawn rendered in `QuickTerminalView`.
- Internal silent prewarm may seed the PTY ahead of time, but that is a single-use implementation detail rather than a documented warm multi-turn surface.
- Recovery — if the harness crashes or exits, the next explicit quick-terminal entry respawns it.

**Agent Chat vs harness terminal:**
- Agent Chat is the user-facing default AI chat surface.
- Quick Terminal is the PTY-backed harness surface for terminal-native verification and authoring flows.
- Do not describe plain Tab as opening the harness terminal or `Shift+Tab` in `AppView::ScriptList` as the default quick-submit path.

**Do not describe as current behavior:**
- Do not describe the old inline chat entity or custom streaming UI as the primary Tab AI surface
- Do not describe the old inline chat or custom streaming UI as the default path
- Do not describe Claude Agent SDK V2 or screenshot attachment support as already landed in the default Tab flow

## File Watching

Script Kit watches and auto-reloads:
| Path | Effect |
|------|--------|
| `config.ts` | Reloads configuration |
| `theme.json` | Applies new theme |
| `plugins/main/scripts/*.ts` | Updates script list |
| `plugins/main/scriptlets/*.md` | Updates extensions |
