# Script Kit Skills

Agent-readable skills for working with Script Kit. Each subdirectory contains a `SKILL.md` with detailed guidance.

## Available Skills

| Skill | Purpose |
|-------|---------|
| [script-authoring](script-authoring/SKILL.md) | Creating, structuring, and running TypeScript scripts |
| [scriptlets](scriptlets/SKILL.md) | Markdown extension bundles with embedded commands |
| [agents](agents/SKILL.md) | Creating mdflow-backed agent files |
| [config](config/SKILL.md) | Configuration, theming, and workspace setup |
| [troubleshooting](troubleshooting/SKILL.md) | Common issues, debugging, and log inspection |

## How to Use

Sample request → expected artifact:
- "make a clipboard cleanup command" → `~/.scriptkit/kit/main/scripts/clipboard-cleanup.ts`
- "make a bundle of text snippets" → `~/.scriptkit/kit/main/extensions/snippets.md`
- "make an agent that reviews staged changes" → `~/.scriptkit/kit/main/agents/review-pr.claude.md`

Read the relevant `SKILL.md` before performing a task. Each skill contains:
- Step-by-step instructions
- Canonical file paths
- Working code examples
- Common pitfalls to avoid
