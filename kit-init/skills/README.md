# Script Kit Skills

Plugin-owned skills for working with Script Kit. Each subdirectory contains a `SKILL.md` with detailed guidance. Skills are the preferred reusable AI unit — they appear in the main menu and always open ACP Chat when selected.

## Available Skills

| Skill | Purpose |
|-------|---------|
| [script-authoring](script-authoring/SKILL.md) | Creating, structuring, and running TypeScript scripts |
| [scriptlets](scriptlets/SKILL.md) | Markdown extension bundles with embedded commands |
| [acp-chat](acp-chat/SKILL.md) | Programmatic ACP Chat flows, typed context parts, streaming, and chat lifecycle operations |
| [custom-actions](custom-actions/SKILL.md) | Actions Menu commands in scripts and shared extension companion `.actions.md` files |
| [agents](agents/SKILL.md) | Creating agent files (compatibility — prefer skills for new work) |
| [config](config/SKILL.md) | Configuration, theming, and workspace setup |
| [notes](notes/SKILL.md) | Working with the Notes window — creation, search, and automation |
| [troubleshooting](troubleshooting/SKILL.md) | Common issues, debugging, and log inspection |

## How to Use

Sample request → expected artifact:
- "make a clipboard cleanup command" → `~/.scriptkit/kit/main/scripts/clipboard-cleanup.ts`
- "make a bundle of text snippets" → `~/.scriptkit/kit/main/extensions/snippets.md`
- "make a skill for reviewing PRs" → `~/.scriptkit/kit/main/skills/review-pr/SKILL.md`

Read the relevant `SKILL.md` before performing a task. Each skill contains:
- Step-by-step instructions
- Canonical file paths
- Working code examples
- Common pitfalls to avoid

## Plugin Architecture

Plugins are the package boundary in Script Kit. Each plugin under `~/.scriptkit/kit/<plugin>/` owns:
- `scripts/` — TypeScript scripts (direct execution)
- `extensions/` — Markdown scriptlet bundles (direct execution)
- `skills/` — AI skills (open ACP Chat when selected)
- `agents/` — Legacy agent files (compatibility only)
