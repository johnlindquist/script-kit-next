# Skill: Scriptlets (Extension Bundles)

Use an extension bundle when the user wants text expansions, snippets, quick shell commands, or several lightweight helpers grouped in one markdown file.

## Write Here

`~/.scriptkit/kit/main/extensions/<name>.md`

Do not create new user bundles in built-in kits or example kits.

## Read These Files In Order

1. `~/.scriptkit/examples/extensions/howto.md`
2. `~/.scriptkit/examples/extensions/main.md`
3. `~/.scriptkit/examples/extensions/advanced.md`

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

## Choose Script vs Extension Bundle

Choose a `.ts` script when the request needs:
- rich UI
- multi-step logic
- file/network workflows
- external APIs

Choose an extension bundle when the request is:
- a snippet
- a text expansion
- a quick shell command
- a small grouped helper set

## Done When

- the file lives in `~/.scriptkit/kit/main/extensions/`
- each `##` heading is one scriptlet
- the fence type matches the intended behavior
- the bundle is the smallest artifact that fits
