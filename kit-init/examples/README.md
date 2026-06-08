# Script Kit Example

This plugin intentionally ships one example script and a small set of scriptlet bundles.

| File | Pattern Demonstrated |
|------|---------------------|
| `scripts/todo-app.ts` | A local Todo app with projects, labels, priorities, due dates, Today/Upcoming views, CRUD, and `;todo` capture sync |
| `scriptlets/agent_chat-chat/main.md` | Agent Chat handoff examples |
| `scriptlets/custom-actions/main.md` | Custom action examples |
| `scriptlets/notes/main.md` | Notes create, update, organize, and automation payload examples |

Copy it into your workspace when you want to experiment:

```bash
cp ~/.scriptkit/plugins/examples/scripts/todo-app.ts ~/.scriptkit/plugins/main/scripts/my-todo-app.ts
```

For new scripts, read `~/.scriptkit/plugins/scriptkit/skills/new-script/SKILL.md` and verify the script before reporting success.
