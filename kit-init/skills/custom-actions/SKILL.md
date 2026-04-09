---
name: custom-actions
description: Add Actions Menu commands to scripts and extension bundles with setActions() and companion .actions.md files.
---

# Custom Actions

Use this skill when the user wants Actions Menu commands (Cmd+K), script-local actions via `setActions()`, or shared extension actions via `<bundle>.actions.md`.

## Write Here

For scripts:
`~/.scriptkit/kit/main/scripts/<name>.ts`

For extension bundles:
`~/.scriptkit/kit/main/extensions/<bundle>.md`
`~/.scriptkit/kit/main/extensions/<bundle>.actions.md`

## Script Example

```typescript
import "@scriptkit/sdk";

await setActions([
  {
    name: "Copy Input",
    shortcut: "cmd+c",
    onAction: async (input) => {
      await copy(input);
      await hud("Copied");
    },
  },
  {
    name: "Clear Input",
    shortcut: "cmd+backspace",
    onAction: async () => {
      await setInput("");
      await hud("Cleared");
    },
  },
]);

await arg("Type something");
```

## Extension Companion File

Parent bundle:

```markdown
## Script Kit

~~~metadata
description: Open the Script Kit homepage
~~~

~~~open
https://www.scriptkit.com
~~~
```

Companion actions file:

```markdown
### Copy URL

~~~bash
echo -n "{{content}}" | pbcopy
~~~

### Open in Safari

~~~bash
open -a Safari "{{content}}"
~~~
```

## Common Pitfalls

- Discovery belongs in the Actions Menu. Do not add persistent chrome to solve discoverability.
- The companion file must share the same basename: `main.md` + `main.actions.md`.
- Use `{{content}}` inside companion actions to read the selected parent command content.

## Related Examples

- `~/.scriptkit/kit/examples/extensions/custom-actions.md` — parent bundle entries that receive shared actions
- `~/.scriptkit/kit/examples/extensions/custom-actions.actions.md` — companion Actions Menu definitions using `{{content}}`

## Related Skills

- [scriptlets](../scriptlets/SKILL.md) — bundle and companion-file structure
- [acp-chat](../acp-chat/SKILL.md) — ACP-oriented actions
- [notes](../notes/SKILL.md) — note-focused actions and handoffs
