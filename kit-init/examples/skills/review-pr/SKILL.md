---
name: review-pr
description: Review a git diff or checked-out change for correctness bugs, regressions, missing tests, and rollout risk. Use when the user wants findings rather than implementation.
---

# Review PR

Use this skill when the user asks for a code review, PR review, or a risk pass on a diff.

## Workflow

1. Inspect the actual change scope first with git commands before forming conclusions.
2. Read the changed files carefully, starting with correctness-critical paths such as persistence, async flows, auth, config, protocol, and user-visible behavior.
3. Look for concrete bugs, regressions, missing validation, broken assumptions, and missing tests.
4. For each issue, explain the failure mode, why it matters, and the smallest plausible fix.
5. If nothing serious stands out, say that explicitly and call out any residual test gaps.

## Output

- Findings first, ordered by severity
- Then open questions or assumptions
- Then a short summary only if it adds value

## Avoid

- Do not spend most of the answer summarizing the diff
- Do not invent issues without a plausible execution path
- Do not bury the highest-risk finding under style notes
