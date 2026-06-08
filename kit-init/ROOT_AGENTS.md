# Script Kit SDK Reference

Complete reference for AI agents creating Script Kit artifacts: scripts, scriptlet bundles, skills, and mdflow agents. Plugins are the package boundary; skills are the preferred reusable AI unit.

> **Package**: `@scriptkit/sdk` — **Runtime**: Bun — **Write under**: `~/.scriptkit/plugins/main/{scripts,scriptlets,skills,agents,profiles}`

## One-Shot First

Use `~/.scriptkit/plugins/examples/START_HERE.md` as the canonical one-shot creation guide.
Open it first when the user wants one new Script Kit artifact in harness mode.
Use this file only after the artifact type is already chosen and you need deeper SDK reference.

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

### Agent Chat profile
- Use for isolated Pi-backed Agent Chat runtime boundaries
- Write to `~/.scriptkit/plugins/main/profiles/<profile-id>/{profile.json,PROMPT.md,README.md}`
- Profiles define prompts, provider/model, tools, cwd/session policy, and ambient-resource isolation. `pathPolicy` is schema metadata until runtime path enforcement is proven.

### mdflow agent (compatibility)
- Use only when you need a specific backend suffix or legacy mdflow features
- Write to `~/.scriptkit/plugins/main/agents/<name>.<backend>.md`
- For new reusable AI work, prefer creating a skill instead

## Guardrails

- Create exactly one artifact per request.
- Save runnable user files only under `~/.scriptkit/plugins/main/`.
- Do not create a `.ts` script when the request is really a bundle, skill, or agent.
- For new reusable AI work, create a skill (`plugins/main/skills/<name>/SKILL.md`), not an agent.
- For a custom isolated Agent Chat runtime, create a profile (`plugins/main/profiles/<profile-id>/profile.json`), not a legacy agent.
- For `tool:<name>` scriptlets, the first line must be `import "@scriptkit/sdk";`.
- Agent files do not use `export const metadata`; use underscore-prefixed `_sk_*` keys.
- Choose the backend suffix deliberately: `.claude.md`, `.agy.md`, `.codex.md`, `.copilot.md`, or `.i.agy.md`.

## Read Next

- Canonical launchpad → `~/.scriptkit/plugins/examples/START_HERE.md`
- Machine-readable SDK reference → `kit://sdk-reference`
- Script details → `~/.scriptkit/plugins/scriptkit/skills/new-script/SKILL.md`
- Bundle details → `~/.scriptkit/plugins/scriptkit/skills/new-scriptlet/SKILL.md`
- Skills overview → `~/.scriptkit/plugins/scriptkit/skills/README.md`
- Profile builder → `~/.scriptkit/plugins/scriptkit/skills/build-profile/SKILL.md`
- Agent details (compatibility) → `~/.scriptkit/plugins/scriptkit/skills/new-agent/SKILL.md`
- Script example → `~/.scriptkit/plugins/examples/scripts/todo-app.ts`

## Artifact-Specific Rules

### Script
- First line: `import "@scriptkit/sdk";`
- Use `export const metadata = { name, description }`
- Use Bun APIs instead of Node-only APIs

### Scriptlet bundle
- Save one markdown file under `~/.scriptkit/plugins/main/scriptlets/`
- Prefer `metadata` code fences for new bundles
- `tool:<name>` fences must begin with `import "@scriptkit/sdk";`
- Do not add `export const metadata` at the top of the markdown file

### Skill (preferred reusable AI unit)
- Create a directory under `~/.scriptkit/plugins/main/skills/<name>/`
- Add a `SKILL.md` file with YAML frontmatter (`name`, `description`)
- Skills appear in the main menu and always open Agent Chat when selected
- Plugins are the package boundary — each plugin owns its own skills
- Prefer skills over agents for any new reusable AI work

### Agent Chat profile
- Create a directory under `~/.scriptkit/plugins/main/profiles/<profile-id>/`
- Add `profile.json`, `PROMPT.md`, `README.md`, and focused examples
- Default to `backend: "pi"`, `provider: "openai-codex"`, disabled extensions, disabled skills, disabled prompt templates, disabled context files, and explicit path-policy metadata
- Do not put profile artifacts in `agents/`; agents are a compatibility/import source

### mdflow agent (compatibility)
- Save to `~/.scriptkit/plugins/main/agents/<name>.<backend>.md`
- Use `_sk_*` metadata keys
- Do not add `import "@scriptkit/sdk"`
- Do not use `export const metadata`
- For new reusable AI work, prefer creating a skill instead

For exact function signatures, treat `kit://sdk-reference` as the source of truth.

The `Prompts`, `Clipboard`, `Shell Commands`, `File Operations`, and `Notifications & Feedback` sections below apply to `.ts` scripts and `tool:<name>` scriptlets. They do not apply to mdflow agent markdown files.

## Prompts

### arg — Universal Input

```typescript
// Simple text input
const text = await arg("Enter something");

// Choices (string array)
const pick = await arg("Choose", ["Red", "Green", "Blue"]);

// Rich choices (objects)
const item = await arg("Search items", [
  { name: "GitHub", description: "Open GitHub", value: "gh" },
  { name: "Docs", description: "Open docs", value: "docs" },
]);

// Choices with preview panel
const selected = await arg(
  "Preview example",
  [
    { name: "Alpha", value: "a", preview: "<h1>Alpha Details</h1>" },
    { name: "Beta", value: "b", preview: "<h1>Beta Details</h1>" },
  ],
);

// Dynamic/async choices
const result = await arg("Search GitHub", async (input) => {
  if (!input) return [];
  const res = await fetch(`https://api.github.com/search/repositories?q=${input}`);
  const data = await res.json();
  return data.items.map((r: any) => ({
    name: r.full_name,
    description: r.description || "",
    value: r.html_url,
  }));
});
```

### div — HTML Display

```typescript
// Display HTML with Tailwind CSS
await div(`
  <div class="flex flex-col items-center justify-center h-full p-8">
    <h1 class="text-4xl font-bold text-yellow-400">Hello!</h1>
    <p class="text-gray-400 mt-4">Press Escape to close</p>
  </div>
`);
```

### editor — Code Editor

```typescript
const code = await editor("// Write code here", "typescript");
const json = await editor('{"key": "value"}', "json");
```

### fields — Form Input

```typescript
const [name, email, age] = await fields([
  { name: "name", label: "Full Name" },
  { name: "email", label: "Email", type: "email" },
  { name: "age", label: "Age", type: "number" },
]);
```

### path — File Picker

```typescript
const file = await path("Choose a file");
const dir = await path("Choose a directory");
```

## Clipboard

```typescript
const text = await paste();       // Read clipboard
await copy("Hello clipboard!");   // Write to clipboard
```

## Shell Commands (Bun Shell)

```typescript
// Simple command
const output = await $`ls -la ~/Desktop`.text();

// Piped commands
const count = await $`find ~/Documents -name "*.md" | wc -l`.text();

// With error handling
try {
  await $`git status`;
} catch (e) {
  console.error("Not a git repo");
}
```

## File Operations (Bun APIs)

```typescript
// Read file
const content = await Bun.file("~/notes.txt").text();

// Write file
await Bun.write("~/output.txt", "Hello, world!");

// Check existence
const exists = await Bun.file("~/config.json").exists();

// Read JSON
const data = await Bun.file("~/data.json").json();

// Write JSON
await Bun.write("~/data.json", JSON.stringify(data, null, 2));
```

## Feedback

```typescript
hud("Task complete!");                   // HUD feedback (fire-and-forget)
await copy("Copied to clipboard");       // Copy + implicit feedback
await open("https://example.com");       // Open URL/file in default app
```

Pick the right feedback API for the caller's intent:

- `hud(message)` — fire-and-forget in-launcher overlay. Best for confirmations while the user is still interacting with the launcher (e.g. "Copied").
- `notify(message)` — OS-level system notification (macOS Notification Center). Use when the message should be visible after the launcher closes, or when the script runs headless/background.

## Extensions (Scriptlet Bundles)

For first-pass creation, copy the minimal starter above. The rest of this section is detailed reference.

Markdown files at `~/.scriptkit/plugins/main/scriptlets/*.md`:

~~~markdown
---
name: My Tools
description: Useful helpers
icon: sparkles
---

## Say Hello

```metadata
keyword: !hello
description: Display a greeting
shortcut: ctrl h
```

```paste
Hello from Script Kit!
```

## Quick Note

```metadata
description: Save a quick note
```

```tool:quick-note
import "@scriptkit/sdk";

const note = await arg("Note");
await Bun.write(`${env.HOME}/quick-note.txt`, note);
hud("Saved");
```
~~~

### Scriptlet Types

| Fence tag | Behavior |
|-----------|----------|
| `` ```paste `` | Static text expansion |
| `` ```bash `` | Run shell command |
| `` ```tool:name `` | Run TypeScript with SDK |
| `` ```template:name `` | Text template with `{{placeholders}}` |

Prefer `metadata` code fences for `keyword`, `description`, `shortcut`, `alias`, `schedule`, `cron`, `icon`, and boolean flags.
Legacy HTML comments are still parsed, but do not generate them for new bundles.

## Metadata Fields

```typescript
export const metadata = {
  name: "Display Name",           // Required: shown in menu
  description: "What it does",    // Required: shown when focused
  shortcut: "cmd shift x",        // Global hotkey
  alias: "dn",                    // Quick search alias
  // trigger: "!hello",           // Snippet trigger
  // background: true,            // Run without showing UI
};
```

## Config Reference

`~/.scriptkit/config.ts`:

```typescript
import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: { key: "Space", modifiers: ["command"] },
  editorFontSize: 16,
  terminalFontSize: 14,
  builtIns: {
    clipboardHistory: true,
    appLauncher: true,
  },
} satisfies Config;
```

## Theme Reference

`~/.scriptkit/theme.json`:

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

## Path Reference

| Purpose | Path |
|---------|------|
| Scripts | `~/.scriptkit/plugins/main/scripts/*.ts` |
| Extensions | `~/.scriptkit/plugins/main/scriptlets/*.md` |
| Skills (preferred AI unit) | `~/.scriptkit/plugins/main/skills/<name>/SKILL.md` |
| Agent Chat profiles | `~/.scriptkit/plugins/main/profiles/<profile-id>/profile.json` |
| Agents (compatibility) | `~/.scriptkit/plugins/main/agents/*.md` |
| Config | `~/.scriptkit/config.ts` |
| Theme | `~/.scriptkit/theme.json` |
| SDK | `~/.scriptkit/sdk/kit-sdk.ts` (do not edit) |
| CLI | `~/.scriptkit/bin/scriptkit` (managed command shim; do not edit) |
| Logs | `~/.scriptkit/logs/` |
| Bundled skills | `~/.scriptkit/plugins/scriptkit/skills/` |
| Examples (scripts) | `~/.scriptkit/plugins/examples/scripts/` |

## Tab AI — Quick Terminal with Flat Context Injection

Tab AI now has two distinct surfaces: Agent Chat is the default AI chat UI, while `AppView::QuickTerminalView` remains the PTY-backed harness surface rendered by `TermPrompt`.

**Entry path:**
- Command+Enter in `AppView::ScriptList` routes through the Agent Chat entry path and context-capture helpers. Do not describe plain Tab as opening Agent Chat or the harness terminal.
- `QuickTerminalView` is opened by explicit harness / verification flows that need a PTY-backed surface.
- `Tab` / `Shift+Tab` inside `AppView::QuickTerminalView` are forwarded to the PTY. Do not describe them as focus-navigation keys once the harness terminal is open.

**Close semantics:**
- `Cmd+W` closes the wrapper and restores the previous view and focus.
- Plain `Escape` is forwarded to the PTY. The harness TUI owns Escape behavior.
- The footer hint strip advertises only `⌘W Close`.

**Runtime contract:**
- Canonical chat entry: Command+Enter via `open_tab_ai_agent_chat_with_entry_intent(...)`
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
- Do not describe plain Tab as opening Agent Chat or the harness terminal, and do not describe `Shift+Tab` in `AppView::ScriptList` as the default quick-submit path.

**Do not describe as current behavior:**
- Do not describe the old inline chat entity or custom streaming UI as the primary Tab AI surface
- Do not describe the old inline chat or custom streaming UI as the default path
- Do not describe Claude Agent SDK V2 or screenshot attachment support as already landed in the default Tab flow

## Avoid These Mistakes

- Do not create more than one artifact for one request
- Do not put scripts in `scriptlets/` or `agents/`
- Do not put bundles in `scripts/`
- Do not put agents in `scripts/` or `scriptlets/`
- Do not put skills in `scripts/` or `scriptlets/` — skills are `SKILL.md` directories under `skills/`
- Do not put Agent Chat profiles in `agents/` — profiles are `profile.json` directories under `profiles/`
- Do not create new agents when a skill would work — agents are a compatibility path
- Do not use CommonJS or the old v1 SDK package
- Do not use Node.js `fs` / `child_process` — use Bun APIs
- Do not edit `sdk/` — managed by the app
- Do not use `export const metadata` in agent markdown files — use `_sk_*` keys
- Do not add `import "@scriptkit/sdk"` to agent markdown files
