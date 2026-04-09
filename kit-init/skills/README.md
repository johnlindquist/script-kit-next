# Script Kit Skills

Plugin-owned skills for working with Script Kit. Plugins are the package boundary; skills are the preferred reusable AI unit. Each subdirectory contains a `SKILL.md` with detailed guidance.

## Available Skills

| Skill | Purpose |
|-------|---------|
| [script-authoring](script-authoring/SKILL.md) | Creating, structuring, and running TypeScript scripts |
| [scriptlets](scriptlets/SKILL.md) | Markdown extension bundles with embedded commands |
| [agents](agents/SKILL.md) | mdflow-backed agent files (compatibility — prefer skills for new AI work) |
| [config](config/SKILL.md) | Configuration, theming, and workspace setup |
| [troubleshooting](troubleshooting/SKILL.md) | Common issues, debugging, and log inspection |

## How to Use

Sample request → expected artifact:
- "make a clipboard cleanup command" → `~/.scriptkit/kit/main/scripts/clipboard-cleanup.ts`
- "make a bundle of text snippets" → `~/.scriptkit/kit/main/extensions/snippets.md`
- "make a skill for reviewing PRs" → `~/.scriptkit/kit/main/skills/review-pr/SKILL.md`
- "make an agent for a specific CLI backend" → `~/.scriptkit/kit/main/agents/review-pr.claude.md` (compatibility)

Read the relevant `SKILL.md` before performing a task. Each skill contains:
- Step-by-step instructions
- Canonical file paths
- Working code examples
- Common pitfalls to avoid
