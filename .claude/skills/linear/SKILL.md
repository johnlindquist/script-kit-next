---
name: linear
description: "Project-local Linear task tracking for Script Kit GPUI. Use when the user asks to read, triage, create, update, move, break down, prioritize, comment on, or track Linear issues/projects/cycles for this repo, especially the Script Kit project, JOH-10 through JOH-51, feature-map work, parent/subtask planning, project migration, or status reporting."
---

# Linear

Use Linear as the Script Kit GPUI task ledger. Prefer Linear MCP tools when they are available in the active session; otherwise use a fresh Codex child process after confirming Linear MCP is configured.

## Quick Start

1. Identify the target team/project/issue keys from the user prompt.
2. Read existing Linear records before writing.
3. Deduplicate by exact issue title or issue key before creating.
4. Preserve parent/subtask relationships, descriptions, team, and project unless the user asks to change them.
5. Return issue keys and URLs, plus any failed keys or tool limitations.

For current Script Kit project constants, read `references/script-kit.md`.

## Linear MCP

Use direct Linear MCP tools when present, commonly:

- `list_teams`
- `list_projects`
- `get_project`
- `save_project`
- `list_issues`
- `get_issue`
- `save_issue`
- `save_comment`
- `create_issue_label`

For writes, use `save_issue` or `save_project`. For creating subtasks, set the parent relationship when the tool supports it. If parent fields are unavailable, create a normal issue in the same project and put `Parent: <key> <url>` as the first description line.

## Fresh Session Fallback

This session may not hot-load newly installed MCP servers. If Linear tools are missing but `codex mcp list` shows `linear` enabled with OAuth, run a child Codex process for Linear-only work.

Use this pattern for read-only probes:

```bash
codex exec --cd "$PWD" --sandbox read-only \
  --output-last-message /tmp/linear-result.txt \
  'Use the Linear MCP server only. Do not edit files or run shell commands. <task>'
```

Use this pattern only when the user clearly requested Linear writes:

```bash
codex exec --cd "$PWD" --dangerously-bypass-approvals-and-sandbox \
  --output-last-message /tmp/linear-result.txt \
  'Use the Linear MCP server only. Do not edit local files or run shell commands. <task>'
```

Constrain child prompts tightly: name the team/project, list exact issue keys or titles, require dedupe, and ask for only keys/URLs/failures in the final output.

## Task Tracking Workflow

When asked to track work in Linear:

1. Read the project and parent issues first.
2. Decide whether the request means creating a project, moving issues into a project, creating parent issues, creating subtasks, updating status/labels/comments, or summarizing current work.
3. For breakdowns, create 3-7 child issues per parent unless the user asks for different granularity.
4. Make child titles stable and searchable, usually `<PARENT-KEY>: <task>`.
5. Put acceptance criteria in the description.
6. Include source context and verification expectations when tasks come from repo work.

## Script Kit Verification

For Script Kit GPUI implementation tasks, include these expectations in Linear descriptions or comments when relevant:

- Prefer state-first runtime proof for UI behavior.

## Status Updates

When reporting back, lead with what changed: project URL, created/updated/moved issue counts, grouped issue keys and URLs, failures or limitations, and whether parent/subtask links were preserved.

Do not paste long issue descriptions unless the user asks.

## Safety

Do not create bulk issues without a concrete list or a dry-run plan. Do not delete or archive Linear records unless the user explicitly asks. If there is ambiguity between Linear workspace/team/project terminology, explain the mapping briefly and choose the least destructive object.
