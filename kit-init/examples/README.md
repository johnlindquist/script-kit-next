# Script Kit Examples

`~/.scriptkit/plugins/examples/START_HERE.md` is the canonical one-shot creation guide.
Use this README for browsing and pattern study after the artifact type is already chosen.

Working examples demonstrating Script Kit patterns. Plugins are the package boundary. Learn by reading, then create your own in the matching workspace directory:

- scripts → `~/.scriptkit/plugins/main/scripts/`
- scriptlet bundles → `~/.scriptkit/plugins/main/scriptlets/`
- skills (preferred reusable AI unit) → `~/.scriptkit/plugins/main/skills/<name>/SKILL.md`
- mdflow agents (compatibility) → `~/.scriptkit/plugins/main/agents/`

## Start Here

| Goal | Copy from | Write to | Read next |
|------|-----------|----------|-----------|
| New script | `scripts/hello-world.ts` | `~/.scriptkit/plugins/main/scripts/<name>.ts` | `~/.scriptkit/plugins/scriptkit/skills/new-script/SKILL.md` |
| New scriptlet bundle | `scriptlets/starter.md` | `~/.scriptkit/plugins/main/scriptlets/<name>.md` | `~/.scriptkit/plugins/scriptkit/skills/new-scriptlet/SKILL.md` |
| New skill (preferred AI unit) | `skills/review-pr/` | `~/.scriptkit/plugins/main/skills/<name>/SKILL.md` | `~/.scriptkit/plugins/scriptkit/skills/README.md` |
| New mdflow agent (compatibility) | `agents/review-pr.claude.md` | `~/.scriptkit/plugins/main/agents/<name>.<backend>.md` | `~/.scriptkit/plugins/scriptkit/skills/new-agent/SKILL.md` |

Pick one artifact, copy one starter, save it under `plugins/main/`, then stop. Do not create multiple artifact types for one request.

## Scripts (`scripts/`)

| File | Pattern Demonstrated |
|------|---------------------|
| `hello-world.ts` | Basic prompt (`arg`) and HTML display (`div`) |
| `choose-from-list.ts` | Rich choices with descriptions and preview panels |
| `clipboard-transform.ts` | Clipboard read/transform/write workflow |
| `path-picker.ts` | File picker and Bun file operations |
| `todoist-demo.ts` | Todoist-style task manager: projects, labels, priorities, views, CRUD, `;todo` sync |
| `github-device-login.ts` | GitHub OAuth device-code sign-in and token persistence |
| `microsoft-graph-device-login.ts` | Microsoft Graph device-code sign-in with calendar scopes |
| `google-calendar-device-login.ts` | Google OAuth device-code sign-in with Calendar event scope |
| `generic-oauth-device-flow.ts` | RFC 8628 template for providers with device authorization |
| `lib/oauth-device-flow.ts` | Shared helper for polling, slow_down handling, and token files |

When copying any device-login example into `plugins/main/scripts/`, copy `lib/oauth-device-flow.ts` with it or inline the helper.

## Scriptlets (`scriptlets/`)

Reference markdown bundles copied from the built-in examples kit.

| File | Pattern Demonstrated |
|------|---------------------|
| `starter.md` | Smallest copyable bundle for one-shot creation |
| `howto.md` | Best first read for bundle rules and metadata |
| `main.md` | Core scriptlet patterns |
| `advanced.md` | Richer bundle patterns and edge cases |

Copy patterns from these files into `~/.scriptkit/plugins/main/scriptlets/`.

### Focused Scriptlets

Nested bundles demonstrating feature-specific patterns. Each lives in its own subdirectory.

| Directory | Pattern Demonstrated | Skill |
|-----------|---------------------|-------|
| `scriptlets/acp-chat/main.md` | ACP-oriented scriptlet helpers | [start-chat](~/.scriptkit/plugins/scriptkit/skills/start-chat/SKILL.md) |
| `scriptlets/custom-actions/main.md` | Shared Actions Menu patterns with companion `.actions.md` | [add-actions](~/.scriptkit/plugins/scriptkit/skills/add-actions/SKILL.md) |
| `scriptlets/notes/main.md` | Notes automation as a scriptlet bundle | [manage-notes](~/.scriptkit/plugins/scriptkit/skills/manage-notes/SKILL.md) |

## Skills (`skills/`)

Reference skills that can be copied into `~/.scriptkit/plugins/main/skills/` and selected from the main menu.

| Directory | Pattern Demonstrated |
|-----------|---------------------|
| `skills/review-pr/` | Findings-first code review skill for diffs and checked-out PRs |
| `skills/plan-feature/` | Implementation planning skill with scope, risks, and verification |
| `skills/explain-code/` | Code-orientation skill that traces flow and contracts across files |

## Agents (Compatibility) (`agents/`)

Reference mdflow agent files. For new reusable AI work, prefer creating a skill (`~/.scriptkit/plugins/main/skills/<name>/SKILL.md`) instead — skills are the preferred reusable AI unit and appear as first-class main-menu items.

| File | Pattern Demonstrated |
|------|---------------------|
| `review-pr.claude.md` | Minimal single-turn Claude agent |
| `plan-feature.i.gemini.md` | Interactive Gemini agent with `_inputs` |

Copy patterns from these files into `~/.scriptkit/plugins/main/agents/`.

## Creating Your Own

Copy any example to your workspace:

```bash
cp ~/.scriptkit/plugins/examples/scripts/hello-world.ts ~/.scriptkit/plugins/main/scripts/my-script.ts
cp ~/.scriptkit/plugins/examples/scriptlets/starter.md ~/.scriptkit/plugins/main/scriptlets/my-bundle.md
mkdir -p ~/.scriptkit/plugins/main/skills && cp -R ~/.scriptkit/plugins/examples/skills/review-pr ~/.scriptkit/plugins/main/skills/review-pr  # skills are the preferred reusable AI unit
cp ~/.scriptkit/plugins/examples/agents/review-pr.claude.md ~/.scriptkit/plugins/main/agents/my-agent.claude.md  # compatibility
```

Then edit to suit your needs. Script Kit will detect it immediately.
