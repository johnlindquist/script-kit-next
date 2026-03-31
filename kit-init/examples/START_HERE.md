# Script Kit One-Shot Starters

Use this file when the fastest harness answer is:
pick one artifact, copy one starter, save it under `kit/main/`, then stop.

## Pick the Artifact

| Request shape | Artifact | Copy from | Write to |
|---------------|----------|-----------|----------|
| "make a clipboard cleanup command" | Script | `scripts/hello-world.ts` | `~/.scriptkit/kit/main/scripts/clipboard-cleanup.ts` |
| "make a bundle of text snippets" | Extension bundle / scriptlet bundle | `extensions/starter.md` | `~/.scriptkit/kit/main/extensions/snippets.md` |
| "make an agent that reviews staged changes" | mdflow agent | `agents/review-pr.claude.md` | `~/.scriptkit/kit/main/agents/review-pr.claude.md` |

Script Kit uses **extension bundle** and **scriptlet bundle** to mean the same artifact: one markdown file under `~/.scriptkit/kit/main/extensions/`.

## When the request says "command", "helper", or "tool"

Use **Script** when the request needs:
- Script Kit UI (`arg`, `div`, `editor`, `fields`, `path`)
- Bun APIs
- file or HTTP work
- multi-step logic

Use **Extension bundle / scriptlet bundle** when the request is:
- a snippet
- a text expansion
- a quick shell command
- a small grouped helper set

Use **mdflow agent** when the request is:
- a reusable reviewer
- a planner
- a backend-specific chat prompt
- an automation that should run through a model backend

## Agent Backend Quick Pick

| Backend wanted | Filename |
|----------------|----------|
| Claude | `<name>.claude.md` |
| Gemini | `<name>.gemini.md` |
| Codex | `<name>.codex.md` |
| Copilot | `<name>.copilot.md` |
| Interactive Gemini | `<name>.i.gemini.md` |
| Generic custom command | `generic.md` with `_command` |

## Fast Picks

- `"make a clipboard cleanup command"` → `~/.scriptkit/kit/main/scripts/clipboard-cleanup.ts`
- `"make a bundle of text snippets"` → `~/.scriptkit/kit/main/extensions/snippets.md`
- `"make an agent that reviews staged changes in Claude"` → `~/.scriptkit/kit/main/agents/review-pr.claude.md`

## Copy Commands

```bash
cp ~/.scriptkit/examples/scripts/hello-world.ts ~/.scriptkit/kit/main/scripts/my-script.ts
cp ~/.scriptkit/examples/extensions/starter.md ~/.scriptkit/kit/main/extensions/my-bundle.md
cp ~/.scriptkit/examples/agents/review-pr.claude.md ~/.scriptkit/kit/main/agents/my-agent.claude.md
```

## Smallest Working Starters

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

### Extension bundle / scriptlet bundle → `~/.scriptkit/kit/main/extensions/<name>.md`

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

## Rules

- Pick the smallest artifact that fits.
- Save only under `~/.scriptkit/kit/main/`.
- For scripts, start with `import "@scriptkit/sdk";`.
- For extension bundles / scriptlet bundles, prefer `metadata` code fences.
- For `tool:<name>` scriptlets, the first line must be `import "@scriptkit/sdk";`.
- For agents, use underscore-prefixed `_sk_*` metadata keys.
