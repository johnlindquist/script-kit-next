---
name: agents
description: Create mdflow-backed agent files for Script Kit. Compatibility path — prefer creating skills (SKILL.md under a plugin's skills/ directory) for new reusable AI work.
---

# Agents (Compatibility)

Create mdflow-backed agent files for Script Kit. For new reusable AI work, prefer creating a skill (`skills/<name>/SKILL.md`) under the appropriate plugin instead — skills are first-class main-menu launchables that open ACP Chat.

## Where Agents Live

`~/.scriptkit/kit/main/agents/*.md`

## Filename Rules

| Filename | Meaning |
|----------|---------|
| `review.claude.md` | Claude backend |
| `plan.gemini.md` | Gemini backend |
| `code.codex.md` | Codex backend |
| `triage.copilot.md` | Copilot backend |
| `chat.i.gemini.md` | Interactive agent (`.i.` or `_interactive: true`) |
| `generic.md` | Generic agent; use `_command` when needed |

## Script Kit Metadata Keys

Use underscore-prefixed keys so Script Kit metadata does not leak to backend CLI flags.

- `_sk_name`
- `_sk_description`
- `_sk_icon`
- `_sk_alias`
- `_sk_shortcut`

## mdflow System Keys

- `_inputs`
- `_interactive` or `_i`
- `_cwd`
- `_env`
- `_command`

## Minimal Template

```markdown
---
_sk_name: "Review PR"
_sk_description: "Review staged changes and call out risks"
_sk_icon: "git-pull-request"
model: sonnet
---

Review the current git diff.

Return:
1. findings ordered by severity
2. concrete fixes
3. tests to add
```

## Interactive Template

```markdown
---
_sk_name: "Plan Feature"
_sk_description: "Turn a feature request into an implementation plan"
_sk_icon: "map"
_interactive: true
_inputs:
  feature_name:
    type: text
    message: "Feature name?"
  risk_tolerance:
    type: select
    message: "Risk tolerance?"
    choices: ["low", "medium", "high"]
model: gemini-2.0-flash
---

Create an implementation plan for {{ feature_name }}.

Risk tolerance: {{ risk_tolerance }}.

Include:
- files to change
- data flow
- tests to add
- rollback plan
```

## Common Mistakes

- Do not put agent files in `scripts/`
- Do not use `export const metadata`
- Do not omit the backend suffix when you want a specific CLI
- Do not use non-underscore `sk_*` keys
