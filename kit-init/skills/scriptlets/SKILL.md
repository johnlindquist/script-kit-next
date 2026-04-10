---
name: scriptlets
description: Create scriptlet bundles for text expansions, snippets, shell commands, and lightweight helpers in a single markdown file. Use when the user wants quick shortcuts or grouped utilities.
---

# Scriptlets

Use a scriptlet bundle when the user wants text expansions, snippets, quick shell commands, or several lightweight helpers grouped in one markdown file.

## Write Here

`~/.scriptkit/kit/main/scriptlets/<name>.md`

Do not create new user bundles in app-managed or example plugins.

## Read These Files In Order

1. `~/.scriptkit/kit/examples/scriptlets/howto.md`
2. `~/.scriptkit/kit/examples/scriptlets/main.md`
3. `~/.scriptkit/kit/examples/scriptlets/advanced.md`

## Canonical Bundle Shape

```markdown
---
name: My Bundle
description: Personal helpers
icon: sparkles
---

## Email Sign-off

```metadata
keyword: !bye
description: Quick email sign-off
```

```paste
Thanks,
Your Name
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
```

## Fence Map

| Fence | Use for |
|------|---------|
| `paste` | Static text expansion |
| `bash` | Shell command |
| `tool:<name>` | TypeScript with the Script Kit SDK |
| `template:<name>` | Template expansion |
| `open` | URL or file target |

`tool:<name>` fences still need `import "@scriptkit/sdk";` as the first line.

## Metadata

Prefer `metadata` code fences for:
- `keyword`
- `description`
- `shortcut`
- `alias`
- `schedule`
- `cron`
- `icon`
- boolean flags

Legacy HTML comments still work, but do not generate them for new harness-authored bundles unless the user explicitly asks for legacy format.

## Choose Script vs Scriptlet Bundle

Choose a `.ts` script when the request needs:
- rich UI
- multi-step logic
- file/network workflows
- external APIs

Choose a scriptlet bundle when the request is:
- a snippet
- a text expansion
- a quick shell command
- a small grouped helper set

## Companion Actions Files

To add shared actions to every command in a bundle, create a matching companion file:

- Parent bundle: `~/.scriptkit/kit/main/scriptlets/<name>.md`
- Shared actions: `~/.scriptkit/kit/main/scriptlets/<name>.actions.md`

Use `{{content}}` inside the companion action to access the selected parent command content.

See [custom-actions](../custom-actions/SKILL.md) for the canonical pattern.

## Focused Feature Examples

Generic examples are flat files under `~/.scriptkit/kit/examples/scriptlets/`. Focused feature examples are **nested bundles**:

- `~/.scriptkit/kit/examples/scriptlets/acp-chat/main.md` — ACP-oriented scriptlet helpers
- `~/.scriptkit/kit/examples/scriptlets/custom-actions/main.md` — shared Actions Menu patterns
- `~/.scriptkit/kit/examples/scriptlets/custom-actions/main.actions.md` — companion actions file
- `~/.scriptkit/kit/examples/scriptlets/notes/main.md` — Notes automation as a scriptlet bundle

Flat mirrors (`~/.scriptkit/kit/examples/scriptlets/acp-chat.md`, `custom-actions.md`, `custom-actions.actions.md`, `notes.md`) are generated from the nested bundles above.

## Related Skills

- [custom-actions](../custom-actions/SKILL.md) — shared Actions Menu patterns for scriptlet bundles
- [acp-chat](../acp-chat/SKILL.md) — ACP-oriented scriptlet helpers
- [notes](../notes/SKILL.md) — package Notes automation examples as scriptlet bundles

## Done When

- the file lives in `~/.scriptkit/kit/main/scriptlets/`
- each `##` heading is one scriptlet
- the fence type matches the intended behavior
- the bundle is the smallest artifact that fits
