# Script Kit One-Shot Starters

> Canonical one-shot authoring guide for harness mode.
> `ROOT_CLAUDE.md`, `ROOT_AGENTS.md`, and harness artifact guidance should route here instead of duplicating starter content.

Use this file when the fastest harness answer is:
1. pick exactly one artifact
2. copy exactly one starter
3. save it under `kit/main/`
4. stop at the smallest working version

## Choose Exactly One Artifact

### Script

Use a script when the request needs Script Kit UI, Bun APIs, file work, HTTP work, or multi-step logic.

Copy from: `scripts/hello-world.ts`
Write to: `~/.scriptkit/kit/main/scripts/<name>.ts`

Then read `~/.scriptkit/kit/authoring/skills/script-authoring/SKILL.md`. For scripts, writing the file is not enough. You must syntax-check and run the script in the current Claude Code terminal before you report success.

Good matches:
- `make a clipboard cleanup command`
- `make a GitHub helper`
- `make a file rename workflow`

### Extension bundle / scriptlet bundle

Use a bundle when the request is a snippet, text expansion, quick shell command, or a small grouped helper set.

Copy from: `extensions/starter.md`
Write to: `~/.scriptkit/kit/main/extensions/<name>.md`

Good matches:
- `make a bundle of text snippets`
- `make an email sign-off snippet`
- `make a few quick shell helpers`

### mdflow agent

Use an agent when the request is a reusable reviewer, planner, backend-specific prompt, or model-backed automation.

Copy from: `agents/review-pr.claude.md`
Write to: `~/.scriptkit/kit/main/agents/<name>.<backend>.md`

Good matches:
- `make an agent that reviews staged changes`
- `make a feature planning agent`
- `make a Codex review agent`

Script Kit uses **extension bundle** and **scriptlet bundle** to mean the same artifact.

## When the request says "command", "helper", or "tool"

Pick **Script** if it needs UI, Bun, files, HTTP, or multiple steps.
Pick **Extension bundle / scriptlet bundle** if it is a snippet, text expansion, quick shell command, or a small grouped helper set.
Pick **mdflow agent** if it should run through a model backend.

## Agent Backend Quick Pick

- Claude → `<name>.claude.md`
- Gemini → `<name>.gemini.md`
- Codex → `<name>.codex.md`
- Copilot → `<name>.copilot.md`
- Interactive Gemini → `<name>.i.gemini.md`
- Generic custom command → `generic.md` with `_command`

## Fast Picks

- `make a clipboard cleanup command` → `~/.scriptkit/kit/main/scripts/clipboard-cleanup.ts`
- `make a bundle of text snippets` → `~/.scriptkit/kit/main/extensions/snippets.md`
- `make an agent that reviews staged changes in Claude` → `~/.scriptkit/kit/main/agents/review-pr.claude.md`

## Mandatory Script Verification

For every script created from the harness:

```bash
bun build ~/.scriptkit/kit/main/scripts/<name>.ts --target=bun --outfile ~/.scriptkit/tmp/test-scripts/<name>.verify.mjs
```

```bash
SK_VERIFY=1 bun ~/.scriptkit/kit/main/scripts/<name>.ts
```

If the script normally needs UI or typed input, add an `SK_VERIFY=1` branch first so the Bun execution step is non-interactive. If either command fails, fix the script and rerun both commands. Do not report success until both commands pass and the observed output matches the request.

## Prompt Sequencing Rule

Script Kit prompt APIs are interactive UI surfaces. Do not call them concurrently.

Never use `Promise.all`, `Promise.race`, `Promise.any`, or `Promise.allSettled` with `arg`, `fields`, `editor`, `div`, `form`, `drop`, `find`, `path`, `textarea`, `select`, or `grid`.

Wrong:

```typescript
const [url1, url2, url3] = await Promise.all([
  arg("URL 1"),
  arg("URL 2"),
  arg("URL 3"),
]);
```

Right:

```typescript
const url1 = await arg("URL 1");
const url2 = await arg("URL 2");
const url3 = await arg("URL 3");
```

## Copy Commands

```bash
cp ~/.scriptkit/kit/examples/scripts/hello-world.ts ~/.scriptkit/kit/main/scripts/my-script.ts
cp ~/.scriptkit/kit/examples/extensions/starter.md ~/.scriptkit/kit/main/extensions/my-bundle.md
cp ~/.scriptkit/kit/examples/agents/review-pr.claude.md ~/.scriptkit/kit/main/agents/my-agent.claude.md
```

## Smallest Working Starters

### Script → `~/.scriptkit/kit/main/scripts/<name>.ts`

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "Hello World",
  description: "Verification-friendly starter script",
};

const isVerify = process.env.SK_VERIFY === "1";

const name = isVerify
  ? "verification"
  : await arg("Who should I greet?");

const greeting = `Hello, ${name}!`;

if (isVerify) {
  console.log(JSON.stringify({ ok: true, greeting }));
} else {
  await div(`<div class="p-8 text-2xl">${greeting}</div>`);
}
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
await Bun.write(home("quick-note.txt"), note);
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
- Prefer `home(...)` for user-relative paths instead of `env.HOME`.
- For extension bundles / scriptlet bundles, prefer `metadata` code fences.
- For `tool:<name>` scriptlets, the first line must be `import "@scriptkit/sdk";`.
- For agents, use underscore-prefixed `_sk_*` metadata keys.
