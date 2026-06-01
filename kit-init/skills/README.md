# Script Kit Skills

Plugin-owned skills for working with Script Kit. Each subdirectory contains a `SKILL.md` with detailed guidance. Skills are the preferred reusable AI unit — they appear in the main menu and always open Agent Chat when selected.

## Available Skills

| Skill | Purpose |
|-------|---------|
| [new-script](new-script/SKILL.md) | Create or update TypeScript scripts |
| [new-scriptlet](new-scriptlet/SKILL.md) | Create or update markdown scriptlet bundles |
| [start-chat](start-chat/SKILL.md) | Programmatic Agent Chat flows, typed context parts, streaming, and chat lifecycle operations |
| [build-profile](build-profile/SKILL.md) | Create isolated Pi-backed Agent Chat profile artifacts |
| [add-actions](add-actions/SKILL.md) | Add Actions Menu commands in scripts and shared scriptlet companion `.actions.md` files |
| [new-agent](new-agent/SKILL.md) | Create agent files (compatibility — prefer skills for new work) |
| [update-config](update-config/SKILL.md) | Update configuration, theming, and workspace setup |
| [configure-mcp](configure-mcp/SKILL.md) | Configure external MCP servers shared by scripts and Agent Chat |
| [manage-notes](manage-notes/SKILL.md) | Manage the Notes window — creation, search, and automation |
| [troubleshoot](troubleshoot/SKILL.md) | Diagnose common issues and inspect logs |

## How to Use

Sample request → expected artifact:
- "make a clipboard cleanup command" → `~/.scriptkit/plugins/main/scripts/clipboard-cleanup.ts`
- "make a bundle of text snippets" → `~/.scriptkit/plugins/main/scriptlets/snippets.md`
- "make a skill for reviewing PRs" → `~/.scriptkit/plugins/main/skills/review-pr/SKILL.md`

Read the relevant `SKILL.md` before performing a task. Each skill contains:
- Step-by-step instructions
- Canonical file paths
- Working code examples
- Common pitfalls to avoid

## Plugin Architecture

Plugins are the package boundary in Script Kit. Each plugin under `~/.scriptkit/plugins/<plugin>/` owns:
- `scripts/` — TypeScript scripts (direct execution)
- `scriptlets/` — Markdown scriptlet bundles (direct execution)
- `skills/` — AI skills (open Agent Chat when selected)
- `profiles/` — Isolated Pi-backed Agent Chat profile artifacts
- `agents/` — Legacy agent files (compatibility only)
