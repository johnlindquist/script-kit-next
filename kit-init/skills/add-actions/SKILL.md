---
name: add-actions
description: Add Actions Menu commands to scripts and scriptlet bundles with setActions() and companion .actions.md files.
---

# Add Actions

Use this skill when the user wants Actions Menu commands (Cmd+K), script-local actions via `setActions()`, or shared scriptlet actions via `<bundle>.actions.md`.

## Write Here

For scripts:
`~/.scriptkit/plugins/main/scripts/<name>.ts`

For scriptlet bundles:
`~/.scriptkit/plugins/main/scriptlets/<bundle>.md`
`~/.scriptkit/plugins/main/scriptlets/<bundle>.actions.md`

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

## Scriptlet Companion File

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

- **Canonical**: `~/.scriptkit/plugins/examples/scriptlets/custom-actions/main.md` — parent bundle entries that receive shared actions
- **Canonical**: `~/.scriptkit/plugins/examples/scriptlets/custom-actions/main.actions.md` — companion Actions Menu definitions using `{{content}}`
- **Flat mirror**: `~/.scriptkit/plugins/examples/scriptlets/custom-actions.md`
- **Flat mirror**: `~/.scriptkit/plugins/examples/scriptlets/custom-actions.actions.md`

## Related Skills

- [new-scriptlet](../new-scriptlet/SKILL.md) — bundle and companion-file structure
- [start-chat](../start-chat/SKILL.md) — ACP-oriented actions
- [manage-notes](../manage-notes/SKILL.md) — note-focused actions and handoffs
