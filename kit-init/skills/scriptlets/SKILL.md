# Skill: Scriptlets (Extension Bundles)

Create markdown-based extension bundles that group quick commands together.

## Where Scriptlets Live

```
~/.scriptkit/kit/main/extensions/*.md
```

Each `.md` file is an extension bundle. Every code block inside becomes a separate menu item.

## Creating an Extension Bundle

### Basic Structure

```markdown
---
name: My Tools
description: A collection of useful tools
---

# My Tools

## Command Name
<!-- name: Display Name -->
<!-- description: What this does -->

\`\`\`bash
echo "Hello from the shell!"
\`\`\`
```

### Scriptlet Types

#### Bash Commands

```markdown
## List Downloads
<!-- name: List Downloads -->

\`\`\`bash
ls -la ~/Downloads | head -20
\`\`\`
```

#### TypeScript Tools (with SDK)

```markdown
## Quick Note
<!-- name: Quick Note -->

\`\`\`tool:quick-note
import "@scriptkit/sdk";
const note = await arg("Enter note:");
await Bun.write(`${env.HOME}/notes/${Date.now()}.txt`, note);
await notify("Saved!");
\`\`\`
```

#### Text Templates

```markdown
## Email Template
<!-- name: Email Template -->

\`\`\`template:email
Hi {{name}},

Thanks for your message about {{topic}}.

Best regards
\`\`\`
```

### Metadata Comments

Place HTML comments before the code fence:

```markdown
<!-- name: Display Name -->
<!-- description: Shown when focused -->
<!-- shortcut: cmd shift x -->
<!-- trigger: !snippet -->
```

| Field | Purpose |
|-------|---------|
| `name` | Display name in Script Kit menu |
| `description` | Subtitle shown when focused |
| `shortcut` | Global hotkey (e.g., `ctrl shift h`) |
| `trigger` | Snippet trigger (e.g., `!hello` expands when typed) |

### Shared Actions

Create `*.actions.md` files to add actions to all scriptlets in a bundle:

```markdown
## Edit Source
<!-- name: Edit Source -->
<!-- shortcut: cmd e -->

\`\`\`bash
open -a "Visual Studio Code" "{{sourceFile}}"
\`\`\`
```

Name it `main.actions.md` alongside `main.md` to attach actions to that bundle.

## Complete Example

```markdown
---
name: Dev Shortcuts
description: Quick developer commands
---

# Dev Shortcuts

## Git Status
<!-- name: Git Status -->
<!-- description: Show current repo status -->
<!-- shortcut: ctrl g -->

\`\`\`bash
cd {{cwd}} && git status
\`\`\`

---

## Docker Cleanup
<!-- name: Docker Cleanup -->
<!-- description: Remove stopped containers and dangling images -->

\`\`\`bash
docker container prune -f && docker image prune -f
echo "Docker cleaned up!"
\`\`\`

---

## Quick Timer
<!-- name: Quick Timer -->
<!-- description: Set a quick countdown timer -->

\`\`\`tool:timer
import "@scriptkit/sdk";
const minutes = await arg("Minutes:", ["1", "5", "10", "15", "30"]);
const seconds = parseInt(minutes) * 60;
await $`sleep ${seconds} && osascript -e 'display notification "Timer done!" with title "Script Kit"'`;
await notify(`Timer set for ${minutes} minutes`);
\`\`\`

---

## Date Stamp
<!-- name: Date Stamp -->
<!-- trigger: !date -->

\`\`\`template:date
{{year}}-{{month}}-{{day}}
\`\`\`
```

## When to Use Scripts vs Scriptlets

| Use Case | Script (.ts) | Scriptlet (.md) |
|----------|-------------|-----------------|
| Complex logic | Yes | No |
| Quick shell commands | Possible but overkill | Yes |
| Text snippets/templates | No | Yes |
| API integrations | Yes | No |
| Grouped related commands | Separate files | One bundle file |
| Rich UI (editor, forms) | Yes | Via `tool:` fence |

## Common Mistakes

- **Wrong fence type**: Use `` ```bash `` for shell, `` ```tool:name `` for TypeScript
- **Missing SDK import**: `tool:` scriptlets still need `import "@scriptkit/sdk";`
- **Wrong directory**: Extensions go in `kit/main/extensions/`, not `extensions/`
- **Overloading bundles**: Keep bundles focused — one theme per `.md` file
