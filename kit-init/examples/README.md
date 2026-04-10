# Script Kit Examples

`~/.scriptkit/kit/examples/START_HERE.md` is the canonical one-shot authoring guide.
Use this README for browsing and pattern study after the artifact type is already chosen.

Working examples demonstrating Script Kit patterns. Plugins are the package boundary. Learn by reading, then create your own in the matching workspace directory:

- scripts → `~/.scriptkit/kit/main/scripts/`
- scriptlet bundles → `~/.scriptkit/kit/main/scriptlets/`
- skills (preferred reusable AI unit) → `~/.scriptkit/kit/main/skills/<name>/SKILL.md`
- mdflow agents (compatibility) → `~/.scriptkit/kit/main/agents/`

## Start Here

| Goal | Copy from | Write to | Read next |
|------|-----------|----------|-----------|
| New script | `scripts/hello-world.ts` | `~/.scriptkit/kit/main/scripts/<name>.ts` | `~/.scriptkit/kit/authoring/skills/script-authoring/SKILL.md` |
| New scriptlet bundle | `scriptlets/starter.md` | `~/.scriptkit/kit/main/scriptlets/<name>.md` | `~/.scriptkit/kit/authoring/skills/scriptlets/SKILL.md` |
| New skill (preferred AI unit) | `skills/review-pr/` | `~/.scriptkit/kit/main/skills/<name>/SKILL.md` | `~/.scriptkit/kit/authoring/skills/README.md` |
| New mdflow agent (compatibility) | `agents/review-pr.claude.md` | `~/.scriptkit/kit/main/agents/<name>.<backend>.md` | `~/.scriptkit/kit/authoring/skills/agents/SKILL.md` |

Pick one artifact, copy one starter, save it under `kit/main/`, then stop. Do not create multiple artifact types for one request.

## Scripts (`scripts/`)

| File | Pattern Demonstrated |
|------|---------------------|
| `hello-world.ts` | Basic prompt (`arg`) and HTML display (`div`) |
| `choose-from-list.ts` | Rich choices with descriptions and preview panels |
| `clipboard-transform.ts` | Clipboard read/transform/write workflow |
| `path-picker.ts` | File picker and Bun file operations |

## Scriptlets (`scriptlets/`)

Reference markdown bundles copied from the built-in examples kit.

| File | Pattern Demonstrated |
|------|---------------------|
| `starter.md` | Smallest copyable bundle for one-shot authoring |
| `howto.md` | Best first read for bundle rules and metadata |
| `main.md` | Core scriptlet patterns |
| `advanced.md` | Richer bundle patterns and edge cases |

Copy patterns from these files into `~/.scriptkit/kit/main/scriptlets/`.

### Focused Scriptlets

Nested bundles demonstrating feature-specific patterns. Each lives in its own subdirectory.

| Directory | Pattern Demonstrated | Skill |
|-----------|---------------------|-------|
| `scriptlets/acp-chat/main.md` | ACP-oriented scriptlet helpers | [acp-chat](~/.scriptkit/kit/authoring/skills/acp-chat/SKILL.md) |
| `scriptlets/custom-actions/main.md` | Shared Actions Menu patterns with companion `.actions.md` | [custom-actions](~/.scriptkit/kit/authoring/skills/custom-actions/SKILL.md) |
| `scriptlets/notes/main.md` | Notes automation as a scriptlet bundle | [notes](~/.scriptkit/kit/authoring/skills/notes/SKILL.md) |

## Skills (`skills/`)

Reference skills that can be copied into `~/.scriptkit/kit/main/skills/` and selected from the main menu.

| Directory | Pattern Demonstrated |
|-----------|---------------------|
| `skills/review-pr/` | Findings-first code review skill for diffs and checked-out PRs |
| `skills/plan-feature/` | Implementation planning skill with scope, risks, and verification |
| `skills/explain-code/` | Code-orientation skill that traces flow and contracts across files |

## Agents (Compatibility) (`agents/`)

Reference mdflow agent files. For new reusable AI work, prefer creating a skill (`~/.scriptkit/kit/main/skills/<name>/SKILL.md`) instead — skills are the preferred reusable AI unit and appear as first-class main-menu items.

| File | Pattern Demonstrated |
|------|---------------------|
| `review-pr.claude.md` | Minimal single-turn Claude agent |
| `plan-feature.i.gemini.md` | Interactive Gemini agent with `_inputs` |

Copy patterns from these files into `~/.scriptkit/kit/main/agents/`.

## Creating Your Own

Copy any example to your workspace:

```bash
cp ~/.scriptkit/kit/examples/scripts/hello-world.ts ~/.scriptkit/kit/main/scripts/my-script.ts
cp ~/.scriptkit/kit/examples/scriptlets/starter.md ~/.scriptkit/kit/main/scriptlets/my-bundle.md
mkdir -p ~/.scriptkit/kit/main/skills && cp -R ~/.scriptkit/kit/examples/skills/review-pr ~/.scriptkit/kit/main/skills/review-pr  # skills are the preferred reusable AI unit
cp ~/.scriptkit/kit/examples/agents/review-pr.claude.md ~/.scriptkit/kit/main/agents/my-agent.claude.md  # compatibility
```

Then edit to suit your needs. Script Kit will detect it immediately.
