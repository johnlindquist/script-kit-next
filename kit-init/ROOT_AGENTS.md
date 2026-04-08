# Script Kit SDK Reference

Complete reference for AI agents creating Script Kit artifacts: scripts, extension bundles, skills, and mdflow agents. Plugins are the package boundary; skills are the preferred reusable AI unit.

> **Package**: `@scriptkit/sdk` — **Runtime**: Bun — **Write under**: `~/.scriptkit/kit/main/{scripts,extensions,skills,agents}`

## One-Shot First

Use `~/.scriptkit/kit/examples/START_HERE.md` as the canonical one-shot authoring guide.
Open it first when the user wants one new Script Kit artifact in harness mode.
Use this file only after the artifact type is already chosen and you need deeper SDK reference.

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

## Artifact-Specific Rules

### Script
- First line: `import "@scriptkit/sdk";`
- Use `export const metadata = { name, description }`
- Use Bun APIs instead of Node-only APIs

### Extension bundle / scriptlet bundle
- Save one markdown file under `~/.scriptkit/kit/main/extensions/`
- Prefer `metadata` code fences for new bundles
- `tool:<name>` fences must begin with `import "@scriptkit/sdk";`
- Do not add `export const metadata` at the top of the markdown file

### Skill (preferred reusable AI unit)
- Create a directory under `~/.scriptkit/kit/main/skills/<name>/`
- Add a `SKILL.md` file with YAML frontmatter (`name`, `description`)
- Skills appear in the main menu and always open ACP Chat when selected
- Plugins are the package boundary — each plugin owns its own skills
- Prefer skills over agents for any new reusable AI work

### mdflow agent (compatibility)
- Save to `~/.scriptkit/kit/main/agents/<name>.<backend>.md`
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

## Notifications & Feedback

```typescript
await notify("Task complete!");          // System notification
await copy("Copied to clipboard");       // Copy + implicit feedback
await open("https://example.com");       // Open URL/file in default app
```

## Extensions (Scriptlet Bundles)

For first-pass authoring, copy the minimal starter above. The rest of this section is detailed reference.

Markdown files at `~/.scriptkit/kit/main/extensions/*.md`:

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
await notify("Saved");
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

`~/.scriptkit/kit/config.ts`:

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

`~/.scriptkit/kit/theme.json`:

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
| Scripts | `~/.scriptkit/kit/main/scripts/*.ts` |
| Extensions | `~/.scriptkit/kit/main/extensions/*.md` |
| Skills (preferred AI unit) | `~/.scriptkit/kit/main/skills/<name>/SKILL.md` |
| Agents (compatibility) | `~/.scriptkit/kit/main/agents/*.md` |
| Config | `~/.scriptkit/kit/config.ts` |
| Theme | `~/.scriptkit/kit/theme.json` |
| SDK | `~/.scriptkit/sdk/kit-sdk.ts` (do not edit) |
| Logs | `~/.scriptkit/logs/` |
| Authoring skills | `~/.scriptkit/kit/authoring/skills/` |
| Examples (scripts) | `~/.scriptkit/kit/examples/scripts/` |
| Examples (extensions) | `~/.scriptkit/kit/examples/extensions/` |
| Examples (agents) | `~/.scriptkit/kit/examples/agents/` |

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

**Harness lifecycle:**
- Each explicit quick-terminal open writes `~/.scriptkit/context/latest.md`, enumerates `~/.scriptkit/kit/authoring/skills/`, and behaves as a one-shot spawn rendered in `QuickTerminalView`.
- Internal silent prewarm may seed the PTY ahead of time, but that is a single-use implementation detail rather than a documented warm multi-turn surface.
- Recovery — if the harness crashes or exits, the next Tab entry respawns it.

**Do not describe as current behavior:**
- Do not describe the old inline chat entity or custom streaming UI as the primary Tab AI surface
- Do not describe the old inline chat or custom streaming UI as the default path
- Do not describe Claude Agent SDK V2 or screenshot attachment support as already landed in the default Tab flow

## Avoid These Mistakes

- Do not create more than one artifact for one request
- Do not put scripts in `extensions/` or `agents/`
- Do not put bundles in `scripts/`
- Do not put agents in `scripts/` or `extensions/`
- Do not put skills in `scripts/` or `extensions/` — skills are `SKILL.md` directories under `skills/`
- Do not create new agents when a skill would work — agents are a compatibility path
- Do not use CommonJS or the old v1 SDK package
- Do not use Node.js `fs` / `child_process` — use Bun APIs
- Do not edit `sdk/` — managed by the app
- Do not use `export const metadata` in agent markdown files — use `_sk_*` keys
- Do not add `import "@scriptkit/sdk"` to agent markdown files
